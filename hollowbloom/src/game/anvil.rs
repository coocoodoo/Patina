//! Hilde's anvil, in the Petalplate Armory: choose a piece of armour to forge and another to
//! melt down into it, pay for the fire, and the first soaks up the second's forge experience.
//! Every forge level (up to +10) makes a piece a little stronger, and every level asks more
//! than the last - more still for armour from deeper down - and costs more to strike.

use glam::{Vec2, Vec3};

use super::Io;
use super::gear::{self, Gear, MAX_FORGE, Rarity, SLOTS};
use super::items::Stack;
use super::menus::{Layout, Menu, Pick, bag_grid, center, inside, nav_bag, worn_hit, worn_pos};
use super::play::Play;
use super::tips::{draw_money, money_width};
use crate::assets::Assets;
use crate::audio::Sfx;
use crate::input::{Action, Button};
use crate::palette::*;
use crate::ui::{Canvas, Style};

const PW: i32 = 300;
const PH: i32 = 190;
const BAG_TOP: i32 = 92;
const BUTTON: usize = 45;

fn layout(w: i32, h: i32) -> Layout {
    center(w, h, PW, PH)
}

fn button_rect(l: &Layout) -> (i32, i32, i32, i32) {
    (l.px + 156, l.py + 66, 136, 15)
}

/// What a strike would do: the armour after it, the forge experience melted in, and what it
/// costs.
#[derive(Clone, Copy)]
pub struct Strike {
    pub after: Gear,
    pub xp: u32,
    pub cost: u64,
}

/// What melting `fodder` into `target` would do.
pub fn strike(target: &Stack, fodder: &Stack) -> Option<Strike> {
    let (before, f) = (target.gear?, fodder.gear?);
    let same = target.item.class() == fodder.item.class();
    let xp = gear::fodder_xp(&f, same);
    let mut after = before;
    after.add_forge_xp(xp);
    Some(Strike {
        after,
        xp,
        cost: gear::forge_cost(before.level, before.forge()),
    })
}

fn armour(s: &Stack) -> bool {
    s.item.class().is_some_and(|c| c.is_armor())
}

impl Play {
    fn picked_armour(&self, pick: Option<Pick>) -> Option<Stack> {
        let s = match pick? {
            Pick::Bag(i) => self.player.inv.slots.get(i).copied().flatten(),
            Pick::Worn(slot) => self.player.equip[slot as usize],
        }?;
        armour(&s).then_some(s)
    }

    fn picked_fodder(&self, i: Option<usize>) -> Option<Stack> {
        let s = self.player.inv.slots.get(i?).copied().flatten()?;
        armour(&s).then_some(s)
    }

    /// Why striking would not work, if it would not.
    fn forge_problem(&self, target: Option<Stack>, fodder: Option<Stack>) -> Option<String> {
        let Some(t) = target else {
            return Some("Choose a piece of armour to forge.".into());
        };
        if t.gear.is_some_and(|g| g.forge() >= MAX_FORGE) {
            return Some(format!(
                "Your {} is forged as far as it goes!",
                t.item.def().name
            ));
        }
        let Some(f) = fodder else {
            return Some("Now choose armour to melt into it.".into());
        };
        let s = strike(&t, &f)?;
        if self.money < s.cost {
            return Some(format!(
                "Hilde charges {} to fire the forge.",
                super::loot::money_text(s.cost)
            ));
        }
        None
    }

    fn put_armour(&mut self, pick: Pick, s: Stack) {
        match pick {
            Pick::Bag(i) => self.player.inv.slots[i] = Some(s),
            Pick::Worn(slot) => self.player.equip[slot as usize] = Some(s),
        }
        self.player.refresh();
    }

