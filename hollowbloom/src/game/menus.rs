//! Menus: inventory and crafting, chests, the shop, the shipping bin, the Hollow elevator,
//! dialogs, pause/settings and the morning summary. Mouse and keyboard both work.

use glam::Vec2;

use super::items::{ITEMS, Inventory, Item, Kind, RECIPES, Stack, ToolKind};
use super::play::{Play, Trans, transfer};
use super::player::HOTBAR;
use super::world::Obj;
use super::{Io, Settings};
use crate::assets::Assets;
use crate::audio::Sfx;
use crate::input::{Action, Button};
use crate::palette::*;
use crate::ui::{Canvas, Style};
use crate::util::thousands;

pub const CELL: i32 = 19;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Choice {
    Close,
    Sleep,
    ReturnHome,
}

pub struct Summary {
    pub day: u32,
    pub earned: u64,
    pub grown: u32,
    pub ready: u32,
    pub rain: bool,
    pub passed_out: bool,
    pub fainted: bool,
}

pub enum Menu {
    None,
    Inventory {
        craft: bool,
        cursor: usize,
        recipe: usize,
        scroll: usize,
    },
    Chest {
        x: i32,
        z: i32,
        cursor: usize,
    },
    Shop {
        cursor: usize,
        sell: bool,
        scroll: usize,
    },
    Ship {
        cursor: usize,
    },
    Descend {
        floors: Vec<u32>,
        sel: usize,
    },
    Dialog {
        text: String,
        choices: Vec<(String, Choice)>,
        sel: usize,
        shown: f32,
    },
    Pause {
        sel: usize,
        settings: bool,
    },
    Summary,
}

impl Menu {
    pub fn inventory(craft: bool) -> Menu {
        Menu::Inventory {
            craft,
            cursor: 0,
            recipe: 0,
            scroll: 0,
        }
    }
    pub fn pause() -> Menu {
        Menu::Pause {
            sel: 0,
            settings: false,
        }
    }
    pub fn dialog(text: &str) -> Menu {
        Menu::Dialog {
            text: text.to_string(),
            choices: vec![],
            sel: 0,
            shown: 0.0,
        }
    }
    pub fn dialog_choice(text: &str, choices: Vec<(&str, Choice)>) -> Menu {
        Menu::Dialog {
            text: text.to_string(),
            choices: choices
                .into_iter()
                .map(|(s, c)| (s.to_string(), c))
                .collect(),
            sel: 0,
            shown: 0.0,
        }
    }
}

/// A grid of item slots on screen.
#[derive(Clone, Copy)]
pub struct Grid {
    pub x: i32,
    pub y: i32,
    pub cols: usize,
    pub rows: usize,
    /// Extra gap after the first row (the hotbar).
    pub gap: i32,
}

impl Grid {
    pub fn w(&self) -> i32 {
        self.cols as i32 * CELL - 1
    }
    pub fn h(&self) -> i32 {
        self.rows as i32 * CELL - 1 + self.gap
    }
    pub fn slot_pos(&self, i: usize) -> (i32, i32) {
        let (c, r) = ((i % self.cols) as i32, (i / self.cols) as i32);
        let gap = if r > 0 { self.gap } else { 0 };
        (self.x + c * CELL, self.y + r * CELL + gap)
    }
    pub fn hit(&self, p: Vec2) -> Option<usize> {
        for i in 0..self.cols * self.rows {
            let (x, y) = self.slot_pos(i);
            if p.x >= x as f32 && p.y >= y as f32 && p.x < (x + 18) as f32 && p.y < (y + 18) as f32
            {
                return Some(i);
            }
        }
        None
    }
}

fn inside(p: Vec2, x: i32, y: i32, w: i32, h: i32) -> bool {
    p.x >= x as f32 && p.y >= y as f32 && p.x < (x + w) as f32 && p.y < (y + h) as f32
}

/// Moves a grid cursor with the arrow keys.
fn nav(io: &Io, cursor: &mut usize, cols: usize, n: usize) -> bool {
    let i = io.input;
    let before = *cursor;
    if i.pressed_repeat(Action::Right) {
        *cursor = (*cursor + 1) % n;
    }
    if i.pressed_repeat(Action::Left) {
        *cursor = (*cursor + n - 1) % n;
    }
    if i.pressed_repeat(Action::Down) {
        *cursor = (*cursor + cols) % n;
    }
    if i.pressed_repeat(Action::Up) {
        *cursor = (*cursor + n - cols) % n;
    }
    if before != *cursor {
        io.audio.play_at(Sfx::UiMove, 0.5, 1.0);
        true
    } else {
        false
    }
}

pub struct Layout {
    pub px: i32,
    pub py: i32,
    pub pw: i32,
    pub ph: i32,
}

pub fn center(w: i32, h: i32, pw: i32, ph: i32) -> Layout {
    Layout {
        px: (w - pw) / 2,
        py: ((h - ph) / 2 - 8).max(2),
        pw,
        ph,
    }
}

pub fn bag_grid(l: &Layout, top: i32) -> Grid {
    let g = Grid {
        x: 0,
        y: 0,
        cols: 10,
        rows: 4,
        gap: 3,
    };
    Grid {
        x: l.px + (l.pw - g.w()) / 2,
        y: l.py + top,
        ..g
    }
}

pub fn shop_goods(p: &Play) -> Vec<(Item, u32)> {
    let d = p.deepest;
    let mut v = vec![(Item::TurnipSeeds, 20), (Item::CarrotSeeds, 40)];
    if d >= 3 {
        v.push((Item::GlowcapSpores, 70));
    }
    if d >= 8 {
        v.push((Item::MelonSeeds, 110));
    }
    if d >= 11 {
        v.push((Item::BerrySeeds, 90));
    }
    if d >= 21 {
        v.push((Item::PumpkinSeeds, 140));
    }
    if d >= 31 {
        v.push((Item::PepperSeeds, 130));
    }
    if d >= 41 {
        v.push((Item::LilyBulb, 170));
    }
    v.extend([
        (Item::Torch, 12),
        (Item::HealingTonic, 120),
        (Item::Feather, 300),
        (Item::Fence, 8),
        (Item::StonePath, 5),
        (Item::WoodPath, 5),
        (Item::FlowerPot, 60),
        (Item::Bench, 120),
        (Item::Chest, 160),
        (Item::Lamp, 200),
    ]);
    if d >= 5 {
        v.push((Item::Sprinkler, 450));
    }
    v
}

