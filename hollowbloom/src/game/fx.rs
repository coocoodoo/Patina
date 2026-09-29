//! Short-lived things: particles, floating text, dropped items and projectiles.

use glam::{Vec2, Vec3};

use super::items::{Item, Stack};
use super::world::World;
use crate::palette::*;
use crate::render::{DrawOpts, Mode, Renderer, Texture, UvRect};
use crate::util::Rng;

pub struct Particle {
    pub pos: Vec3,
    pub vel: Vec3,
    pub life: f32,
    pub color: u8,
    pub size: i32,
    pub grav: f32,
}

pub struct Popup {
    pub text: String,
    pub pos: Vec3,
    pub t: f32,
    pub color: u8,
    pub big: bool,
}

/// A hot spark thrown off when metal strikes something hard. It streaks and bounces, cools
/// from white through gold to a dull red, and lights up the ground around it as it goes.
pub struct Spark {
    pub pos: Vec3,
    pub prev: Vec3,
    pub vel: Vec3,
    pub life: f32,
    pub max: f32,
    /// Shards of ice rather than hot metal.
    pub cold: bool,
}

impl Spark {
    /// 1 when it leaves the anvil, 0 when it goes out.
    pub fn heat(&self) -> f32 {
        (self.life / self.max).clamp(0.0, 1.0)
    }

    pub fn color(&self) -> u8 {
        if self.cold {
            return match self.heat() {
                h if h > 0.6 => WHITE,
                h if h > 0.3 => SKY,
                _ => BLUE,
            };
        }
        match self.heat() {
            h if h > 0.72 => WHITE,
            h if h > 0.5 => CREAM,
            h if h > 0.3 => GOLD,
            h if h > 0.14 => ORANGE,
            _ => RUST,
        }
    }
}

/// A puff of smoke from a blast: it flashes white-hot, swells, drifts upwards and thins
/// away.
pub struct Puff {
    pub pos: Vec3,
    pub vel: Vec3,
    /// Radius when it appears and when it's gone.
    pub r0: f32,
    pub r1: f32,
    pub life: f32,
    pub max: f32,
    /// Seconds before it appears.
    pub wait: f32,
}

impl Puff {
    /// 0 when it appears, 1 when it's gone.
    pub fn age(&self) -> f32 {
        (1.0 - self.life / self.max).clamp(0.0, 1.0)
    }

    pub fn radius(&self) -> f32 {
        let k = 1.0 - (1.0 - self.age()).powi(2);
        self.r0 + (self.r1 - self.r0) * k
    }
}

#[derive(Default)]
pub struct Fx {
    pub parts: Vec<Particle>,
    pub pops: Vec<Popup>,
    pub sparks: Vec<Spark>,
    pub puffs: Vec<Puff>,
    pub rng: Option<Rng>,
}

impl Fx {
    fn rng(&mut self) -> &mut Rng {
        self.rng.get_or_insert_with(|| Rng::new(12345))
    }

    /// A burst of particles.
    pub fn burst(&mut self, pos: Vec3, n: usize, colors: &[u8], speed: f32, up: f32) {
        for _ in 0..n {
            let r = self.rng();
            let a = r.f32() * std::f32::consts::TAU;
            let s = speed * (0.4 + r.f32() * 0.8);
            let vel = Vec3::new(a.cos() * s, up * (0.5 + r.f32()), a.sin() * s);
            let color = colors[r.below(colors.len())];
            let life = 0.35 + r.f32() * 0.45;
            let size = if r.chance(0.3) { 2 } else { 1 };
            self.parts.push(Particle {
                pos,
                vel,
                life,
                color,
                size,
                grav: 9.0,
            });
        }
    }

    /// Slow rising motes (sparkles, hearts, magic).
    pub fn motes(&mut self, pos: Vec3, n: usize, colors: &[u8], spread: f32) {
        for _ in 0..n {
            let r = self.rng();
            let off = Vec3::new(
                r.range_f(-spread, spread),
                r.range_f(0.0, 0.4),
                r.range_f(-spread, spread),
            );
            let color = colors[r.below(colors.len())];
            let life = 0.6 + r.f32() * 0.6;
            let rise = 0.8 + r.f32();
            self.parts.push(Particle {
                pos: pos + off,
                vel: Vec3::new(0.0, rise, 0.0),
                life,
                color,
                size: 1,
                grav: 0.0,
            });
        }
    }

