//! The farm's pets: the cat, and the jumping spider that hatches from Pip's mysterious egg.
//!
//! The cat strolls about, sits, naps by the house at night and loves a fuss. Once there's a
//! spider, the two of them play: the cat stalks it, wiggles and pounces, the spider springs
//! out of the way (or onto the cat's back for a ride), and they curl up together at night.
//!
//! The egg sits in its nest for ten days: on the ninth it tips over, and on the tenth it
//! hatches as soon as you come near.

use std::f32::consts::{PI, TAU};

use glam::{Mat4, Vec2, Vec3};

use super::Io;
use super::play::{Banner, Play, tile_center};
use super::world::{Area, Obj, World};
use crate::assets::Assets;
use crate::assets::pet_art::*;
use crate::audio::Sfx;
use crate::palette::*;
use crate::render::{DrawOpts, Renderer};
use crate::util::{Rng, hash2, wrap_angle};

/// Where the cat likes to be: its spot by the farmhouse.
pub const CAT_HOME: Vec2 = Vec2::new(27.0, 12.0);
/// Days from setting the egg down to it hatching; it tips over the day before.
pub const HATCH_DAYS: u32 = 10;
pub const TILT_DAY: u32 = 9;
/// On the day, it hatches once you're this close.
const HATCH_NEAR: f32 = 6.0;
/// Seconds of shaking before it pops.
const HATCH_POP: f32 = 1.8;
/// The pounce: how long it's in the air and how high.
const POUNCE: f32 = 0.45;
/// The spider's size (its parts are modelled a little larger than this).
const SPIDER_SCALE: f32 = 0.9;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum CatMode {
    /// Wandering about, or off to see you.
    Stroll,
    /// Sitting with its tail round its paws.
    Sit,
    /// Low to the ground, creeping up on the spider.
    Stalk,
    /// Wiggling its rump, about to spring.
    Wiggle,
    /// In the air.
    Pounce { from: Vec2, to: Vec2 },
    /// Swatting with a forepaw.
    Bat,
    /// Curled up asleep.
    Nap,
}

#[derive(Clone, Copy, Debug)]
pub struct Cat {
    pub pos: Vec2,
    pub target: Vec2,
    /// Seconds before it makes up its mind again.
    pub t: f32,
    pub yaw: f32,
    /// Seconds left of being petted.
    pub pet: f32,
    pub mode: CatMode,
    /// Seconds in the current mode.
    pub mode_t: f32,
    /// The walk cycle, and how fast it's going.
    pub stride: f32,
    pub speed: f32,
    /// Height off the ground (pouncing).
    pub y: f32,
    /// Seconds until the next "z" while it sleeps.
    pub snore: f32,
}

impl Cat {
    pub fn new(pos: Vec2) -> Cat {
        Cat {
            pos,
            target: pos,
            t: 0.0,
            yaw: 0.0,
            pet: 0.0,
            mode: CatMode::Sit,
            mode_t: 0.0,
            stride: 0.0,
            speed: 0.0,
            y: 0.0,
            snore: 1.0,
        }
    }

    fn set(&mut self, mode: CatMode, secs: f32) {
        self.mode = mode;
        self.mode_t = 0.0;
        self.t = secs;
    }

    /// Sitting, sleeping or curled up: somewhere a spider can sit on its head.
    fn still(&self) -> bool {
        self.pet > 0.0 || matches!(self.mode, CatMode::Sit | CatMode::Nap | CatMode::Bat)
    }
}

/// How the cat holds itself this moment (see `Cat::pose`).
#[derive(Clone, Copy, Default)]
struct CatPose {
    /// Body raised off the ground (a pounce), and lowered at the hips (sitting, crouching).
    lift: f32,
    drop: f32,
    /// The body's front raised about the hips.
    pitch: f32,
    /// Nose up (+) or down, and turned to the side.
    nod: f32,
    turn: f32,
    /// Each leg swung forwards (front left, front right, back left, back right), and
    /// whether it stretches to reach the ground.
    swing: [f32; 4],
    reach: [bool; 4],
    /// The tail's rise off the back, its curl along its length, and its sway.
    rise: f32,
    curl: f32,
    sway: f32,
    /// Extra swish at the tip.
    flick: f32,
    shut: bool,
    stretch: f32,
    /// A side-to-side wiggle of the whole body.
    wiggle: f32,
}