fn can_sell(item: Item) -> bool {
    !matches!(item.def().kind, Kind::Tool(_, 0))
}

impl Play {
    /// Click logic shared by every inventory grid.
    fn grid_click(held: &mut Option<Stack>, inv: &mut Inventory, i: usize, right: bool) {
        let slot = inv.slots[i];
        match (*held, slot, right) {
            (None, Some(s), false) => {
                *held = Some(s);
                inv.slots[i] = None;
            }
            (None, Some(s), true) => {
                let half = s.n.div_ceil(2);
                *held = Some(Stack::new(s.item, half));
                inv.slots[i] = if s.n - half > 0 {
                    Some(Stack::new(s.item, s.n - half))
                } else {
                    None
                };
            }
            (Some(h), None, false) => {
                inv.slots[i] = Some(h);
                *held = None;
            }
            (Some(h), None, true) => {
                inv.slots[i] = Some(Stack::new(h.item, 1));
                *held = if h.n > 1 {
                    Some(Stack::new(h.item, h.n - 1))
                } else {
                    None
                };
            }
            (Some(h), Some(s), _) if h.item == s.item => {
                let max = s.item.def().stack;
                let add = if right { 1 } else { h.n };
                let k = (max - s.n).min(add);
                inv.slots[i] = Some(Stack::new(s.item, s.n + k));
                *held = if h.n - k > 0 {
                    Some(Stack::new(h.item, h.n - k))
                } else {
                    None
                };
            }
            (Some(h), Some(s), false) => {
                inv.slots[i] = Some(h);
                *held = Some(s);
            }
            _ => {}
        }
    }

    /// Puts whatever is on the cursor back into the bag (or on the ground).
    fn return_held(&mut self) {
        if let Some(h) = self.held.take() {
            let left = self.player.inv.add(h.item, h.n);
            if left > 0 {
                self.drop_item(h.item, left);
            }
        }
    }

    fn drop_item(&mut self, item: Item, n: u16) {
        let at = self.player.world_pos()
            + glam::Vec3::new(self.player.facing.x * 0.6, 0.0, self.player.facing.y * 0.6);
        let mut d = super::fx::Drop::item(item, n, at, &mut self.rng);
        d.age = -0.8;
        self.drops.push(d);
    }

    fn close_menu(&mut self, io: &Io) {
        self.return_held();
        self.menu = Menu::None;
        io.audio.play(Sfx::UiBack);
    }

