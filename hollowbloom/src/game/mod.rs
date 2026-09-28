//! The game: title screen, playing state, settings and saving.

pub mod draw;
pub mod dungeon;
pub mod farm;
pub mod foes;
pub mod fx;
pub mod hud;
pub mod items;
pub mod menus;
pub mod play;
pub mod player;
pub mod save;
#[cfg(test)]
mod tests;
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
    pub dirty: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            music: 0.7,
            sfx: 0.8,
            fullscreen: false,
            shake: true,
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
}

impl Game {
    pub fn new() -> Game {
        Game {
            assets: Assets::new(),
            state: State::Title(Box::new(Self::title())),
            settings: save::load_settings(),
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
            if p.area == world::Area::Farm && p.fade.is_none() {
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
                    if p.area == world::Area::Farm {
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
                };
                draw::draw_world(r, a, &mut t.farm, &env, &[]);
                // The hero, idling by the house.
                let p = Vec3::new(29.5, 0.0, 11.5);
                r.shadow(a.tex(a.disk), p, 0.3);
                let pose = draw::Pose {
                    bob: (t.t * 2.0).sin().abs() * 0.02,
                    ..Default::default()
                };
                draw::draw_humanoid(
                    r,
                    a,
                    &a.hero,
                    p,
                    0.3,
                    &pose,
                    &DrawOpts::at(p).with_tag(1),
                    None,
                    Some(&a.sprout),
                );
                r.fb.outline(INK);
                let mut c = Canvas {
                    fb: &mut r.fb,
                    font: &a.font,
                    darken: &r.sh.darken,
                };
                draw_title(&mut c, t);
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
                p.draw_menu(&mut c, a, &self.settings, input.mouse);
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
    v.extend(["New game", "Fullscreen", "Quit"]);
    v
}

fn title_item_rect(w: i32, h: i32, i: usize) -> (i32, i32, i32) {
    let bw = 110;
    ((w - bw) / 2, h / 2 + 10 + i as i32 * 14, bw)
}

fn draw_title(c: &mut Canvas, t: &Title) {
    let (w, h) = (c.w(), c.h());
    let title = "Hollowbloom";
    let scale = if w >= 400 { 4 } else { 3 };
    let tw = c.big_width(title, scale);
    let bob = ((t.t * 1.5).sin() * 2.0) as i32;
    let ty = h / 2 - 72 + bob;
    c.text_big((w - tw) / 2 + 2, ty + 3, title, scale, INK, INK);
    c.text_big((w - tw) / 2, ty, title, scale, CREAM, RUST);
    let sub = "a cozy farm above an endless Hollow";
    let sw = c.text_width(sub);
    c.text_outline((w - sw) / 2, ty + 9 * scale + 6, sub, GOLD, INK);
    let items = title_items(t);
    let (x0, y0, _) = title_item_rect(w, h, 0);
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
    let help = "WASD move  -  J / click use  -  E / right click interact  -  Space roll  -  Tab bag  -  C craft";
    let lines = c.font.wrap(help, w - 20);
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

fn draw_cursor(c: &mut Canvas, a: &Assets, input: &Input) {
    if !input.mouse_inside || input.idle > 4.0 && !input.mouse_aim {
        return;
    }
    let p: Vec2 = input.mouse;
    c.sprite(a.tex(a.icon("pointer")), p.x as i32, p.y as i32);
}
