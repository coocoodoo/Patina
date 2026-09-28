//! The playing state: the farm, the Hollow, and everything the player does in them.

use glam::{Vec2, Vec3};

use super::combat::{Bolt, Flash};
use super::draw::Env;
use super::dungeon::{self, Level, biome_for, is_waystone_floor, ore_item};
use super::farm::{self, MARKS};
use super::foes::{Enemy, St, boss_name};
use super::fx::{Drop, Fx, Shot};
use super::gear::{Class, Rarity, Stat};
use super::items::{Crop, Inventory, Item, Kind, Placeable, Stack};
use super::loot::{self, Fortune};
use super::menus::Menu;
use super::player::{Act, ActKind, HOTBAR, Player, RADIUS};
use super::world::{Area, FERTILE, Floor, Obj, WATERED, Wall, World};
use super::{Io, Settings};
use crate::assets::BIOME_STYLES;
use crate::audio::{Sfx, Song};
use crate::input::Action;
use crate::palette::*;
use crate::render::Camera;
use crate::util::{Rng, damp, hash2, wrap_angle};

pub const MIN_PER_SEC: f32 = 1.55;
pub const DAY_START: f32 = 360.0;
pub const DAY_END: f32 = 1560.0;

#[derive(Clone, Copy, Debug)]
pub struct Clock {
    pub day: u32,
    pub min: f32,
}

impl Clock {
    pub fn label(&self) -> String {
        let m = (self.min as u32 / 10) * 10;
        let h24 = (m / 60) % 24;
        let mm = m % 60;
        let (h12, ap) = match h24 {
            0 => (12, "am"),
            1..=11 => (h24, "am"),
            12 => (12, "pm"),
            _ => (h24 - 12, "pm"),
        };
        format!("{h12}:{mm:02}{ap}")
    }

    pub fn weekday(&self) -> &'static str {
        ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"][((self.day.max(1) - 1) % 7) as usize]
    }
}

/// What happens when a screen fade reaches black.
#[derive(Clone, Copy, Debug)]
pub enum Trans {
    Descend { depth: u32, via_waystone: bool },
    Home,
    Sleep { passed_out: bool },
    Faint,
}

pub struct Fade {
    pub t: f32,
    pub action: Trans,
    pub fired: bool,
}

pub struct Toast {
    pub text: String,
    pub icon: Option<Item>,
    pub n: u32,
    pub t: f32,
    pub color: u8,
}

pub struct Banner {
    pub title: String,
    pub sub: String,
    pub t: f32,
}

pub struct Cat {
    pub pos: Vec2,
    pub target: Vec2,
    pub t: f32,
    pub yaw: f32,
    pub pet: f32,
}

pub struct Play {
    pub seed: u64,
    pub farm: World,
    pub level: Option<Level>,
    pub area: Area,
    pub player: Player,
    pub foes: Vec<Enemy>,
    pub drops: Vec<Drop>,
    pub shots: Vec<Shot>,
    /// The hero's wand bolts.
    pub bolts: Vec<Bolt>,
    /// Brief flashes of light from magic.
    pub flashes: Vec<Flash>,
    /// Burrowby's specials already bought today.
    pub bought: Vec<usize>,
    pub fx: Fx,
    pub rng: Rng,
    pub clock: Clock,
    /// Money, counted in copper (10 copper = 1 silver, 100 copper = 1 gold).
    pub money: u64,
    pub deepest: u32,
    pub waystones: Vec<u32>,
    pub shipping: Vec<Stack>,
    pub menu: Menu,
    pub held: Option<Stack>,
    pub cam: Camera,
    pub cam_pos: Vec3,
    pub shake: f32,
    pub fade: Option<Fade>,
    pub toasts: Vec<Toast>,
    pub banner: Option<Banner>,
    pub cat: Cat,
    pub rain: bool,
    pub time: f32,
    pub target: Option<(i32, i32)>,
    pub target_ok: bool,
    /// The ground point under the mouse, when aiming with it.
    pub aim: Option<Vec2>,
    pub hint: Option<String>,
    pub sel_name_t: f32,
    pub revealed: Vec<bool>,
    pub show_map: bool,
    pub boss_seen: Option<String>,
    pub stats: Stats,
    pub quit_to_title: bool,
    pub last_summary: Option<super::menus::Summary>,
    pub saved_day: u32,
}

#[derive(Default, Clone, serde::Serialize, serde::Deserialize)]
pub struct Stats {
    pub kills: u32,
    pub harvested: u32,
    pub earned: u64,
    pub floors: u32,
}

impl Play {
    pub fn new(seed: u64) -> Play {
        let farm = farm::generate(seed);
        let spawn = MARKS.spawn;
        let player = Player::new(Vec2::new(spawn.0 as f32 + 0.5, spawn.1 as f32 + 0.5));
        let mut p = Play {
            seed,
            farm,
            level: None,
            area: Area::Farm,
            player,
            foes: Vec::new(),
            drops: Vec::new(),
            shots: Vec::new(),
            bolts: Vec::new(),
            flashes: Vec::new(),
            bought: Vec::new(),
            fx: Fx::default(),
            rng: Rng::new(seed ^ 0xC0FFEE),
            clock: Clock {
                day: 1,
                min: DAY_START + 60.0,
            },
            money: 150,
            deepest: 0,
            waystones: Vec::new(),
            shipping: Vec::new(),
            menu: Menu::None,
            held: None,
            cam: Camera::default(),
            cam_pos: Vec3::ZERO,
            shake: 0.0,
            fade: None,
            toasts: Vec::new(),
            banner: None,
            cat: Cat {
                pos: Vec2::new(26.5, 11.5),
                target: Vec2::new(26.5, 11.5),
                t: 0.0,
                yaw: 0.0,
                pet: 0.0,
            },
            rain: false,
            time: 0.0,
            target: None,
            target_ok: false,
            aim: None,
            hint: None,
            sel_name_t: 0.0,
            revealed: Vec::new(),
            show_map: true,
            boss_seen: None,
            stats: Stats::default(),
            quit_to_title: false,
            last_summary: None,
            saved_day: 0,
        };
        p.cam_pos = p.player.world_pos();
        p.banner = Some(Banner {
            title: "Hollowbloom Farm".into(),
            sub: "Day 1 - welcome home".into(),
            t: 0.0,
        });
        p.menu = Menu::dialog(
            "Welcome home! This little farm sits right on top of the Hollow, a cave that goes \
             down forever. Seeds from the deep grow up here, so delve, dig and plant.\n\
             Till with the hoe, plant, water every day and sleep to let things grow. The \
             Hollow is full of treasure: gear, coins and scrolls you can bind to your gear at \
             the enchanting table by the house. Every tenth floor, a waystone brings you home.",
        );
        p
    }

    /// The hero's luck, for loot rolls.
    pub fn fortune(&self) -> Fortune {
        Fortune {
            luck: self.player.luck(),
            greed: self.player.greed(),
        }
    }

    pub fn world(&self) -> &World {
        match &self.level {
            Some(l) if matches!(self.area, Area::Hollow { .. }) => &l.world,
            _ => &self.farm,
        }
    }

    pub fn world_mut(&mut self) -> &mut World {
        match &mut self.level {
            Some(l) if matches!(self.area, Area::Hollow { .. }) => &mut l.world,
            _ => &mut self.farm,
        }
    }

    pub fn depth(&self) -> u32 {
        match self.area {
            Area::Hollow { depth } => depth,
            Area::Farm => 0,
        }
    }

    pub fn toast(&mut self, text: impl Into<String>, icon: Option<Item>, n: u32) {
        self.toast_colored(text, icon, n, CREAM);
    }

    pub fn toast_colored(
        &mut self,
        text: impl Into<String>,
        icon: Option<Item>,
        n: u32,
        color: u8,
    ) {
        let text = text.into();
        if let Some(t) = self
            .toasts
            .iter_mut()
            .find(|t| t.icon.is_some() && t.icon == icon && t.text == text && t.t < 2.0)
        {
            t.n += n;
            t.t = 0.0;
            return;
        }
        self.toasts.push(Toast {
            text,
            icon,
            n,
            t: 0.0,
            color,
        });
        if self.toasts.len() > 5 {
            self.toasts.remove(0);
        }
    }

    pub fn start_fade(&mut self, action: Trans) {
        if self.fade.is_none() {
            self.fade = Some(Fade {
                t: 0.0,
                action,
                fired: false,
            });
        }
    }

    // --------------------------------------------------------------------------------------
    // Environment
    // --------------------------------------------------------------------------------------

    pub fn env(&self) -> Env {
        match self.area {
            Area::Farm => {
                const KEYS: [(f32, f32, f32); 9] = [
                    (360.0, 0.72, 5.4),
                    (430.0, 1.0, 4.6),
                    (720.0, 1.06, 4.0),
                    (1020.0, 1.0, 4.4),
                    (1110.0, 0.92, 6.4),
                    (1180.0, 0.72, 6.0),
                    (1250.0, 0.46, 2.0),
                    (1320.0, 0.44, 1.3),
                    (1560.0, 0.42, 1.1),
                ];
                let m = self.clock.min;
                let mut amb = KEYS[0].1;
                let mut warm = KEYS[0].2;
                for w in KEYS.windows(2) {
                    let (a, b) = (w[0], w[1]);
                    if m >= a.0 && m <= b.0 {
                        let k = (m - a.0) / (b.0 - a.0);
                        amb = a.1 + (b.1 - a.1) * k;
                        warm = a.2 + (b.2 - a.2) * k;
                    }
                }
                if m > KEYS[8].0 {
                    amb = KEYS[8].1;
                    warm = KEYS[8].2;
                }
                if self.rain {
                    amb *= 0.84;
                    warm = (warm - 0.9).max(0.5);
                }
                let night = ((m - 1150.0) / 110.0).clamp(0.0, 1.0);
                Env {
                    ambient: amb,
                    warmth: warm,
                    clear: if night > 0.5 { INK } else { DEEP_TEAL },
                    time: self.time,
                    night,
                }
            }
            Area::Hollow { depth } => {
                let st = &BIOME_STYLES[biome_for(depth)];
                Env {
                    ambient: st.ambient,
                    warmth: st.warmth,
                    clear: st.clear,
                    time: self.time,
                    night: 1.0,
                }
            }
        }
    }

