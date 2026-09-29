//! Drawing the world: lights, chunks, objects, characters and effects.

use std::f32::consts::{FRAC_PI_2, PI};

use glam::{Mat4, Vec2, Vec3};

use super::glowcave::{EAST, NORTH, cap_colour, cap_size, shelf_side};
use super::home::{self, Furn};
use super::items::{Item, Stack};
use super::labyrinth::FALLEN;
use super::play::Season;
use super::world::{Floor, Obj, POOL_GLOW, SEWER_GLOW, WATER_Y, World};
use crate::assets::Assets;
use crate::assets::glowcave_art::{GLOW, HALO};
use crate::assets::models::{ARM_L, ARM_R, BODY, HEAD, Humanoid, LEG_L, LEG_R};
use crate::palette::*;
use crate::render::{DrawOpts, Light, Mesh, Mode, PointLight, Renderer, TexId, UvRect, Warp};
use crate::util::hash2;

/// How strongly ambient occlusion darkens creases and contact points.
pub const AO_STRENGTH: f32 = 1.5;

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
    /// How hard the breeze blows (0 indoors and underground).
    pub wind: f32,
    /// Where the hero is wading through the grass.
    pub push: Vec2,
    /// Seconds left of a spin given to the house globe.
    pub spin: f32,
    /// Outdoors, the season: blossom, turned leaves or snow (None indoors and below).
    pub season: Option<Season>,
}

impl Env {
    /// How far a plant `flex` bends at a tile in the breeze right now.
    pub fn lean(&self, x: f32, z: f32, flex: f32) -> Vec2 {
        if self.wind <= 0.0 {
            return Vec2::ZERO;
        }
        crate::render::wind(x, z, self.time) * (flex * self.wind)
    }

    /// Bends a plant `h` tall standing at `base` with the breeze.
    pub fn sway(&self, base: Vec3, h: f32, flex: f32) -> Warp {
        if self.wind <= 0.0 {
            return Warp::None;
        }
        Warp::Bend {
            base: base.y,
            h,
            lean: self.lean(base.x, base.z, flex),
        }
    }
}

pub fn full_uv(a: &Assets, id: crate::render::TexId) -> UvRect {
    let t = a.tex(id);
    UvRect::new(0.0, 0.0, t.w as f32, t.h as f32)
}

