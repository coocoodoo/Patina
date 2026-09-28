//! Hollow creatures: stats, behaviour and drawing.

use glam::{Mat4, Vec2, Vec3};

use super::draw::{Pose, draw_humanoid};
use super::dungeon::Foe;
use super::fx::{Fx, Shot};
use super::world::World;
use crate::assets::Assets;
use crate::palette::*;
use crate::render::{DrawOpts, Light, Mode, PointLight, Renderer, Warp};
use crate::util::{Rng, approach};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum St {
    Idle,
    Chase,
    Windup,
    Dash,
    Rest,
    Hop,
}

pub struct Enemy {
    pub foe: Foe,
    pub boss: bool,
    pub biome: usize,
    pub pos: Vec2,
    pub vel: Vec2,
    pub y: f32,
    pub vy: f32,
    pub hp: i32,
    pub max_hp: i32,
    pub dmg: i32,
    pub speed: f32,
    pub radius: f32,
    pub xp: u32,
    pub st: St,
    pub t: f32,
    pub dir: Vec2,
    pub yaw: f32,
    pub flash: f32,
    pub hurt_cd: f32,
    pub anim: f32,
    pub alert: bool,
    pub hops: u32,
    pub squash: f32,
    pub seed: u32,
    /// Seconds of burning left, and how much each tick hurts.
    pub burn: f32,
    pub burn_dmg: i32,
    pub burn_tick: f32,
    /// Seconds of chill left (chilled foes move at half speed).
    pub chill: f32,
    /// How riled up the moon has them: how far they notice you, how fast they move and how
    /// quickly they strike (1 = a half moon).
    pub fury: f32,
    /// Under a full moon: glowing red, tougher, and carrying better loot.
    pub moonlit: bool,
}

/// The outline tag for creatures maddened by the full moon (outlined in red).
pub const MOONLIT_TAG: u8 = 6;

pub struct Base {
    pub hp: i32,
    pub dmg: i32,
    pub speed: f32,
    pub radius: f32,
    pub xp: u32,
}

pub fn base(f: Foe) -> Base {
    let b = |hp, dmg, speed, radius, xp, _name: &str| Base {
        hp,
        dmg,
        speed,
        radius,
        xp,
    };
    match f {
        Foe::Slime => b(14, 6, 2.4, 0.34, 4, "Slime"),
        Foe::Bat => b(9, 5, 3.3, 0.28, 4, "Bat"),
        Foe::Shroom => b(18, 7, 1.7, 0.3, 5, "Shroomling"),
        Foe::Crab => b(22, 9, 1.9, 0.34, 7, "Crystal Crab"),
        Foe::Wisp => b(12, 7, 1.6, 0.26, 7, "Wisp"),
        Foe::Beetle => b(26, 9, 1.9, 0.32, 8, "Beetle"),
        Foe::Imp => b(20, 10, 2.4, 0.3, 9, "Imp"),
        Foe::Skeleton => b(30, 11, 2.2, 0.3, 10, "Skeleton"),
        Foe::Golem => b(60, 16, 1.3, 0.5, 16, "Golem"),
        Foe::Ghost => b(18, 10, 1.8, 0.3, 9, "Ghost"),
        Foe::Frog => b(16, 7, 2.6, 0.3, 6, "Bog Frog"),
        Foe::Jelly => b(14, 8, 1.2, 0.3, 7, "Drift Jelly"),
        Foe::Puffer => b(20, 9, 1.4, 0.32, 8, "Puffer"),
    }
}

pub fn boss_name(f: Foe) -> &'static str {
    match f {
        Foe::Slime => "King Slime",
        Foe::Crab => "Crystal Matriarch",
        Foe::Shroom => "Old Capwood",
        Foe::Imp => "Ember Lord",
        Foe::Golem => "Frost Colossus",
        Foe::Skeleton => "The Bone Warden",
        _ => "Guardian",
    }
}

impl Enemy {
    pub fn new(foe: Foe, x: f32, z: f32, depth: u32, biome: usize, boss: bool, seed: u32) -> Enemy {
        let b = base(foe);
        let d = depth.max(1) as f32 - 1.0;
        let mut hp = (b.hp as f32 * (1.0 + 0.14 * d)) as i32;
        let mut dmg = (b.dmg as f32 * (1.0 + 0.08 * d)) as i32;
        let mut xp = (b.xp as f32 * (1.0 + 0.1 * d)) as u32;
        let mut radius = b.radius;
        if boss {
            hp *= 9;
            dmg = dmg * 3 / 2;
            xp *= 12;
            radius *= 1.9;
        }
        Enemy {
            foe,
            boss,
            biome,
            pos: Vec2::new(x, z),
            vel: Vec2::ZERO,
            y: if matches!(
                foe,
                Foe::Bat | Foe::Wisp | Foe::Ghost | Foe::Jelly | Foe::Puffer
            ) {
                0.55
            } else {
                0.0
            },
            vy: 0.0,
            hp,
            max_hp: hp,
            dmg,
            speed: b.speed,
            radius,
            xp,
            st: St::Idle,
            t: 0.5 + (seed % 100) as f32 / 100.0,
            dir: Vec2::new(0.0, 1.0),
            yaw: 0.0,
            flash: 0.0,
            hurt_cd: 0.0,
            anim: (seed % 628) as f32 / 100.0,
            alert: false,
            hops: 0,
            squash: 0.0,
            seed,
            burn: 0.0,
            burn_dmg: 0,
            burn_tick: 0.5,
            chill: 0.0,
            fury: 1.0,
            moonlit: false,
        }
    }