    pub fn music(&self) -> (Option<Song>, i32, f32) {
        match self.area {
            Area::Farm => {
                if self.clock.min > 1170.0 {
                    (Some(Song::Night), 0, 1.0)
                } else {
                    (Some(Song::Morning), 0, 1.0)
                }
            }
            Area::Hollow { depth } => {
                if self.foes.iter().any(|f| f.boss && f.alert) {
                    return (Some(Song::Boss), 0, 1.0);
                }
                if is_waystone_floor(depth) && !self.foes.iter().any(|f| f.boss) {
                    return (Some(Song::Haven), 0, 1.0);
                }
                let b = biome_for(depth);
                let tr = [0, 2, -2, -3, 5, 3][b];
                let tempo = [1.0, 1.04, 0.96, 1.1, 0.92, 1.0][b];
                (Some(Song::Hollow), tr, tempo)
            }
        }
    }

    // --------------------------------------------------------------------------------------
    // Travel
    // --------------------------------------------------------------------------------------

    fn enter_hollow(&mut self, depth: u32, via_waystone: bool) {
        let level = dungeon::generate(self.seed, depth, via_waystone);
        self.foes.clear();
        self.drops.clear();
        self.shots.clear();
        self.bolts.clear();
        let biome = biome_for(depth);
        for (i, s) in level.spawns.iter().enumerate() {
            self.foes.push(Enemy::new(
                s.foe,
                s.x,
                s.z,
                depth,
                biome,
                s.boss,
                hash2(depth as i32, i as i32, self.seed as u32),
            ));
        }
        self.player.pos = Vec2::new(level.start.0 as f32 + 0.5, level.start.1 as f32 + 0.5);
        self.player.act = None;
        self.revealed = vec![false; (level.world.w * level.world.h) as usize];
        self.level = Some(level);
        self.area = Area::Hollow { depth };
        self.cam_pos = self.player.world_pos();
        if depth > self.deepest {
            self.deepest = depth;
        }
        self.stats.floors += 1;
        self.boss_seen = None;
        self.banner = Some(Banner {
            title: format!("Floor {depth}"),
            sub: BIOME_STYLES[biome].name.to_string(),
            t: 0.0,
        });
    }

    fn go_home(&mut self) {
        self.level = None;
        self.foes.clear();
        self.drops.clear();
        self.shots.clear();
        self.bolts.clear();
        self.area = Area::Farm;
        let (x, z) = MARKS.hollow;
        self.player.pos = Vec2::new(x as f32 + 0.5, z as f32 + 1.6);
        self.player.facing = Vec2::new(0.0, 1.0);
        self.player.act = None;
        self.cam_pos = self.player.world_pos();
        self.banner = Some(Banner {
            title: "Home Sweet Farm".into(),
            sub: format!("Deepest floor: {}", self.deepest),
            t: 0.0,
        });
    }

    /// Ends the day: growth, sales, rest.
    fn sleep(&mut self, passed_out: bool) {
        let day = self.clock.day;
        let mut earned = 0u64;
        for s in self.shipping.drain(..) {
            earned += s.value();
        }
        self.money += earned;
        self.bought.clear();
        self.stats.earned += earned;
        let night = farm::new_day(&mut self.farm, day + 1, false);
        self.rain = Rng::new(self.seed ^ ((day as u64 + 1) * 31)).chance(0.18);
        if self.rain {
            // Rain waters everything for the new day.
            let (w, h) = (self.farm.w, self.farm.h);
            for z in 0..h {
                for x in 0..w {
                    if self.farm.floor(x, z) == Floor::Tilled {
                        self.farm.set_flag(x, z, WATERED, true);
                    }
                }
            }
        }
        self.clock.day += 1;
        self.clock.min = DAY_START;
        let p = &mut self.player;
        p.buffs.clear();
        p.refresh();
        p.energy = if passed_out {
            p.max_energy() as f32 * 0.6
        } else {
            p.max_energy() as f32
        };
        p.hp = p.max_hp();
        p.mana = p.max_mana() as f32;
        p.water = p.can_capacity();
        let (dx, dz) = MARKS.door;
        p.pos = Vec2::new(dx as f32 + 0.5, dz as f32 + 0.6);
        p.facing = Vec2::new(0.0, 1.0);
        p.act = None;
        self.level = None;
        self.foes.clear();
        self.shots.clear();
        self.bolts.clear();
        self.drops.clear();
        self.area = Area::Farm;
        self.cam_pos = self.player.world_pos();
        self.last_summary = Some(super::menus::Summary {
            day: self.clock.day,
            earned,
            grown: night.grown,
            ready: night.ready,
            rain: self.rain,
            passed_out,
            fainted: false,
        });
        self.menu = Menu::Summary;
    }

    fn faint(&mut self) {
        let lost = (self.money / 10).min(1000);
        self.money -= lost;
        self.sleep(true);
        if let Some(s) = &mut self.last_summary {
            s.fainted = true;
        }
        self.toast(
            format!("Lost {} while you were out", loot::money_text(lost)),
            None,
            0,
        );
        self.player.hp = self.player.max_hp() / 2;
    }

    // --------------------------------------------------------------------------------------
    // Update
    // --------------------------------------------------------------------------------------

    pub fn update(&mut self, io: &mut Io, settings: &mut Settings) {
        let dt = io.dt;
        self.time += dt;
        self.update_camera(io.view, dt);

        // Fades run even over menus.
        if let Some(f) = &mut self.fade {
            f.t += dt * 2.4;
            if f.t >= 1.0 && !f.fired {
                f.fired = true;
                let action = f.action;
                match action {
                    Trans::Descend {
                        depth,
                        via_waystone,
                    } => self.enter_hollow(depth, via_waystone),
                    Trans::Home => self.go_home(),
                    Trans::Sleep { passed_out } => self.sleep(passed_out),
                    Trans::Faint => self.faint(),
                }
                io.audio.play(Sfx::Stairs);
            }
            if let Some(f) = &self.fade {
                if f.t >= 2.0 {
                    self.fade = None;
                }
            }
        }
        let (song, tr, tempo) = if matches!(self.menu, Menu::Summary) {
            (Some(Song::Title), 0, 1.0)
        } else {
            self.music()
        };
        io.audio.music(song, tr, tempo);

        for t in &mut self.toasts {
            t.t += dt;
        }
        self.toasts.retain(|t| t.t < 3.2);
        if let Some(b) = &mut self.banner {
            b.t += dt;
            if b.t > 3.5 {
                self.banner = None;
            }
        }
        self.sel_name_t += dt;

        if !matches!(self.menu, Menu::None) {
            self.update_menu(io, settings);
            self.fx.update(dt);
            return;
        }
        if self.fade.is_some() {
            self.fx.update(dt);
            return;
        }
        let input = io.input;
        if input.pressed(Action::Menu) {
            self.menu = Menu::pause();
            io.audio.play(Sfx::UiSelect);
            return;
        }
        if input.pressed(Action::Inventory) {
            self.menu = Menu::inventory(false);
            io.audio.play(Sfx::UiSelect);
            return;
        }
        if input.pressed(Action::Crafting) {
            self.menu = Menu::inventory(true);
            io.audio.play(Sfx::UiSelect);
            return;
        }
        if input.pressed(Action::Map) {
            self.show_map = !self.show_map;
        }

        // Time.
        self.clock.min += dt * MIN_PER_SEC;
        if self.clock.min >= DAY_END {
            self.clock.min = DAY_END;
            self.toast("You're exhausted... you collapse.", None, 0);
            self.start_fade(Trans::Sleep { passed_out: true });
        }

        // Hotbar.
        if let Some(n) = input.number_pressed() {
            self.select(n, io);
        }
        if input.wheel != 0.0 {
            let dir = if input.wheel > 0.0 { HOTBAR - 1 } else { 1 };
            let n = (self.player.sel + dir) % HOTBAR;
            self.select(n, io);
        }
        if input.pressed(Action::NextSlot) {
            let n = (self.player.sel + 1) % HOTBAR;
            self.select(n, io);
        }
        if input.pressed(Action::PrevSlot) {
            let n = (self.player.sel + HOTBAR - 1) % HOTBAR;
            self.select(n, io);
        }

        self.player.refresh();
        self.update_player(io);
        self.update_foes(io);
        self.update_statuses(dt, io);
        self.update_bolts(dt, io);
        self.update_drops(io);
        self.update_cat(dt);
        self.update_vitals(dt);
        self.fx.update(dt);
        self.shake = (self.shake - dt * 2.5).max(0.0);
        if let Area::Hollow { .. } = self.area {
            self.reveal();
        }
        if self.player.hp <= 0 && self.fade.is_none() {
            self.player.hp = 0;
            io.audio.play(Sfx::PlayerHurt);
            self.toast("You fainted!", None, 0);
            self.start_fade(Trans::Faint);
        }
    }

