//! Drawing the world: lights, chunks, objects, characters and effects.

use std::f32::consts::PI;

use glam::{Mat4, Vec2, Vec3};

use super::world::{Floor, Obj, World};
use crate::assets::Assets;
use crate::assets::models::{ARM_L, ARM_R, BODY, HEAD, Humanoid, LEG_L, LEG_R};
use crate::palette::*;
use crate::render::{DrawOpts, Light, Mesh, Mode, PointLight, Renderer, UvRect};
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
                Obj::Lamp | Obj::House => env.night > 0.2,
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
}

/// Draws a humanoid at `pos` facing `yaw` (0 = +z). `held` is attached to the right hand.
#[allow(clippy::too_many_arguments)]
pub fn draw_humanoid(
    r: &mut Renderer,
    a: &Assets,
    h: &Humanoid,
    pos: Vec3,
    yaw: f32,
    pose: &Pose,
    o: &DrawOpts,
    held: Option<&Mesh>,
    hat: Option<&Mesh>,
) {
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
    }
    let body = root * Mat4::from_translation(Vec3::new(0.0, h.hip - 0.02 + bob, 0.0));
    r.mesh(&a.bank, &h.parts[BODY], &body, o);
    // Arms: the right arm follows the swing.
    let left = root
        * Mat4::from_translation(Vec3::new(-h.shoulder_x, h.shoulder + bob, 0.0))
        * Mat4::from_rotation_x(-sw * 0.6)
        * Mat4::from_rotation_z(-0.12);
    r.mesh(&a.bank, &h.parts[ARM_L], &left, o);
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
    if let Some(hat) = hat {
        let m = head * Mat4::from_translation(Vec3::new(0.0, 0.46, 0.0));
        r.mesh(&a.bank, hat, &m, o);
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
