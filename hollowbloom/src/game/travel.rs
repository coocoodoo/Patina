//! Getting about: the little bus between the farm and Bramblewick, and walking in and out
//! of the town's buildings.

use glam::{Mat4, Vec2, Vec3};

use super::Io;
use super::dungeon::Level;
use super::farm::{self, MARKS};
use super::gear::Stat;
use super::items::{Buff, Item};
use super::play::{Banner, Play, Trans};
use super::town::{self, BUILDINGS, Place, Room, TOWN};
use super::world::{Area, World};
use crate::assets::Assets;
use crate::assets::town_art::{WHEELS, wheel_turn};
use crate::audio::Sfx;
use crate::palette::*;
use crate::render::{DrawOpts, Mode, PointLight, Renderer};

/// Where the bus runs on the farm (the middle of the road) and where it stops.
pub const FARM_ROAD: f32 = farm::ROAD_X as f32 + 1.0;
pub const FARM_STOP: f32 = 19.5;
/// Where it runs through town and where it stops.
pub const TOWN_ROAD: f32 = 44.0;
pub const TOWN_STOP: f32 = 19.8;

const CRUISE: f32 = 14.0;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum BusPhase {
    /// Pulling in to the stop.
    Arriving,
    /// Doors open.
    Waiting,
    /// Off down the road.
    Leaving,
}

pub struct Bus {
    pub area: Area,
    pub pos: Vec2,
    /// Heading.
    pub dir: Vec2,
    pub speed: f32,
    pub stop: Vec2,
    pub phase: BusPhase,
    pub t: f32,
    /// Distance driven, for turning the wheels.
    pub odo: f32,
    /// The hero is waiting to get on.
    pub boarding: bool,
}

impl Bus {
    pub fn world_pos(&self) -> Vec3 {
        Vec3::new(self.pos.x, 0.0, self.pos.y)
    }
}

/// The world for an area, borrowing only the fields it needs.
pub fn area_world<'a>(
    area: Area,
    farm: &'a World,
    town: &'a World,
    home: &'a World,
    level: &'a Option<Level>,
    room: &'a Option<Room>,
) -> &'a World {
    match (area, level, room) {
        (Area::Hollow { .. }, Some(l), _) => &l.world,
        (Area::Town, _, _) => town,
        (Area::Inside(_), _, Some(r)) => &r.world,
        (Area::Home, _, _) => home,
        _ => farm,
    }
}

pub fn area_world_mut<'a>(
    area: Area,
    farm: &'a mut World,
    town: &'a mut World,
    home: &'a mut World,
    level: &'a mut Option<Level>,
    room: &'a mut Option<Room>,
) -> &'a mut World {
    match (area, level, room) {
        (Area::Hollow { .. }, Some(l), _) => &mut l.world,
        (Area::Town, _, _) => town,
        (Area::Inside(_), _, Some(r)) => &mut r.world,
        (Area::Home, _, _) => home,
        _ => farm,
    }
}

impl Play {
    /// True while the hero is in Bramblewick (outdoors or in).
    pub fn in_town(&self) -> bool {
        matches!(self.area, Area::Town | Area::Inside(_))
    }

    /// Waves the bus down at the stop here.
    pub fn call_bus(&mut self, io: &mut Io) {
        let (area, start, dir, stop) = match self.area {
            Area::Farm => (
                Area::Farm,
                Vec2::new(FARM_ROAD, -4.0),
                Vec2::new(0.0, 1.0),
                Vec2::new(FARM_ROAD, FARM_STOP),
            ),
            _ => (
                Area::Town,
                Vec2::new(-5.0, TOWN_ROAD),
                Vec2::new(1.0, 0.0),
                Vec2::new(TOWN_STOP, TOWN_ROAD),
            ),
        };
        self.bus = Some(Bus {
            area,
            pos: start,
            dir,
            speed: CRUISE,
            stop,
            phase: BusPhase::Arriving,
            t: 0.0,
            odo: 0.0,
            boarding: true,
        });
        self.player.act = None;
        io.audio.play(Sfx::BusHorn);
    }