    /// Regeneration, mana, food buffs and magic flashes.
    fn update_vitals(&mut self, dt: f32) {
        let p = &mut self.player;
        // Health: the Regen stat every five seconds, heartsip, and a slow trickle at home.
        let mut rate = p.stat(Stat::Regen).max(0) as f32 / 5.0;
        if self.area == Area::Farm {
            rate += 1.0;
        }
        p.regen_acc += rate * dt;
        if p.regen_acc >= 1.0 {
            let heal = p.regen_acc.floor();
            p.regen_acc -= heal;
            if p.hp > 0 {
                p.hp = (p.hp + heal as i32).min(p.max_hp());
            }
        }
        let spirit = p.stat(Stat::Spirit).max(0) as f32;
        p.mana = (p.mana + (2.5 + spirit * 0.3) * dt).min(p.max_mana() as f32);
        p.no_mana_t = (p.no_mana_t - dt).max(0.0);
        let mut ended = Vec::new();
        for b in &mut p.buffs {
            b.left -= dt;
            if b.left <= 0.0 {
                ended.push(b.from);
            }
        }
        p.buffs.retain(|b| b.left > 0.0);
        if !ended.is_empty() {
            p.refresh();
        }
        for item in ended {
            self.toast(format!("{} wore off", item.def().name), None, 0);
        }
        for f in &mut self.flashes {
            f.t += dt;
        }
        self.flashes.retain(|f| f.t < 0.35);
    }

    fn select(&mut self, n: usize, io: &Io) {
        if n != self.player.sel {
            self.player.sel = n;
            self.sel_name_t = 0.0;
            io.audio.play_at(Sfx::UiMove, 0.6, 1.0);
        }
    }

    fn update_camera(&mut self, view: (usize, usize), dt: f32) {
        let p = self.player.world_pos()
            + Vec3::new(self.player.facing.x * 0.6, 0.0, self.player.facing.y * 0.4);
        let k = damp(7.0, dt);
        self.cam_pos += (p - self.cam_pos) * k;
        let mut t = self.cam_pos;
        if self.area == Area::Farm {
            let w = self.farm.w as f32;
            let h = self.farm.h as f32;
            t.x = t.x.clamp(10.5, w - 10.5);
            t.z = t.z.clamp(7.0, h - 3.5);
        }
        if self.shake > 0.0 {
            let s = self.shake * self.shake * 0.25;
            t.x += (self.time * 71.0).sin() * s;
            t.z += (self.time * 53.0).cos() * s;
        }
        self.cam.target = t;
        self.cam.update(view.0.max(1), view.1.max(1));
    }

    fn update_player(&mut self, io: &mut Io) {
        let dt = io.dt;
        let input = io.input;
        let p = &mut self.player;
        p.hurt = (p.hurt - dt).max(0.0);
        p.flash = (p.flash - dt).max(0.0);
        p.dodge_cd = (p.dodge_cd - dt).max(0.0);
        p.eat_t = (p.eat_t - dt).max(0.0);
        let mut mv = input.move_axis();
        let busy = p.act.is_some();
        // Dodge roll.
        if input.pressed(Action::Dodge) && p.dodge_cd <= 0.0 && p.dodge <= 0.0 {
            let dir = if mv.length_squared() > 0.0 {
                mv
            } else {
                p.facing
            };
            p.dodge = 0.26;
            p.dodge_cd = 0.6;
            p.dodge_dir = dir.normalize_or_zero();
            p.act = None;
            io.audio.play(Sfx::Dodge);
            self.fx
                .burst(p.world_pos() + Vec3::Y * 0.05, 6, &[SAND, WHITE], 1.5, 0.8);
        }
        let world = match &self.level {
            Some(l) if matches!(self.area, Area::Hollow { .. }) => &l.world,
            _ => &self.farm,
        };
        let p = &mut self.player;
        if p.dodge > 0.0 {
            p.dodge -= dt;
            let d = p.dodge_dir * 9.0 * dt;
            p.pos = world.move_circle(p.pos, d, RADIUS);
            mv = Vec2::ZERO;
        } else if busy {
            mv *= 0.25;
        }
        let tired = p.energy <= 0.0;
        let speed = p.move_speed() * if tired { 0.65 } else { 1.0 };
        if mv.length_squared() > 0.0 {
            p.pos = world.move_circle(p.pos, mv * speed * dt, RADIUS);
            if !busy {
                p.facing = mv.normalize();
            }
            p.walk += dt * 11.0;
            p.stride = (p.stride + dt * 6.0).min(1.0);
            p.step_t -= dt;
            if p.step_t <= 0.0 {
                p.step_t = 0.3;
                io.audio.play_at(Sfx::Step, 0.5, 0.9 + (p.walk % 0.3));
            }
        } else {
            p.stride = (p.stride - dt * 6.0).max(0.0);
        }
        if p.vel.length_squared() > 0.001 {
            p.pos = world.move_circle(p.pos, p.vel * dt, RADIUS);
            p.vel *= (1.0 - dt * 9.0).max(0.0);
        }
        // Face the mouse when aiming with it.
        if input.mouse_aim {
            if let Some(g) = self.cam.ground(input.mouse.x, input.mouse.y, 0.0) {
                let d = Vec2::new(g.x, g.z) - p.pos;
                if d.length_squared() > 0.04
                    && (busy || mv.length_squared() == 0.0 || input.down(Action::Use))
                {
                    p.facing = d.normalize();
                }
            }
        }
        let want = p.facing.x.atan2(p.facing.y);
        p.yaw += wrap_angle(want - p.yaw) * damp(18.0, dt);

        // Current action.
        if let Some(act) = &mut p.act {
            act.t += dt;
            let prog = act.progress();
            let fire = !act.fired && prog >= act.kind.impact();
            if fire {
                act.fired = true;
            }
            let done = prog >= 1.0;
            let (kind, tile, dir) = (act.kind, act.tile, act.dir);
            if done {
                p.act = None;
            }
            if fire {
                self.fire(kind, tile, dir, io);
            }
        }
        // Aim at the tile in front (or under the mouse) before acting on it.
        self.update_target(io);
        // Start actions.
        let p = &self.player;
        if p.act.is_none() && p.dodge <= 0.0 {
            if input.pressed(Action::Interact) {
                self.interact(io);
            } else if input.down(Action::Use) && (input.pressed(Action::Use) || self.held_repeats())
            {
                self.use_held(io);
            }
        }
    }

    fn held_repeats(&self) -> bool {
        self.player
            .held_class()
            .is_some_and(|c| ActKind::for_class(c).is_some())
    }

    /// The tile the player is working on: the one under the mouse when it is close,
    /// otherwise the one in front.
    fn update_target(&mut self, io: &Io) {
        let input = io.input;
        let p = &self.player;
        let (px, pz) = p.tile();
        let front = {
            let f = p.facing;
            if f.x.abs() > f.y.abs() {
                (px + f.x.signum() as i32, pz)
            } else {
                (px, pz + f.y.signum() as i32)
            }
        };
        let mut t = front;
        self.aim = None;
        if input.mouse_aim {
            if let Some(g) = self.cam.ground(input.mouse.x, input.mouse.y, 0.0) {
                self.aim = Some(Vec2::new(g.x, g.z));
                let (mx, mz) = (g.x.floor() as i32, g.z.floor() as i32);
                if (mx - px).abs() <= 1 && (mz - pz).abs() <= 1 {
                    t = (mx, mz);
                } else {
                    let d = Vec2::new(g.x, g.z) - p.pos;
                    let a = d.y.atan2(d.x);
                    let oct = ((a / (std::f32::consts::PI / 4.0)).round() as i32).rem_euclid(8);
                    let (dx, dz) = [
                        (1, 0),
                        (1, 1),
                        (0, 1),
                        (-1, 1),
                        (-1, 0),
                        (-1, -1),
                        (0, -1),
                        (1, -1),
                    ][oct as usize];
                    t = (px + dx, pz + dz);
                }
            }
        }
        self.target = Some(t);
        self.target_ok = self.can_act_on(t);
        self.hint = self.interact_hint(t);
    }

    fn can_act_on(&self, (x, z): (i32, i32)) -> bool {
        let w = self.world();
        let Some(item) = self.player.held() else {
            return false;
        };
        match item.def().kind {
            Kind::Gear(b) => match b.class {
                Class::Hoe => {
                    self.area == Area::Farm
                        && matches!(w.floor(x, z), Floor::Grass | Floor::Soil)
                        && w.wall(x, z) == Wall::None
                        && w.obj(x, z)
                            .is_none_or(|o| matches!(o, Obj::Weed { .. } | Obj::Flower { .. }))
                }
                Class::Can => w.floor(x, z) == Floor::Tilled || w.floor(x, z) == Floor::Water,
                Class::Sickle => w.obj(x, z).is_some_and(|o| match o {
                    Obj::Weed { .. } | Obj::Flower { .. } => true,
                    Obj::Crop { crop, days, .. } => crop.stage(*days) == 3,
                    _ => false,
                }),
                Class::Pickaxe => {
                    matches!(
                        w.wall(x, z),
                        Wall::Rock | Wall::Ore(_) | Wall::Brick | Wall::Timber
                    ) || w.obj(x, z).is_some_and(|o| {
                        matches!(
                            o,
                            Obj::Rock { .. }
                                | Obj::Boulder { .. }
                                | Obj::Crystal { .. }
                                | Obj::Stalagmite { .. }
                                | Obj::Lamp
                                | Obj::Sprinkler { .. }
                                | Obj::FlowerPot { .. }
                                | Obj::Torch
                                | Obj::Chest { .. }
                                | Obj::Pot { .. }
                                | Obj::Crate { .. }
                                | Obj::EnchantTable
                        )
                    }) || matches!(w.floor(x, z), Floor::Planks | Floor::Cobble)
                }
                Class::Axe => {
                    w.obj(x, z).is_some_and(|o| {
                        matches!(
                            o,
                            Obj::Tree { .. }
                                | Obj::Pine { .. }
                                | Obj::Stump { .. }
                                | Obj::Log { .. }
                                | Obj::Fence
                                | Obj::Bench
                                | Obj::Workbench
                                | Obj::Chest { .. }
                                | Obj::Crate { .. }
                                | Obj::Weed { .. }
                        )
                    }) || w.wall(x, z) == Wall::Timber
                }
                _ => false,
            },
            Kind::Seed(_) => {
                self.area == Area::Farm && w.floor(x, z) == Floor::Tilled && w.obj(x, z).is_none()
            }
            Kind::Place(pl) => self.can_place(pl, x, z),
            _ => false,
        }
    }