    pub fn update_menu(&mut self, io: &mut Io, settings: &mut Settings) {
        let (w, h) = (io.view.0 as i32, io.view.1 as i32);
        let input = io.input;
        let mouse = input.mouse;
        let lclick = input.button_pressed(Button::Left);
        let rclick = input.button_pressed(Button::Right);
        let menu = std::mem::replace(&mut self.menu, Menu::None);
        self.menu = match menu {
            Menu::None => Menu::None,
            Menu::Summary => {
                if input.pressed(Action::Confirm) || lclick || input.pressed(Action::Cancel) {
                    io.audio.play(Sfx::UiSelect);
                    self.banner = Some(super::play::Banner {
                        title: format!("Day {}", self.clock.day),
                        sub: format!(
                            "{} - {}",
                            self.clock.weekday(),
                            if self.rain { "rainy" } else { "sunny" }
                        ),
                        t: 0.0,
                    });
                    Menu::None
                } else {
                    Menu::Summary
                }
            }
            Menu::Dialog {
                text,
                choices,
                mut sel,
                mut shown,
            } => {
                let total = text.chars().count() as f32;
                let skip = input.pressed(Action::Confirm) || lclick;
                if shown < total {
                    shown += io.dt * 70.0;
                    if skip {
                        shown = total;
                    }
                    Menu::Dialog {
                        text,
                        choices,
                        sel,
                        shown,
                    }
                } else if choices.is_empty() {
                    if skip || input.pressed(Action::Cancel) || rclick {
                        io.audio.play(Sfx::UiBack);
                        Menu::None
                    } else {
                        Menu::Dialog {
                            text,
                            choices,
                            sel,
                            shown,
                        }
                    }
                } else {
                    let n = choices.len();
                    if input.pressed_repeat(Action::Down) || input.pressed_repeat(Action::Right) {
                        sel = (sel + 1) % n;
                        io.audio.play_at(Sfx::UiMove, 0.5, 1.0);
                    }
                    if input.pressed_repeat(Action::Up) || input.pressed_repeat(Action::Left) {
                        sel = (sel + n - 1) % n;
                        io.audio.play_at(Sfx::UiMove, 0.5, 1.0);
                    }
                    let l = dialog_layout(w, h, 2);
                    let mut picked = None;
                    for (i, _) in choices.iter().enumerate() {
                        let (cx, cy) = (l.px + l.pw - 70, l.py + 8 + i as i32 * 12);
                        if inside(mouse, cx, cy - 1, 64, 11) {
                            if input.mouse_moved {
                                sel = i;
                            }
                            if lclick {
                                picked = Some(i);
                            }
                        }
                    }
                    if input.pressed(Action::Confirm) {
                        picked = Some(sel);
                    }
                    if input.pressed(Action::Cancel) || rclick {
                        io.audio.play(Sfx::UiBack);
                        return;
                    }
                    if let Some(i) = picked {
                        io.audio.play(Sfx::UiSelect);
                        match choices[i].1 {
                            Choice::Close => {}
                            Choice::Sleep => {
                                io.audio.play(Sfx::Sleep);
                                self.start_fade(Trans::Sleep { passed_out: false });
                            }
                            Choice::ReturnHome => {
                                io.audio.play(Sfx::Waystone);
                                self.start_fade(Trans::Home);
                            }
                        }
                        Menu::None
                    } else {
                        Menu::Dialog {
                            text,
                            choices,
                            sel,
                            shown,
                        }
                    }
                }
            }
            Menu::Descend { floors, mut sel } => {
                let n = floors.len() + 1;
                if input.pressed_repeat(Action::Down) {
                    sel = (sel + 1) % n;
                    io.audio.play_at(Sfx::UiMove, 0.5, 1.0);
                }
                if input.pressed_repeat(Action::Up) {
                    sel = (sel + n - 1) % n;
                    io.audio.play_at(Sfx::UiMove, 0.5, 1.0);
                }
                let l = center(w, h, 150, 30 + n as i32 * 13);
                let mut picked = None;
                for i in 0..n {
                    let y = l.py + 22 + i as i32 * 13;
                    if inside(mouse, l.px + 8, y - 2, l.pw - 16, 12) {
                        if input.mouse_moved {
                            sel = i;
                        }
                        if lclick {
                            picked = Some(i);
                        }
                    }
                }
                if input.pressed(Action::Confirm) {
                    picked = Some(sel);
                }
                if input.pressed(Action::Cancel) || rclick || picked == Some(n - 1) {
                    io.audio.play(Sfx::UiBack);
                    Menu::None
                } else if let Some(i) = picked {
                    let floor = floors[i];
                    self.start_fade(Trans::Descend {
                        depth: floor,
                        via_waystone: floor > 1,
                    });
                    Menu::None
                } else {
                    Menu::Descend { floors, sel }
                }
            }
            Menu::Pause {
                mut sel,
                settings: in_settings,
            } => {
                let items = if in_settings { 5 } else { 4 };
                if input.pressed_repeat(Action::Down) {
                    sel = (sel + 1) % items;
                    io.audio.play_at(Sfx::UiMove, 0.5, 1.0);
                }
                if input.pressed_repeat(Action::Up) {
                    sel = (sel + items - 1) % items;
                    io.audio.play_at(Sfx::UiMove, 0.5, 1.0);
                }
                let l = center(w, h, 170, 40 + items as i32 * 14);
                let mut picked = None;
                for i in 0..items {
                    let y = l.py + 24 + i as i32 * 14;
                    if inside(mouse, l.px + 8, y - 2, l.pw - 16, 13) {
                        if input.mouse_moved {
                            sel = i;
                        }
                        if lclick {
                            picked = Some(i);
                        }
                    }
                }
                if input.pressed(Action::Confirm) {
                    picked = Some(sel);
                }
                let left = input.pressed_repeat(Action::Left);
                let right = input.pressed_repeat(Action::Right);
                if in_settings {
                    let step = |v: &mut f32, d: f32| {
                        *v = ((*v + d) * 10.0).round().clamp(0.0, 10.0) / 10.0
                    };
                    match sel {
                        0 if left || right || picked == Some(0) => {
                            step(&mut settings.music, if left { -0.1 } else { 0.1 });
                            if picked == Some(0) && settings.music > 1.0 {
                                settings.music = 0.0;
                            }
                        }
                        1 if left || right || picked == Some(1) => {
                            step(&mut settings.sfx, if left { -0.1 } else { 0.1 })
                        }
                        _ => {}
                    }
                    if picked == Some(2) {
                        io.toggle_fullscreen = true;
                    }
                    if picked == Some(3) || left && sel == 3 || right && sel == 3 {
                        settings.shake = !settings.shake;
                    }
                    io.audio.set_volume(settings.music, settings.sfx);
                    if picked.is_some() || left || right {
                        io.audio.play(Sfx::UiSelect);
                        settings.dirty = true;
                    }
                    if picked == Some(4) || input.pressed(Action::Cancel) || rclick {
                        Menu::Pause {
                            sel: 1,
                            settings: false,
                        }
                    } else {
                        Menu::Pause {
                            sel,
                            settings: true,
                        }
                    }
                } else if input.pressed(Action::Cancel)
                    || input.pressed(Action::Menu) && !input.pressed(Action::Cancel)
                    || rclick
                {
                    io.audio.play(Sfx::UiBack);
                    Menu::None
                } else {
                    match picked {
                        Some(0) => {
                            io.audio.play(Sfx::UiBack);
                            Menu::None
                        }
                        Some(1) => {
                            io.audio.play(Sfx::UiSelect);
                            Menu::Pause {
                                sel: 0,
                                settings: true,
                            }
                        }
                        Some(2) => {
                            self.quit_to_title = true;
                            Menu::None
                        }
                        Some(3) => {
                            io.quit = true;
                            Menu::None
                        }
                        _ => Menu::Pause {
                            sel,
                            settings: false,
                        },
                    }
                }
            }
            Menu::Inventory {
                mut craft,
                mut cursor,
                mut recipe,
                mut scroll,
            } => {
                let l = center(w, h, 214, 176);
                // Tabs.
                if lclick && inside(mouse, l.px + 8, l.py + 5, 40, 12) {
                    craft = false;
                    io.audio.play(Sfx::UiMove);
                } else if lclick && inside(mouse, l.px + 52, l.py + 5, 56, 12) {
                    craft = true;
                    io.audio.play(Sfx::UiMove);
                }
                if input.pressed(Action::Crafting) {
                    craft = !craft;
                    io.audio.play(Sfx::UiMove);
                }
                let close = input.pressed(Action::Cancel) || input.pressed(Action::Inventory);
                if close {
                    self.close_menu(io);
                    return;
                }
                if !craft {
                    let g = bag_grid(&l, 22);
                    nav(io, &mut cursor, 10, 40);
                    let hit = g.hit(mouse);
                    if let Some(i) = hit {
                        if input.mouse_moved {
                            cursor = i;
                        }
                    }
                    if let Some(i) = hit.filter(|_| lclick || rclick) {
                        Self::grid_click(&mut self.held, &mut self.player.inv, i, rclick);
                        io.audio.play_at(Sfx::UiMove, 0.8, 1.2);
                    } else if input.pressed(Action::Confirm) {
                        Self::grid_click(&mut self.held, &mut self.player.inv, cursor, false);
                        io.audio.play_at(Sfx::UiMove, 0.8, 1.2);
                    } else if lclick && !inside(mouse, l.px, l.py, l.pw, l.ph) {
                        if let Some(hs) = self.held.take() {
                            self.drop_item(hs.item, hs.n);
                            io.audio.play(Sfx::Place);
                        }
                    }
                    // Eat straight from the bag with a right click on food.
                } else {
                    let n = RECIPES.len();
                    let rows = 8;
                    if input.pressed_repeat(Action::Down) {
                        recipe = (recipe + 1) % n;
                        io.audio.play_at(Sfx::UiMove, 0.5, 1.0);
                    }
                    if input.pressed_repeat(Action::Up) {
                        recipe = (recipe + n - 1) % n;
                        io.audio.play_at(Sfx::UiMove, 0.5, 1.0);
                    }
                    if input.wheel != 0.0 {
                        scroll = (scroll as i32 - input.wheel.signum() as i32)
                            .clamp(0, (n - rows) as i32) as usize;
                    }
                    if recipe < scroll {
                        scroll = recipe;
                    }
                    if recipe >= scroll + rows {
                        scroll = recipe + 1 - rows;
                    }
                    for row in 0..rows {
                        let y = l.py + 22 + row as i32 * 18;
                        if inside(mouse, l.px + 6, y, 100, 17) && lclick && scroll + row < n {
                            recipe = scroll + row;
                            io.audio.play_at(Sfx::UiMove, 0.6, 1.0);
                        }
                    }
                    let button = (l.px + 118, l.py + l.ph - 24, 86, 14);
                    let craft_now = input.pressed(Action::Confirm)
                        || lclick && inside(mouse, button.0, button.1, button.2, button.3);
                    if craft_now {
                        let r = &RECIPES[recipe];
                        if r.can_craft(&self.player.inv) {
                            for (item, k) in r.needs {
                                self.player.inv.take(*item, *k as u32);
                            }
                            self.player.inv.add(r.out, r.n);
                            io.audio.play(Sfx::Craft);
                            self.toast(
                                format!("Crafted {}", r.out.def().name),
                                Some(r.out),
                                r.n as u32,
                            );
                        } else {
                            io.audio.play(Sfx::Denied);
                        }
                    }
                }
                Menu::Inventory {
                    craft,
                    cursor,
                    recipe,
                    scroll,
                }
            }
            Menu::Chest { x, z, mut cursor } => {
                let l = center(w, h, 214, 176);
                let cg = Grid {
                    x: l.px + (l.pw - (10 * CELL - 1)) / 2,
                    y: l.py + 20,
                    cols: 10,
                    rows: 3,
                    gap: 0,
                };
                let bg = bag_grid(&l, 20 + 3 * CELL + 16);
                if input.pressed(Action::Cancel) || input.pressed(Action::Inventory) {
                    self.close_menu(io);
                    return;
                }
                nav(io, &mut cursor, 10, 70);
                let mut chest = match self.world_mut().obj_mut(x, z) {
                    Some(Obj::Chest { items }) => Inventory {
                        slots: std::mem::take(items),
                    },
                    _ => {
                        self.return_held();
                        return;
                    }
                };
                let shift = input.key_down(crate::input::KeyCode::ShiftLeft)
                    || input.key_down(crate::input::KeyCode::ShiftRight);
                if let Some(i) = cg.hit(mouse) {
                    if input.mouse_moved {
                        cursor = i;
                    }
                    if lclick && shift {
                        transfer(&mut chest, i, &mut self.player.inv);
                    } else if lclick || rclick {
                        Self::grid_click(&mut self.held, &mut chest, i, rclick);
                    }
                } else if let Some(i) = bg.hit(mouse) {
                    if input.mouse_moved {
                        cursor = 30 + i;
                    }
                    if lclick && shift {
                        transfer(&mut self.player.inv, i, &mut chest);
                    } else if lclick || rclick {
                        Self::grid_click(&mut self.held, &mut self.player.inv, i, rclick);
                    }
                } else if input.pressed(Action::Confirm) {
                    if cursor < 30 {
                        transfer(&mut chest, cursor, &mut self.player.inv);
                    } else {
                        transfer(&mut self.player.inv, cursor - 30, &mut chest);
                    }
                }
                if lclick || rclick || input.pressed(Action::Confirm) {
                    io.audio.play_at(Sfx::UiMove, 0.8, 1.2);
                }
                if let Some(Obj::Chest { items }) = self.world_mut().obj_mut(x, z) {
                    *items = chest.slots;
                }
                Menu::Chest { x, z, cursor }
            }
            Menu::Shop {
                mut cursor,
                mut sell,
                mut scroll,
            } => {
                let l = center(w, h, 214, 176);
                if input.pressed(Action::Cancel) {
                    self.close_menu(io);
                    return;
                }
                if lclick && inside(mouse, l.px + 8, l.py + 5, 36, 12) {
                    sell = false;
                } else if lclick && inside(mouse, l.px + 48, l.py + 5, 36, 12) {
                    sell = true;
                }
                if input.pressed(Action::Crafting) || input.pressed(Action::Inventory) {
                    sell = !sell;
                }
                let goods = shop_goods(self);
                if !sell {
                    let rows = 7;
                    let n = goods.len();
                    if input.pressed_repeat(Action::Down) {
                        cursor = (cursor + 1) % n;
                    }
                    if input.pressed_repeat(Action::Up) {
                        cursor = (cursor + n - 1) % n;
                    }
                    if input.wheel != 0.0 {
                        scroll = (scroll as i32 - input.wheel.signum() as i32)
                            .clamp(0, n.saturating_sub(rows) as i32)
                            as usize;
                    }
                    cursor = cursor.min(n - 1);
                    if cursor < scroll {
                        scroll = cursor;
                    }
                    if cursor >= scroll + rows {
                        scroll = cursor + 1 - rows;
                    }
                    let mut buy = input.pressed(Action::Confirm);
                    for row in 0..rows {
                        let y = l.py + 22 + row as i32 * 18;
                        if scroll + row < n && inside(mouse, l.px + 6, y, l.pw - 12, 17) {
                            if input.mouse_moved {
                                cursor = scroll + row;
                            }
                            if lclick {
                                cursor = scroll + row;
                                buy = true;
                            }
                        }
                    }
                    if buy {
                        let (item, price) = goods[cursor];
                        if self.gold >= price as u64 && self.player.inv.can_fit(item, 1) {
                            self.gold -= price as u64;
                            self.player.inv.add(item, 1);
                            io.audio.play(Sfx::Coin);
                        } else {
                            io.audio.play(Sfx::Denied);
                        }
                    }
                } else {
                    let g = bag_grid(&l, 40);
                    nav(io, &mut cursor, 10, 40);
                    cursor = cursor.min(39);
                    let mut target = None;
                    if let Some(i) = g.hit(mouse) {
                        if input.mouse_moved {
                            cursor = i;
                        }
                        if lclick || rclick {
                            target = Some((i, rclick));
                        }
                    }
                    if input.pressed(Action::Confirm) {
                        target = Some((cursor, false));
                    }
                    if let Some((i, one)) = target {
                        if let Some(s) = self.player.inv.slots[i] {
                            if can_sell(s.item) {
                                let n = if one { 1 } else { s.n };
                                self.gold += s.item.def().price as u64 * n as u64;
                                self.stats.earned += s.item.def().price as u64 * n as u64;
                                self.player.inv.slots[i] = if s.n > n {
                                    Some(Stack::new(s.item, s.n - n))
                                } else {
                                    None
                                };
                                io.audio.play(Sfx::Coin);
                            } else {
                                io.audio.play(Sfx::Denied);
                            }
                        }
                    }
                }
                Menu::Shop {
                    cursor,
                    sell,
                    scroll,
                }
            }
            Menu::Ship { mut cursor } => {
                let l = center(w, h, 214, 176);
                if input.pressed(Action::Cancel) || input.pressed(Action::Inventory) {
                    self.close_menu(io);
                    return;
                }
                let g = bag_grid(&l, 60);
                nav(io, &mut cursor, 10, 40);
                let mut target = None;
                if let Some(i) = g.hit(mouse) {
                    if input.mouse_moved {
                        cursor = i;
                    }
                    if lclick || rclick {
                        target = Some((i, rclick));
                    }
                }
                if input.pressed(Action::Confirm) {
                    target = Some((cursor, false));
                }
                if let Some((i, one)) = target {
                    if let Some(s) = self.player.inv.slots[i] {
                        if can_sell(s.item) && s.item.def().price > 0 {
                            let n = if one { 1 } else { s.n };
                            self.player.inv.slots[i] = if s.n > n {
                                Some(Stack::new(s.item, s.n - n))
                            } else {
                                None
                            };
                            if let Some(e) = self
                                .shipping
                                .iter_mut()
                                .find(|e| e.item == s.item && e.n < 999)
                            {
                                e.n += n;
                            } else {
                                self.shipping.push(Stack::new(s.item, n));
                            }
                            io.audio.play(Sfx::Place);
                        } else {
                            io.audio.play(Sfx::Denied);
                        }
                    }
                }
                // Take the last shipment back.
                if lclick && inside(mouse, l.px + 8, l.py + 20, l.pw - 16, 34) {
                    if let Some(s) = self.shipping.pop() {
                        let left = self.player.inv.add(s.item, s.n);
                        if left > 0 {
                            self.shipping.push(Stack::new(s.item, left));
                        }
                        io.audio.play(Sfx::UiBack);
                    }
                }
                Menu::Ship { cursor }
            }
        };
        let _ = HOTBAR;
    }