/// Glints twinkling about an unopened chest, each popping up somewhere new over its front
/// or lid, swelling to a star and fading. A gleaming chest throws a shower of bigger gold
/// ones, a glow, and motes of gold drifting up.
pub fn chest_sparkles(r: &mut Renderer, base: Vec3, time: f32, (x, z): (i32, i32), gleam: bool) {
    let n = if gleam { 6 } else { 2 };
    for k in 0..n {
        let seed = (x * 73 + z * 151 + k * 37) as u32;
        let period = if gleam { 0.8 } else { 1.7 } + (seed % 7) as f32 * 0.12;
        let phase = time / period + (seed % 13) as f32 / 13.0;
        let life = phase.fract();
        // Each glint lives for the first half of its cycle.
        if life > 0.5 {
            continue;
        }
        let k2 = life / 0.5;
        let h = crate::util::hash2(seed as i32, phase.floor() as i32, 17);
        let (u, v) = ((h & 0xFF) as f32 / 255.0, ((h >> 8) & 0xFF) as f32 / 255.0);
        // On the front face or just over the lid, never inside the chest.
        let spot = if v < 0.65 {
            Vec3::new(u * 0.72 - 0.36, 0.08 + v / 0.65 * 0.42, 0.31)
        } else {
            Vec3::new(u * 0.72 - 0.36, 0.52 + (v - 0.65) * 0.9, 0.1)
        };
        let peak = 1.0 - (k2 - 0.5).abs() * 2.0;
        let arm = if gleam {
            1 + (peak * 3.2) as i32
        } else {
            1 + (peak * 1.6) as i32
        };
        let edge = if gleam { GOLD } else { CREAM };
        r.sparkle(base + spot, arm, WHITE, edge);
    }
    if gleam {
        let pulse = (time * 3.0 + x as f32).sin() * 0.5 + 0.5;
        r.halo(base + Vec3::Y * 0.45, 0.8, GOLD, 0.25 + pulse * 0.15);
        for k in 0..4 {
            let t = (time * 0.6 + k as f32 * 0.25).fract();
            let a = k as f32 * 1.7 + time * 0.8;
            let p = base + Vec3::new(a.cos() * 0.3, 0.5 + t * 0.9, 0.15 + a.sin() * 0.2);
            r.point(p, 1, if t < 0.6 { GOLD } else { CREAM });
        }
    }
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
                | Obj::WishTree { .. }
                | Obj::Egg { .. }
                | Obj::Building { .. } => env.night > 0.2,
                Obj::Crop { crop, days, .. } => {
                    crop.def().glow && *days >= crop.def().days && env.night > 0.2
                }
                _ => true,
            };
            if !lit {
                continue;
            }
            let base = match o {
                Obj::Furniture { f, rot, .. } => home::center(*f, *rot, x, z),
                // Stood on four tiles, from the corner they share.
                Obj::GiantShroom { .. } => Vec3::new(x as f32 + 1.0, 0.0, z as f32 + 1.0),
                _ => Vec3::new(x as f32 + 0.5, 0.0, z as f32 + 0.5),
            };
            if let Some((hgt, radius, power, warmth)) = o.light() {
                let flicker =
                    if matches!(o, Obj::Torch | Obj::Campfire | Obj::Brazier | Obj::Sconce) {
                        1.0 + ((env.time * 9.0 + (x * 7 + z * 13) as f32).sin() * 0.06)
                    } else {
                        1.0
                    };
                // The canyon's pale sand is lit bright enough already.
                let dim = if w.canyon { 0.6 } else { 1.0 };
                out.push(PointLight {
                    pos: base + Vec3::Y * hgt,
                    radius: radius * flicker,
                    power: power * flicker * dim,
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
    // The glowcap caves' pools light up the rock round them.
    if w.glowcave {
        for z in z0.max(0)..z1.min(w.h) {
            for x in x0.max(0)..x1.min(w.w) {
                if w.floor(x, z) == Floor::Water && (x + z) % 2 == 0 {
                    out.push(PointLight {
                        pos: Vec3::new(x as f32 + 0.5, 0.1, z as f32 + 0.5),
                        radius: 2.8,
                        power: 0.4,
                        warmth: 1.2,
                    });
                }
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
    if w.sewer {
        r.remap.push((a.sewer.water[0], a.sewer.water[frame]));
    }
    if w.canyon {
        let q = &a.canyon.quicksand;
        r.remap.push((q[0], q[((env.time * 1.5) as usize) % 4]));
    }
    if w.glowcave {
        let water = &a.glowcave.water;
        r.remap
            .extend((0..16).map(|m| (water[0][m], water[frame][m])));
    }
    if let Some(s) = env.season {
        r.remap.extend_from_slice(&a.props.seasons[s as usize]);
    }
    let opts = DrawOpts::default();
    for chunk in w.visible_chunks(rect) {
        r.mesh(&a.bank, chunk, &Mat4::IDENTITY, &opts);
    }
    let glow = DrawOpts {
        glow: if w.glowcave { POOL_GLOW } else { SEWER_GLOW },
        ..opts
    };
    for chunk in w.visible_glow(rect) {
        r.mesh(&a.bank, chunk, &Mat4::IDENTITY, &glow);
    }
    let (x0, z0, x1, z1) = rect;
    for z in z0.max(0)..=z1.min(w.h - 1) {
        for x in x0.max(0)..=x1.min(w.w - 1) {
            if let Some(o) = w.obj(x, z) {
                draw_object(r, a, w, x, z, o, env);
            }
        }
    }
    if w.glowcave {
        pool_shimmer(r, w, rect, env.time);
    }
}

/// Glowing pools shimmer: a soft glow here and there on the water, and motes of light
/// rising off it.
fn pool_shimmer(r: &mut Renderer, w: &World, (x0, z0, x1, z1): (i32, i32, i32, i32), time: f32) {
    for z in z0.max(0)..=z1.min(w.h - 1) {
        for x in x0.max(0)..=x1.min(w.w - 1) {
            if w.floor(x, z) != Floor::Water {
                continue;
            }
            let h = hash2(x, z, 41);
            let at = Vec3::new(x as f32 + 0.5, w.water_y() + 0.02, z as f32 + 0.5);
            if h % 4 == 1 {
                let pulse = (time * 1.1 + (h % 13) as f32).sin() * 0.5 + 0.5;
                r.halo(at, 0.6, AQUA, 0.1 + pulse * 0.06);
            }
            if h % 3 == 0 {
                let k = (time * 0.35 + (h % 100) as f32 / 100.0).fract();
                if k < 0.55 {
                    let sway = (time * 1.7 + (h % 17) as f32).sin() * 0.1;
                    let q = at + Vec3::new(sway, k * 1.1, ((h / 7) % 5) as f32 * 0.08 - 0.16);
                    r.point(q, 1, if k < 0.3 { WHITE } else { MINT });
                }
            }
        }
    }
}

/// A cluster of glowcaps: pale stems, caps shining in their colour with a glow all round,
/// and spores drifting up off the bigger ones.
fn draw_glowcap(r: &mut Renderer, a: &Assets, (x, z): (i32, i32), var: u8, env: &Env) {
    let base = Vec3::new(x as f32 + 0.5, 0.0, z as f32 + 0.5);
    let (c, size) = (cap_colour(var), cap_size(var));
    let m = Mat4::from_translation(base) * small_rot(x, z);
    r.shadow(a.tex(a.disk), base, [0.2, 0.3, 0.38][size]);
    r.mesh(
        &a.bank,
        &a.glowcave.glowcaps[c][size],
        &m,
        &DrawOpts::default().with_mode(Mode::Unlit),
    );
    let pulse = (env.time * 1.3 + (x * 7 + z * 3) as f32 * 0.7).sin() * 0.5 + 0.5;
    let (hy, hr, hs) = [(0.12, 0.36, 0.28), (0.3, 0.52, 0.32), (0.66, 0.72, 0.36)][size];
    r.halo(base + Vec3::Y * hy, hr, HALO[c], hs + pulse * 0.08);
    if size > 0 {
        let h = (hash2(x, z, 77) % 1000) as f32 / 1000.0;
        let t = (env.time * 0.25 + h).fract();
        if t < 0.8 {
            let k = h * std::f32::consts::TAU + env.time * 0.8;
            let rise = [0.3, 0.4, 0.8][size] + t * 1.1;
            let q = base + Vec3::new(k.sin() * 0.2, rise, k.cos() * 0.14);
            r.point(q, 1, if t < 0.45 { GLOW[c][0] } else { GLOW[c][1] });
        }
    }
}

fn small_rot(x: i32, z: i32) -> Mat4 {
    Mat4::from_rotation_y((hash2(x, z, 3) % 628) as f32 / 100.0)
}

/// Grass rippling in the breeze and parting round the hero's feet. Drawn after ambient
/// occlusion, so a lawn full of blades doesn't turn to speckle.
pub fn draw_grass(r: &mut Renderer, a: &Assets, w: &World, env: &Env) {
    let grass = DrawOpts {
        warp: Warp::Wind {
            t: env.time,
            amp: 0.15 * env.wind,
            h: 0.4,
            push: env.push,
        },
        ..DrawOpts::default().two_sided()
    };
    // Grass is short: only what can actually be on screen.
    let near = r.cam.visible_tiles(0.5);
    for tufts in w.visible_grass(near) {
        r.mesh(&a.bank, tufts, &Mat4::IDENTITY, &grass);
    }
}

/// Draws every object in a tile rectangle into the shadow map (the renderer must be in its
/// shadow pass): trees, buildings, fences, crops, furniture and all.
pub fn cast_objects(
    r: &mut Renderer,
    a: &Assets,
    w: &World,
    env: &Env,
    rect: (i32, i32, i32, i32),
) {
    let (x0, z0, x1, z1) = rect;
    for z in z0.max(0)..=z1.min(w.h - 1) {
        for x in x0.max(0)..=x1.min(w.w - 1) {
            if let Some(o) = w.obj(x, z) {
                draw_object(r, a, w, x, z, o, env);
            }
        }
    }
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
            // The trunk stands firm; the crown rides the breeze.
            let warp = env.sway(base + Vec3::Y * 0.5, 1.4, 0.07);
            r.mesh(
                &a.bank,
                &p.trees[*var as usize % p.trees.len()],
                &(at * small_rot(x, z)),
                &lit.with_warp(warp),
            );
        }
        Obj::Pine { var, .. } => {
            r.shadow(a.tex(a.disk), base, 0.7);
            r.mesh(
                &a.bank,
                &p.pines[*var as usize % p.pines.len()],
                &(at * small_rot(x, z)),
                &lit.with_warp(env.sway(base + Vec3::Y * 0.3, 1.5, 0.05)),
            );
        }
        Obj::Shrub { var, .. } => {
            r.shadow(a.tex(a.disk), base, 0.42);
            r.mesh(
                &a.bank,
                &p.shrubs[*var as usize % p.shrubs.len()],
                &(at * small_rot(x, z)),
                &lit.with_warp(env.sway(base, 0.6, 0.06)),
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
        Obj::Weed { var } => r.mesh(
            &a.bank,
            &p.weeds[*var as usize % p.weeds.len()],
            &(at * small_rot(x, z)),
            &lit.two_sided().with_warp(env.sway(base, 0.5, 0.16)),
        ),
        Obj::Flower { var } => {
            let n = [
                "blossom_pink",
                "blossom_white",
                "blossom_blue",
                "blossom_gold",
            ][*var as usize % 4];
            bb(r, n, 0.9, &lit.with_warp(env.sway(base, 0.9, 0.12)));
        }
        // Drawn with their animations by `candy` and `pets`.
        Obj::CandyRock { .. } | Obj::Egg { .. } => {}
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
        Obj::Crack => {
            let d = &a.delve;
            let full = UvRect::new(0.0, 0.0, 16.0, 16.0);
            r.decal(
                a.tex(d.crack),
                full,
                base + Vec3::Y * 0.012,
                Vec2::splat(0.5),
                &lit,
            );
            // A draught from below stirs up a mote of dust now and then.
            let k = (env.time * 0.7 + (x * 13 + z * 7) as f32 * 0.37).fract();
            if k < 0.6 {
                let sway = (env.time * 2.0 + x as f32).sin() * 0.08;
                let q = base + Vec3::new(sway, 0.05 + k * 0.7, 0.05);
                r.point(q, 1, if k < 0.3 { SAND } else { KHAKI });
            }
        }
        Obj::Hole => {
            let d = &a.delve;
            let full = UvRect::new(0.0, 0.0, 16.0, 16.0);
            r.decal(
                a.tex(d.pit),
                full,
                base + Vec3::Y * 0.014,
                Vec2::splat(0.56),
                &lit,
            );
            let stake = at
                * Mat4::from_translation(Vec3::new(0.34, 0.0, 0.3))
                * Mat4::from_rotation_y(-2.29);
            r.mesh(&a.bank, &d.stake, &stake, &lit);
        }
        Obj::Rope => {
            r.mesh(&a.bank, &a.delve.rope, &at, &lit);
            // The shaft of daylight it hangs in.
            let pulse = (env.time * 1.3).sin() * 0.05;
            r.halo(base + Vec3::Y * 0.05, 0.9, CREAM, 0.22 + pulse);
        }
        Obj::LootChest { opened, gleam } => {
            let mesh = match (*opened, *gleam) {
                (true, _) => &p.chest_open,
                (false, true) => &p.gleam_chest,
                (false, false) => &p.loot_chest,
            };
            let o = if *gleam && !*opened {
                // A gleaming chest shines from within.
                lit.with_glow(0.7 + 0.2 * (env.time * 3.0 + x as f32).sin())
            } else {
                lit
            };
            r.mesh(&a.bank, mesh, &at, &o);
            if !*opened {
                chest_sparkles(r, base, env.time, (x, z), *gleam);
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
        Obj::Drain => {
            // Out of the wall behind, a trickle running out onto the walkway.
            let mouth = Vec3::new(base.x, 0.42, z as f32);
            r.mesh(
                &a.bank,
                &a.sewer.drain,
                &Mat4::from_translation(mouth),
                &lit,
            );
            let lip = mouth + Vec3::new(0.0, -0.1, 0.27);
            for k in 0..5 {
                let t = (env.time * 1.8 + k as f32 * 0.2 + (x * 7 + z) as f32 * 0.13).fract();
                let p =
                    lip + Vec3::new(((k * 5) % 3) as f32 * 0.02 - 0.02, -t * t * 0.32, t * 0.06);
                r.point(p, 1, if k % 2 == 0 { SKY } else { AQUA });
            }
            // Where it lands: a little splash and a puddle.
            let pool = Vec3::new(base.x, 0.02, z as f32 + 0.36);
            r.halo(pool, 0.18, TEAL, 0.3);
            if (env.time * 5.0 + x as f32).sin() > 0.3 {
                r.point(pool + Vec3::new(0.06, 0.03, 0.0), 1, WHITE);
            }
        }
        Obj::Grate => r.mesh(&a.bank, &a.sewer.grate, &at, &lit),
        Obj::Keg { .. } => {
            // Now and then one brimming with glowing green goo.
            let goo = hash2(x, z, 5) % 3 == 0;
            let mesh = if goo {
                &a.sewer.barrel_goo
            } else {
                &a.sewer.barrel
            };
            r.mesh(&a.bank, mesh, &(at * small_rot(x, z)), &lit);
            if goo {
                let pulse = (env.time * 2.0 + x as f32).sin() * 0.5 + 0.5;
                r.halo(base + Vec3::Y * 0.68, 0.24, LIME, 0.18 + pulse * 0.1);
            }
        }
        Obj::Debris { var } => {
            let mesh = &a.sewer.debris[*var as usize % 3];
            if w.floor(x, z) == Floor::Water {
                // Afloat: bobbing and turning slowly on the current.
                let t = env.time + (x * 3 + z) as f32;
                let at = Vec3::new(
                    base.x + (t * 0.4).sin() * 0.08,
                    WATER_Y + 0.02 + (t * 1.7).sin() * 0.012,
                    base.z + (t * 0.3).cos() * 0.06,
                );
                r.mesh(
                    &a.bank,
                    mesh,
                    &(Mat4::from_translation(at)
                        * small_rot(x, z)
                        * Mat4::from_rotation_y((t * 0.25).sin() * 0.3)),
                    &lit,
                );
            } else {
                r.mesh(&a.bank, mesh, &(at * small_rot(x, z)), &lit);
            }
        }
        Obj::Glowcap { var } => draw_glowcap(r, a, (x, z), *var, env),
        Obj::ShelfFungus { var } => {
            // Out of the face of the wall beside the tile, slid along it a little.
            let c = cap_colour(*var);
            let slide = ((hash2(x, z, 5) % 5) as f32 - 2.0) * 0.05;
            let (at, turn, out) = match shelf_side(*var) {
                NORTH => (Vec3::new(base.x + slide, 0.0, z as f32), 0.0, Vec3::Z),
                EAST => (
                    Vec3::new(x as f32 + 1.0, 0.0, base.z + slide),
                    -PI / 2.0,
                    Vec3::NEG_X,
                ),
                _ => (Vec3::new(x as f32, 0.0, base.z + slide), PI / 2.0, Vec3::X),
            };
            r.mesh(
                &a.bank,
                &a.glowcave.shelves[c],
                &(Mat4::from_translation(at) * Mat4::from_rotation_y(turn)),
                &DrawOpts::default().with_mode(Mode::Unlit),
            );
            let pulse = (env.time * 1.1 + (x * 5 + z) as f32).sin() * 0.5 + 0.5;
            r.halo(
                at + out * 0.15 + Vec3::Y * 0.6,
                0.42,
                HALO[c],
                0.2 + pulse * 0.06,
            );
        }
        Obj::GiantShroom { var } => {
            let c = *var as usize % 4;
            let g = &a.glowcave;
            let mid = Vec3::new(x as f32 + 1.0, 0.0, z as f32 + 1.0);
            let m = Mat4::from_translation(mid) * small_rot(x, z);
            r.shadow(a.tex(a.disk), mid, 1.3);
            r.mesh(
                &a.bank,
                &g.giant_stem[c],
                &m,
                &DrawOpts::at(mid + Vec3::Y).with_glow(0.8),
            );
            r.mesh(
                &a.bank,
                &g.giant_cap[c],
                &m,
                &DrawOpts::default().with_mode(Mode::Unlit),
            );
            let pulse = (env.time * 0.9 + x as f32).sin() * 0.5 + 0.5;
            r.halo(mid + Vec3::Y * 2.1, 2.3, HALO[c], 0.18 + pulse * 0.08);
            // Spores sifting down from under the cap.
            for k in 0..7 {
                let t = (env.time * 0.16 + k as f32 / 7.0).fract();
                let turn = k as f32 * 0.9 + env.time * 0.12;
                let rr = 0.9 + (k % 3) as f32 * 0.4;
                let q = mid + Vec3::new(turn.cos() * rr, 1.55 - t * 1.45, turn.sin() * rr);
                r.point(q, 1, if t < 0.5 { GLOW[c][0] } else { GLOW[c][1] });
            }
        }
        Obj::Column { var } => {
            // Marble in the labyrinth, sandstone in the canyon's tombs.
            let columns = if w.canyon {
                &a.canyon.columns
            } else {
                &a.labyrinth.columns
            };
            let var = *var as usize % columns.len();
            let turn = if var == FALLEN as usize {
                small_rot(x, z)
            } else {
                // Square plinths stand square to the walls.
                Mat4::from_rotation_y((hash2(x, z, 3) % 4) as f32 * FRAC_PI_2)
            };
            if var != FALLEN as usize {
                r.shadow(a.tex(a.disk), base, 0.38);
            }
            r.mesh(&a.bank, &columns[var], &(at * turn), &lit);
        }
        Obj::Sandfall { var } => {
            let c = &a.canyon;
            r.mesh(&a.bank, &c.sandfall, &at, &lit);
            // Sand pouring off the top of the pillar down its front, and dust rising where
            // it lands.
            let top = base + Vec3::new(0.0, 2.5, 0.32);
            for k in 0..28 {
                let t = (env.time * 0.9 + k as f32 / 28.0 + *var as f32 * 0.31).fract();
                let spread = ((k * 7) % 5) as f32 * 0.04 - 0.08;
                let q = top + Vec3::new(spread, -t * t * 2.4, 0.02 + t * 0.1);
                let col = [SAND, GOLD, CREAM, WHITE][k % 4];
                r.point(q, if k % 3 == 0 { 2 } else { 1 }, col);
            }
            let pulse = (env.time * 2.0 + x as f32).sin() * 0.5 + 0.5;
            r.halo(
                base + Vec3::new(0.0, 0.15, 0.3),
                0.45,
                SAND,
                0.12 + pulse * 0.06,
            );
            for k in 0..3 {
                let t = (env.time * 0.5 + k as f32 / 3.0).fract();
                let a2 = k as f32 * 2.1 + env.time;
                let q = base + Vec3::new(a2.cos() * 0.35, 0.1 + t * 0.4, 0.35 + a2.sin() * 0.15);
                r.point(q, 1, GOLD);
            }
        }
        Obj::Skull { var } => {
            let c = &a.canyon;
            let turn = if *var == super::canyon::SKULL_PILE {
                small_rot(x, z)
            } else {
                // Great skulls look out towards you, give or take.
                r.shadow(a.tex(a.disk), base, 0.42);
                Mat4::from_rotation_y(((hash2(x, z, 9) % 100) as f32 / 100.0 - 0.5) * 0.9)
            };
            r.mesh(&a.bank, &c.skulls[*var as usize % 3], &(at * turn), &lit);
        }
        Obj::Cactus { var } => {
            r.shadow(a.tex(a.disk), base, 0.3);
            r.mesh(
                &a.bank,
                &a.canyon.cacti[*var as usize % 3],
                &(at * small_rot(x, z)),
                &lit,
            );
        }
        Obj::Sarcophagus => {
            r.mesh(&a.bank, &a.canyon.sarcophagus, &at, &lit);
        }
        Obj::Wyvern => {
            // Round the middle of its six tiles.
            use super::canyon::{WYVERN_H, WYVERN_W};
            let mid = Vec3::new(
                x as f32 + WYVERN_W as f32 * 0.5,
                0.0,
                z as f32 + WYVERN_H as f32 * 0.5,
            );
            r.mesh(
                &a.bank,
                &a.canyon.wyvern,
                &Mat4::from_translation(mid),
                &DrawOpts::at(mid),
            );
        }
        Obj::GoldPile => {
            r.mesh(&a.bank, &a.canyon.gold, &(at * small_rot(x, z)), &lit);
            for k in 0..2 {
                let seed = (x * 31 + z * 17 + k * 13) as u32;
                let t = (env.time / (1.3 + (seed % 5) as f32 * 0.2) + (seed % 11) as f32 / 11.0)
                    .fract();
                if t < 0.4 {
                    let h = hash2(seed as i32, (env.time * 0.8) as i32, 3);
                    let (u, v) = ((h & 0xFF) as f32 / 255.0, ((h >> 8) & 0xFF) as f32 / 255.0);
                    let q = base + Vec3::new(u * 0.5 - 0.25, 0.08 + v * 0.1, v * 0.4 - 0.2);
                    r.sparkle(q, 1 + (t * 5.0) as i32 % 2, WHITE, GOLD);
                }
            }
        }
        Obj::Gateway => r.mesh(&a.bank, &a.canyon.gateway, &at, &lit),
        Obj::Nest => {
            r.shadow(a.tex(a.disk), base, 0.4);
            r.mesh(&a.bank, &a.labyrinth.nest, &at, &lit);
        }
        Obj::Brazier => {
            r.shadow(a.tex(a.disk), base, 0.36);
            r.mesh(&a.bank, &a.labyrinth.brazier, &at, &lit);
            flames(
                r,
                a,
                base + Vec3::new(0.0, 0.74, 0.0),
                0.55,
                env.time,
                x * 3 + z,
            );
            flames(
                r,
                a,
                base + Vec3::new(0.12, 0.72, -0.08),
                0.36,
                env.time + 0.3,
                x + z * 5,
            );
            let pulse = (env.time * 7.0 + (x * 5 + z) as f32).sin() * 0.5 + 0.5;
            r.halo(base + Vec3::Y * 0.95, 0.7, ORANGE, 0.16 + pulse * 0.06);
        }
        Obj::Statue { var } => {
            let l = &a.labyrinth;
            r.shadow(a.tex(a.disk), base, 0.4);
            r.mesh(
                &a.bank,
                &l.statues[*var as usize % l.statues.len()],
                &at,
                &lit,
            );
        }
        Obj::Rubble { var } => {
            let l = &a.labyrinth;
            r.mesh(
                &a.bank,
                &l.rubble[*var as usize % l.rubble.len()],
                &(at * small_rot(x, z)),
                &lit,
            );
        }
        Obj::Arch => r.mesh(&a.bank, &a.labyrinth.arch, &at, &lit),
        Obj::Sconce => {
            r.mesh(&a.bank, &a.labyrinth.sconce, &at, &lit);
            flames(
                r,
                a,
                base + Vec3::new(0.0, 0.8, -0.28),
                0.3,
                env.time,
                x * 7 + z,
            );
        }
        Obj::Sunbeam => {
            // Daylight falling from far above: a shaft of added light, a pool of it on
            // the floor, and dust drifting in it.
            let pulse = (env.time * 0.8 + x as f32).sin() * 0.5 + 0.5;
            r.mesh(
                &a.bank,
                &a.labyrinth.beam,
                &at,
                &DrawOpts {
                    mode: Mode::Glow,
                    alpha: 0.14 + pulse * 0.04,
                    zwrite: false,
                    cull: false,
                    ..DrawOpts::default()
                },
            );
            let lean = Vec3::new(1.3, 3.6, -1.0);
            for k in 0..6 {
                let up = k as f32 / 6.0;
                r.halo(
                    base + lean * up + Vec3::Y * 0.1,
                    0.7,
                    CREAM,
                    (0.2 - up * 0.16) + pulse * 0.04,
                );
            }
            r.halo(base + Vec3::Y * 0.03, 0.8, CREAM, 0.2 + pulse * 0.06);
            for k in 0..5 {
                let t = (env.time * 0.12 + k as f32 / 5.0 + (x * 3 + z) as f32 * 0.1).fract();
                let turn = k as f32 * 1.3 + env.time * 0.3;
                let up = t * 2.6;
                let q = base
                    + Vec3::new(turn.sin() * 0.25, up, turn.cos() * 0.25)
                    + Vec3::new(1.3, 0.0, -1.0) * (up / 3.6);
                r.point(q, 1, if k % 2 == 0 { CREAM } else { WHITE });
            }
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
        Obj::Furniture { f, rot, fish } => draw_furniture(r, a, x, z, *f, *rot, fish, env),
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
                    let ang = i as f32 * std::f32::consts::FRAC_PI_6;
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
        Obj::StreetLamp { lit, bunting } => {
            r.mesh(&a.bank, &a.town.street_lamp, &at, &lit_opts(base));
            if *bunting {
                // A string of little flags to the next lamp along the street.
                if let Some(k) =
                    (2..=9).find(|k| matches!(w.obj(x + k, z), Some(Obj::StreetLamp { .. })))
                {
                    draw_bunting(r, a, base + Vec3::Y * 1.62, k as f32, env.time + x as f32);
                }
            }
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
        Obj::WishTree { blooming } => {
            r.shadow(a.tex(a.disk), base, 1.3);
            let tree = if *blooming { &p.trees[2] } else { &p.trees[0] };
            let m = at * Mat4::from_scale(Vec3::splat(1.9));
            let o = if *blooming { lit.with_glow(0.9) } else { lit };
            let warp = env.sway(base + Vec3::Y * 1.0, 2.6, 0.08);
            r.mesh(&a.bank, tree, &m, &o.with_warp(warp));
            if *blooming {
                // Petals drifting down, and fireflies of light in the branches.
                for i in 0..14 {
                    let t = (env.time * 0.25 + i as f32 * 0.071).fract();
                    let ang = i as f32 * 2.4;
                    let rad = 0.4 + (i % 5) as f32 * 0.25;
                    let q = base
                        + Vec3::new(
                            ang.cos() * rad + (t * 9.0 + i as f32).sin() * 0.2,
                            3.0 - t * 3.0,
                            ang.sin() * rad,
                        );
                    r.point(q, 1, if i % 3 == 0 { WHITE } else { BLUSH });
                }
                for i in 0..5 {
                    let on = (env.time * 2.0 + i as f32 * 1.3).sin() > 0.4;
                    if on {
                        let ang = i as f32 * 1.26 + env.time * 0.2;
                        r.point(
                            base + Vec3::new(
                                ang.cos() * 1.0,
                                2.2 + (i % 2) as f32 * 0.5,
                                ang.sin() * 0.8,
                            ),
                            2,
                            CREAM,
                        );
                    }
                }
            }
        }
        Obj::Planter { var } => {
            let m = &a.town.planters[*var as usize % a.town.planters.len()];
            r.mesh(&a.bank, m, &at, &lit);
        }
        Obj::Bush { var } => {
            r.shadow(a.tex(a.disk), base, 0.45);
            let m = &a.town.bushes[*var as usize % a.town.bushes.len()];
            r.mesh(
                &a.bank,
                m,
                &(at * small_rot(x, z)),
                &lit.with_warp(env.sway(base, 0.62, 0.06)),
            );
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
            bb(r, name, size, &o.with_warp(env.sway(base, size, 0.07)));
            if ready {
                let s = (env.time * 3.0 + (x * 3 + z) as f32).sin();
                if s > 0.6 {
                    r.point(base + Vec3::new(0.25, 0.75, 0.15), 1, WHITE);
                }
            }
        }
    }
}

/// Festival flags sagging from one lamp to another `len` tiles east.
fn draw_bunting(r: &mut Renderer, a: &Assets, from: Vec3, len: f32, t: f32) {
    let tex = a.tex(a.town.bunting);
    let segs = (len * 2.0) as i32;
    let o = DrawOpts {
        light: Light::At(from),
        cull: false,
        ..Default::default()
    };
    for i in 0..segs {
        let k0 = i as f32 / segs as f32;
        let k1 = (i + 1) as f32 / segs as f32;
        let sag = |k: f32| -(k * (1.0 - k)) * 1.1 + (t * 2.0 + k * 6.0).sin() * 0.015;
        let p0 = from + Vec3::new(k0 * len, sag(k0), 0.0);
        let p1 = from + Vec3::new(k1 * len, sag(k1), 0.0);
        let h = Vec3::Y * 0.22;
        let mut m = Mesh::new();
        m.quad(
            [p0 - h, p1 - h, p1, p0],
            UvRect::new(0.0, 0.0, 8.0, 8.0),
            a.town.bunting,
        );
        let _ = tex;
        r.mesh(&a.bank, &m, &Mat4::IDENTITY, &o);
    }
}

/// Lit evenly from the object's position.
fn lit_opts(base: Vec3) -> DrawOpts {
    DrawOpts::at(base)
}

/// A piece of furniture in the house, with whatever moves on it: flames, a swinging
/// pendulum, a spinning globe, fish swimming in their tank.
#[allow(clippy::too_many_arguments)]
fn draw_furniture(
    r: &mut Renderer,
    a: &Assets,
    x: i32,
    z: i32,
    f: Furn,
    rot: u8,
    fish: &[Item],
    env: &Env,
) {
    let h = &a.home;
    let c = home::center(f, rot, x, z);
    let turn = Mat4::from_rotation_y(rot as f32 * PI * 0.5);
    let at = Mat4::from_translation(c) * turn;
    // Local points on the piece, out in the room.
    let here = |p: Vec3| c + turn.transform_vector3(p);
    let lit = DrawOpts::at(c + Vec3::Y * 0.6);
    let glow = DrawOpts::default().with_mode(Mode::Unlit);
    r.mesh(&a.bank, &h.furn[f as usize], &at, &lit);
    let seed = x * 7 + z * 3;
    match f {
        Furn::FloorLamp => r.mesh(&a.bank, &h.lamp_shade, &at, &glow),
        Furn::SnailLamp => {
            r.mesh(&a.bank, &h.snail_shade, &at, &glow);
            let pulse = (env.time * 1.6 + seed as f32).sin() * 0.5 + 0.5;
            r.halo(
                here(Vec3::new(0.0, 0.98, 0.0)),
                0.65,
                PINK,
                0.2 + pulse * 0.1,
            );
        }
        Furn::Candelabra => {
            for (lx, ly) in [(-0.22f32, 1.1f32), (0.0, 1.18), (0.22, 1.1)] {
                flames(
                    r,
                    a,
                    here(Vec3::new(lx, ly, 0.0)),
                    0.16,
                    env.time,
                    seed + (lx * 9.0) as i32,
                );
            }
        }
        Furn::Fireplace => {
            flames(
                r,
                a,
                here(Vec3::new(-0.12, 0.08, -0.24)),
                0.5,
                env.time,
                seed,
            );
            flames(
                r,
                a,
                here(Vec3::new(0.14, 0.06, -0.2)),
                0.42,
                env.time,
                seed + 2,
            );
            if (env.time * 3.0 + x as f32).sin() > 0.7 {
                let k = (env.time * 1.7).fract();
                r.point(here(Vec3::new(0.0, 0.3 + k * 0.6, -0.26)), 1, GOLD);
            }
        }
        Furn::Stove | Furn::Range => {
            // Embers flicker behind the oven door.
            let doors: &[f32] = if f == Furn::Range {
                &[-0.45, 0.45]
            } else {
                &[0.0]
            };
            for (i, dx) in doors.iter().enumerate() {
                if (env.time * 7.0 + i as f32 * 2.0 + x as f32).sin() > 0.2 {
                    r.point(here(Vec3::new(*dx, 0.32, 0.38)), 1, GOLD);
                }
            }
        }
        Furn::Clock => {
            let swing = (env.time * PI).sin() * 0.28;
            let m = Mat4::from_translation(here(Vec3::new(0.0, 1.26, 0.145)))
                * turn
                * Mat4::from_rotation_z(swing);
            r.mesh(&a.bank, &h.pendulum, &m, &lit);
        }
        Furn::Globe => {
            // Idles slowly; a spin whirls it round twice and settles exactly where it began.
            let k = 4.0 * PI / 3.125;
            let ang = env.time * 0.3 + k * (6.25 - env.spin * env.spin) * 0.5;
            let m = Mat4::from_translation(here(Vec3::new(0.0, 0.66, 0.0)))
                * turn
                * Mat4::from_rotation_z(0.4)
                * Mat4::from_rotation_y(ang);
            r.mesh(&a.bank, &h.globe, &m, &lit);
        }
        Furn::FishTank | Furn::FishBowl => {
            let tank = f == Furn::FishTank;
            let (span, y0, dy, depth, size) = if tank {
                (0.6f32, 0.64f32, 0.13f32, 0.16f32, 0.3f32)
            } else {
                (0.1, 0.6, 0.06, 0.05, 0.2)
            };
            let along = turn.transform_vector3(Vec3::X);
            for (i, item) in fish.iter().enumerate() {
                let fi = i as f32;
                let speed = 0.5 + (i % 3) as f32 * 0.17;
                let ph = env.time * speed + fi * 1.9 + seed as f32;
                let lx = ph.sin() * span;
                let dir = ph.cos();
                let ly = y0 + (i % 3) as f32 * dy + (env.time * 1.3 + fi).sin() * 0.02;
                let lz = depth * (((i * 5) % 4) as f32 / 1.5 - 1.0);
                let id = a.icon(item.def().icon);
                // Icons face right; turn the card round when swimming the other way.
                let flip = along.dot(r.cam.right) * dir < 0.0;
                let uv = if flip {
                    UvRect::new(15.96, 0.0, 0.04, 16.0)
                } else {
                    full_uv(a, id)
                };
                r.billboard(
                    a.tex(id),
                    uv,
                    here(Vec3::new(lx, ly, lz)),
                    Vec2::splat(size),
                    &DrawOpts::at(c + Vec3::Y * 0.8).with_glow(1.0),
                );
                // Now and then a bubble rises from a fish.
                let b = (env.time * 0.6 + fi * 0.37).fract();
                if b < 0.5 {
                    let top = if tank { 1.08 } else { 0.8 };
                    let by = ly + size * 0.5 + b * (top - ly - size * 0.5) * 2.0;
                    r.point(here(Vec3::new(lx + 0.05, by.min(top), lz)), 1, WHITE);
                }
            }
            let (water, glass) = if tank {
                (&h.tank_water, &h.tank_glass)
            } else {
                (&h.bowl_water, &h.bowl_glass)
            };
            r.mesh(&a.bank, water, &at, &lit.glass(0.2).with_glow(0.8));
            r.mesh(&a.bank, glass, &at, &lit.glass(0.55).with_glow(1.0));
        }
        _ => {}
    }
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
        10 => &[
            "hp_potion_s",
            "hp_potion_m",
            "hp_potion_l",
            "mp_potion_s",
            "mp_potion_m",
            "mp_potion_l",
            "ep_potion_m",
            "vial",
            "heartleaf",
            "spellbook",
        ],
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
    /// Both arms held straight out in front, 0..1: the way the walking dead go about.
    pub reach: f32,
    /// Drawn this much bigger than life (0 = life size, 1 = twice as big).
    pub grow: f32,
    /// The head tipped back this far (radians): a werewolf howling at the moon.
    pub look_up: f32,
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
    /// Holding a fishing rod out, raised by the progress (0 level, 1 high over the shoulder).
    Fish,
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
    /// Worn on the back (made in the body's own space: see `pack_art`).
    pub pack: Option<&'a Mesh>,
    /// The hat is a hood, hiding all loose hair (other hats tuck it under the brim).
    pub hood: bool,
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
    let root = Mat4::from_translation(pos)
        * Mat4::from_rotation_y(yaw)
        * Mat4::from_scale(Vec3::splat(1.0 + pose.grow));
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
    if let Some(pack) = fit.pack {
        // It jostles a little as you go.
        let jostle = Mat4::from_rotation_x(pose.walk.cos() * 0.05 * pose.stride);
        r.mesh(&a.bank, pack, &(body * jostle), o);
    }
    // Arms: the right follows the swing, the left carries any shield.
    let guard = fit.shield.is_some() && matches!(pose.swing, None | Some((_, Swing::Slash)));
    let off = -HAND;
    let left = root
        * Mat4::from_translation(Vec3::new(off * h.shoulder_x, h.shoulder + bob, 0.0))
        * Mat4::from_rotation_x(if guard {
            -0.5 - sw * 0.2
        } else {
            -sw * 0.6 * (1.0 - pose.reach * 0.7) - pose.reach * 1.45
        })
        * Mat4::from_rotation_z(off * 0.12 * (1.0 - pose.reach));
    r.mesh(&a.bank, &h.parts[ARM_L], &left, o);
    if let Some(shield) = fit.shield {
        // Worn on the forearm, facing outwards and a little forwards.
        let m = left
            * Mat4::from_translation(Vec3::new(off * 0.07, -0.16, 0.02))
            * Mat4::from_rotation_y(off * 1.2);
        r.mesh(&a.bank, shield, &m, o);
    }
    let (rx, rz, twist) = match pose.swing {
        Some((t, Swing::Slash)) => {
            let k = ease_out(t);
            (
                -1.4 + (k * 2.6 - 1.2) * 0.2,
                0.0,
                SLASH_TWIST * (1.0 - 2.0 * k),
            )
        }
        Some((t, Swing::Chop)) => {
            // From overhead, a little behind, down to the ground in front.
            let k = ease_out(t);
            (-3.3 + k * 2.55, 0.0, 0.0)
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
        Some((t, Swing::Fish)) => (-1.5 - t * 1.5, 0.0, 0.0),
        None if held.is_some() => (-0.95 + sw * 0.15, 0.1, 0.0),
        None => (
            sw * 0.6 * (1.0 - pose.reach * 0.7) - pose.reach * 1.45,
            0.12 * (1.0 - pose.reach),
            0.0,
        ),
    };
    // The turns above are for an arm on the +x side: mirrored to the hand's side.
    let right = root
        * Mat4::from_translation(Vec3::new(HAND * h.shoulder_x, h.shoulder + bob, 0.0))
        * Mat4::from_rotation_y(HAND * twist)
        * Mat4::from_rotation_x(rx)
        * Mat4::from_rotation_z(HAND * rz);
    r.mesh(&a.bank, &h.parts[ARM_R], &right, o);
    if let Some(tool) = held {
        let hand = right * Mat4::from_translation(Vec3::new(0.0, -h.hand, 0.02));
        let orient = match pose.swing {
            Some((_, Swing::Pour)) => Mat4::from_rotation_x(0.9),
            _ => Mat4::IDENTITY,
        };
        r.mesh(&a.bank, tool, &(hand * orient), o);
    }
    let head = root
        * Mat4::from_translation(Vec3::new(0.0, h.neck + bob, 0.0))
        * Mat4::from_rotation_x(-0.22 - pose.look_up)
        * Mat4::from_rotation_z(sw * 0.05);
    let head_mesh = match fit.hat {
        Some(_) if fit.hood => &h.head_hooded,
        Some(_) => &h.head_tucked,
        None => &h.parts[HEAD],
    };
    r.mesh(&a.bank, head_mesh, &head, o);
    // Hats are made for a head of scale 1: bigger and smaller folk get theirs to size.
    let hs = h.neck / 0.43;
    let crown = head
        * Mat4::from_translation(Vec3::new(0.0, 0.46 * hs, 0.0))
        * Mat4::from_scale(Vec3::splat(hs));
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

/// The side of a figure the hand holding tools and weapons is on, as x in its own space
/// (where it faces +z): -1 is its right.
pub const HAND: f32 = -1.0;

/// How far round a slash swings the sword arm: from this far round on its own side (radians)
/// to as far round the other way.
const SLASH_TWIST: f32 = 1.3;

/// A translucent slash arc in front of a character, trailing the blade from where the swing
/// began to where the blade is now.
pub fn slash_arc(r: &mut Renderer, pos: Vec3, yaw: f32, t: f32, reach: f32, color: u8) {
    if t > 0.7 {
        return;
    }
    let start = HAND * SLASH_TWIST;
    let end = HAND * SLASH_TWIST * (1.0 - 2.0 * ease_out(t));
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
