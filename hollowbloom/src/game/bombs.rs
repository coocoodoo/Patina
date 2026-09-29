//! Bombs, and what they open up: a lit bomb flies from your hand, bounces, fizzes and goes
//! off, blasting monsters, pots and crates. Blow open a cracked floor in the Hollow and a
//! rope leads down to a secret room: its treasure and its keepers, and the rope back up to
//! where you were.

use glam::{Mat4, Vec2, Vec3};

use super::Io;
use super::combat::{Flash, Hit};
use super::dungeon::{self, Level};
use super::foes::Enemy;
use super::fx::Drop;
use super::play::{Banner, Play, Trans};
use super::travel::area_world;
use super::world::{Area, Obj};
use crate::assets::Assets;
use crate::assets::delve_art::FUSE_TIP;
use crate::audio::Sfx;
use crate::palette::*;
use crate::render::{DrawOpts, Mode, Renderer};
use crate::util::hash2;

/// Seconds from throw to bang.
pub const FUSE: f32 = 2.0;
/// How far the blast reaches, in tiles.
pub const BLAST: f32 = 2.2;
/// Bombs the smith will sell in a day.
pub const BOMBS_PER_DAY: u8 = 5;

pub struct Bomb {
    pub pos: Vec3,
    pub vel: Vec3,
    pub fuse: f32,
    pub spin: f32,
}

/// A floor of the Hollow put aside while you're somewhere else on it (down in its secret
/// room, or back up top): its layout, creatures, what's lying about, and where you were.
pub struct Stash {
    pub level: Level,
    pub foes: Vec<Enemy>,
    pub drops: Vec<Drop>,
    pub revealed: Vec<bool>,
    pub at: Vec2,
}

impl Play {
    /// In a floor's secret room.
    pub fn in_vault(&self) -> bool {
        self.below.is_some()
    }

    /// Lights the bomb in your hand and lobs it a couple of steps ahead.
    pub fn throw_bomb(&mut self, io: &mut Io) {
        if matches!(self.area, Area::Town | Area::Inside(_) | Area::Home) {
            if self.nag <= 0.0 {
                self.nag = 2.5;
                self.toast("Not here! Save bombs for the Hollow.", None, 0);
                io.audio.play(super::super::audio::Sfx::Denied);
            }
            return;
        }
        let sel = self.player.sel;
        self.player.inv.take_one(sel);
        let p = &self.player;
        let dir = p.facing.normalize_or(Vec2::new(0.0, 1.0));
        self.bombs.push(Bomb {
            pos: p.world_pos() + Vec3::new(dir.x * 0.35, 0.85, dir.y * 0.35),
            vel: Vec3::new(dir.x * 3.2, 2.6, dir.y * 3.2),
            fuse: FUSE,
            spin: 0.0,
        });
        io.audio.play_at(Sfx::Swing, 0.7, 0.8);
    }

    /// Bombs in flight bounce and roll to a stop; fuses burn down; the ones that reach the
    /// end go off.
    pub fn update_bombs(&mut self, dt: f32, io: &mut Io) {
        if self.bombs.is_empty() {
            return;
        }
        let world = area_world(
            self.area,
            &self.farm,
            &self.town,
            &self.house.world,
            &self.level,
            &self.room,
        );
        let mut bangs = Vec::new();
        for b in self.bombs.iter_mut() {
            b.fuse -= dt;
            b.vel.y -= 14.0 * dt;
            let next = b.pos + b.vel * dt;
            if world.blocked(next.x.floor() as i32, next.z.floor() as i32) && next.y < 1.4 {
                // Off a wall.
                b.vel.x *= -0.35;
                b.vel.z *= -0.35;
            } else {
                b.pos.x = next.x;
                b.pos.z = next.z;
            }
            b.pos.y = next.y;
            if b.pos.y <= 0.0 {
                // A thud and a little hop, then it sits there fizzing.
                b.pos.y = 0.0;
                if b.vel.y < -1.2 {
                    b.vel.y = -b.vel.y * 0.35;
                    b.vel.x *= 0.4;
                    b.vel.z *= 0.4;
                } else {
                    b.vel.y = 0.0;
                    b.vel.x *= 0.8;
                    b.vel.z *= 0.8;
                }
            }
            b.spin += Vec2::new(b.vel.x, b.vel.z).length() * dt * 5.0;
            if b.fuse <= 0.0 {
                bangs.push(b.pos);
            }
        }
        self.bombs.retain(|b| b.fuse > 0.0);
        for at in bangs {
            self.explode(at, io);
        }
    }