    fn can_place(&self, pl: Placeable, x: i32, z: i32) -> bool {
        let w = self.world();
        if !w.inside(x, z) || w.wall(x, z) != Wall::None {
            return false;
        }
        let floor = w.floor(x, z);
        if matches!(floor, Floor::Water | Floor::Lava | Floor::Void) {
            return false;
        }
        let in_hollow = self.area != Area::Farm;
        match pl {
            Placeable::WoodPath | Placeable::StonePath => {
                w.obj(x, z).is_none()
                    && !matches!(floor, Floor::Tilled | Floor::Planks | Floor::Cobble)
            }
            _ => {
                if w.obj(x, z).is_some() {
                    return false;
                }
                if in_hollow
                    && !matches!(
                        pl,
                        Placeable::Torch | Placeable::StoneWall | Placeable::WoodWall
                    )
                {
                    return false;
                }
                // Keep solid things off the player.
                let solid = !matches!(pl, Placeable::Torch);
                if solid {
                    let c = Vec2::new(x as f32 + 0.5, z as f32 + 0.5);
                    let d = (self.player.pos - c).abs();
                    if d.x < 0.5 + RADIUS && d.y < 0.5 + RADIUS {
                        return false;
                    }
                }
                true
            }
        }
    }

    fn interact_hint(&self, (x, z): (i32, i32)) -> Option<String> {
        let w = self.world();
        let (ax, az) = w.anchor(x, z);
        let o = w.obj(ax, az)?;
        let s = match o {
            Obj::House => "Sleep",
            Obj::Bin => "Ship items",
            Obj::Hollow => "Enter the Hollow",
            Obj::Stall => "Shop",
            Obj::Chest { .. } => "Open chest",
            Obj::LootChest { opened: false } => "Open",
            Obj::Sign { .. } => "Read",
            Obj::StairsDown => "Descend",
            Obj::Waystone => "Touch the waystone",
            Obj::Workbench => "Craft",
            Obj::EnchantTable => "Enchant",
            Obj::Crop { crop, days, .. } if crop.stage(*days) == 3 => "Harvest",
            Obj::Bench => "Sit",
            _ => return None,
        };
        Some(s.to_string())
    }

    // --------------------------------------------------------------------------------------
    // Using things
    // --------------------------------------------------------------------------------------

    fn use_held(&mut self, io: &mut Io) {
        let Some(item) = self.player.held() else {
            return;
        };
        let tile = self.target.unwrap_or(self.player.tile());
        let dir = self.player.facing;
        let kind = item.class().and_then(ActKind::for_class);
        let Some(kind) = kind else {
            self.use_item(item, tile, io);
            return;
        };
        let p = &mut self.player;
        match kind {
            ActKind::Bolt | ActKind::Blast => {
                let cost = p.mana_cost(if kind == ActKind::Bolt { 5.0 } else { 12.0 });
                if p.mana < cost {
                    if p.no_mana_t <= 0.0 {
                        p.no_mana_t = 1.2;
                        self.toast("Not enough mana.", None, 0);
                        io.audio.play(Sfx::Denied);
                    }
                    return;
                }
                p.mana -= cost;
            }
            ActKind::Slash => {}
            _ => {
                let base = match kind {
                    ActKind::Water => 1.4,
                    ActKind::Reap => 1.2,
                    _ => 2.0,
                };
                let cost = p.tool_cost(base);
                p.energy = (p.energy - cost).max(-10.0);
            }
        }
        let dur = kind.base_duration() * p.haste();
        p.act = Some(Act::new(kind, dur, tile, dir));
        match kind {
            ActKind::Slash | ActKind::Reap => io.audio.play(Sfx::Swing),
            ActKind::Bolt | ActKind::Blast => io.audio.play_at(Sfx::Swing, 0.5, 1.4),
            _ => {}
        }
    }

    /// Non-tool items: plant, place, eat, special.
    fn use_item(&mut self, item: Item, (x, z): (i32, i32), io: &mut Io) {
        let sel = self.player.sel;
        match item.def().kind {
            Kind::Seed(crop) => {
                if self.area != Area::Farm {
                    self.toast("Seeds need farm soil.", None, 0);
                    io.audio.play(Sfx::Denied);
                } else if self.world().floor(x, z) == Floor::Tilled
                    && self.world().obj(x, z).is_none()
                {
                    self.player.inv.take_one(sel);
                    self.farm.set_obj(
                        x,
                        z,
                        Some(Obj::Crop {
                            crop,
                            days: 0,
                            harvested: false,
                        }),
                    );
                    io.audio.play(Sfx::Plant);
                    self.fx
                        .burst(tile_center(x, z), 5, &[RUST, CLAY, GREEN], 1.2, 1.5);
                } else {
                    io.audio.play(Sfx::Denied);
                }
            }
            Kind::Place(pl) => {
                if !self.can_place(pl, x, z) {
                    io.audio.play(Sfx::Denied);
                    if self.area != Area::Farm
                        && !matches!(
                            pl,
                            Placeable::Torch | Placeable::StoneWall | Placeable::WoodWall
                        )
                    {
                        self.toast("That belongs on the farm.", None, 0);
                    }
                    return;
                }
                self.player.inv.take_one(sel);
                let w = self.world_mut();
                match pl {
                    Placeable::WoodPath => w.set_floor(x, z, Floor::Planks),
                    Placeable::StonePath => w.set_floor(x, z, Floor::Cobble),
                    Placeable::StoneWall => w.set_wall(x, z, Wall::Brick),
                    Placeable::WoodWall => w.set_wall(x, z, Wall::Timber),
                    Placeable::Chest => w.set_obj(
                        x,
                        z,
                        Some(Obj::Chest {
                            items: vec![None; 30],
                        }),
                    ),
                    Placeable::Torch => w.set_obj(x, z, Some(Obj::Torch)),
                    Placeable::Lamp => w.set_obj(x, z, Some(Obj::Lamp)),
                    Placeable::Fence => w.set_obj(x, z, Some(Obj::Fence)),
                    Placeable::Sprinkler(t) => w.set_obj(x, z, Some(Obj::Sprinkler { tier: t })),
                    Placeable::Workbench => w.set_obj(x, z, Some(Obj::Workbench)),
                    Placeable::FlowerPot => w.set_obj(x, z, Some(Obj::FlowerPot { var: 0 })),
                    Placeable::Bench => w.set_obj(x, z, Some(Obj::Bench)),
                    Placeable::EnchantTable => w.set_obj(x, z, Some(Obj::EnchantTable)),
                }
                io.audio.play(Sfx::Place);
                self.fx
                    .burst(tile_center(x, z), 6, &[SAND, KHAKI], 1.2, 1.0);
            }
            Kind::Produce { hp, energy } => self.eat(item, hp, energy, 0, None, io),
            Kind::Food {
                hp,
                energy,
                mana,
                buff,
            } => self.eat(item, hp, energy, mana, buff, io),
            Kind::Gear(b) if b.class.is_armor() => {
                // Wear it straight from the hotbar.
                if self.player.equip_from(sel) {
                    io.audio.play(Sfx::Equip);
                    self.toast(format!("Wearing {}", item.def().name), Some(item), 0);
                    self.fx.motes(
                        self.player.world_pos() + Vec3::Y * 0.5,
                        8,
                        &[WHITE, CREAM, SKY],
                        0.3,
                    );
                }
            }
            Kind::Scroll(_) => {
                self.toast("Bind scrolls at an enchanting table.", None, 0);
            }
            Kind::Feather => {
                if let Area::Hollow { .. } = self.area {
                    self.player.inv.take_one(sel);
                    io.audio.play(Sfx::Waystone);
                    self.fx
                        .motes(self.player.world_pos(), 20, &[WHITE, SKY, MINT], 0.5);
                    self.start_fade(Trans::Home);
                } else {
                    self.toast("It only works in the Hollow.", None, 0);
                }
            }
            Kind::HeartCrystal => {
                self.player.inv.take_one(sel);
                self.player.base_hp += 10;
                self.player.hp = self.player.max_hp();
                io.audio.play(Sfx::LevelUp);
                self.fx
                    .motes(self.player.world_pos(), 16, &[PINK, BLUSH, WHITE], 0.4);
                self.toast("Max HP +10", None, 0);
            }
            Kind::SunStone => {
                self.player.inv.take_one(sel);
                self.player.base_energy += 15;
                self.player.energy = self.player.max_energy() as f32;
                io.audio.play(Sfx::LevelUp);
                self.fx
                    .motes(self.player.world_pos(), 16, &[GOLD, CREAM, WHITE], 0.4);
                self.toast("Max energy +15", None, 0);
            }
            Kind::WishStar => {
                self.player.inv.take_one(sel);
                self.player.base_mana += 10;
                self.player.mana = self.player.max_mana() as f32;
                io.audio.play(Sfx::LevelUp);
                self.fx
                    .motes(self.player.world_pos(), 16, &[LAVENDER, BLUSH, WHITE], 0.4);
                self.toast("Max mana +10", None, 0);
            }
            _ => {}
        }
    }

