//! Drawing the world: lights, chunks, objects, characters and effects.

use std::f32::consts::PI;

use glam::{Mat4, Vec2, Vec3};

use super::items::{Item, Stack};
use super::world::{Floor, Obj, World};
use crate::assets::Assets;
use crate::assets::models::{ARM_L, ARM_R, BODY, HEAD, Humanoid, LEG_L, LEG_R};
use crate::palette::*;
use crate::render::{DrawOpts, Light, Mesh, Mode, PointLight, Renderer, TexId, UvRect};
use crate::util::hash2;

/// Lighting and mood for a frame.
#[derive(Clone, Copy, Debug)]
pub struct Env {
    pub ambient: f32,
    pub warmth: f32,
    pub clear: u8,
    /// Seconds, for animation.
    pub time: f32,
    /// 0 = day, 1 = full night (lamps and glowing things switch on).
    pub night: f32,
}

pub fn full_uv(a: &Assets, id: crate::render::TexId) -> UvRect {
    let t = a.tex(id);
    UvRect::new(0.0, 0.0, t.w as f32, t.h as f32)
}

/// Gathers lights from objects around the visible area.
pub fn object_lights(w: &World, rect: (i32, i32, i32, i32), env: &Env, out: &mut Vec<PointLight>) {
    let (x0, z0, x1, z1) = rect;
    for z in (z0 - 6).max(0)..(z1 + 6).min(w.h) {
        for x in (x0 - 6).max(0)..(x1 + 6).min(w.w) {
            let Some(o) = w.obj(x, z) else { continue };
            let lit = match o {
                Obj::Lamp
                | Obj::House
                | Obj::EnchantTable
                | Obj::StreetLamp { .. }
                | Obj::Building { .. } => env.night > 0.2,
                Obj::Crop { crop, days, .. } => {
                    crop.def().glow && *days >= crop.def().days && env.night > 0.2
                }
                _ => true,
            };
            if !lit {
                continue;
            }
            let base = Vec3::new(x as f32 + 0.5, 0.0, z as f32 + 0.5);
            if let Some((hgt, radius, power, warmth)) = o.light() {
                let flicker = if matches!(o, Obj::Torch | Obj::Campfire) {
                    1.0 + ((env.time * 9.0 + (x * 7 + z * 13) as f32).sin() * 0.06)
                } else {
                    1.0
                };
                out.push(PointLight {
                    pos: base + Vec3::Y * hgt,
                    radius: radius * flicker,
                    power: power * flicker,
                    warmth,
                });
            } else if matches!(o, Obj::House) {
                // Warm windows at night.
                out.push(PointLight {
                    pos: base + Vec3::new(1.0, 0.8, 1.2),
                    radius: 5.0,
                    power: 0.75 * env.night,
                    warmth: 7.5,
                });
            } else if let Obj::Building { id } = o {
                let b = &super::town::BUILDINGS[*id as usize];
                out.push(PointLight {
                    pos: base + Vec3::new(b.door as f32, 0.9, 1.3),
                    radius: 4.5 + b.w as f32 * 0.4,
                    power: 0.7 * env.night,
                    warmth: 7.0,
                });
            } else if matches!(o, Obj::Crop { .. }) {
                out.push(PointLight {
                    pos: base + Vec3::Y * 0.3,
                    radius: 2.8,
                    power: 0.5,
                    warmth: 2.0,
                });
            }
        }
    }
    if w.biome == 3 && matches!(w.area, super::world::Area::Hollow { .. }) {
        for z in z0.max(0)..z1.min(w.h) {
            for x in x0.max(0)..x1.min(w.w) {
                if w.floor(x, z) == Floor::Lava && (x + z) % 2 == 0 {
                    out.push(PointLight {
                        pos: Vec3::new(x as f32 + 0.5, 0.2, z as f32 + 0.5),
                        radius: 3.0,
                        power: 0.5,
                        warmth: 8.0,
                    });
                }
            }
        }
    }
}

