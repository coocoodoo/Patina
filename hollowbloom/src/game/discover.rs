//! Learning recipes: nobody hands you a cookbook. Carry one of each of a recipe's
//! ingredients and you work out how it's made, and a card stops everything to show it off:
//! its picture, its name, and what it's good for.

use glam::Vec2;

use super::Io;
use super::items::{Cat, RECIPES, Recipe, Stack};
use super::menus::{Layout, Menu, center, inside};
use super::play::Play;
use super::tips::{Line, line, tip_box};
use crate::assets::Assets;
use crate::audio::Sfx;
use crate::input::{Action, Button};
use crate::palette::*;
use crate::ui::{Canvas, Style};

const CARD_W: i32 = 272;
const CARD_H: i32 = 192;

fn card(w: i32, h: i32) -> Layout {
    center(w, h, CARD_W, CARD_H)
}

/// The OK button along the bottom of the card.
fn ok_button(l: &Layout) -> (i32, i32, i32, i32) {
    (l.px + l.pw / 2 - 32, l.py + l.ph - 21, 64, 14)
}

/// Where a recipe gets made, for the card's subtitle.
fn where_made(r: &Recipe) -> &'static str {
    match r.cat {
        Cat::Kitchen => "A dish - cook it at your stove",
        Cat::Potions => "A potion - brew it in the crafting book",
        Cat::Magic => "Magic - make it in the crafting book",
        Cat::Home => "For your home - make it in the crafting book",
        Cat::Tools => "A tool - make it in the crafting book",
        Cat::Gear => "Gear - make it in the crafting book",
    }
}

impl Play {
    /// The card waits for OK (after a moment, so a button already held down can't wave it
    /// away unseen); the game carries on once it's gone.
    pub fn update_recipe_card(&mut self, io: &mut Io, recipe: usize, t: f32) -> Menu {
        let t = t + io.dt;
        let input = io.input;
        let (w, h) = (io.view.0 as i32, io.view.1 as i32);
        let (bx, by, bw, bh) = ok_button(&card(w, h));
        let click = input.button_pressed(Button::Left) && inside(input.mouse, bx, by, bw, bh);
        let ok = input.pressed(Action::Confirm) || input.pressed(Action::Cancel) || click;
        if t > 0.4 && ok {
            io.audio.play(Sfx::UiSelect);
            return Menu::None;
        }
        Menu::Recipe { recipe, t }
    }

    /// What the card says a recipe's dish (or potion, or gear) is good for.
    fn card_lines(&self, r: &Recipe) -> Vec<Line> {
        let s = Stack::new(r.out, r.n);
        let mut lines = self.stack_lines(&s);
        if let Some(b) = r.out.base() {
            // A plain copy says nothing of what a freshly made one rolls.
            lines[0] = line(
                format!(
                    "{} - made at level {}",
                    b.class.name(),
                    super::menus::crafted_level(self, r.out)
                ),
                KHAKI,
            );
            // Sockets come empty on everything: no need to say so here.
            lines.retain(|l| l.icon != Some("socket"));
            lines.push(line("Rolls random stats as it's made", SKY));
        } else if let Some(g) = r.out.scroll_group() {
            lines = vec![
                line(format!("Scroll for {}", g.name().to_lowercase()), KHAKI),
                line("A random enchantment for your depth", SKY),
                line("Bind it at an enchanting table", SKY),
            ];
        }
        lines
    }