    fn eat(
        &mut self,
        item: Item,
        hp: i32,
        energy: i32,
        mana: i32,
        buff: Option<super::items::Buff>,
        io: &mut Io,
    ) {
        let p = &mut self.player;
        if p.eat_t > 0.0 {
            return;
        }
        let full = p.hp >= p.max_hp()
            && p.energy >= p.max_energy() as f32
            && (mana == 0 || p.mana >= p.max_mana() as f32);
        if full && buff.is_none() {
            self.toast("You're full.", None, 0);
            return;
        }
        let sel = p.sel;
        p.inv.take_one(sel);
        if let Some(b) = buff {
            p.add_buff(b, item);
            p.refresh();
        }
        p.hp = (p.hp + hp).min(p.max_hp());
        p.energy = (p.energy + energy as f32).min(p.max_energy() as f32);
        p.mana = (p.mana + mana as f32).min(p.max_mana() as f32);
        p.eat_t = 0.6;
        io.audio.play(Sfx::Eat);
        let pos = p.world_pos() + Vec3::Y * 0.9;
        if hp > 0 {
            self.fx.popup(pos, format!("+{hp}"), PINK);
        }
        if energy > 0 {
            self.fx
                .popup(pos + Vec3::new(0.3, 0.2, 0.0), format!("+{energy}"), GOLD);
        }
        if mana > 0 {
            self.fx.popup(
                pos + Vec3::new(-0.3, 0.2, 0.0),
                format!("+{mana}"),
                LAVENDER,
            );
        }
        match buff {
            Some(b) => {
                self.fx.motes(
                    self.player.world_pos() + Vec3::Y * 0.4,
                    10,
                    &[b.stat.def().color, WHITE],
                    0.3,
                );
                self.toast(
                    format!("{}: {}", item.def().name, b.stat.line(b.val as i32)),
                    Some(item),
                    0,
                );
            }
            None => self.toast(format!("Ate {}", item.def().name), None, 0),
        }
    }

    fn interact(&mut self, io: &mut Io) {
        let Some((tx, tz)) = self.target else { return };
        // Check the target tile, then the tile we stand on, then anything adjacent.
        let mut cands = vec![(tx, tz)];
        let (px, pz) = self.player.tile();
        for dz in -1..=1 {
            for dx in -1..=1 {
                if (px + dx, pz + dz) != (tx, tz) {
                    cands.push((px + dx, pz + dz));
                }
            }
        }
        for (x, z) in cands {
            if self.interact_at(x, z, io) {
                return;
            }
        }
        // Pet the cat.
        if self.area == Area::Farm && (self.cat.pos - self.player.pos).length() < 1.3 {
            self.cat.pet = 1.5;
            self.fx.motes(
                Vec3::new(self.cat.pos.x, 0.5, self.cat.pos.y),
                5,
                &[PINK, BLUSH],
                0.2,
            );
            self.fx
                .popup(Vec3::new(self.cat.pos.x, 0.9, self.cat.pos.y), "♥", PINK);
            io.audio.play_at(Sfx::Pickup, 0.7, 1.3);
            return;
        }
        // Nothing there: use the held item (eat, plant, place).
        if let Some(item) = self.player.held() {
            if item.class().and_then(ActKind::for_class).is_none() {
                self.use_item(item, (tx, tz), io);
            }
        }
    }

    fn interact_at(&mut self, x: i32, z: i32, io: &mut Io) -> bool {
        let (ax, az) = self.world().anchor(x, z);
        let Some(obj) = self.world().obj(ax, az).cloned() else {
            return false;
        };
        match obj {
            Obj::House => {
                self.menu = Menu::dialog_choice(
                    "Your cozy bed is waiting. Go to sleep and end the day?",
                    vec![
                        ("Sleep", super::menus::Choice::Sleep),
                        ("Not yet", super::menus::Choice::Close),
                    ],
                );
            }
            Obj::Bin => self.menu = Menu::Ship { cursor: 0 },
            Obj::Stall => {
                self.menu = Menu::shop();
            }
            Obj::Hollow => {
                let mut floors = vec![1];
                floors.extend(self.waystones.iter().copied());
                self.menu = Menu::Descend { floors, sel: 0 };
            }
            Obj::Chest { .. } => {
                self.menu = Menu::Chest {
                    x: ax,
                    z: az,
                    cursor: 0,
                }
            }
            Obj::Workbench => self.menu = Menu::inventory(true),
            Obj::EnchantTable => self.menu = Menu::enchant(),
            Obj::Sign { text } => {
                let msg = match text {
                    0 => {
                        "THE HOLLOW\nMind your step. It goes down forever. Waystones every ten floors will bring you home."
                    }
                    _ => "BURROWBY'S GOODS\nSeeds, supplies and a warm hello.",
                };
                self.menu = Menu::dialog(msg);
            }
            Obj::LootChest { opened: false } => {
                self.world_mut()
                    .set_obj(ax, az, Some(Obj::LootChest { opened: true }));
                self.open_loot(ax, az, io);
                io.audio.play(Sfx::Chest);
            }
            Obj::StairsDown => {
                if self.foes.iter().any(|f| f.boss) {
                    self.toast("The guardian blocks the way down.", None, 0);
                    io.audio.play(Sfx::Denied);
                } else {
                    let d = self.depth() + 1;
                    self.start_fade(Trans::Descend {
                        depth: d,
                        via_waystone: false,
                    });
                }
            }
            Obj::Waystone => {
                if self.foes.iter().any(|f| f.boss) {
                    self.toast("The waystone is silent while the guardian lives.", None, 0);
                    io.audio.play(Sfx::Denied);
                } else {
                    let d = self.depth();
                    if !self.waystones.contains(&d) {
                        self.waystones.push(d);
                        self.waystones.sort();
                        io.audio.play(Sfx::Waystone);
                        self.fx.motes(
                            tile_center(ax, az) + Vec3::Y * 0.5,
                            24,
                            &[MINT, AQUA, WHITE],
                            0.5,
                        );
                        self.toast(format!("Waystone {d} attuned!"), None, 0);
                    }
                    self.menu = Menu::dialog_choice(
                        "The waystone hums warmly. Return to the surface?",
                        vec![
                            ("Go home", super::menus::Choice::ReturnHome),
                            ("Keep delving", super::menus::Choice::Close),
                        ],
                    );
                }
            }
            Obj::Crop { crop, days, .. } if crop.stage(days) == 3 => {
                self.harvest(ax, az, crop, io);
            }
            Obj::Bench => {
                self.player.pos = Vec2::new(ax as f32 + 0.5, az as f32 + 0.75);
                self.player.facing = Vec2::new(0.0, 1.0);
                self.fx.motes(
                    self.player.world_pos() + Vec3::Y * 0.6,
                    4,
                    &[PINK, WHITE],
                    0.2,
                );
                self.toast("Ahh, a nice rest.", None, 0);
                self.player.energy =
                    (self.player.energy + 5.0).min(self.player.max_energy() as f32);
            }
            _ => return false,
        }
        if !matches!(self.menu, Menu::None) {
            io.audio.play(Sfx::UiSelect);
        }
        true
    }

    fn harvest(&mut self, x: i32, z: i32, crop: Crop, io: &mut Io) {
        let def = crop.def();
        let mut n = 1 + self.rng.below(def.yield_max as usize) as u16;
        if self.rng.chance(self.player.sheet.frac(Stat::Bounty, 90)) {
            n += 1;
            self.fx
                .popup(tile_center(x, z) + Vec3::Y * 0.8, "Bounty!", ORANGE);
        }
        let mut left = self.player.inv.add(def.produce, n);
        if left > 0 {
            let at = tile_center(x, z);
            while left > 0 {
                self.drops
                    .push(Drop::item(def.produce, 1, at, &mut self.rng));
                left -= 1;
            }
        } else {
            self.toast(def.produce.def().name, Some(def.produce), n as u32);
        }
        // Crops sometimes give a seed back.
        if crop != Crop::Turnip
            && self
                .rng
                .chance(0.12 + self.player.sheet.frac(Stat::Forage, 60))
        {
            if let Some(s) = Item::seed_of(crop) {
                if self.player.inv.add(s, 1) == 0 {
                    self.toast(s.def().name, Some(s), 1);
                }
            }
        }
        if def.regrow > 0 {
            self.farm.set_obj(
                x,
                z,
                Some(Obj::Crop {
                    crop,
                    days: def.days - def.regrow,
                    harvested: true,
                }),
            );
        } else {
            self.farm.set_obj(x, z, None);
        }
        self.stats.harvested += n as u32;
        io.audio.play(Sfx::Harvest);
        self.fx.motes(
            tile_center(x, z) + Vec3::Y * 0.3,
            8,
            &[CREAM, WHITE, LIME],
            0.3,
        );
    }

    fn open_loot(&mut self, x: i32, z: i32, io: &mut Io) {
        let depth = self.depth().max(1);
        let at = tile_center(x, z);
        let fortune = self.fortune();
        let loot = loot::chest_loot(depth, biome_for(depth), fortune, &mut self.rng);
        self.spill(loot, at, io);
        self.fx
            .motes(at + Vec3::Y * 0.4, 16, &[GOLD, CREAM, WHITE], 0.4);
    }

    /// Throws loot out onto the floor, with a chime for anything rare.
    pub fn spill(&mut self, loot: Vec<Stack>, at: Vec3, io: &mut Io) {
        let mut best = None;
        for s in loot {
            best = best.max(s.rarity());
            self.drops.push(Drop::new(s, at, &mut self.rng));
        }
        if best >= Some(Rarity::Rare) {
            io.audio.play(Sfx::Rare);
        }
    }

