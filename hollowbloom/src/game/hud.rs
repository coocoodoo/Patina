//! The heads-up display.

use glam::Vec3;

use super::fish::{self, Hooked, Phase};
use super::gear::Class;
use super::home::charm_title;
use super::items::{Kind, Placeable};
use super::menus::Menu;
use super::play::Play;
use super::player::HOTBAR;
use super::tips::{draw_money, money_width};
use super::world::{Area, Floor, Obj, Wall};
use crate::assets::Assets;
use crate::input::Action;
use crate::palette::*;
use crate::render::Camera;
use crate::ui::{Canvas, Style};

impl Play {
    pub fn draw_hud(&self, c: &mut Canvas, a: &Assets, cam: &Camera) {
        let (w, h) = (c.w(), c.h());
        // Floating numbers over the world.
        for p in &self.fx.pops {
            let rise = p.t * if p.big { 0.5 } else { 1.2 };
            if let Some(s) = cam.project(p.pos + Vec3::Y * rise) {
                let text = &p.text;
                if p.big {
                    if (p.t * 8.0).fract() < 0.85 {
                        let tw = c.big_width(text, 2);
                        c.text_big(s.x as i32 - tw / 2, s.y as i32 - 8, text, 2, p.color, INK);
                    }
                } else {
                    let tw = c.text_width(text);
                    c.text_outline(s.x as i32 - tw / 2, s.y as i32 - 4, text, p.color, INK);
                }
            }
        }
        // Enemy health bars once they are hurt, named for a moment after each hit.
        for f in &self.foes {
            if f.hp < f.max_hp && !f.boss {
                if let Some(s) = cam.project(f.world_pos() + Vec3::Y * (0.75 * f.scale() + 0.2)) {
                    let bw = 14;
                    c.bar(
                        s.x as i32 - bw / 2,
                        s.y as i32 - 3,
                        bw,
                        4,
                        f.hp as f32 / f.max_hp as f32,
                        RED,
                        SALMON,
                        INK,
                    );
                    if f.named > 0.0 && (f.named > 0.4 || (f.named * 10.0).fract() < 0.5) {
                        let name = f.name();
                        let tw = c.text_width(name);
                        let col = if f.moonlit { SALMON } else { CREAM };
                        c.text_outline(s.x as i32 - tw / 2, s.y as i32 - 12, name, col, INK);
                    }
                }
            }
        }
        if matches!(self.menu, Menu::None) {
            self.draw_folk_hud(c, cam);
        }
        if !matches!(
            self.menu,
            Menu::None | Menu::Dialog { .. } | Menu::Talk { .. }
        ) {
            return;
        }

        // Health, energy and mana.
        let p = &self.player;
        c.panel(3, 3, 94, 35, Style::Dark);
        c.text_shadow(7, 6, "♥", PINK, INK);
        let low = p.hp * 4 < p.max_hp() && (self.time * 4.0).fract() < 0.5;
        c.bar(
            15,
            7,
            60,
            7,
            p.hp as f32 / p.max_hp() as f32,
            if low { SALMON } else { CRIMSON },
            PINK,
            SHADOW,
        );
        c.tiny(94, 8, &p.hp.max(0).to_string(), WHITE, INK);
        c.text_shadow(7, 16, "⚡", GOLD, INK);
        c.bar(
            15,
            17,
            60,
            7,
            p.energy.max(0.0) / p.max_energy() as f32,
            CLAY,
            GOLD,
            SHADOW,
        );
        c.tiny(94, 18, &(p.energy.max(0.0) as i32).to_string(), WHITE, INK);
        c.text_shadow(7, 26, "★", LAVENDER, INK);
        let flash = p.no_mana_t > 0.0 && (self.time * 8.0).fract() < 0.5;
        c.bar(
            15,
            27,
            60,
            7,
            p.mana / p.max_mana() as f32,
            if flash { SALMON } else { PURPLE },
            LAVENDER,
            SHADOW,
        );
        c.tiny(94, 28, &(p.mana as i32).to_string(), WHITE, INK);
        // Food buffs, with how long they have left.
        let mut bx = 4;
        for b in &p.buffs {
            let icon = a.tex(a.icon(b.from.def().icon));
            c.panel(bx, 40, 14, 14, Style::Dark);
            for sy in 0..8 {
                for sx in 0..8 {
                    let col = icon.get(sx * 2, sy * 2);
                    if col != CLEAR {
                        c.px(bx + 3 + sx, 43 + sy, col);
                    }
                }
            }
            let secs = b.left.ceil() as i32;
            let t = if secs >= 60 {
                format!("{}", secs / 60)
            } else {
                format!("{secs}")
            };
            let blink = b.left < 10.0 && (self.time * 4.0).fract() < 0.5;
            if !blink {
                c.tiny(bx + 14, 50, &t, WHITE, INK);
            }
            bx += 16;
        }

        // The quest tracker, under any food buffs.
        let ty0 = if p.buffs.is_empty() { 40 } else { 56 };
        self.draw_tracker(c, a, ty0);

        // Clock, date and money.
        let pw = 86;
        let px = w - pw - 3;
        c.panel(px, 3, pw, 35, Style::Paper);
        c.text(
            px + 6,
            6,
            &format!("Day {} {}", self.clock.day, self.clock.weekday()),
            INK,
        );
        // Tonight's moon in the evening and down in the Hollow (where it stirs the
        // creatures), the sun or the rain by day.
        if matches!(self.area, Area::Hollow { .. }) || self.clock.min > 1110.0 {
            draw_moon(c, px + pw - 15, 5, self.moon(), self.area != Area::Farm);
        } else {
            let (icon, col) = if self.rain {
                ("●", SKY)
            } else {
                ("●", GOLD)
            };
            c.text(px + pw - 12, 6, icon, col);
        }
        c.text(px + 6, 16, &self.clock.label(), RUST);
        let season = self.clock.season();
        let sw = c.text_width(season.name());
        c.text(px + pw - 6 - sw, 16, season.name(), season.color());
        let mw = money_width(c, self.money);
        draw_money(c, a, px + pw - 6 - mw, 26, self.money, INK);
        if let Area::Hollow { depth } = self.area {
            c.panel(px, 40, pw, 14, Style::Dark);
            let label = if self.in_vault() {
                "Secret room".to_string()
            } else {
                format!("Floor {depth}")
            };
            c.text_shadow(px + 6, 43, &label, CREAM, INK);
            if self.show_map {
                self.draw_minimap(c, px, 56, pw);
            }
        }
        match self.area {
            Area::Town if self.show_map => {
                c.panel(px, 40, pw, 14, Style::Dark);
                c.text_shadow(px + 6, 43, "Bramblewick", CREAM, INK);
                self.draw_town_map(c, px + 3, 56);
            }
            Area::Inside(place) => {
                let name = place.def().name;
                let pw2 = c.text_width(name) + 12;
                c.panel(w - pw2 - 3, 40, pw2, 14, Style::Dark);
                c.text_shadow(w - pw2 + 3, 43, name, CREAM, INK);
            }
            Area::Home => {
                // How charming the place is.
                let ch = self.charisma();
                let line = format!("{ch} {}", charm_title(ch));
                let pw2 = c.text_width(&line) + 22;
                c.panel(w - pw2 - 3, 40, pw2, 14, Style::Dark);
                c.text_shadow(w - pw2 + 3, 43, "♥", PINK, INK);
                c.text_shadow(w - pw2 + 13, 43, &line, CREAM, INK);
            }
            _ => {}
        }
        self.draw_fishing_hud(c, a, cam);
        // How the dish is coming along.
        if let Some(k) = &self.cooking {
            if let Some(s) = cam.project(k.at + Vec3::Y * 1.5) {
                let (bx, by) = (s.x as i32 - 16, s.y as i32 - 4);
                c.rect(bx - 1, by - 1, 34, 7, INK);
                c.bar(
                    bx,
                    by,
                    32,
                    5,
                    k.t / super::home::COOK_TIME,
                    ORANGE,
                    GOLD,
                    SHADOW,
                );
                let name = super::items::RECIPES[k.recipe].out.def().name;
                let tw = c.text_width(name);
                c.text_outline(s.x as i32 - tw / 2, by - 12, name, CREAM, INK);
            }
        }

        // Boss bar.
        if let Some(b) = self.foes.iter().find(|f| f.boss && f.alert) {
            let bw = (w / 2).min(200);
            let bx = (w - bw) / 2;
            c.text_outline(
                bx + (bw - c.text_width(b.name())) / 2,
                5,
                b.name(),
                CREAM,
                INK,
            );
            c.bar(
                bx,
                16,
                bw,
                7,
                b.hp as f32 / b.max_hp as f32,
                CRIMSON,
                PINK,
                INK,
            );
        }

        // Hotbar.
        let hb_w = HOTBAR as i32 * 19 - 1;
        let hx = (w - hb_w) / 2;
        let hy = h - 22;
        c.panel(hx - 3, hy - 3, hb_w + 6, 24, Style::Dark);
        for i in 0..HOTBAR {
            let x = hx + i as i32 * 19;
            let s = p.inv.slots[i];
            self.draw_slot(c, a, x, hy, s, i == p.sel);
            if let Some(s) = s {
                if s.item.class() == Some(Class::Can) {
                    let frac = p.water as f32 / p.can_capacity() as f32;
                    c.rect(x + 2, hy + 15, 14, 2, SHADOW);
                    c.rect(x + 2, hy + 15, (14.0 * frac.min(1.0)) as i32, 2, SKY);
                }
            }
            let key = if i == 9 {
                "0".to_string()
            } else {
                (i + 1).to_string()
            };
            c.tiny(
                x + 4,
                hy + 1,
                &key,
                if i == p.sel { CREAM } else { KHAKI },
                INK,
            );
        }
        // The two readied spells, on Q and R.
        if !self.spells.known.is_empty() {
            let sx = hx + hb_w + 9;
            c.panel(sx - 3, hy - 3, 2 * 19 + 5, 24, Style::Dark);
            for k in 0..2 {
                let x = sx + k as i32 * 19;
                c.panel(x, hy, 18, 18, Style::Inset);
                if let Some(s) = self.spells.slots[k] {
                    let d = s.def();
                    let known = self.spells.get(s).copied();
                    let cost = known.map_or(d.cost, |k| self.player.mana_cost(k.cost()));
                    let icon = a.tex(a.icon(d.icon));
                    let cool = self.spells.cool[k] / d.cooldown.max(0.01);
                    if self.player.mana < cost {
                        let dim = c.darken[1];
                        c.sprite_map(icon, x + 1, hy + 1, |col| dim[col as usize]);
                    } else {
                        c.sprite(icon, x + 1, hy + 1);
                    }
                    // Cooling down: a shadow that shrinks away.
                    if cool > 0.0 {
                        let rows = (16.0 * cool).ceil() as i32;
                        c.shade(x + 1, hy + 1, 16, rows, 1);
                    }
                    if let Some(k) = known {
                        c.tiny(x + 17, hy + 12, &k.level.to_string(), CREAM, INK);
                    }
                }
                let key = self.key([Action::Spell1, Action::Spell2][k]);
                let right = x + 4 * key.len() as i32;
                c.tiny(right, hy + 1, key, CREAM, INK);
            }
        }

        // Name of the selected item, briefly.
        if self.sel_name_t < 1.6 {
            if let Some(s) = p.held_stack() {
                let name = s.name();
                let tw = c.text_width(&name);
                let col = s.rarity().map_or(CREAM, |r| r.color());
                c.text_outline((w - tw) / 2, hy - 14, &name, col, INK);
            }
        } else if let Some(hint) = self
            .hint
            .as_ref()
            .filter(|_| self.fishing.as_ref().and_then(|f| f.showing()).is_none())
        {
            let t = format!("[{}] {hint}", self.key(Action::Interact));
            let tw = c.text_width(&t);
            c.text_outline((w - tw) / 2, hy - 14, &t, WHITE, INK);
        }
        if self.area == Area::Home && self.sel_name_t >= 1.6 && self.cooking.is_none() {
            // What your hands can do about the house.
            let held = p.held().map(|i| i.def().kind);
            let place = self.key(Action::Interact);
            let t = match held {
                Some(Kind::Place(Placeable::Furniture(_))) => Some(format!(
                    "[{place}] Place   [{}] Turn",
                    self.key(Action::Turn)
                )),
                Some(Kind::Place(_)) => Some(format!("[{place}] Place")),
                Some(Kind::Wallpaper(_) | Kind::Flooring(_)) => {
                    Some(format!("[{place}] Redecorate"))
                }
                _ if self.target_ok => Some(format!("[{}] Pick up", self.key(Action::Use))),
                _ => None,
            };
            if let Some(t) = t {
                let y = if self.hint.is_some() {
                    hy - 26
                } else {
                    hy - 14
                };
                let tw = c.text_width(&t);
                c.text_outline((w - tw) / 2, y, &t, CREAM, INK);
            }
        }

        // Pickup toasts.
        let mut ty = h - 36;
        for t in self.toasts.iter().rev() {
            let alpha_out = t.t > 2.6;
            if alpha_out && (t.t * 10.0) as i32 % 2 == 0 {
                ty -= 13;
                continue;
            }
            let label = match t.icon {
                Some(_) if t.n > 1 => format!("{} x{}", t.text, t.n),
                _ => t.text.clone(),
            };
            let tw = c.text_width(&label) + if t.icon.is_some() { 20 } else { 8 };
            c.panel(3, ty - 2, tw, 13, Style::Dark);
            let mut x = 7;
            if let Some(i) = t.icon {
                let icon = a.tex(a.icon(i.def().icon));
                // Small icons as they are; big ones at half size.
                let step = if icon.w > 8 { 2 } else { 1 };
                for sy in 0..8 {
                    for sx in 0..8 {
                        let col = icon.get(sx * step, sy * step);
                        if col != CLEAR {
                            c.px(x + sx, ty + 1 + sy, col);
                        }
                    }
                }
                x += 12;
            }
            c.text_shadow(x, ty, &label, t.color, INK);
            ty -= 14;
        }

        // Area banner.
        if let Some(b) = &self.banner {
            let fade = if b.t < 0.3 {
                b.t / 0.3
            } else if b.t > 2.8 {
                (3.5 - b.t) / 0.7
            } else {
                1.0
            };
            if fade > 0.0 {
                let tw = c.big_width(&b.title, 2);
                let y = h / 4 - 10;
                c.text_big((w - tw) / 2, y, &b.title, 2, CREAM, INK);
                let sw = c.text_width(&b.sub);
                c.text_outline((w - sw) / 2, y + 20, &b.sub, GOLD, INK);
            }
        }
    }