impl Cat {
    fn pose(&self, time: f32) -> CatPose {
        let t = time + self.pos.x * 0.37;
        let sit = CatPose {
            drop: 0.07,
            pitch: 0.72,
            nod: 0.05,
            swing: [0.0, 0.0, 1.25, 1.25],
            reach: [true, true, false, false],
            rise: -1.25,
            curl: 0.05,
            sway: 0.95,
            flick: (t * 1.7).sin() * 0.25,
            stretch: 1.0,
            ..Default::default()
        };
        if self.pet > 0.0 {
            // Eyes closed, chin up, tail high and quivering.
            return CatPose {
                nod: 0.55,
                shut: true,
                rise: 0.1,
                curl: 0.35,
                sway: (t * 18.0).sin() * 0.08,
                flick: 0.0,
                ..sit
            };
        }
        match self.mode {
            CatMode::Sit => CatPose {
                turn: (t * 0.4).sin() * 0.35,
                ..sit
            },
            CatMode::Bat => {
                // Up on its haunches, one paw swiping.
                let mut p = CatPose {
                    nod: -0.1,
                    flick: (t * 6.0).sin() * 0.5,
                    ..sit
                };
                p.swing[0] = 1.0 + (self.mode_t * 16.0).sin() * 0.55;
                p.reach[0] = false;
                p
            }
            CatMode::Nap => CatPose {
                drop: 0.1,
                pitch: 0.0,
                nod: -0.12,
                turn: 0.5,
                swing: [1.5, 1.5, 1.45, 1.45],
                reach: [false; 4],
                rise: -0.25,
                curl: 0.1,
                sway: 1.25,
                flick: (t * 0.8).sin() * 0.1,
                shut: true,
                stretch: 1.0,
                ..Default::default()
            },
            CatMode::Stalk | CatMode::Wiggle => {
                let s = if self.mode == CatMode::Stalk {
                    self.stride.sin() * 0.3
                } else {
                    0.0
                };
                CatPose {
                    drop: 0.075,
                    pitch: -0.12,
                    nod: 0.1,
                    swing: [s, -s, -s, s],
                    reach: [true; 4],
                    rise: 0.05,
                    curl: 0.02,
                    sway: 0.0,
                    flick: (t * 9.0).sin() * 0.6,
                    stretch: 1.0,
                    wiggle: if self.mode == CatMode::Wiggle {
                        (self.mode_t * 28.0).sin() * 0.16
                    } else {
                        0.0
                    },
                    ..Default::default()
                }
            }
            CatMode::Pounce { .. } => {
                let k = (self.mode_t / POUNCE).min(1.0);
                CatPose {
                    lift: self.y,
                    pitch: 0.35 - k * 0.7,
                    nod: 0.0,
                    swing: [1.0, 1.0, -1.0, -1.0],
                    reach: [false; 4],
                    rise: 0.3,
                    curl: 0.0,
                    stretch: 1.14,
                    ..Default::default()
                }
            }
            CatMode::Stroll => {
                let moving = self.speed > 0.05;
                let s = if moving { self.stride.sin() * 0.5 } else { 0.0 };
                CatPose {
                    lift: if moving {
                        self.stride.sin().abs() * 0.01
                    } else {
                        0.0
                    },
                    nod: if moving {
                        (self.stride * 2.0).sin() * 0.04
                    } else {
                        0.1
                    },
                    swing: [s, -s, -s, s],
                    reach: [true; 4],
                    rise: 1.05,
                    curl: 0.3,
                    sway: (t * 2.2).sin() * 0.25,
                    flick: (t * 3.1).sin() * 0.2,
                    stretch: 1.0,
                    ..Default::default()
                }
            }
        }
    }

    /// Its body and head, placed in the world for a pose.
    fn frames(&self, p: &CatPose) -> (Mat4, Mat4) {
        let root = Mat4::from_translation(Vec3::new(self.pos.x, 0.0, self.pos.y))
            * Mat4::from_rotation_y(self.yaw + p.wiggle);
        let body = root
            * Mat4::from_translation(Vec3::new(0.0, CAT_BODY_Y - p.drop + p.lift, 0.0))
            * Mat4::from_translation(Vec3::new(0.0, 0.0, -0.15))
            * Mat4::from_rotation_x(-p.pitch)
            * Mat4::from_translation(Vec3::new(0.0, 0.0, 0.15));
        let head = body
            * Mat4::from_translation(CAT_NECK * Vec3::new(1.0, 1.0, p.stretch))
            * Mat4::from_rotation_x(p.pitch - p.nod)
            * Mat4::from_rotation_y(p.turn);
        (body, head)
    }

    /// Where a spider rides: up between its shoulders when it's sitting or asleep, the
    /// middle of its back otherwise.
    pub fn perch(&self, time: f32) -> Vec3 {
        let p = self.pose(time);
        let (body, _) = self.frames(&p);
        let spot = if self.still() {
            Vec3::new(0.0, 0.09, 0.06)
        } else {
            Vec3::new(0.0, 0.085, -0.04)
        };
        body.transform_point3(spot)
    }
}

// ------------------------------------------------------------------------------------------
// The jumping spider
// ------------------------------------------------------------------------------------------

/// A hop through the air.
#[derive(Clone, Copy, Debug)]
pub struct Hop {
    pub from: Vec3,
    pub to: Vec3,
    pub t: f32,
    pub dur: f32,
    pub h: f32,
    /// Landing on the cat (which may have moved by then).
    pub onto_cat: bool,
}

#[derive(Clone, Copy, Debug)]
pub struct Spider {
    pub pos: Vec2,
    /// Height off the ground (hopping, riding).
    pub y: f32,
    pub yaw: f32,
    pub hop: Option<Hop>,
    /// Seconds before it next moves, and before it next snaps round to look.
    pub rest: f32,
    pub glance: f32,
    /// Seconds left riding the cat.
    pub ride: f32,
    /// Seconds left of waving its front legs.
    pub wave: f32,
    /// Seconds before it can be startled again.
    pub startle: f32,
    /// A squash on landing.
    pub land: f32,
    pub t: f32,
}

