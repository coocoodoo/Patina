//! Casting spells: mana, cooldowns and practice, and what each spell does when it leaves
//! your hands. Spells are learned and readied at the Starfall Spellery (`spellery.rs`).

use glam::{Vec2, Vec3};

use super::Io;
use super::combat::{Bolt, Flash, Hit};
use super::foes::St;
use super::gear::{Class, Stat};
use super::play::Play;
use super::player::{Act, ActKind, RADIUS};
use super::spells::Spell;
use super::world::{Area, Floor, Obj, WATERED};
use crate::audio::Sfx;
use crate::palette::*;
use crate::render::{PointLight, Renderer};

/// A star falling from the sky onto a spot (Starfall).
pub struct Star {
    pub to: Vec2,
    /// 0 when it appears high above, 1 when it lands.
    pub t: f32,
    /// Seconds to wait before it starts to fall.
    pub delay: f32,
    pub hit: Hit,
}

impl Star {
    const HEIGHT: f32 = 6.0;

    pub fn pos(&self) -> Vec3 {
        let k = self.t.clamp(0.0, 1.0);
        let fall = 1.0 - k;
        Vec3::new(
            self.to.x - 1.6 * fall,
            0.3 + Self::HEIGHT * fall,
            self.to.y - 1.1 * fall,
        )
    }
}

/// A crackling bolt of lightning between two points, fading out.
pub struct Zap {
    pub from: Vec3,
    pub to: Vec3,
    pub t: f32,
    pub seed: u32,
}

impl Zap {
    pub const LIFE: f32 = 0.28;
}

impl Play {
    /// How strong your magic is before a spell's own level: grows with you, and with the
    /// wand or staff in your hand.
    fn spell_base(&self) -> f32 {
        let p = &self.player;
        let focus = match p.held_class() {
            Some(Class::Wand | Class::Staff) => {
                p.held_stack().and_then(|s| s.main_value()).unwrap_or(0) as f32 * 0.35
            }
            _ => 0.0,
        };
        7.0 + p.level as f32 * 1.5 + focus + p.stat(Stat::Wisdom).max(0) as f32 * 0.1
    }

    /// A hit from a spell at some share of your magic.
    fn spell_hit(&mut self, share: f32, power: f32, knock: f32) -> Hit {
        let base = self.spell_base() * share * power;
        let crit = self.rng.chance(self.player.crit_chance());
        let mut dmg = base * (0.9 + self.rng.f32() * 0.2);
        if crit {
            dmg *= self.player.crit_mult();
        }
        Hit {
            dmg: (dmg.round() as i32).max(1),
            crit,
            burn: false,
            chill: false,
            shock: false,
            knock,
        }
    }

    /// Q or R: starts casting the spell readied in a slot.
    pub fn begin_cast(&mut self, slot: usize, io: &mut Io) {
        let Some(spell) = self.spells.slots[slot] else {
            if self.nag <= 0.0 {
                self.nag = 2.5;
                let text = if self.spells.known.is_empty() {
                    "You don't know any spells yet. Hazel at the Starfall Spellery teaches them."
                } else {
                    "No spell readied there. Hazel's attuning circle can set one."
                };
                self.toast(text, None, 0);
                io.audio.play(Sfx::Denied);
            }
            return;
        };
        let Some(k) = self.spells.get(spell).copied() else {
            return;
        };
        if self.spells.cool[slot] > 0.0 || self.player.act.is_some() || self.player.dodge > 0.0 {
            return;
        }
        let def = spell.def();
        if !def.gentle && (self.in_town() || matches!(self.area, Area::Inside(_))) {
            if self.nag <= 0.0 {
                self.nag = 2.5;
                self.toast(format!("Save {} for the Hollow.", def.name), None, 0);
                io.audio.play(Sfx::Denied);
            }
            return;
        }
        let cost = self.player.mana_cost(k.cost());
        if self.player.mana < cost {
            if self.player.no_mana_t <= 0.0 {
                self.player.no_mana_t = 1.2;
                self.toast("Not enough mana.", None, 0);
                io.audio.play(Sfx::Denied);
            }
            return;
        }
        self.player.mana -= cost;
        self.spells.cool[slot] = def.cooldown;
        self.casting = Some(spell);
        let tile = self.target.unwrap_or(self.player.tile());
        let dur = 0.32 * self.player.haste();
        self.player.act = Some(Act::new(ActKind::Cast, dur, tile, self.player.facing));
        let pitch = [1.0, 0.8, 1.2, 0.7, 1.1, 0.9, 1.3, 1.05][spell as usize];
        io.audio.play_at(Sfx::Spell, 0.8, pitch);
        let hand = self.player.world_pos() + Vec3::Y * 0.9;
        self.fx.motes(hand, 6, &def.colors, 0.15);
    }