    /// A ring of sparks racing outwards along the ground (magic blasts).
    pub fn ring(&mut self, pos: Vec3, speed: f32, n: usize, colors: &[u8]) {
        for i in 0..n {
            let a = i as f32 / n as f32 * std::f32::consts::TAU;
            let r = self.rng();
            let color = colors[r.below(colors.len())];
            let s = speed * (0.85 + r.f32() * 0.3);
            let up = 0.3 + r.f32() * 0.5;
            let life = 0.35 + r.f32() * 0.15;
            self.parts.push(Particle {
                pos: pos + Vec3::Y * 0.15,
                vel: Vec3::new(a.cos() * s, up, a.sin() * s),
                life,
                color,
                size: if i % 3 == 0 { 2 } else { 1 },
                grav: 0.0,
            });
        }
    }

    /// A spray of `n` sparks from a hard hit at `pos`, flying off along `away` (the way out
    /// of the surface that was struck) and upwards.
    pub fn sparks(&mut self, pos: Vec3, away: Vec3, n: usize) {
        let away = away.normalize_or_zero();
        for _ in 0..n {
            let r = self.rng();
            let spray = Vec3::new(
                r.range_f(-1.0, 1.0),
                r.range_f(0.2, 1.3),
                r.range_f(-1.0, 1.0),
            );
            let speed = r.range_f(3.5, 7.5);
            let vel = (away * 1.1 + spray).normalize_or_zero() * speed;
            let life = r.range_f(0.22, 0.6);
            self.sparks.push(Spark {
                pos,
                prev: pos,
                vel,
                life,
                max: life,
                cold: false,
            });
        }
        // Keep a lid on it when everything is being hit at once.
        if self.sparks.len() > 90 {
            let extra = self.sparks.len() - 90;
            self.sparks.drain(..extra);
        }
    }

    /// Shards of ice flung out in a ring along the ground (Frost Nova).
    pub fn ice(&mut self, pos: Vec3, n: usize, speed: f32) {
        for i in 0..n {
            let r = self.rng();
            let a = i as f32 / n as f32 * std::f32::consts::TAU + r.range_f(-0.1, 0.1);
            let s = speed * r.range_f(0.8, 1.2);
            let vel = Vec3::new(a.cos() * s, r.range_f(0.6, 2.0), a.sin() * s);
            let life = r.range_f(0.35, 0.6);
            let p = pos + Vec3::Y * 0.2;
            self.sparks.push(Spark {
                pos: p,
                prev: p,
                vel,
                life,
                max: life,
                cold: true,
            });
        }
    }

    /// `n` puffs of smoke billowing out round `pos`, `spread` across, each growing to about
    /// `size` across.
    pub fn smoke(&mut self, pos: Vec3, n: usize, spread: f32, size: f32) {
        for _ in 0..n {
            let r = self.rng();
            let dir = Vec3::new(
                r.range_f(-1.0, 1.0),
                r.range_f(0.0, 0.8),
                r.range_f(-1.0, 1.0),
            )
            .normalize_or_zero();
            let life = r.range_f(0.5, 0.95);
            let puff = Puff {
                pos: pos + dir * (spread * r.range_f(0.0, 0.5)),
                vel: dir * r.range_f(0.6, 1.6) * spread + Vec3::Y * 0.4,
                r0: size * r.range_f(0.25, 0.4),
                r1: size * r.range_f(0.9, 1.3),
                life,
                max: life,
                wait: r.range_f(0.0, 0.12),
            };
            self.puffs.push(puff);
        }
        if self.puffs.len() > 120 {
            let extra = self.puffs.len() - 120;
            self.puffs.drain(..extra);
        }
    }

    pub fn popup(&mut self, pos: Vec3, text: impl Into<String>, color: u8) {
        self.pops.push(Popup {
            text: text.into(),
            pos,
            t: 0.0,
            color,
            big: false,
        });
    }

    pub fn popup_big(&mut self, pos: Vec3, text: impl Into<String>, color: u8) {
        self.pops.push(Popup {
            text: text.into(),
            pos,
            t: 0.0,
            color,
            big: true,
        });
    }