impl Spider {
    pub fn new(pos: Vec2) -> Spider {
        Spider {
            pos,
            y: 0.0,
            yaw: 0.0,
            hop: None,
            rest: 1.0,
            glance: 0.5,
            ride: 0.0,
            wave: 0.0,
            startle: 0.0,
            land: 0.0,
            t: 0.0,
        }
    }

    pub fn riding(&self) -> bool {
        self.ride > 0.0 || self.hop.is_some_and(|h| h.onto_cat)
    }

    pub fn world_pos(&self) -> Vec3 {
        Vec3::new(self.pos.x, self.y, self.pos.y)
    }

    /// Springs towards `to` on the ground, if it can land there.
    fn hop_to(&mut self, farm: &World, to: Vec2, h: f32, dur: f32) -> bool {
        if farm.blocked(to.x.floor() as i32, to.y.floor() as i32) {
            return false;
        }
        let d = to - self.pos;
        if d.length() > 0.05 {
            self.yaw = d.x.atan2(d.y);
        }
        self.hop = Some(Hop {
            from: self.world_pos(),
            to: Vec3::new(to.x, 0.0, to.y),
            t: 0.0,
            dur,
            h,
            onto_cat: false,
        });
        self.ride = 0.0;
        true
    }

    /// Springs up onto the cat.
    fn hop_onto(&mut self, perch: Vec3) {
        let d = Vec2::new(perch.x, perch.z) - self.pos;
        if d.length() > 0.05 {
            self.yaw = d.x.atan2(d.y);
        }
        self.hop = Some(Hop {
            from: self.world_pos(),
            to: perch,
            t: 0.0,
            dur: 0.45,
            h: 0.35,
            onto_cat: true,
        });
    }

    /// A hop off somewhere nearby, trying a few directions.
    fn hop_about(&mut self, farm: &World, rng: &mut Rng, toward: Option<Vec2>, far: f32) {
        for _ in 0..6 {
            let dir = match toward {
                Some(t) if (t - self.pos).length() > 0.3 => {
                    let d = (t - self.pos).normalize();
                    let a = rng.range_f(-0.6, 0.6);
                    Vec2::new(d.x * a.cos() - d.y * a.sin(), d.x * a.sin() + d.y * a.cos())
                }
                _ => {
                    let a = rng.f32() * TAU;
                    Vec2::new(a.cos(), a.sin())
                }
            };
            let len = rng.range_f(far * 0.5, far);
            let h = 0.12 + len * 0.18;
            if self.hop_to(farm, self.pos + dir * len, h, 0.24 + len * 0.1) {
                return;
            }
        }
        self.rest = 0.5;
    }
}

/// An egg on its way to hatching.
#[derive(Clone, Copy, Debug)]
pub struct Hatch {
    pub x: i32,
    pub z: i32,
    pub t: f32,
}

impl Play {
    // --------------------------------------------------------------------------------------
    // The cat
    // --------------------------------------------------------------------------------------

    pub fn update_cat(&mut self, dt: f32) {
        if self.area != Area::Farm {
            return;
        }
        let night = self.clock.min >= 1260.0 || self.clock.min < 400.0;
        let spider = self.spider.map(|s| (s.pos, s.riding()));
        let mut c = self.cat;
        c.t -= dt;
        c.mode_t += dt;
        c.pet = (c.pet - dt).max(0.0);
        c.speed = 0.0;
        if c.mode == CatMode::Nap {
            c.snore -= dt;
            if c.snore <= 0.0 {
                c.snore = 2.4;
                let (_, head) = c.frames(&c.pose(self.time));
                let at = head.transform_point3(Vec3::new(0.05, 0.2, 0.0));
                self.fx.popup(at, "z", SKY);
            }
        }
        if c.pet > 0.0 {
            self.cat = c;
            return;
        }
        match c.mode {
            CatMode::Pounce { from, to } => {
                let k = (c.mode_t / POUNCE).min(1.0);
                c.pos = from.lerp(to, k);
                c.y = (k * PI).sin() * 0.28;
                if k >= 1.0 {
                    c.y = 0.0;
                    if self.rng.chance(0.5) {
                        c.set(CatMode::Bat, self.rng.range_f(0.8, 1.4));
                    } else {
                        c.set(CatMode::Sit, self.rng.range_f(1.5, 3.0));
                    }
                }
                self.cat = c;
                return;
            }
            CatMode::Wiggle => {
                if let Some((sp, _)) = spider {
                    let d = sp - c.pos;
                    if d.length() > 0.05 {
                        c.yaw = d.x.atan2(d.y);
                    }
                }
                if c.mode_t > 0.9 {
                    // Spring at where the spider is (or was), not too far and not into a
                    // wall.
                    let aim = spider.map_or(c.pos, |s| s.0);
                    let d = aim - c.pos;
                    let to = c.pos + d.clamp_length_max(1.5);
                    let to = if self.farm.blocked(to.x.floor() as i32, to.y.floor() as i32) {
                        c.pos
                    } else {
                        to
                    };
                    c.mode = CatMode::Pounce { from: c.pos, to };
                    c.mode_t = 0.0;
                }
                self.cat = c;
                return;
            }
            _ => {}
        }
        if c.t <= 0.0 {
            self.decide_cat(&mut c, night, spider);
        }
        // Stalking follows the spider; giving up once it's out of reach (or on its back).
        if c.mode == CatMode::Stalk {
            match spider {
                Some((sp, false)) => {
                    c.target = sp;
                    if (sp - c.pos).length() < 1.3 {
                        c.set(CatMode::Wiggle, 2.0);
                    }
                }
                _ => c.set(CatMode::Sit, 2.0),
            }
        }
        if c.mode == CatMode::Bat {
            if let Some((sp, _)) = spider {
                let d = sp - c.pos;
                if d.length() > 0.05 {
                    c.yaw = d.x.atan2(d.y);
                }
            }
        }
        if matches!(c.mode, CatMode::Stroll | CatMode::Stalk) {
            let pace = if c.mode == CatMode::Stalk { 0.55 } else { 1.2 };
            let d = c.target - c.pos;
            if d.length() > 0.2 {
                let step = d.normalize() * pace * dt;
                let before = c.pos;
                c.pos = self.farm.move_circle(c.pos, step, 0.2);
                let moved = (c.pos - before).length();
                c.speed = moved / dt.max(1e-4);
                c.stride += moved * 14.0;
                let want = step.x.atan2(step.y);
                c.yaw += wrap_angle(want - c.yaw) * (dt * 10.0).min(1.0);
                if moved < pace * dt * 0.2 {
                    // Stuck on something: think again.
                    c.t = c.t.min(0.2);
                }
            } else if c.mode == CatMode::Stroll && c.t > 0.6 {
                c.t = 0.6;
            }
        }
        self.cat = c;
    }

