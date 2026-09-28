//! Hazel's counter at the Starfall Spellery: learning spells, and the attuning circle where
//! you choose the two you carry. It is the only place those two can be changed.

use glam::Vec2;

use super::Io;
use super::menus::{Menu, center, inside, tabs};
use super::play::Play;
use super::spells::{MAX_LEVEL, SPELLS, Spell, xp_to_next};
use super::tips::{draw_money, money_width};
use crate::assets::Assets;
use crate::audio::Sfx;
use crate::input::{Action, Button, KeyCode};
use crate::palette::*;
use crate::ui::{Canvas, Style};

const PW: i32 = 300;
const PH: i32 = 196;
const ROW: i32 = 19;
const LIST_W: i32 = 150;
const TAB_W: [i32; 2] = [36, 40];

fn layout(io_w: i32, io_h: i32) -> super::menus::Layout {
    center(io_w, io_h, PW, PH)
}

/// Where the two slot boxes sit on the attuning tab.
fn slot_box(l: &super::menus::Layout, slot: usize) -> (i32, i32) {
    (l.px + 14 + slot as i32 * 70, l.py + 24)
}

impl Play {
    /// Spells known, in the order Hazel lists them.
    pub(crate) fn known_spells(&self) -> Vec<Spell> {
        SPELLS
            .iter()
            .copied()
            .filter(|s| self.spells.knows(*s))
            .collect()
    }

    fn spell_rows(&self, tab: usize) -> Vec<Spell> {
        if tab == 0 {
            SPELLS.to_vec()
        } else {
            self.known_spells()
        }
    }

    /// Buys a lesson from Hazel.
    pub fn learn_spell(&mut self, s: Spell, io: &mut Io) -> bool {
        let d = s.def();
        if self.spells.knows(s) {
            self.toast(format!("You already know {}.", d.name), None, 0);
            io.audio.play(Sfx::Denied);
            return false;
        }
        if self.deepest < d.depth {
            self.toast(
                format!("Hazel: \"Come back once you've seen floor {}.\"", d.depth),
                None,
                0,
            );
            io.audio.play(Sfx::Denied);
            return false;
        }
        if self.money < d.price {
            self.toast("Not enough money.", None, 0);
            io.audio.play(Sfx::Denied);
            return false;
        }
        self.money -= d.price;
        self.spells.learn(s);
        io.audio.play(Sfx::SpellUp);
        io.audio.play_at(Sfx::Enchant, 0.5, 1.2);
        let ready = self.spells.slots.contains(&Some(s));
        let text = if ready {
            format!("Learned {}! It's ready to cast.", d.name)
        } else {
            format!("Learned {}! Ready it at the attuning circle.", d.name)
        };
        self.toast_colored(text, None, 0, d.colors[1]);
        self.fx.motes(self.player.world_pos(), 16, &d.colors, 0.4);
        true
    }

    fn attune_spell(&mut self, slot: usize, s: Spell, io: &mut Io) {
        if self.spells.attune(slot, s) {
            io.audio.play_at(Sfx::Enchant, 0.45, 1.5);
            let key = ["Q", "R"][slot];
            self.toast_colored(
                format!("{} readied on {key}", s.def().name),
                None,
                0,
                s.def().colors[1],
            );
        } else {
            io.audio.play(Sfx::Denied);
        }
    }