/// Draws terrain and objects. Characters are drawn by the caller afterwards.
pub fn draw_world(r: &mut Renderer, a: &Assets, w: &mut World, env: &Env, lights: &[PointLight]) {
    let rect = r.cam.visible_tiles(2.5);
    w.update_meshes(a, rect);
    let mut all = lights.to_vec();
    object_lights(w, rect, env, &mut all);
    r.grid
        .build(rect, env.ambient, env.warmth, &all, &|x, z| w.opaque(x, z));
    r.fb.clear(env.clear);
    let frame = ((env.time * 3.0) as usize) % 4;
    r.remap.clear();
    r.remap.push((a.water[0], a.water[frame]));
    r.remap
        .push((a.lava[0], a.lava[((env.time * 2.0) as usize) % 2]));
    let opts = DrawOpts::default();
    for chunk in w.visible_chunks(rect) {
        r.mesh(&a.bank, chunk, &Mat4::IDENTITY, &opts);
    }
    let (x0, z0, x1, z1) = rect;
    for z in z0.max(0)..=z1.min(w.h - 1) {
        for x in x0.max(0)..=x1.min(w.w - 1) {
            if let Some(o) = w.obj(x, z) {
                draw_object(r, a, w, x, z, o, env);
            }
        }
    }
}

fn small_rot(x: i32, z: i32) -> Mat4 {
    Mat4::from_rotation_y((hash2(x, z, 3) % 628) as f32 / 100.0)
}