    /// Tonight's moon stirs them up (or calms them down). A full moon makes them glow red,
    /// hit harder and take more beating, and they drop better things for it.
    pub fn feel_the_moon(&mut self, phase: super::sky::MoonPhase) {
        self.fury = phase.fury();
        self.speed *= self.fury.sqrt();
        if phase.full() {
            self.moonlit = true;
            self.max_hp = (self.max_hp as f32 * 1.6).ceil() as i32;
            self.hp = self.max_hp;
            self.dmg = (self.dmg as f32 * 1.25).ceil() as i32;
            self.xp = self.xp * 3 / 2 + 1;
        }
    }

    pub fn scale(&self) -> f32 {
        if self.boss { 2.0 } else { 1.0 }
    }

    pub fn world_pos(&self) -> Vec3 {
        Vec3::new(self.pos.x, self.y, self.pos.y)
    }

    /// Jelly and ghosts are see-through, so they are drawn after everything else.
    pub fn translucent(&self) -> bool {
        matches!(self.foe, Foe::Slime | Foe::Ghost | Foe::Jelly)
    }

    /// Hard shells and bones strike sparks.
    pub fn armored(&self) -> bool {
        matches!(
            self.foe,
            Foe::Crab | Foe::Beetle | Foe::Golem | Foe::Skeleton
        )
    }

    /// Does this enemy touch the ground (and so can bump the player)?
    pub fn grounded(&self) -> bool {
        match self.foe {
            Foe::Slime | Foe::Frog => self.y < 0.35 * self.scale(),
            _ => true,
        }
    }

    fn passes_walls(&self) -> bool {
        self.foe == Foe::Ghost
    }

    fn step(&mut self, world: &World, delta: Vec2) {
        if self.passes_walls() {
            let n = self.pos + delta;
            if world.inside(n.x as i32, n.y as i32) {
                self.pos = n;
            }
        } else {
            self.pos = world.move_circle(self.pos, delta, self.radius.min(0.45));
        }
    }

