//! Fighting: hits rolled from the hero's stats, sword arcs, wand bolts, staff blasts,
//! burning, chill and shock, heartsip and thorns, dodging and blocking, and what happens
//! when a creature is defeated.

use glam::{Vec2, Vec3};

use super::Io;
use super::dungeon::{Foe, biome_for};
use super::foes::{St, boss_name};
use super::fx::Drop;
use super::gear::Stat;
use super::items::Item;
use super::loot;
use super::play::{Banner, Play};
use super::world::Area;
use crate::audio::Sfx;
use crate::palette::*;

#[derive(Clone, Copy, Debug)]
pub struct Hit {
    pub dmg: i32,
    pub crit: bool,
    pub burn: bool,
    pub chill: bool,
    pub shock: bool,
    /// Knockback speed.
    pub knock: f32,
}

impl Hit {
    /// A plain hit with no rolls (thorns, burning, shock chains).
    pub fn plain(dmg: i32) -> Hit {
        Hit {
            dmg: dmg.max(1),
            crit: false,
            burn: false,
            chill: false,
            shock: false,
            knock: 1.5,
        }
    }
}

/// A magic bolt from a wand.
pub struct Bolt {
    pub pos: Vec2,
    pub vel: Vec2,
    pub hit: Hit,
    pub life: f32,
    pub colors: [u8; 3],
}

impl Bolt {
    pub fn world_pos(&self) -> Vec3 {
        Vec3::new(self.pos.x, 0.5, self.pos.y)
    }
}

/// A short-lived flash of light (blasts, level ups).
pub struct Flash {
    pub pos: Vec3,
    pub t: f32,
    pub warmth: f32,
}

/// The colours of a bolt: from the element it carries, or from the wand itself.
pub fn bolt_colors(hit: &Hit, wand: Option<Item>) -> [u8; 3] {
    if hit.burn {
        [CREAM, GOLD, ORANGE]
    } else if hit.chill {
        [WHITE, SKY, BLUE]
    } else if hit.shock {
        [WHITE, CREAM, GOLD]
    } else {
        match wand {
            Some(Item::BubbleWand) => [WHITE, SKY, AQUA],
            Some(Item::GlowcapWand) => [WHITE, MINT, AQUA],
            Some(Item::CandyWand) => [WHITE, BLUSH, PINK],
            Some(Item::CrystalWand) => [WHITE, MINT, TEAL],
            Some(Item::EmberWand) => [CREAM, GOLD, ORANGE],
            Some(Item::FrostWand) => [WHITE, SKY, BLUE],
            Some(Item::StarWand) => [WHITE, CREAM, GOLD],
            Some(Item::MoonpetalWand) => [WHITE, BLUSH, LAVENDER],
            _ => [WHITE, BLUSH, LAVENDER],
        }
    }
}

impl Play {
    /// Rolls a hit from the hero's stats on top of a base damage.
    pub fn roll_hit(&mut self, base: i32, knock: f32) -> Hit {
        let p = &self.player;
        let (crit_c, crit_m) = (p.crit_chance(), p.crit_mult());
        let burn = p.sheet.frac(Stat::Burn, 80);
        let chill = p.sheet.frac(Stat::Chill, 80);
        let shock = p.sheet.frac(Stat::Shock, 80);
        let crit = self.rng.chance(crit_c);
        let mut dmg = base + self.rng.below(3) as i32;
        if crit {
            dmg = (dmg as f32 * crit_m).round() as i32;
        }
        Hit {
            dmg: dmg.max(1),
            crit,
            burn: self.rng.chance(burn),
            chill: self.rng.chance(chill),
            shock: self.rng.chance(shock),
            knock,
        }
    }