    /// The moment a spell leaves your hands.
    pub fn release_spell(&mut self, dir: Vec2, io: &mut Io) {
        let Some(spell) = self.casting.take() else {
            return;
        };
        let Some(k) = self.spells.get(spell).copied() else {
            return;
        };
        let dir = if dir.length_squared() > 0.0 {
            dir.normalize()
        } else {
            Vec2::new(0.0, 1.0)
        };
        let hits = match spell {
            Spell::Firebolt => self.firebolt(k.level, k.power(), dir, io),
            Spell::FrostNova => self.frost_nova(k.level, k.power(), io),
            Spell::ChainSpark => self.chain_spark(k.level, k.power(), dir, io),
            Spell::Starfall => self.starfall(k.level, k.power(), dir, io),
            Spell::Mend => self.mend(k.level, io),
            Spell::Ward => self.ward(k.level, io),
            Spell::Blink => self.blink(k.level, dir, io),
            Spell::Bloom => self.bloom(k.level, k.power(), io),
        };
        self.stats.casts += 1;
        self.on_cast(k.level);
        // Practice makes perfect: every cast counts, and landing it on things counts more.
        let xp = 1 + hits.min(3);
        let Some(known) = self.spells.get_mut(spell) else {
            return;
        };
        if known.practise(xp) {
            let lv = known.level;
            self.on_cast(lv);
            let def = spell.def();
            io.audio.play(Sfx::SpellUp);
            let at = self.player.world_pos() + Vec3::Y * 1.3;
            self.fx
                .popup_big(at, format!("{} Lv {lv}!", def.name), def.colors[1]);
            self.fx
                .motes(self.player.world_pos(), 18, &def.colors, 0.45);
            self.toast_colored(
                format!(
                    "{} reached level {lv}: stronger and cheaper to cast",
                    def.name
                ),
                None,
                0,
                def.colors[1],
            );
        }
    }

    fn firebolt(&mut self, lv: u8, power: f32, dir: Vec2, io: &mut Io) -> u32 {
        let n: i32 = if lv >= 8 {
            5
        } else if lv >= 4 {
            3
        } else {
            1
        };
        for i in 0..n {
            let a = (i - (n - 1) / 2) as f32 * 0.24;
            let d = Vec2::from_angle(a).rotate(dir);
            let mut hit = self.spell_hit(1.0, power, 3.0);
            hit.burn = true;
            self.bolts.push(Bolt {
                pos: self.player.pos + d * 0.45,
                vel: d * 10.5,
                hit,
                life: 0.9,
                colors: [CREAM, GOLD, ORANGE],
            });
        }
        io.audio.play_at(Sfx::Magic, 0.7, 0.7);
        let hand = self.player.world_pos() + Vec3::new(dir.x * 0.5, 0.6, dir.y * 0.5);
        self.fx.burst(hand, 8, &[CREAM, GOLD, ORANGE], 1.5, 1.0);
        0
    }