    /// Boom: everything close enough is hit, pots and crates burst, and a cracked floor
    /// caves in.
    pub fn explode(&mut self, at: Vec3, io: &mut Io) {
        io.audio.play(Sfx::Blast);
        io.audio.play_at(Sfx::Break, 0.8, 0.7);
        self.shake = self.shake.max(0.9);
        let mid = at + Vec3::Y * 0.3;
        self.fx
            .burst(mid, 44, &[WHITE, CREAM, GOLD, ORANGE, RED], 6.5, 3.5);
        self.fx.burst(mid, 26, &[SHADOW, SLATE, KHAKI], 3.0, 1.6);
        self.fx.sparks(mid, Vec3::Y, 14);
        self.flashes.push(Flash {
            pos: at + Vec3::Y * 0.6,
            t: 0.0,
            warmth: 7.0,
        });
        let ground = Vec2::new(at.x, at.z);
        // Creatures in the blast.
        let depth = self.depth().max(1);
        let power = 30 + depth as i32 * 3;
        for i in 0..self.foes.len() {
            let f = &self.foes[i];
            let d = ((f.pos - ground).length() - f.radius).max(0.0);
            if d < BLAST && f.hp > 0 {
                let dmg = (power as f32 * (1.0 - d / BLAST * 0.5)).round() as i32;
                let hit = Hit {
                    dmg,
                    crit: false,
                    burn: false,
                    chill: false,
                    shock: false,
                    knock: 9.0,
                };
                self.strike(i, hit, ground);
            }
        }
        self.reap(io);
        // And you, if you stood too close.
        let to_me = self.player.pos - ground;
        if to_me.length() < 1.3 && self.player.dodge <= 0.0 {
            let dmg = 6 + depth as i32 / 2;
            self.hurt_player(dmg, to_me.normalize_or_zero(), None, io);
        }
        // Pots, crates and cracked floors round about.
        let (cx, cz) = (ground.x.floor() as i32, ground.y.floor() as i32);
        for z in cz - 2..=cz + 2 {
            for x in cx - 2..=cx + 2 {
                let d = (Vec2::new(x as f32 + 0.5, z as f32 + 0.5) - ground).length();
                if d > BLAST {
                    continue;
                }
                match self.world().obj(x, z) {
                    Some(Obj::Pot { .. } | Obj::Crate { .. }) => self.hit_soft(x, z, io),
                    Some(Obj::Crack) if d < 1.8 => self.open_crack(x, z, io),
                    _ => {}
                }
            }
        }
    }

    /// The cracked floor caves in: a hole, and a rope down into the dark.
    fn open_crack(&mut self, x: i32, z: i32, io: &mut Io) {
        self.world_mut().set_obj(x, z, Some(Obj::Hole));
        let at = super::play::tile_center(x, z);
        self.fx
            .burst(at, 30, &[KHAKI, ROSEWOOD, SHADOW, SAND], 3.5, 2.5);
        io.audio.play_at(Sfx::Stairs, 0.8, 0.8);
        self.toast("The floor gave way! A secret room below...", None, 0);
    }