    /// Advances the AI. Returns true if the enemy wants to deal contact damage this frame.
    #[allow(clippy::too_many_arguments)]
    pub fn update(
        &mut self,
        dt: f32,
        world: &World,
        player: Vec2,
        shots: &mut Vec<Shot>,
        spawns: &mut Vec<Enemy>,
        fx: &mut Fx,
        rng: &mut Rng,
        depth: u32,
    ) {
        self.anim += dt;
        self.flash = (self.flash - dt).max(0.0);
        self.hurt_cd = (self.hurt_cd - dt).max(0.0);
        self.squash = approach(self.squash, 0.0, dt * 3.0);
        // Knockback decays.
        if self.vel.length_squared() > 0.001 {
            let v = self.vel * dt;
            self.step(world, v);
            self.vel *= (1.0 - dt * 8.0).max(0.0);
        }
        let to = player - self.pos;
        let dist = to.length();
        let dirp = to.normalize_or_zero();
        if !self.alert {
            let sees = dist < 7.0 * self.fury
                && (self.passes_walls() || world.clear_line(self.pos, player));
            if sees || (self.boss && dist < 9.0) {
                self.alert = true;
                fx.popup(self.world_pos() + Vec3::Y * (0.8 * self.scale()), "!", GOLD);
            } else {
                self.idle_wander(dt, world, rng);
                return;
            }
        }
        if dist > 16.0 * self.fury.max(1.0) && !self.boss {
            self.alert = false;
            return;
        }
        // A bigger moon, a shorter fuse.
        self.t -= dt * self.fury;
        match self.foe {
            Foe::Slime => self.slime(dt, world, dirp, dist, spawns, rng, depth),
            Foe::Bat => {
                self.y = 0.55 + (self.anim * 5.0).sin() * 0.1;
                let side =
                    Vec2::new(-dirp.y, dirp.x) * (self.anim * 3.0 + self.seed as f32).sin() * 0.8;
                let dir = if self.st == St::Rest {
                    -dirp
                } else {
                    (dirp + side).normalize_or_zero()
                };
                if self.st == St::Rest && self.t <= 0.0 {
                    self.st = St::Chase;
                }
                self.dir = dir;
                self.step(world, dir * self.speed * dt);
            }
            Foe::Shroom | Foe::Skeleton | Foe::Ghost => {
                if self.foe == Foe::Ghost {
                    self.y = 0.5 + (self.anim * 2.0).sin() * 0.12;
                }
                match self.st {
                    St::Windup => {
                        if self.t <= 0.0 {
                            self.st = St::Dash;
                            self.t = 0.25;
                        }
                    }
                    St::Dash => {
                        let d = self.dir * self.speed * 3.2 * dt;
                        self.step(world, d);
                        if self.t <= 0.0 {
                            self.st = St::Rest;
                            self.t = 0.6;
                        }
                    }
                    St::Rest => {
                        if self.t <= 0.0 {
                            self.st = St::Chase;
                        }
                    }
                    _ => {
                        self.dir = dirp;
                        if self.foe == Foe::Skeleton && dist < 1.6 {
                            self.st = St::Windup;
                            self.t = 0.35;
                        } else {
                            self.step(world, dirp * self.speed * dt);
                        }
                        if self.boss && self.t <= 0.0 {
                            // Summon helpers now and then.
                            self.t = 4.0;
                            for k in 0..2 {
                                let a = k as f32 * 3.0 + self.anim;
                                let p = self.pos + Vec2::new(a.cos(), a.sin()) * 1.2;
                                spawns.push(Enemy::new(
                                    self.foe,
                                    p.x,
                                    p.y,
                                    depth,
                                    self.biome,
                                    false,
                                    self.seed + k,
                                ));
                            }
                        }
                    }
                }
            }
            Foe::Crab | Foe::Beetle | Foe::Golem => {
                self.charger(dt, world, dirp, dist, fx, shots, depth)
            }
            Foe::Wisp | Foe::Imp => self.shooter(dt, world, dirp, dist, shots, rng),
            Foe::Frog => self.frog(dt, world, dirp, dist, shots, rng),
            Foe::Jelly | Foe::Puffer => self.floater(dt, world, dirp, dist, shots, fx, rng),
        }
        if dist > 0.01 && self.st != St::Dash {
            self.yaw = dirp.x.atan2(dirp.y);
        } else if self.st == St::Dash {
            self.yaw = self.dir.x.atan2(self.dir.y);
        }
    }