pub fn draw_object(r: &mut Renderer, a: &Assets, w: &World, x: i32, z: i32, o: &Obj, env: &Env) {
    let base = Vec3::new(x as f32 + 0.5, 0.0, z as f32 + 0.5);
    let at = Mat4::from_translation(base);
    let lit = DrawOpts::at(base);
    let p = &a.props;
    let bb = |r: &mut Renderer, name: &str, size: f32, o: &DrawOpts| {
        let id = a.icon(name);
        r.billboard(
            a.tex(id),
            full_uv(a, id),
            base + Vec3::new(0.0, 0.0, 0.1),
            Vec2::splat(size),
            o,
        );
    };
    match o {
        Obj::Tree { var, .. } => {
            r.shadow(a.tex(a.disk), base, 0.75);
            let sway = (env.time * 1.3 + (x + z) as f32).sin() * 0.02;
            let m = at * small_rot(x, z) * Mat4::from_rotation_z(sway);
            r.mesh(&a.bank, &p.trees[*var as usize % p.trees.len()], &m, &lit);
        }
        Obj::Pine { var, .. } => {
            r.shadow(a.tex(a.disk), base, 0.7);
            r.mesh(
                &a.bank,
                &p.pines[*var as usize % p.pines.len()],
                &(at * small_rot(x, z)),
                &lit,
            );
        }
        Obj::Stump { .. } => r.mesh(&a.bank, &p.stump, &(at * small_rot(x, z)), &lit),
        Obj::Log { .. } => r.mesh(&a.bank, &p.log, &(at * Mat4::from_rotation_y(0.3)), &lit),
        Obj::Rock { var, .. } => r.mesh(
            &a.bank,
            &p.rocks[*var as usize % p.rocks.len()],
            &(at * small_rot(x, z)),
            &lit,
        ),
        Obj::Boulder { .. } => r.mesh(&a.bank, &p.boulder, &(at * small_rot(x, z)), &lit),
        Obj::Weed { var } => bb(r, if *var == 0 { "tuft" } else { "tuft_dry" }, 0.9, &lit),
        Obj::Flower { var } => {
            let n = [
                "blossom_pink",
                "blossom_white",
                "blossom_blue",
                "blossom_gold",
            ][*var as usize % 4];
            bb(r, n, 0.9, &lit);
        }
        Obj::Crystal { var, .. } => {
            let m = at * small_rot(x, z);
            r.mesh(
                &a.bank,
                &p.crystal[*var as usize % p.crystal.len()],
                &m,
                &DrawOpts::default().with_mode(Mode::Unlit),
            );
        }
        Obj::Stalagmite { var } => r.mesh(
            &a.bank,
            &p.stalagmite[*var as usize % p.stalagmite.len()],
            &(at * small_rot(x, z)),
            &lit,
        ),
        Obj::Mushroom { var } => r.mesh(
            &a.bank,
            &p.mushroom[*var as usize % p.mushroom.len()],
            &(at * small_rot(x, z)),
            &lit.with_glow(0.9),
        ),
        Obj::Bones => r.mesh(&a.bank, &p.bones, &(at * small_rot(x, z)), &lit),
        Obj::Pot { .. } => {
            r.shadow(a.tex(a.disk), base, 0.3);
            r.mesh(&a.bank, &p.pot, &(at * small_rot(x, z)), &lit)
        }
        Obj::Crate { .. } => r.mesh(&a.bank, &p.crate_, &(at * Mat4::from_rotation_y(0.1)), &lit),
        Obj::LootChest { opened } => {
            let mesh = if *opened {
                &p.chest_open
            } else {
                &p.loot_chest
            };
            r.mesh(&a.bank, mesh, &at, &lit);
            if !*opened {
                let s = (env.time * 4.0 + x as f32).sin() * 0.5 + 0.5;
                r.point(base + Vec3::new(0.2, 0.6 + s * 0.2, 0.2), 1, CREAM);
            }
        }
        Obj::StairsDown => {
            r.mesh(&a.bank, &p.stairs, &at, &lit);
        }
        Obj::Waystone => {
            r.shadow(a.tex(a.disk), base, 0.55);
            r.mesh(&a.bank, &p.waystone, &at, &lit);
            let pulse = 0.5 + 0.5 * (env.time * 2.0).sin();
            r.mesh(
                &a.bank,
                &p.waystone_rune,
                &at,
                &DrawOpts::default().with_mode(Mode::Unlit),
            );
            r.point(base + Vec3::new(0.0, 1.6 + pulse * 0.2, 0.0), 2, MINT);
        }
        Obj::Campfire => {
            r.mesh(&a.bank, &p.campfire, &at, &lit);
            flames(r, a, base + Vec3::new(0.0, 0.05, 0.0), 0.7, env.time, x);
        }
        Obj::Torch => {
            r.mesh(&a.bank, &p.torch_stick, &at, &lit);
            flames(r, a, base + Vec3::new(0.0, 0.7, 0.0), 0.3, env.time, x + z);
        }
        Obj::House => {
            r.mesh(&a.bank, &p.house, &at, &DrawOpts::default());
            let mode = if env.night > 0.3 {
                Mode::Unlit
            } else {
                Mode::Lit
            };
            r.mesh(
                &a.bank,
                &p.house_windows,
                &at,
                &DrawOpts::default().with_mode(mode),
            );
        }
        Obj::Part { .. } => {}
        Obj::Bin => r.mesh(&a.bank, &p.bin, &at, &lit),
        Obj::Hollow => r.mesh(&a.bank, &p.hollow, &at, &DrawOpts::default()),
        Obj::Stall => {
            let m = Mat4::from_translation(base + Vec3::new(0.5, 0.0, 0.0));
            r.mesh(&a.bank, &p.stall, &m, &DrawOpts::default());
            let bob = (env.time * 2.2).sin().abs() * 0.04;
            let mole = Mat4::from_translation(base + Vec3::new(0.5, bob, -0.25));
            r.mesh(
                &a.bank,
                &a.critters.mole,
                &mole,
                &DrawOpts::at(base).with_tag(1),
            );
        }
        Obj::Sign { .. } => r.mesh(&a.bank, &p.sign, &at, &lit),
        Obj::Chest { .. } => r.mesh(&a.bank, &p.chest, &at, &lit),
        Obj::Lamp => {
            r.mesh(&a.bank, &p.lamp, &at, &lit);
            let cap = Mat4::from_translation(base + Vec3::new(0.0, 0.82, 0.0))
                * Mat4::from_scale(Vec3::splat(1.3));
            let o = if env.night > 0.2 {
                DrawOpts::default().with_mode(Mode::Unlit)
            } else {
                lit
            };
            r.mesh(&a.bank, &p.mushroom[0], &cap, &o);
        }
        Obj::Fence => {
            r.mesh(&a.bank, &p.fence_post, &at, &lit);
            if matches!(w.obj(x + 1, z), Some(Obj::Fence)) {
                r.mesh(&a.bank, &p.fence_rail_e, &at, &lit);
            }
            if matches!(w.obj(x, z + 1), Some(Obj::Fence)) {
                r.mesh(&a.bank, &p.fence_rail_s, &at, &lit);
            }
        }
        Obj::Sprinkler { tier } => r.mesh(&a.bank, &p.sprinkler[*tier as usize % 3], &at, &lit),
        Obj::Workbench => r.mesh(&a.bank, &p.workbench, &at, &lit),
        Obj::FlowerPot { .. } => {
            r.mesh(&a.bank, &p.flower_pot, &at, &lit);
            let id = a.icon("blossom_pink");
            r.billboard(
                a.tex(id),
                full_uv(a, id),
                base + Vec3::new(0.0, 0.05, 0.05),
                Vec2::splat(0.8),
                &lit,
            );
        }
        Obj::Bench => r.mesh(&a.bank, &p.bench, &at, &lit),
        Obj::EnchantTable => {
            r.shadow(a.tex(a.disk), base, 0.45);
            r.mesh(&a.bank, &p.enchant_table, &at, &lit);
            // A little book floats above it, turning slowly, with runes circling round.
            let bob = (env.time * 2.0 + x as f32).sin() * 0.04;
            let book = Mat4::from_translation(base + Vec3::new(0.0, 0.86 + bob, 0.0))
                * Mat4::from_rotation_y(env.time * 0.6)
                * Mat4::from_rotation_x(-0.35);
            r.mesh(
                &a.bank,
                &p.enchant_book,
                &book,
                &lit.with_glow(0.9).with_tag(3),
            );
            for k in 0..3 {
                let ang = env.time * 1.4 + k as f32 * 2.094;
                let q = base
                    + Vec3::new(
                        ang.cos() * 0.42,
                        0.7 + (env.time * 2.3 + k as f32).sin() * 0.12,
                        ang.sin() * 0.42,
                    );
                r.point(q, 1, [LAVENDER, MINT, BLUSH][k]);
            }
        }
        Obj::BusStop => {
            r.mesh(&a.bank, &a.town.shelter, &at, &DrawOpts::default());
        }
        Obj::Building { id } => {
            let (body, windows) = &a.town.buildings[*id as usize];
            r.mesh(&a.bank, body, &at, &DrawOpts::default());
            let mode = if env.night > 0.3 {
                Mode::Unlit
            } else {
                Mode::Lit
            };
            r.mesh(&a.bank, windows, &at, &DrawOpts::default().with_mode(mode));
        }
        Obj::Fountain { flowing } => {
            r.mesh(&a.bank, &a.town.fountain, &at, &DrawOpts::default());
            if *flowing {
                r.mesh(&a.bank, &a.town.fountain_water, &at, &DrawOpts::default());
                // Water arcing from the top bowl and splashing below.
                for i in 0..12 {
                    let k = (env.time * 0.9 + i as f32 / 12.0).fract();
                    let ang = i as f32 * 0.5236;
                    let rad = 0.35 + k * 0.6;
                    let y = 1.3 + k * 0.25 - k * k * 1.2;
                    let p = base + Vec3::new(ang.cos() * rad, y, ang.sin() * rad);
                    r.point(p, 1, if i % 3 == 0 { WHITE } else { SKY });
                }
                let s = (env.time * 5.0).sin() > 0.0;
                r.point(
                    base + Vec3::new(0.0, 1.62, 0.0),
                    if s { 2 } else { 1 },
                    WHITE,
                );
            }
        }
        Obj::StreetLamp { lit } => {
            r.mesh(&a.bank, &a.town.street_lamp, &at, &lit_opts(base));
            let glow = *lit && env.night > 0.2;
            let o = if glow {
                DrawOpts::default().with_mode(Mode::Unlit)
            } else {
                lit_opts(base)
            };
            r.mesh(&a.bank, &a.town.lamp_glass, &at, &o);
            if glow && (env.time * 2.0 + x as f32).sin() > 0.95 {
                r.point(base + Vec3::new(0.0, 1.95, 0.0), 1, CREAM);
            }
        }
        Obj::Board => r.mesh(&a.bank, &a.town.board, &at, &lit),
        Obj::Planter { var } => {
            let m = &a.town.planters[*var as usize % a.town.planters.len()];
            r.mesh(&a.bank, m, &at, &lit);
        }
        Obj::Bush { var } => {
            r.shadow(a.tex(a.disk), base, 0.45);
            let m = &a.town.bushes[*var as usize % a.town.bushes.len()];
            r.mesh(&a.bank, m, &(at * small_rot(x, z)), &lit);
        }
        Obj::Well => r.mesh(&a.bank, &a.town.well, &at, &lit),
        Obj::Stand { var } => {
            let m = &a.town.stands[*var as usize % a.town.stands.len()];
            r.mesh(&a.bank, m, &at, &lit);
        }
        Obj::Barrel => r.mesh(&a.bank, &a.town.barrel, &at, &lit),
        Obj::Counter => r.mesh(&a.bank, &a.town.counter, &at, &lit),
        Obj::Shelf { var } => {
            r.mesh(&a.bank, &a.town.shelf, &at, &lit);
            let goods = shelf_goods(*var);
            for (row, y) in [0.12f32, 0.62, 1.12].into_iter().enumerate() {
                for k in 0..3 {
                    let pick = hash2(x * 3 + k, z * 5 + row as i32, 17) as usize % goods.len();
                    let id = a.icon(goods[pick]);
                    r.billboard(
                        a.tex(id),
                        full_uv(a, id),
                        base + Vec3::new(-0.26 + k as f32 * 0.26, y, -0.3),
                        Vec2::splat(0.3),
                        &lit,
                    );
                }
            }
        }
        Obj::Rack { var } => {
            r.mesh(&a.bank, &a.town.rack, &at, &lit);
            let set: [&str; 3] = match var % 3 {
                0 => ["twig_sword", "carrot_blade", "mighty_leek"],
                1 => ["bubble_wand", "crystal_wand", "star_wand"],
                _ => ["oak_staff", "mossy_staff", "crystal_staff"],
            };
            for (k, icon) in set.iter().enumerate() {
                if let Some(mesh) = a.held_mesh(icon) {
                    let m = at
                        * Mat4::from_translation(Vec3::new(-0.26 + k as f32 * 0.26, 0.28, -0.34))
                        * Mat4::from_rotation_z(PI);
                    r.mesh(&a.bank, mesh, &m, &lit);
                }
            }
        }
        Obj::Mannequin { var } => {
            r.shadow(a.tex(a.disk), base, 0.3);
            r.mesh(&a.bank, &a.town.dummy_base, &at, &lit);
            draw_mannequin(r, a, base + Vec3::Y * 0.2, *var);
        }
        Obj::Table { var } => {
            let m = if matches!(var, 1 | 9) {
                &a.town.tables[1]
            } else {
                &a.town.tables[0]
            };
            r.mesh(&a.bank, m, &at, &lit);
        }
        Obj::Stool => r.mesh(&a.bank, &a.town.stool, &at, &lit),
        Obj::Hearth => {
            r.mesh(&a.bank, &a.town.hearth, &at, &lit);
            flames(
                r,
                a,
                base + Vec3::new(0.0, 0.08, -0.25),
                0.55,
                env.time,
                x * 7 + z,
            );
        }
        Obj::Fixture { var } => {
            let m = &a.town.fixtures[*var as usize % a.town.fixtures.len()];
            r.mesh(&a.bank, m, &at, &lit);
            match var {
                0 => {
                    // The oven's glow.
                    if (env.time * 3.0).sin() > 0.3 {
                        r.point(base + Vec3::new(0.0, 0.42, 0.4), 1, GOLD);
                    }
                }
                2 => {
                    // Bubbles in the cauldron.
                    for i in 0..3 {
                        let k = (env.time * 0.8 + i as f32 * 0.33).fract();
                        let p = base + Vec3::new((i as f32 - 1.0) * 0.1, 0.52 + k * 0.4, 0.0);
                        if k < 0.8 {
                            r.point(p, 1, if i == 1 { MINT } else { AQUA });
                        }
                    }
                }
                _ => {}
            }
        }
        Obj::Crop { crop, days, .. } => {
            let def = crop.def();
            let name = match crop.stage(*days) {
                0 => "seedling",
                1 => "sprout",
                2 => def.young,
                _ => def.produce.def().icon,
            };
            let ready = crop.stage(*days) == 3;
            let o = if ready && def.glow && env.night > 0.2 {
                lit.with_glow(1.1)
            } else {
                lit
            };
            let size = if ready { 0.78 } else { 0.9 };
            bb(r, name, size, &o);
            if ready {
                let s = (env.time * 3.0 + (x * 3 + z) as f32).sin();
                if s > 0.6 {
                    r.point(base + Vec3::new(0.25, 0.75, 0.15), 1, WHITE);
                }
            }
        }
    }
}