    pub fn draw_recipe_card(&self, c: &mut Canvas, a: &Assets, recipe: usize, t: f32, mouse: Vec2) {
        let Some(r) = RECIPES.get(recipe) else { return };
        let (w, h) = (c.w(), c.h());
        c.fade(0.3, INK);
        let l = card(w, h);
        let pop = (t * 6.0).min(1.0);
        let dy = ((1.0 - pop) * 16.0) as i32;
        let (x, y) = (l.px, l.py + dy);
        c.panel(x, y, l.pw, l.ph, Style::Paper);
        // Title and subtitle.
        let title = "New recipe!";
        let tw = c.big_width(title, 2);
        let col = if (t * 4.0).fract() < 0.5 { GOLD } else { CREAM };
        c.text_big(x + (l.pw - tw) / 2, y + 6, title, 2, col, RUST);
        c.text_center(x + l.pw / 2, y + 27, where_made(r), ROSEWOOD);

        // The picture, three times over, with a sunburst turning behind it.
        let (px, py) = (x + 10, y + 40);
        c.panel(px, py, 60, 60, Style::Inset);
        let (cx, cy) = (px + 30, py + 30);
        for k in 0..12 {
            let ang = t * 0.8 + k as f32 * std::f32::consts::TAU / 12.0;
            let col = if k % 2 == 0 { GOLD } else { CREAM };
            for d in 17..28 {
                let sx = cx + (ang.cos() * d as f32) as i32;
                let sy = cy + (ang.sin() * d as f32) as i32;
                if (px + 2..px + 58).contains(&sx) && (py + 2..py + 58).contains(&sy) {
                    c.px(sx, sy, col);
                }
            }
        }
        c.sprite_scaled(a.tex(a.icon(r.out.def().icon)), px + 6, py + 6, 3);

        // Its name and what it does, on a dark slate so the colours read.
        let (ix, iw) = (x + 76, l.pw - 86);
        tip_box(c, ix, py, iw, 60, KHAKI);
        let name = if r.n > 1 {
            format!("{} x{}", r.out.def().name, r.n)
        } else {
            r.out.def().name.to_string()
        };
        c.text_shadow(ix + 5, py + 4, &name, WHITE, SHADOW);
        let mut ly = py + 16;
        for ln in self.card_lines(r).iter().take(4) {
            let mut tx = ix + 5;
            if let Some(icon) = ln.icon {
                c.sprite(a.tex(a.icon(icon)), tx, ly);
                tx += 10;
            }
            let text = c.font.wrap(&ln.text, ix + iw - 4 - tx);
            if let Some(first) = text.first() {
                c.text(tx, ly, first, ln.color);
            }
            ly += 10;
        }

        // A word about it.
        let desc = c.font.wrap(r.out.def().desc, l.pw - 20);
        for (k, t) in desc.iter().take(2).enumerate() {
            c.text(x + 10, y + 104 + k as i32 * 9, t, SHADOW);
        }

        // What goes into it (how many of each it really takes).
        let mut nx = x + 10;
        let mut ny = y + 125;
        let lw = c.text(nx, ny + 4, "Needs:", ROSEWOOD);
        nx += lw + 6;
        for (item, k) in r.needs {
            let label = format!("{k} {}", item.def().name);
            let need = 18 + c.text_width(&label) + 8;
            if nx + need > x + l.pw - 8 {
                nx = x + 10 + lw + 6;
                ny += 18;
            }
            c.sprite(a.tex(a.icon(item.def().icon)), nx, ny);
            c.text(nx + 18, ny + 4, &label, INK);
            nx += need;
        }

        // OK.
        let (bx, by, bw, bh) = ok_button(&l);
        let by = by + dy;
        let hot = inside(mouse, bx, by, bw, bh);
        c.rect(bx, by, bw, bh, if hot { LIME } else { GREEN });
        c.frame(bx, by, bw, bh, INK);
        let ready = t > 0.4;
        c.text_center(
            bx + bw / 2,
            by + 3,
            &format!("OK {}", self.prompt(Action::Confirm)),
            if ready { WHITE } else { MINT },
        );
        let more = self.discoveries.len();
        if more > 0 {
            let t = format!("+{more} more");
            let tw = c.text_width(&t);
            c.text(x + l.pw - tw - 10, by + 3, &t, RUST);
        }
        // Sparkles round the card.
        for k in 0..12 {
            let ang = self.time * 1.3 + k as f32 * 0.52;
            let sx = x + l.pw / 2 + (ang.cos() * (l.pw as f32 * 0.54)) as i32;
            let sy = y + l.ph / 2 + (ang.sin() * (l.ph as f32 * 0.58)) as i32;
            c.px(sx, sy, [GOLD, WHITE, PINK, SKY][k % 4]);
        }
    }
}