    fn decide_cat(&mut self, c: &mut Cat, night: bool, spider: Option<(Vec2, bool)>) {
        let r = &mut self.rng;
        if night {
            // Off to its spot by the house for the night.
            if (c.pos - CAT_HOME).length() > 0.7 {
                c.target = CAT_HOME;
                c.set(CatMode::Stroll, 4.0);
            } else {
                c.set(CatMode::Nap, 10.0);
            }
            return;
        }
        if let Some((sp, false)) = spider {
            if (sp - c.pos).length() < 5.0 && r.chance(0.55) {
                c.target = sp;
                c.set(CatMode::Stalk, 6.0);
                return;
            }
        }
        let k = r.f32();
        if k < 0.1 {
            c.set(CatMode::Nap, r.range_f(6.0, 12.0));
        } else if k < 0.38 {
            c.set(CatMode::Sit, r.range_f(2.5, 5.0));
        } else {
            let base = if r.chance(0.45) {
                self.player.pos
            } else {
                CAT_HOME
            };
            c.target = base + Vec2::new(r.range_f(-3.0, 3.0), r.range_f(-2.0, 2.0));
            c.set(CatMode::Stroll, r.range_f(2.0, 5.0));
        }
    }

    // --------------------------------------------------------------------------------------
    // The spider
    // --------------------------------------------------------------------------------------

    pub fn update_spider(&mut self, dt: f32) {
        if self.area != Area::Farm {
            return;
        }
        let Some(mut s) = self.spider else { return };
        let cat = self.cat;
        let perch = cat.perch(self.time);
        s.t += dt;
        s.wave = (s.wave - dt).max(0.0);
        s.startle = (s.startle - dt).max(0.0);
        s.land = (s.land - dt * 4.0).max(0.0);
        let night = self.clock.min >= 1260.0 || self.clock.min < 400.0;
        if let Some(mut h) = s.hop {
            if h.onto_cat {
                h.to = perch;
            }
            h.t += dt;
            let k = (h.t / h.dur).min(1.0);
            let p = h.from.lerp(h.to, k);
            s.pos = Vec2::new(p.x, p.z);
            s.y = p.y + (k * PI).sin() * h.h;
            if k >= 1.0 {
                s.hop = None;
                s.land = 1.0;
                s.y = h.to.y;
                if h.onto_cat {
                    s.ride = if night {
                        30.0
                    } else {
                        self.rng.range_f(6.0, 14.0)
                    };
                }
                s.rest = self.rng.range_f(0.4, 1.4);
            } else {
                s.hop = Some(h);
            }
            self.spider = Some(s);
            return;
        }
        if s.ride > 0.0 {
            // Along for the ride, facing where the cat faces.
            s.pos = Vec2::new(perch.x, perch.z);
            s.y = perch.y;
            s.yaw = cat.yaw;
            s.ride -= dt;
            let napping = cat.mode == CatMode::Nap;
            if night && napping {
                s.ride = s.ride.max(1.0);
            }
            if s.ride <= 0.0 || matches!(cat.mode, CatMode::Pounce { .. }) {
                s.ride = 0.0;
                let a = self.rng.f32() * TAU;
                let to = s.pos + Vec2::new(a.cos(), a.sin()) * 0.8;
                if !s.hop_to(&self.farm, to, 0.3, 0.4) {
                    s.hop_to(&self.farm, cat.pos, 0.3, 0.4);
                }
            }
            self.spider = Some(s);
            return;
        }
        // The cat's about to pounce: spring clear, or right up onto its back.
        let to_cat = cat.pos - s.pos;
        if cat.mode == CatMode::Wiggle && to_cat.length() < 1.8 && s.startle <= 0.0 {
            s.startle = 2.0;
            if self.rng.chance(0.45) {
                s.hop_onto(perch);
            } else {
                let away = (-to_cat).normalize_or(Vec2::X);
                s.hop_about(&self.farm, &mut self.rng, Some(s.pos + away * 3.0), 1.7);
                if let Some(h) = &mut s.hop {
                    h.h = 0.45;
                }
            }
            self.spider = Some(s);
            return;
        }
        // Looking round in quick little snaps, at you if you're near, else at the cat.
        let me = self.player.pos - s.pos;
        let look = if me.length() < 3.0 { me } else { to_cat };
        s.glance -= dt;
        if s.glance <= 0.0 && look.length() > 0.1 {
            s.glance = self.rng.range_f(0.25, 0.7);
            let want = look.x.atan2(look.y);
            let d = wrap_angle(want - s.yaw);
            if d.abs() > 0.25 {
                s.yaw += d.clamp(-0.9, 0.9);
            }
        }
        s.rest -= dt;
        if s.rest <= 0.0 {
            s.rest = self.rng.range_f(0.5, 1.6);
            let near_me = me.length() < 2.5;
            if near_me && self.rng.chance(0.3) {
                // Hello!
                s.wave = 1.4;
                s.yaw = me.x.atan2(me.y);
            } else if (night || cat.mode == CatMode::Nap) && cat.still() {
                // Bedtime: up onto the cat to sleep.
                if to_cat.length() < 1.2 {
                    s.hop_onto(perch);
                } else {
                    s.hop_about(&self.farm, &mut self.rng, Some(cat.pos), 1.1);
                }
            } else if to_cat.length() > 1.0 && self.rng.chance(0.65) {
                // Off to play with the cat.
                s.hop_about(&self.farm, &mut self.rng, Some(cat.pos), 1.2);
            } else if cat.still() && to_cat.length() < 1.2 && self.rng.chance(0.3) {
                s.hop_onto(perch);
            } else {
                s.hop_about(&self.farm, &mut self.rng, None, 1.0);
            }
        }
        self.spider = Some(s);
    }