    /// Lands a hit on a foe: damage, numbers, statuses and heartsip. Defeated foes are
    /// cleared away by [`Play::reap`].
    pub fn strike(&mut self, i: usize, hit: Hit, from: Vec2) {
        let Some(f) = self.foes.get_mut(i) else {
            return;
        };
        if f.hp <= 0 {
            return;
        }
        f.hp -= hit.dmg;
        f.flash = 0.12;
        f.hurt_cd = 0.22;
        let heavy = if f.boss || f.foe == Foe::Golem {
            0.3
        } else {
            1.0
        };
        f.vel = (f.pos - from).normalize_or_zero() * hit.knock * heavy;
        if f.st == St::Windup && !f.boss {
            f.st = St::Rest;
            f.t = 0.4;
        }
        f.alert = true;
        let at = f.world_pos();
        let top = at + Vec3::Y * (0.7 * f.scale());
        let col = f.color();
        let pos = f.pos;
        if hit.burn {
            f.burn = 3.0;
            f.burn_dmg = (hit.dmg / 5).max(1);
        }
        if hit.chill {
            f.chill = 2.5;
        }
        let (text, color) = if hit.crit {
            (format!("{}!", hit.dmg), GOLD)
        } else {
            (hit.dmg.to_string(), WHITE)
        };
        self.fx.popup(top, text, color);
        self.fx
            .burst(at + Vec3::Y * 0.3, 6, &[WHITE, CREAM, col], 2.5, 1.5);
        if hit.crit {
            self.fx.motes(top, 5, &[GOLD, CREAM, WHITE], 0.2);
        }
        if hit.burn {
            self.fx
                .burst(at + Vec3::Y * 0.4, 8, &[GOLD, ORANGE, RED], 1.5, 2.5);
        }
        if hit.chill {
            self.fx.motes(at + Vec3::Y * 0.2, 8, &[WHITE, SKY], 0.35);
        }
        // Heartsip.
        let sip = self.player.sheet.frac(Stat::Lifesteal, 30);
        if sip > 0.0 {
            self.player.regen_acc += hit.dmg as f32 * sip;
        }
        // Shock jumps to up to two foes nearby.
        if hit.shock {
            let mut chained = 0;
            for j in 0..self.foes.len() {
                if chained >= 2 {
                    break;
                }
                if j == i || self.foes[j].hp <= 0 {
                    continue;
                }
                let d = self.foes[j].pos - pos;
                if d.length() < 3.0 {
                    let to = self.foes[j].world_pos() + Vec3::Y * 0.4;
                    let from3 = Vec3::new(pos.x, 0.4, pos.y);
                    for k in 0..8 {
                        let t = k as f32 / 7.0;
                        let jitter = Vec3::new(
                            (k as f32 * 7.3).sin() * 0.08,
                            (k as f32 * 3.1).cos() * 0.08,
                            0.0,
                        );
                        self.fx
                            .motes(from3.lerp(to, t) + jitter, 1, &[WHITE, CREAM, GOLD], 0.02);
                    }
                    let zap = Hit::plain(hit.dmg / 2);
                    let f2 = &mut self.foes[j];
                    f2.hp -= zap.dmg;
                    f2.flash = 0.12;
                    f2.alert = true;
                    let top2 = f2.world_pos() + Vec3::Y * (0.7 * f2.scale());
                    self.fx.popup(top2, zap.dmg.to_string(), CREAM);
                    chained += 1;
                }
            }
        }
    }

    /// Clears away defeated foes: experience, loot and fanfare.
    pub fn reap(&mut self, io: &mut Io) {
        let mut xp = 0;
        let mut i = 0;
        while i < self.foes.len() {
            if self.foes[i].hp > 0 {
                i += 1;
                continue;
            }
            let f = self.foes.remove(i);
            xp += f.xp;
            io.audio.play(Sfx::EnemyDie);
            let at = f.world_pos();
            self.fx.burst(
                at + Vec3::Y * 0.3,
                if f.boss { 40 } else { 14 },
                &[f.color(), WHITE, CREAM],
                3.0,
                2.5,
            );
            let depth = self.depth().max(1);
            let fortune = self.fortune();
            let loot = loot::foe_loot(
                f.foe,
                f.boss,
                biome_for(depth),
                depth,
                fortune,
                &mut self.rng,
            );
            let mut rare = false;
            for s in loot {
                rare |= s.rarity().is_some_and(|r| r >= super::gear::Rarity::Rare);
                self.drops.push(Drop::new(s, at, &mut self.rng));
            }
            if rare {
                io.audio.play(Sfx::Rare);
            }
            self.stats.kills += 1;
            if f.boss {
                self.toast(format!("{} defeated!", boss_name(f.foe)), None, 0);
                self.banner = Some(Banner {
                    title: "Guardian defeated".into(),
                    sub: "The waystone awakens".into(),
                    t: 0.0,
                });
                self.shake = 1.2;
            }
        }
        if xp > 0 {
            let ups = self.player.gain_xp(xp);
            if ups > 0 {
                io.audio.play(Sfx::LevelUp);
                let at = self.player.world_pos();
                self.fx.popup_big(at + Vec3::Y * 1.2, "LEVEL UP!", GOLD);
                self.fx.motes(at, 20, &[GOLD, CREAM, WHITE], 0.5);
                self.flashes.push(Flash {
                    pos: at + Vec3::Y,
                    t: 0.0,
                    warmth: 6.0,
                });
                self.toast(
                    format!(
                        "Level {}! Max HP {}",
                        self.player.level,
                        self.player.max_hp()
                    ),
                    None,
                    0,
                );
            }
        }
    }

