//! Candy rocks. While a quest wants them (Pip's, in autumn), they burst up out of the floor
//! where a guardian falls, one after another: each grinds up out of a cracking floor in a
//! spray of sparks, hisses and flashes white faster and faster as it swells, then goes off
//! with a bang, a flash and a cloud of smoke, and sets hard, ready for a pickaxe.

use glam::{Mat4, Vec2, Vec3};

use super::Io;
use super::combat::Flash;
use super::fx::Drop;
use super::items::Item;
use super::play::{Play, tile_center};
use super::world::{Area, Obj};
use crate::assets::Assets;
use crate::audio::Sfx;
use crate::palette::*;
use crate::render::{DrawOpts, Mode, Renderer, UvRect};
use crate::util::hash2;

/// How many burst out.
pub const ROCKS: usize = 5;
/// Seconds pushing up out of the floor; the bang (after hissing and flashing); settling.
pub const RISE: f32 = 0.55;
pub const BANG: f32 = 2.2;
pub const SETTLE: f32 = 0.45;
/// Seconds between one rock starting and the next.
const STAGGER: f32 = 0.45;
/// Hits from a pickaxe to knock one loose.
const HP: i16 = 2;
/// Drawn a little larger than a crystal, to stand out among the guardian's chests.
const SIZE: f32 = 1.3;

/// A candy rock on its way up.
#[derive(Clone, Copy)]
pub struct Eruption {
    pub x: i32,
    pub z: i32,
    /// Seconds since it started (negative while it waits its turn).
    pub t: f32,
}

impl Play {
    /// Candy rocks come up in a ring round where a guardian fell, if a quest wants them.
    /// Returns how many.
    pub fn erupt(&mut self, at: Vec2) -> usize {
        let Area::Hollow { depth } = self.area else {
            return 0;
        };
        if self.eruption_wanted(depth).is_none() {
            return 0;
        }
        let (px, pz) = self.player.tile();
        let Some(level) = self.level.as_mut() else {
            return 0;
        };
        let w = &mut level.world;
        let (cx, cz) = (at.x.floor() as i32, at.y.floor() as i32);
        // Open floor round the spot, nearest to a ring a few steps out first; never right
        // under your feet. (A rock in the way of a chest is only a swing of a pickaxe.)
        let mut spots: Vec<(f32, i32, i32)> = Vec::new();
        for z in cz - 8..=cz + 8 {
            for x in cx - 8..=cx + 8 {
                let d = ((x - cx) as f32).hypot((z - cz) as f32);
                if !(1.0..=8.2).contains(&d)
                    || ((x - px).abs() <= 1 && (z - pz).abs() <= 1)
                    || w.blocked(x, z)
                    || w.obj(x, z).is_some()
                {
                    continue;
                }
                spots.push(((d - 3.4).abs() + self.rng.f32() * 0.8, x, z));
            }
        }
        spots.sort_by(|a, b| a.0.total_cmp(&b.0));
        let mut placed: Vec<(i32, i32)> = Vec::new();
        for &(_, x, z) in &spots {
            if placed.len() >= ROCKS {
                break;
            }
            if placed
                .iter()
                .any(|&(qx, qz)| (qx - x).abs() <= 1 && (qz - z).abs() <= 1)
            {
                continue;
            }
            w.set_obj(x, z, Some(Obj::CandyRock { hp: HP }));
            placed.push((x, z));
        }
        for (i, &(x, z)) in placed.iter().enumerate() {
            self.erupting.push(Eruption {
                x,
                z,
                t: -0.8 - i as f32 * STAGGER,
            });
        }
        placed.len()
    }

