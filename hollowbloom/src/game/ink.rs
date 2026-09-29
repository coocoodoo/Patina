//! Book-worm ink. Wherever a gob of it lands it dries into a glowing rune on the floor, its
//! arrow pointing along the way to the stairs down; now and then, while a floor still hides
//! a cracked floor, a gold one points towards that instead. An Ink Map, made from the ink,
//! lays a whole line of them from your feet.

use glam::{Mat4, Vec2, Vec3};

use super::Io;
use super::dungeon::biome_for;
use super::fx::{RUNE_SECS, Rune};
use super::play::Play;
use super::world::{Area, Obj, World};
use crate::assets::Assets;
use crate::assets::deep_art::INK_COLORS;
use crate::audio::Sfx;
use crate::palette::*;
use crate::render::{DrawOpts, Mesh, Mode, Renderer, UvRect};

/// Runes on the floor at once, at most (the oldest go first).
const MAX_RUNES: usize = 40;
/// How far ahead along the way a rune's arrow looks, in tiles (so it points round corners
/// smoothly rather than zig-zagging).
const LOOK_AHEAD: usize = 3;
/// An Ink Map's runes: how many, how far apart (in tiles), and how long they last.
const MAP_RUNES: usize = 14;
const MAP_GAP: usize = 3;
const MAP_SECS: f32 = 40.0;

fn tile(p: Vec2) -> (i32, i32) {
    (p.x.floor() as i32, p.y.floor() as i32)
}

fn centre((x, z): (i32, i32)) -> Vec2 {
    Vec2::new(x as f32 + 0.5, z as f32 + 0.5)
}

/// Which way to go from `at` to reach `goal` on foot: along the walk if there is one,
/// otherwise straight at it.
pub fn way(world: &World, at: Vec2, goal: (i32, i32)) -> Vec2 {
    let path = world.path(tile(at), goal);
    let ahead = path
        .get(LOOK_AHEAD.min(path.len().saturating_sub(1)))
        .copied()
        .unwrap_or(goal);
    (centre(ahead) - at).normalize_or(Vec2::Y)
}

impl Play {
    /// A secret on this floor that hasn't been found yet (a cracked floor still whole).
    fn hidden_secret(&self) -> Option<(i32, i32)> {
        let level = self.level.as_ref()?;
        level
            .crack
            .filter(|&(x, z)| matches!(level.world.obj(x, z), Some(Obj::Crack)))
    }

    /// Whose ink it is on this floor: the biome's book-worms', or (in the old sewers) the
    /// bone worms' green.
    fn ink_look(&self, depth: u32) -> usize {
        if self.world().sewer {
            crate::assets::SEWER_LOOK
        } else {
            biome_for(depth) % crate::assets::BIOMES
        }
    }

    /// Ink splashes down at `at` and dries into a rune pointing the way on.
    pub fn splash_ink(&mut self, at: Vec2) {
        let depth = self.depth();
        let secret = self.hidden_secret().filter(|_| self.rng.chance(0.35));
        let Some(level) = &self.level else { return };
        let goal = secret.unwrap_or(level.stairs);
        let dir = way(&level.world, at, goal);
        let biome = self.ink_look(depth);
        let [light, main] = INK_COLORS[biome];
        self.fx.burst(
            Vec3::new(at.x, 0.1, at.y),
            8,
            &[light, main, main],
            1.6,
            1.2,
        );
        self.runes.push(Rune {
            pos: at,
            dir,
            age: 0.0,
            life: RUNE_SECS,
            biome,
            secret: secret.is_some(),
        });
        if self.runes.len() > MAX_RUNES {
            self.runes.remove(0);
        }
    }