/// Lit evenly from the object's position.
fn lit_opts(base: Vec3) -> DrawOpts {
    DrawOpts::at(base)
}

/// What a shop's shelves hold, by the shop's goods.
pub fn shelf_goods(var: u8) -> &'static [&'static str] {
    match var {
        0 => &[
            "copper_helm",
            "iron_helm",
            "leather_boots",
            "iron_boots",
            "copper_shield",
            "leaf_shield",
            "straw_hat",
            "bunny_hood",
            "frog_hood",
        ],
        1 => &[
            "twig_sword",
            "carrot_blade",
            "mighty_leek",
            "bubble_wand",
            "glowcap_wand",
            "oak_staff",
            "sunflower_staff",
            "crystal_wand",
        ],
        2 => &[
            "turnip_seeds",
            "carrot_seeds",
            "tomato_seeds",
            "strawberry_seeds",
            "corn_seeds",
            "sunflower_seeds",
            "rose_seeds",
            "pumpkin_seeds",
            "melon_seeds",
        ],
        3 => &[
            "weapon_scroll",
            "armor_scroll",
            "tool_scroll",
            "mana_tonic",
            "wish_star",
        ],
        4 => &[
            "copper_hoe",
            "iron_hoe",
            "duck_can",
            "teapot_can",
            "copper_sickle",
            "beaver_axe",
            "mole_pick",
            "sprinkler",
        ],
        5 => &[
            "ruby",
            "sapphire",
            "emerald",
            "topaz",
            "amethyst",
            "moonstone",
            "moon_pearl",
        ],
        7 => &["lamp", "flower_pot", "bench", "chest", "fence", "torch"],
        8 => &[
            "fresh_bread",
            "blueberry_muffin",
            "carrot_cake",
            "shortcake",
            "berry_tart",
            "pumpkin_pie",
            "sunflower_cookies",
            "garlic_bread",
        ],
        9 => &[
            "veggie_stew",
            "tomato_soup",
            "corn_chowder",
            "ember_curry",
            "lava_lemonade",
            "mint_tea",
            "popcorn",
        ],
        _ => &["request", "music_box", "lost_button", "ancient_coin"],
    }
}