    #[allow(clippy::too_many_arguments)]
    pub fn update_anvil(
        &mut self,
        io: &mut Io,
        mut target: Option<Pick>,
        mut fodder: Option<usize>,
        mut cursor: usize,
        mut msg: Option<(String, u8)>,
        mut glow: f32,
        mut armed: bool,
    ) -> Menu {
        let (w, h) = (io.view.0 as i32, io.view.1 as i32);
        let input = io.input;
        let mouse = input.mouse;
        let lclick = input.button_pressed(Button::Left);
        let rclick = input.button_pressed(Button::Right);
        glow = (glow - io.dt).max(0.0);
        if input.pressed(Action::Cancel) || input.pressed(Action::Inventory) {
            self.close_menu(io);
            return Menu::None;
        }
        // Forget picks that no longer point at armour.
        if self.picked_armour(target).is_none() {
            target = None;
        }
        if self.picked_fodder(fodder).is_none() || target == fodder.map(Pick::Bag) {
            fodder = None;
        }
        let l = layout(w, h);
        let g = bag_grid(&l, BAG_TOP);
        // Keyboard.
        if cursor < BUTTON {
            if input.pressed_repeat(Action::Up) && (cursor < 10 || cursor == 40) {
                cursor = BUTTON;
                io.audio.play_at(Sfx::UiMove, 0.5, 1.0);
            } else {
                nav_bag(io, &mut cursor);
            }
        } else if input.pressed_repeat(Action::Down) {
            cursor = 0;
            io.audio.play_at(Sfx::UiMove, 0.5, 1.0);
        }
        // Mouse hover.
        let bag_hit = g.hit(mouse);
        let worn = worn_hit(&l, BAG_TOP, mouse);
        let (bx, by, bw, bh) = button_rect(&l);
        let on_button = inside(mouse, bx, by, bw, bh);
        if input.mouse_moved {
            if let Some(i) = bag_hit {
                cursor = i;
            } else if let Some(i) = worn {
                cursor = 40 + i;
            } else if on_button {
                cursor = BUTTON;
            }
        }
        // Clearing a pick with a right click on its big slot.
        if rclick && inside(mouse, l.px + 9, l.py + 22, 18, 18) {
            target = None;
            armed = false;
        }
        if rclick && inside(mouse, l.px + 9, l.py + 44, 18, 18) {
            fodder = None;
            armed = false;
        }
        let mut chosen: Option<usize> = None;
        if lclick {
            chosen =
                bag_hit
                    .or(worn.map(|i| 40 + i))
                    .or(if on_button { Some(BUTTON) } else { None });
        } else if input.pressed(Action::Confirm) {
            chosen = Some(cursor);
        }
        if let Some(c) = chosen {
            match c {
                0..40 => match self.player.inv.slots[c] {
                    Some(st) if armour(&st) => {
                        // The first piece chosen is the one to forge, the next the one to
                        // melt; choosing either again puts it back.
                        if target == Some(Pick::Bag(c)) {
                            target = None;
                        } else if fodder == Some(c) {
                            fodder = None;
                        } else if target.is_none() {
                            target = Some(Pick::Bag(c));
                        } else {
                            fodder = Some(c);
                        }
                        msg = None;
                        armed = false;
                        io.audio.play_at(Sfx::UiSelect, 0.7, 1.1);
                    }
                    Some(_) => {
                        msg = Some(("Only armour goes on the anvil.".into(), SHADOW));
                        io.audio.play(Sfx::Denied);
                    }
                    None => {}
                },
                40..45 => {
                    // Worn armour can be forged (but take it off to melt it down).
                    let slot = SLOTS[c - 40];
                    if self.player.equip[slot as usize].is_some() {
                        target = if target == Some(Pick::Worn(slot)) {
                            None
                        } else {
                            Some(Pick::Worn(slot))
                        };
                        msg = None;
                        armed = false;
                        io.audio.play_at(Sfx::UiSelect, 0.7, 1.1);
                    }
                }
                _ => {
                    let (ts, fs) = (self.picked_armour(target), self.picked_fodder(fodder));
                    match self.forge_problem(ts, fs) {
                        Some(p) => {
                            msg = Some((p, CRIMSON));
                            io.audio.play(Sfx::Denied);
                        }
                        None => {
                            let (mut t, f) = (ts.unwrap(), fs.unwrap());
                            let precious = f.rarity().is_some_and(|r| r >= Rarity::Epic)
                                || f.gear.is_some_and(|g| g.enchants().next().is_some());
                            if precious && !armed {
                                // Make sure.
                                armed = true;
                                let what = match f.rarity() {
                                    Some(r) if r >= Rarity::Epic => {
                                        format!("{} {}", r.name(), f.name())
                                    }
                                    _ => format!("enchanted {}", f.name()),
                                };
                                msg = Some((
                                    format!("Melt down your {what}? Strike again to be sure."),
                                    RUST,
                                ));
                                io.audio.play_at(Sfx::UiMove, 0.7, 0.8);
                            } else {
                                let s = strike(&t, &f).unwrap();
                                self.money -= s.cost;
                                self.player.inv.slots[fodder.unwrap()] = None;
                                fodder = None;
                                armed = false;
                                let ups = t.gear.as_mut().map_or(0, |g| g.add_forge_xp(s.xp));
                                self.put_armour(target.unwrap(), t);
                                glow = 1.2;
                                io.audio.play(Sfx::Clang);
                                let at = self.player.world_pos() + Vec3::Y * 0.6;
                                self.fx.motes(at, 20, &[GOLD, ORANGE, CREAM, WHITE], 0.6);
                                let text = if ups > 0 {
                                    io.audio.play(Sfx::LevelUp);
                                    format!("Forged! It's {} now!", t.name())
                                } else {
                                    format!("Your {} soaks up {} forge xp.", t.name(), s.xp)
                                };
                                msg = Some((text.clone(), if ups > 0 { RUST } else { TEAL }));
                                if ups > 0 {
                                    self.toast_colored(text, Some(t.item), 0, GOLD);
                                }
                            }
                        }
                    }
                }
            }
        }
        Menu::Anvil {
            target,
            fodder,
            cursor,
            msg,
            glow,
            armed,
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn draw_anvil(
        &self,
        c: &mut Canvas,
        a: &Assets,
        target: Option<Pick>,
        fodder: Option<usize>,
        cursor: usize,
        msg: &Option<(String, u8)>,
        glow: f32,
        mouse: Vec2,
    ) {
        let (w, h) = (c.w(), c.h());
        let l = layout(w, h);
        c.panel(l.px, l.py, l.pw, l.ph, Style::Paper);
        c.text(l.px + 10, l.py + 7, "Hilde's Anvil", RUST);
        draw_money(
            c,
            a,
            l.px + l.pw - 10 - money_width(c, self.money),
            l.py + 7,
            self.money,
            INK,
        );
        let ts = self.picked_armour(target);
        let fs = self.picked_fodder(fodder);
        let s = match (ts, fs) {
            (Some(t), Some(f)) => strike(&t, &f),
            _ => None,
        };

        // The armour to forge.
        let (tx, ty) = (l.px + 9, l.py + 22);
        self.draw_slot(c, a, tx, ty, ts, false);
        match ts {
            Some(t) => {
                let r = t.rarity().unwrap_or_default();
                c.text(tx + 22, ty, &t.name(), r.ink());
                let lvl = t.gear.map_or(1, |g| g.level);
                let class = t.item.class().map_or("", |c| c.name());
                c.text(tx + 22, ty + 9, &format!("{class} - Lv {lvl}"), SHADOW);
            }
            None => {
                c.sprite(a.tex(a.icon("ghost_chest")), tx + 1, ty + 1);
                c.text(tx + 22, ty + 4, "Armour to forge", KHAKI);
            }
        }
        // The armour to melt down.
        let (fx, fy) = (l.px + 9, l.py + 44);
        self.draw_slot(c, a, fx, fy, fs, false);
        match (fs, s) {
            (Some(f), Some(s)) => {
                let r = f.rarity().unwrap_or_default();
                c.text(fx + 22, fy, &f.name(), r.ink());
                let same = ts.and_then(|t| t.item.class()) == f.item.class();
                let t = if same {
                    format!("+{} xp (own kind!)", s.xp)
                } else {
                    format!("+{} forge xp", s.xp)
                };
                c.text(fx + 22, fy + 9, &t, CLAY);
            }
            (Some(f), None) => {
                let r = f.rarity().unwrap_or_default();
                c.text(fx + 22, fy, &f.name(), r.ink());
            }
            _ => {
                c.sprite(a.tex(a.icon("ghost_chest")), fx + 1, fy + 1);
                c.text(fx + 22, fy + 4, "Armour to melt", KHAKI);
            }
        }
        // Advice, or what just happened.
        let problem = self.forge_problem(ts, fs);
        let (text, col) = match (msg, &problem) {
            (Some((m, col)), _) => (m.clone(), *col),
            (None, Some(p)) => (p.clone(), SHADOW),
            (None, None) => ("Ready! Strike while the iron's hot.".to_string(), TEAL),
        };
        let lines = c.font.wrap(&text, 140);
        for (k, t) in lines.iter().take(2).enumerate() {
            c.text(l.px + 10, l.py + 67 + k as i32 * 9, t, col);
        }

        // Its forge level, as pips, and how far along the next it is.
        let (rx, ry) = (l.px + 156, l.py + 22);
        let now = ts.and_then(|t| t.gear);
        let then = s.map(|s| s.after);
        let f0 = now.map_or(0, |g| g.forge());
        let f1 = then.map_or(f0, |g| g.forge());
        let blink = (self.time * 4.0).sin() > 0.0;
        let (head, head_col) = match (now, then) {
            (Some(_), Some(_)) if f1 > f0 => (
                format!("Forge +{f0} -> +{f1}!"),
                if blink { RUST } else { ORANGE },
            ),
            (Some(_), _) => (format!("Forge +{f0}"), INK),
            (None, _) => ("Forge level".to_string(), KHAKI),
        };
        c.text(rx, ry, &head, head_col);
        for k in 0..MAX_FORGE {
            let (x, y) = (rx + k as i32 * 13, ry + 10);
            let col = if k < f0 {
                GOLD
            } else if k < f1 {
                if blink { ORANGE } else { CREAM }
            } else {
                SAND
            };
            c.rect(x, y, 11, 5, col);
            c.frame(x - 1, y - 1, 13, 7, if k < f1 { RUST } else { KHAKI });
        }
        // The bar towards the next level, with what this strike adds.
        let (bar_x, bar_y, bar_w) = (rx, ry + 19, 136);
        c.rect(bar_x, bar_y, bar_w, 6, SAND);
        c.frame(bar_x - 1, bar_y - 1, bar_w + 2, 8, KHAKI);
        if let Some(g) = then.or(now) {
            let (into, need) = g.forge_progress();
            let fill = |v: u32, n: u32| (v as i64 * bar_w as i64 / n.max(1) as i64) as i32;
            match need {
                Some(n) => {
                    // What it had, and (if it went up no levels) what this adds.
                    let had = match (now, then) {
                        (Some(a), Some(_)) if f1 == f0 => a.forge_progress().0,
                        (Some(_), Some(_)) => 0,
                        _ => into,
                    };
                    c.rect(
                        bar_x,
                        bar_y,
                        fill(into, n),
                        6,
                        if blink { CREAM } else { GOLD },
                    );
                    c.rect(bar_x, bar_y, fill(had, n), 6, GOLD);
                    let t = format!("{into}/{n} xp");
                    c.text(bar_x, bar_y + 8, &t, SHADOW);
                }
                None => {
                    c.rect(bar_x, bar_y, bar_w, 6, GOLD);
                    c.text(bar_x, bar_y + 8, "As strong as it gets", RUST);
                }
            }
        }
        // What it does for its defense.
        if let (Some(t), Some(after)) = (ts, then) {
            let before = t.main_value().unwrap_or(0);
            let mut t2 = t;
            t2.gear = Some(after);
            let now_v = t2.main_value().unwrap_or(before);
            if now_v != before {
                let text = format!("Defense {before} -> {now_v}");
                c.text(bar_x + 70, bar_y + 8, &text, GREEN);
            }
        }
        // The button.
        let (bx, by, bw, bh) = button_rect(&l);
        let ready = problem.is_none();
        c.rect(bx, by, bw, bh, if ready { RUST } else { KHAKI });
        c.frame(bx, by, bw, bh, INK);
        if cursor == BUTTON {
            c.frame(bx - 1, by - 1, bw + 2, bh + 2, GOLD);
        }
        let label = "Strike";
        let cost = s.map_or(0, |s| s.cost);
        if cost > 0 {
            let mw = money_width(c, cost);
            let tw = c.text_width(label);
            let x0 = bx + (bw - tw - mw - 6) / 2;
            c.text(x0, by + 4, label, if ready { WHITE } else { ROSEWOOD });
            draw_money(
                c,
                a,
                x0 + tw + 6,
                by + 4,
                cost,
                if ready { CREAM } else { ROSEWOOD },
            );
        } else {
            c.text_center(
                bx + bw / 2,
                by + 4,
                label,
                if ready { WHITE } else { ROSEWOOD },
            );
        }

        // Bag and worn gear to choose from.
        let g = bag_grid(&l, BAG_TOP);
        self.draw_grid(
            c,
            a,
            &g,
            &self.player.inv,
            if cursor < 40 { Some(cursor) } else { None },
        );
        self.draw_worn(
            c,
            a,
            &l,
            BAG_TOP,
            if (40..45).contains(&cursor) {
                Some(cursor - 40)
            } else {
                None
            },
        );
        let mark = |c: &mut Canvas, x: i32, y: i32, col: u8| {
            c.frame(x - 1, y - 1, 20, 20, col);
        };
        match target {
            Some(Pick::Bag(i)) => {
                let (x, y) = g.slot_pos(i);
                mark(c, x, y, RUST);
            }
            Some(Pick::Worn(slot)) => {
                let (x, y) = worn_pos(&l, BAG_TOP, slot as usize);
                mark(c, x, y, RUST);
            }
            None => {}
        }
        if let Some(i) = fodder {
            let (x, y) = g.slot_pos(i);
            mark(c, x, y, ORANGE);
        }
        // Armour in the bag twinkles: it could go on the anvil.
        for (i, st) in self.player.inv.slots.iter().enumerate().take(40) {
            let Some(st) = st else { continue };
            if armour(st) && (self.time * 3.0 + i as f32).sin() > 0.7 {
                let (x, y) = g.slot_pos(i);
                c.px(x + 15, y + 2, WHITE);
                c.px(x + 14, y + 3, GOLD);
            }
        }
        let rx = g.x + g.w() + 6;
        let tips = ["Armour", "+ armour", "+ coins", "= stronger!"];
        for (k, t) in tips.iter().enumerate() {
            c.text(rx, g.y + 2 + k as i32 * 11, t, KHAKI);
        }
        // Sparks flying off a strike.
        if glow > 0.0 {
            for k in 0..12 {
                let ang = k as f32 * 0.524 + (k % 3) as f32;
                let rad = 6.0 + (1.2 - glow) * 26.0;
                let x = tx + 9 + (ang.cos() * rad) as i32;
                let y = ty + 9 + (ang.sin() * rad * 0.8 - (1.2 - glow) * 8.0) as i32;
                c.px(x, y, [GOLD, ORANGE, CREAM, WHITE][k % 4]);
            }
        }
        // Tooltips: for what's under the mouse, or else under the cursor.
        let hovered = g.hit(mouse).is_some()
            || worn_hit(&l, BAG_TOP, mouse).is_some()
            || inside(mouse, tx, ty, 18, 18)
            || inside(mouse, fx, fy, 18, 18);
        let bag = g.hit(mouse).or((!hovered && cursor < 40).then_some(cursor));
        let worn = worn_hit(&l, BAG_TOP, mouse)
            .or((!hovered && (40..45).contains(&cursor)).then(|| cursor - 40));
        if let Some(i) = bag {
            if let Some(st) = self.player.inv.slots[i] {
                let (x, y) = g.slot_pos(i);
                self.stack_tooltip(c, a, x + 22, y - 30, &st);
            }
        } else if let Some(i) = worn {
            if let Some(st) = self.player.equip[i] {
                let (x, y) = worn_pos(&l, BAG_TOP, i);
                self.stack_tooltip(c, a, x + 22, y - 30, &st);
            }
        } else if inside(mouse, tx, ty, 18, 18) {
            if let Some(st) = ts {
                self.stack_tooltip(c, a, l.px + l.pw + 4, ty, &st);
            }
        } else if inside(mouse, fx, fy, 18, 18) {
            if let Some(st) = fs {
                self.stack_tooltip(c, a, l.px + l.pw + 4, fy, &st);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::items::Item;

    #[test]
    fn a_strike_melts_one_piece_into_another() {
        let helm = Stack::with_gear(Item::IronHelm, Gear::plain(17));
        let old = Stack::with_gear(Item::CopperHelm, Gear::plain(9));
        let boots = Stack::with_gear(Item::CopperSabatons, Gear::plain(9));
        let s = strike(&helm, &old).unwrap();
        assert!(s.xp > 0);
        assert_eq!(s.after.xp, s.xp);
        assert_eq!(s.cost, gear::forge_cost(17, 0));
        // A helm melts better into a helm.
        assert!(s.xp > strike(&helm, &boots).unwrap().xp);
        // Forged armour costs more to strike.
        let mut forged = helm;
        forged.gear.as_mut().unwrap().add_forge_xp(10_000);
        assert!(strike(&forged, &old).unwrap().cost > s.cost);
    }
}
