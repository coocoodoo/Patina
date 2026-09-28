//! The controls page from the pause menu: the Steam Deck's layout, drawn as the Deck with
//! every button labelled, and the keyboard's.

use super::Io;
use super::menus::{Menu, center};
use super::play::Play;
use crate::audio::Sfx;
use crate::input::Action;
use crate::palette::*;
use crate::ui::{Canvas, Style};

/// What each of the Deck's controls does, left side then right.
const DECK_LEFT: [(&str, &str); 7] = [
    ("L2", "Cast spell 1"),
    ("L1", "Hotbar left"),
    ("L", "Walk"),
    ("L3", "Map (click stick)"),
    ("+", "Walk; menus"),
    ("View", "Crafting book"),
    ("Menu", "Pause"),
];
const DECK_RIGHT: [(&str, &str); 8] = [
    ("R2", "Cast spell 2"),
    ("R1", "Hotbar right"),
    ("R", "Aim"),
    ("R3", "Quest journal"),
    ("A", "Talk, open, place; OK"),
    ("B", "Roll; back"),
    ("X", "Use tool or weapon"),
    ("Y", "Bag"),
];

const KEYS: [(&str, &str); 14] = [
    ("WASD / arrows", "Walk; menus"),
    ("Mouse", "Aim and click"),
    ("J / Z / click", "Use tool or weapon"),
    ("E / right click", "Talk, open, place"),
    ("Space / Shift", "Roll"),
    ("Tab / I", "Bag"),
    ("C", "Crafting book"),
    ("L", "Quest journal"),
    ("M", "Map"),
    ("Q / R", "Cast spells"),
    ("1-0 / wheel / [ ]", "Hotbar"),
    ("T (at home)", "Turn furniture"),
    ("Esc", "Pause"),
    ("F11 / F12", "Fullscreen / screenshot"),
];

/// A button's name in a little dark cap, as it's printed on the Deck.
fn cap(c: &mut Canvas, x: i32, y: i32, name: &str) -> i32 {
    let round = matches!(name, "A" | "B" | "X" | "Y");
    let w = if round { 11 } else { c.text_width(name) + 7 };
    c.rect(x + 1, y, w - 2, 11, SHADOW);
    c.rect(x, y + 1, w, 9, SHADOW);
    if round {
        c.px(x + 1, y + 1, SHADOW);
    }
    c.rect(x + 1, y + 1, w - 2, 1, SLATE);
    c.text(x + (w - c.text_width(name)) / 2, y + 2, name, WHITE);
    w
}

/// A stick seen from above.
fn stick(c: &mut Canvas, cx: i32, cy: i32) {
    for dy in -4i32..=4 {
        for dx in -4i32..=4 {
            let d = dx * dx + dy * dy;
            if d <= 16 {
                c.px(cx + dx, cy + dy, if d <= 5 { SLATE } else { SHADOW });
            }
        }
    }
}

/// The Deck itself, with its controls where they sit on it.
fn draw_deck(c: &mut Canvas, x: i32, y: i32) {
    let (w, h) = (170, 58);
    // Body, with rounded grips.
    c.rect(x + 6, y, w - 12, h, INK);
    c.rect(x, y + 6, w, h - 12, INK);
    c.rect(x + 2, y + 2, w - 4, h - 4, INK);
    c.rect(x + 6, y, w - 12, 1, SHADOW);
    // The screen.
    c.rect(x + 45, y + 8, 80, 42, SHADOW);
    c.rect(x + 47, y + 10, 76, 38, DEEP_TEAL);
    c.rect(x + 49, y + 40, 20, 5, GREEN);
    c.rect(x + 86, y + 22, 6, 8, GOLD);
    // Left: the D-pad, the stick, View.
    let (dx, dy) = (x + 16, y + 30);
    c.rect(dx - 1, dy - 5, 3, 11, SLATE);
    c.rect(dx - 5, dy - 1, 11, 3, SLATE);
    stick(c, x + 30, y + 16);
    c.rect(x + 36, y + 6, 5, 2, SLATE);
    // Right: the face buttons, the stick, Menu.
    let (fx, fy) = (x + w - 18, y + 30);
    for (ox, oy) in [(0, -5), (-5, 0), (5, 0), (0, 5)] {
        c.rect(fx + ox - 1, fy + oy - 1, 3, 3, SLATE);
    }
    stick(c, x + w - 32, y + 16);
    c.rect(x + w - 42, y + 6, 5, 2, SLATE);
    // Shoulders.
    c.rect(x + 8, y - 2, 22, 2, SHADOW);
    c.rect(x + w - 30, y - 2, 22, 2, SHADOW);
}

impl Play {
    pub fn update_controls(&mut self, io: &mut Io, deck: bool) -> Menu {
        let input = io.input;
        let flip = input.pressed(Action::Left)
            || input.pressed(Action::Right)
            || input.pressed(Action::NextSlot)
            || input.pressed(Action::PrevSlot);
        if flip {
            io.audio.play_at(Sfx::UiMove, 0.6, 1.0);
            return Menu::Controls { deck: !deck };
        }
        if input.pressed(Action::Cancel)
            || input.pressed(Action::Confirm)
            || input.pressed(Action::Menu)
            || input.button_pressed(crate::input::Button::Left)
            || input.button_pressed(crate::input::Button::Right)
        {
            io.audio.play(Sfx::UiBack);
            return Menu::Pause {
                sel: 2,
                settings: false,
            };
        }
        Menu::Controls { deck }
    }

    pub fn draw_controls(&self, c: &mut Canvas, deck: bool) {
        let (w, h) = (c.w(), c.h());
        let l = center(w, h, 300.min(w - 8), 214.min(h - 8));
        c.panel(l.px, l.py, l.pw, l.ph, Style::Paper);
        let title = if deck {
            "Controls - Steam Deck"
        } else {
            "Controls - keyboard and mouse"
        };
        c.text_center(l.px + l.pw / 2, l.py + 6, title, RUST);
        let flip = format!(
            "{} / {}: {}",
            self.key(Action::PrevSlot),
            self.key(Action::NextSlot),
            if deck { "keyboard" } else { "Steam Deck" }
        );
        c.text(
            l.px + l.pw - c.text_width(&flip) - 8,
            l.py + l.ph - 13,
            &flip,
            SHADOW,
        );
        c.text(
            l.px + 8,
            l.py + l.ph - 13,
            &format!("{} back", self.key(Action::Cancel)),
            SHADOW,
        );
        if deck {
            let dx = l.px + (l.pw - 170) / 2;
            draw_deck(c, dx, l.py + 20);
            // Labels either side, joined to where they are on the Deck.
            let top = l.py + 84;
            for (i, (b, what)) in DECK_LEFT.iter().enumerate() {
                let y = top + i as i32 * 13;
                let name = match *b {
                    "L" => "L stick",
                    "+" => "D-pad",
                    other => other,
                };
                let cw = cap(c, l.px + 8, y, name);
                c.text(l.px + 12 + cw, y + 2, what, INK);
            }
            let mid = l.px + l.pw / 2 + 4;
            for (i, (b, what)) in DECK_RIGHT.iter().enumerate() {
                let y = top + i as i32 * 13;
                let name = if *b == "R" { "R stick" } else { b };
                let cw = cap(c, mid, y, name);
                c.text(mid + 4 + cw, y + 2, what, INK);
            }
        } else {
            let top = l.py + 22;
            for (i, (k, what)) in KEYS.iter().enumerate() {
                let y = top + i as i32 * 12;
                c.text(l.px + 10, y, k, RUST);
                c.text(l.px + 118, y, what, INK);
            }
        }
    }
}