/// Outfits the shop dummies show off.
pub const DUMMY_OUTFITS: [[Item; 5]; 6] = [
    [
        Item::CopperHelm,
        Item::CopperMail,
        Item::CopperGreaves,
        Item::CopperSabatons,
        Item::CopperShield,
    ],
    [
        Item::FrogHood,
        Item::FrogRaincoat,
        Item::PumpkinBloomers,
        Item::FrogSlippers,
        Item::LeafShield,
    ],
    [
        Item::IronHelm,
        Item::IronPlate,
        Item::IronGreaves,
        Item::IronBoots,
        Item::IronShield,
    ],
    [
        Item::WizardHat,
        Item::MageRobe,
        Item::StarryLeggings,
        Item::FeatherBoots,
        Item::MushroomShield,
    ],
    [
        Item::CrystalCirclet,
        Item::CrystalMail,
        Item::CrystalGreaves,
        Item::CrystalBoots,
        Item::CrystalAegis,
    ],
    [
        Item::BunnyHood,
        Item::WoollyPoncho,
        Item::LeatherLeggings,
        Item::BunnySlippers,
        Item::TurtleShell,
    ],
];

/// A wooden dummy wearing one of the shop's outfits.
pub fn draw_mannequin(r: &mut Renderer, a: &Assets, pos: Vec3, var: u8) {
    let mut equip: [Option<Stack>; 5] = [None; 5];
    for it in DUMMY_OUTFITS[var as usize % DUMMY_OUTFITS.len()] {
        if let Some(slot) = it.class().and_then(|c| c.slot()) {
            equip[slot as usize] = Some(Stack::new(it, 1));
        }
    }
    let mut dressed = super::scene::dress(a, &equip, None);
    let wood = a.town.dummy_wood;
    let t = a.hero.tex;
    dressed.remap.push((t.head, wood));
    for skin in [t.body, t.arm, t.leg] {
        if !dressed.remap.iter().any(|(from, _)| *from == skin) {
            dressed.remap.push((skin, wood));
        }
    }
    let pose = Pose::default();
    let o = DrawOpts::at(pos).with_tag(1);
    draw_humanoid(r, a, &a.hero, pos, 0.0, &pose, &o, &dressed.outfit(None));
}

