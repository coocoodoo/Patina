//! The game: title screen, playing state, settings and saving.

pub mod bombs;
pub mod combat;
pub mod controls;
pub mod discover;
pub mod draw;
pub mod dungeon;
pub mod enchant;
pub mod farm;
pub mod fish;
pub mod foes;
pub mod folk;
pub mod fx;
pub mod gear;
pub mod home;
pub mod hud;
pub mod items;
pub mod loot;
pub mod magic;
pub mod menus;
pub mod play;
pub mod player;
pub mod quests;
pub mod save;
pub mod scene;
pub mod shops;
pub mod sky;
pub mod spellery;
pub mod spells;
pub mod talk;
#[cfg(test)]
mod tests;
pub mod tips;
pub mod town;
pub mod travel;
pub mod world;

use glam::{Vec2, Vec3};

use crate::assets::Assets;
use crate::audio::{Audio, Sfx, Song};
use crate::input::{Action, Button, Input};
use crate::palette::*;
use crate::render::{DrawOpts, Renderer};
use crate::ui::{Canvas, Style};
use menus::Menu;
use play::Play;

/// What the game gets from (and asks of) the window each frame.
pub struct Io<'a> {
    pub dt: f32,
    pub input: &'a Input,
    pub audio: &'a Audio,
    pub view: (usize, usize),
    pub quit: bool,
    pub toggle_fullscreen: bool,
}

#[derive(Clone, Debug)]
pub struct Settings {
    pub music: f32,
    pub sfx: f32,
    pub fullscreen: bool,
    pub shake: bool,
    /// Sun and moon shadows outdoors.
    pub shadows: bool,
    /// Ambient occlusion in corners and under things.
    pub ao: bool,
    pub dirty: bool,
}

/// Running on a Steam Deck (Steam tells the games it starts there).
pub fn steam_deck() -> bool {
    std::env::var("SteamDeck").is_ok_and(|v| v == "1")
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            music: 0.7,
            sfx: 0.8,
            // A Deck's small screen is best filled edge to edge.
            fullscreen: steam_deck(),
            shake: true,
            // A handheld's little processor keeps up better without them.
            shadows: !crate::input::handheld(),
            ao: !crate::input::handheld(),
            dirty: false,
        }
    }
}

pub struct Title {
    sel: usize,
    t: f32,
    has_save: bool,
    farm: world::World,
    confirm_new: bool,
    msg: Option<String>,
}

pub enum State {
    Title(Box<Title>),
    Play(Box<Play>),
}

pub struct Game {
    pub assets: Assets,
    pub state: State,
    pub settings: Settings,
    /// A little renderer for faces in the talk box.
    portrait: Renderer,
}

impl Game {
    pub fn new() -> Game {
        Game {
            assets: Assets::new(),
            state: State::Title(Box::new(Self::title())),
            settings: save::load_settings(),
            portrait: Renderer::new(talk::PORTRAIT as usize, talk::PORTRAIT as usize),
        }
    }

    fn title() -> Title {
        Title {
            sel: 0,
            t: 0.0,
            has_save: save::exists(),
            farm: farm::generate(7),
            confirm_new: false,
            msg: None,
        }
    }

    /// Starts straight into a fresh game (used by screenshots and tests).
    pub fn new_game(&mut self, seed: u64) {
        self.state = State::Play(Box::new(Play::new(seed)));
    }

    pub fn play_mut(&mut self) -> Option<&mut Play> {
        match &mut self.state {
            State::Play(p) => Some(p),
            _ => None,
        }
    }

    /// Called when the window closes.
    pub fn shutdown(&mut self) {
        if let State::Play(p) = &self.state {
            if !matches!(p.area, world::Area::Hollow { .. }) && p.fade.is_none() {
                let _ = save::write(p);
            }
        }
        save::save_settings(&self.settings);
    }