    fn idle_wander(&mut self, dt: f32, world: &World, rng: &mut Rng) {
        self.t -= dt;
        if self.t <= 0.0 {
            self.t = rng.range_f(1.0, 2.5);
            let a = rng.f32() * std::f32::consts::TAU;
            self.dir = if rng.chance(0.5) {
                Vec2::new(a.cos(), a.sin())
            } else {
                Vec2::ZERO
            };
        }
        if matches!(self.foe, Foe::Bat | Foe::Wisp | Foe::Jelly | Foe::Puffer) {
            self.y = 0.55 + (self.anim * 4.0).sin() * 0.1;
        }
        if !matches!(self.foe, Foe::Slime | Foe::Frog) {
            let d = self.dir * self.speed * 0.35 * dt;
            self.step(world, d);
            if self.dir.length_squared() > 0.0 {
                self.yaw = self.dir.x.atan2(self.dir.y);
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn slime(
        &mut self,
        dt: f32,
        world: &World,
        dirp: Vec2,
        dist: f32,
        spawns: &mut Vec<Enemy>,
        rng: &mut Rng,
        depth: u32,
    ) {
        let s = self.scale();
        match self.st {
            St::Hop => {
                self.vy -= 16.0 * dt;
                self.y += self.vy * dt;
                let d = self.dir * self.speed * 1.5 * dt;
                self.step(world, d);
                if self.y <= 0.0 {
                    self.y = 0.0;
                    self.vy = 0.0;
                    self.st = St::Idle;
                    self.t = rng.range_f(0.35, 0.9);
                    self.squash = 1.0;
                    self.hops += 1;
                    if self.boss && self.hops % 3 == 0 {
                        for k in 0..2 {
                            let a = rng.f32() * std::f32::consts::TAU;
                            let p = self.pos + Vec2::new(a.cos(), a.sin()) * 1.4;
                            if !world.blocked(p.x as i32, p.y as i32) {
                                spawns.push(Enemy::new(
                                    Foe::Slime,
                                    p.x,
                                    p.y,
                                    depth,
                                    self.biome,
                                    false,
                                    self.seed + k,
                                ));
                            }
                        }
                    }
                }
            }
            _ => {
                if self.t <= 0.0 {
                    self.st = St::Hop;
                    self.vy = 4.6 * s.sqrt();
                    self.dir = if dist < 9.0 { dirp } else { Vec2::ZERO };
                    self.squash = 0.6;
                } else if self.t < 0.2 {
                    self.squash = 0.8;
                }
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn charger(
        &mut self,
        dt: f32,
        world: &World,
        dirp: Vec2,
        dist: f32,
        fx: &mut Fx,
        shots: &mut Vec<Shot>,
        _depth: u32,
    ) {
        let golem = self.foe == Foe::Golem;
        match self.st {
            St::Windup => {
                if self.t <= 0.0 {
                    if golem {
                        // Ground slam: a ring of shards.
                        fx.burst(self.world_pos(), 18, &[SAND, KHAKI, WHITE], 4.0, 2.0);
                        let n = if self.boss { 12 } else { 6 };
                        for k in 0..n {
                            let a = k as f32 / n as f32 * std::f32::consts::TAU + self.anim;
                            shots.push(Shot {
                                pos: self.pos,
                                vel: Vec2::new(a.cos(), a.sin()) * 4.0,
                                dmg: self.dmg * 2 / 3,
                                life: 1.2,
                                color: SKY,
                                radius: 0.18,
                            });
                        }
                        self.st = St::Rest;
                        self.t = 1.0;
                    } else {
                        self.st = St::Dash;
                        self.t = 0.45;
                    }
                }
            }
            St::Dash => {
                let before = self.pos;
                let d = self.dir * 7.5 * dt;
                self.step(world, d);
                if self.t <= 0.0 || (self.pos - before).length() < d.length() * 0.3 {
                    self.st = St::Rest;
                    self.t = 0.8;
                }
            }
            St::Rest => {
                if self.t <= 0.0 {
                    self.st = St::Chase;
                }
            }
            _ => {
                self.dir = dirp;
                let trigger = if golem { 2.0 } else { 4.2 };
                if dist < trigger && world.clear_line(self.pos, self.pos + dirp * dist) {
                    self.st = St::Windup;
                    self.t = if golem { 0.7 } else { 0.5 };
                } else {
                    self.step(world, dirp * self.speed * dt);
                }
            }
        }
    }

    fn shooter(
        &mut self,
        dt: f32,
        world: &World,
        dirp: Vec2,
        dist: f32,
        shots: &mut Vec<Shot>,
        rng: &mut Rng,
    ) {
        if self.foe == Foe::Wisp {
            self.y = 0.55 + (self.anim * 3.0).sin() * 0.1;
        }
        match self.st {
            St::Windup => {
                if self.t <= 0.0 {
                    let color = if self.foe == Foe::Imp { ORANGE } else { AQUA };
                    let spread: &[f32] = if self.boss {
                        &[-0.5, -0.25, 0.0, 0.25, 0.5]
                    } else {
                        &[0.0]
                    };
                    for a in spread {
                        let d = Vec2::from_angle(*a).rotate(dirp);
                        shots.push(Shot {
                            pos: self.pos + d * 0.3,
                            vel: d * 4.6,
                            dmg: self.dmg,
                            life: 2.5,
                            color,
                            radius: 0.16,
                        });
                    }
                    self.st = St::Chase;
                    self.t = rng.range_f(1.6, 2.4);
                }
            }
            _ => {
                let strafe =
                    Vec2::new(-dirp.y, dirp.x) * (self.anim * 0.7 + self.seed as f32).sin();
                let want = if dist < 3.0 {
                    -dirp
                } else if dist > 6.0 {
                    dirp
                } else {
                    strafe
                };
                self.step(world, want * self.speed * dt);
                if self.t <= 0.0 && dist < 9.0 && world.clear_line(self.pos, self.pos + dirp * dist)
                {
                    self.st = St::Windup;
                    self.t = 0.45;
                }
            }
        }
    }

    /// A bog frog: long hops towards you, and now and then a spat bubble.
    fn frog(
        &mut self,
        dt: f32,
        world: &World,
        dirp: Vec2,
        dist: f32,
        shots: &mut Vec<Shot>,
        rng: &mut Rng,
    ) {
        match self.st {
            St::Hop => {
                self.vy -= 18.0 * dt;
                self.y += self.vy * dt;
                let d = self.dir * self.speed * 1.8 * dt;
                self.step(world, d);
                if self.y <= 0.0 {
                    self.y = 0.0;
                    self.vy = 0.0;
                    self.squash = 1.0;
                    self.hops += 1;
                    if dist < 6.5 && rng.chance(0.35) {
                        self.st = St::Windup;
                        self.t = 0.4;
                    } else {
                        self.st = St::Idle;
                        self.t = rng.range_f(0.3, 0.8);
                    }
                }
            }
            St::Windup => {
                self.squash = 0.5;
                if self.t <= 0.0 {
                    shots.push(Shot {
                        pos: self.pos + dirp * 0.3,
                        vel: dirp * 4.2,
                        dmg: self.dmg,
                        life: 2.0,
                        color: SKY,
                        radius: 0.17,
                    });
                    self.st = St::Idle;
                    self.t = rng.range_f(0.5, 1.0);
                }
            }
            _ => {
                if self.t <= 0.0 {
                    self.st = St::Hop;
                    self.vy = 5.4;
                    self.dir = if dist < 10.0 { dirp } else { Vec2::ZERO };
                    self.squash = 0.6;
                }
            }
        }
    }

    /// Drift jellies and puffers bob towards you, then let fly: a ring of sparks, or a
    /// burst of spines once the puffer has blown itself up.
    #[allow(clippy::too_many_arguments)]
    fn floater(
        &mut self,
        dt: f32,
        world: &World,
        dirp: Vec2,
        dist: f32,
        shots: &mut Vec<Shot>,
        fx: &mut Fx,
        rng: &mut Rng,
    ) {
        let jelly = self.foe == Foe::Jelly;
        self.y = 0.55 + (self.anim * if jelly { 2.2 } else { 3.0 }).sin() * 0.08;
        match self.st {
            St::Windup => {
                if !jelly {
                    self.squash = (0.6 - self.t).clamp(0.0, 0.6) / 0.6;
                }
                if self.t <= 0.0 {
                    let n = if self.boss { 14 } else { 8 };
                    let (color, speed, life) = if jelly {
                        (CREAM, 5.0, 0.35)
                    } else {
                        (SAND, 4.6, 0.9)
                    };
                    for k in 0..n {
                        let a = k as f32 / n as f32 * std::f32::consts::TAU + self.anim;
                        shots.push(Shot {
                            pos: self.pos,
                            vel: Vec2::new(a.cos(), a.sin()) * speed,
                            dmg: self.dmg * 2 / 3,
                            life,
                            color,
                            radius: 0.15,
                        });
                    }
                    if jelly {
                        fx.burst(self.world_pos(), 12, &[WHITE, CREAM, AQUA], 3.0, 1.0);
                    }
                    self.st = St::Rest;
                    self.t = if jelly { 1.3 } else { 1.7 };
                }
            }
            St::Rest => {
                self.squash = (self.squash - dt * 2.0).max(0.0);
                if self.t <= 0.0 {
                    self.st = St::Chase;
                }
            }
            _ => {
                let near = if jelly { 1.9 } else { 3.2 };
                let keep = if jelly { 0.8 } else { 2.2 };
                if dist < near && self.t <= 0.0 {
                    self.st = St::Windup;
                    self.t = if jelly { 0.55 } else { 0.6 };
                } else {
                    let want = if dist > keep {
                        dirp
                    } else {
                        Vec2::new(-dirp.y, dirp.x)
                    };
                    let wobble = Vec2::new((self.anim * 1.7).sin(), (self.anim * 1.3).cos()) * 0.3;
                    let d = (want + wobble).normalize_or_zero();
                    self.dir = d;
                    self.step(world, d * self.speed * dt);
                    if self.t <= 0.0 {
                        self.t = rng.range_f(0.2, 0.6);
                    }
                }
            }
        }
    }

    pub fn color(&self) -> u8 {
        match (self.foe, self.biome) {
            (Foe::Slime, 0) => GREEN,
            (Foe::Slime, 1) => BLUE,
            (Foe::Slime, 2) => PINK,
            (Foe::Slime, 3) => ORANGE,
            (Foe::Slime, 4) => WHITE,
            (Foe::Slime, _) => PURPLE,
            (Foe::Bat, _) => PURPLE,
            (Foe::Shroom, _) => RED,
            (Foe::Crab, _) => AQUA,
            (Foe::Wisp, _) => MINT,
            (Foe::Beetle, _) => LAVENDER,
            (Foe::Imp, _) => RED,
            (Foe::Skeleton, _) => SAND,
            (Foe::Golem, _) => KHAKI,
            (Foe::Ghost, _) => BLUSH,
            (Foe::Frog, 2) => PURPLE,
            (Foe::Frog, _) => GREEN,
            (Foe::Jelly, _) => LAVENDER,
            (Foe::Puffer, _) => GOLD,
        }
    }

    pub fn light(&self) -> Option<PointLight> {
        if self.moonlit {
            return Some(PointLight {
                pos: self.world_pos() + Vec3::Y * 0.4,
                radius: 2.4 * self.scale(),
                power: 0.4,
                warmth: 8.0,
            });
        }
        match self.foe {
            Foe::Wisp => Some(PointLight {
                pos: self.world_pos() + Vec3::Y * 0.3,
                radius: 3.5,
                power: 0.6,
                warmth: 2.0,
            }),
            Foe::Imp => Some(PointLight {
                pos: self.world_pos() + Vec3::Y * 0.5,
                radius: 2.5,
                power: 0.35,
                warmth: 8.0,
            }),
            Foe::Jelly => Some(PointLight {
                pos: self.world_pos() + Vec3::Y * 0.2,
                radius: 3.0,
                power: 0.45,
                warmth: 2.2,
            }),
            _ => None,
        }
    }

    /// The full moon's red aura, drawn over everything once the scene is lit.
    pub fn draw_moonglow(&self, r: &mut Renderer) {
        if !self.moonlit {
            return;
        }
        let s = self.scale();
        let pulse = (self.anim * 3.0).sin() * 0.5 + 0.5;
        let at = self.world_pos() + Vec3::Y * (0.35 * s);
        r.halo(at, 0.55 * s + pulse * 0.1, RED, 0.35 + pulse * 0.2);
        // Motes of red light drifting up off them.
        for k in 0..3 {
            let t = (self.anim * 0.8 + k as f32 * 0.33).fract();
            let a = self.anim * 1.3 + k as f32 * 2.1;
            let q = at + Vec3::new(a.cos() * 0.3 * s, t * 0.8 * s, a.sin() * 0.3 * s);
            r.point(q, 1, if k == 0 { SALMON } else { RED });
        }
    }

    pub fn draw(&self, r: &mut Renderer, a: &Assets) {
        let s = self.scale();
        let base = Vec3::new(self.pos.x, 0.0, self.pos.y);
        r.shadow(a.tex(a.disk), base, self.radius * 0.95);
        let mut o = DrawOpts {
            light: Light::At(base),
            tag: if self.moonlit { MOONLIT_TAG } else { 2 },
            ..Default::default()
        };
        if self.flash > 0.0 {
            o.mode = Mode::Solid(WHITE);
        } else if self.st == St::Windup && (self.anim * 20.0).sin() > 0.0 {
            o.glow = 1.3;
        } else if self.burn > 0.0 {
            o.glow = 0.9 + (self.anim * 12.0).sin() * 0.15;
        }
        let b = self.biome.min(a.foes.slime.len() - 1);
        let rot = Mat4::from_rotation_y(self.yaw);
        let at = |y: f32| Mat4::from_translation(Vec3::new(self.pos.x, y, self.pos.y));
        let sc = |x: f32, y: f32| Mat4::from_scale(Vec3::new(x * s, y * s, x * s));
        match self.foe {
            Foe::Slime => {
                let breathe = (self.anim * 4.0).sin() * 0.04;
                let (sx, sy) = if self.st == St::Hop {
                    (0.9, 1.15)
                } else {
                    (
                        1.0 + self.squash * 0.25 + breathe,
                        1.0 - self.squash * 0.3 - breathe,
                    )
                };
                // Jelly wobbles: the top lags behind a shove and quivers after a landing
                // or a hit.
                let side = if self.dir.length_squared() > 0.01 {
                    self.dir.normalize()
                } else {
                    Vec2::new(0.8, 0.6)
                };
                let quiver = (self.anim * 17.0).sin()
                    * (0.012 + self.squash * 0.07 + (self.flash * 3.0).min(0.3));
                let lean = (-self.vel * 0.035 + side * quiver) * s;
                let warp = Warp::Bend {
                    base: self.y,
                    h: 0.45 * s * sy,
                    lean,
                };
                let body = at(self.y) * rot * sc(sx, sy);
                let cols = a.foes.slime_cols[b];
                // The nucleus drifts about inside, and bubbles rise through the jelly.
                let drift = Vec3::new(
                    (self.anim * 1.3).sin() * 0.05,
                    0.2 + (self.anim * 2.1).sin() * 0.025,
                    (self.anim * 1.7).cos() * 0.04,
                );
                let core = at(self.y)
                    * sc(sx, sy)
                    * Mat4::from_translation(drift)
                    * Mat4::from_rotation_y(self.anim * 0.8);
                r.mesh(&a.bank, &a.foes.slime_core[b], &core, &o.with_warp(warp));
                for i in 0..3 {
                    let k = (self.anim * 0.45 + i as f32 * 0.33 + self.seed as f32 * 0.07).fract();
                    let ang = i as f32 * 2.1 + self.seed as f32;
                    let p = Vec3::new(
                        self.pos.x + ang.cos() * 0.17 * s * sx,
                        self.y + (0.06 + k * 0.3) * s * sy,
                        self.pos.y + ang.sin() * 0.17 * s * sx,
                    );
                    r.point(warp.apply(p), 1, if i == 0 { WHITE } else { cols[0] });
                }
                r.mesh(&a.bank, &a.critters.slime_face, &body, &o.with_warp(warp));
                let shell = if self.flash > 0.0 {
                    o
                } else {
                    o.glass(0.36).with_glow(o.glow.max(0.55))
                };
                r.mesh(&a.bank, &a.foes.slime[b], &body, &shell.with_warp(warp));
                // A crisp glint where the dome mirrors the key light.
                let base = Vec3::new(self.pos.x, self.y, self.pos.y);
                let view = (r.cam.eye - base).normalize_or_zero();
                let hv = (crate::render::light::KEY + view).normalize_or_zero();
                let (ra, rb) = (0.42 * s * sx, 0.45 * s * sy);
                let (a2, b2) = (ra * ra, rb * rb);
                let norm = (a2 * hv.x * hv.x + b2 * hv.y * hv.y + a2 * hv.z * hv.z).sqrt();
                let spot =
                    base + Vec3::new(a2 * hv.x, b2 * hv.y, a2 * hv.z) / norm.max(1e-4) + hv * 0.04;
                let spot = warp.apply(spot);
                r.point(spot, if s > 1.0 { 3 } else { 2 }, WHITE);
                r.point(spot + Vec3::new(0.09, -0.07, 0.02) * s, 1, cols[0]);
            }
            Foe::Bat => {
                let m = at(self.y) * rot * sc(1.0, 1.0);
                r.mesh(&a.bank, &a.foes.bat_body[b], &m, &o);
                let flap = (self.anim * 18.0).sin() * 0.7;
                let wo = o.two_sided();
                r.mesh(
                    &a.bank,
                    &a.foes.bat_wing[b],
                    &(m * Mat4::from_translation(Vec3::new(0.1, 0.0, 0.0))
                        * Mat4::from_rotation_z(flap)),
                    &wo,
                );
                r.mesh(
                    &a.bank,
                    &a.foes.bat_wing[b],
                    &(m * Mat4::from_translation(Vec3::new(-0.1, 0.0, 0.0))
                        * Mat4::from_scale(Vec3::new(-1.0, 1.0, 1.0))
                        * Mat4::from_rotation_z(flap)),
                    &wo,
                );
            }
            Foe::Shroom => {
                let waddle = (self.anim * 8.0).sin() * 0.15;
                let hop = (self.anim * 8.0).sin().abs() * 0.08;
                r.mesh(
                    &a.bank,
                    &a.foes.shroom[b],
                    &(at(hop) * rot * Mat4::from_rotation_z(waddle) * sc(1.0, 1.0)),
                    &o,
                );
            }
            Foe::Crab => {
                let shuffle = (self.anim * 10.0).sin() * 0.05;
                let shake = if self.st == St::Windup {
                    (self.anim * 40.0).sin() * 0.04
                } else {
                    0.0
                };
                r.mesh(
                    &a.bank,
                    &a.foes.crab[b],
                    &(at(0.0)
                        * Mat4::from_translation(Vec3::new(shake, shuffle.abs(), 0.0))
                        * rot
                        * sc(1.0, 1.0)),
                    &o,
                );
            }
            Foe::Wisp => {
                let spin = Mat4::from_rotation_y(self.anim * 2.0);
                let wo = DrawOpts {
                    mode: if self.flash > 0.0 {
                        Mode::Solid(WHITE)
                    } else {
                        Mode::Unlit
                    },
                    ..o
                };
                r.mesh(
                    &a.bank,
                    &a.foes.wisp[b],
                    &(at(self.y) * rot * spin * sc(1.0, 1.0)),
                    &wo,
                );
            }
            Foe::Beetle => {
                let shake = if self.st == St::Windup {
                    (self.anim * 40.0).sin() * 0.04
                } else {
                    0.0
                };
                r.mesh(
                    &a.bank,
                    &a.foes.beetle[b],
                    &(at(0.0)
                        * Mat4::from_translation(Vec3::new(shake, 0.0, 0.0))
                        * rot
                        * sc(1.0, 1.0)),
                    &o,
                );
            }
            Foe::Golem => {
                let sway = (self.anim * 2.0).sin() * 0.05;
                let lift = if self.st == St::Windup { 0.2 } else { 0.0 };
                r.mesh(
                    &a.bank,
                    &a.foes.golem[b],
                    &(at(lift) * rot * Mat4::from_rotation_z(sway) * sc(1.0, 1.0)),
                    &o,
                );
            }
            Foe::Ghost => {
                // A wisp of sheet trailing behind as it drifts.
                let m = at(self.y - 0.3) * rot * sc(1.0, 1.0);
                let tail = Vec2::new((self.anim * 3.0).sin(), (self.anim * 2.3).cos()) * 0.05
                    - self.dir * 0.08;
                let warp = Warp::Bend {
                    base: self.y - 0.3 + 0.4 * s,
                    h: -0.4 * s,
                    lean: tail * s,
                };
                r.mesh(&a.bank, &a.critters.ghost_face, &m, &o);
                let sheet = if self.flash > 0.0 {
                    o
                } else {
                    o.glass(0.28).with_glow(o.glow.max(0.8))
                };
                r.mesh(&a.bank, &a.critters.ghost, &m, &sheet.with_warp(warp));
            }
            Foe::Frog => {
                let (sx, sy) = if self.st == St::Hop {
                    (0.85, 1.2)
                } else {
                    (1.0 + self.squash * 0.25, 1.0 - self.squash * 0.3)
                };
                let m = at(self.y) * rot * sc(sx, sy);
                r.mesh(&a.bank, &a.foes.frog[b], &m, &o);
                // Legs trail out behind on a jump.
                let kick = if self.st == St::Hop { 0.5 } else { 0.0 };
                for side in [-1.0f32, 1.0] {
                    let leg = m
                        * Mat4::from_translation(Vec3::new(side * 0.2, 0.08, -0.12))
                        * Mat4::from_rotation_x(kick);
                    r.mesh(&a.bank, &a.foes.frog_leg[b], &leg, &o);
                }
            }
            Foe::Jelly => {
                // A glowing bell, and threads that stream behind it.
                let pulse = 1.0 + (self.anim * 3.0).sin() * 0.06;
                let m = at(self.y) * rot * sc(pulse, 1.0 / pulse);
                let trail = -self.dir * 0.12
                    + Vec2::new((self.anim * 2.3).sin(), (self.anim * 1.9).cos()) * 0.05;
                let warp = Warp::Bend {
                    base: self.y,
                    h: -0.45 * s,
                    lean: trail * s,
                };
                let lit = if self.flash > 0.0 {
                    o
                } else {
                    o.with_glow(o.glow.max(0.9))
                };
                r.mesh(&a.bank, &a.critters.jelly_core, &m, &lit.with_warp(warp));
                r.mesh(
                    &a.bank,
                    &a.critters.jelly_threads,
                    &m,
                    &lit.two_sided().with_warp(warp),
                );
                let bell = if self.flash > 0.0 {
                    o
                } else {
                    o.glass(0.3).with_glow(o.glow.max(0.8))
                };
                r.mesh(&a.bank, &a.critters.jelly_bell, &m, &bell);
                if self.st == St::Windup && (self.anim * 30.0).sin() > 0.0 {
                    r.halo(self.world_pos() + Vec3::Y * 0.1, 0.5 * s, CREAM, 0.7);
                }
            }
            Foe::Puffer => {
                // Blows itself up into a spiky ball before letting fly.
                let puff = 1.0 + self.squash * 0.55;
                let m = at(self.y) * rot * sc(puff, puff);
                r.mesh(&a.bank, &a.foes.puffer[b], &m, &o);
                if self.squash > 0.2 {
                    r.mesh(&a.bank, &a.critters.puffer_spikes, &m, &o);
                }
                let flap = (self.anim * 14.0).sin() * 0.5;
                for side in [-1.0f32, 1.0] {
                    let fin = m
                        * Mat4::from_translation(Vec3::new(side * 0.22, 0.0, 0.0))
                        * Mat4::from_rotation_y(side * (0.4 + flap));
                    r.mesh(&a.bank, &a.critters.puffer_fin, &fin, &o.two_sided());
                }
            }
            Foe::Imp | Foe::Skeleton => {
                let h = if self.foe == Foe::Imp {
                    &a.critters.imp
                } else {
                    &a.critters.skeleton
                };
                let moving = self.alert && self.st != St::Windup;
                let pose = Pose {
                    walk: self.anim * 10.0,
                    stride: if moving { 0.8 } else { 0.0 },
                    swing: if self.st == St::Windup || self.st == St::Dash {
                        Some((
                            1.0 - self.t.clamp(0.0, 0.4) / 0.4,
                            super::draw::Swing::Slash,
                        ))
                    } else {
                        None
                    },
                    ..Default::default()
                };
                let mut ho = o;
                if self.boss {
                    ho.glow = 0.2;
                }
                let root = Vec3::new(self.pos.x, 0.0, self.pos.y);
                if self.boss {
                    // Bosses are drawn at double size.
                    draw_humanoid_scaled(r, a, h, root, self.yaw, &pose, &ho, s);
                } else {
                    draw_humanoid(r, a, h, root, self.yaw, &pose, &ho, &Default::default());
                }
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn draw_humanoid_scaled(
    r: &mut Renderer,
    a: &Assets,
    h: &crate::assets::models::Humanoid,
    pos: Vec3,
    yaw: f32,
    pose: &Pose,
    o: &DrawOpts,
    s: f32,
) {
    // Scale the parts by baking a scaled copy of each pivot offset.
    let mut big = h.clone();
    for p in big.parts.iter_mut() {
        p.transform(Mat4::from_scale(Vec3::splat(s)));
    }
    big.hip *= s;
    big.shoulder *= s;
    big.neck *= s;
    big.shoulder_x *= s;
    big.hip_x *= s;
    draw_humanoid(r, a, &big, pos, yaw, pose, o, &Default::default());
}