    pub fn update_spellery(&mut self, io: &mut Io, mut tab: usize, mut sel: usize) -> Menu {
        let input = io.input;
        let (w, h) = (io.view.0 as i32, io.view.1 as i32);
        let l = layout(w, h);
        let lclick = input.button_pressed(Button::Left);
        if input.pressed(Action::Cancel) || input.pressed(Action::Inventory) {
            io.audio.play(Sfx::UiBack);
            return Menu::None;
        }
        // Tabs: arrows or a click.
        let before = tab;
        if input.pressed_repeat(Action::Right) || input.pressed_repeat(Action::Left) {
            tab = 1 - tab;
        }
        for (i, (x, tw)) in super::menus::tab_rects(&l, &TAB_W).into_iter().enumerate() {
            if lclick && inside(input.mouse, x, l.py + 5, tw, 12) {
                tab = i;
            }
        }
        if tab != before {
            sel = 0;
            io.audio.play_at(Sfx::UiMove, 0.6, 1.0);
        }
        let rows = self.spell_rows(tab);
        let top = if tab == 0 { l.py + 22 } else { l.py + 66 };
        let n = rows.len().max(1);
        if input.pressed_repeat(Action::Down) {
            sel = (sel + 1) % n;
            io.audio.play_at(Sfx::UiMove, 0.5, 1.0);
        }
        if input.pressed_repeat(Action::Up) {
            sel = (sel + n - 1) % n;
            io.audio.play_at(Sfx::UiMove, 0.5, 1.0);
        }
        // A click picks a row; a second click on it (or E) acts on it.
        let was = sel;
        let mut again = false;
        for i in 0..rows.len() {
            let y = top + i as i32 * ROW;
            if inside(input.mouse, l.px + 6, y, LIST_W, ROW - 1) {
                if input.mouse_moved {
                    sel = i;
                }
                if lclick {
                    again = i == was;
                    sel = i;
                }
            }
        }
        sel = sel.min(n - 1);
        let chosen = rows.get(sel).copied();
        if tab == 0 {
            if input.pressed(Action::Confirm) || again {
                if let Some(s) = chosen {
                    self.learn_spell(s, io);
                }
            }
        } else if let Some(s) = chosen {
            // The attuning circle: Q, R (or 1, 2) or a click on a slot readies it there.
            let mut slot = None;
            if input.pressed(Action::Spell1) || input.key_pressed(KeyCode::Digit1) {
                slot = Some(0);
            }
            if input.pressed(Action::Spell2) || input.key_pressed(KeyCode::Digit2) {
                slot = Some(1);
            }
            for k in 0..2 {
                let (bx, by) = slot_box(&l, k);
                if lclick && inside(input.mouse, bx, by, 60, 34) {
                    slot = Some(k);
                }
            }
            if input.pressed(Action::Confirm) {
                slot = Some(if self.spells.slots[0] == Some(s) {
                    1
                } else {
                    0
                });
            }
            if let Some(k) = slot {
                self.attune_spell(k, s, io);
            }
        }
        Menu::Spells { tab, sel }
    }