    /// The moment a swing, cast or tool use connects.
    fn fire(&mut self, kind: ActKind, (x, z): (i32, i32), dir: Vec2, io: &mut Io) {
        match kind {
            ActKind::Slash => {
                let dmg = self.player.weapon_damage();
                let lvl = self
                    .player
                    .held_stack()
                    .and_then(|s| s.gear)
                    .map_or(1, |g| g.level);
                let reach = 1.35 + (lvl as f32 / 60.0).min(1.0) * 0.25;
                let hits = self.melee(dmg, reach, 1.15, dir, 6.0, io);
                // Cut grass and smash pots in the arc.
                let p = self.player.pos;
                for (dx, dz) in [(0.0, 0.0), (0.7, 0.0), (-0.7, 0.0), (0.0, 0.7), (0.0, -0.7)] {
                    let q = p + dir * 0.9 + Vec2::new(dx, dz) * 0.5;
                    let (tx, tz) = (q.x.floor() as i32, q.y.floor() as i32);
                    self.hit_soft(tx, tz, io);
                }
                if hits == 0 {
                    self.hit_soft(x, z, io);
                }
            }
            ActKind::Bolt => self.cast_bolt(dir, io),
            ActKind::Blast => {
                let at = self.blast_point(dir);
                self.cast_blast(at, io);
            }
            ActKind::Mine => {
                let dmg = self.player.weapon_damage();
                if self.melee(dmg, 1.0, 0.9, dir, 4.0, io) == 0 {
                    let power = self.player.tool_power();
                    self.mine(x, z, power, io);
                }
            }
            ActKind::Chop => {
                let dmg = self.player.weapon_damage();
                if self.melee(dmg, 1.0, 0.9, dir, 4.0, io) == 0 {
                    let power = self.player.tool_power();
                    self.chop(x, z, power, io);
                }
            }
            ActKind::Till => {
                let n = self
                    .player
                    .held_stack()
                    .and_then(|s| s.main_value())
                    .unwrap_or(1)
                    + self.player.reach();
                for (tx, tz) in self.line((x, z), n) {
                    self.till(tx, tz, io);
                }
            }
            ActKind::Water => {
                let n = 1 + self.player.reach();
                if self.world().floor(x, z) == Floor::Water {
                    self.water(x, z, io);
                } else {
                    for (tx, tz) in self.line((x, z), n) {
                        if self.world().floor(tx, tz) != Floor::Water {
                            self.water(tx, tz, io);
                        }
                    }
                }
            }
            ActKind::Reap => self.reap_patch((x, z), dir, io),
        }
    }

    /// Where a staff blast lands: under the mouse when it is close, otherwise ahead.
    fn blast_point(&self, dir: Vec2) -> Vec2 {
        let p = self.player.pos;
        let ahead = p + dir.normalize_or_zero() * 2.4;
        let mut at = match self.aim {
            Some(a) if (a - p).length() < 4.5 => a,
            _ => ahead,
        };
        // Keep it on this side of walls.
        let w = self.world();
        if !w.clear_line(p, at) || w.opaque(at.x.floor() as i32, at.y.floor() as i32) {
            at = p + dir.normalize_or_zero() * 1.2;
        }
        at
    }

    /// `n` tiles in a row, starting at `start` and heading away from the hero.
    fn line(&self, start: (i32, i32), n: i32) -> Vec<(i32, i32)> {
        let (px, pz) = self.player.tile();
        let (mut dx, mut dz) = (start.0 - px, start.1 - pz);
        if dx == 0 && dz == 0 {
            let f = self.player.facing;
            if f.x.abs() > f.y.abs() {
                dx = f.x.signum() as i32;
            } else {
                dz = f.y.signum() as i32;
            }
        } else if dx != 0 && dz != 0 {
            // Diagonal: go along the stronger facing axis.
            let f = self.player.facing;
            if f.x.abs() > f.y.abs() {
                dz = 0;
            } else {
                dx = 0;
            }
        }
        (0..n.max(1))
            .map(|k| (start.0 + dx.signum() * k, start.1 + dz.signum() * k))
            .collect()
    }

    /// A sickle sweep: harvests ripe crops and cuts grass in a patch in front, and nicks
    /// anything in the way.
    fn reap_patch(&mut self, (x, z): (i32, i32), dir: Vec2, io: &mut Io) {
        let dmg = self.player.weapon_damage();
        self.melee(dmg, 1.4, 1.3, dir, 4.0, io);
        let r = 1 + self.player.reach() / 2;
        let mut any = false;
        for dz in -r..=r {
            for dx in -r..=r {
                let (tx, tz) = (x + dx, z + dz);
                match self.world().obj(tx, tz).cloned() {
                    Some(Obj::Crop { crop, days, .. }) if crop.stage(days) == 3 => {
                        if self.area == Area::Farm {
                            self.harvest(tx, tz, crop, io);
                            any = true;
                        }
                    }
                    Some(Obj::Weed { .. } | Obj::Flower { .. }) => {
                        self.hit_soft(tx, tz, io);
                        any = true;
                    }
                    _ => {}
                }
            }
        }
        if !any {
            self.hit_soft(x, z, io);
        }
    }

    /// Sword/any-hit things: weeds, flowers, pots, crates.
    pub fn hit_soft(&mut self, x: i32, z: i32, io: &mut Io) {
        let Some(o) = self.world().obj(x, z).cloned() else {
            return;
        };
        let at = tile_center(x, z);
        match o {
            Obj::Weed { .. } => {
                self.world_mut().set_obj(x, z, None);
                self.fx
                    .burst(at + Vec3::Y * 0.2, 8, &[GREEN, LIME, TEAL], 1.8, 1.8);
                if self.rng.chance(0.8) {
                    self.drops
                        .push(Drop::item(Item::Fiber, 1, at, &mut self.rng));
                }
                if self
                    .rng
                    .chance(0.04 + self.player.sheet.frac(Stat::Forage, 60) * 0.5)
                {
                    let seed = self.forage_seed();
                    self.drops.push(Drop::item(seed, 1, at, &mut self.rng));
                }
                io.audio.play_at(Sfx::Swing, 0.5, 1.4);
            }
            Obj::Flower { .. } => {
                self.world_mut().set_obj(x, z, None);
                self.fx
                    .burst(at + Vec3::Y * 0.2, 8, &[PINK, WHITE, SKY], 1.8, 1.8);
                if self.rng.chance(0.5) {
                    self.drops
                        .push(Drop::item(Item::Fiber, 1, at, &mut self.rng));
                }
            }
            Obj::Pot { .. } | Obj::Crate { .. } => {
                self.world_mut().set_obj(x, z, None);
                io.audio.play(Sfx::Break);
                let col = if matches!(o, Obj::Pot { .. }) {
                    [CLAY, GOLD, RUST]
                } else {
                    [CLAY, RUST, SAND]
                };
                self.fx.burst(at + Vec3::Y * 0.3, 12, &col, 2.5, 2.0);
                let depth = self.depth().max(1);
                let fortune = self.fortune();
                let loot = loot::pot_loot(depth, biome_for(depth), fortune, &mut self.rng);
                self.spill(loot, at, io);
            }
            _ => {}
        }
    }

    /// A seed found while working: from the Hollow's biome, or the valley's own on the farm.
    fn forage_seed(&mut self) -> Item {
        match self.area {
            Area::Hollow { depth } => loot::biome_seed(biome_for(depth), &mut self.rng),
            Area::Farm => {
                let seeds = [
                    Item::TurnipSeeds,
                    Item::RadishSeeds,
                    Item::WheatSeeds,
                    Item::SeedPotato,
                    Item::PeaSeeds,
                    Item::GarlicBulb,
                    Item::RoseSeeds,
                    Item::CarrotSeeds,
                ];
                seeds[self.rng.below(seeds.len())]
            }
        }
    }

    /// A lucky extra drop (Bounty) when breaking something.
    fn bounty(&mut self) -> bool {
        let b = self.player.sheet.frac(Stat::Bounty, 90);
        b > 0.0 && self.rng.chance(b)
    }