    pub fn update_eruptions(&mut self, dt: f32, io: &mut Io) {
        if !matches!(self.area, Area::Hollow { .. }) {
            self.erupting.clear();
            return;
        }
        let mut hissing = false;
        let mut i = 0;
        while i < self.erupting.len() {
            let before = self.erupting[i].t;
            self.erupting[i].t += dt;
            let Eruption { x, z, t } = self.erupting[i];
            let c = tile_center(x, z);
            if before < 0.0 && t >= 0.0 {
                // The floor cracks and heaves.
                io.audio.play_at(Sfx::Mine, 0.9, 0.6);
                io.audio.play_at(Sfx::Break, 0.5, 0.5);
                self.fx
                    .burst(c + Vec3::Y * 0.05, 16, &[KHAKI, ROSEWOOD, SHADOW], 2.2, 2.2);
                self.fx.sparks(c + Vec3::Y * 0.1, Vec3::Y, 10);
                self.fx.smoke(c + Vec3::Y * 0.1, 4, 0.4, 0.4);
                self.shake = self.shake.max(0.3);
            }
            if (0.0..RISE).contains(&t) {
                // Grit and sparks thrown off as it grinds its way up.
                if self.rng.chance(dt * 22.0) {
                    let a = self.rng.f32() * std::f32::consts::TAU;
                    let away = Vec3::new(a.cos(), 0.6, a.sin());
                    self.fx.sparks(c + Vec3::Y * (0.1 + t), away, 1);
                }
                if self.rng.chance(dt * 30.0) {
                    self.fx.burst(c, 1, &[KHAKI, SHADOW], 1.5, 1.5);
                }
            } else if (RISE..BANG).contains(&t) {
                // The fuse: sparks hissing off its tip and a wisp of smoke.
                hissing = true;
                if self.rng.chance(dt * 26.0) {
                    self.fx.sparks(c + Vec3::Y * 0.5, Vec3::Y, 1);
                }
                if self.rng.chance(dt * 5.0) {
                    self.fx.smoke(c + Vec3::Y * 0.55, 1, 0.1, 0.2);
                }
            }
            if before < BANG && t >= BANG {
                self.candy_bang(c, io);
            }
            if t >= BANG + SETTLE {
                self.erupting.remove(i);
                if self.erupting.is_empty() {
                    self.toast_colored(
                        "Candy rocks! Knock them loose with your pickaxe.",
                        Some(Item::CandyRock),
                        0,
                        PINK,
                    );
                }
            } else {
                i += 1;
            }
        }
        // One fizz for all of them, now and then.
        if hissing && self.rng.chance(dt * 3.0) {
            io.audio.play_at(Sfx::Sizzle, 0.35, 1.3);
        }
    }

    /// Bang: a white flash, smoke billowing out, sparks and sugar everywhere.
    fn candy_bang(&mut self, c: Vec3, io: &mut Io) {
        io.audio.play_at(Sfx::Blast, 0.7, 1.15);
        io.audio.play_at(Sfx::Break, 0.6, 1.4);
        let mid = c + Vec3::Y * 0.35;
        self.fx.smoke(mid, 12, 0.9, 0.75);
        self.fx.sparks(mid, Vec3::Y, 16);
        self.fx.burst(
            mid,
            26,
            &[WHITE, BLUSH, PINK, MINT, CREAM, LAVENDER],
            3.4,
            3.0,
        );
        self.fx.motes(mid, 10, &[WHITE, PINK, CREAM], 0.4);
        self.flashes.push(Flash {
            pos: c + Vec3::Y * 0.6,
            t: 0.0,
            warmth: 5.0,
        });
        self.shake = self.shake.max(0.6);
        // It shoves you back if you're standing right over it.
        let ground = Vec2::new(c.x, c.z);
        let to_me = self.player.pos - ground;
        if to_me.length() < 1.2 {
            let push = to_me.normalize_or(Vec2::Y) * (1.2 - to_me.length());
            let w = self.world();
            let to = w.move_circle(self.player.pos, push, super::player::RADIUS);
            self.player.pos = to;
        }
    }

    /// A pickaxe on a candy rock: a crunch of sugar, and once it's loose, a Candy Rock.
    pub fn mine_candy(&mut self, x: i32, z: i32, hp: i16, power: i32, io: &mut Io) {
        if self.erupting.iter().any(|e| (e.x, e.z) == (x, z)) {
            return;
        }
        let at = tile_center(x, z);
        let hp = hp - power.max(1) as i16;
        io.audio.play_at(Sfx::Mine, 1.0, 1.5);
        self.fx.burst(
            at + Vec3::Y * 0.4,
            10,
            &[WHITE, BLUSH, PINK, MINT],
            2.0,
            1.8,
        );
        if hp <= 0 {
            self.world_mut().set_obj(x, z, None);
            io.audio.play_at(Sfx::Break, 0.8, 1.6);
            self.drops
                .push(Drop::item(Item::CandyRock, 1, at, &mut self.rng));
            self.fx
                .motes(at + Vec3::Y * 0.3, 10, &[WHITE, PINK, CREAM], 0.3);
        } else {
            self.world_mut().set_obj(x, z, Some(Obj::CandyRock { hp }));
        }
    }