    /// Fishing: the cast power bar, the bite cue, the reeling meter and the catch.
    fn draw_fishing_hud(&self, c: &mut Canvas, a: &Assets, cam: &Camera) {
        let Some(f) = &self.fishing else { return };
        let (w, h) = (c.w(), c.h());
        let feet = cam.project(self.player.world_pos());
        let head = cam.project(self.player.world_pos() + Vec3::Y * 1.3);
        match f.phase {
            Phase::Charge => {
                let Some(s) = feet else { return };
                let (bw, bx, by) = (36, s.x as i32 - 18, s.y as i32 + 8);
                let col = if f.power > 0.9 {
                    GOLD
                } else if f.power > 0.5 {
                    LIME
                } else {
                    GREEN
                };
                c.rect(bx - 1, by - 1, bw + 2, 7, INK);
                c.bar(bx, by, bw, 5, f.power, col, CREAM, SHADOW);
                if f.power > 0.9 {
                    c.tiny(bx + bw + 14, by - 1, "MAX", GOLD, INK);
                }
            }
            Phase::Bite => {
                let Some(s) = head else { return };
                if (self.time * 10.0).fract() < 0.7 {
                    let bw = c.big_width("!", 3);
                    c.text_big(s.x as i32 - bw / 2, s.y as i32 - 30, "!", 3, GOLD, INK);
                }
            }
            Phase::Reel => {
                let Some(s) = feet else { return };
                let Some(Hooked::Fish(item, _)) = f.hooked else {
                    return;
                };
                let th = 96;
                let x0 = (s.x as i32 + 26).clamp(6, w - 34);
                let y0 = (s.y as i32 - th - 12).clamp(18, h - th - 30);
                c.panel(x0 - 4, y0 - 4, 32, th + 8, Style::Dark);
                // The water, with the bar you steer.
                c.rect(x0, y0, 16, th, DEEP_TEAL);
                for k in (4..th).step_by(9) {
                    c.rect(x0 + 2 + (k / 9) % 3, y0 + k, 6, 1, TEAL);
                }
                let (fish_y, zone, zone_h) = fish::reel_bar(f);
                let inside = fish_y >= zone - 0.02 && fish_y <= zone + zone_h + 0.02;
                let zt = y0 + th - ((zone + zone_h) * th as f32) as i32;
                let zh = ((zone_h * th as f32) as i32).max(4);
                let (zc, zhi) = if inside { (LIME, MINT) } else { (GREEN, LIME) };
                c.rect(x0 + 1, zt, 14, zh, zc);
                c.rect(x0 + 1, zt, 14, 1, zhi);
                c.frame(x0 + 1, zt, 14, zh, INK);
                // The fish, wriggling.
                let icon = a.tex(a.icon(item.def().icon));
                let fy = y0 + th - (fish_y * th as f32) as i32 - 4;
                let wig = if (self.time * 8.0).fract() < 0.5 {
                    0
                } else {
                    1
                };
                for sy in 0..8 {
                    for sx in 0..8 {
                        let col = icon.get(sx * 2, sy * 2);
                        if col != CLEAR {
                            c.px(x0 + 4 + sx + wig, fy + sy, col);
                        }
                    }
                }
                // How close you are to landing it.
                let ph = (f.progress.clamp(0.0, 1.0) * th as f32) as i32;
                let pc = if f.progress < 0.25 {
                    if (self.time * 6.0).fract() < 0.5 {
                        RED
                    } else {
                        SALMON
                    }
                } else if f.progress > 0.75 {
                    GOLD
                } else {
                    ORANGE
                };
                c.rect(x0 + 19, y0, 5, th, SHADOW);
                c.rect(x0 + 19, y0 + th - ph, 5, ph, pc);
                c.frame(x0 + 18, y0 - 1, 7, th + 2, INK);
                let tip = "Hold to reel";
                let tw = c.text_width(tip);
                c.text_outline(
                    (x0 + 12 - tw / 2).clamp(2, w - tw - 2),
                    y0 - 16,
                    tip,
                    CREAM,
                    INK,
                );
            }
            Phase::Caught => {
                // Up over the hero's head and the catch held up high, where the hotbar can't
                // cover it however low on the screen the hero stands, kept on the screen.
                let Some(s) = head else { return };
                let p = &self.player;
                let tip = fish::rod_tip(p.world_pos(), p.yaw, f.lift(self.time));
                // (The catch hangs 0.62 under the tip, a sprite half a unit tall standing up
                // towards the camera.)
                let above = cam
                    .project(tip - Vec3::Y * 0.62 + cam.up * 0.5)
                    .map_or(s.y, |q| q.y.min(s.y));
                let Some(Hooked::Fish(item, size)) = f.hooked else {
                    return;
                };
                let rare = fish::fish_def(item).map_or(0, |d| d.rarity);
                let mut tags = Vec::new();
                if f.record {
                    tags.push(("Record!", GOLD));
                } else if f.new {
                    tags.push(("New!", LIME));
                }
                if f.perfect {
                    tags.push(("Perfect!", ORANGE));
                }
                let name = item.def().name;
                let sub = format!("{size} cm  {}", fish::rarity_name(rare));
                let tw = c.text_width(name).max(c.text_width(&sub));
                let x = (s.x as i32).clamp(tw / 2 + 2, w - tw / 2 - 2);
                // The tag on top, then the name, then its size.
                let y = (above as i32 - 36).clamp(26, h - 64);
                if !tags.is_empty() && (self.time * 6.0).fract() < 0.75 {
                    let gap = 8;
                    let all = tags.iter().map(|(t, _)| c.text_width(t)).sum::<i32>()
                        + gap * (tags.len() as i32 - 1);
                    let mut tx = x - all / 2;
                    for (t, col) in tags {
                        c.text_outline(tx, y, t, col, INK);
                        tx += c.text_width(t) + gap;
                    }
                }
                let nw = c.text_width(name);
                c.text_outline(x - nw / 2, y + 11, name, fish::rarity_color(rare), INK);
                let sw = c.text_width(&sub);
                c.text_outline(x - sw / 2, y + 22, &sub, CREAM, INK);
                // What A does with it, just above the hotbar.
                if f.t > fish::SHOW_MIN {
                    let key = self.key(Action::Interact);
                    let (t, col) = if self.player.inv.can_fit(item, 1) {
                        (format!("[{key}] Put it in your bag"), WHITE)
                    } else {
                        (format!("[{key}] Bag's full - throw it back"), SALMON)
                    };
                    let tw = c.text_width(&t);
                    c.text_outline((w - tw) / 2, h - 36, &t, col, INK);
                }
            }
            _ => {}
        }
    }