    /// Hits every foe in an arc in front of the hero. Returns how many were hit.
    pub fn melee(
        &mut self,
        base: i32,
        reach: f32,
        half_angle: f32,
        dir: Vec2,
        knock: f32,
        io: &mut Io,
    ) -> usize {
        let p = self.player.pos;
        let targets: Vec<usize> = self
            .foes
            .iter()
            .enumerate()
            .filter(|(_, f)| {
                let d = f.pos - p;
                let dist = d.length();
                if dist > reach + f.radius || f.hurt_cd > 0.0 || f.hp <= 0 {
                    return false;
                }
                dist <= 0.4 || d.normalize_or_zero().angle_to(dir).abs() <= half_angle
            })
            .map(|(i, _)| i)
            .collect();
        for &i in &targets {
            let hit = self.roll_hit(base, knock);
            self.strike(i, hit, p);
        }
        if !targets.is_empty() {
            io.audio.play(Sfx::Hit);
            io.audio.play_at(Sfx::EnemyHurt, 0.8, 1.0);
            self.shake = self.shake.max(0.35);
        }
        self.reap(io);
        targets.len()
    }

    /// Fires a wand bolt.
    pub fn cast_bolt(&mut self, dir: Vec2, io: &mut Io) {
        let base = self.player.weapon_damage();
        let hit = self.roll_hit(base, 3.5);
        let colors = bolt_colors(&hit, self.player.held());
        let dir = dir.normalize_or_zero();
        self.bolts.push(Bolt {
            pos: self.player.pos + dir * 0.45,
            vel: dir * 9.5,
            hit,
            life: 0.75,
            colors,
        });
        io.audio.play(Sfx::Magic);
        self.fx.motes(
            self.player.world_pos() + Vec3::new(dir.x * 0.5, 0.5, dir.y * 0.5),
            4,
            &colors,
            0.1,
        );
    }

    /// A staff blast: everything around the point gets hit.
    pub fn cast_blast(&mut self, at: Vec2, io: &mut Io) {
        let base = self.player.weapon_damage();
        let radius = 1.6 + self.player.reach() as f32 * 0.2;
        let targets: Vec<usize> = self
            .foes
            .iter()
            .enumerate()
            .filter(|(_, f)| f.hp > 0 && (f.pos - at).length() < radius + f.radius)
            .map(|(i, _)| i)
            .collect();
        let probe = self.roll_hit(0, 0.0);
        let colors = if probe.burn {
            [GOLD, ORANGE, RED]
        } else if probe.chill {
            [WHITE, SKY, BLUE]
        } else if probe.shock {
            [WHITE, CREAM, GOLD]
        } else {
            [WHITE, MINT, AQUA]
        };
        for &i in &targets {
            let mut hit = self.roll_hit(base, 5.0);
            hit.burn |= probe.burn;
            hit.chill |= probe.chill;
            self.strike(i, hit, at);
        }
        let c = Vec3::new(at.x, 0.0, at.y);
        self.fx.ring(c, radius * 2.8, 28, &colors);
        self.fx.motes(c + Vec3::Y * 0.2, 14, &colors, radius * 0.6);
        self.flashes.push(Flash {
            pos: c + Vec3::Y * 0.6,
            t: 0.0,
            warmth: if probe.burn { 7.5 } else { 2.5 },
        });
        io.audio.play(Sfx::Blast);
        self.shake = self.shake.max(0.45);
        // Blasts also smash pots and cut the grass.
        let r = radius.ceil() as i32;
        for dz in -r..=r {
            for dx in -r..=r {
                let (x, z) = (at.x.floor() as i32 + dx, at.y.floor() as i32 + dz);
                let d = Vec2::new(x as f32 + 0.5, z as f32 + 0.5) - at;
                if d.length() <= radius {
                    self.hit_soft(x, z, io);
                }
            }
        }
        self.reap(io);
    }

    /// Moves wand bolts and lets them hit things.
    pub fn update_bolts(&mut self, dt: f32, io: &mut Io) {
        if self.bolts.is_empty() {
            return;
        }
        let mut bolts = std::mem::take(&mut self.bolts);
        bolts.retain_mut(|b| {
            b.life -= dt;
            let step = b.vel * dt;
            let n = ((step.length() / 0.2).ceil() as i32).max(1);
            for _ in 0..n {
                b.pos += step / n as f32;
                let (x, z) = (b.pos.x.floor() as i32, b.pos.y.floor() as i32);
                if self.world().opaque(x, z) {
                    self.fx.burst(b.world_pos(), 6, &b.colors, 1.5, 1.0);
                    return false;
                }
                let target = self
                    .foes
                    .iter()
                    .position(|f| f.hp > 0 && (f.pos - b.pos).length() < f.radius + 0.18);
                if let Some(i) = target {
                    let from = b.pos - b.vel.normalize_or_zero();
                    self.strike(i, b.hit, from);
                    self.fx.burst(b.world_pos(), 10, &b.colors, 2.5, 1.5);
                    io.audio.play_at(Sfx::Hit, 0.7, 1.3);
                    return false;
                }
                // Bolts pop pots and snip weeds on the way.
                if self.world().obj(x, z).is_some_and(|o| {
                    matches!(
                        o,
                        super::world::Obj::Pot { .. } | super::world::Obj::Crate { .. }
                    )
                }) {
                    self.hit_soft(x, z, io);
                    return false;
                }
            }
            if self.rng.chance(0.6) {
                self.fx.motes(b.world_pos(), 1, &b.colors, 0.05);
            }
            b.life > 0.0
        });
        bolts.append(&mut self.bolts);
        self.bolts = bolts;
        self.reap(io);
    }