    /// Drives the bus: in, a pause at the stop, and away.
    pub fn update_bus(&mut self, io: &mut Io) {
        let dt = io.dt;
        let Some(b) = &mut self.bus else { return };
        b.t += dt;
        let mut board = false;
        match b.phase {
            BusPhase::Arriving => {
                let left = (b.stop - b.pos).dot(b.dir);
                // Ease to a stop.
                b.speed = (left * 3.0).clamp(1.0, CRUISE);
                let step = (b.speed * dt).min(left.max(0.0));
                b.pos += b.dir * step;
                b.odo += step;
                if left <= 0.02 {
                    b.pos = b.stop;
                    b.phase = BusPhase::Waiting;
                    b.t = 0.0;
                    b.speed = 0.0;
                    io.audio.play_at(Sfx::Door, 0.7, 1.0);
                }
            }
            BusPhase::Waiting => {
                if b.boarding && b.t > 0.55 {
                    board = true;
                } else if !b.boarding && b.t > 1.2 {
                    b.phase = BusPhase::Leaving;
                    b.t = 0.0;
                    io.audio.play_at(Sfx::BusHorn, 0.6, 1.1);
                }
            }
            BusPhase::Leaving => {
                b.speed = (b.speed + dt * 6.0).min(CRUISE * 1.2);
                let step = b.speed * dt;
                b.pos += b.dir * step;
                b.odo += step;
                if b.t > 8.0 {
                    self.bus = None;
                    return;
                }
            }
        }
        if board && self.fade.is_none() {
            let to_town = self.area == Area::Farm;
            self.start_fade(Trans::Bus { to_town });
        }
    }

    /// Steps off the bus at the other end.
    pub fn ride_bus(&mut self, to_town: bool) {
        self.level = None;
        self.room = None;
        self.foes.clear();
        self.drops.clear();
        self.shots.clear();
        self.bolts.clear();
        self.player.act = None;
        let (area, pos, dir, stop, facing) = if to_town {
            self.town = town::generate(self.restored);
            let (x, z) = TOWN.arrive;
            (
                Area::Town,
                Vec2::new(x as f32 + 2.4, z as f32 - 0.6),
                Vec2::new(1.0, 0.0),
                Vec2::new(TOWN_STOP, TOWN_ROAD),
                Vec2::new(0.0, -1.0),
            )
        } else {
            let (sx, sz) = MARKS.stop;
            (
                Area::Farm,
                Vec2::new(sx as f32 + 1.5, sz as f32 + 1.6),
                Vec2::new(0.0, 1.0),
                Vec2::new(FARM_ROAD, FARM_STOP),
                Vec2::new(-1.0, 0.0),
            )
        };
        self.area = area;
        self.player.pos = pos;
        self.player.facing = facing;
        self.cam_pos = self.player.world_pos();
        self.bus = Some(Bus {
            area,
            pos: stop,
            dir,
            speed: 0.0,
            stop,
            phase: BusPhase::Waiting,
            t: 0.0,
            odo: 0.0,
            boarding: false,
        });
        self.banner = Some(if to_town {
            Banner {
                title: "Bramblewick".into(),
                sub: "the town at the end of the line".into(),
                t: 0.0,
            }
        } else {
            Banner {
                title: "Home Sweet Farm".into(),
                sub: format!("Day {}", self.clock.day),
                t: 0.0,
            }
        });
        self.arrive_folk();
    }

    /// Goes through a door into one of the town's buildings.
    pub fn enter_place(&mut self, place: Place) {
        let room = town::room(place);
        let (ex, ez) = room.exit;
        self.player.pos = Vec2::new(ex as f32 + 0.5, ez as f32 - 0.45);
        self.player.facing = Vec2::new(0.0, -1.0);
        self.player.act = None;
        self.room = Some(room);
        self.area = Area::Inside(place);
        self.cam_pos = self.room_center();
        self.banner = Some(Banner {
            title: place.def().name.into(),
            sub: place.def().trade.into(),
            t: 0.8,
        });
        self.arrive_folk();
    }