    // --------------------------------------------------------------------------------------
    // Drawing
    // --------------------------------------------------------------------------------------

    pub fn draw_slot(c: &mut Canvas, a: &Assets, x: i32, y: i32, s: Option<Stack>, selected: bool) {
        c.panel(x, y, 18, 18, Style::Inset);
        if let Some(s) = s {
            c.sprite(a.tex(a.icon(s.item.def().icon)), x + 1, y + 1);
            if s.n > 1 {
                c.tiny(x + 17, y + 12, &s.n.to_string(), WHITE, INK);
            }
        }
        if selected {
            c.frame(x - 1, y - 1, 20, 20, ORANGE);
            c.frame(x, y, 18, 18, GOLD);
        }
    }

    pub fn draw_grid(c: &mut Canvas, a: &Assets, g: &Grid, inv: &Inventory, cursor: Option<usize>) {
        for i in 0..(g.cols * g.rows).min(inv.slots.len()) {
            let (x, y) = g.slot_pos(i);
            Self::draw_slot(c, a, x, y, inv.slots[i], cursor == Some(i));
        }
    }

    pub fn tooltip(&self, c: &mut Canvas, x: i32, y: i32, item: Item) {
        let d = item.def();
        let mut lines = vec![];
        let kind = match d.kind {
            Kind::Seed(crop) => {
                let cd = crop.def();
                if cd.regrow > 0 {
                    format!("Seed - {} days, regrows", cd.days)
                } else {
                    format!("Seed - {} days", cd.days)
                }
            }
            Kind::Produce { hp, energy } | Kind::Food { hp, energy } => {
                format!("Food - +{hp} HP  +{energy} energy")
            }
            Kind::Tool(ToolKind::Sword, t) => {
                format!("Weapon - {} damage", self.player.sword_damage(t))
            }
            Kind::Tool(_, t) => format!("Tool - tier {}", t + 1),
            Kind::Place(_) => "Placeable".to_string(),
            Kind::Material => "Material".to_string(),
            _ => "Special".to_string(),
        };
        lines.push(kind);
        let desc = c.font.wrap(d.desc, 130);
        let w = desc
            .iter()
            .chain(lines.iter())
            .map(|l| c.text_width(l))
            .max()
            .unwrap_or(0)
            .max(c.text_width(d.name))
            .max(60)
            + 10;
        let h = 16 + (lines.len() + desc.len()) as i32 * 10 + 12;
        let x = x.min(c.w() - w - 2).max(2);
        let y = y.min(c.h() - h - 2).max(2);
        c.panel(x, y, w, h, Style::Paper);
        c.text(x + 5, y + 4, d.name, INK);
        let mut yy = y + 15;
        for l in &lines {
            c.text(x + 5, yy, l, TEAL);
            yy += 10;
        }
        for l in &desc {
            c.text(x + 5, yy, l, SHADOW);
            yy += 10;
        }
        if can_sell(item) {
            c.text(x + 5, yy + 1, &format!("Sells for {}g", d.price), RUST);
        }
    }