    /// Pets whichever pet is close enough. Returns true if one was.
    pub fn pet_nearby(&mut self, io: &mut Io) -> bool {
        if self.area != Area::Farm {
            return false;
        }
        let me = self.player.pos;
        let cat = (self.cat.pos - me).length();
        let spider = self.spider.map_or(f32::MAX, |s| (s.pos - me).length());
        if spider < 1.2 && spider <= cat {
            if let Some(s) = &mut self.spider {
                s.wave = 1.8;
                let d = me - s.pos;
                if s.ride <= 0.0 && s.hop.is_none() {
                    s.yaw = d.x.atan2(d.y);
                }
                let at = s.world_pos() + Vec3::Y * 0.4;
                self.fx.motes(at, 5, &[PINK, BLUSH, WHITE], 0.15);
                self.fx.popup(at + Vec3::Y * 0.2, "♥", PINK);
                io.audio.play_at(Sfx::Pickup, 0.6, 1.8);
            }
            return true;
        }
        if cat < 1.3 {
            self.cat.pet = 1.5;
            let at = Vec3::new(self.cat.pos.x, 0.5, self.cat.pos.y);
            self.fx.motes(at, 5, &[PINK, BLUSH], 0.2);
            self.fx.popup(at + Vec3::Y * 0.4, "♥", PINK);
            io.audio.play_at(Sfx::Pickup, 0.7, 1.3);
            // A spider riding along says hello too.
            if let Some(s) = &mut self.spider {
                if s.ride > 0.0 {
                    s.wave = 1.5;
                }
            }
            return true;
        }
        false
    }

    // --------------------------------------------------------------------------------------
    // The egg
    // --------------------------------------------------------------------------------------

    /// Counts the egg's days, and hatches it once it's time and you're close.
    pub fn update_egg(&mut self, dt: f32, io: &mut Io) {
        if self.area != Area::Farm {
            self.hatching = None;
            return;
        }
        if let Some(mut h) = self.hatching {
            let before = h.t;
            h.t += dt;
            let c = tile_center(h.x, h.z);
            // Taps from inside, and chips of shell flying.
            if (before * 2.5).floor() != (h.t * 2.5).floor() && h.t < HATCH_POP {
                io.audio.play_at(Sfx::Hop, 0.6, 1.6 + h.t * 0.3);
                self.fx
                    .burst(c + Vec3::Y * 0.3, 3, &[LAVENDER, BLUSH, WHITE], 1.2, 1.6);
            }
            if h.t >= HATCH_POP {
                self.hatching = None;
                self.hatch(h.x, h.z, io);
            } else {
                self.hatching = Some(h);
            }
            return;
        }
        // Only an egg near you, on the day.
        let (px, pz) = self.player.tile();
        let r = HATCH_NEAR.ceil() as i32;
        let day = self.clock.day;
        for z in pz - r..=pz + r {
            for x in px - r..=px + r {
                if let Some(Obj::Egg { laid }) = self.farm.obj(x, z) {
                    let near = (tile_center(x, z) - self.player.world_pos()).length();
                    if day.saturating_sub(*laid) >= HATCH_DAYS && near <= HATCH_NEAR {
                        self.hatching = Some(Hatch { x, z, t: 0.0 });
                        return;
                    }
                }
            }
        }
    }

