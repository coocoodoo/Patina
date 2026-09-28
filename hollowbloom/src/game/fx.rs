//! Short-lived things: particles, floating text, dropped items and projectiles.

use glam::{Vec2, Vec3};

use super::items::Item;
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

/// An item lying in the world (or coins).
pub struct Drop {
    pub item: Option<Item>,
    pub n: u16,
    pub gold: u32,
    pub pos: Vec3,
    pub vel: Vec3,
    pub age: f32,
}

impl Drop {
    pub fn item(item: Item, n: u16, at: Vec3, rng: &mut Rng) -> Drop {
        Drop {
            item: Some(item),
            n,
            gold: 0,
            pos: at + Vec3::Y * 0.3,
            vel: Vec3::new(
                rng.range_f(-1.6, 1.6),
                3.2 + rng.f32(),
                rng.range_f(-1.6, 1.6),
            ),
            age: 0.0,
        }
    }

    pub fn coins(gold: u32, at: Vec3, rng: &mut Rng) -> Drop {
        Drop {
            item: None,
            n: 1,
            gold,
            pos: at + Vec3::Y * 0.3,
            vel: Vec3::new(
                rng.range_f(-1.4, 1.4),
                3.4 + rng.f32(),
                rng.range_f(-1.4, 1.4),
            ),
            age: 0.0,
        }
    }

    /// Physics and pull towards the player; returns true when collected.
    pub fn update(&mut self, dt: f32, world: &World, player: Vec2, can_take: bool) -> bool {
        self.age += dt;
        let flat = Vec2::new(self.pos.x, self.pos.z);
        let to = player - flat;
        let d = to.length();
        if can_take && self.age > 0.45 && d < 1.8 {
            let pull = (1.8 - d + 0.5) * 7.0;
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