    pub fn draw_spellery(&self, c: &mut Canvas, a: &Assets, tab: usize, sel: usize, mouse: Vec2) {
        let (w, h) = (c.w(), c.h());
        let l = layout(w, h);
        c.panel(l.px, l.py, l.pw, l.ph, Style::Paper);
        tabs(c, &l, &["Learn", "Attune"], tab, &TAB_W);
        let title = "Starfall Spellery";
        c.text(
            l.px + l.pw - 10 - c.text_width(title),
            l.py + 7,
            title,
            PURPLE,
        );
        let rows = self.spell_rows(tab);
        let top = if tab == 0 { l.py + 22 } else { l.py + 66 };
        if tab == 1 {
            // The two slots, Q and R.
            for k in 0..2 {
                let (bx, by) = slot_box(&l, k);
                c.panel(bx, by, 60, 34, Style::Inset);
                c.text(bx + 4, by + 3, ["Q", "R"][k], RUST);
                match self.spells.slots[k] {
                    Some(s) => {
                        c.sprite(a.tex(a.icon(s.def().icon)), bx + 22, by + 3);
                        let lv = self.spells.get(s).map_or(1, |k| k.level);
                        c.text_center(bx + 30, by + 22, &format!("Lv {lv}"), INK);
                    }
                    None => {
                        c.text_center(bx + 30, by + 13, "empty", KHAKI);
                    }
                }
            }
            c.text(l.px + 156, l.py + 28, "The attuning circle", PURPLE);
            c.paragraph(
                l.px + 156,
                l.py + 38,
                l.pw - 164,
                &format!(
                    "Pick a spell, then press {} or {} to carry it.",
                    self.key(Action::Spell1),
                    self.key(Action::Spell2)
                ),
                SHADOW,
            );
            if rows.is_empty() {
                c.text(l.px + 12, top + 4, "You don't know any spells yet.", SHADOW);
            }
        }
        for (i, s) in rows.iter().enumerate() {
            let d = s.def();
            let y = top + i as i32 * ROW;
            if i == sel {
                c.rect(l.px + 6, y, LIST_W, ROW - 1, GOLD);
            }
            let known = self.spells.get(*s).copied();
            let open = self.deepest >= d.depth;
            let icon = a.tex(a.icon(d.icon));
            if known.is_some() || open {
                c.sprite(icon, l.px + 8, y + 1);
            } else {
                let dim = c.darken[1];
                c.sprite_map(icon, l.px + 8, y + 1, |col| dim[col as usize]);
            }
            let name_col = if known.is_some() {
                d.ink
            } else if open {
                INK
            } else {
                ROSEWOOD
            };
            c.text(l.px + 27, y + 5, d.name, name_col);
            let right = l.px + 6 + LIST_W - 4;
            match known {
                Some(k) => {
                    let t = format!("Lv {}", k.level);
                    c.text(right - c.text_width(&t), y + 5, &t, TEAL);
                    if let Some(slot) = self.spells.slots.iter().position(|x| *x == Some(*s)) {
                        let key = ["Q", "R"][slot];
                        c.text(right - c.text_width(&t) - 12, y + 5, key, RUST);
                    }
                }
                None if !open => {
                    let t = format!("floor {}", d.depth);
                    c.text(right - c.text_width(&t), y + 5, &t, ROSEWOOD);
                }
                None => {
                    let mw = money_width(c, d.price);
                    let col = if self.money >= d.price {
                        RUST
                    } else {
                        ROSEWOOD
                    };
                    draw_money(c, a, right - mw, y + 5, d.price, col);
                }
            }
        }
        // Details of the chosen spell.
        let Some(s) = rows.get(sel.min(rows.len().saturating_sub(1))).copied() else {
            return;
        };
        let d = s.def();
        let dx = l.px + LIST_W + 12;
        let dw = l.pw - LIST_W - 18;
        let dy = if tab == 0 { l.py + 22 } else { l.py + 66 };
        c.panel(dx, dy, dw, 20, Style::Inset);
        c.sprite(a.tex(a.icon(d.icon)), dx + 2, dy + 2);
        c.text(dx + 21, dy + 6, d.name, d.ink);
        let mut y = dy + 24;
        y += c.paragraph(dx, y, dw, d.desc, INK) + 2;
        y += c.paragraph(dx, y, dw, d.grows, SHADOW) + 3;
        let known = self.spells.get(s).copied();
        let k = known.unwrap_or(super::spells::Known::new(s));
        let cost = self.player.mana_cost(k.cost());
        c.text(
            dx,
            y,
            &format!("Mana {:.0}   Wait {:.1}s", cost, d.cooldown),
            INDIGO,
        );
        y += 11;
        if let Some(k) = known {
            if k.level >= MAX_LEVEL {
                c.text(dx, y, "Mastered!", GOLD);
            } else {
                // Practice towards the next level.
                let need = xp_to_next(k.level);
                let bw = dw - 4;
                c.rect(dx, y + 1, bw, 5, KHAKI);
                let fill = (bw as u32 * k.xp / need.max(1)) as i32;
                c.rect(dx, y + 1, fill, 5, d.colors[1]);
                c.frame(dx - 1, y, bw + 2, 7, SHADOW);
                c.text(
                    dx,
                    y + 9,
                    &format!("Lv {} - {}/{} to next", k.level, k.xp, need),
                    SHADOW,
                );
            }
        } else if tab == 0 {
            let hint = if self.deepest < d.depth {
                format!("Reach floor {} first.", d.depth)
            } else if self.money < d.price {
                "Save up a little more.".to_string()
            } else {
                format!("Press {} to learn it.", self.key(Action::Confirm))
            };
            c.text(dx, y, &hint, RUST);
        }
        // Your purse.
        let mw = money_width(c, self.money);
        c.panel(
            l.px + l.pw - 16 - mw,
            l.py + l.ph - 17,
            mw + 10,
            13,
            Style::Inset,
        );
        draw_money(
            c,
            a,
            l.px + l.pw - 11 - mw,
            l.py + l.ph - 14,
            self.money,
            INK,
        );
        let _ = mouse;
    }
}