/// A few flickering flame sprites.
pub fn flames(r: &mut Renderer, a: &Assets, pos: Vec3, size: f32, t: f32, seed: i32) {
    let f = ((t * 10.0) as i32 + seed).rem_euclid(4) as usize;
    let tex = a.tex(a.flame[f]);
    let o = DrawOpts {
        mode: Mode::Unlit,
        light: Light::Fixed(1.0, NEUTRAL),
        ..Default::default()
    };
    r.billboard(
        tex,
        UvRect::new(0.0, 0.0, 8.0, 8.0),
        pos,
        Vec2::splat(size),
        &o,
    );
    let spark_y = (t * 1.7 + seed as f32).fract();
    r.point(
        pos + Vec3::new(
            (seed as f32 + t * 3.0).sin() * 0.1,
            size + spark_y * 0.5,
            0.0,
        ),
        1,
        GOLD,
    );
}

/// Pose parameters for a humanoid.
#[derive(Clone, Copy, Default)]
pub struct Pose {
    /// Walk cycle phase (radians) and how much of it to show (0..1).
    pub walk: f32,
    pub stride: f32,
    /// Tool/attack progress 0..1, with the kind of swing.
    pub swing: Option<(f32, Swing)>,
    pub bob: f32,
    /// Extra squash on hit.
    pub squash: f32,
}