    pub fn update(&mut self, dt: f32) {
        for p in &mut self.parts {
            p.vel.y -= p.grav * dt;
            p.pos += p.vel * dt;
            if p.pos.y < 0.02 && p.grav > 0.0 {
                p.pos.y = 0.02;
                p.vel *= 0.4;
                p.vel.y = p.vel.y.abs() * 0.3;
            }
            p.life -= dt;
        }
        self.parts.retain(|p| p.life > 0.0);
        for s in &mut self.sparks {
            s.prev = s.pos;
            s.vel.y -= 11.0 * dt;
            // A little drag, so they arc and settle.
            s.vel *= (1.0 - dt * 1.5).max(0.0);
            s.pos += s.vel * dt;
            if s.pos.y < 0.02 {
                s.pos.y = 0.02;
                s.vel.y = s.vel.y.abs() * 0.45;
                s.vel.x *= 0.6;
                s.vel.z *= 0.6;
            }
            s.life -= dt;
        }
        self.sparks.retain(|s| s.life > 0.0);
        for p in &mut self.puffs {
            if p.wait > 0.0 {
                p.wait -= dt;
                continue;
            }
            p.pos += p.vel * dt;
            // Quick out of the blast, then a slow drift upwards.
            p.vel *= (1.0 - dt * 3.0).max(0.0);
            p.vel.y += 0.5 * dt;
            p.life -= dt;
        }
        self.puffs.retain(|p| p.life > 0.0);
        for p in &mut self.pops {
            p.t += dt;
        }
        self.pops.retain(|p| p.t < if p.big { 1.8 } else { 0.9 });
    }

    pub fn draw(&self, r: &mut Renderer) {
        for p in &self.parts {
            r.point(p.pos, p.size, p.color);
        }
        for s in &self.sparks {
            let h = s.heat();
            // A soft glow first, then the bright streak on top of it.
            let glow = match (s.cold, h > 0.5) {
                (true, _) => SKY,
                (false, true) => GOLD,
                (false, false) => ORANGE,
            };
            r.halo(s.pos, 0.06 + h * 0.12, glow, 0.25 + h * 0.5);
            let c = s.color();
            let tail = s.prev + (s.prev - s.pos) * 0.6;
            let trail = if s.cold { SKY } else { GOLD };
            for k in 0..3 {
                r.point(
                    tail.lerp(s.pos, k as f32 / 2.0),
                    1,
                    if k == 2 { c } else { trail },
                );
            }
            if h > 0.75 {
                r.point(s.pos, 2, WHITE);
            }
        }
    }

    /// The smoke, see-through and thinning as it goes: white-hot for a moment, then grey.
    pub fn draw_puffs(&self, r: &mut Renderer, hot: &Texture, cold: &Texture) {
        let uv = UvRect::new(0.0, 0.0, 16.0, 16.0);
        for p in self.puffs.iter().filter(|p| p.wait <= 0.0) {
            let k = p.age();
            let size = p.radius() * 2.0;
            let o = DrawOpts {
                mode: Mode::Glass,
                alpha: (1.0 - k).powf(1.3) * 0.9,
                glow: 1.0,
                zwrite: false,
                ..Default::default()
            };
            let tex = if k < 0.16 { hot } else { cold };
            let base = p.pos - r.cam.up * (size * 0.5);
            r.billboard(tex, uv, base, Vec2::splat(size), &o);
        }
    }

    /// The light the sparks throw on their surroundings, a few at a time.
    pub fn spark_lights(&self, out: &mut Vec<crate::render::PointLight>) {
        for s in self.sparks.iter().rev().step_by(3).take(12) {
            let h = s.heat();
            out.push(crate::render::PointLight {
                pos: s.pos + Vec3::Y * 0.1,
                radius: 1.4 + h * 1.4,
                power: 0.25 + h * 0.5,
                warmth: if s.cold { 1.0 } else { 7.0 },
            });
        }
    }
}

/// An item (or a pile of coins) lying in the world.
pub struct Drop {
    pub stack: Stack,
    pub pos: Vec3,
    pub vel: Vec3,
    pub age: f32,
}