    /// Back out through the door, onto the step.
    pub fn leave_place(&mut self) {
        let Area::Inside(place) = self.area else {
            return;
        };
        let b = &BUILDINGS[place.building()];
        let (sx, sz) = b.step();
        self.player.pos = Vec2::new(sx as f32 + 0.5, sz as f32 + 0.5);
        self.player.facing = Vec2::new(0.0, 1.0);
        self.player.act = None;
        self.room = None;
        self.area = Area::Town;
        // Anything finished indoors (a mended fountain, say) shows up outside.
        self.town = town::generate(self.restored);
        self.cam_pos = self.player.world_pos();
        self.arrive_folk();
    }

    /// The middle of the room the hero is in, where the camera rests.
    pub fn room_center(&self) -> Vec3 {
        if self.area == Area::Home {
            let w = &self.house.world;
            return Vec3::new(w.w as f32 * 0.5, 0.0, w.h as f32 * 0.5 + 0.2);
        }
        match &self.room {
            Some(r) => Vec3::new(r.world.w as f32 * 0.5, 0.0, r.world.h as f32 * 0.5 + 0.2),
            None => self.player.world_pos(),
        }
    }

    /// Walking into a shop door opens it, as long as the shop is open.
    pub fn door_ahead(&self) -> Option<usize> {
        if self.area != Area::Town {
            return None;
        }
        let (px, pz) = self.player.tile();
        BUILDINGS
            .iter()
            .position(|b| b.x + b.door == px && b.z + 1 == pz)
    }

    /// Tries a building's door: in if it's open, a note if not.
    pub fn knock(&mut self, id: usize, io: &mut Io) {
        let b = &BUILDINGS[id];
        match b.place {
            Some(place) if place.is_open(self.clock.min) => {
                io.audio.play(Sfx::Door);
                self.start_fade(Trans::Enter(place));
            }
            Some(place) => {
                let (a, z) = place.def().open;
                io.audio.play(Sfx::Denied);
                self.toast(
                    format!(
                        "{} is closed. Open {} to {}.",
                        place.def().name,
                        clock_text(a),
                        clock_text(z)
                    ),
                    None,
                    0,
                );
            }
            None => {
                io.audio.play_at(Sfx::Door, 0.4, 1.3);
                self.toast(format!("{} - nobody answers.", b.name), None, 0);
            }
        }
    }

    /// Headlights after dark.
    pub fn bus_light(&self, night: f32) -> Option<PointLight> {
        let b = self.bus.as_ref().filter(|b| b.area == self.area)?;
        (night > 0.2).then(|| PointLight {
            pos: b.world_pos() + Vec3::new(b.dir.x, 0.0, b.dir.y) * 2.2 + Vec3::Y * 0.6,
            radius: 4.0,
            power: 0.7,
            warmth: 5.0,
        })
    }

    /// Leaning on a shop door opens it; stepping onto the mat inside leads back out.
    pub fn update_doors(&mut self, io: &mut Io) {
        let dt = io.dt;
        match self.area {
            Area::Home => {
                let out = self.player.tile() == super::home::HOUSE_DOOR;
                if out && self.fade.is_none() {
                    io.audio.play(Sfx::Door);
                    self.start_fade(Trans::House { enter: false });
                }
            }
            Area::Farm => {
                // Walking up to the farmhouse door goes in.
                let pushing = io.input.move_axis().y < -0.5;
                if self.house_door_ahead() && pushing && self.fade.is_none() {
                    self.door_push += dt;
                    if self.door_push > 0.2 {
                        self.door_push = -1.0;
                        self.go_indoors(io);
                    }
                } else {
                    self.door_push = self.door_push.min(0.0) + dt;
                }
                self.door_push = self.door_push.min(1.0);
            }
            Area::Inside(_) => {
                let out = self
                    .room
                    .as_ref()
                    .is_some_and(|r| self.player.tile() == r.exit);
                if out && self.fade.is_none() {
                    io.audio.play(Sfx::Door);
                    self.start_fade(Trans::Leave);
                }
            }
            Area::Town => {
                let pushing = io.input.move_axis().y < -0.5;
                match self.door_ahead() {
                    Some(id) if pushing => {
                        self.door_push += dt;
                        if self.door_push > 0.2 {
                            self.door_push = -1.0;
                            self.knock(id, io);
                        }
                    }
                    _ => self.door_push = self.door_push.min(0.0) + dt,
                }
                self.door_push = self.door_push.min(1.0);
            }
            _ => {}
        }
    }