    /// Down the rope into this floor's secret room: as you left it, if you've been before.
    pub fn enter_vault(&mut self) {
        let depth = self.depth();
        let Some(level) = self.level.take() else {
            return;
        };
        let key = level.crack.unwrap_or(self.player.tile());
        let above = Stash {
            level,
            foes: std::mem::take(&mut self.foes),
            drops: std::mem::take(&mut self.drops),
            revealed: std::mem::take(&mut self.revealed),
            at: self.player.pos,
        };
        self.shots.clear();
        self.runes.clear();
        self.bolts.clear();
        self.bombs.clear();
        match self.vault.take() {
            Some(v) => {
                let v = *v;
                self.player.pos = v.at;
                self.level = Some(v.level);
                self.foes = v.foes;
                self.drops = v.drops;
                self.revealed = v.revealed;
            }
            None => {
                let level = dungeon::vault(self.seed, depth, key);
                let biome = dungeon::biome_for(depth);
                let moon = self.moon();
                for (i, s) in level.spawns.iter().enumerate() {
                    let mut f = Enemy::new(
                        s.foe,
                        s.x,
                        s.z,
                        depth,
                        biome,
                        false,
                        hash2(key.0 + i as i32, key.1, self.seed as u32 ^ 0x5EC2),
                    );
                    f.feel_the_moon(moon);
                    self.foes.push(f);
                }
                self.player.pos = Vec2::new(level.start.0 as f32 + 0.5, level.start.1 as f32 + 0.5);
                self.revealed = vec![false; (level.world.w * level.world.h) as usize];
                self.level = Some(level);
            }
        }
        self.below = Some(Box::new(above));
        self.player.facing = Vec2::new(0.0, -1.0);
        self.player.act = None;
        self.cam_pos = self.player.world_pos();
        self.banner = Some(Banner {
            title: "Secret room".into(),
            sub: format!("Beneath floor {depth}"),
            t: 0.0,
        });
    }

    /// Back up the rope, to the hole you came down.
    pub fn leave_vault(&mut self) {
        let Some(above) = self.below.take() else {
            return;
        };
        let Some(level) = self.level.take() else {
            return;
        };
        let rope = level.start;
        let here = Stash {
            level,
            foes: std::mem::take(&mut self.foes),
            drops: std::mem::take(&mut self.drops),
            revealed: std::mem::take(&mut self.revealed),
            at: Vec2::new(rope.0 as f32 + 0.5, rope.1 as f32 + 0.5),
        };
        self.vault = Some(Box::new(here));
        let above = *above;
        self.shots.clear();
        self.runes.clear();
        self.bolts.clear();
        self.bombs.clear();
        self.player.pos = above.at;
        self.level = Some(above.level);
        self.foes = above.foes;
        self.drops = above.drops;
        self.revealed = above.revealed;
        self.player.act = None;
        self.cam_pos = self.player.world_pos();
    }

    /// Forgets any secret room: leaving a floor puts its secrets back where they were.
    pub fn forget_vault(&mut self) {
        self.vault = None;
        self.below = None;
        self.bombs.clear();
    }

    /// Down a blasted hole.
    pub fn climb_down(&mut self, io: &mut Io) {
        io.audio.play_at(Sfx::Stairs, 0.8, 1.1);
        self.start_fade(Trans::Vault { enter: true });
    }

    /// Up the rope.
    pub fn climb_up(&mut self, io: &mut Io) {
        io.audio.play_at(Sfx::Stairs, 0.8, 1.3);
        self.start_fade(Trans::Vault { enter: false });
    }

    /// Bombs in flight or fizzing on the floor, blinking red as the fuse runs out.
    pub fn draw_bombs(&self, r: &mut Renderer, a: &Assets) {
        for b in &self.bombs {
            let m = Mat4::from_translation(b.pos)
                * Mat4::from_rotation_y(b.spin * 0.7)
                * Mat4::from_rotation_z(b.spin.sin() * 0.3);
            let mut o = DrawOpts::at(b.pos);
            if b.fuse < 0.7 && (b.fuse * 14.0).fract() < 0.5 {
                o.mode = Mode::Solid(RED);
            }
            r.shadow(a.tex(a.disk), Vec3::new(b.pos.x, 0.0, b.pos.z), 0.18);
            r.mesh(&a.bank, &a.delve.bomb, &m, &o);
            // The fizzing fuse.
            let tip = m.transform_point3(FUSE_TIP);
            let flick = (b.fuse * 31.0).sin() > 0.0;
            r.sparkle(
                tip,
                if flick { 2 } else { 1 },
                WHITE,
                if flick { GOLD } else { ORANGE },
            );
            r.halo(tip, 0.25, ORANGE, 0.4);
        }
    }
}