    fn mine(&mut self, x: i32, z: i32, power: i32, io: &mut Io) {
        let at = tile_center(x, z);
        let depth = self.depth();
        let wall = self.world().wall(x, z);
        if matches!(wall, Wall::Rock | Wall::Ore(_) | Wall::Brick | Wall::Timber) {
            let hard = match wall {
                Wall::Rock => 3 + depth as i32 / 6,
                Wall::Ore(o) => 5 + depth as i32 / 6 + o as i32,
                _ => 4,
            };
            let w = self.world_mut();
            let i = w.idx(x, z);
            let dmg = w.wall_dmg.entry(i).or_insert(0);
            *dmg += power as i16;
            let broke = *dmg as i32 >= hard;
            let face = at + Vec3::new(0.0, 0.5, 0.5);
            let chips = match wall {
                Wall::Ore(o) => crate::assets::ORE_COLORS[o as usize % 6].to_vec(),
                Wall::Timber => vec![CLAY, RUST],
                _ => vec![KHAKI, ROSEWOOD, SAND],
            };
            self.fx.burst(face, 5, &chips, 2.0, 1.5);
            io.audio.play(Sfx::Mine);
            if broke {
                self.world_mut().set_wall(x, z, Wall::None);
                io.audio.play(Sfx::Break);
                self.fx.burst(at + Vec3::Y * 0.5, 14, &chips, 3.0, 2.0);
                let forage = self.player.sheet.frac(Stat::Forage, 60);
                match wall {
                    Wall::Ore(o) => {
                        let n = 1 + self.rng.below(2) as u16 + u16::from(self.bounty());
                        self.drops
                            .push(Drop::item(ore_item(o), n, at, &mut self.rng));
                        if self.rng.chance(0.5) {
                            self.drops
                                .push(Drop::item(Item::Stone, 1, at, &mut self.rng));
                        }
                        if self.rng.chance(0.06 + forage) {
                            let gem = loot::random_gem(depth, &mut self.rng);
                            self.drops.push(Drop::item(gem, 1, at, &mut self.rng));
                            self.fx.motes(at + Vec3::Y * 0.5, 8, &[WHITE, CREAM], 0.3);
                        }
                    }
                    Wall::Rock => {
                        if self.rng.chance(0.55) {
                            let n = 1 + u16::from(self.bounty());
                            self.drops
                                .push(Drop::item(Item::Stone, n, at, &mut self.rng));
                        }
                        if self.rng.chance(0.03) {
                            self.drops
                                .push(Drop::item(Item::Amber, 1, at, &mut self.rng));
                        }
                        if self.rng.chance(0.012 + forage * 0.25) {
                            let gem = loot::random_gem(depth, &mut self.rng);
                            self.drops.push(Drop::item(gem, 1, at, &mut self.rng));
                        }
                        if self.rng.chance(0.004 + forage * 0.02) {
                            let relic = loot::random_relic(depth, &mut self.rng);
                            self.drops.push(Drop::item(relic, 1, at, &mut self.rng));
                        }
                    }
                    Wall::Brick => {
                        self.drops
                            .push(Drop::item(Item::StoneWall, 1, at, &mut self.rng))
                    }
                    Wall::Timber => {
                        self.drops
                            .push(Drop::item(Item::WoodWall, 1, at, &mut self.rng))
                    }
                    _ => {}
                }
                self.shake = self.shake.max(0.3);
            }
            return;
        }
        let Some(o) = self.world().obj(x, z).cloned() else {
            // Pick up placed floors.
            let f = self.world().floor(x, z);
            if matches!(f, Floor::Planks | Floor::Cobble) {
                let back = if self.area == Area::Farm {
                    Floor::Grass
                } else {
                    Floor::Cave
                };
                self.world_mut().set_floor(x, z, back);
                let item = if f == Floor::Planks {
                    Item::WoodPath
                } else {
                    Item::StonePath
                };
                self.drops.push(Drop::item(item, 1, at, &mut self.rng));
                io.audio.play(Sfx::Place);
            }
            return;
        };
        match o {
            Obj::Rock { hp, var } => {
                let hp = hp - power as i16;
                io.audio.play(Sfx::Mine);
                self.fx
                    .burst(at + Vec3::Y * 0.2, 5, &[SAND, KHAKI, ROSEWOOD], 2.0, 1.5);
                if hp <= 0 {
                    self.world_mut().set_obj(x, z, None);
                    io.audio.play(Sfx::Break);
                    let n = 1 + self.rng.below(2) as u16 + u16::from(self.bounty());
                    self.drops
                        .push(Drop::item(Item::Stone, n, at, &mut self.rng));
                    if self.area == Area::Farm && self.rng.chance(0.15) {
                        self.drops
                            .push(Drop::item(Item::CopperOre, 1, at, &mut self.rng));
                    }
                    if self
                        .rng
                        .chance(0.015 + self.player.sheet.frac(Stat::Forage, 60) * 0.3)
                    {
                        let gem = loot::random_gem(depth.max(1), &mut self.rng);
                        self.drops.push(Drop::item(gem, 1, at, &mut self.rng));
                    }
                } else {
                    self.world_mut().set_obj(x, z, Some(Obj::Rock { hp, var }));
                }
            }
            Obj::Boulder { hp } => {
                let hp = hp - power as i16;
                io.audio.play(Sfx::Mine);
                self.fx
                    .burst(at + Vec3::Y * 0.4, 6, &[SAND, KHAKI, ROSEWOOD], 2.0, 1.5);
                if hp <= 0 {
                    self.world_mut().set_obj(x, z, None);
                    io.audio.play(Sfx::Break);
                    self.shake = 0.5;
                    let n = 5 + self.rng.below(4) as u16;
                    self.drops
                        .push(Drop::item(Item::Stone, n, at, &mut self.rng));
                    if self.rng.chance(0.5) {
                        self.drops
                            .push(Drop::item(Item::IronOre, 1, at, &mut self.rng));
                    }
                } else {
                    self.world_mut().set_obj(x, z, Some(Obj::Boulder { hp }));
                }
            }
            Obj::Crystal { hp, var } => {
                let hp = hp - power as i16;
                io.audio.play_at(Sfx::Mine, 1.0, 1.3);
                self.fx
                    .burst(at + Vec3::Y * 0.4, 6, &[MINT, AQUA, WHITE], 2.0, 1.5);
                if hp <= 0 {
                    self.world_mut().set_obj(x, z, None);
                    let item = match var {
                        3 | 5 => Item::Amber,
                        4 => Item::FrostGem,
                        _ => Item::Crystal,
                    };
                    let n = 1 + self.rng.below(2) as u16 + u16::from(self.bounty());
                    self.drops.push(Drop::item(item, n, at, &mut self.rng));
                    if self.rng.chance(0.08) {
                        let gem = loot::random_gem(depth.max(1), &mut self.rng);
                        self.drops.push(Drop::item(gem, 1, at, &mut self.rng));
                    }
                } else {
                    self.world_mut()
                        .set_obj(x, z, Some(Obj::Crystal { hp, var }));
                }
            }
            Obj::Stalagmite { .. } => {
                self.world_mut().set_obj(x, z, None);
                io.audio.play(Sfx::Break);
                self.fx
                    .burst(at + Vec3::Y * 0.4, 10, &[KHAKI, ROSEWOOD], 2.5, 2.0);
                self.drops
                    .push(Drop::item(Item::Stone, 1, at, &mut self.rng));
            }
            Obj::Pot { .. } | Obj::Crate { .. } => self.hit_soft(x, z, io),
            Obj::Lamp
            | Obj::Sprinkler { .. }
            | Obj::FlowerPot { .. }
            | Obj::Torch
            | Obj::Chest { .. }
            | Obj::EnchantTable => {
                self.pick_up(x, z, o, io);
            }
            _ => {}
        }
    }

    fn chop(&mut self, x: i32, z: i32, power: i32, io: &mut Io) {
        let at = tile_center(x, z);
        if self.world().wall(x, z) == Wall::Timber {
            self.world_mut().set_wall(x, z, Wall::None);
            self.drops
                .push(Drop::item(Item::WoodWall, 1, at, &mut self.rng));
            io.audio.play(Sfx::Chop);
            return;
        }
        let Some(o) = self.world().obj(x, z).cloned() else {
            return;
        };
        let leaves = [GREEN, LIME, TEAL];
        match o {
            Obj::Tree { hp, var } | Obj::Pine { hp, var } => {
                let hp = hp - power as i16;
                io.audio.play(Sfx::Chop);
                self.fx.burst(at + Vec3::Y * 1.0, 8, &leaves, 1.8, 1.0);
                self.fx
                    .burst(at + Vec3::Y * 0.3, 4, &[CLAY, RUST], 1.5, 1.5);
                if hp <= 0 {
                    self.world_mut().set_obj(x, z, Some(Obj::Stump { hp: 6 }));
                    io.audio.play(Sfx::Break);
                    self.shake = 0.5;
                    let n = 4 + self.rng.below(3) as u16 + 3 * u16::from(self.bounty());
                    self.drops
                        .push(Drop::item(Item::Wood, n, at, &mut self.rng));
                    if self.rng.chance(0.25) {
                        self.drops
                            .push(Drop::item(Item::Fiber, 2, at, &mut self.rng));
                    }
                    if self
                        .rng
                        .chance(0.05 + self.player.sheet.frac(Stat::Forage, 60))
                    {
                        let seed = self.forage_seed();
                        self.drops.push(Drop::item(seed, 1, at, &mut self.rng));
                    }
                    if self.rng.chance(0.01) {
                        self.drops
                            .push(Drop::item(Item::GoldenAcorn, 1, at, &mut self.rng));
                    }
                    self.fx.burst(at + Vec3::Y * 1.2, 20, &leaves, 2.5, 1.0);
                } else {
                    let o = if matches!(o, Obj::Tree { .. }) {
                        Obj::Tree { hp, var }
                    } else {
                        Obj::Pine { hp, var }
                    };
                    self.world_mut().set_obj(x, z, Some(o));
                }
            }
            Obj::Stump { hp } | Obj::Log { hp } => {
                let hp = hp - power as i16;
                io.audio.play(Sfx::Chop);
                self.fx
                    .burst(at + Vec3::Y * 0.2, 5, &[CLAY, RUST, SAND], 1.8, 1.5);
                if hp <= 0 {
                    self.world_mut().set_obj(x, z, None);
                    io.audio.play(Sfx::Break);
                    let n = if matches!(o, Obj::Log { .. }) {
                        8
                    } else {
                        2 + self.rng.below(2) as u16
                    };
                    self.drops
                        .push(Drop::item(Item::Wood, n, at, &mut self.rng));
                } else {
                    let o = if matches!(o, Obj::Log { .. }) {
                        Obj::Log { hp }
                    } else {
                        Obj::Stump { hp }
                    };
                    self.world_mut().set_obj(x, z, Some(o));
                }
            }
            Obj::Weed { .. } => self.hit_soft(x, z, io),
            Obj::Crate { .. } => self.hit_soft(x, z, io),
            Obj::Fence | Obj::Bench | Obj::Workbench | Obj::Chest { .. } => {
                self.pick_up(x, z, o, io)
            }
            _ => {}
        }
    }