#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum Swing {
    #[default]
    Slash,
    Chop,
    Pour,
    Use,
    /// A wand thrust forward.
    Cast,
    /// A staff raised high and brought down.
    Raise,
}

/// What a humanoid is wearing and holding.
#[derive(Clone, Copy, Default)]
pub struct Outfit<'a> {
    /// Attached to the right hand, pointing down the arm.
    pub held: Option<&'a Mesh>,
    pub hat: Option<&'a Mesh>,
    /// The hero's sprout, drawn on top of the head (and poking out of any hat).
    pub sprout: Option<&'a Mesh>,
    /// Drawn on each foot.
    pub boot: Option<&'a Mesh>,
    /// Strapped to the left arm.
    pub shield: Option<&'a Mesh>,
    /// Texture swaps for clothes (body, arms, legs).
    pub remap: &'a [(TexId, TexId)],
}

/// Draws a humanoid at `pos` facing `yaw` (0 = +z) in an outfit.
#[allow(clippy::too_many_arguments)]
pub fn draw_humanoid(
    r: &mut Renderer,
    a: &Assets,
    h: &Humanoid,
    pos: Vec3,
    yaw: f32,
    pose: &Pose,
    o: &DrawOpts,
    fit: &Outfit,
) {
    let before = r.remap.len();
    r.remap.extend_from_slice(fit.remap);
    draw_body(r, a, h, pos, yaw, pose, o, fit);
    r.remap.truncate(before);
}