    fn frost_nova(&mut self, lv: u8, power: f32, io: &mut Io) -> u32 {
        let radius = 2.0 + 0.15 * lv as f32;
        let at = self.player.pos;
        let targets: Vec<usize> = self
            .foes
            .iter()
            .enumerate()
            .filter(|(_, f)| f.hp > 0 && (f.pos - at).length() < radius + f.radius)
            .map(|(i, _)| i)
            .collect();
        for &i in &targets {
            let mut hit = self.spell_hit(0.8, power, 5.0);
            hit.chill = true;
            self.strike(i, hit, at);
            let f = &mut self.foes[i];
            f.chill = 2.5 + 0.25 * lv as f32;
            if lv >= 5 && !f.boss {
                // Frozen solid for a moment.
                f.st = St::Rest;
                f.t = 0.6 + 0.08 * lv as f32;
            }
        }
        let c = self.player.world_pos();
        self.fx.ice(c, 26, radius * 2.4);
        self.fx.ring(c, radius * 2.8, 36, &[WHITE, SKY, BLUE]);
        self.fx
            .motes(c + Vec3::Y * 0.2, 18, &[WHITE, SKY, WHITE], radius * 0.6);
        self.flashes.push(Flash {
            pos: c + Vec3::Y * 0.6,
            t: 0.0,
            warmth: 0.5,
        });
        io.audio.play_at(Sfx::Blast, 0.8, 1.35);
        self.shake = self.shake.max(0.3);
        self.reap(io);
        targets.len() as u32
    }

    fn chain_spark(&mut self, lv: u8, power: f32, dir: Vec2, io: &mut Io) -> u32 {
        let jumps = 2 + lv as usize / 2;
        let p = self.player.pos;
        let hand = self.player.world_pos() + Vec3::new(dir.x * 0.4, 0.8, dir.y * 0.4);
        // The first target: close, in sight, and roughly where you're facing.
        let w = self.world();
        let first = self
            .foes
            .iter()
            .enumerate()
            .filter(|(_, f)| {
                let d = f.pos - p;
                f.hp > 0 && d.length() < 7.5 && w.clear_line(p, f.pos)
            })
            .min_by(|a, b| {
                let score = |f: &super::foes::Enemy| {
                    let d = f.pos - p;
                    d.length() * (1.6 - d.normalize_or_zero().dot(dir))
                };
                score(a.1).total_cmp(&score(b.1))
            })
            .map(|(i, _)| i);
        let Some(first) = first else {
            // Nothing to strike: it crackles into the ground ahead.
            let to = self.player.world_pos() + Vec3::new(dir.x * 2.5, 0.05, dir.y * 2.5);
            self.zaps.push(Zap {
                from: hand,
                to,
                t: 0.0,
                seed: self.rng.next_u32(),
            });
            self.fx.sparks(to, Vec3::Y, 6);
            io.audio.play_at(Sfx::Clang, 0.4, 1.6);
            return 0;
        };
        let mut chain = vec![first];
        while chain.len() < jumps {
            let last = self.foes[*chain.last().unwrap()].pos;
            let next = self
                .foes
                .iter()
                .enumerate()
                .filter(|(i, f)| f.hp > 0 && !chain.contains(i) && (f.pos - last).length() < 3.8)
                .min_by(|a, b| {
                    (a.1.pos - last)
                        .length()
                        .total_cmp(&(b.1.pos - last).length())
                })
                .map(|(i, _)| i);
            match next {
                Some(i) => chain.push(i),
                None => break,
            }
        }
        let mut from = hand;
        for (n, &i) in chain.iter().enumerate() {
            let to = self.foes[i].world_pos() + Vec3::Y * (0.35 * self.foes[i].scale());
            self.zaps.push(Zap {
                from,
                to,
                t: -(n as f32) * 0.05,
                seed: self.rng.next_u32(),
            });
            // Each jump a touch weaker than the last.
            let share = 1.15 * (1.0 - n as f32 * 0.1).max(0.5);
            let hit = self.spell_hit(share, power, 2.5);
            let origin = Vec2::new(from.x, from.z);
            self.strike(i, hit, origin);
            self.foes[i].st = if self.foes[i].boss {
                self.foes[i].st
            } else {
                St::Rest
            };
            self.foes[i].t = self.foes[i].t.max(0.35);
            self.fx.motes(to, 4, &[WHITE, CREAM, GOLD], 0.15);
            from = to;
        }
        io.audio.play_at(Sfx::Clang, 0.55, 1.8);
        io.audio.play_at(Sfx::Hit, 0.6, 1.4);
        self.shake = self.shake.max(0.25);
        self.reap(io);
        chain.len() as u32
    }