    /// Bramblewick at a glance: streets, roofs, the stream, and who wants you.
    fn draw_town_map(&self, c: &mut Canvas, x0: i32, y0: i32) {
        let w = &self.town;
        c.rect(x0 - 1, y0 - 1, w.w + 2, w.h + 2, INK);
        for z in 0..w.h {
            for x in 0..w.w {
                let col = match w.obj(x, z) {
                    Some(Obj::Building { id }) => super::town::BUILDINGS[*id as usize].look.roof[1],
                    Some(Obj::Part { ax, az }) => match w.obj(*ax as i32, *az as i32) {
                        Some(Obj::Building { id }) => {
                            super::town::BUILDINGS[*id as usize].look.roof[1]
                        }
                        Some(Obj::Fountain { .. }) => SKY,
                        _ => KHAKI,
                    },
                    Some(Obj::Tree { .. } | Obj::Pine { .. } | Obj::WishTree { .. }) => DEEP_TEAL,
                    Some(Obj::Fountain { .. }) => SKY,
                    _ => match (w.wall(x, z), w.floor(x, z)) {
                        (Wall::Hedge, _) => DEEP_TEAL,
                        (_, Floor::Water) => BLUE,
                        (_, Floor::Street | Floor::Plaza | Floor::Planks) => SAND,
                        (_, Floor::Path | Floor::Sand) => KHAKI,
                        _ => TEAL,
                    },
                };
                c.px(x0 + x, y0 + z, col);
            }
        }
        // Shop doors that are open right now.
        for b in &super::town::BUILDINGS {
            if let Some(p) = b.place {
                if p.is_open(self.clock.min) {
                    let (dx, dz) = b.step();
                    c.px(x0 + dx, y0 + dz - 1, CREAM);
                }
            }
        }
        let blink = (self.time * 3.0).fract() < 0.6;
        for n in &self.folk {
            let (nx, nz) = n.tile();
            let col = match self.marker(n.who) {
                Some(true) => GREEN,
                Some(false) if blink => GOLD,
                _ => PINK,
            };
            c.px(x0 + nx, y0 + nz, col);
            if self.marker(n.who).is_some() {
                c.px(x0 + nx, y0 + nz - 1, col);
            }
        }
        if blink {
            let (px, pz) = self.player.tile();
            c.rect(x0 + px, y0 + pz, 2, 2, WHITE);
        }
    }