    /// Pop! Out comes the jumping spider.
    fn hatch(&mut self, x: i32, z: i32, io: &mut Io) {
        self.farm.set_obj(x, z, None);
        let c = tile_center(x, z);
        io.audio.play_at(Sfx::Break, 0.7, 1.8);
        io.audio.play(Sfx::Discover);
        self.fx.burst(
            c + Vec3::Y * 0.3,
            24,
            &[LAVENDER, PURPLE, BLUSH, WHITE, MINT],
            2.4,
            2.6,
        );
        self.fx.smoke(c + Vec3::Y * 0.25, 6, 0.3, 0.4);
        self.fx
            .motes(c + Vec3::Y * 0.3, 16, &[PINK, WHITE, MINT, GOLD], 0.4);
        self.shake = self.shake.max(0.3);
        let mut s = Spider::new(Vec2::new(c.x, c.z));
        // Straight up out of the shell with a little hop, then a wave hello.
        s.hop = Some(Hop {
            from: Vec3::new(c.x, 0.15, c.z),
            to: Vec3::new(c.x, 0.0, c.z + 0.3),
            t: 0.0,
            dur: 0.5,
            h: 0.4,
            onto_cat: false,
        });
        s.wave = 3.0;
        s.rest = 2.5;
        let me = self.player.pos - s.pos;
        s.yaw = me.x.atan2(me.y);
        self.spider = Some(s);
        self.banner = Some(Banner {
            title: "The egg hatched!".into(),
            sub: "A baby jumping spider! Go and say hello".into(),
            t: 0.0,
        });
        self.toast_colored(
            "It's waving at you! (It wants to meet the cat.)",
            None,
            0,
            PINK,
        );
    }

    /// Listening to the egg: how it's getting on.
    pub fn listen_to_egg(&mut self, laid: u32, io: &mut Io) {
        let age = self.clock.day.saturating_sub(laid);
        let line = egg_line(age);
        io.audio.play_at(Sfx::Hop, 0.4, 1.3);
        self.toast_colored(line, None, 0, LAVENDER);
    }

    /// Morning news about the egg, for the day's summary.
    pub fn egg_news(&self) -> Option<(String, u8)> {
        let laid = self.farm.objs.iter().find_map(|o| match o {
            Some(Obj::Egg { laid }) => Some(*laid),
            _ => None,
        })?;
        let age = self.clock.day.saturating_sub(laid);
        Some(match age {
            a if a >= HATCH_DAYS => (
                "Something is tapping its way out of the egg!".into(),
                CRIMSON,
            ),
            TILT_DAY => (
                "The egg tipped over in the night... it's nearly time!".into(),
                PURPLE,
            ),
            a => (
                format!("The mysterious egg hums in its nest (day {a} of {HATCH_DAYS})."),
                PURPLE,
            ),
        })
    }

    // --------------------------------------------------------------------------------------
    // Drawing
    // --------------------------------------------------------------------------------------

    /// The cat, the spider and any egg in view.
    pub fn draw_pets(&self, r: &mut Renderer, a: &Assets) {
        if self.area != Area::Farm {
            return;
        }
        self.draw_cat(r, a);
        if let Some(s) = &self.spider {
            self.draw_spider(r, a, s);
        }
        self.draw_eggs(r, a);
    }

    fn draw_cat(&self, r: &mut Renderer, a: &Assets) {
        let c = &self.cat;
        let pa = &a.pets;
        let base = Vec3::new(c.pos.x, 0.0, c.pos.y);
        r.shadow(a.tex(a.disk), base, 0.26);
        let o = DrawOpts::at(base + Vec3::Y * 0.2).with_tag(1);
        let p = c.pose(self.time);
        let (body, head) = c.frames(&p);
        r.mesh(
            &a.bank,
            &pa.cat_body,
            &(body * Mat4::from_scale(Vec3::new(1.0, 1.0, p.stretch))),
            &o,
        );
        r.mesh(&a.bank, &pa.cat_head, &head, &o);
        let eyes = if p.shut {
            &pa.cat_eyes_shut
        } else {
            &pa.cat_eyes
        };
        r.mesh(&a.bank, eyes, &head, &o);
        let yaw = Mat4::from_rotation_y(c.yaw + p.wiggle);
        for (k, hip) in CAT_HIPS.iter().enumerate() {
            let at = body.transform_point3(*hip * Vec3::new(1.0, 1.0, p.stretch));
            let reach = if p.reach[k] {
                (at.y / CAT_LEG).clamp(0.25, 1.7)
            } else {
                1.0
            };
            let m = Mat4::from_translation(at)
                * yaw
                * Mat4::from_rotation_x(p.swing[k])
                * Mat4::from_scale(Vec3::new(1.0, reach, 1.0));
            r.mesh(&a.bank, &pa.cat_leg, &m, &o);
        }
        // The tail, segment by segment: up off the back, curling at the end.
        let mut m = body
            * Mat4::from_translation(CAT_TAIL_BASE * Vec3::new(1.0, 1.0, p.stretch))
            * Mat4::from_rotation_y(p.sway * 0.5)
            * Mat4::from_rotation_x(p.rise);
        let n = pa.cat_tail.len();
        for (i, seg) in pa.cat_tail.iter().enumerate() {
            r.mesh(&a.bank, seg, &m, &o.two_sided());
            let tip = i as f32 / (n - 1) as f32;
            m = m
                * Mat4::from_translation(Vec3::new(0.0, 0.0, -CAT_TAIL_SEG))
                * Mat4::from_rotation_x(p.curl * (0.6 + tip))
                * Mat4::from_rotation_y(p.sway * 0.2 + p.flick * tip * 0.6);
        }
        // Whiskers, by daylight.
        if self.env().night < 0.5 && !r.shadow_pass {
            for s in [-1.0f32, 1.0] {
                for dy in [-0.03f32, -0.058] {
                    let from = head.transform_point3(Vec3::new(s * 0.06, -0.045, 0.1));
                    let to = head.transform_point3(Vec3::new(s * 0.15, dy, 0.085));
                    for k in 1..=3 {
                        r.point(from.lerp(to, k as f32 / 3.0), 1, WHITE);
                    }
                }
            }
        }
    }