    pub fn update(&mut self, io: &mut Io) {
        let next = match &mut self.state {
            State::Title(t) => Self::update_title(t, io),
            State::Play(p) => {
                p.update(io, &mut self.settings);
                if !self.settings.shake {
                    p.shake = 0.0;
                }
                // Autosave at the start of every day.
                if matches!(p.menu, Menu::Summary) && p.saved_day != p.clock.day {
                    p.saved_day = p.clock.day;
                    match save::write(p) {
                        Ok(()) => p.toast("Game saved", None, 0),
                        Err(e) => p.toast(format!("Save failed: {e}"), None, 0),
                    }
                }
                if p.quit_to_title {
                    if !matches!(p.area, world::Area::Hollow { .. }) {
                        let _ = save::write(p);
                    }
                    Some(State::Title(Box::new(Self::title())))
                } else {
                    None
                }
            }
        };
        if let Some(s) = next {
            self.state = s;
        }
        if self.settings.dirty {
            self.settings.dirty = false;
            save::save_settings(&self.settings);
        }
    }

    fn update_title(t: &mut Title, io: &mut Io) -> Option<State> {
        t.t += io.dt;
        io.audio.music(Some(Song::Title), 0, 1.0);
        let input = io.input;
        let items = title_items(t);
        let n = items.len();
        if input.pressed_repeat(Action::Down) {
            t.sel = (t.sel + 1) % n;
            t.confirm_new = false;
            io.audio.play_at(Sfx::UiMove, 0.6, 1.0);
        }
        if input.pressed_repeat(Action::Up) {
            t.sel = (t.sel + n - 1) % n;
            t.confirm_new = false;
            io.audio.play_at(Sfx::UiMove, 0.6, 1.0);
        }
        let (w, h) = (io.view.0 as i32, io.view.1 as i32);
        let mut picked = None;
        for i in 0..n {
            let (x, y, bw) = title_item_rect(w, h, i);
            let m = input.mouse;
            if m.x >= x as f32
                && m.x < (x + bw) as f32
                && m.y >= y as f32 - 2.0
                && m.y < y as f32 + 11.0
            {
                if input.mouse_moved && t.sel != i {
                    t.sel = i;
                    t.confirm_new = false;
                }
                if input.button_pressed(Button::Left) {
                    picked = Some(i);
                }
            }
        }
        if input.pressed(Action::Confirm) {
            picked = Some(t.sel);
        }
        let i = picked?;
        io.audio.play(Sfx::UiSelect);
        match items[i] {
            "Continue" => match save::read() {
                Ok(p) => return Some(State::Play(Box::new(p))),
                Err(e) => t.msg = Some(format!("Could not load: {e}")),
            },
            "New game" => {
                if t.has_save && !t.confirm_new {
                    t.confirm_new = true;
                    t.msg = Some("This replaces your saved farm. Choose again to confirm.".into());
                } else {
                    let seed = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .map(|d| d.as_nanos() as u64)
                        .unwrap_or(1);
                    return Some(State::Play(Box::new(Play::new(seed))));
                }
            }
            "Fullscreen" => io.toggle_fullscreen = true,
            _ => io.quit = true,
        }
        None
    }