impl Drop {
    pub fn new(stack: Stack, at: Vec3, rng: &mut Rng) -> Drop {
        let coin = stack.item.coin_value().is_some();
        let (spread, up) = if coin { (2.0, 3.8) } else { (1.6, 3.2) };
        Drop {
            stack,
            pos: at + Vec3::Y * 0.3,
            vel: Vec3::new(
                rng.range_f(-spread, spread),
                up + rng.f32(),
                rng.range_f(-spread, spread),
            ),
            age: 0.0,
        }
    }

    pub fn item(item: Item, n: u16, at: Vec3, rng: &mut Rng) -> Drop {
        Drop::new(Stack::new(item, n), at, rng)
    }

    pub fn is_coin(&self) -> bool {
        self.stack.item.coin_value().is_some()
    }

    /// Physics and pull towards the player; returns true when collected.
    pub fn update(&mut self, dt: f32, world: &World, player: Vec2, can_take: bool) -> bool {
        self.age += dt;
        let flat = Vec2::new(self.pos.x, self.pos.z);
        let to = player - flat;
        let d = to.length();
        let range = if self.is_coin() { 2.4 } else { 1.8 };
        if can_take && self.age > 0.45 && d < range {
            let pull = (range - d + 0.5) * 7.0;
            let step = to.normalize_or_zero() * pull * dt;
            self.pos.x += step.x;
            self.pos.z += step.y;
            self.pos.y = (self.pos.y - dt * 2.0).max(0.1);
            return d < 0.35;
        }
        self.vel.y -= 14.0 * dt;
        let next = flat + Vec2::new(self.vel.x, self.vel.z) * dt;
        let next = world.collide(next, 0.1);
        self.pos.x = next.x;
        self.pos.z = next.y;
        self.pos.y += self.vel.y * dt;
        if self.pos.y < 0.08 {
            self.pos.y = 0.08;
            self.vel.y = -self.vel.y * 0.35;
            self.vel.x *= 0.6;
            self.vel.z *= 0.6;
        }
        false
    }
}

/// A projectile (enemy magic).
pub struct Shot {
    pub pos: Vec2,
    pub vel: Vec2,
    pub dmg: i32,
    pub life: f32,
    pub color: u8,
    pub radius: f32,
    pub kind: ShotKind,
}

/// What a creature's shot is, for how it looks and what becomes of it.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ShotKind {
    /// A ball of light in its colour (a spark, a dagger's glint, a bubble, a shard).
    Spark,
    /// Book-worm ink: where it lands it dries into a rune (see `Rune`).
    Ink,
    /// A puff of a drakeling's breath, fire or frost, billowing out as it goes.
    Breath,
    /// A leafling's leaf, spinning as it flies.
    Leaf,
}

/// How long a puff of a drakeling's breath lasts.
pub const BREATH_LIFE: f32 = 0.6;

/// A splash of book-worm ink dried into a glowing rune on the floor. Its arrow points the
/// way on to the stairs down (or, in gold, towards a secret), and it fades after a while.
#[derive(Clone, Copy, Debug)]
pub struct Rune {
    pub pos: Vec2,
    pub dir: Vec2,
    /// Seconds since it landed (below zero, it's still to come), and how long it lasts.
    pub age: f32,
    pub life: f32,
    /// Whose ink: the biome's colour, or the sewers' (see `deep_art::INK_COLORS`).
    pub biome: usize,
    /// Points to a secret rather than the stairs.
    pub secret: bool,
}

/// How long an ink rune glows on the floor.
pub const RUNE_SECS: f32 = 14.0;

impl Shot {
    pub fn update(&mut self, dt: f32, world: &World) -> bool {
        self.pos += self.vel * dt;
        self.life -= dt;
        let (x, z) = (self.pos.x.floor() as i32, self.pos.y.floor() as i32);
        self.life > 0.0 && !world.opaque(x, z)
    }

    pub fn world_pos(&self) -> Vec3 {
        Vec3::new(self.pos.x, 0.45, self.pos.y)
    }
}

pub fn shot_colors(c: u8) -> [u8; 3] {
    match c {
        RED | ORANGE => [CREAM, GOLD, ORANGE],
        AQUA | MINT => [WHITE, MINT, AQUA],
        PINK | LAVENDER => [WHITE, BLUSH, PINK],
        _ => [WHITE, SKY, BLUE],
    }
}
