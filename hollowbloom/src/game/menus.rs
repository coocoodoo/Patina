//! Menus: the bag with worn gear, stats and crafting, chests, the shop, the shipping bin, the
//! Hollow elevator, dialogs, pause/settings and the morning summary. Mouse and keyboard both
//! work. The enchanting table lives in `enchant.rs`.

use glam::Vec2;

use super::folk::Villager;
use super::gear::{Rarity, SLOTS, Slot, Stat};
use super::items::{CATS, Inventory, Item, Kind, RECIPES, Recipe, Stack, seasonal_seeds};
use super::loot;
use super::play::{Play, Trans, transfer};
use super::player::BAG;
use super::shops;
use super::talk::Say;
use super::tips::{draw_money, money_width, stat_icon};
use super::town::Place;
use super::{Io, Settings};
use crate::assets::Assets;
use crate::audio::Sfx;
use crate::input::{Action, Button, Input, KeyCode};
use crate::palette::*;
use crate::ui::{Canvas, Style};

pub const CELL: i32 = 19;
/// Every menu window is this big.
pub const PW: i32 = 236;
pub const PH: i32 = 180;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Choice {
    Close,
    Sleep,
    ReturnHome,
    Bus,
}

pub struct Summary {
    pub day: u32,
    pub earned: u64,
    pub grown: u32,
    pub ready: u32,
    /// Weeds and bushes that sprang up on the farm.
    pub sprouted: u32,
    pub rain: bool,
    pub passed_out: bool,
    pub fainted: bool,
    /// Anything else worth knowing this morning: a new season, the egg.
    pub notes: Vec<(String, u8)>,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Tab {
    Bag,
    Stats,
    Craft,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ShopTab {
    Seeds,
    Goods,
    Specials,
    Sell,
}

/// A piece of gear chosen in a menu: in the bag, or worn.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Pick {
    Bag(usize),
    Worn(Slot),
}

pub enum Menu {
    None,
    Inventory {
        tab: Tab,
        /// The bag and the backpack's pouch (bag indices), or the worn gear and the
        /// backpack (see `WORN_AT`, `PACK_AT`).
        cursor: usize,
        recipe: usize,
        scroll: usize,
        /// Recipe category: 0 = everything, then one per `Cat`.
        cat: usize,
    },
    Chest {
        x: i32,
        z: i32,
        cursor: usize,
    },
    Shop {
        /// The town shop, or Burrowby's stall on the farm.
        at: Option<Place>,
        tab: ShopTab,
        cursor: usize,
        scroll: usize,
    },
    /// A conversation.
    Talk {
        who: Villager,
        text: String,
        shown: f32,
        choices: Vec<(String, Say)>,
        sel: usize,
        blip: f32,
    },
    /// The request board or the guild's bounty board.
    Board {
        guild: bool,
        sel: usize,
    },
    /// Quests, friends and records.
    Journal {
        tab: usize,
        sel: usize,
    },
    /// Hazel's spells: learning them (tab 0) and the attuning circle (tab 1).
    Spells {
        tab: usize,
        sel: usize,
    },
    /// Quest complete!
    Cheer,
    /// Every control, on the Steam Deck (`deck`) or the keyboard.
    Controls {
        deck: bool,
    },
    /// A recipe just worked out: its picture, name and what it's good for, until OK.
    Recipe {
        recipe: usize,
        t: f32,
    },
    /// A fish tank in the house: the fish in it and your bag.
    Tank {
        x: i32,
        z: i32,
        /// 0..cap the tank, cap.. the bag.
        cursor: usize,
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
    Enchant {
        gear: Option<Pick>,
        scroll: Option<usize>,
        socket: usize,
        /// 0..40 the bag, 40..45 worn gear, 45..48 sockets, 48 the button.
        cursor: usize,
        msg: Option<(String, u8)>,
        /// Sparkle timer after a successful enchantment.
        glow: f32,
    },
}

impl Menu {
    pub fn inventory(craft: bool) -> Menu {
        Menu::Inventory {
            tab: if craft { Tab::Craft } else { Tab::Bag },
            cursor: 0,
            recipe: 0,
            scroll: 0,
            cat: 0,
        }
    }
    pub fn shop() -> Menu {
        Menu::Shop {
            at: None,
            tab: ShopTab::Seeds,
            cursor: 0,
            scroll: 0,
        }
    }
    /// A town shop, opened on its first tab.
    pub fn shop_at(place: Place) -> Menu {
        Menu::Shop {
            at: Some(place),
            tab: shops::tabs(Some(place))[0],
            cursor: 0,
            scroll: 0,
        }
    }
    pub fn enchant() -> Menu {
        Menu::Enchant {
            gear: None,
            scroll: None,
            socket: 0,
            cursor: 0,
            msg: None,
            glow: 0.0,
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

pub fn inside(p: Vec2, x: i32, y: i32, w: i32, h: i32) -> bool {
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

/// Moves a cursor over the bag (0..40) and the worn-gear column (40..45) to its left.
pub fn nav_bag(io: &Io, cursor: &mut usize) {
    let i = io.input;
    let before = *cursor;
    if *cursor >= 40 {
        let k = *cursor - 40;
        if i.pressed_repeat(Action::Down) {
            *cursor = 40 + (k + 1) % 5;
        }
        if i.pressed_repeat(Action::Up) {
            *cursor = 40 + (k + 4) % 5;
        }
        if i.pressed_repeat(Action::Right) {
            *cursor = k.min(3) * 10;
        }
    } else {
        let (col, row) = (*cursor % 10, *cursor / 10);
        if i.pressed_repeat(Action::Left) {
            *cursor = if col == 0 { 40 + row } else { *cursor - 1 };
        } else if i.pressed_repeat(Action::Right) {
            *cursor = if col == 9 { row * 10 } else { *cursor + 1 };
        } else if i.pressed_repeat(Action::Down) {
            *cursor = (*cursor + 10) % 40;
        } else if i.pressed_repeat(Action::Up) {
            *cursor = (*cursor + 30) % 40;
        }
    }
    if before != *cursor {
        io.audio.play_at(Sfx::UiMove, 0.5, 1.0);
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

pub fn panel_layout(w: i32, h: i32) -> Layout {
    center(w, h, PW, PH)
}

/// The bag grid inside a panel, `top` pixels down, shifted right of the worn-gear column.
pub fn bag_grid(l: &Layout, top: i32) -> Grid {
    Grid {
        x: l.px + 34,
        y: l.py + top,
        cols: 10,
        rows: 4,
        gap: 3,
    }
}

/// The worn-gear column beside the bag.
pub fn worn_pos(l: &Layout, top: i32, slot: usize) -> (i32, i32) {
    (l.px + 9, l.py + top + slot as i32 * CELL)
}

pub fn worn_hit(l: &Layout, top: i32, p: Vec2) -> Option<usize> {
    (0..5).find(|&i| {
        let (x, y) = worn_pos(l, top, i);
        inside(p, x, y, 18, 18)
    })
}

/// What's said when a backpack's pouch holds more than the rest of the bag has room for.
const NO_ROOM: &str = "No room in your bag for what's in the backpack!";

/// Armour and backpacks: things you wear.
pub fn wearable(s: &Stack) -> bool {
    s.item.pack_style().is_some() || s.item.class().is_some_and(|c| c.is_armor())
}

/// Bag-tab cursor spots past the bag and the backpack's pouch (which are bag indices): the
/// worn gear (head to shield), and the backpack itself.
pub const WORN_AT: usize = 100;
pub const PACK_AT: usize = 110;

/// Columns in a backpack's pouch.
pub const POUCH_COLS: usize = 5;
/// The pouch's panel: how wide it is, and the gap between it and the main panel.
pub const POUCH_W: i32 = POUCH_COLS as i32 * CELL - 1 + 14;
pub const POUCH_GAP: i32 = 3;

/// The main panel, moved over to make room for the pouch's panel beside it (when there's
/// one to show).
pub fn pouch_layout(w: i32, h: i32, pouch: bool) -> Layout {
    let l = panel_layout(w, h);
    if pouch {
        Layout {
            px: ((w - l.pw - POUCH_GAP - POUCH_W) / 2).max(2),
            ..l
        }
    } else {
        l
    }
}

/// A backpack's pouch: its slots (the bag's from `BAG` on) in a panel of their own beside
/// the bag, row for row with it; on the bag tab, with the backpack's own slot below.
pub struct Pouch {
    pub x: i32,
    pub y: i32,
    /// How many slots it has.
    pub n: usize,
    /// The bag's extra gap after its first row, so the rows line up.
    pub gap: i32,
    /// Shows the backpack's own slot.
    pub slot: bool,
}

impl Pouch {
    pub fn rows(&self) -> usize {
        self.n.div_ceil(POUCH_COLS)
    }

    /// Where pouch slot `j` is (slot 0 is the bag's slot `BAG`).
    pub fn slot_pos(&self, j: usize) -> (i32, i32) {
        let (c, r) = ((j % POUCH_COLS) as i32, (j / POUCH_COLS) as i32);
        let gap = if r > 0 { self.gap } else { 0 };
        (self.x + c * CELL, self.y + r * CELL + gap)
    }

    pub fn hit(&self, p: Vec2) -> Option<usize> {
        (0..self.n).find(|&j| {
            let (x, y) = self.slot_pos(j);
            inside(p, x, y, 18, 18)
        })
    }

    /// Where the backpack's own slot is: under the pouch's slots.
    pub fn pack_pos(&self) -> (i32, i32) {
        let rows = self.rows() as i32;
        let gap = if rows > 1 { self.gap } else { 0 };
        let below = if rows > 0 { 12 } else { 0 };
        (self.x, self.y + rows * CELL + gap + below)
    }

    pub fn on_pack(&self, p: Vec2) -> bool {
        let (x, y) = self.pack_pos();
        self.slot && inside(p, x, y, 18, 18)
    }

    /// The panel round it all: (x, y, w, h).
    pub fn frame(&self) -> (i32, i32, i32, i32) {
        let top = self.y - 16;
        let bottom = if self.slot {
            self.pack_pos().1 + 18 + 16
        } else {
            self.slot_pos(self.n.saturating_sub(1)).1 + 18 + 7
        };
        (self.x - 7, top, POUCH_W, bottom - top)
    }
}

/// How wide the main panel and the pouch's panel beside it are, together.
pub fn w_with_pouch(l: &Layout) -> i32 {
    l.pw + POUCH_GAP + POUCH_W
}

/// The pouch of a backpack with `n` slots beside a bag grid.
pub fn pouch_of(l: &Layout, bag: &Grid, n: usize, slot: bool) -> Pouch {
    Pouch {
        x: l.px + l.pw + POUCH_GAP + 7,
        y: bag.y,
        n,
        gap: bag.gap,
        slot,
    }
}

/// Moves a bag cursor with the arrow keys over the bag (0..40, ten across) and, beside it,
/// the backpack's pouch (from 40, `POUCH_COLS` across, row for row with the bag). `edge` is
/// where the cursor goes off either side: the bag tab's worn gear and backpack, say.
fn step_bag(i: &Input, cursor: usize, pouch: usize, edge: (Option<usize>, Option<usize>)) -> usize {
    let (left, right) = edge;
    if cursor < BAG {
        let (col, row) = (cursor % 10, cursor / 10);
        if i.pressed_repeat(Action::Left) {
            if col > 0 {
                return cursor - 1;
            }
            return left.unwrap_or(row * 10 + 9);
        }
        if i.pressed_repeat(Action::Right) {
            if col < 9 {
                return cursor + 1;
            }
            if row * POUCH_COLS < pouch {
                return BAG + row * POUCH_COLS;
            }
            return right.unwrap_or(row * 10);
        }
        if i.pressed_repeat(Action::Down) {
            return (cursor + 10) % BAG;
        }
        if i.pressed_repeat(Action::Up) {
            return (cursor + BAG - 10) % BAG;
        }
        return cursor;
    }
    let j = cursor - BAG;
    let (col, row) = (j % POUCH_COLS, j / POUCH_COLS);
    if i.pressed_repeat(Action::Left) {
        if col > 0 {
            return cursor - 1;
        }
        return row.min(3) * 10 + 9;
    }
    if i.pressed_repeat(Action::Right) {
        if col + 1 < POUCH_COLS && j + 1 < pouch {
            return cursor + 1;
        }
        return right.unwrap_or(row.min(3) * 10);
    }
    if i.pressed_repeat(Action::Down) {
        if j + POUCH_COLS < pouch {
            return cursor + POUCH_COLS;
        }
        return right.unwrap_or(BAG + col);
    }
    if i.pressed_repeat(Action::Up) {
        if j >= POUCH_COLS {
            return cursor - POUCH_COLS;
        }
        // Round to the bottom of the column.
        let last = (pouch - 1 - col) / POUCH_COLS * POUCH_COLS + col;
        return if right.is_some() { cursor } else { BAG + last };
    }
    cursor
}

/// Moves the bag tab's cursor: over the worn gear, the bag, the backpack's pouch and the
/// backpack's own slot (see `WORN_AT`, `PACK_AT`).
pub fn nav_inventory(io: &Io, cursor: &mut usize, pouch: usize) {
    let i = io.input;
    let before = *cursor;
    if *cursor >= PACK_AT {
        if i.pressed_repeat(Action::Up) && pouch > 0 {
            *cursor = BAG + (pouch - 1) / POUCH_COLS * POUCH_COLS;
        } else if i.pressed_repeat(Action::Left) {
            *cursor = if pouch > 0 { BAG + pouch - 1 } else { 9 };
        } else if i.pressed_repeat(Action::Right) {
            *cursor = WORN_AT;
        }
    } else if *cursor >= WORN_AT {
        let k = *cursor - WORN_AT;
        if i.pressed_repeat(Action::Down) {
            *cursor = WORN_AT + (k + 1) % 5;
        } else if i.pressed_repeat(Action::Up) {
            *cursor = WORN_AT + (k + 4) % 5;
        } else if i.pressed_repeat(Action::Right) {
            *cursor = k.min(3) * 10;
        } else if i.pressed_repeat(Action::Left) {
            *cursor = PACK_AT;
        }
    } else {
        let row = if *cursor < BAG { *cursor / 10 } else { 0 };
        *cursor = step_bag(i, *cursor, pouch, (Some(WORN_AT + row), Some(PACK_AT)));
    }
    if before != *cursor {
        io.audio.play_at(Sfx::UiMove, 0.5, 1.0);
    }
}

/// Moves a cursor over the bag and the pouch beside it (as bag indices), with `base` the
/// cursor of the bag's first slot and anything before it (a chest's slots, say) handled by
/// `nav` over `before` slots ten across. Returns true if it moved.
pub fn nav_bag_pouch(io: &Io, cursor: &mut usize, base: usize, pouch: usize) -> bool {
    let i = io.input;
    let before = *cursor;
    if *cursor < base {
        // Above the bag: plain rows of ten, dropping into the bag's first row.
        nav(io, cursor, 10, base + BAG);
        return before != *cursor;
    }
    let k = *cursor - base;
    let up_out = k < 10 && base > 0 && i.pressed_repeat(Action::Up);
    let down_out = (30..BAG).contains(&k) && base > 0 && i.pressed_repeat(Action::Down);
    if up_out {
        *cursor = base - 10 + k;
    } else if down_out {
        *cursor = k % 10;
    } else {
        *cursor = base + step_bag(i, k, pouch, (None, None));
    }
    if before != *cursor {
        io.audio.play_at(Sfx::UiMove, 0.5, 1.0);
    }
    before != *cursor
}

/// Seeds Burrowby sells; more as you go deeper.
pub fn shop_seeds(p: &Play) -> Vec<(Item, u32)> {
    let d = p.deepest;
    let mut v = vec![
        (Item::TurnipSeeds, 20),
        (Item::CarrotSeeds, 40),
        (Item::RadishSeeds, 25),
        (Item::SeedPotato, 40),
        (Item::WheatSeeds, 15),
        (Item::GarlicBulb, 40),
        (Item::PeaSeeds, 35),
        (Item::TomatoSeeds, 60),
        (Item::CabbageSeeds, 70),
    ];
    let unlocks: [(u32, Item, u32); 23] = [
        (2, Item::StrawberrySeeds, 90),
        (3, Item::GlowcapSpores, 70),
        (3, Item::RoseSeeds, 70),
        (4, Item::MossberrySeeds, 60),
        (5, Item::CornSeeds, 80),
        (5, Item::SunflowerSeeds, 80),
        (6, Item::BunnyrootSeeds, 50),
        (7, Item::BlueberrySeeds, 100),
        (7, Item::EggplantSeeds, 60),
        (8, Item::MelonSeeds, 110),
        (11, Item::BerrySeeds, 90),
        (15, Item::PrismPearSeeds, 140),
        (21, Item::PumpkinSeeds, 140),
        (22, Item::PuffballSpores, 60),
        (26, Item::JellySpores, 120),
        (31, Item::PepperSeeds, 130),
        (33, Item::FlameTulipBulb, 140),
        (35, Item::LavaLemonSeeds, 150),
        (41, Item::LilyBulb, 170),
        (43, Item::SnowPeaSeeds, 110),
        (45, Item::FrostMintSeeds, 100),
        (53, Item::GhostPepperSeeds, 180),
        (55, Item::AncientGrainSeeds, 200),
    ];
    for (need, item, price) in unlocks {
        if d >= need {
            v.push((item, price));
        }
    }
    // The cheaper seeds of the season; Posy keeps the whole range.
    for item in seasonal_seeds(p.clock.season().bit()) {
        if item.def().price <= 40 {
            v.push((item, item.def().price * 2));
        }
    }
    v
}

/// Supplies, food and furniture.
pub fn shop_goods(p: &Play) -> Vec<(Item, u32)> {
    let d = p.deepest;
    let mut v = vec![
        (Item::Torch, 12),
        (Item::Vial, 10),
        (Item::SmallHealthPotion, 70),
        (Item::SmallManaPotion, 80),
        (Item::HealingTonic, 120),
        (Item::ManaTonic, 120),
        (Item::StaminaTonic, 120),
        (Item::FreshBread, 110),
        (Item::Feather, 300),
        (Item::Fence, 8),
        (Item::StonePath, 5),
        (Item::WoodPath, 5),
        (Item::FlowerPot, 60),
        (Item::Bench, 120),
        (Item::Chest, 160),
        (Item::Lamp, 200),
        (Item::EnchantTable, 900),
        (Item::Sickle, 60),
        (Item::TwigWand, 50),
        (Item::PotLid, 40),
    ];
    if d >= 5 {
        v.push((Item::Sprinkler, 450));
    }
    if d >= 12 {
        v.push((Item::Ruby, 240));
        v.push((Item::Sapphire, 240));
        v.push((Item::Emerald, 240));
    }
    v
}

/// Things you can sell (the four starter tools stay with you).
pub fn can_sell(s: &Stack) -> bool {
    !matches!(s.item, Item::Hoe | Item::Can0 | Item::Axe0 | Item::Pick0) && s.unit_price() > 0
}

/// The level a crafted piece of gear comes out at.
pub(super) fn crafted_level(p: &Play, item: Item) -> u16 {
    let b = item.base().map_or(1, |b| b.lvl);
    b.max((p.deepest as u16).min(b + 8))
}

/// Recipes shown under a category (0 = all).
pub fn recipes_in(cat: usize) -> Vec<&'static Recipe> {
    RECIPES
        .iter()
        .filter(|r| cat == 0 || r.cat == CATS[cat - 1])
        .collect()
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
            (None, Some(s), true) if s.n > 1 => {
                let half = s.n.div_ceil(2);
                *held = Some(Stack { n: half, ..s });
                inv.slots[i] = Some(Stack { n: s.n - half, ..s });
            }
            (None, Some(s), true) => {
                *held = Some(s);
                inv.slots[i] = None;
            }
            (Some(h), None, false) => {
                inv.slots[i] = Some(h);
                *held = None;
            }
            (Some(h), None, true) => {
                inv.slots[i] = Some(Stack { n: 1, ..h });
                *held = if h.n > 1 {
                    Some(Stack { n: h.n - 1, ..h })
                } else {
                    None
                };
            }
            (Some(h), Some(s), _) if s.stacks_with(&h) => {
                let max = s.item.def().stack;
                let add = if right { 1 } else { h.n };
                let k = (max - s.n.min(max)).min(add);
                inv.slots[i] = Some(Stack { n: s.n + k, ..s });
                *held = if h.n - k > 0 {
                    Some(Stack { n: h.n - k, ..h })
                } else {
                    None
                };
            }
            (Some(h), Some(s), _) => {
                inv.slots[i] = Some(h);
                *held = Some(s);
            }
            _ => {}
        }
    }

    /// Clicking a worn-gear slot: wear what is on the cursor, or take the piece off.
    fn worn_click(&mut self, slot: usize, right: bool, io: &Io) {
        let sl = SLOTS[slot];
        match self.held {
            Some(h) => {
                if h.item.class().and_then(|c| c.slot()) == Some(sl) {
                    self.held = self.player.equip[slot].take();
                    self.player.equip[slot] = Some(h);
                    self.player.refresh();
                    io.audio.play(Sfx::Equip);
                } else {
                    io.audio.play(Sfx::Denied);
                }
            }
            None => {
                if right {
                    if self.player.unequip(sl) {
                        io.audio.play(Sfx::Equip);
                    }
                } else if let Some(s) = self.player.equip[slot].take() {
                    self.held = Some(s);
                    self.player.refresh();
                    io.audio.play_at(Sfx::UiMove, 0.8, 1.2);
                }
            }
        }
    }

    /// Wears whatever's in bag slot `i`: a piece of armour, or a backpack.
    fn wear_from(&mut self, i: usize, io: &Io) {
        let Some(s) = self.player.inv.slots.get(i).copied().flatten() else {
            return;
        };
        let worn = if s.item.pack_style().is_some() {
            let ok = self.player.wear_pack_from(i);
            if !ok {
                self.toast(NO_ROOM, None, 0);
            }
            ok
        } else {
            self.player.equip_from(i)
        };
        io.audio.play(if worn { Sfx::Equip } else { Sfx::Denied });
    }

    /// The backpack's own slot: put one on (or swap it for the one on the cursor), take it
    /// off onto the cursor, or (`quick`) straight into the bag.
    fn pack_click(&mut self, quick: bool, io: &Io) {
        match self.held {
            Some(h) if h.item.pack_style().is_some() => match self.player.swap_pack(h) {
                Ok(old) => {
                    self.held = old;
                    io.audio.play(Sfx::Equip);
                }
                Err(_) => {
                    self.toast(NO_ROOM, None, 0);
                    io.audio.play(Sfx::Denied);
                }
            },
            Some(_) => io.audio.play(Sfx::Denied),
            None if self.player.pack.is_none() => {}
            None => {
                let off = if quick {
                    self.player.unwear_pack()
                } else {
                    match self.player.lift_pack() {
                        Some(p) => {
                            self.held = Some(p);
                            true
                        }
                        None => false,
                    }
                };
                if off {
                    io.audio.play_at(Sfx::Equip, 0.8, 0.9);
                } else {
                    self.toast(NO_ROOM, None, 0);
                    io.audio.play(Sfx::Denied);
                }
            }
        }
    }

    /// Puts whatever is on the cursor back into the bag (or on the ground).
    fn return_held(&mut self) {
        if let Some(h) = self.held.take() {
            let left = self.player.inv.add_stack(h);
            if left > 0 {
                self.drop_stack(Stack { n: left, ..h });
            }
        }
    }

    fn drop_stack(&mut self, s: Stack) {
        let at = self.player.world_pos()
            + glam::Vec3::new(self.player.facing.x * 0.6, 0.0, self.player.facing.y * 0.6);
        let mut d = super::fx::Drop::new(s, at, &mut self.rng);
        d.age = -0.8;
        self.drops.push(d);
    }

    pub fn close_menu(&mut self, io: &Io) {
        self.return_held();
        self.menu = Menu::None;
        io.audio.play(Sfx::UiBack);
    }

    /// Crafts a recipe: gear and scrolls come out freshly rolled. Dishes go on the stove
    /// instead (see `home::Cooking`).
    fn craft(&mut self, r: &Recipe, book: (usize, usize, usize), io: &mut Io) -> bool {
        if !self.known.contains(&r.out) || !r.can_craft(&self.player.inv) {
            io.audio.play(Sfx::Denied);
            return false;
        }
        if r.cat == super::items::Cat::Kitchen {
            let k = RECIPES.iter().position(|x| std::ptr::eq(x, r)).unwrap_or(0);
            return self.start_cooking(k, book, io);
        }
        for (item, k) in r.needs {
            self.player.inv.take(*item, *k as u32);
        }
        let luck = self.player.luck();
        let made = if r.out.base().is_some() {
            let lvl = crafted_level(self, r.out);
            loot::roll_gear(r.out, lvl, luck, &mut self.rng)
        } else if let Some(g) = r.out.scroll_group() {
            let depth = self.deepest.max(1);
            loot::scroll_of(g, depth, loot::Fortune { luck, greed: 1.0 }, &mut self.rng)
        } else {
            Stack::new(r.out, r.n)
        };
        let left = self.player.inv.add_stack(made);
        if left > 0 {
            self.drop_stack(Stack { n: left, ..made });
        }
        self.on_craft(made.item);
        if matches!(made.item.def().kind, Kind::Potion { .. }) {
            self.on_brew(made.n);
            io.audio.play(Sfx::Brew);
        }
        io.audio.play(Sfx::Craft);
        match made.rarity() {
            Some(rar) => {
                let text = format!("Crafted {} {}", rar.name(), made.name());
                self.toast_colored(text, Some(made.item), 0, rar.color());
                if rar >= Rarity::Rare {
                    io.audio.play(Sfx::Rare);
                }
            }
            None => self.toast(
                format!("Crafted {}", made.name()),
                Some(made.item),
                r.n as u32,
            ),
        }
        true
    }

    pub fn update_menu(&mut self, io: &mut Io, settings: &mut Settings) {
        let (w, h) = (io.view.0 as i32, io.view.1 as i32);
        let input = io.input;
        let mouse = input.mouse;
        let lclick = input.button_pressed(Button::Left);
        let rclick = input.button_pressed(Button::Right);
        let shift = input.key_down(KeyCode::ShiftLeft) || input.key_down(KeyCode::ShiftRight);
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
                            Choice::Bus => self.call_bus(io),
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
                let items = 7;
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
                    if picked == Some(4) || left && sel == 4 || right && sel == 4 {
                        settings.shadows = !settings.shadows;
                    }
                    if picked == Some(5) || left && sel == 5 || right && sel == 5 {
                        settings.ao = !settings.ao;
                    }
                    io.audio.set_volume(settings.music, settings.sfx);
                    if picked.is_some() || left || right {
                        io.audio.play(Sfx::UiSelect);
                        settings.dirty = true;
                    }
                    if picked == Some(6) || input.pressed(Action::Cancel) || rclick {
                        Menu::Pause {
                            sel: 3,
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
                    // The journal and the map are here too, for pads whose sticks don't
                    // click in.
                    match picked {
                        Some(0) => {
                            io.audio.play(Sfx::UiBack);
                            Menu::None
                        }
                        Some(1) => {
                            io.audio.play(Sfx::UiSelect);
                            Menu::Journal { tab: 0, sel: 0 }
                        }
                        Some(2) => {
                            io.audio.play(Sfx::UiSelect);
                            self.show_map = !self.show_map;
                            Menu::None
                        }
                        Some(3) => {
                            io.audio.play(Sfx::UiSelect);
                            Menu::Pause {
                                sel: 0,
                                settings: true,
                            }
                        }
                        Some(4) => {
                            io.audio.play(Sfx::UiSelect);
                            Menu::Controls { deck: self.pad }
                        }
                        Some(5) => {
                            self.quit_to_title = true;
                            Menu::None
                        }
                        Some(6) => {
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
                mut tab,
                mut cursor,
                mut recipe,
                mut scroll,
                mut cat,
            } => {
                let l = pouch_layout(w, h, true);
                // Tabs.
                for (i, (x, tw)) in tab_rects(&l, &[34, 38, 54]).into_iter().enumerate() {
                    if lclick && inside(mouse, x, l.py + 5, tw, 12) {
                        tab = [Tab::Bag, Tab::Stats, Tab::Craft][i];
                        io.audio.play(Sfx::UiMove);
                    }
                }
                if input.pressed(Action::Crafting) {
                    tab = if tab == Tab::Craft {
                        Tab::Bag
                    } else {
                        Tab::Craft
                    };
                    io.audio.play(Sfx::UiMove);
                }
                let close = input.pressed(Action::Cancel) || input.pressed(Action::Inventory);
                if close {
                    self.close_menu(io);
                    return;
                }
                // L1/R1 (or [ and ]) flip between the bag and your stats; the crafting book
                // keeps them for its categories.
                if tab != Tab::Craft {
                    let flip = input.pressed(Action::NextSlot) || input.pressed(Action::PrevSlot);
                    if flip {
                        tab = if tab == Tab::Bag {
                            Tab::Stats
                        } else {
                            Tab::Bag
                        };
                        io.audio.play(Sfx::UiMove);
                    }
                }
                match tab {
                    Tab::Bag => {
                        let g = bag_grid(&l, 22);
                        let pouch = pouch_of(&l, &g, self.player.pack_slots(), true);
                        nav_inventory(io, &mut cursor, pouch.n);
                        let hit = g.hit(mouse).or_else(|| pouch.hit(mouse).map(|j| BAG + j));
                        let worn = worn_hit(&l, 22, mouse);
                        let on_pack = pouch.on_pack(mouse);
                        if input.mouse_moved {
                            if let Some(i) = hit {
                                cursor = i;
                            } else if let Some(i) = worn {
                                cursor = WORN_AT + i;
                            } else if on_pack {
                                cursor = PACK_AT;
                            }
                        }
                        if let Some(i) = hit.filter(|_| lclick || rclick) {
                            let wearable = self.player.inv.slots[i].is_some_and(|s| wearable(&s));
                            if (rclick || shift) && self.held.is_none() && wearable {
                                self.wear_from(i, io);
                            } else {
                                Self::grid_click(&mut self.held, &mut self.player.inv, i, rclick);
                                io.audio.play_at(Sfx::UiMove, 0.8, 1.2);
                            }
                        } else if let Some(i) = worn.filter(|_| lclick || rclick) {
                            self.worn_click(i, rclick || shift, io);
                        } else if on_pack && (lclick || rclick) {
                            self.pack_click(rclick || shift, io);
                        } else if input.pressed(Action::Alt) || input.pressed(Action::Confirm) {
                            // A controller's buttons: A picks up (or wears, or swaps), X is a
                            // right click (take off, wear, or split a stack).
                            let alt = input.pressed(Action::Alt);
                            if cursor >= PACK_AT {
                                self.pack_click(alt || self.held.is_none(), io);
                            } else if cursor >= WORN_AT {
                                self.worn_click(cursor - WORN_AT, alt || self.held.is_none(), io);
                            } else if cursor < self.player.inv.slots.len() {
                                let wearable =
                                    self.player.inv.slots[cursor].is_some_and(|s| wearable(&s));
                                if wearable && self.held.is_none() {
                                    self.wear_from(cursor, io);
                                } else {
                                    Self::grid_click(
                                        &mut self.held,
                                        &mut self.player.inv,
                                        cursor,
                                        alt,
                                    );
                                    io.audio.play_at(Sfx::UiMove, 0.8, 1.2);
                                }
                            }
                        } else if lclick && !inside(mouse, l.px, l.py, l.pw, l.ph) {
                            let (fx, fy, fw, fh) = pouch.frame();
                            if !inside(mouse, fx, fy, fw, fh) {
                                if let Some(hs) = self.held.take() {
                                    self.drop_stack(hs);
                                    io.audio.play(Sfx::Place);
                                }
                            }
                        }
                        // A backpack taken off takes its pouch with it.
                        if cursor < WORN_AT && cursor >= self.player.inv.slots.len() {
                            cursor = PACK_AT;
                        }
                    }
                    Tab::Stats => {}
                    Tab::Craft => {
                        // Categories.
                        let ncat = CATS.len() + 1;
                        for (i, (x, cw)) in cat_rects(&l).into_iter().enumerate() {
                            if lclick && inside(mouse, x, l.py + 21, cw, 11) {
                                cat = i;
                                recipe = 0;
                                scroll = 0;
                                io.audio.play_at(Sfx::UiMove, 0.6, 1.0);
                            }
                        }
                        if input.pressed(Action::NextSlot) || input.pressed_repeat(Action::Right) {
                            cat = (cat + 1) % ncat;
                            recipe = 0;
                            scroll = 0;
                            io.audio.play_at(Sfx::UiMove, 0.5, 1.0);
                        }
                        if input.pressed(Action::PrevSlot) || input.pressed_repeat(Action::Left) {
                            cat = (cat + ncat - 1) % ncat;
                            recipe = 0;
                            scroll = 0;
                            io.audio.play_at(Sfx::UiMove, 0.5, 1.0);
                        }
                        let list = recipes_in(cat);
                        let n = list.len().max(1);
                        let rows = 7;
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
                                .clamp(0, n.saturating_sub(rows) as i32)
                                as usize;
                        }
                        recipe = recipe.min(n - 1);
                        if recipe < scroll {
                            scroll = recipe;
                        }
                        if recipe >= scroll + rows {
                            scroll = recipe + 1 - rows;
                        }
                        for row in 0..rows {
                            let y = l.py + 36 + row as i32 * 18;
                            if inside(mouse, l.px + 6, y, 118, 17) && lclick && scroll + row < n {
                                recipe = scroll + row;
                                io.audio.play_at(Sfx::UiMove, 0.6, 1.0);
                            }
                        }
                        let (bx, by, bw, bh) = craft_button(&l);
                        let craft_now = input.pressed(Action::Confirm)
                            || lclick && inside(mouse, bx, by, bw, bh);
                        if craft_now {
                            if let Some(r) = list.get(recipe) {
                                self.craft(r, (cat, recipe, scroll), io);
                            }
                        }
                    }
                }
                if self.cooking.is_some() {
                    // Off to the stove; the book opens again when the dish is done.
                    self.return_held();
                    Menu::None
                } else {
                    Menu::Inventory {
                        tab,
                        cursor,
                        recipe,
                        scroll,
                        cat,
                    }
                }
            }
            Menu::Chest { x, z, mut cursor } => {
                let n = self.player.pack_slots();
                let l = pouch_layout(w, h, n > 0);
                let cg = chest_grid(&l);
                let bg = bag_grid(&l, 20 + 3 * CELL + 16);
                let pouch = pouch_of(&l, &bg, n, false);
                if input.pressed(Action::Cancel) || input.pressed(Action::Inventory) {
                    self.close_menu(io);
                    return;
                }
                nav_bag_pouch(io, &mut cursor, 30, n);
                cursor = cursor.min(30 + BAG + n - 1);
                let mut chest = match self.world_mut().obj_mut(x, z) {
                    Some(super::world::Obj::Chest { items }) => Inventory {
                        slots: std::mem::take(items),
                    },
                    _ => {
                        self.return_held();
                        return;
                    }
                };
                if let Some(i) = cg.hit(mouse) {
                    if input.mouse_moved {
                        cursor = i;
                    }
                    if lclick && shift {
                        transfer(&mut chest, i, &mut self.player.inv);
                    } else if lclick || rclick {
                        Self::grid_click(&mut self.held, &mut chest, i, rclick);
                    }
                } else if let Some(i) = bg.hit(mouse).or_else(|| pouch.hit(mouse).map(|j| BAG + j))
                {
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
                if let Some(super::world::Obj::Chest { items }) = self.world_mut().obj_mut(x, z) {
                    *items = chest.slots;
                }
                Menu::Chest { x, z, cursor }
            }
            Menu::Talk {
                who,
                text,
                shown,
                choices,
                sel,
                blip,
            } => self.update_talk(io, who, text, shown, choices, sel, blip),
            Menu::Board { guild, sel } => self.update_board(io, guild, sel),
            Menu::Journal { tab, sel } => self.update_journal(io, tab, sel),
            Menu::Spells { tab, sel } => self.update_spellery(io, tab, sel),
            Menu::Cheer => self.update_cheer(io),
            Menu::Recipe { recipe, t } => self.update_recipe_card(io, recipe, t),
            Menu::Controls { deck } => self.update_controls(io, deck),
            Menu::Tank { x, z, cursor } => self.update_tank(io, x, z, cursor),
            Menu::Shop {
                at,
                mut tab,
                mut cursor,
                mut scroll,
            } => {
                let l = pouch_layout(w, h, self.player.pack_slots() > 0);
                if input.pressed(Action::Cancel) {
                    self.close_menu(io);
                    return;
                }
                let tabs = shops::tabs(at);
                let nt = tabs.len();
                let before = tab;
                let widths = shop_tab_widths(at);
                for (i, (x, tw)) in tab_rects(&l, &widths).into_iter().enumerate() {
                    if lclick && inside(mouse, x, l.py + 5, tw, 12) {
                        tab = tabs[i];
                    }
                }
                let ti = tabs.iter().position(|t| *t == tab).unwrap_or(0);
                if input.pressed(Action::Crafting)
                    || input.pressed(Action::Inventory)
                    || input.pressed(Action::NextSlot)
                {
                    tab = tabs[(ti + 1) % nt];
                }
                if input.pressed(Action::PrevSlot) {
                    tab = tabs[(ti + nt - 1) % nt];
                }
                if tab != before {
                    cursor = 0;
                    scroll = 0;
                    io.audio.play(Sfx::UiMove);
                }
                if tab != ShopTab::Sell {
                    let rows = 7;
                    let entries = shops::rows(self, at, tab);
                    let n = entries.len().max(1);
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
                        if let Some(&(stack, price, sold)) = entries.get(cursor) {
                            if !sold && self.money >= price && self.player.inv.can_fit_stack(&stack)
                            {
                                self.money -= price;
                                // Plain gear off the shelf still gets its own rolls, except
                                // fishing rods: the shop's are plain, the Hollow's are not.
                                let mut stack = match (stack.item.base(), stack.gear) {
                                    (Some(b), Some(g))
                                        if g.affixes.iter().all(|a| a.is_none())
                                            && b.class != super::gear::Class::Rod =>
                                    {
                                        loot::roll_gear(stack.item, b.lvl, 0.0, &mut self.rng)
                                    }
                                    _ => stack,
                                };
                                // A backpack off the shelf comes in colours of its own.
                                if let Some(p) = &mut stack.pack {
                                    p.hue = self.rng.below(crate::assets::pack_art::HUES) as u8;
                                }
                                self.player.inv.add_stack(stack);
                                self.on_pickup(stack.item);
                                if stack.item == Item::Bomb {
                                    self.bomb_stock = self.bomb_stock.saturating_sub(1);
                                }
                                if tab == ShopTab::Specials {
                                    self.bought.push(shops::special_id(at, cursor));
                                    if stack.rarity() >= Some(Rarity::Rare) {
                                        io.audio.play(Sfx::Rare);
                                    }
                                }
                                io.audio.play(Sfx::Coin);
                            } else {
                                io.audio.play(Sfx::Denied);
                            }
                        }
                    }
                } else {
                    let g = bag_grid(&l, 40);
                    let n = self.player.pack_slots();
                    let pouch = pouch_of(&l, &g, n, false);
                    nav_bag_pouch(io, &mut cursor, 0, n);
                    cursor = cursor.min(BAG + n - 1);
                    let mut target = None;
                    if let Some(i) = g.hit(mouse).or_else(|| pouch.hit(mouse).map(|j| BAG + j)) {
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
                    if input.pressed(Action::Alt) {
                        target = Some((cursor, true));
                    }
                    if let Some((i, one)) = target {
                        if let Some(s) = self.player.inv.slots[i] {
                            if can_sell(&s) {
                                let n = if one { 1 } else { s.n };
                                let v = s.unit_price() * n as u64 * self.sell_rate(at, &s) / 100;
                                self.money += v;
                                self.stats.earned += v;
                                self.player.inv.slots[i] = if s.n > n {
                                    Some(Stack { n: s.n - n, ..s })
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
                    at,
                    tab,
                    cursor,
                    scroll,
                }
            }
            Menu::Ship { mut cursor } => {
                let n = self.player.pack_slots();
                let l = pouch_layout(w, h, n > 0);
                if input.pressed(Action::Cancel) || input.pressed(Action::Inventory) {
                    self.close_menu(io);
                    return;
                }
                let g = bag_grid(&l, 60);
                let pouch = pouch_of(&l, &g, n, false);
                nav_bag_pouch(io, &mut cursor, 0, n);
                cursor = cursor.min(BAG + n - 1);
                let mut target = None;
                if let Some(i) = g.hit(mouse).or_else(|| pouch.hit(mouse).map(|j| BAG + j)) {
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
                        if can_sell(&s) && s.item.coin_value().is_none() {
                            let n = if one { 1 } else { s.n };
                            self.player.inv.slots[i] = if s.n > n {
                                Some(Stack { n: s.n - n, ..s })
                            } else {
                                None
                            };
                            let add = Stack { n, ..s };
                            if let Some(e) = self
                                .shipping
                                .iter_mut()
                                .find(|e| e.stacks_with(&add) && e.n < 999)
                            {
                                e.n += n;
                            } else {
                                self.shipping.push(add);
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
                        let left = self.player.inv.add_stack(s);
                        if left > 0 {
                            self.shipping.push(Stack { n: left, ..s });
                        }
                        io.audio.play(Sfx::UiBack);
                    }
                }
                Menu::Ship { cursor }
            }
            Menu::Enchant {
                gear,
                scroll,
                socket,
                cursor,
                msg,
                glow,
            } => self.update_enchant(io, gear, scroll, socket, cursor, msg, glow),
        };
    }

    // --------------------------------------------------------------------------------------
    // Drawing
    // --------------------------------------------------------------------------------------

    pub fn draw_grid(
        &self,
        c: &mut Canvas,
        a: &Assets,
        g: &Grid,
        inv: &Inventory,
        cursor: Option<usize>,
    ) {
        for i in 0..(g.cols * g.rows).min(inv.slots.len()) {
            let (x, y) = g.slot_pos(i);
            self.draw_slot(c, a, x, y, inv.slots[i], cursor == Some(i));
        }
    }

    /// A backpack's pouch in its panel beside the main one, and on the bag tab the backpack's
    /// own slot below it (with a faint outline when there's none on).
    pub fn draw_pouch(
        &self,
        c: &mut Canvas,
        a: &Assets,
        pouch: &Pouch,
        cursor: Option<usize>,
        on_pack: bool,
    ) {
        if pouch.n == 0 && !pouch.slot {
            return;
        }
        let (fx, fy, fw, fh) = pouch.frame();
        c.panel(fx, fy, fw, fh, Style::Paper);
        c.text(fx + 7, fy + 4, "Backpack", RUST);
        for j in 0..pouch.n {
            let (x, y) = pouch.slot_pos(j);
            let s = self.player.inv.slots.get(BAG + j).copied().flatten();
            self.draw_slot(c, a, x, y, s, cursor == Some(j));
        }
        if pouch.slot {
            let (x, y) = pouch.pack_pos();
            self.draw_slot(c, a, x, y, self.player.pack, on_pack);
            c.frame(x - 3, y - 3, 24, 24, KHAKI);
            match self.player.pack {
                Some(p) => {
                    let r = p.rarity().unwrap_or_default();
                    c.text(
                        x + 24,
                        y + 1,
                        &format!("+{} slots", p.pack_slots()),
                        r.ink(),
                    );
                    c.text(x + 24, y + 10, r.name(), KHAKI);
                }
                None => {
                    c.sprite(
                        a.tex(a.icon(super::super::assets::pack_art::ICONS[1])),
                        x + 1,
                        y + 1,
                    );
                    c.shade(x + 1, y + 1, 16, 16, 1);
                    c.text(x + 24, y + 1, "None on", KHAKI);
                    c.text(x + 24, y + 10, "(more room!)", KHAKI);
                }
            }
        }
    }

    /// The worn-gear column: pieces, or faint outlines of what goes where.
    pub fn draw_worn(
        &self,
        c: &mut Canvas,
        a: &Assets,
        l: &Layout,
        top: i32,
        cursor: Option<usize>,
    ) {
        for (i, slot) in SLOTS.iter().enumerate() {
            let (x, y) = worn_pos(l, top, i);
            let s = self.player.equip[i];
            self.draw_slot(c, a, x, y, s, cursor == Some(i));
            if s.is_none() {
                c.sprite(a.tex(a.icon(slot.ghost_icon())), x + 1, y + 1);
            }
        }
        let (x, y) = worn_pos(l, top, 0);
        c.frame(x - 3, y - 3, 24, 5 * CELL + 5, KHAKI);
    }

    /// Shows the tooltip for a stack beside a slot.
    pub(crate) fn tip_at(
        &self,
        c: &mut Canvas,
        a: &Assets,
        l: &Layout,
        sx: i32,
        sy: i32,
        s: &Stack,
    ) {
        let w = c.w();
        if l.px + l.pw + 150 < w {
            self.stack_tooltip(c, a, l.px + l.pw + 4, sy - 4, s);
        } else {
            self.stack_tooltip(c, a, sx + 22, sy - 4, s);
        }
    }

    /// Draws the open menu. `pointer` says the mouse is what's pointing (it moved or clicked
    /// last), rather than a controller or the keys.
    pub fn draw_menu(
        &self,
        c: &mut Canvas,
        a: &Assets,
        settings: &Settings,
        mouse: Vec2,
        pointer: bool,
    ) {
        let (w, h) = (c.w(), c.h());
        // Only a mouse that's pointing hovers over things; otherwise the cursor picks what
        // to show (a Deck's pointer sits parked wherever it was left, often over a slot).
        let mouse = if pointer { mouse } else { Vec2::splat(-1000.0) };
        match &self.menu {
            Menu::None => {}
            Menu::Summary => {
                let Some(s) = &self.last_summary else { return };
                c.shade(0, 0, w, h, 2);
                let title = format!("Day {}", s.day);
                let slept = if s.fainted {
                    ("You were found asleep in the Hollow...", CRIMSON)
                } else if s.passed_out {
                    ("You fell asleep on your feet.", CRIMSON)
                } else {
                    ("You slept soundly.", SHADOW)
                };
                // The rest of the night's news, one line each.
                let mut news: Vec<(String, u8)> = s.notes.clone();
                news.push((format!("{} crops grew overnight", s.grown), TEAL));
                if s.ready > 0 {
                    news.push((format!("{} are ready to harvest!", s.ready), GREEN));
                }
                if s.sprouted > 0 {
                    news.push((
                        format!("{} weeds and bushes sprang up on the farm", s.sprouted),
                        ROSEWOOD,
                    ));
                }
                news.push((
                    if s.rain {
                        "It's raining - the crops are watered."
                    } else {
                        "The sun is out."
                    }
                    .into(),
                    INDIGO,
                ));
                // What tonight's moon means below.
                let moon = super::sky::MoonPhase::of_day(s.day);
                news.push((
                    format!("{}: {}", moon.name(), moon.omen()),
                    if moon.full() { CRIMSON } else { PURPLE },
                ));
                // The box grows to fit its widest line (and wraps anything wider than the
                // screen allows).
                let label = "Shipped goods sold for ";
                let money_w = c.text_width(label) + 4 + money_width(c, s.earned);
                let widest = news
                    .iter()
                    .map(|(t, _)| c.text_width(t))
                    .chain([c.text_width(slept.0), money_w, c.big_width(&title, 2)])
                    .max()
                    .unwrap_or(0);
                let pw = (widest + 24).clamp(190, (w - 16).max(190));
                let mut rows: Vec<(String, u8)> = Vec::new();
                for (t, col) in std::iter::once((slept.0.to_string(), slept.1)).chain(news) {
                    for part in c.font.wrap(&t, pw - 16) {
                        rows.push((part.to_string(), col));
                    }
                }
                let ph = 34 + (rows.len() as i32 + 1) * 11 + 22;
                let l = center(w, h, pw, ph);
                c.panel(l.px, l.py, l.pw, l.ph, Style::Paper);
                c.text_big(
                    l.px + (l.pw - c.big_width(&title, 2)) / 2,
                    l.py + 8,
                    &title,
                    2,
                    CREAM,
                    RUST,
                );
                let mut y = l.py + 34;
                for (i, (t, col)) in rows.iter().enumerate() {
                    c.text_center(l.px + l.pw / 2, y, t, *col);
                    y += 11;
                    // Earnings, with coins, straight after how you slept.
                    if i == 0 {
                        let x0 = l.px + (l.pw - money_w) / 2;
                        let lw = c.text(x0, y, label, RUST);
                        draw_money(c, a, x0 + lw + 4, y, s.earned, RUST);
                        y += 11;
                    }
                }
                c.text_center(
                    l.px + l.pw / 2,
                    l.py + l.ph - 14,
                    &format!("Press {} to begin the day", self.key(Action::Confirm)),
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
                        format!(
                            "Sun shadows: {}",
                            if settings.shadows { "on" } else { "off" }
                        ),
                        format!(
                            "Ambient occlusion: {}",
                            if settings.ao { "on" } else { "off" }
                        ),
                        "Back".to_string(),
                    ]
                } else {
                    vec![
                        "Resume".into(),
                        "Quest journal".into(),
                        if self.show_map {
                            "Hide minimap".into()
                        } else {
                            "Show minimap".into()
                        },
                        "Settings".into(),
                        "Controls".into(),
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
                tab,
                cursor,
                recipe,
                scroll,
                cat,
            } => {
                let l = pouch_layout(w, h, true);
                c.panel(l.px, l.py, l.pw, l.ph, Style::Paper);
                let ti = match tab {
                    Tab::Bag => 0,
                    Tab::Stats => 1,
                    Tab::Craft => 2,
                };
                tabs(c, &l, &["Bag", "Stats", "Crafting"], ti, &[34, 38, 54]);
                draw_money(
                    c,
                    a,
                    l.px + l.pw - 10 - money_width(c, self.money),
                    l.py + 7,
                    self.money,
                    INK,
                );
                match tab {
                    Tab::Bag => self.draw_bag_tab(c, a, &l, *cursor, mouse),
                    Tab::Stats => self.draw_stats_tab(c, a, &l),
                    Tab::Craft => self.draw_craft_tab(c, a, &l, *recipe, *scroll, *cat, mouse),
                }
            }
            Menu::Chest { x, z, cursor } => {
                let n = self.player.pack_slots();
                let l = pouch_layout(w, h, n > 0);
                c.panel(l.px, l.py, l.pw, l.ph, Style::Paper);
                c.text(l.px + 10, l.py + 7, "Chest", RUST);
                let tip = if self.pad {
                    "A moves a stack"
                } else {
                    "Shift-click moves a stack"
                };
                c.text(l.px + 80, l.py + 7, tip, KHAKI);
                let cg = chest_grid(&l);
                if let Some(super::world::Obj::Chest { items }) = self.world().obj(*x, *z) {
                    let inv = Inventory {
                        slots: items.clone(),
                    };
                    self.draw_grid(
                        c,
                        a,
                        &cg,
                        &inv,
                        if *cursor < 30 { Some(*cursor) } else { None },
                    );
                    let bg = bag_grid(&l, 20 + 3 * CELL + 16);
                    let pouch = pouch_of(&l, &bg, n, false);
                    c.text(bg.x, bg.y - 11, "Bag", RUST);
                    let mine = cursor.checked_sub(30);
                    self.draw_grid(c, a, &bg, &self.player.inv, mine.filter(|&i| i < BAG));
                    self.draw_pouch(c, a, &pouch, mine.and_then(|i| i.checked_sub(BAG)), false);
                    if self.held.is_none() {
                        let at = |i: usize| {
                            if i < 30 {
                                (inv.slots.get(i).copied().flatten(), cg.slot_pos(i))
                            } else if i - 30 >= BAG {
                                let k = i - 30;
                                (
                                    self.player.inv.slots.get(k).copied().flatten(),
                                    pouch.slot_pos(k - BAG),
                                )
                            } else {
                                (self.player.inv.slots[i - 30], bg.slot_pos(i - 30))
                            }
                        };
                        let hover = cg
                            .hit(mouse)
                            .or_else(|| bg.hit(mouse).map(|i| i + 30))
                            .or_else(|| pouch.hit(mouse).map(|j| 30 + BAG + j))
                            .map_or_else(|| at(*cursor), at);
                        if let (Some(s), (sx, sy)) = hover {
                            let tip = if n > 0 {
                                Layout {
                                    pw: w_with_pouch(&l),
                                    ..l
                                }
                            } else {
                                l
                            };
                            self.tip_at(c, a, &tip, sx, sy, &s);
                        }
                    }
                }
            }
            Menu::Shop {
                at,
                tab,
                cursor,
                scroll,
            } => self.draw_shop(c, a, *at, *tab, *cursor, *scroll, mouse),
            Menu::Talk {
                who,
                text,
                shown,
                choices,
                sel,
                ..
            } => self.draw_talk(c, a, *who, text, *shown, choices, *sel),
            Menu::Board { guild, sel } => self.draw_board(c, a, *guild, *sel),
            Menu::Journal { tab, sel } => self.draw_journal(c, a, *tab, *sel),
            Menu::Spells { tab, sel } => self.draw_spellery(c, a, *tab, *sel, mouse),
            Menu::Cheer => self.draw_cheer(c, a),
            Menu::Recipe { recipe, t } => self.draw_recipe_card(c, a, *recipe, *t, mouse),
            Menu::Controls { deck } => self.draw_controls(c, *deck),
            Menu::Tank { x, z, cursor } => self.draw_tank(c, a, *x, *z, *cursor, mouse),
            Menu::Ship { cursor } => {
                let n = self.player.pack_slots();
                let l = pouch_layout(w, h, n > 0);
                c.panel(l.px, l.py, l.pw, l.ph, Style::Paper);
                c.text(l.px + 10, l.py + 7, "Shipping bin - sold overnight", RUST);
                c.panel(l.px + 8, l.py + 20, l.pw - 16, 34, Style::Inset);
                let total: u64 = self.shipping.iter().map(|s| s.value()).sum();
                for (i, s) in self.shipping.iter().rev().take(11).enumerate() {
                    let x = l.px + 10 + i as i32 * 19;
                    c.sprite(a.tex(a.stack_icon(s)), x, l.py + 22);
                    if s.n > 1 {
                        c.tiny(x + 16, l.py + 33, &s.n.to_string(), WHITE, INK);
                    }
                }
                let lw = c.text(l.px + 12, l.py + 42, "Today:", SHADOW);
                let mw = draw_money(c, a, l.px + 16 + lw, l.py + 42, total, INK);
                c.text(
                    l.px + 22 + lw + mw,
                    l.py + 42,
                    "(click to take back)",
                    SHADOW,
                );
                let g = bag_grid(&l, 60);
                let pouch = pouch_of(&l, &g, n, false);
                self.draw_grid(c, a, &g, &self.player.inv, Some(*cursor));
                self.draw_pouch(c, a, &pouch, cursor.checked_sub(BAG), false);
                let hover = g.hit(mouse).or_else(|| pouch.hit(mouse).map(|j| BAG + j));
                if let Some(i) = hover.or(Some(*cursor)) {
                    if let Some(s) = self.player.inv.slots.get(i).copied().flatten() {
                        let t = format!("{} x{} -", s.name(), s.n);
                        let tw = c.text(l.px + 10, g.y + g.h() + 8, &t, INK);
                        draw_money(c, a, l.px + 14 + tw, g.y + g.h() + 8, s.value(), RUST);
                    }
                }
            }
            Menu::Enchant {
                gear,
                scroll,
                socket,
                cursor,
                msg,
                glow,
            } => self.draw_enchant(c, a, *gear, *scroll, *socket, *cursor, msg, *glow, mouse),
        }
        // The stack being carried rides on the cursor (or follows the mouse).
        if let Some(hs) = self.held {
            let (x, y) = self.held_spot(w, h, mouse, pointer);
            c.sprite(a.tex(a.stack_icon(&hs)), x, y);
            if hs.n > 1 {
                c.tiny(x + 16, y + 11, &hs.n.to_string(), WHITE, INK);
            }
        }
    }

    /// Where to draw the stack being carried: on the mouse when that's what's pointing,
    /// otherwise lifted off the cursor's slot (or, with no slots in view, tucked into the
    /// panel's corner).
    pub fn held_spot(&self, w: i32, h: i32, mouse: Vec2, pointer: bool) -> (i32, i32) {
        if pointer {
            return (mouse.x as i32 - 4, mouse.y as i32 - 4);
        }
        let l = panel_layout(w, h);
        let slot = match self.menu {
            Menu::Inventory {
                tab: Tab::Bag,
                cursor,
                ..
            } => {
                let l = pouch_layout(w, h, true);
                let g = bag_grid(&l, 22);
                let pouch = pouch_of(&l, &g, self.player.pack_slots(), true);
                Some(if cursor >= PACK_AT {
                    pouch.pack_pos()
                } else if cursor >= WORN_AT {
                    worn_pos(&l, 22, cursor - WORN_AT)
                } else if cursor >= BAG {
                    pouch.slot_pos(cursor - BAG)
                } else {
                    g.slot_pos(cursor)
                })
            }
            Menu::Chest { cursor, .. } => {
                let l = pouch_layout(w, h, self.player.pack_slots() > 0);
                let g = bag_grid(&l, 20 + 3 * CELL + 16);
                let pouch = pouch_of(&l, &g, self.player.pack_slots(), false);
                Some(if cursor < 30 {
                    chest_grid(&l).slot_pos(cursor)
                } else if cursor - 30 >= BAG {
                    pouch.slot_pos(cursor - 30 - BAG)
                } else {
                    g.slot_pos(cursor - 30)
                })
            }
            _ => None,
        };
        match slot {
            Some((x, y)) => (x + 7, y - 7),
            None => (l.px + l.pw - 26, l.py + l.ph - 26),
        }
    }

    fn draw_bag_tab(&self, c: &mut Canvas, a: &Assets, l: &Layout, cursor: usize, mouse: Vec2) {
        let g = bag_grid(l, 22);
        let pouch = pouch_of(l, &g, self.player.pack_slots(), true);
        self.draw_grid(
            c,
            a,
            &g,
            &self.player.inv,
            if cursor < BAG { Some(cursor) } else { None },
        );
        self.draw_worn(
            c,
            a,
            l,
            22,
            (WORN_AT..PACK_AT)
                .contains(&cursor)
                .then(|| cursor - WORN_AT),
        );
        self.draw_pouch(
            c,
            a,
            &pouch,
            cursor.checked_sub(BAG).filter(|_| cursor < WORN_AT),
            cursor >= PACK_AT,
        );
        // Hotbar marker.
        let (hx, hy) = g.slot_pos(self.player.sel);
        c.rect(hx + 7, hy - 3, 4, 2, RUST);
        let p = &self.player;
        let y = g.y + g.h() + 20;
        let x = l.px + 10;
        c.text(x, y, &format!("Level {}", p.level), INK);
        let need = super::player::xp_needed(p.level);
        c.bar(
            x + 44,
            y + 1,
            64,
            7,
            p.xp as f32 / need as f32,
            AQUA,
            MINT,
            SHADOW,
        );
        c.text(x + 112, y, &format!("{}/{} xp", p.xp, need), SHADOW);
        let y = y + 11;
        c.text(x, y, &format!("♥ {}/{}", p.hp, p.max_hp()), CRIMSON);
        c.text(
            x + 62,
            y,
            &format!("⚡ {}/{}", p.energy.max(0.0) as i32, p.max_energy()),
            RUST,
        );
        c.text(
            x + 132,
            y,
            &format!("★ {}/{}", p.mana as i32, p.max_mana()),
            PURPLE,
        );
        let y = y + 11;
        c.sprite(a.tex(a.icon(stat_icon(Stat::Defense))), x, y);
        c.text(x + 10, y, &format!("{} Defense", p.defense()), INDIGO);
        c.sprite(a.tex(a.icon(stat_icon(Stat::Damage))), x + 80, y);
        c.text(x + 90, y, &format!("{} Damage", p.weapon_damage()), MAROON);
        let y = y + 12;
        c.text(
            x,
            y,
            if self.pad {
                "A: pick up or wear. X: split. L1/R1: stats"
            } else {
                "Right click: wear or split. Shift: quick wear."
            },
            KHAKI,
        );
        // What's under the mouse (or else the cursor) gets its tooltip, beside it (the pouch
        // takes the room to the right of the panel).
        if self.held.is_none() {
            let under = if let Some(i) = g.hit(mouse) {
                Some(i)
            } else if let Some(j) = pouch.hit(mouse) {
                Some(BAG + j)
            } else if let Some(i) = worn_hit(l, 22, mouse) {
                Some(WORN_AT + i)
            } else if pouch.on_pack(mouse) {
                Some(PACK_AT)
            } else {
                Some(cursor)
            };
            let tip = Layout {
                pw: w_with_pouch(l),
                ..*l
            };
            match under {
                Some(k) if k >= PACK_AT => {
                    if let Some(s) = self.player.pack {
                        let (sx, sy) = pouch.pack_pos();
                        self.tip_at(c, a, &tip, sx, sy, &s);
                    }
                }
                Some(k) if k >= WORN_AT => {
                    if let Some(s) = self.player.equip[k - WORN_AT] {
                        let (sx, sy) = worn_pos(l, 22, k - WORN_AT);
                        self.tip_at(c, a, &tip, sx, sy, &s);
                    }
                }
                Some(k) => {
                    if let Some(s) = self.player.inv.slots.get(k).copied().flatten() {
                        let (sx, sy) = if k >= BAG {
                            pouch.slot_pos(k - BAG)
                        } else {
                            g.slot_pos(k)
                        };
                        self.tip_at(c, a, &tip, sx, sy, &s);
                    }
                }
                None => {}
            }
        }
    }

    fn draw_stats_tab(&self, c: &mut Canvas, a: &Assets, l: &Layout) {
        let p = &self.player;
        let x = l.px + 12;
        let mut y = l.py + 23;
        let head = [
            (format!("Level {}", p.level), INK),
            (format!("♥ {} HP", p.max_hp()), CRIMSON),
            (format!("⚡ {} energy", p.max_energy()), RUST),
            (format!("★ {} mana", p.max_mana()), PURPLE),
        ];
        let mut hx = x;
        for (t, col) in &head {
            hx += c.text(hx, y, t, *col) + 10;
        }
        y += 12;
        c.rect(x, y - 2, l.pw - 24, 1, KHAKI);
        let derived = [
            (Stat::Defense, format!("{} Defense", p.defense())),
            (Stat::Damage, format!("{} Damage", p.weapon_damage())),
            (
                Stat::Crit,
                format!(
                    "{}% Crit, x{:.1}",
                    (p.crit_chance() * 100.0).round(),
                    p.crit_mult()
                ),
            ),
            (
                Stat::Block,
                format!("{}% Block", (p.block_chance() * 100.0).round()),
            ),
            (
                Stat::Dodge,
                format!("{}% Dodge", (p.dodge_chance() * 100.0).round()),
            ),
            (
                Stat::Swift,
                format!(
                    "{}% Move Speed",
                    (p.move_speed() / super::player::SPEED * 100.0).round()
                ),
            ),
        ];
        for (i, (st, t)) in derived.iter().enumerate() {
            let (cx, cy) = (x + (i as i32 % 2) * 108, y + (i as i32 / 2) * 11);
            c.sprite(a.tex(a.icon(stat_icon(*st))), cx, cy);
            c.text(cx + 10, cy, t, INK);
        }
        y += 36;
        c.rect(x, y - 2, l.pw - 24, 1, KHAKI);
        // Every bonus from gear and food.
        let shown: Vec<(Stat, i32)> = super::gear::ALL_STATS
            .iter()
            .map(|s| (*s, p.stat(*s)))
            .filter(|(_, v)| *v != 0)
            .collect();
        if shown.is_empty() {
            c.text(
                x,
                y + 2,
                "No bonuses yet. Wear gear or eat a good meal!",
                SHADOW,
            );
        }
        for (i, (st, v)) in shown.iter().enumerate().take(16) {
            let (cx, cy) = (x + (i as i32 % 2) * 108, y + (i as i32 / 2) * 10);
            c.sprite(a.tex(a.icon(stat_icon(*st))), cx, cy);
            c.text(cx + 10, cy, &st.line(*v), SHADOW);
        }
        // A little journal.
        let jy = l.py + l.ph - 40;
        c.rect(x, jy - 3, l.pw - 24, 1, KHAKI);
        let journal = [
            format!("Deepest floor {}", self.deepest),
            format!("Day {}", self.clock.day),
            format!("{} creatures calmed", self.stats.kills),
            format!("{} crops harvested", self.stats.harvested),
        ];
        for (i, t) in journal.iter().enumerate() {
            c.text(
                x + (i as i32 % 2) * 108,
                jy + (i as i32 / 2) * 10,
                t,
                ROSEWOOD,
            );
        }
        // Food buffs.
        let by = l.py + l.ph - 14;
        let mut bx = x;
        for b in &p.buffs {
            c.sprite(a.tex(a.icon(b.from.def().icon)), bx, by - 5);
            let t = super::tips::duration(b.left.ceil() as u16);
            bx += 17 + c.text(bx + 17, by, &t, TEAL) + 6;
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn draw_craft_tab(
        &self,
        c: &mut Canvas,
        a: &Assets,
        l: &Layout,
        recipe: usize,
        scroll: usize,
        cat: usize,
        mouse: Vec2,
    ) {
        // Category buttons.
        for (i, (x, cw)) in cat_rects(l).into_iter().enumerate() {
            let on = i == cat;
            let name = if i == 0 { "All" } else { CATS[i - 1].name() };
            c.rect(x, l.py + 21, cw, 11, if on { RUST } else { KHAKI });
            c.text_center(x + cw / 2, l.py + 23, name, if on { CREAM } else { SHADOW });
        }
        let list = recipes_in(cat);
        let rows = 7;
        for row in 0..rows {
            let i = scroll + row;
            let Some(r) = list.get(i) else { break };
            let y = l.py + 36 + row as i32 * 18;
            let known = self.known.contains(&r.out);
            let ok = known && r.can_craft(&self.player.inv);
            if i == recipe {
                c.rect(l.px + 6, y, 120, 17, GOLD);
            }
            let icon = a.tex(a.icon(r.out.def().icon));
            if !known {
                c.sprite_map(icon, l.px + 8, y + 1, |_| SHADOW);
                c.text(l.px + 27, y + 5, "???", SHADOW);
                continue;
            }
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
        let n = list.len().max(1) as i32;
        let track = rows as i32 * 18;
        c.rect(l.px + 127, l.py + 36, 2, track, KHAKI);
        if n > rows as i32 {
            let th = (track * rows as i32 / n).max(6);
            let ty = l.py + 36 + (track - th) * scroll as i32 / (n - rows as i32).max(1);
            c.rect(l.px + 127, ty, 2, th, RUST);
        }
        // How many are known.
        let known = RECIPES
            .iter()
            .filter(|r| self.known.contains(&r.out))
            .count();
        let count = format!("{known}/{} recipes known", RECIPES.len());
        c.text(l.px + 6, l.py + 36 + rows as i32 * 18 + 3, &count, SHADOW);
        // Details.
        let Some(r) = list.get(recipe) else { return };
        let dx = l.px + 133;
        let dw = l.pw - 139;
        if !self.known.contains(&r.out) {
            c.panel(dx, l.py + 36, dw, 20, Style::Inset);
            c.sprite_map(a.tex(a.icon(r.out.def().icon)), dx + 2, l.py + 38, |_| {
                SHADOW
            });
            c.text(dx + 20, l.py + 42, "Undiscovered", SHADOW);
            let mut y = l.py + 60;
            c.text(dx, y, "Needs:", SHADOW);
            y += 10;
            for _ in r.needs {
                c.panel(dx, y - 3, 16, 16, Style::Inset);
                c.text(dx + 5, y + 1, "?", SHADOW);
                y += 16;
            }
            let hint = "Carry one of each thing it needs and you'll work it out.";
            for (k, t) in c.font.wrap(hint, dw).iter().take(3).enumerate() {
                c.text(dx, y + k as i32 * 9, t, SHADOW);
            }
            let (bx, by, bw, bh) = craft_button(l);
            c.rect(bx, by, bw, bh, KHAKI);
            c.frame(bx, by, bw, bh, INK);
            c.text_center(bx + bw / 2, by + 3, "Undiscovered", ROSEWOOD);
            return;
        }
        c.panel(dx, l.py + 36, dw, 20, Style::Inset);
        c.sprite(a.tex(a.icon(r.out.def().icon)), dx + 2, l.py + 38);
        let title = if r.n > 1 {
            format!("{} x{}", r.out.def().name, r.n)
        } else {
            r.out.def().name.to_string()
        };
        let lines = c.font.wrap(&title, dw - 22);
        for (k, t) in lines.iter().take(2).enumerate() {
            c.text(dx + 20, l.py + 38 + k as i32 * 9, t, INK);
        }
        let mut y = l.py + 60;
        c.text(dx, y, "Needs:", SHADOW);
        y += 10;
        for (item, k) in r.needs {
            let have = self.player.inv.count(*item);
            c.sprite(a.tex(a.icon(item.def().icon)), dx, y - 3);
            let col = if have >= *k as u32 { TEAL } else { CRIMSON };
            c.text(dx + 18, y + 2, &format!("{have}/{k}"), col);
            y += 16;
        }
        let kitchen = r.cat == super::items::Cat::Kitchen;
        let stove = self.stove_near().is_some();
        let note = if kitchen && !stove {
            "Cook it at the stove in your house (or a waystone campfire).".to_string()
        } else if r.out.base().is_some() {
            format!(
                "Comes out at level {} with random stats.",
                crafted_level(self, r.out)
            )
        } else if r.out.scroll_group().is_some() {
            "A random enchantment for your depth.".to_string()
        } else {
            r.out.def().desc.to_string()
        };
        let desc = c.font.wrap(&note, dw);
        for (k, t) in desc.iter().take(3).enumerate() {
            c.text(dx, y + k as i32 * 9, t, SHADOW);
        }
        let has = r.can_craft(&self.player.inv);
        let ok = has && (!kitchen || stove);
        let (bx, by, bw, bh) = craft_button(l);
        c.rect(bx, by, bw, bh, if ok { GREEN } else { KHAKI });
        c.frame(bx, by, bw, bh, INK);
        let label = match (kitchen, stove, has) {
            (true, false, _) => "Needs a stove",
            (true, true, true) => "Cook",
            (_, _, true) => "Craft",
            _ => "Missing items",
        };
        let label = if has && (!kitchen || stove) {
            format!("{label} {}", self.prompt(Action::Confirm))
        } else {
            label.to_string()
        };
        c.text_center(
            bx + bw / 2,
            by + 3,
            &label,
            if ok { WHITE } else { ROSEWOOD },
        );
        // A gear recipe hovered (or else picked) shows what the base item is like.
        let row = (0..rows)
            .find(|&row| inside(mouse, l.px + 6, l.py + 36 + row as i32 * 18, 120, 17))
            .or_else(|| recipe.checked_sub(scroll).filter(|&row| row < rows));
        if let Some(row) = row {
            if let Some(r) = list.get(scroll + row) {
                if r.out.base().is_some() && self.known.contains(&r.out) {
                    let s = Stack::new(r.out, 1);
                    let y = l.py + 36 + row as i32 * 18;
                    self.stack_tooltip(c, a, l.px + l.pw + 4, y, &s);
                }
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn draw_shop(
        &self,
        c: &mut Canvas,
        a: &Assets,
        at: Option<Place>,
        tab: ShopTab,
        cursor: usize,
        scroll: usize,
        mouse: Vec2,
    ) {
        let (w, h) = (c.w(), c.h());
        let n = self.player.pack_slots();
        let l = pouch_layout(w, h, n > 0);
        c.panel(l.px, l.py, l.pw, l.ph, Style::Paper);
        let all = shops::tabs(at);
        let ti = all.iter().position(|t| *t == tab).unwrap_or(0);
        let names: Vec<&str> = all.iter().map(|t| shops::tab_name(at, *t)).collect();
        tabs(c, &l, &names, ti, &shop_tab_widths(at));
        if let Some(p) = at {
            let name = p.def().name;
            let x = l.px + l.pw - 10 - c.text_width(name);
            c.text(x, l.py + 7, name, ROSEWOOD);
        }
        // Your purse, in the corner at the bottom.
        c.panel(
            l.px + l.pw - 16 - money_width(c, self.money),
            l.py + l.ph - 17,
            money_width(c, self.money) + 10,
            13,
            Style::Inset,
        );
        draw_money(
            c,
            a,
            l.px + l.pw - 11 - money_width(c, self.money),
            l.py + l.ph - 14,
            self.money,
            INK,
        );
        if tab != ShopTab::Sell {
            let rows = shops::rows(self, at, tab);
            let mut tip = None;
            for row in 0..7 {
                let i = scroll + row;
                let Some(&(s, price, sold)) = rows.get(i) else {
                    break;
                };
                let y = l.py + 22 + row as i32 * 18;
                if i == cursor {
                    c.rect(l.px + 6, y, l.pw - 12, 17, GOLD);
                }
                self.draw_slot(c, a, l.px + 7, y, Some(s), false);
                let name_col = match s.rarity() {
                    Some(r) if r > Rarity::Common => r.ink(),
                    _ => INK,
                };
                let name = match (sold, s.item) {
                    (true, Item::Bomb) => format!("{} (sold out - more tomorrow)", s.name()),
                    (true, _) => format!("{} (sold)", s.name()),
                    (false, Item::Bomb) => format!("{} ({} left today)", s.name(), self.bomb_stock),
                    _ => s.name(),
                };
                c.text(l.px + 28, y + 5, &name, if sold { KHAKI } else { name_col });
                if let Some(g) = s.gear.filter(|_| tab == ShopTab::Specials) {
                    c.tiny(
                        l.px + 26 + c.text_width(&name) + 18,
                        y + 7,
                        &g.level.to_string(),
                        CREAM,
                        SHADOW,
                    );
                }
                let afford = self.money >= price;
                let mw = money_width(c, price);
                draw_money(
                    c,
                    a,
                    l.px + l.pw - 12 - mw,
                    y + 5,
                    price,
                    if afford && !sold { RUST } else { ROSEWOOD },
                );
                if inside(mouse, l.px + 6, y, l.pw - 12, 17)
                    || (i == cursor && !inside(mouse, l.px, l.py, l.pw, l.ph))
                {
                    tip = Some((s, y));
                }
            }
            let talk = shops::patter(at, tab);
            c.text(l.px + 10, l.py + l.ph - 14, talk, SHADOW);
            if let Some((s, y)) = tip {
                if tab == ShopTab::Specials || s.item.def().kind != Kind::Material {
                    self.stack_tooltip(c, a, l.px + l.pw + 4, y - 4, &s);
                }
            }
        } else {
            c.text(
                l.px + 10,
                l.py + 24,
                if self.pad {
                    "A sells a stack, X sells just one."
                } else {
                    "Click to sell a stack, right click sells one."
                },
                SHADOW,
            );
            let g = bag_grid(&l, 40);
            let pouch = pouch_of(&l, &g, n, false);
            self.draw_grid(c, a, &g, &self.player.inv, Some(cursor));
            self.draw_pouch(c, a, &pouch, cursor.checked_sub(BAG), false);
            let hover = g.hit(mouse).or_else(|| pouch.hit(mouse).map(|j| BAG + j));
            if let Some(i) = hover.or(Some(cursor)) {
                if let Some(s) = self.player.inv.slots.get(i).copied().flatten() {
                    let y = g.y + g.h() + 8;
                    if can_sell(&s) {
                        let rate = self.sell_rate(at, &s);
                        let t = format!("{} x{} -", s.name(), s.n);
                        let tw = c.text(l.px + 10, y, &t, INK);
                        let mw = draw_money(c, a, l.px + 14 + tw, y, s.value() * rate / 100, RUST);
                        if rate > 100 {
                            c.text(l.px + 18 + tw + mw, y, &format!("+{}%!", rate - 100), GREEN);
                        }
                    } else {
                        c.text(l.px + 10, y, &format!("{} - keep this one!", s.name()), INK);
                    }
                    let (sx, sy) = if i >= BAG {
                        pouch.slot_pos(i - BAG)
                    } else {
                        g.slot_pos(i)
                    };
                    let tip = if n > 0 {
                        Layout {
                            pw: w_with_pouch(&l),
                            ..l
                        }
                    } else {
                        l
                    };
                    self.tip_at(c, a, &tip, sx, sy, &s);
                }
            }
        }
    }
}

/// Tab widths for a shop's tabs.
pub fn shop_tab_widths(at: Option<Place>) -> Vec<i32> {
    shops::tabs(at)
        .iter()
        .map(|t| match shops::tab_name(at, *t) {
            "Specials" => 46,
            "Supplies" => 46,
            "Seeds" | "Goods" | "Gems" | "Decor" | "Magic" | "Treats" | "Today" => 36,
            "Potions" => 42,
            "Menu" => 32,
            _ => 30,
        })
        .collect()
}

/// Tab positions: (x, width).
pub fn tab_rects(l: &Layout, widths: &[i32]) -> Vec<(i32, i32)> {
    let mut x = l.px + 8;
    widths
        .iter()
        .map(|w| {
            let r = (x, *w);
            x += w + 4;
            r
        })
        .collect()
}

fn cat_rects(l: &Layout) -> Vec<(i32, i32)> {
    let widths = [15, 28, 25, 26, 37, 27, 37];
    let mut x = l.px + 8;
    widths
        .iter()
        .map(|w| {
            let r = (x, *w);
            x += w + 2;
            r
        })
        .collect()
}

fn craft_button(l: &Layout) -> (i32, i32, i32, i32) {
    (l.px + 133, l.py + l.ph - 22, l.pw - 139, 14)
}

pub fn chest_grid(l: &Layout) -> Grid {
    Grid {
        x: l.px + 34,
        y: l.py + 20,
        cols: 10,
        rows: 3,
        gap: 0,
    }
}

pub fn tabs(c: &mut Canvas, l: &Layout, names: &[&str], sel: usize, widths: &[i32]) {
    for (i, (x, w)) in tab_rects(l, widths).into_iter().enumerate() {
        if i == sel {
            c.rect(x, l.py + 5, w, 12, GOLD);
            c.frame(x, l.py + 5, w, 12, RUST);
        }
        c.text_center(
            x + w / 2,
            l.py + 7,
            names[i],
            if i == sel { INK } else { ROSEWOOD },
        );
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
