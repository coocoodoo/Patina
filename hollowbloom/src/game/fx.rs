//! Short-lived things: particles, floating text, dropped items and projectiles.

use glam::{Vec2, Vec3};

use super::items::{Item, Stack};
use super::world::World;
use crate::palette::*;
use crate::render::Renderer;
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

#[derive(Default)]
pub struct Fx {
    pub parts: Vec<Particle>,
    pub pops: Vec<Popup>,
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
        for p in &mut self.pops {
            p.t += dt;
        }
        self.pops.retain(|p| p.t < if p.big { 1.8 } else { 0.9 });
    }

    pub fn draw(&self, r: &mut Renderer) {
        for p in &self.parts {
            r.point(p.pos, p.size, p.color);
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
}

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
