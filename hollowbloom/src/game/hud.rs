//! The heads-up display.

use glam::Vec3;

use super::foes::boss_name;
use super::gear::Class;
use super::menus::Menu;
use super::play::Play;
use super::player::HOTBAR;
use super::tips::{draw_money, money_width};
use super::world::{Area, Floor, Obj, Wall};
use crate::assets::Assets;
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
        // Enemy health bars once they are hurt.
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
                }
            }
        }
        if !matches!(self.menu, Menu::None | Menu::Dialog { .. }) {
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
        let icon = match self.area {
            Area::Hollow { .. } => "◆",
            _ if self.rain => "●",
            _ if self.clock.min > 1170.0 => "★",
            _ => "●",
        };
        let icon_col = match self.area {
            Area::Hollow { .. } => LAVENDER,
            _ if self.rain => SKY,
            _ if self.clock.min > 1170.0 => CREAM,
            _ => GOLD,
        };
        c.text(px + pw - 12, 6, icon, icon_col);
        c.text(px + 6, 16, &self.clock.label(), RUST);
        let mw = money_width(c, self.money);
        draw_money(c, a, px + pw - 6 - mw, 26, self.money, INK);
        if let Area::Hollow { depth } = self.area {
            c.panel(px, 40, pw, 14, Style::Dark);
            c.text_shadow(px + 6, 43, &format!("Floor {depth}"), CREAM, INK);
            if self.show_map {
                self.draw_minimap(c, px, 56, pw);
            }
        }

        // Boss bar.
        if let Some(b) = self.foes.iter().find(|f| f.boss && f.alert) {
            let bw = (w / 2).min(200);
            let bx = (w - bw) / 2;
            c.text_outline(
                bx + (bw - c.text_width(boss_name(b.foe))) / 2,
                5,
                boss_name(b.foe),
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
        // Name of the selected item, briefly.
        if self.sel_name_t < 1.6 {
            if let Some(s) = p.held_stack() {
                let name = s.name();
                let tw = c.text_width(&name);
                let col = s.rarity().map_or(CREAM, |r| r.color());
                c.text_outline((w - tw) / 2, hy - 14, &name, col, INK);
            }
        } else if let Some(hint) = &self.hint {
            let t = format!("[E] {hint}");
            let tw = c.text_width(&t);
            c.text_outline((w - tw) / 2, hy - 14, &t, WHITE, INK);
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
                    (Wall::None, _) if wld.floor(x, z) == Floor::Lava => ORANGE,
                    (Wall::None, Some(Obj::LootChest { opened: false })) => PINK,
                    (Wall::None, _) => KHAKI,
                    (Wall::Ore(_), _) => CLAY,
                    _ => continue,
                };
                c.px(x0 + half + dx, y0 + half + dz, col);
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