    fn draw_spider(&self, r: &mut Renderer, a: &Assets, s: &Spider) {
        let pa = &a.pets;
        let at = s.world_pos();
        if s.y < 0.05 {
            r.shadow(a.tex(a.disk), Vec3::new(at.x, 0.0, at.z), 0.18);
        }
        let squash = s.land * 0.18;
        // Standing still, it rears up a little to look at you.
        let rear = if s.hop.is_none() && s.ride <= 0.0 {
            let me = self.player.pos - s.pos;
            if me.length() < 3.0 { 0.32 } else { 0.12 }
        } else {
            0.0
        };
        let root = Mat4::from_translation(at + Vec3::Y * rear * 0.12)
            * Mat4::from_rotation_y(s.yaw)
            * Mat4::from_rotation_x(-rear)
            * Mat4::from_scale(Vec3::new(
                SPIDER_SCALE * (1.0 + squash * 0.5),
                SPIDER_SCALE * (1.0 - squash),
                SPIDER_SCALE * (1.0 + squash * 0.5),
            ));
        let o = DrawOpts::at(at + Vec3::Y * 0.15).with_tag(1);
        let limbs = o.two_sided();
        // A big head for its size, and a plump little abdomen breathing behind it.
        let head = Vec3::new(0.0, 0.1, 0.03);
        let big = Mat4::from_translation(head)
            * Mat4::from_scale(Vec3::splat(1.15))
            * Mat4::from_translation(-head);
        r.mesh(&a.bank, &pa.spider_head, &(root * big), &o);
        let breath = 1.0 + (s.t * 3.0).sin() * 0.03;
        let tail = root
            * Mat4::from_translation(Vec3::new(0.0, 0.1, -0.04))
            * Mat4::from_rotation_x(0.3)
            * Mat4::from_scale(Vec3::new(breath * 0.88, breath * 0.88, 0.88));
        r.mesh(&a.bank, &pa.spider_abdomen, &tail, &o);
        let flying = s.hop.is_some();
        let riding = s.ride > 0.0;
        for (k, &(z, splay)) in SPIDER_LEGS.iter().enumerate() {
            for side in [-1.0f32, 1.0] {
                let front = k == 0;
                // Knee angle up off the hip, and the shin's angle down from the knee.
                let (up, down) = if front && s.wave > 0.0 {
                    let w = (s.t * 11.0 + side * 1.6).sin();
                    (1.55 + w * 0.3, -0.35 + w * 0.25)
                } else if flying {
                    (0.55, 1.95)
                } else if riding {
                    (0.55, 0.95)
                } else {
                    let twitch = ((s.t * 1.3 + k as f32 * 2.0 + side).sin() * 0.06).max(0.0);
                    (0.42 + twitch, f32::NAN)
                };
                let reach = if front && s.wave > 0.0 {
                    splay * 0.4
                } else {
                    splay
                };
                let hip = root
                    * Mat4::from_translation(Vec3::new(side * SPIDER_HIP_X, SPIDER_HIP_Y, z))
                    * Mat4::from_scale(Vec3::new(side, 1.0, 1.0))
                    * Mat4::from_rotation_y(-reach);
                let (thigh, shin) = if front {
                    (&pa.spider_front_thigh, &pa.spider_front_shin)
                } else {
                    (&pa.spider_thigh, &pa.spider_shin)
                };
                r.mesh(&a.bank, thigh, &(hip * Mat4::from_rotation_z(up)), &limbs);
                let knee = Vec3::new(SPIDER_THIGH * up.cos(), SPIDER_THIGH * up.sin(), 0.0);
                // Standing, the foot comes down to the ground.
                let down = if down.is_nan() {
                    ((SPIDER_HIP_Y + knee.y) / SPIDER_SHIN)
                        .clamp(0.0, 1.0)
                        .asin()
                } else {
                    down
                };
                let m = hip * Mat4::from_translation(knee) * Mat4::from_rotation_z(-down);
                r.mesh(&a.bank, shin, &m, &limbs);
            }
        }
        // Pedipalps, drumming away.
        for side in [-1.0f32, 1.0] {
            let bob = (s.t * 7.0 + side * 1.3).sin() * 0.3;
            let m = root
                * Mat4::from_translation(SPIDER_PALP * Vec3::new(side, 1.0, 1.0))
                * Mat4::from_rotation_x(-0.3 + bob);
            r.mesh(&a.bank, &pa.spider_palp, &m, &o);
        }
    }