    /// The candy rocks on this floor: rising, flashing and swelling while they erupt, then
    /// glinting where they've set.
    pub fn draw_candy(&self, r: &mut Renderer, a: &Assets) {
        if !matches!(self.area, Area::Hollow { .. }) {
            return;
        }
        let Some(level) = &self.level else { return };
        let w = &level.world;
        let (x0, z0, x1, z1) = r.cam.visible_tiles(2.5);
        for z in z0.max(0)..=z1.min(w.h - 1) {
            for x in x0.max(0)..=x1.min(w.w - 1) {
                if !matches!(w.obj(x, z), Some(Obj::CandyRock { .. })) {
                    continue;
                }
                let t = self
                    .erupting
                    .iter()
                    .find(|e| (e.x, e.z) == (x, z))
                    .map_or(f32::MAX, |e| e.t);
                if t < 0.0 {
                    // Still under the floor.
                    continue;
                }
                self.draw_candy_rock(r, a, x, z, t);
            }
        }
    }

    fn draw_candy_rock(&self, r: &mut Renderer, a: &Assets, x: i32, z: i32, t: f32) {
        let base = tile_center(x, z);
        let pets = &a.pets;
        let mesh = &pets.candy[hash2(x, z, 7) as usize % pets.candy.len()];
        let turn = (hash2(x, z, 11) % 628) as f32 / 100.0;
        let lit = DrawOpts::at(base).with_glow(0.55);
        let big = Mat4::from_scale(Vec3::splat(SIZE));
        if t >= BANG + SETTLE {
            let m = Mat4::from_translation(base) * Mat4::from_rotation_y(turn) * big;
            r.mesh(&a.bank, mesh, &m, &lit);
            // A glint on a crystal now and then.
            let k = (self.time * 0.5 + (x * 7 + z * 3) as f32 * 0.13).fract();
            if k < 0.12 {
                let arm = 1 + (k * 20.0) as i32 % 2;
                r.sparkle(base + Vec3::new(0.02, 0.44 * SIZE, 0.02), arm, WHITE, BLUSH);
            }
            return;
        }
        // Rising up out of the floor, easing to a stop.
        let rise = if t < RISE {
            let k = t / RISE;
            -0.55 * (1.0 - k) * (1.0 - k)
        } else {
            0.0
        };
        // Swelling before the bang; a wobble as it sets.
        let (sx, sy) = if t < BANG {
            let k = ((t - (RISE + (BANG - RISE) * 0.55)) / ((BANG - RISE) * 0.45)).clamp(0.0, 1.0);
            let s = 1.0 + 0.18 * k * k * (3.0 - 2.0 * k);
            (s, s)
        } else {
            let k = (t - BANG) / SETTLE;
            let wob = (k * std::f32::consts::TAU * 1.5).sin() * (1.0 - k) * 0.14;
            (1.0 - wob * 0.5, 1.0 + wob)
        };
        // A jitter while it grinds upwards.
        let jig = if t < RISE {
            Vec3::new((t * 90.0).sin() * 0.02, 0.0, (t * 70.0).cos() * 0.02)
        } else {
            Vec3::ZERO
        };
        let m = Mat4::from_translation(base + jig + Vec3::Y * rise)
            * Mat4::from_rotation_y(turn)
            * Mat4::from_scale(Vec3::new(sx, sy, sx))
            * big;
        // Flashing white on and off, faster and faster, like a lit fuse.
        let flash = (RISE..BANG).contains(&t) && {
            let k = (t - RISE) / (BANG - RISE);
            let phase = (t - RISE) * (2.6 + 7.0 * k);
            phase.fract() < 0.5
        };
        let o = if flash {
            DrawOpts::at(base).with_mode(Mode::Solid(WHITE))
        } else {
            lit
        };
        r.mesh(&a.bank, mesh, &m, &o);
        // The floor it broke through.
        let full = UvRect::new(0.0, 0.0, 16.0, 16.0);
        r.decal(
            a.tex(a.delve.crack),
            full,
            base + Vec3::Y * 0.012,
            Vec2::splat(0.6),
            &DrawOpts::at(base),
        );
        if flash {
            r.halo(base + Vec3::Y * 0.3, 0.7, WHITE, 0.35);
        }
        // The flash of the bang itself.
        if (BANG..BANG + 0.22).contains(&t) {
            let k = (t - BANG) / 0.22;
            r.halo(
                base + Vec3::Y * 0.4,
                1.8 * (1.0 - k * 0.5),
                WHITE,
                0.95 * (1.0 - k),
            );
        }
    }
}
