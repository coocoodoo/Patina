//! The enchanting table: pick a piece of gear and a scroll, pay a few coins, and the
//! scroll's enchantment is bound into one of the gear's three sockets.

use glam::{Vec2, Vec3};

use super::Io;
use super::gear::{Affix, Class, Group, Rarity, SLOTS, SOCKETS};
use super::items::Stack;
use super::menus::{
    CELL, Layout, Menu, Pick, bag_grid, center, inside, nav_bag, worn_hit, worn_pos,
};
use super::play::Play;
use super::tips::{draw_money, money_width, scroll_group_name, stat_icon};
use crate::assets::Assets;
use crate::audio::Sfx;
use crate::input::{Action, Button};
use crate::palette::*;
use crate::ui::{Canvas, Style};

const PW: i32 = 300;
const PH: i32 = 190;
const BAG_TOP: i32 = 92;
const BUTTON: usize = 48;

fn layout(w: i32, h: i32) -> Layout {
    center(w, h, PW, PH)
}

fn socket_rect(l: &Layout, k: usize) -> (i32, i32, i32, i32) {
    (l.px + 156, l.py + 22 + k as i32 * 14, 136, 13)
}

fn button_rect(l: &Layout) -> (i32, i32, i32, i32) {
    (l.px + 156, l.py + 66, 136, 15)
}

/// What binding costs: more for high levels, strong scrolls and already busy gear.
pub fn enchant_cost(level: u16, filled: usize, scroll: Rarity) -> u64 {
    let base = 20 + level as u64 * 6;
    let r = 2 + scroll as u64;
    base * (1 + filled as u64) * r / 2
}

impl Play {
    fn picked_gear(&self, pick: Option<Pick>) -> Option<Stack> {
        let s = match pick? {
            Pick::Bag(i) => self.player.inv.slots.get(i).copied().flatten(),
            Pick::Worn(slot) => self.player.equip[slot as usize],
        }?;
        s.item.base().map(|_| s)
    }

    fn picked_scroll(&self, i: Option<usize>) -> Option<Stack> {
        let s = self.player.inv.slots.get(i?).copied().flatten()?;
        s.item.scroll_group().map(|_| s)
    }

    /// Why binding this scroll to this gear would not work, if it would not.
    fn enchant_problem(
        &self,
        gear: Option<Stack>,
        scroll: Option<Stack>,
        cost: u64,
    ) -> Option<String> {
        let Some(g) = gear else {
            return Some("Choose a piece of gear first.".into());
        };
        let Some(s) = scroll else {
            return Some("Now choose a scroll to bind.".into());
        };
        let class = g.item.class()?;
        let group = s.item.scroll_group()?;
        if class.group() != group {
            return Some(format!(
                "{} scrolls only fit {}.",
                group.name(),
                scroll_group_name(group)
            ));
        }
        let e = s.gear.and_then(|g| g.scroll_enchant())?;
        if !class.suits(e.stat) {
            return Some(format!(
                "{} does nothing for {}.",
                e.stat.def().name,
                class.plural()
            ));
        }
        if self.money < cost {
            return Some(format!("Binding costs {}.", super::loot::money_text(cost)));
        }
        None
    }

    fn put_gear(&mut self, pick: Pick, s: Stack) {
        match pick {
            Pick::Bag(i) => self.player.inv.slots[i] = Some(s),
            Pick::Worn(slot) => self.player.equip[slot as usize] = Some(s),
        }
        self.player.refresh();
    }