    pub fn draw(&mut self, r: &mut Renderer, input: &Input) {
        r.want_shadows = self.settings.shadows;
        r.want_ao = self.settings.ao;
        let a = &self.assets;
        match &mut self.state {
            State::Title(t) => {
                // A slow pan over a sleepy farm at dawn.
                let k = (t.t * 0.05).sin();
                r.cam.target = Vec3::new(30.0 + k * 8.0, 0.0, 17.0 + (t.t * 0.03).cos() * 3.0);
                r.cam.update(r.width(), r.height());
                let env = draw::Env {
                    ambient: 0.9,
                    warmth: 5.6,
                    clear: DEEP_TEAL,
                    time: t.t,
                    night: 0.0,
                    wind: 1.0,
                    push: glam::Vec2::new(29.5, 11.5),
                    spin: 0.0,
                };
                // The hero, idling by the house.
                let p = Vec3::new(29.5, 0.0, 11.5);
                let pose = draw::Pose {
                    bob: (t.t * 2.0).sin().abs() * 0.02,
                    ..Default::default()
                };
                let kit = player::Player::new(glam::Vec2::ZERO);
                let dressed = scene::dress(a, &kit.equip, kit.held_stack());
                let hero = |r: &mut Renderer| {
                    draw::draw_humanoid(
                        r,
                        a,
                        &a.hero,
                        p,
                        0.3,
                        &pose,
                        &DrawOpts::at(p).with_tag(1),
                        &dressed.outfit(Some(&a.sprout)),
                    );
                };
                // Early sun from the east throws long shadows across the lawn.
                let sun = sky::sky_light(450.0, sky::MoonPhase::New, false);
                r.key = sun.key();
                r.blobs = false;
                let vis = r.cam.visible_tiles(3.0);
                r.begin_shadows(sun.dir, vis);
                draw::cast_objects(r, a, &t.farm, &env, (vis.0, vis.1, vis.2 + 10, vis.3 + 3));
                hero(r);
                r.shadow_pass = false;
                draw::draw_world(r, a, &mut t.farm, &env, &[]);
                hero(r);
                if r.want_ao {
                    r.ambient_occlusion(draw::AO_STRENGTH);
                }
                draw::draw_grass(r, a, &t.farm, &env);
                if r.want_shadows {
                    r.sun_shadows(sun.shadow);
                } else {
                    r.shadow_pass = false;
                }
                r.fb.outline(INK);
                let mut c = Canvas {
                    fb: &mut r.fb,
                    font: &a.font,
                    darken: &r.sh.darken,
                };
                draw_title(&mut c, a, t, input.pad_active);
                draw_cursor(&mut c, a, input);
            }
            State::Play(p) => {
                p.draw_scene(r, a);
                let cam = r.cam.clone();
                let mut c = Canvas {
                    fb: &mut r.fb,
                    font: &a.font,
                    darken: &r.sh.darken,
                };
                p.draw_hud(&mut c, a, &cam);
                p.draw_menu(&mut c, a, &self.settings, input.mouse, input.mouse_aim);
                let (vw, vh) = (c.w(), c.h());
                if let Some((who, x, y)) = talk::portrait_rect(vw, vh, &p.menu) {
                    draw_portrait(&mut self.portrait, a, who, p.time);
                    let pf = &self.portrait.fb;
                    for yy in 0..pf.h as i32 {
                        for xx in 0..pf.w as i32 {
                            c.px(x + xx, y + yy, pf.color[(yy as usize) * pf.w + xx as usize]);
                        }
                    }
                }
                if let Some(f) = &p.fade {
                    let amt = if f.t < 1.0 { f.t } else { 2.0 - f.t };
                    c.fade(amt.clamp(0.0, 1.0) * 1.07, INK);
                }
                draw_cursor(&mut c, a, input);
            }
        }
    }
}

impl Default for Game {
    fn default() -> Self {
        Self::new()
    }
}

fn title_items(t: &Title) -> Vec<&'static str> {
    let mut v = Vec::new();
    if t.has_save {
        v.push("Continue");
    }
    v.push("New game");
    // A handheld's screen is always filled.
    if !crate::input::handheld() {
        v.push("Fullscreen");
    }
    v.push("Quit");
    v
}

fn title_item_rect(w: i32, h: i32, i: usize) -> (i32, i32, i32) {
    let bw = 110;
    // A tall screen (a handheld's 480x320) gives the logo more room.
    let y0 = h / 2 + 10 + ((h - 280) / 2).max(0);
    ((w - bw) / 2, y0 + i as i32 * 14, bw)
}