    fn draw_minimap(&self, c: &mut Canvas, x0: i32, y0: i32, size: i32) {
        let Some(l) = &self.level else { return };
        let wld = &l.world;
        let scale = 1;
        let (px, pz) = self.player.tile();
        let half = size / 2 / scale;
        c.rect(x0 + 1, y0 + 1, size - 2, size - 2, INK);
        c.frame(x0, y0, size, size, SHADOW);
        for dz in -half + 1..half - 1 {
            for dx in -half + 1..half - 1 {
                let (x, z) = (px + dx, pz + dz);
                if !wld.inside(x, z) {
                    continue;
                }
                let i = (z * wld.w + x) as usize;
                if !self.revealed.get(i).copied().unwrap_or(false) {
                    continue;
                }
                let col = match (wld.wall(x, z), wld.obj(x, z)) {
                    (Wall::None, Some(Obj::StairsDown)) => GOLD,
                    (Wall::None, Some(Obj::Waystone)) => MINT,
                    (Wall::None, _) if wld.floor(x, z) == Floor::Void => continue,
                    (Wall::None, _) if wld.floor(x, z) == Floor::Lava => ORANGE,
                    (Wall::None, _) if wld.floor(x, z) == Floor::Water => {
                        if wld.glowcave {
                            AQUA
                        } else {
                            TEAL
                        }
                    }
                    (Wall::None, _) if wld.floor(x, z) == Floor::Bridge => CLAY,
                    (Wall::None, _) if wld.floor(x, z) == Floor::CopperBridge => AQUA,
                    (
                        Wall::None,
                        Some(Obj::LootChest {
                            opened: false,
                            gleam,
                        }),
                    ) => {
                        if *gleam {
                            GOLD
                        } else {
                            PINK
                        }
                    }
                    (Wall::None, _) => KHAKI,
                    (Wall::Ore(_), _) => CLAY,
                    _ => continue,
                };
                c.px(x0 + half + dx, y0 + half + dz, col);
            }
        }
        // Keepsakes glint on the map so they're easy to find.
        if (self.time * 3.0).fract() < 0.6 {
            for d in &self.drops {
                if d.stack.item.def().kind != super::items::Kind::Keepsake {
                    continue;
                }
                let (dx, dz) = (d.pos.x as i32 - px, d.pos.z as i32 - pz);
                if dx.abs() < half - 1 && dz.abs() < half - 1 {
                    c.px(x0 + half + dx, y0 + half + dz, GOLD);
                    c.px(x0 + half + dx + 1, y0 + half + dz, CREAM);
                }
            }
        }
        for f in &self.foes {
            let (fx, fz) = (f.pos.x as i32 - px, f.pos.y as i32 - pz);
            let i = (f.pos.y as i32 * wld.w + f.pos.x as i32) as usize;
            if fx.abs() < half - 1
                && fz.abs() < half - 1
                && self.revealed.get(i).copied().unwrap_or(false)
            {
                c.px(
                    x0 + half + fx,
                    y0 + half + fz,
                    if f.boss { RED } else { SALMON },
                );
            }
        }
        let blink = (self.time * 4.0).fract() < 0.7;
        if blink {
            c.px(x0 + half, y0 + half, WHITE);
            c.px(x0 + half + 1, y0 + half, WHITE);
            c.px(x0 + half, y0 + half + 1, WHITE);
            c.px(x0 + half + 1, y0 + half + 1, WHITE);
        }
    }
}

/// Tonight's moon as a little disc, lit on the side it's lit. A full moon down in the
/// Hollow gets a red rim: the creatures there are wild tonight.
pub fn draw_moon(c: &mut Canvas, x: i32, y: i32, phase: super::sky::MoonPhase, below: bool) {
    const R: f32 = 4.0;
    let f = phase.light();
    for dy in -4i32..=4 {
        for dx in -4i32..=4 {
            let (fx, fy) = (dx as f32, dy as f32);
            let d2 = fx * fx + fy * fy;
            if d2 > (R + 0.4) * (R + 0.4) {
                continue;
            }
            let edge = d2 > (R - 0.6) * (R - 0.6);
            let half = (R * R - fy * fy).max(0.0).sqrt();
            let term = half * (1.0 - 2.0 * f);
            let lit = if phase.waxing() {
                fx > term
            } else {
                fx < -term
            };
            let col = if edge {
                if phase.full() && below { RED } else { INK }
            } else if lit {
                if (dx + dy * 2) % 5 == 0 { SAND } else { CREAM }
            } else {
                SLATE
            };
            c.px(x + 4 + dx, y + 4 + dy, col);
        }
    }
}