    #[allow(clippy::too_many_arguments)]
    pub fn update_enchant(
        &mut self,
        io: &mut Io,
        mut gear: Option<Pick>,
        mut scroll: Option<usize>,
        mut socket: usize,
        mut cursor: usize,
        mut msg: Option<(String, u8)>,
        mut glow: f32,
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
        // Forget picks that no longer point at the right kind of thing.
        if self.picked_gear(gear).is_none() {
            gear = None;
        }
        if self.picked_scroll(scroll).is_none() {
            scroll = None;
        }
        let l = layout(w, h);
        let g = bag_grid(&l, BAG_TOP);
        // Keyboard.
        if cursor < 45 {
            if input.pressed_repeat(Action::Up) && (cursor < 10 || cursor == 40) {
                cursor = BUTTON;
                io.audio.play_at(Sfx::UiMove, 0.5, 1.0);
            } else {
                nav_bag(io, &mut cursor);
            }
        } else {
            if input.pressed_repeat(Action::Up) {
                cursor = if cursor == 45 { BUTTON } else { cursor - 1 };
                io.audio.play_at(Sfx::UiMove, 0.5, 1.0);
            }
            if input.pressed_repeat(Action::Down) {
                cursor = if cursor == BUTTON { 0 } else { cursor + 1 };
                io.audio.play_at(Sfx::UiMove, 0.5, 1.0);
            }
        }
        // Mouse hover.
        let bag_hit = g.hit(mouse);
        let worn = worn_hit(&l, BAG_TOP, mouse);
        let sock_hit = (0..SOCKETS).find(|&k| {
            let (x, y, sw, sh) = socket_rect(&l, k);
            inside(mouse, x, y, sw, sh)
        });
        let (bx, by, bw, bh) = button_rect(&l);
        let on_button = inside(mouse, bx, by, bw, bh);
        if input.mouse_moved {
            if let Some(i) = bag_hit {
                cursor = i;
            } else if let Some(i) = worn {
                cursor = 40 + i;
            } else if let Some(k) = sock_hit {
                cursor = 45 + k;
            } else if on_button {
                cursor = BUTTON;
            }
        }
        // Clearing a pick with a right click on its big slot.
        if rclick && inside(mouse, l.px + 9, l.py + 22, 18, 18) {
            gear = None;
        }
        if rclick && inside(mouse, l.px + 9, l.py + 44, 18, 18) {
            scroll = None;
        }
        // What was chosen this frame.
        let mut chosen: Option<usize> = None;
        if lclick {
            chosen = bag_hit
                .or(worn.map(|i| 40 + i))
                .or(sock_hit.map(|k| 45 + k))
                .or(if on_button { Some(BUTTON) } else { None });
        } else if input.pressed(Action::Confirm) {
            chosen = Some(cursor);
        }
        if let Some(c) = chosen {
            match c {
                0..40 => {
                    let s = self.player.inv.slots[c];
                    match s {
                        Some(st) if st.item.base().is_some() => {
                            gear = Some(Pick::Bag(c));
                            socket = st.gear.and_then(|g| g.free_socket()).unwrap_or(0);
                            msg = None;
                            io.audio.play_at(Sfx::UiSelect, 0.7, 1.1);
                        }
                        Some(st) if st.item.scroll_group().is_some() => {
                            scroll = Some(c);
                            msg = None;
                            io.audio.play_at(Sfx::UiSelect, 0.7, 1.3);
                        }
                        Some(_) => {
                            msg = Some(("Only gear and scrolls go on the table.".into(), SHADOW));
                            io.audio.play(Sfx::Denied);
                        }
                        None => {}
                    }
                }
                40..45 => {
                    let slot = SLOTS[c - 40];
                    if let Some(st) = self.player.equip[slot as usize] {
                        gear = Some(Pick::Worn(slot));
                        socket = st.gear.and_then(|g| g.free_socket()).unwrap_or(0);
                        msg = None;
                        io.audio.play_at(Sfx::UiSelect, 0.7, 1.1);
                    }
                }
                45..48 => {
                    socket = c - 45;
                    io.audio.play_at(Sfx::UiMove, 0.7, 1.0);
                }
                _ => {
                    // Bind!
                    let gs = self.picked_gear(gear);
                    let ss = self.picked_scroll(scroll);
                    let cost = match (gs, ss) {
                        (Some(g), Some(s)) => enchant_cost(
                            g.gear.map_or(1, |x| x.level),
                            g.gear.map_or(0, |x| x.enchants().count()),
                            s.rarity().unwrap_or_default(),
                        ),
                        _ => 0,
                    };
                    match self.enchant_problem(gs, ss, cost) {
                        Some(p) => {
                            msg = Some((p, CRIMSON));
                            io.audio.play(Sfx::Denied);
                        }
                        None => {
                            let (mut g, s) = (gs.unwrap(), ss.unwrap());
                            let e: Affix = s.gear.unwrap().scroll_enchant().unwrap();
                            let before = g.rarity();
                            if let Some(gd) = &mut g.gear {
                                gd.enchant(socket, e);
                            }
                            self.money -= cost;
                            self.player.inv.slots[scroll.unwrap()] = None;
                            scroll = None;
                            self.put_gear(gear.unwrap(), g);
                            glow = 1.2;
                            self.on_enchant();
                            io.audio.play(Sfx::Enchant);
                            let after = g.rarity();
                            let text = if after > before {
                                format!(
                                    "Bound! Your {} is now {}!",
                                    g.item.def().name,
                                    after.unwrap_or_default().name()
                                )
                            } else {
                                format!(
                                    "Bound {} to your {}!",
                                    e.stat.line(e.val as i32),
                                    g.item.def().name
                                )
                            };
                            let col = after.map_or(TEAL, |r| r.ink());
                            msg = Some((text.clone(), col));
                            self.toast_colored(
                                text,
                                Some(g.item),
                                0,
                                after.map_or(CREAM, |r| r.color()),
                            );
                            self.fx.motes(
                                self.player.world_pos() + Vec3::Y * 0.6,
                                24,
                                &[LAVENDER, MINT, BLUSH, WHITE],
                                0.5,
                            );
                            if let Some(gd) = g.gear {
                                socket = gd.free_socket().unwrap_or(socket);
                            }
                        }
                    }
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
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn draw_enchant(
        &self,
        c: &mut Canvas,
        a: &Assets,
        gear: Option<Pick>,
        scroll: Option<usize>,
        socket: usize,
        cursor: usize,
        msg: &Option<(String, u8)>,
        glow: f32,
        mouse: Vec2,
    ) {
        let (w, h) = (c.w(), c.h());
        let l = layout(w, h);
        c.panel(l.px, l.py, l.pw, l.ph, Style::Paper);
        c.text(l.px + 10, l.py + 7, "Enchanting Table", PURPLE);
        draw_money(
            c,
            a,
            l.px + l.pw - 10 - money_width(c, self.money),
            l.py + 7,
            self.money,
            INK,
        );
        let gs = self.picked_gear(gear);
        let ss = self.picked_scroll(scroll);

        // The gear.
        let (gx, gy) = (l.px + 9, l.py + 22);
        self.draw_slot(c, a, gx, gy, gs, false);
        match gs {
            Some(g) => {
                let r = g.rarity().unwrap_or_default();
                c.text(gx + 22, gy, &g.name(), r.ink());
                let d = g.gear.unwrap_or_else(|| super::gear::Gear::plain(1));
                let class = g.item.class().map_or("", Class::name);
                c.text(
                    gx + 22,
                    gy + 9,
                    &format!("{} {} - Lv {}", r.name(), class, d.level),
                    SHADOW,
                );
            }
            None => {
                c.sprite(a.tex(a.icon("ghost_chest")), gx + 1, gy + 1);
                c.text(gx + 22, gy + 4, "Gear", KHAKI);
            }
        }
        // The scroll.
        let (sx, sy) = (l.px + 9, l.py + 44);
        self.draw_slot(c, a, sx, sy, ss, false);
        let scroll_affix = ss.and_then(|s| s.gear).and_then(|g| g.scroll_enchant());
        match (ss, scroll_affix) {
            (Some(s), Some(e)) => {
                let r = s.rarity().unwrap_or_default();
                c.text(sx + 22, sy, &s.name(), r.ink());
                c.sprite(a.tex(a.icon(stat_icon(e.stat))), sx + 22, sy + 9);
                c.text(sx + 32, sy + 9, &e.stat.line(e.val as i32), SHADOW);
            }
            _ => {
                c.sprite(a.tex(a.icon("ghost_scroll")), sx + 1, sy + 1);
                c.text(sx + 22, sy + 4, "Scroll", KHAKI);
            }
        }
        // Advice, or what just happened.
        let cost = match (gs, ss) {
            (Some(g), Some(s)) => enchant_cost(
                g.gear.map_or(1, |x| x.level),
                g.gear.map_or(0, |x| x.enchants().count()),
                s.rarity().unwrap_or_default(),
            ),
            _ => 0,
        };
        let problem = self.enchant_problem(gs, ss, cost);
        let (text, col) = match (msg, &problem) {
            (Some((m, col)), _) => (m.clone(), *col),
            (None, Some(p)) => (p.clone(), SHADOW),
            (None, None) => ("Ready to bind. Choose a socket.".to_string(), TEAL),
        };
        let lines = c.font.wrap(&text, 140);
        for (k, t) in lines.iter().take(2).enumerate() {
            c.text(l.px + 10, l.py + 67 + k as i32 * 9, t, col);
        }

        // Sockets.
        let gd = gs.and_then(|g| g.gear);
        for k in 0..SOCKETS {
            let (x, y, sw, sh) = socket_rect(&l, k);
            let chosen = gs.is_some() && k == socket;
            c.panel(x, y, sw, sh, Style::Inset);
            if chosen {
                c.frame(x, y, sw, sh, PURPLE);
            }
            if cursor == 45 + k {
                c.frame(x - 1, y - 1, sw + 2, sh + 2, GOLD);
            }
            let current = gd.and_then(|g| g.enchants[k]);
            let preview = if chosen && problem.is_none() {
                scroll_affix
            } else {
                None
            };
            // Shown as they'll count, with the gear's forging.
            let forged = |v: i16| gd.map_or(v as i32, |g| g.forged_stat(v as i32));
            match (preview, current) {
                (Some(e), _) => {
                    c.sprite(a.tex(a.icon(stat_icon(e.stat))), x + 2, y + 2);
                    let t = format!("{} new!", e.stat.line(forged(e.val)));
                    c.text(
                        x + 12,
                        y + 3,
                        &t,
                        if (self.time * 4.0).sin() > 0.0 {
                            GREEN
                        } else {
                            TEAL
                        },
                    );
                }
                (None, Some(e)) => {
                    c.sprite(a.tex(a.icon(stat_icon(e.stat))), x + 2, y + 2);
                    c.text(x + 12, y + 3, &e.stat.line(forged(e.val)), PLUM);
                }
                (None, None) => {
                    c.sprite(a.tex(a.icon("socket")), x + 2, y + 2);
                    c.text(x + 12, y + 3, "Empty socket", ROSEWOOD);
                }
            }
        }
        // The button.
        let (bx, by, bw, bh) = button_rect(&l);
        let ready = problem.is_none();
        c.rect(bx, by, bw, bh, if ready { PURPLE } else { KHAKI });
        c.frame(bx, by, bw, bh, INK);
        if cursor == BUTTON {
            c.frame(bx - 1, by - 1, bw + 2, bh + 2, GOLD);
        }
        let label = "Enchant";
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
        // Mark the picks.
        let mark = |c: &mut Canvas, x: i32, y: i32, col: u8| {
            c.frame(x - 1, y - 1, 20, 20, col);
        };
        if let Some(Pick::Bag(i)) = gear {
            let (x, y) = g.slot_pos(i);
            mark(c, x, y, PURPLE);
        }
        if let Some(Pick::Worn(slot)) = gear {
            let (x, y) = worn_pos(&l, BAG_TOP, slot as usize);
            mark(c, x, y, PURPLE);
        }
        if let Some(i) = scroll {
            let (x, y) = g.slot_pos(i);
            mark(c, x, y, PINK);
        }
        // Scrolls that fit the chosen gear twinkle.
        if let Some(class) = gs.and_then(|g| g.item.class()) {
            for (i, s) in self.player.inv.slots.iter().enumerate() {
                let Some(s) = s else { continue };
                if s.item.scroll_group() == Some(class.group())
                    && (self.time * 3.0 + i as f32).sin() > 0.6
                {
                    let (x, y) = g.slot_pos(i);
                    c.px(x + 15, y + 2, WHITE);
                    c.px(x + 14, y + 3, CREAM);
                }
            }
        }
        // Rarity before and after.
        let rx = g.x + g.w() + 6;
        if let (Some(gs), Some(e)) = (gs, scroll_affix.filter(|_| problem.is_none())) {
            let mut after = gs.gear.unwrap();
            after.enchant(socket, e);
            let (r0, r1) = (gs.rarity().unwrap_or_default(), after.rarity);
            c.text(rx, g.y + 2, "Rarity", SHADOW);
            c.text(rx, g.y + 13, r0.name(), r0.ink());
            c.text(rx, g.y + 24, "▼", if r1 > r0 { GREEN } else { KHAKI });
            c.text(rx, g.y + 35, r1.name(), r1.ink());
        } else {
            let tips = ["Gear", "+ scroll", "+ coins", "= magic!"];
            for (k, t) in tips.iter().enumerate() {
                c.text(rx, g.y + 2 + k as i32 * 11, t, KHAKI);
            }
        }
        let _ = CELL;
        let _ = Group::Weapon;
        // Sparkles after a binding.
        if glow > 0.0 {
            for k in 0..10 {
                let ang = self.time * 5.0 + k as f32 * 0.628;
                let rad = 14.0 + (1.2 - glow) * 20.0;
                let x = gx + 9 + (ang.cos() * rad) as i32;
                let y = gy + 9 + (ang.sin() * rad * 0.7) as i32;
                c.px(x, y, [LAVENDER, MINT, BLUSH, WHITE][k % 4]);
            }
        }
        // Tooltips: for what's under the mouse, or else under the cursor.
        let hovered = g.hit(mouse).is_some()
            || worn_hit(&l, BAG_TOP, mouse).is_some()
            || inside(mouse, gx, gy, 18, 18)
            || inside(mouse, sx, sy, 18, 18);
        let bag = g.hit(mouse).or((!hovered && cursor < 40).then_some(cursor));
        let worn = worn_hit(&l, BAG_TOP, mouse)
            .or((!hovered && (40..45).contains(&cursor)).then(|| cursor - 40));
        if let Some(i) = bag {
            if let Some(s) = self.player.inv.slots[i] {
                let (x, y) = g.slot_pos(i);
                self.stack_tooltip(c, a, x + 22, y - 30, &s);
            }
        } else if let Some(i) = worn {
            if let Some(s) = self.player.equip[i] {
                let (x, y) = worn_pos(&l, BAG_TOP, i);
                self.stack_tooltip(c, a, x + 22, y - 30, &s);
            }
        } else if inside(mouse, gx, gy, 18, 18) {
            if let Some(s) = gs {
                self.stack_tooltip(c, a, l.px + l.pw + 4, gy, &s);
            }
        } else if inside(mouse, sx, sy, 18, 18) {
            if let Some(s) = ss {
                self.stack_tooltip(c, a, l.px + l.pw + 4, sy, &s);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn costs_climb() {
        assert!(enchant_cost(30, 0, Rarity::Common) > enchant_cost(5, 0, Rarity::Common));
        assert!(enchant_cost(10, 2, Rarity::Common) > enchant_cost(10, 0, Rarity::Common));
        assert!(enchant_cost(10, 0, Rarity::Legendary) > enchant_cost(10, 0, Rarity::Common));
    }
}