    /// A copper in the fountain, and sometimes a little luck for it.
    pub fn wish(&mut self, io: &mut Io) {
        if self.restored & town::FOUNTAIN == 0 {
            self.toast("The fountain is dry and full of leaves.", None, 0);
            return;
        }
        if self.money == 0 {
            self.toast("You'll need a coin to make a wish.", None, 0);
            return;
        }
        self.money -= 1;
        io.audio.play(Sfx::Coin);
        let (fx, fz) = TOWN.fountain;
        let at = Vec3::new(fx as f32 + 0.5, 0.5, fz as f32 + 0.5);
        self.fx.burst(at, 8, &[SKY, WHITE, AQUA], 1.5, 1.2);
        if self.rng.chance(0.25) && !self.player.buffs.iter().any(|b| b.from == Item::GoldCoin) {
            self.player.add_buff(
                Buff {
                    stat: Stat::Luck,
                    val: 3,
                    secs: 300,
                },
                Item::GoldCoin,
            );
            self.player.refresh();
            self.fx.motes(at, 14, &[GOLD, CREAM, WHITE], 0.5);
            self.toast("Your wish sparkles: Luck +3 for a while!", None, 0);
            io.audio.play(Sfx::Rare);
        } else {
            self.toast("Plip! You make a wish.", None, 0);
        }
    }

    /// The Wishing Tree grants a little luck each day once it blooms.
    pub fn wish_tree(&mut self, blooming: bool, io: &mut Io) {
        if !blooming {
            self.toast(
                "The old tree is bare. Mayor Thistle says it once granted wishes.",
                None,
                0,
            );
            return;
        }
        if self.blessed == self.clock.day {
            self.toast("The tree hums softly. Come back tomorrow.", None, 0);
            return;
        }
        self.blessed = self.clock.day;
        self.player.add_buff(
            Buff {
                stat: Stat::Luck,
                val: 8,
                secs: 900,
            },
            Item::WishStar,
        );
        self.player.refresh();
        let (tx, tz) = TOWN.wish_tree;
        let at = Vec3::new(tx as f32 + 0.5, 1.5, tz as f32 + 0.5);
        self.fx.motes(at, 30, &[BLUSH, PINK, WHITE, CREAM], 1.2);
        self.fx
            .popup_big(self.player.world_pos() + Vec3::Y * 1.2, "BLESSED!", PINK);
        self.toast_colored(
            "The Wishing Tree blesses you: Luck +8 today!",
            None,
            0,
            PINK,
        );
        io.audio.play(Sfx::Enchant);
    }

    pub fn draw_bus(&self, r: &mut Renderer, a: &Assets, night: f32) {
        let Some(b) = &self.bus else { return };
        if b.area != self.area {
            return;
        }
        let yaw = b.dir.x.atan2(b.dir.y);
        let base = b.world_pos();
        // A little bounce while the engine idles.
        let idle = if b.phase == BusPhase::Waiting {
            (self.time * 18.0).sin().abs() * 0.012
        } else {
            0.0
        };
        let m = Mat4::from_translation(base + Vec3::Y * idle) * Mat4::from_rotation_y(yaw);
        let o = DrawOpts::at(base).with_tag(2);
        r.shadow(a.tex(a.disk), base, 1.5);
        r.mesh(&a.bank, &a.town.bus, &m, &o);
        let glass = if night > 0.3 {
            o.with_mode(Mode::Unlit)
        } else {
            o
        };
        r.mesh(&a.bank, &a.town.bus_glass, &m, &glass);
        for w in WHEELS {
            let wm = Mat4::from_translation(base)
                * Mat4::from_rotation_y(yaw)
                * Mat4::from_translation(w)
                * Mat4::from_rotation_x(wheel_turn(b.odo));
            r.mesh(&a.bank, &a.town.wheel, &wm, &o);
        }
    }
}

/// Minutes after midnight as "8:00am".
pub fn clock_text(min: f32) -> String {
    super::play::Clock { day: 1, min }.label()
}