    /// Burning and chill wear off; burning hurts.
    pub fn update_statuses(&mut self, dt: f32, io: &mut Io) {
        let mut any = false;
        for i in 0..self.foes.len() {
            let f = &mut self.foes[i];
            if f.chill > 0.0 {
                f.chill -= dt;
                if self.rng.chance(dt * 6.0) {
                    let at = f.world_pos() + Vec3::Y * 0.3;
                    self.fx.motes(at, 1, &[WHITE, SKY], 0.3);
                }
            }
            if f.burn > 0.0 {
                f.burn -= dt;
                f.burn_tick -= dt;
                if self.rng.chance(dt * 10.0) {
                    let at = f.world_pos() + Vec3::Y * 0.3;
                    self.fx.motes(at, 1, &[GOLD, ORANGE, RED], 0.25);
                }
                if f.burn_tick <= 0.0 {
                    f.burn_tick = 0.5;
                    f.hp -= f.burn_dmg;
                    f.flash = 0.06;
                    let top = f.world_pos() + Vec3::Y * (0.7 * f.scale());
                    let d = f.burn_dmg;
                    self.fx.popup(top, d.to_string(), ORANGE);
                    any = true;
                }
            }
        }
        if any {
            self.reap(io);
        }
    }

    /// The hero takes a hit, unless they dodge or block it. Defense softens it; thorns
    /// bite back at `attacker`. Returns true if it hurt.
    pub fn hurt_player(
        &mut self,
        dmg: i32,
        dir: Vec2,
        attacker: Option<usize>,
        io: &mut Io,
    ) -> bool {
        if self.player.invulnerable() {
            return false;
        }
        let head = self.player.world_pos() + Vec3::Y * 1.0;
        if self.rng.chance(self.player.dodge_chance()) {
            self.player.hurt = 0.45;
            self.fx.popup(head, "Dodge!", MINT);
            self.fx.motes(head - Vec3::Y * 0.6, 6, &[WHITE, MINT], 0.3);
            io.audio.play(Sfx::Dodge);
            return false;
        }
        if self.rng.chance(self.player.block_chance()) {
            self.player.hurt = 0.45;
            self.player.vel = dir * 3.0;
            self.fx.popup(head, "Block!", SKY);
            self.fx
                .burst(head - Vec3::Y * 0.5, 8, &[WHITE, SKY, CREAM], 2.5, 1.0);
            io.audio.play(Sfx::Block);
            self.thorns(attacker);
            return false;
        }
        let depth = match self.area {
            Area::Hollow { depth } => depth,
            Area::Farm => 1,
        };
        let def = self.player.defense().max(0) as f32;
        let k = 12.0 + 3.0 * depth as f32;
        let taken = ((dmg as f32) * k / (k + def)).round().max(1.0) as i32;
        let p = &mut self.player;
        p.hp -= taken;
        p.hurt = 0.9;
        p.flash = 0.25;
        p.vel = dir * 7.0;
        p.act = None;
        self.shake = self.shake.max(0.6);
        io.audio.play(Sfx::PlayerHurt);
        self.fx.popup(head, format!("-{taken}"), RED);
        self.fx
            .burst(head - Vec3::Y * 0.5, 8, &[RED, SALMON, WHITE], 2.0, 1.5);
        self.thorns(attacker);
        true
    }

    fn thorns(&mut self, attacker: Option<usize>) {
        let thorns = self.player.stat(Stat::Thorns);
        if let Some(i) = attacker {
            if thorns > 0 && i < self.foes.len() {
                let from = self.player.pos;
                self.strike(i, Hit::plain(thorns), from);
                let at = self.foes[i].world_pos() + Vec3::Y * 0.3;
                self.fx.burst(at, 6, &[LIME, GREEN], 2.0, 1.0);
            }
        }
    }
}