fn draw_title(c: &mut Canvas, a: &Assets, t: &Title, pad: bool) {
    let (w, h) = (c.w(), c.h());
    let items = title_items(t);
    let (x0, y0, _) = title_item_rect(w, h, 0);
    // The logo: the biggest that fits between the credit line and the menu, bobbing gently.
    let (top, bottom) = (14, y0 - 12);
    if let Some(logo) = a
        .logo
        .iter()
        .find(|l| l.h <= bottom - top)
        .or(a.logo.last())
    {
        let bob = ((t.t * 1.5).sin() * 2.0) as i32;
        let y = top + (bottom - top - logo.h).max(0) / 2 + bob;
        c.picture(logo, (w - logo.w) / 2, y);
    }
    c.panel(
        x0 - 8,
        y0 - 8,
        126,
        items.len() as i32 * 14 + 12,
        Style::Paper,
    );
    for (i, s) in items.iter().enumerate() {
        let (x, y, bw) = title_item_rect(w, h, i);
        if i == t.sel {
            c.rect(x - 2, y - 2, bw + 4, 12, GOLD);
            c.text(x, y, "▶", RUST);
        }
        c.text(x + 10, y, s, INK);
    }
    if let Some(m) = &t.msg {
        let mw = c.text_width(m);
        c.text_outline(
            (w - mw) / 2,
            y0 + items.len() as i32 * 14 + 10,
            m,
            SALMON,
            INK,
        );
    }
    let (view, menu) = crate::input::view_menu_names();
    let help = if pad {
        format!(
            "L stick walk  -  X use  -  A talk, interact  -  B roll  -  Y bag  -  {view} craft  -  R3 quests  -  {menu}: all controls"
        )
    } else {
        "WASD move  -  J / click use  -  E / right click talk, interact  -  Space roll  -  Tab bag  -  C craft  -  L quests".to_string()
    };
    let lines = c.font.wrap(&help, w - 20);
    for (i, l) in lines.iter().enumerate() {
        let lw = c.text_width(l);
        c.text_outline(
            (w - lw) / 2,
            h - 24 + i as i32 * 10 - (lines.len() as i32 - 1) * 10,
            l,
            WHITE,
            INK,
        );
    }
    let credit = "Palette: Resurrect 32 by Kerrie Lake";
    c.text_outline(w - c.text_width(credit) - 4, 4, credit, KHAKI, INK);
}

/// A villager's face and shoulders, for the talk box.
fn draw_portrait(r: &mut Renderer, a: &Assets, who: folk::Villager, time: f32) {
    use crate::render::{Light, Mode};
    let d = who.def();
    let s = d.look.scale;
    let (w, h) = (r.width(), r.height());
    r.cam.target = Vec3::new(0.0, 0.64 * s, 0.0);
    r.cam.pitch = 10f32.to_radians();
    r.cam.dist = 2.5 * s;
    r.cam.update(w, h);
    r.fb.clear(d.look.shirt[0]);
    r.remap.clear();
    // A soft band behind the head.
    for y in 0..h {
        for x in 0..w {
            if (x + y) % 7 == 0 {
                r.fb.color[y * w + x] = d.look.shirt[1];
            }
        }
    }
    let o = DrawOpts {
        light: Light::Fixed(1.05, crate::palette::NEUTRAL),
        mode: Mode::Lit,
        tag: 1,
        ..Default::default()
    };
    let (dressed, remap) = scene::villager_dress(a, who);
    let pose = draw::Pose {
        bob: (time * 2.0).sin() * 0.01,
        ..Default::default()
    };
    let mut fit = dressed.outfit_with(&remap);
    fit.held = None;
    draw::draw_humanoid(
        r,
        a,
        &a.folk[who as usize],
        Vec3::ZERO,
        0.35,
        &pose,
        &o,
        &fit,
    );
    r.fb.outline(INK);
}

fn draw_cursor(c: &mut Canvas, a: &Assets, input: &Input) {
    // Hidden while playing on a controller, and once the mouse has sat still a while.
    if !input.mouse_inside || !input.mouse_aim && (input.pad_active || input.idle > 4.0) {
        return;
    }
    let p: Vec2 = input.mouse;
    c.sprite(a.tex(a.icon("pointer")), p.x as i32, p.y as i32);
}