    fn draw_eggs(&self, r: &mut Renderer, a: &Assets) {
        let (x0, z0, x1, z1) = r.cam.visible_tiles(2.5);
        let w = &self.farm;
        for z in z0.max(0)..=z1.min(w.h - 1) {
            for x in x0.max(0)..=x1.min(w.w - 1) {
                if let Some(Obj::Egg { laid }) = w.obj(x, z) {
                    self.draw_egg(r, a, x, z, self.clock.day.saturating_sub(*laid));
                }
            }
        }
    }

    fn draw_egg(&self, r: &mut Renderer, a: &Assets, x: i32, z: i32, age: u32) {
        let pa = &a.pets;
        let base = tile_center(x, z);
        let night = self.env().night;
        let lit = DrawOpts::at(base + Vec3::Y * 0.2);
        let turn = (hash2(x, z, 5) % 628) as f32 / 100.0;
        r.mesh(
            &a.bank,
            &pa.nest,
            &(Mat4::from_translation(base) * Mat4::from_rotation_y(turn)),
            &lit.two_sided(),
        );
        let t = self.time + x as f32 * 1.3;
        let hatching = self.hatching.filter(|h| (h.x, h.z) == (x, z));
        // Leaning over on the ninth day, rocking now and then; shaking itself to bits on
        // the tenth.
        let (lean, rock, hop) = if let Some(h) = hatching {
            let k = h.t / HATCH_POP;
            let amp = 0.12 + k * 0.3;
            (
                0.1,
                (h.t * 34.0).sin() * amp,
                ((h.t * 12.0).sin().max(0.0)) * 0.04 * k,
            )
        } else if age >= HATCH_DAYS {
            let burst = (t * 0.8).fract();
            let rock = if burst < 0.3 {
                (burst * 40.0).sin() * 0.2 * (1.0 - burst / 0.3)
            } else {
                0.0
            };
            (0.28, rock, 0.0)
        } else if age >= TILT_DAY {
            let burst = (t * 0.33).fract();
            let rock = if burst < 0.2 {
                (burst * 30.0).sin() * 0.12 * (1.0 - burst / 0.2)
            } else {
                0.0
            };
            (0.45, rock, 0.0)
        } else {
            (0.0, 0.0, 0.0)
        };
        // It leans sideways (as you look at it), whichever way this one tips.
        let side = if hash2(x, z, 9) % 2 == 0 { 1.0 } else { -1.0 };
        let m = Mat4::from_translation(base + Vec3::Y * (0.02 + hop))
            * Mat4::from_rotation_z((lean + rock) * side)
            * Mat4::from_rotation_x(rock * 0.5)
            * Mat4::from_rotation_y(turn);
        let mesh = if age >= HATCH_DAYS || hatching.is_some() {
            &pa.egg_cracked
        } else {
            &pa.egg
        };
        // Its spots glow after dark.
        let glow = 0.2 + night * 0.5;
        r.mesh(&a.bank, mesh, &m, &lit.with_glow(glow).with_tag(1));
        if r.shadow_pass {
            return;
        }
        if night > 0.3 {
            let pulse = (t * 1.6).sin() * 0.5 + 0.5;
            r.halo(
                base + Vec3::Y * 0.2,
                0.55,
                MINT,
                (0.12 + pulse * 0.12) * night,
            );
            // A mote of light drifting up now and then.
            for k in 0..2 {
                let f = (t * 0.35 + k as f32 * 0.5).fract();
                let a2 = k as f32 * 2.4 + t * 0.4;
                let p = base + Vec3::new(a2.cos() * 0.12, 0.3 + f * 0.8, a2.sin() * 0.1);
                r.point(p, 1, if f < 0.6 { MINT } else { LAVENDER });
            }
        }
    }
}

/// What you hear with your ear to the egg, by how many days it's been in its nest.
pub fn egg_line(age: u32) -> String {
    let feel = match age {
        0 => "It's warm. Something inside is very, very still.",
        1..=3 => "It's warm, and hums very softly.",
        4..=6 => "Tap... tap. Something inside taps back!",
        7..=8 => "Scritch, scratch. Something's wriggling in there.",
        TILT_DAY => "It's leaning right over. It's nearly time!",
        _ => "It's cracking!",
    };
    format!("{feel} (day {} of {HATCH_DAYS})", age.min(HATCH_DAYS))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_egg_counts_its_days() {
        assert!(egg_line(0).contains("day 0 of 10"));
        assert!(egg_line(9).contains("nearly time"));
        assert!(egg_line(12).contains("day 10 of 10"));
    }

    #[test]
    fn a_spider_rides_up_on_the_cat() {
        let mut c = Cat::new(Vec2::new(10.0, 10.0));
        c.mode = CatMode::Sit;
        let shoulders = c.perch(0.0);
        c.mode = CatMode::Stroll;
        let back = c.perch(0.0);
        assert!(shoulders.y > back.y + 0.05, "{shoulders} {back}");
        assert!(back.y > CAT_BODY_Y, "{back}");
    }
}