#[allow(clippy::too_many_arguments)]
fn draw_body(
    r: &mut Renderer,
    a: &Assets,
    h: &Humanoid,
    pos: Vec3,
    yaw: f32,
    pose: &Pose,
    o: &DrawOpts,
    fit: &Outfit,
) {
    let held = fit.held;
    let root = Mat4::from_translation(pos) * Mat4::from_rotation_y(yaw);
    let sw = pose.walk.sin() * pose.stride;
    let squash = 1.0 - pose.squash * 0.25;
    let widen = 1.0 + pose.squash * 0.2;
    let scale = Mat4::from_scale(Vec3::new(widen, squash, widen));
    let root = root * scale;
    let bob = pose.bob + pose.walk.cos().abs() * 0.04 * pose.stride;
    // Legs.
    for (i, s) in [(LEG_L, -1.0f32), (LEG_R, 1.0)] {
        let m = root
            * Mat4::from_translation(Vec3::new(s * h.hip_x, h.hip, 0.0))
            * Mat4::from_rotation_x(sw * s * 0.7);
        r.mesh(&a.bank, &h.parts[i], &m, o);
        if let Some(boot) = fit.boot {
            let foot = m * Mat4::from_translation(Vec3::new(0.0, -h.hip, 0.0));
            let foot = if s < 0.0 {
                foot * Mat4::from_scale(Vec3::new(-1.0, 1.0, 1.0))
            } else {
                foot
            };
            let bo = if s < 0.0 { o.two_sided() } else { *o };
            r.mesh(&a.bank, boot, &foot, &bo);
        }
    }
    let body = root * Mat4::from_translation(Vec3::new(0.0, h.hip - 0.02 + bob, 0.0));
    r.mesh(&a.bank, &h.parts[BODY], &body, o);
    // Arms: the right arm follows the swing.
    let guard = fit.shield.is_some() && matches!(pose.swing, None | Some((_, Swing::Slash)));
    let left = root
        * Mat4::from_translation(Vec3::new(-h.shoulder_x, h.shoulder + bob, 0.0))
        * Mat4::from_rotation_x(if guard { -0.5 - sw * 0.2 } else { -sw * 0.6 })
        * Mat4::from_rotation_z(-0.12);
    r.mesh(&a.bank, &h.parts[ARM_L], &left, o);
    if let Some(shield) = fit.shield {
        // Worn on the forearm, facing outwards and a little forwards.
        let m = left
            * Mat4::from_translation(Vec3::new(-0.07, -0.16, 0.02))
            * Mat4::from_rotation_y(-1.2);
        r.mesh(&a.bank, shield, &m, o);
    }
    let (rx, rz, twist) = match pose.swing {
        Some((t, Swing::Slash)) => {
            let k = ease_out(t);
            (-1.4 + (k * 2.6 - 1.2) * 0.2, 0.0, 1.3 - k * 2.6)
        }
        Some((t, Swing::Chop)) => {
            let k = ease_out(t);
            (-2.7 + k * 3.3, 0.0, 0.0)
        }
        Some((t, Swing::Pour)) => (-1.2, 0.0, (t * PI).sin() * 0.3),
        Some((t, Swing::Use)) => (-1.3 * (t * PI).sin(), 0.0, 0.0),
        Some((t, Swing::Cast)) => {
            let k = (t * PI).sin();
            (-1.35 - k * 0.25, 0.0, 0.0)
        }
        Some((t, Swing::Raise)) => {
            let k = ease_out(t);
            (-3.0 + k * 1.9, 0.0, 0.0)
        }
        None if held.is_some() => (-0.95 + sw * 0.15, 0.1, 0.0),
        None => (sw * 0.6, 0.12, 0.0),
    };
    let right = root
        * Mat4::from_translation(Vec3::new(h.shoulder_x, h.shoulder + bob, 0.0))
        * Mat4::from_rotation_y(twist)
        * Mat4::from_rotation_x(rx)
        * Mat4::from_rotation_z(rz);
    r.mesh(&a.bank, &h.parts[ARM_R], &right, o);
    if let Some(tool) = held {
        let hand = right * Mat4::from_translation(Vec3::new(0.0, -0.22, 0.02));
        let orient = match pose.swing {
            Some((_, Swing::Pour)) => Mat4::from_rotation_x(0.9),
            _ => Mat4::IDENTITY,
        };
        r.mesh(&a.bank, tool, &(hand * orient), o);
    }
    let head = root
        * Mat4::from_translation(Vec3::new(0.0, h.neck + bob, 0.0))
        * Mat4::from_rotation_x(-0.22)
        * Mat4::from_rotation_z(sw * 0.05);
    r.mesh(&a.bank, &h.parts[HEAD], &head, o);
    let crown = head * Mat4::from_translation(Vec3::new(0.0, 0.46, 0.0));
    if let Some(hat) = fit.hat {
        r.mesh(&a.bank, hat, &crown, o);
    }
    if let Some(sprout) = fit.sprout {
        // The sprout always finds its way out, a little higher with a hat on.
        let lift = if fit.hat.is_some() { 0.1 } else { 0.0 };
        let m = crown * Mat4::from_translation(Vec3::new(0.0, lift, 0.0));
        r.mesh(&a.bank, sprout, &m, o);
    }
}

fn ease_out(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    1.0 - (1.0 - t) * (1.0 - t)
}

/// A translucent slash arc in front of a character.
pub fn slash_arc(r: &mut Renderer, pos: Vec3, yaw: f32, t: f32, reach: f32, color: u8) {
    if t > 0.7 {
        return;
    }
    let k = ease_out(t / 0.7);
    let span = 2.4f32;
    let start = -span * 0.5;
    let end = start + span * k;
    let steps = 10;
    for i in 0..=steps {
        let ang = start + (end - start) * i as f32 / steps as f32;
        let a = yaw + ang;
        for rr in [reach * 0.55, reach * 0.8, reach] {
            let p = pos + Vec3::new(a.sin() * rr, 0.4, a.cos() * rr);
            r.point(
                p,
                if rr >= reach { 2 } else { 1 },
                if rr >= reach { WHITE } else { color },
            );
        }
    }
}