    pub fn draw_menu(&self, c: &mut Canvas, a: &Assets, settings: &Settings, mouse: Vec2) {
        let (w, h) = (c.w(), c.h());
        match &self.menu {
            Menu::None => {}
            Menu::Summary => {
                let Some(s) = &self.last_summary else { return };
                c.shade(0, 0, w, h, 2);
                let l = center(w, h, 190, 120);
                c.panel(l.px, l.py, l.pw, l.ph, Style::Paper);
                let title = format!("Day {}", s.day);
                c.text_big(
                    l.px + (l.pw - c.big_width(&title, 2)) / 2,
                    l.py + 8,
                    &title,
                    2,
                    CREAM,
                    RUST,
                );
                let mut y = l.py + 34;
                let mut line = |c: &mut Canvas, t: String, col: u8| {
                    c.text_center(l.px + l.pw / 2, y, &t, col);
                    y += 11;
                };
                if s.fainted {
                    line(c, "You were found asleep in the Hollow...".into(), CRIMSON);
                } else if s.passed_out {
                    line(c, "You fell asleep on your feet.".into(), CRIMSON);
                } else {
                    line(c, "You slept soundly.".into(), SHADOW);
                }
                line(
                    c,
                    format!("Shipped goods sold for {}g", thousands(s.earned)),
                    RUST,
                );
                line(c, format!("{} crops grew overnight", s.grown), TEAL);
                if s.ready > 0 {
                    line(c, format!("{} are ready to harvest!", s.ready), GREEN);
                }
                line(
                    c,
                    if s.rain {
                        "It's raining - the crops are watered."
                    } else {
                        "The sun is out."
                    }
                    .into(),
                    INDIGO,
                );
                c.text_center(
                    l.px + l.pw / 2,
                    l.py + l.ph - 14,
                    "Press E to begin the day",
                    SHADOW,
                );
            }
            Menu::Dialog {
                text,
                choices,
                sel,
                shown,
            } => {
                let pw = dialog_layout(w, h, 0).pw;
                let tw = if choices.is_empty() { pw - 16 } else { pw - 90 };
                let lines: usize = text.split('\n').map(|p| c.font.wrap(p, tw).len()).sum();
                let l = dialog_layout(w, h, lines as i32);
                c.panel(l.px, l.py, l.pw, l.ph, Style::Paper);
                let n = *shown as usize;
                let visible: String = text.chars().take(n).collect();
                let mut y = l.py + 7;
                for line in visible.split('\n') {
                    y += c.paragraph(l.px + 8, y, tw, line, INK);
                }
                let done = n >= text.chars().count();
                if done && !choices.is_empty() {
                    for (i, (label, _)) in choices.iter().enumerate() {
                        let (cx, cy) = (l.px + l.pw - 70, l.py + 8 + i as i32 * 12);
                        if i == *sel {
                            c.rect(cx - 2, cy - 2, 66, 11, GOLD);
                            c.text(cx - 1, cy, "▶", RUST);
                        }
                        c.text(cx + 6, cy, label, INK);
                    }
                } else if done && (self.time * 3.0).fract() < 0.6 {
                    c.text(l.px + l.pw - 12, l.py + l.ph - 11, "▼", RUST);
                }
            }
            Menu::Descend { floors, sel } => {
                let n = floors.len() + 1;
                let l = center(w, h, 150, 30 + n as i32 * 13);
                c.panel(l.px, l.py, l.pw, l.ph, Style::Paper);
                c.text_center(l.px + l.pw / 2, l.py + 6, "Descend into the Hollow", RUST);
                for i in 0..n {
                    let y = l.py + 22 + i as i32 * 13;
                    let label = match floors.get(i) {
                        None => "Not today".to_string(),
                        Some(1) => "Floor 1 - the beginning".to_string(),
                        Some(f) => format!("Waystone - floor {f}"),
                    };
                    if i == *sel {
                        c.rect(l.px + 8, y - 2, l.pw - 16, 12, GOLD);
                        c.text(l.px + 10, y, "▶", RUST);
                    }
                    c.text(l.px + 18, y, &label, INK);
                }
            }
            Menu::Pause {
                sel,
                settings: in_settings,
            } => {
                c.shade(0, 0, w, h, 1);
                let items: Vec<String> = if *in_settings {
                    vec![
                        format!(
                            "Music volume  ◀ {:>3}% ▶",
                            (settings.music * 100.0).round() as i32
                        ),
                        format!(
                            "Sound volume  ◀ {:>3}% ▶",
                            (settings.sfx * 100.0).round() as i32
                        ),
                        "Toggle fullscreen (F11)".to_string(),
                        format!(
                            "Screen shake: {}",
                            if settings.shake { "on" } else { "off" }
                        ),
                        "Back".to_string(),
                    ]
                } else {
                    vec![
                        "Resume".into(),
                        "Settings".into(),
                        "Save & quit to title".into(),
                        "Quit game".into(),
                    ]
                };
                let l = center(w, h, 170, 40 + items.len() as i32 * 14);
                c.panel(l.px, l.py, l.pw, l.ph, Style::Paper);
                c.text_center(
                    l.px + l.pw / 2,
                    l.py + 8,
                    if *in_settings { "Settings" } else { "Paused" },
                    RUST,
                );
                for (i, s) in items.iter().enumerate() {
                    let y = l.py + 24 + i as i32 * 14;
                    if i == *sel {
                        c.rect(l.px + 8, y - 2, l.pw - 16, 13, GOLD);
                    }
                    c.text(l.px + 14, y + 1, s, INK);
                }
            }
            Menu::Inventory {
                craft,
                cursor,
                recipe,
                scroll,
            } => {
                let l = center(w, h, 214, 176);
                c.panel(l.px, l.py, l.pw, l.ph, Style::Paper);
                tabs(
                    c,
                    &l,
                    &["Bag", "Crafting"],
                    if *craft { 1 } else { 0 },
                    &[40, 56],
                );
                if !*craft {
                    let g = bag_grid(&l, 22);
                    Self::draw_grid(c, a, &g, &self.player.inv, Some(*cursor));
                    // Hotbar marker.
                    let (hx, hy) = g.slot_pos(self.player.sel);
                    c.rect(hx + 7, hy - 3, 4, 2, RUST);
                    let p = &self.player;
                    let y = g.y + g.h() + 6;
                    let x = l.px + 12;
                    c.text(x, y, &format!("Level {}", p.level), INK);
                    let need = super::player::xp_needed(p.level);
                    c.bar(
                        x + 48,
                        y + 1,
                        70,
                        7,
                        p.xp as f32 / need as f32,
                        AQUA,
                        MINT,
                        SHADOW,
                    );
                    c.text(x + 124, y, &format!("{}/{} xp", p.xp, need), SHADOW);
                    c.text(x, y + 11, &format!("♥ {}/{}", p.hp, p.max_hp), CRIMSON);
                    c.text(
                        x + 70,
                        y + 11,
                        &format!("⚡ {}/{}", p.energy.max(0.0) as i32, p.max_energy),
                        RUST,
                    );
                    c.text(x + 140, y + 11, &format!("{}g", thousands(self.gold)), RUST);
                    c.text(
                        x,
                        y + 22,
                        &format!("Deepest floor {}  -  Day {}", self.deepest, self.clock.day),
                        SHADOW,
                    );
                    c.text(x, y + 33, "Click to move items. Right click splits.", KHAKI);
                    let hover = g.hit(mouse).or(Some(*cursor));
                    if self.held.is_none() {
                        if let Some(i) = hover {
                            if let Some(s) = self.player.inv.slots[i] {
                                // Beside the panel when there is room, otherwise under the slot.
                                let (sx, sy) = g.slot_pos(i);
                                if l.px + l.pw + 150 < w {
                                    self.tooltip(c, l.px + l.pw + 4, sy - 4, s.item);
                                } else {
                                    self.tooltip(c, sx - 20, sy + 22, s.item);
                                }
                            }
                        }
                    }
                } else {
                    let rows = 8;
                    for row in 0..rows {
                        let i = scroll + row;
                        if i >= RECIPES.len() {
                            break;
                        }
                        let r = &RECIPES[i];
                        let y = l.py + 22 + row as i32 * 18;
                        let ok = r.can_craft(&self.player.inv);
                        if i == *recipe {
                            c.rect(l.px + 6, y, 104, 17, GOLD);
                        }
                        let icon = a.tex(a.icon(r.out.def().icon));
                        if ok {
                            c.sprite(icon, l.px + 8, y + 1);
                        } else {
                            let dim = c.darken[0];
                            c.sprite_map(icon, l.px + 8, y + 1, |col| dim[col as usize]);
                        }
                        let name = r.out.def().name;
                        c.text(l.px + 27, y + 5, name, if ok { INK } else { ROSEWOOD });
                    }
                    // Scroll bar.
                    let n = RECIPES.len() as i32;
                    let track = rows as i32 * 18;
                    c.rect(l.px + 111, l.py + 22, 2, track, KHAKI);
                    let th = (track * rows as i32 / n).max(6);
                    let ty = l.py + 22 + (track - th) * *scroll as i32 / (n - rows as i32).max(1);
                    c.rect(l.px + 111, ty, 2, th, RUST);
                    // Details.
                    let r = &RECIPES[*recipe];
                    let dx = l.px + 118;
                    c.panel(dx, l.py + 22, 88, 20, Style::Inset);
                    c.sprite(a.tex(a.icon(r.out.def().icon)), dx + 2, l.py + 24);
                    let title = if r.n > 1 {
                        format!("{} x{}", r.out.def().name, r.n)
                    } else {
                        r.out.def().name.to_string()
                    };
                    let lines = c.font.wrap(&title, 66);
                    for (k, t) in lines.iter().take(2).enumerate() {
                        c.text(dx + 20, l.py + 24 + k as i32 * 9, t, INK);
                    }
                    let mut y = l.py + 48;
                    c.text(dx, y, "Needs:", SHADOW);
                    y += 11;
                    for (item, k) in r.needs {
                        let have = self.player.inv.count(*item);
                        c.sprite(a.tex(a.icon(item.def().icon)), dx, y - 3);
                        let col = if have >= *k as u32 { TEAL } else { CRIMSON };
                        c.text(dx + 18, y + 2, &format!("{have}/{k}"), col);
                        y += 17;
                    }
                    y = y.max(l.py + 48);
                    let desc = c.font.wrap(r.out.def().desc, 86);
                    for (k, t) in desc.iter().take(3).enumerate() {
                        c.text(dx, y + k as i32 * 9, t, SHADOW);
                    }
                    let ok = r.can_craft(&self.player.inv);
                    let (bx, by) = (dx, l.py + l.ph - 24);
                    c.rect(bx, by, 86, 14, if ok { GREEN } else { KHAKI });
                    c.frame(bx, by, 86, 14, INK);
                    c.text_center(
                        bx + 43,
                        by + 3,
                        if ok { "Craft (E)" } else { "Missing items" },
                        if ok { WHITE } else { ROSEWOOD },
                    );
                }
            }
            Menu::Chest { x, z, cursor } => {
                let l = center(w, h, 214, 176);
                c.panel(l.px, l.py, l.pw, l.ph, Style::Paper);
                c.text(l.px + 10, l.py + 7, "Chest", RUST);
                c.text(l.px + 80, l.py + 7, "Shift-click moves a stack", KHAKI);
                let cg = Grid {
                    x: l.px + (l.pw - (10 * CELL - 1)) / 2,
                    y: l.py + 20,
                    cols: 10,
                    rows: 3,
                    gap: 0,
                };
                if let Some(Obj::Chest { items }) = self.world().obj(*x, *z) {
                    let inv = Inventory {
                        slots: items.clone(),
                    };
                    Self::draw_grid(
                        c,
                        a,
                        &cg,
                        &inv,
                        if *cursor < 30 { Some(*cursor) } else { None },
                    );
                    let bg = bag_grid(&l, 20 + 3 * CELL + 16);
                    c.text(l.px + 10, bg.y - 11, "Bag", RUST);
                    Self::draw_grid(
                        c,
                        a,
                        &bg,
                        &self.player.inv,
                        if *cursor >= 30 {
                            Some(*cursor - 30)
                        } else {
                            None
                        },
                    );
                }
            }
            Menu::Shop {
                cursor,
                sell,
                scroll,
            } => {
                let l = center(w, h, 214, 176);
                c.panel(l.px, l.py, l.pw, l.ph, Style::Paper);
                tabs(
                    c,
                    &l,
                    &["Buy", "Sell"],
                    if *sell { 1 } else { 0 },
                    &[36, 36],
                );
                c.text(
                    l.px + l.pw - 60,
                    l.py + 7,
                    &format!("{}g", thousands(self.gold)),
                    RUST,
                );
                if !*sell {
                    let goods = shop_goods(self);
                    for row in 0..7 {
                        let i = scroll + row;
                        if i >= goods.len() {
                            break;
                        }
                        let (item, price) = goods[i];
                        let y = l.py + 22 + row as i32 * 18;
                        if i == *cursor {
                            c.rect(l.px + 6, y, l.pw - 12, 17, GOLD);
                        }
                        c.sprite(a.tex(a.icon(item.def().icon)), l.px + 8, y + 1);
                        c.text(l.px + 28, y + 5, item.def().name, INK);
                        let pt = format!("{price}g");
                        let afford = self.gold >= price as u64;
                        c.text(
                            l.px + l.pw - 12 - c.text_width(&pt),
                            y + 5,
                            &pt,
                            if afford { RUST } else { ROSEWOOD },
                        );
                    }
                    c.text(
                        l.px + 10,
                        l.py + l.ph - 14,
                        "Burrowby: \"New seeds as you go deeper!\"",
                        SHADOW,
                    );
                } else {
                    c.text(
                        l.px + 10,
                        l.py + 24,
                        "Click an item to sell the stack.",
                        SHADOW,
                    );
                    let g = bag_grid(&l, 40);
                    Self::draw_grid(c, a, &g, &self.player.inv, Some(*cursor));
                    if let Some(i) = g.hit(mouse).or(Some(*cursor)) {
                        if let Some(s) = self.player.inv.slots[i] {
                            let v = s.item.def().price as u64 * s.n as u64;
                            let t = if can_sell(s.item) {
                                format!("{} x{} - {}g", s.item.def().name, s.n, thousands(v))
                            } else {
                                format!("{} - keep this one!", s.item.def().name)
                            };
                            c.text(l.px + 10, g.y + g.h() + 8, &t, INK);
                        }
                    }
                }
            }
            Menu::Ship { cursor } => {
                let l = center(w, h, 214, 176);
                c.panel(l.px, l.py, l.pw, l.ph, Style::Paper);
                c.text(l.px + 10, l.py + 7, "Shipping bin - sold overnight", RUST);
                c.panel(l.px + 8, l.py + 20, l.pw - 16, 34, Style::Inset);
                let total: u64 = self
                    .shipping
                    .iter()
                    .map(|s| s.item.def().price as u64 * s.n as u64)
                    .sum();
                for (i, s) in self.shipping.iter().rev().take(9).enumerate() {
                    let x = l.px + 10 + i as i32 * 19;
                    c.sprite(a.tex(a.icon(s.item.def().icon)), x, l.py + 22);
                    if s.n > 1 {
                        c.tiny(x + 16, l.py + 33, &s.n.to_string(), WHITE, INK);
                    }
                }
                c.text(
                    l.px + 12,
                    l.py + 42,
                    &format!("Today: {}g   (click here to take back)", thousands(total)),
                    SHADOW,
                );
                let g = bag_grid(&l, 60);
                Self::draw_grid(c, a, &g, &self.player.inv, Some(*cursor));
                if let Some(i) = g.hit(mouse).or(Some(*cursor)) {
                    if let Some(s) = self.player.inv.slots[i] {
                        let t = format!(
                            "{} x{} - {}g",
                            s.item.def().name,
                            s.n,
                            thousands(s.item.def().price as u64 * s.n as u64)
                        );
                        c.text(l.px + 10, g.y + g.h() + 8, &t, INK);
                    }
                }
            }
        }
        // The stack being carried follows the mouse.
        if let Some(hs) = self.held {
            let (x, y) = (mouse.x as i32 - 4, mouse.y as i32 - 4);
            c.sprite(a.tex(a.icon(hs.item.def().icon)), x, y);
            if hs.n > 1 {
                c.tiny(x + 16, y + 11, &hs.n.to_string(), WHITE, INK);
            }
        }
        let _ = ITEMS.len();
    }
}

fn tabs(c: &mut Canvas, l: &Layout, names: &[&str], sel: usize, widths: &[i32]) {
    let mut x = l.px + 8;
    for (i, n) in names.iter().enumerate() {
        let w = widths[i];
        if i == sel {
            c.rect(x, l.py + 5, w, 12, GOLD);
            c.frame(x, l.py + 5, w, 12, RUST);
        }
        c.text_center(
            x + w / 2,
            l.py + 7,
            n,
            if i == sel { INK } else { ROSEWOOD },
        );
        x += w + 4;
    }
}

/// The dialog box, tall enough for `lines` lines of text.
pub fn dialog_layout(w: i32, h: i32, lines: i32) -> Layout {
    let pw = (w - 40).min(320);
    let ph = (14 + lines * 10).max(52);
    Layout {
        px: (w - pw) / 2,
        py: h - 24 - ph,
        pw,
        ph,
    }
}