    fn starfall(&mut self, lv: u8, power: f32, dir: Vec2, io: &mut Io) -> u32 {
        let n = 3 + lv as usize / 2;
        let p = self.player.pos;
        let aim = match self.aim {
            Some(a) if (a - p).length() < 6.0 => a,
            _ => p + dir * 2.5,
        };
        let mut targets: Vec<Vec2> = self
            .foes
            .iter()
            .filter(|f| f.hp > 0 && (f.pos - p).length() < 6.5)
            .map(|f| f.pos)
            .collect();
        targets.sort_by(|a, b| (*a - p).length().total_cmp(&(*b - p).length()));
        let found = targets.len() as u32;
        for i in 0..n {
            let to = if targets.is_empty() {
                let a = self.rng.f32() * std::f32::consts::TAU;
                aim + Vec2::new(a.cos(), a.sin()) * self.rng.range_f(0.2, 1.8)
            } else {
                let t = targets[i % targets.len()];
                t + Vec2::new(self.rng.range_f(-0.2, 0.2), self.rng.range_f(-0.2, 0.2))
            };
            let hit = self.spell_hit(0.95, power, 3.0);
            self.stars.push(Star {
                to,
                t: 0.0,
                delay: i as f32 * 0.11,
                hit,
            });
        }
        io.audio.play_at(Sfx::Enchant, 0.4, 1.4);
        found.min(n as u32)
    }

    fn mend(&mut self, lv: u8, io: &mut Io) -> u32 {
        let p = &mut self.player;
        let mut heal = 18.0 + 10.0 * lv as f32;
        if lv >= 3 {
            heal += p.max_hp() as f32 * 0.02 * lv as f32;
        }
        let before = p.hp;
        p.hp = (p.hp + heal.round() as i32).min(p.max_hp());
        let got = p.hp - before;
        let at = p.world_pos();
        self.fx.popup(at + Vec3::Y * 1.0, format!("+{got}"), PINK);
        self.fx
            .motes(at + Vec3::Y * 0.2, 16, &[WHITE, BLUSH, PINK], 0.4);
        self.fx.ring(at, 1.8, 18, &[BLUSH, PINK, WHITE]);
        self.flashes.push(Flash {
            pos: at + Vec3::Y * 0.8,
            t: 0.05,
            warmth: 6.5,
        });
        io.audio.play_at(Sfx::Heart, 0.7, 1.2);
        u32::from(got > 0)
    }

    fn ward(&mut self, lv: u8, io: &mut Io) -> u32 {
        let p = &mut self.player;
        p.ward = 15.0 + 9.0 * lv as f32;
        p.ward_t = 10.0 + lv as f32;
        p.ward_lv = lv;
        let at = p.world_pos();
        self.fx.ring(at, 1.6, 20, &[WHITE, MINT, AQUA]);
        self.fx
            .motes(at + Vec3::Y * 0.4, 12, &[WHITE, MINT, AQUA], 0.4);
        io.audio.play_at(Sfx::Block, 0.6, 1.4);
        1
    }

    fn blink(&mut self, lv: u8, dir: Vec2, io: &mut Io) -> u32 {
        let dist = 3.0 + 0.25 * lv as f32;
        let dir = match self.aim {
            Some(a) if (a - self.player.pos).length() > 0.5 => (a - self.player.pos).normalize(),
            _ => dir,
        };
        let start = self.player.pos;
        let world = self.world();
        let mut to = start;
        let steps = (dist / 0.1) as i32;
        for s in 1..=steps {
            let q = start + dir * (s as f32 * 0.1);
            if (world.collide(q, RADIUS) - q).length() > 0.01 {
                break;
            }
            to = q;
        }
        let a = Vec3::new(start.x, 0.0, start.y);
        let b = Vec3::new(to.x, 0.0, to.y);
        for k in 0..10 {
            let q = a.lerp(b, k as f32 / 9.0) + Vec3::Y * 0.6;
            self.fx.motes(q, 2, &[WHITE, LAVENDER, PURPLE], 0.12);
        }
        self.fx.ring(a, 1.4, 12, &[LAVENDER, PURPLE]);
        self.fx.ring(b, 2.0, 16, &[WHITE, LAVENDER, PURPLE]);
        let p = &mut self.player;
        p.pos = to;
        p.vel = Vec2::ZERO;
        // A heartbeat where nothing can touch you.
        p.hurt = p.hurt.max(0.35);
        self.cam_pos += (b - a) * 0.5;
        io.audio.play_at(Sfx::Dodge, 0.8, 1.6);
        u32::from((to - start).length() > 1.0)
    }

