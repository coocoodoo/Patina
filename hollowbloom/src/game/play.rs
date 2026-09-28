//! The playing state: the farm, the Hollow, and everything the player does in them.

use glam::{Mat4, Vec2, Vec3};

use super::draw::{self, Env, Pose, Swing, draw_humanoid, full_uv};
use super::dungeon::{self, Level, biome_for, is_waystone_floor, ore_item};
use super::farm::{self, MARKS};
use super::foes::{Enemy, St, boss_name};
use super::fx::{Drop, Fx, Shot, shot_colors};
use super::items::{Crop, Inventory, Item, Kind, Placeable, Stack, ToolKind};
use super::menus::Menu;
use super::player::{Act, ActKind, HOTBAR, Player, RADIUS, SPEED};
use super::world::{Area, Floor, Obj, WATERED, Wall, World};
use super::{Io, Settings};
use crate::assets::{Assets, BIOME_STYLES};
use crate::audio::{Sfx, Song};
use crate::input::Action;
use crate::palette::*;
use crate::render::{Camera, DrawOpts, Light, Mode, PointLight, Renderer, UvRect};
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
    pub fx: Fx,
    pub rng: Rng,
    pub clock: Clock,
    pub gold: u64,
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
            fx: Fx::default(),
            rng: Rng::new(seed ^ 0xC0FFEE),
            clock: Clock {
                day: 1,
                min: DAY_START + 60.0,
            },
            gold: 150,
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
             Till with the hoe, plant, water every day and sleep to let things grow. Every \
             tenth floor down there, a waystone can bring you home.",
        );
        p
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
        let text = text.into();
        if let Some(t) = self
            .toasts
            .iter_mut()
            .find(|t| t.icon.is_some() && t.icon == icon && t.t < 2.0)
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
            earned += s.item.def().price as u64 * s.n as u64;
        }
        self.gold += earned;
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
        p.energy = if passed_out {
            p.max_energy as f32 * 0.6
        } else {
            p.max_energy as f32
        };
        p.hp = p.max_hp;
        p.water = p.can_capacity();
        let (dx, dz) = MARKS.door;
        p.pos = Vec2::new(dx as f32 + 0.5, dz as f32 + 0.6);
        p.facing = Vec2::new(0.0, 1.0);
        p.act = None;
        self.level = None;
        self.foes.clear();
        self.shots.clear();
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
        let lost = (self.gold / 10).min(1000);
        self.gold -= lost;
        self.sleep(true);
        if let Some(s) = &mut self.last_summary {
            s.fainted = true;
            s.earned = s.earned.saturating_sub(0);
        }
        self.toast(format!("Lost {lost}g while you were out"), None, 0);
        self.player.hp = self.player.max_hp / 2;
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

        self.update_player(io);
        self.update_foes(io);
        self.update_drops(io);
        self.update_cat(dt);
        self.fx.update(dt);
        self.shake = (self.shake - dt * 2.5).max(0.0);

        // Regeneration while on the farm.
        if self.area == Area::Farm
            && self.player.hp < self.player.max_hp
            && (self.time * 2.0) as i32 % 2 == 0
            && self.time.fract() < dt * 2.0
        {
            self.player.hp += 1;
        }
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
        let speed = if tired { SPEED * 0.65 } else { SPEED };
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
        matches!(
            self.player.held().and_then(|i| i.tool()),
            Some((
                ToolKind::Sword | ToolKind::Pickaxe | ToolKind::Axe | ToolKind::Hoe | ToolKind::Can,
                _
            ))
        )
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
        if input.mouse_aim {
            if let Some(g) = self.cam.ground(input.mouse.x, input.mouse.y, 0.0) {
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
            Kind::Tool(ToolKind::Hoe, _) => {
                self.area == Area::Farm
                    && matches!(w.floor(x, z), Floor::Grass | Floor::Soil)
                    && w.wall(x, z) == Wall::None
                    && w.obj(x, z)
                        .is_none_or(|o| matches!(o, Obj::Weed { .. } | Obj::Flower { .. }))
            }
            Kind::Tool(ToolKind::Can, _) => {
                w.floor(x, z) == Floor::Tilled || w.floor(x, z) == Floor::Water
            }
            Kind::Tool(ToolKind::Pickaxe, _) => {
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
                    )
                }) || matches!(w.floor(x, z), Floor::Planks | Floor::Cobble)
            }
            Kind::Tool(ToolKind::Axe, _) => {
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
        let energy = |t: u8| 2.0 - t as f32 * 0.25;
        let kind = match item.def().kind {
            Kind::Tool(ToolKind::Sword, t) => Some((ActKind::Sword(t), 0.0)),
            Kind::Tool(ToolKind::Pickaxe, t) => Some((ActKind::Pick(t), energy(t))),
            Kind::Tool(ToolKind::Axe, t) => Some((ActKind::Axe(t), energy(t))),
            Kind::Tool(ToolKind::Hoe, _) => Some((ActKind::Hoe, 2.0)),
            Kind::Tool(ToolKind::Can, t) => Some((ActKind::Water, energy(t) * 0.7)),
            _ => None,
        };
        if let Some((kind, cost)) = kind {
            let p = &mut self.player;
            p.energy = (p.energy - cost).max(-10.0);
            p.act = Some(Act {
                kind,
                t: 0.0,
                fired: false,
                tile,
                dir,
            });
            if matches!(kind, ActKind::Sword(_)) {
                io.audio.play(Sfx::Swing);
            }
            return;
        }
        self.use_item(item, tile, io);
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
                }
                io.audio.play(Sfx::Place);
                self.fx
                    .burst(tile_center(x, z), 6, &[SAND, KHAKI], 1.2, 1.0);
            }
            Kind::Produce { hp, energy } | Kind::Food { hp, energy } => {
                self.eat(item, hp, energy, io)
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
                self.player.max_hp += 10;
                self.player.hp = self.player.max_hp;
                io.audio.play(Sfx::LevelUp);
                self.fx
                    .motes(self.player.world_pos(), 16, &[PINK, BLUSH, WHITE], 0.4);
                self.toast("Max HP +10", None, 0);
            }
            Kind::SunStone => {
                self.player.inv.take_one(sel);
                self.player.max_energy += 15;
                self.player.energy = self.player.max_energy as f32;
                io.audio.play(Sfx::LevelUp);
                self.fx
                    .motes(self.player.world_pos(), 16, &[GOLD, CREAM, WHITE], 0.4);
                self.toast("Max energy +15", None, 0);
            }
            _ => {}
        }
    }

    fn eat(&mut self, item: Item, hp: i32, energy: i32, io: &mut Io) {
        let p = &mut self.player;
        if p.eat_t > 0.0 {
            return;
        }
        if p.hp >= p.max_hp && p.energy >= p.max_energy as f32 {
            self.toast("You're full.", None, 0);
            return;
        }
        let sel = p.sel;
        p.inv.take_one(sel);
        p.hp = (p.hp + hp).min(p.max_hp);
        p.energy = (p.energy + energy as f32).min(p.max_energy as f32);
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
        self.toast(format!("Ate {}", item.def().name), None, 0);
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
            if item.tool().is_none() {
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
                self.menu = Menu::Shop {
                    cursor: 0,
                    sell: false,
                    scroll: 0,
                };
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
                self.open_loot(ax, az);
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
                self.player.energy = (self.player.energy + 5.0).min(self.player.max_energy as f32);
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
        let n = 1 + self.rng.below(def.yield_max as usize) as u16;
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
        // Dungeon crops sometimes give a seed back.
        if crop != Crop::Turnip && self.rng.chance(0.12) {
            let seed = super::items::ALL_ITEMS
                .iter()
                .copied()
                .find(|i| matches!(i.def().kind, Kind::Seed(c) if c == crop));
            if let Some(s) = seed {
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

    fn open_loot(&mut self, x: i32, z: i32) {
        let depth = self.depth().max(1);
        let biome = biome_for(depth);
        let at = tile_center(x, z);
        let seeds = dungeon::biome_seeds(biome);
        let w: Vec<f32> = seeds.iter().map(|s| s.1).collect();
        let mut loot: Vec<(Item, u16)> = Vec::new();
        loot.push((seeds[self.rng.weighted(&w)].0, 2 + self.rng.below(3) as u16));
        let ores = [
            Item::CopperOre,
            Item::IronOre,
            Item::GoldOre,
            Item::Crystal,
            Item::EmberOre,
            Item::FrostGem,
        ];
        let top = ((depth / 9) as usize).min(5);
        loot.push((ores[self.rng.below(top + 1)], 3 + self.rng.below(4) as u16));
        let foods = [
            Item::HealingTonic,
            Item::VeggieStew,
            Item::StaminaTonic,
            Item::GlowSoup,
        ];
        loot.push((foods[self.rng.below(foods.len())], 1));
        if self.rng.chance(0.25) {
            loot.push((Item::Feather, 1));
        }
        if self.rng.chance(0.06 + depth as f32 * 0.002) {
            loot.push((
                if self.rng.chance(0.5) {
                    Item::HeartCrystal
                } else {
                    Item::SunStone
                },
                1,
            ));
        }
        if self.rng.chance(0.2) {
            loot.push((Item::Amber, 1));
        }
        for (item, n) in loot {
            self.drops.push(Drop::item(item, n, at, &mut self.rng));
        }
        let gold = 10 + depth * 4 + self.rng.below(20) as u32;
        self.drops.push(Drop::coins(gold, at, &mut self.rng));
        self.fx
            .motes(at + Vec3::Y * 0.4, 16, &[GOLD, CREAM, WHITE], 0.4);
    }

    /// The moment a swing connects.
    fn fire(&mut self, kind: ActKind, (x, z): (i32, i32), dir: Vec2, io: &mut Io) {
        match kind {
            ActKind::Sword(t) => {
                let dmg = self.player.sword_damage(t);
                let reach = 1.35 + t as f32 * 0.05;
                let hits = self.melee(dmg, reach, 1.15, dir, io);
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
            ActKind::Pick(t) => {
                let power = 2 + t as i32 * 2;
                if self.melee(
                    self.player.sword_damage(0) / 2 + t as i32 * 2,
                    1.0,
                    0.9,
                    dir,
                    io,
                ) == 0
                {
                    self.mine(x, z, power, io);
                }
            }
            ActKind::Axe(t) => {
                let power = 2 + t as i32 * 2;
                if self.melee(
                    self.player.sword_damage(0) / 2 + t as i32 * 2,
                    1.0,
                    0.9,
                    dir,
                    io,
                ) == 0
                {
                    self.chop(x, z, power, io);
                }
            }
            ActKind::Hoe => self.till(x, z, io),
            ActKind::Water => self.water(x, z, io),
        }
    }

    /// Hits enemies in an arc. Returns how many were hit.
    fn melee(&mut self, dmg: i32, reach: f32, half_angle: f32, dir: Vec2, io: &mut Io) -> usize {
        let p = self.player.pos;
        let mut hits = 0;
        let mut xp = 0;
        let mut dead = Vec::new();
        for (i, f) in self.foes.iter_mut().enumerate() {
            let d = f.pos - p;
            let dist = d.length();
            if dist > reach + f.radius || f.hurt_cd > 0.0 {
                continue;
            }
            let ang = d.normalize_or_zero().angle_to(dir).abs();
            if dist > 0.4 && ang > half_angle {
                continue;
            }
            let crit = self.rng.chance(0.08);
            let amount = if crit { dmg * 2 } else { dmg } + self.rng.below(3) as i32;
            f.hp -= amount;
            f.flash = 0.12;
            f.hurt_cd = 0.22;
            let heavy = if f.boss || f.foe == dungeon::Foe::Golem {
                0.3
            } else {
                1.0
            };
            f.vel = d.normalize_or_zero() * 6.0 * heavy;
            if f.st == St::Windup && !f.boss {
                f.st = St::Rest;
                f.t = 0.4;
            }
            f.alert = true;
            hits += 1;
            self.fx.popup(
                f.world_pos() + Vec3::Y * (0.7 * f.scale()),
                amount.to_string(),
                if crit { GOLD } else { WHITE },
            );
            self.fx.burst(
                f.world_pos() + Vec3::Y * 0.3,
                6,
                &[WHITE, CREAM, f.color()],
                2.5,
                1.5,
            );
            if f.hp <= 0 {
                dead.push(i);
                xp += f.xp;
            }
        }
        if hits > 0 {
            io.audio.play(Sfx::Hit);
            io.audio.play_at(Sfx::EnemyHurt, 0.8, 1.0);
            self.shake = self.shake.max(0.35);
        }
        for i in dead.into_iter().rev() {
            let f = self.foes.remove(i);
            self.kill(f, io);
        }
        if xp > 0 {
            let ups = self.player.gain_xp(xp);
            if ups > 0 {
                io.audio.play(Sfx::LevelUp);
                self.fx
                    .popup_big(self.player.world_pos() + Vec3::Y * 1.2, "LEVEL UP!", GOLD);
                self.fx
                    .motes(self.player.world_pos(), 20, &[GOLD, CREAM, WHITE], 0.5);
                self.toast(
                    format!("Level {}! Max HP {}", self.player.level, self.player.max_hp),
                    None,
                    0,
                );
            }
        }
        hits
    }

    fn kill(&mut self, f: Enemy, io: &mut Io) {
        io.audio.play(Sfx::EnemyDie);
        let at = f.world_pos();
        self.fx.burst(
            at + Vec3::Y * 0.3,
            if f.boss { 40 } else { 14 },
            &[f.color(), WHITE, CREAM],
            3.0,
            2.5,
        );
        for (item, n) in f.loot(&mut self.rng) {
            self.drops.push(Drop::item(item, n, at, &mut self.rng));
        }
        let depth = self.depth().max(1);
        if self.rng.chance(0.6) || f.boss {
            let g = 1 + self.rng.below(3 + depth as usize) as u32 * if f.boss { 12 } else { 1 };
            self.drops.push(Drop::coins(g, at, &mut self.rng));
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

    /// Sword/any-hit things: weeds, flowers, pots, crates.
    fn hit_soft(&mut self, x: i32, z: i32, io: &mut Io) {
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
                if self.rng.chance(0.04) {
                    self.drops
                        .push(Drop::item(Item::TurnipSeeds, 1, at, &mut self.rng));
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
                self.breakable_loot(at);
            }
            _ => {}
        }
    }

    fn breakable_loot(&mut self, at: Vec3) {
        let depth = self.depth().max(1);
        let r = self.rng.f32();
        if r < 0.45 {
            let g = 1 + self.rng.below(2 + depth as usize / 2) as u32;
            self.drops.push(Drop::coins(g, at, &mut self.rng));
        } else if r < 0.6 {
            let seeds = dungeon::biome_seeds(biome_for(depth));
            let w: Vec<f32> = seeds.iter().map(|s| s.1).collect();
            let s = seeds[self.rng.weighted(&w)].0;
            self.drops.push(Drop::item(s, 1, at, &mut self.rng));
        } else if r < 0.72 {
            self.drops
                .push(Drop::item(Item::Torch, 2, at, &mut self.rng));
        } else if r < 0.8 {
            self.drops
                .push(Drop::item(Item::HealingTonic, 1, at, &mut self.rng));
        } else if r < 0.9 {
            self.drops
                .push(Drop::item(Item::SlimeGel, 1, at, &mut self.rng));
        }
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
                match wall {
                    Wall::Ore(o) => {
                        let n = 1 + self.rng.below(2) as u16;
                        self.drops
                            .push(Drop::item(ore_item(o), n, at, &mut self.rng));
                        if self.rng.chance(0.5) {
                            self.drops
                                .push(Drop::item(Item::Stone, 1, at, &mut self.rng));
                        }
                    }
                    Wall::Rock => {
                        if self.rng.chance(0.55) {
                            self.drops
                                .push(Drop::item(Item::Stone, 1, at, &mut self.rng));
                        }
                        if self.rng.chance(0.03) {
                            self.drops
                                .push(Drop::item(Item::Amber, 1, at, &mut self.rng));
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
                    let n = 1 + self.rng.below(2) as u16;
                    self.drops
                        .push(Drop::item(Item::Stone, n, at, &mut self.rng));
                    if self.area == Area::Farm && self.rng.chance(0.15) {
                        self.drops
                            .push(Drop::item(Item::CopperOre, 1, at, &mut self.rng));
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
                    let n = 1 + self.rng.below(2) as u16;
                    self.drops.push(Drop::item(item, n, at, &mut self.rng));
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
            | Obj::Chest { .. } => {
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
                    let n = 4 + self.rng.below(3) as u16;
                    self.drops
                        .push(Drop::item(Item::Wood, n, at, &mut self.rng));
                    if self.rng.chance(0.25) {
                        self.drops
                            .push(Drop::item(Item::Fiber, 2, at, &mut self.rng));
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
                    self.drops.push(Drop::item(s.item, s.n, at, &mut self.rng));
                }
                Item::Chest
            }
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
            if self.rng.chance(0.03) {
                self.drops
                    .push(Drop::item(Item::CarrotSeeds, 1, at, &mut self.rng));
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
            f.update(
                dt,
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
        let mut hurt: Option<(i32, Vec2)> = None;
        for f in &self.foes {
            let d = ppos - f.pos;
            if d.length() < f.radius + RADIUS && f.grounded() && f.flash <= 0.0 {
                hurt = Some((f.dmg, d.normalize_or_zero()));
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
                hurt = Some((s.dmg, s.vel.normalize_or_zero()));
                return false;
            }
            true
        });
        self.shots = shots;
        if let Some((dmg, dir)) = hurt {
            self.hurt_player(dmg, dir, io);
        }
    }

    pub fn hurt_player(&mut self, dmg: i32, dir: Vec2, io: &mut Io) {
        let p = &mut self.player;
        if p.invulnerable() {
            return;
        }
        p.hp -= dmg;
        p.hurt = 0.9;
        p.flash = 0.25;
        p.vel = dir * 7.0;
        p.act = None;
        self.shake = self.shake.max(0.6);
        io.audio.play(Sfx::PlayerHurt);
        let pos = p.world_pos() + Vec3::Y * 1.0;
        self.fx.popup(pos, format!("-{dmg}"), RED);
        self.fx
            .burst(pos - Vec3::Y * 0.5, 8, &[RED, SALMON, WHITE], 2.0, 1.5);
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
            let can = match d.item {
                Some(item) => self.player.inv.can_fit(item, d.n),
                None => true,
            };
            if d.update(dt, world, ppos, can) {
                picked.push(i);
            }
        }
        for i in picked.into_iter().rev() {
            let d = self.drops.remove(i);
            match d.item {
                Some(item) => {
                    let left = self.player.inv.add(item, d.n);
                    let got = d.n - left;
                    if got > 0 {
                        self.toast(item.def().name, Some(item), got as u32);
                        io.audio
                            .play_at(Sfx::Pickup, 0.8, 1.0 + self.rng.f32() * 0.1);
                    }
                }
                None => {
                    self.gold += d.gold as u64;
                    self.toast(format!("+{}g", d.gold), None, 0);
                    io.audio.play_at(Sfx::Coin, 0.7, 1.0);
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

    // --------------------------------------------------------------------------------------
    // Drawing
    // --------------------------------------------------------------------------------------

    pub fn draw_scene(&mut self, r: &mut Renderer, a: &Assets) {
        r.cam = self.cam.clone();
        r.cam.update(r.width(), r.height());
        let env = self.env();
        let mut lights = Vec::new();
        let ppos = self.player.world_pos();
        match self.area {
            Area::Hollow { .. } => lights.push(PointLight {
                pos: ppos + Vec3::Y * 1.2,
                radius: 6.8,
                power: 0.8,
                warmth: 5.0,
            }),
            Area::Farm if env.night > 0.2 => lights.push(PointLight {
                pos: ppos + Vec3::Y * 1.0,
                radius: 3.8,
                power: 0.4 * env.night,
                warmth: 6.0,
            }),
            _ => {}
        }
        for f in &self.foes {
            if let Some(l) = f.light() {
                lights.push(l);
            }
        }
        for s in &self.shots {
            lights.push(PointLight {
                pos: s.world_pos(),
                radius: 2.2,
                power: 0.6,
                warmth: if s.color == ORANGE { 8.0 } else { 2.0 },
            });
        }
        // Fireflies on warm summer nights.
        let fireflies: Vec<Vec3> = if self.area == Area::Farm && env.night > 0.5 {
            (0..10)
                .map(|i| {
                    let t = self.time * 0.3 + i as f32 * 1.7;
                    let base = self.cam.target;
                    Vec3::new(
                        base.x + (t * 1.3).sin() * 7.0 + (i as f32 * 2.1).cos() * 3.0,
                        0.6 + (t * 2.0).sin() * 0.3,
                        base.z + (t * 0.9).cos() * 4.0,
                    )
                })
                .collect()
        } else {
            Vec::new()
        };
        let world = match &mut self.level {
            Some(l) if matches!(self.area, Area::Hollow { .. }) => &mut l.world,
            _ => &mut self.farm,
        };
        draw::draw_world(r, a, world, &env, &lights);

        // Target cursor.
        if let (Some((tx, tz)), Some(item)) = (self.target, self.player.held()) {
            let shows = matches!(
                item.def().kind,
                Kind::Tool(
                    ToolKind::Hoe | ToolKind::Can | ToolKind::Pickaxe | ToolKind::Axe,
                    _
                ) | Kind::Seed(_)
                    | Kind::Place(_)
            );
            if shows && self.fade.is_none() {
                let tex = if self.target_ok {
                    a.cursor
                } else {
                    a.cursor_bad
                };
                let y = if world.wall(tx, tz) != Wall::None {
                    1.02
                } else {
                    0.03
                };
                let pulse = 0.46 + (self.time * 6.0).sin() * 0.03;
                r.decal(
                    a.tex(tex),
                    UvRect::new(0.0, 0.0, 16.0, 16.0),
                    Vec3::new(tx as f32 + 0.5, y, tz as f32 + 0.5),
                    Vec2::splat(pulse),
                    &DrawOpts {
                        mode: Mode::Unlit,
                        zwrite: false,
                        ..Default::default()
                    },
                );
            }
        }

        // Dropped items bob and spin gently.
        for d in &self.drops {
            let bob = (self.time * 4.0 + d.age).sin() * 0.04;
            let base = d.pos + Vec3::Y * bob;
            r.shadow(a.tex(a.disk), Vec3::new(d.pos.x, 0.0, d.pos.z), 0.14);
            let id = match d.item {
                Some(i) => a.icon(i.def().icon),
                None => a.icon("coin"),
            };
            let size = if d.item.is_some() { 0.5 } else { 0.3 };
            r.billboard(
                a.tex(id),
                full_uv(a, id),
                base - Vec3::Y * 0.05,
                Vec2::splat(size),
                &DrawOpts::at(base).with_tag(3).with_glow(0.8),
            );
        }

        for f in &self.foes {
            f.draw(r, a);
        }
        for s in &self.shots {
            let c = shot_colors(s.color);
            r.point(s.world_pos(), 3, c[1]);
            r.point(s.world_pos(), 1, c[0]);
            let tail = s.world_pos() - Vec3::new(s.vel.x, 0.0, s.vel.y) * 0.05;
            r.point(tail, 2, c[2]);
        }

        // The cat.
        if self.area == Area::Farm {
            let c = &self.cat;
            let base = Vec3::new(c.pos.x, 0.0, c.pos.y);
            r.shadow(a.tex(a.disk), base, 0.22);
            let hop = if c.pet > 0.0 {
                (self.time * 10.0).sin().abs() * 0.05
            } else {
                0.0
            };
            let m = Mat4::from_translation(base + Vec3::Y * hop) * Mat4::from_rotation_y(c.yaw);
            let o = DrawOpts::at(base).with_tag(1);
            r.mesh(&a.bank, &a.critters.cat, &m, &o);
            let wag = (self.time * 5.0).sin() * 0.5;
            let tail = m
                * Mat4::from_translation(Vec3::new(0.0, 0.2, -0.2))
                * Mat4::from_rotation_x(0.8)
                * Mat4::from_rotation_y(wag);
            r.mesh(&a.bank, &a.critters.cat_tail, &tail, &o);
        }

        self.draw_player(r, a);
        self.fx.draw(r);
        for f in &fireflies {
            let on = ((self.time * 3.0 + f.x).sin() * 0.5 + 0.5) > 0.3;
            if on {
                r.point(*f, 1, CREAM);
            }
        }
        // Rain.
        if self.rain && self.area == Area::Farm {
            let t = self.cam.target;
            for i in 0..90 {
                let fx = (hash2(i, 0, 1) % 1000) as f32 / 1000.0;
                let fz = (hash2(i, 1, 1) % 1000) as f32 / 1000.0;
                let fall = (self.time * 1.8 + fx * 7.0).fract();
                let p = Vec3::new(
                    t.x - 11.0 + fx * 22.0,
                    3.5 - fall * 3.5,
                    t.z - 7.0 + fz * 13.0,
                );
                r.point(p, 1, SKY);
                r.point(p + Vec3::Y * 0.12, 1, BLUE);
            }
        }
        r.fb.outline(INK);
    }

    fn draw_player(&self, r: &mut Renderer, a: &Assets) {
        let p = &self.player;
        let mut pos = p.world_pos();
        let mut pose = Pose {
            walk: p.walk,
            stride: p.stride,
            ..Default::default()
        };
        if p.dodge > 0.0 {
            pos.y += (p.dodge / 0.26 * std::f32::consts::PI).sin() * 0.25;
            pose.squash = 0.5;
        }
        if let Some(act) = &p.act {
            pose.swing = Some((act.progress(), act.kind.swing()));
        } else if p.eat_t > 0.0 {
            pose.swing = Some((1.0 - p.eat_t / 0.6, Swing::Use));
        }
        let held = p.held().and_then(|i| i.tool()).map(|(k, t)| match k {
            ToolKind::Sword => &a.tools.sword[t as usize % 6],
            ToolKind::Pickaxe => &a.tools.pick[t as usize % 6],
            ToolKind::Axe => &a.tools.axe[t as usize % 6],
            ToolKind::Hoe => &a.tools.hoe,
            ToolKind::Can => &a.tools.can[t as usize % 3],
        });
        r.shadow(a.tex(a.disk), p.world_pos(), 0.3);
        let blink = p.hurt > 0.0 && (self.time * 20.0).sin() > 0.3;
        let mut o = DrawOpts {
            light: Light::At(p.world_pos()),
            tag: 1,
            ..Default::default()
        };
        if p.flash > 0.0 {
            o.mode = Mode::Solid(WHITE);
        }
        if !blink {
            draw_humanoid(r, a, &a.hero, pos, p.yaw, &pose, &o, held, Some(&a.sprout));
            // A soft silhouette where walls hide the hero.
            let ghost = DrawOpts {
                mode: Mode::Hidden(LAVENDER),
                zwrite: false,
                tag: 1,
                ..o
            };
            draw_humanoid(
                r,
                a,
                &a.hero,
                pos,
                p.yaw,
                &pose,
                &ghost,
                held,
                Some(&a.sprout),
            );
        }
        if let Some(act) = &p.act {
            if let ActKind::Sword(t) = act.kind {
                let col = [KHAKI, GOLD, SKY, CREAM, MINT, ORANGE][t as usize % 6];
                let yaw = act.dir.x.atan2(act.dir.y);
                draw::slash_arc(r, p.world_pos(), yaw, act.progress(), 1.25, col);
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
        let left = to.add(s.item, s.n);
        from.slots[slot] = if left > 0 {
            Some(Stack::new(s.item, left))
        } else {
            None
        };
    }
}