    /// Returns a placed object to the player as an item.
    fn pick_up(&mut self, x: i32, z: i32, o: Obj, io: &mut Io) {
        let at = tile_center(x, z);
        let item = match &o {
            Obj::Lamp => Item::Lamp,
            Obj::Sprinkler { tier } => [
                Item::Sprinkler,
                Item::QualitySprinkler,
                Item::CrystalSprinkler,
            ][*tier as usize % 3],
            Obj::FlowerPot { .. } => Item::FlowerPot,
            Obj::Torch => Item::Torch,
            Obj::Fence => Item::Fence,
            Obj::Bench => Item::Bench,
            Obj::Workbench => Item::Workbench,
            Obj::Chest { items } => {
                for s in items.iter().flatten() {
                    self.drops.push(Drop::new(*s, at, &mut self.rng));
                }
                Item::Chest
            }
            Obj::EnchantTable => Item::EnchantTable,
            _ => return,
        };
        self.world_mut().set_obj(x, z, None);
        self.drops.push(Drop::item(item, 1, at, &mut self.rng));
        io.audio.play(Sfx::Place);
        self.fx
            .burst(at + Vec3::Y * 0.3, 6, &[SAND, KHAKI], 1.5, 1.5);
    }

    fn till(&mut self, x: i32, z: i32, io: &mut Io) {
        if self.area != Area::Farm {
            return;
        }
        let at = tile_center(x, z);
        if let Some(Obj::Weed { .. } | Obj::Flower { .. }) = self.farm.obj(x, z) {
            self.hit_soft(x, z, io);
        }
        if matches!(self.farm.floor(x, z), Floor::Grass | Floor::Soil)
            && self.farm.obj(x, z).is_none()
            && self.farm.wall(x, z) == Wall::None
        {
            self.farm.set_floor(x, z, Floor::Tilled);
            io.audio.play(Sfx::Till);
            self.fx
                .burst(at + Vec3::Y * 0.05, 8, &[RUST, CLAY, MAROON], 1.6, 1.8);
            let forage = self.player.sheet.frac(Stat::Forage, 60);
            if self.rng.chance(0.03 + forage * 0.4) {
                let seed = self.forage_seed();
                self.drops.push(Drop::item(seed, 1, at, &mut self.rng));
            }
            if self.rng.chance(0.004 + forage * 0.02) {
                let relic = loot::random_relic(1, &mut self.rng);
                self.drops.push(Drop::item(relic, 1, at, &mut self.rng));
            }
        }
    }

    fn water(&mut self, x: i32, z: i32, io: &mut Io) {
        let at = tile_center(x, z);
        let floor = self.world().floor(x, z);
        if floor == Floor::Water {
            self.player.water = self.player.can_capacity();
            io.audio.play(Sfx::Refill);
            self.fx
                .burst(at + Vec3::Y * 0.1, 10, &[WHITE, SKY, BLUE], 1.8, 2.2);
            self.toast("Watering can refilled", None, 0);
            return;
        }
        if self.player.water == 0 {
            self.toast("Your can is empty. Refill it at the pond.", None, 0);
            io.audio.play(Sfx::Denied);
            return;
        }
        self.player.water -= 1;
        io.audio.play(Sfx::Water);
        self.fx
            .burst(at + Vec3::Y * 0.4, 10, &[WHITE, SKY, BLUE], 1.2, 0.5);
        if floor == Floor::Tilled {
            self.world_mut().set_flag(x, z, WATERED, true);
            let growth = self.player.sheet.frac(Stat::Growth, 60);
            if growth > 0.0 && self.rng.chance(growth) {
                self.world_mut().set_flag(x, z, FERTILE, true);
                self.fx
                    .motes(at + Vec3::Y * 0.2, 5, &[LIME, MINT, WHITE], 0.25);
            }
        }
    }

    // --------------------------------------------------------------------------------------
    // Foes, drops, the cat
    // --------------------------------------------------------------------------------------

    fn update_foes(&mut self, io: &mut Io) {
        if self.foes.is_empty() && self.shots.is_empty() {
            return;
        }
        let dt = io.dt;
        let depth = self.depth();
        let Some(level) = &self.level else { return };
        let world = &level.world;
        let ppos = self.player.pos;
        let mut spawns = Vec::new();
        let mut shots = std::mem::take(&mut self.shots);
        let before = shots.len();
        for f in self.foes.iter_mut() {
            let was_alert = f.alert;
            // Chilled creatures move (and think) at half speed.
            let fdt = if f.chill > 0.0 { dt * 0.5 } else { dt };
            f.update(
                fdt,
                world,
                ppos,
                &mut shots,
                &mut spawns,
                &mut self.fx,
                &mut self.rng,
                depth,
            );
            if !was_alert && f.alert {
                io.audio.play_at(Sfx::Alert, 0.6, 1.0);
                if f.boss {
                    self.boss_seen = Some(boss_name(f.foe).to_string());
                }
            }
        }
        if shots.len() > before {
            io.audio.play(Sfx::Shoot);
        }
        // Keep enemies from stacking up.
        let n = self.foes.len();
        for i in 0..n {
            for j in (i + 1)..n {
                let d = self.foes[j].pos - self.foes[i].pos;
                let min = self.foes[i].radius + self.foes[j].radius;
                let l = d.length();
                if l < min && l > 1e-4 {
                    let push = d / l * (min - l) * 0.5;
                    self.foes[i].pos -= push;
                    self.foes[j].pos += push;
                }
            }
        }
        if !spawns.is_empty() && self.foes.len() < 40 {
            self.foes.extend(spawns);
        }
        // Contact damage.
        let mut hurt: Option<(i32, Vec2, Option<usize>)> = None;
        for (i, f) in self.foes.iter().enumerate() {
            let d = ppos - f.pos;
            if d.length() < f.radius + RADIUS && f.grounded() && f.flash <= 0.0 && f.hp > 0 {
                hurt = Some((f.dmg, d.normalize_or_zero(), Some(i)));
            }
        }
        // Bats back off after touching you.
        if hurt.is_some() {
            for f in self.foes.iter_mut() {
                if f.foe == dungeon::Foe::Bat && (f.pos - ppos).length() < 0.8 {
                    f.st = St::Rest;
                    f.t = 0.8;
                }
            }
        }
        let world = &self.level.as_ref().unwrap().world;
        shots.retain_mut(|s| {
            if !s.update(dt, world) {
                return false;
            }
            if (s.pos - ppos).length() < s.radius + RADIUS {
                hurt = Some((s.dmg, s.vel.normalize_or_zero(), None));
                return false;
            }
            true
        });
        self.shots = shots;
        if let Some((dmg, dir, from)) = hurt {
            self.hurt_player(dmg, dir, from, io);
            self.reap(io);
        }
    }

    fn update_drops(&mut self, io: &mut Io) {
        let dt = io.dt;
        let world = match &self.level {
            Some(l) if matches!(self.area, Area::Hollow { .. }) => &l.world,
            _ => &self.farm,
        };
        let ppos = self.player.pos;
        let mut picked: Vec<usize> = Vec::new();
        for (i, d) in self.drops.iter_mut().enumerate() {
            let can = d.is_coin() || self.player.inv.can_fit_stack(&d.stack);
            if d.update(dt, world, ppos, can) {
                picked.push(i);
            }
        }
        for i in picked.into_iter().rev() {
            let d = self.drops.remove(i);
            let item = d.stack.item;
            if let Some(v) = item.coin_value() {
                let worth = v as u64 * d.stack.n as u64;
                self.money += worth;
                self.stats.earned += worth;
                self.toast(item.def().name, Some(item), d.stack.n as u32);
                io.audio
                    .play_at(Sfx::Coin, 0.6, 0.95 + self.rng.f32() * 0.15);
                continue;
            }
            let left = self.player.inv.add_stack(d.stack);
            let got = d.stack.n - left;
            if got > 0 {
                let name = d.stack.name();
                let color = d.stack.rarity().map_or(CREAM, |r| r.color());
                self.toast_colored(name, Some(item), got as u32, color);
                let chime = d.stack.rarity().is_some_and(|r| r >= Rarity::Rare);
                if chime {
                    io.audio.play(Sfx::Rare);
                } else {
                    io.audio
                        .play_at(Sfx::Pickup, 0.8, 1.0 + self.rng.f32() * 0.1);
                }
            }
        }
    }

    fn update_cat(&mut self, dt: f32) {
        if self.area != Area::Farm {
            return;
        }
        let c = &mut self.cat;
        c.t -= dt;
        c.pet = (c.pet - dt).max(0.0);
        if c.t <= 0.0 {
            c.t = self.rng.range_f(2.0, 5.0);
            let near_player = self.rng.chance(0.4);
            let base = if near_player {
                self.player.pos
            } else {
                Vec2::new(27.0, 12.0)
            };
            c.target = base + Vec2::new(self.rng.range_f(-3.0, 3.0), self.rng.range_f(-2.0, 2.0));
        }
        let d = c.target - c.pos;
        if d.length() > 0.2 && c.pet <= 0.0 {
            let step = d.normalize() * 1.2 * dt;
            c.pos = self.farm.move_circle(c.pos, step, 0.2);
            c.yaw = step.x.atan2(step.y);
        }
    }

    fn reveal(&mut self) {
        let Some(l) = &self.level else { return };
        let w = &l.world;
        let (px, pz) = self.player.tile();
        let r = 7;
        for z in (pz - r).max(0)..=(pz + r).min(w.h - 1) {
            for x in (px - r).max(0)..=(px + r).min(w.w - 1) {
                if (x - px).pow(2) + (z - pz).pow(2) <= r * r {
                    let i = (z * w.w + x) as usize;
                    if i < self.revealed.len() {
                        self.revealed[i] = true;
                    }
                }
            }
        }
    }
}

pub fn tile_center(x: i32, z: i32) -> Vec3 {
    Vec3::new(x as f32 + 0.5, 0.0, z as f32 + 0.5)
}

/// Moves the stack in `slot` of `from` into `to`, as much as fits.
pub fn transfer(from: &mut Inventory, slot: usize, to: &mut Inventory) {
    if let Some(s) = from.slots[slot] {
        let left = to.add_stack(s);
        from.slots[slot] = if left > 0 {
            Some(Stack { n: left, ..s })
        } else {
            None
        };
    }
}