    fn bloom(&mut self, lv: u8, power: f32, io: &mut Io) -> u32 {
        let at = self.player.world_pos();
        self.fx.ring(at, 2.2, 20, &[LIME, GREEN, PINK]);
        io.audio.play_at(Sfx::Harvest, 0.6, 1.2);
        match self.area {
            Area::Farm => {
                let r = 1 + lv as i32 / 3;
                let (px, pz) = self.player.tile();
                let chance = 0.05 + 0.02 * lv as f32;
                let (mut watered, mut grown) = (0, 0);
                for dz in -r..=r {
                    for dx in -r..=r {
                        let (x, z) = (px + dx, pz + dz);
                        if self.farm.floor(x, z) != Floor::Tilled {
                            continue;
                        }
                        self.farm.set_flag(x, z, WATERED, true);
                        watered += 1;
                        let c = Vec3::new(x as f32 + 0.5, 0.2, z as f32 + 0.5);
                        self.fx.motes(c, 2, &[LIME, SKY, WHITE], 0.25);
                        let spurt = self.rng.chance(chance);
                        if let Some(Obj::Crop { crop, days, .. }) = self.farm.obj_mut(x, z) {
                            if spurt && *days < crop.def().days {
                                *days += 1;
                                grown += 1;
                                self.fx.motes(c, 6, &[LIME, PINK, WHITE], 0.2);
                                self.fx.popup(c + Vec3::Y * 0.6, "Grew!", LIME);
                            }
                        }
                    }
                }
                if watered == 0 {
                    self.toast("Bloom waters dug soil around you.", None, 0);
                }
                (watered / 4 + grown) as u32
            }
            Area::Hollow { .. } => {
                // Vines burst from the floor and tangle everything near.
                let radius = 2.5 + 0.12 * lv as f32;
                let p = self.player.pos;
                let targets: Vec<usize> = self
                    .foes
                    .iter()
                    .enumerate()
                    .filter(|(_, f)| f.hp > 0 && (f.pos - p).length() < radius + f.radius)
                    .map(|(i, _)| i)
                    .collect();
                for &i in &targets {
                    let hit = self.spell_hit(0.35, power, 0.5);
                    self.strike(i, hit, p);
                    self.foes[i].chill = self.foes[i].chill.max(3.0 + 0.2 * lv as f32);
                    let c = self.foes[i].world_pos();
                    self.fx
                        .burst(c + Vec3::Y * 0.1, 8, &[GREEN, LIME, TEAL], 1.2, 2.5);
                }
                self.reap(io);
                targets.len() as u32
            }
            _ => {
                // In town it just makes the flowers smile.
                self.fx
                    .motes(at + Vec3::Y * 0.2, 14, &[PINK, BLUSH, LIME, WHITE], 1.2);
                0
            }
        }
    }