    /// Reads an Ink Map: a line of runes lights up from your feet towards the stairs down,
    /// and a gold one points to any secret the floor still hides.
    pub fn read_ink_map(&mut self, io: &mut Io) {
        if !matches!(self.area, Area::Hollow { .. }) || self.level.is_none() {
            if self.nag <= 0.0 {
                self.nag = 2.5;
                self.toast("The ink only knows the ways of the Hollow.", None, 0);
                io.audio.play(Sfx::Denied);
            }
            return;
        }
        let sel = self.player.sel;
        self.player.inv.take_one(sel);
        let depth = self.depth();
        let biome = self.ink_look(depth);
        let from = self.player.pos;
        let secret = self.hidden_secret();
        let Some(level) = &self.level else { return };
        let path = level.world.path(tile(from), level.stairs);
        let mut runes = Vec::new();
        for (k, step) in path.iter().enumerate().skip(1).step_by(MAP_GAP) {
            if runes.len() >= MAP_RUNES {
                break;
            }
            let at = centre(*step);
            let ahead = path
                .get((k + LOOK_AHEAD).min(path.len() - 1))
                .copied()
                .unwrap_or(level.stairs);
            runes.push(Rune {
                pos: at,
                dir: (centre(ahead) - at).normalize_or(Vec2::Y),
                // They light up one after another, down the line.
                age: -(runes.len() as f32) * 0.12,
                life: MAP_SECS,
                biome,
                secret: false,
            });
        }
        if let Some(s) = secret {
            let at = from + way(&level.world, from, s) * 1.2;
            runes.push(Rune {
                pos: at,
                dir: way(&level.world, at, s),
                age: 0.0,
                life: MAP_SECS,
                biome,
                secret: true,
            });
        }
        let found = !path.is_empty();
        self.runes.extend(runes);
        let excess = self.runes.len().saturating_sub(MAX_RUNES);
        self.runes.drain(..excess);
        let [light, main] = INK_COLORS[biome];
        self.fx.motes(
            self.player.world_pos() + Vec3::Y * 0.6,
            12,
            &[light, main, WHITE],
            0.4,
        );
        io.audio.play(Sfx::Pickup);
        let msg = match (found, secret.is_some()) {
            (true, true) => "The ink shows the way down... and something hidden!",
            (true, false) => "The ink shows the way down.",
            (false, true) => "The ink swirls towards something hidden!",
            (false, false) => "The ink swirls, lost. Is the way down blocked?",
        };
        self.toast(msg, Some(super::items::Item::InkMap), 0);
    }

    /// Runes fade away.
    pub fn update_runes(&mut self, dt: f32) {
        self.runes.retain_mut(|r| {
            r.age += dt;
            r.age < r.life
        });
    }
}

/// A rune lying on the floor, glowing: it swells in as it lands and flickers out at the end.
pub fn draw_rune(r: &mut Renderer, a: &Assets, rune: &Rune, time: f32) {
    let left = rune.life - rune.age;
    if rune.age < 0.0 || (left < 2.5 && (time * 14.0 + rune.pos.x).sin() > left / 2.5 * 2.0 - 1.0) {
        return;
    }
    let look = rune.biome.min(INK_COLORS.len() - 1);
    let [light, main] = if rune.secret {
        [CREAM, GOLD]
    } else {
        INK_COLORS[look]
    };
    let tex = a.deep.runes[if rune.secret {
        a.deep.runes.len() - 1
    } else {
        look
    }];
    let grow = (rune.age * 5.0).clamp(0.2, 1.0);
    let s = 0.44 * grow;
    let d = rune.dir.normalize_or(Vec2::Y);
    let (d3, p3) = (Vec3::new(d.x, 0.0, d.y) * s, Vec3::new(-d.y, 0.0, d.x) * s);
    let c = Vec3::new(rune.pos.x, 0.03, rune.pos.y);
    let mut m = Mesh::new();
    m.quad(
        [c - d3 - p3, c - d3 + p3, c + d3 + p3, c + d3 - p3],
        UvRect::new(0.0, 0.0, 16.0, 16.0),
        tex,
    );
    let o = DrawOpts {
        mode: Mode::Unlit,
        cull: false,
        ..Default::default()
    };
    r.mesh(&a.bank, &m, &Mat4::IDENTITY, &o);
    let pulse = (time * 3.0 + rune.pos.y).sin() * 0.5 + 0.5;
    r.halo(c + Vec3::Y * 0.04, 0.42, main, 0.2 + pulse * 0.12);
    // A mote rising off it now and then.
    let k = (time * 0.7 + rune.pos.x * 0.37).fract();
    if k < 0.6 {
        r.point(c + d3 * 0.5 + Vec3::Y * (k * 0.5), 1, light);
    }
}