    /// Falling stars and fading lightning.
    pub fn update_spells(&mut self, dt: f32, io: &mut Io) {
        self.spells.tick(dt);
        let p = &mut self.player;
        if p.ward_t > 0.0 {
            p.ward_t -= dt;
            if p.ward_t <= 0.0 || p.ward <= 0.0 {
                p.ward_t = 0.0;
                p.ward = 0.0;
            }
        }
        for z in &mut self.zaps {
            z.t += dt;
        }
        self.zaps.retain(|z| z.t < Zap::LIFE);
        if self.stars.is_empty() {
            return;
        }
        let mut stars = std::mem::take(&mut self.stars);
        let mut landed = false;
        stars.retain_mut(|s| {
            if s.delay > 0.0 {
                s.delay -= dt;
                return true;
            }
            s.t += dt / 0.38;
            if s.t < 1.0 {
                if self.rng.chance(0.5) {
                    self.fx.motes(s.pos(), 1, &[WHITE, CREAM, LAVENDER], 0.05);
                }
                return true;
            }
            // Impact.
            let at = Vec3::new(s.to.x, 0.1, s.to.y);
            for i in 0..self.foes.len() {
                let f = &self.foes[i];
                if f.hp > 0 && (f.pos - s.to).length() < 0.9 + f.radius {
                    self.strike(i, s.hit, s.to);
                }
            }
            self.fx.ring(at, 2.4, 14, &[WHITE, CREAM, LAVENDER]);
            self.fx
                .burst(at + Vec3::Y * 0.2, 10, &[WHITE, CREAM, GOLD], 2.5, 2.0);
            self.flashes.push(Flash {
                pos: at + Vec3::Y * 0.5,
                t: 0.1,
                warmth: 3.0,
            });
            landed = true;
            false
        });
        stars.append(&mut self.stars);
        self.stars = stars;
        if landed {
            io.audio.play_at(Sfx::Blast, 0.45, 1.6);
            self.shake = self.shake.max(0.25);
            self.reap(io);
        }
    }

    /// The light thrown by falling stars and lightning.
    pub fn spell_lights(&self, out: &mut Vec<PointLight>) {
        for s in &self.stars {
            if s.delay <= 0.0 {
                out.push(PointLight {
                    pos: s.pos(),
                    radius: 2.6,
                    power: 0.7,
                    warmth: 3.0,
                });
            }
        }
        for z in &self.zaps {
            let k = 1.0 - (z.t / Zap::LIFE).clamp(0.0, 1.0);
            out.push(PointLight {
                pos: z.to,
                radius: 3.2,
                power: 0.9 * k,
                warmth: 2.0,
            });
        }
    }

    /// Stars, lightning and the ward's bubble.
    pub fn draw_spells(&self, r: &mut Renderer) {
        for s in &self.stars {
            if s.delay > 0.0 {
                continue;
            }
            let p = s.pos();
            // A glowing streak falling on a slant.
            for k in 1..5 {
                let tail = p + Vec3::new(-0.16, 0.6, -0.11) * k as f32 * 0.25;
                r.point(tail, 1, if k < 3 { CREAM } else { LAVENDER });
            }
            r.halo(p, 0.35, CREAM, 0.9);
            r.point(p, 3, WHITE);
            // Where it will land.
            let ground = Vec3::new(s.to.x, 0.05, s.to.y);
            if (self.time * 12.0).sin() > 0.0 {
                r.point(ground, 2, LAVENDER);
            }
        }
        for z in &self.zaps {
            if z.t < 0.0 {
                continue;
            }
            let k = 1.0 - (z.t / Zap::LIFE).clamp(0.0, 1.0);
            let segs = 9;
            let mut prev = z.from;
            // A jagged path that re-rolls a few times a second, so it crackles.
            let flicker = (z.t * 30.0) as u32;
            for i in 1..=segs {
                let f = i as f32 / segs as f32;
                let h = crate::util::hash2(i, z.seed.wrapping_add(flicker) as i32, 41);
                let jit = if i == segs {
                    Vec3::ZERO
                } else {
                    Vec3::new(
                        (h % 100) as f32 / 100.0 - 0.5,
                        ((h / 100) % 100) as f32 / 100.0 - 0.5,
                        ((h / 10000) % 100) as f32 / 100.0 - 0.5,
                    ) * 0.35
                };
                let q = z.from.lerp(z.to, f) + jit;
                for t in 0..4 {
                    let pt = prev.lerp(q, t as f32 / 4.0);
                    r.point(
                        pt,
                        if k > 0.6 { 2 } else { 1 },
                        if k > 0.5 { WHITE } else { CREAM },
                    );
                }
                if i % 3 == 0 {
                    r.halo(q, 0.25, CREAM, 0.5 * k);
                }
                prev = q;
            }
            r.halo(z.to, 0.45 * k + 0.1, GOLD, 0.8 * k);
            r.halo(z.from, 0.25 * k, CREAM, 0.6 * k);
        }
    }
}
