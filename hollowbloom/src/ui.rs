//! 2D drawing straight into the palette framebuffer: text, panels, sprites, bars and fades.
//! Everything here writes exact palette indices, so the UI is palette-locked by construction.

use crate::assets::font::{Font, tiny_glyph};
use crate::palette::*;
use crate::render::{Frame, Texture};

pub struct Canvas<'a> {
    pub fb: &'a mut Frame,
    pub font: &'a Font,
    pub darken: &'a [[u8; 32]; 3],
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Style {
    /// Warm parchment window for menus.
    Paper,
    /// Dark, see-through box for the HUD.
    Dark,
    /// A slot or inset well.
    Inset,
}

impl<'a> Canvas<'a> {
    pub fn w(&self) -> i32 {
        self.fb.w as i32
    }

    pub fn h(&self) -> i32 {
        self.fb.h as i32
    }

    #[inline]
    pub fn px(&mut self, x: i32, y: i32, c: u8) {
        if x >= 0 && y >= 0 && x < self.fb.w as i32 && y < self.fb.h as i32 {
            self.fb.color[y as usize * self.fb.w + x as usize] = c;
        }
    }

    pub fn rect(&mut self, x: i32, y: i32, w: i32, h: i32, c: u8) {
        let x0 = x.max(0);
        let y0 = y.max(0);
        let x1 = (x + w).min(self.fb.w as i32);
        let y1 = (y + h).min(self.fb.h as i32);
        for yy in y0..y1 {
            let row = yy as usize * self.fb.w;
            self.fb.color[row + x0 as usize..row + x1.max(x0) as usize].fill(c);
        }
    }

    pub fn frame(&mut self, x: i32, y: i32, w: i32, h: i32, c: u8) {
        self.rect(x, y, w, 1, c);
        self.rect(x, y + h - 1, w, 1, c);
        self.rect(x, y, 1, h, c);
        self.rect(x + w - 1, y, 1, h, c);
    }

    /// Darkens what is already there (translucent overlay).
    pub fn shade(&mut self, x: i32, y: i32, w: i32, h: i32, k: usize) {
        let t = self.darken[k.min(2)];
        let x0 = x.max(0);
        let y0 = y.max(0);
        let x1 = (x + w).min(self.fb.w as i32);
        let y1 = (y + h).min(self.fb.h as i32);
        for yy in y0..y1 {
            for xx in x0..x1 {
                let i = yy as usize * self.fb.w + xx as usize;
                let c = self.fb.color[i];
                if c < 32 {
                    self.fb.color[i] = t[c as usize];
                }
            }
        }
    }

    /// Ordered-dither fade of the whole screen towards a colour (0 = none, 1 = solid).
    pub fn fade(&mut self, amount: f32, c: u8) {
        if amount <= 0.0 {
            return;
        }
        for y in 0..self.fb.h {
            for x in 0..self.fb.w {
                if bayer(x, y) < amount {
                    self.fb.color[y * self.fb.w + x] = c;
                }
            }
        }
    }

    pub fn panel(&mut self, x: i32, y: i32, w: i32, h: i32, style: Style) {
        match style {
            Style::Paper => {
                self.rect(x + 1, y + 1, w - 2, h - 2, SAND);
                self.rect(x + 2, y + h - 3, w - 4, 1, KHAKI);
                self.rect(x + 2, y + 2, w - 4, 1, CREAM);
                // Rounded ink border with a warm inner line.
                self.rect(x + 2, y, w - 4, 1, INK);
                self.rect(x + 2, y + h - 1, w - 4, 1, INK);
                self.rect(x, y + 2, 1, h - 4, INK);
                self.rect(x + w - 1, y + 2, 1, h - 4, INK);
                self.px(x + 1, y + 1, INK);
                self.px(x + w - 2, y + 1, INK);
                self.px(x + 1, y + h - 2, INK);
                self.px(x + w - 2, y + h - 2, INK);
                self.rect(x + 2, y + 1, w - 4, 1, CLAY);
                self.rect(x + 2, y + h - 2, w - 4, 1, RUST);
                self.rect(x + 1, y + 2, 1, h - 4, CLAY);
                self.rect(x + w - 2, y + 2, 1, h - 4, RUST);
            }
            Style::Dark => {
                self.shade(x + 1, y + 1, w - 2, h - 2, 1);
                self.rect(x + 1, y, w - 2, 1, INK);
                self.rect(x + 1, y + h - 1, w - 2, 1, INK);
                self.rect(x, y + 1, 1, h - 2, INK);
                self.rect(x + w - 1, y + 1, 1, h - 2, INK);
            }
            Style::Inset => {
                self.rect(x, y, w, h, KHAKI);
                self.rect(x, y, w, 1, ROSEWOOD);
                self.rect(x, y, 1, h, ROSEWOOD);
                self.rect(x + 1, y + h - 1, w - 1, 1, SAND);
                self.rect(x + w - 1, y + 1, 1, h - 1, SAND);
            }
        }
    }

    /// Draws text, returns its width. Supports `\n`.
    pub fn text(&mut self, x: i32, y: i32, s: &str, c: u8) -> i32 {
        let mut cx = x;
        let mut cy = y;
        let mut maxw = 0;
        for ch in s.chars() {
            if ch == '\n' {
                maxw = maxw.max(cx - x);
                cx = x;
                cy += self.font.line_h;
                continue;
            }
            if let Some(g) = self.font.glyph(ch) {
                for (row, bits) in g.rows.iter().enumerate() {
                    if *bits == 0 {
                        continue;
                    }
                    for col in 0..g.w as i32 {
                        if bits & (1 << col) != 0 {
                            self.px(cx + col, cy + row as i32, c);
                        }
                    }
                }
            }
            cx += self.font.advance(ch);
        }
        maxw.max(cx - x - 1)
    }

    /// Text with a drop shadow one pixel down.
    pub fn text_shadow(&mut self, x: i32, y: i32, s: &str, c: u8, shadow: u8) -> i32 {
        self.text(x, y + 1, s, shadow);
        self.text(x, y, s, c)
    }

    /// Text with a full 1px outline (readable over the 3D scene).
    pub fn text_outline(&mut self, x: i32, y: i32, s: &str, c: u8, outline: u8) -> i32 {
        for (dx, dy) in [
            (-1, 0),
            (1, 0),
            (0, -1),
            (0, 1),
            (1, 1),
            (-1, 1),
            (1, -1),
            (-1, -1),
        ] {
            self.text(x + dx, y + dy, s, outline);
        }
        self.text(x, y, s, c)
    }

    pub fn text_center(&mut self, cx: i32, y: i32, s: &str, c: u8) {
        let w = self.font.width(s);
        self.text(cx - w / 2, y, s, c);
    }

    pub fn text_width(&self, s: &str) -> i32 {
        self.font.width(s)
    }

    /// Wrapped text inside a width; returns the height used.
    pub fn paragraph(&mut self, x: i32, y: i32, w: i32, s: &str, c: u8) -> i32 {
        let lines = self.font.wrap(s, w);
        for (i, l) in lines.iter().enumerate() {
            self.text(x, y + i as i32 * self.font.line_h, l, c);
        }
        lines.len() as i32 * self.font.line_h
    }

    /// 3x5 digits with an outline, right-aligned at `right`.
    pub fn tiny(&mut self, right: i32, y: i32, s: &str, c: u8, outline: u8) {
        let w = s.chars().count() as i32 * 4 - 1;
        let x0 = right - w;
        for pass in 0..2 {
            for (i, ch) in s.chars().enumerate() {
                let Some(rows) = tiny_glyph(ch) else { continue };
                let gx = x0 + i as i32 * 4;
                for (ry, bits) in rows.iter().enumerate() {
                    for col in 0..3 {
                        if bits & (1 << col) != 0 {
                            let (px, py) = (gx + col, y + ry as i32);
                            if pass == 0 {
                                for (dx, dy) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
                                    self.px(px + dx, py + dy, outline);
                                }
                            } else {
                                self.px(px, py, c);
                            }
                        }
                    }
                }
            }
        }
    }

    pub fn sprite(&mut self, t: &Texture, x: i32, y: i32) {
        for sy in 0..t.h as i32 {
            for sx in 0..t.w as i32 {
                let c = t.get(sx, sy);
                if c != CLEAR {
                    self.px(x + sx, y + sy, c);
                }
            }
        }
    }

    /// Sprite with every opaque pixel mapped through `f` (greyed out, silhouettes).
    pub fn sprite_map(&mut self, t: &Texture, x: i32, y: i32, f: impl Fn(u8) -> u8) {
        for sy in 0..t.h as i32 {
            for sx in 0..t.w as i32 {
                let c = t.get(sx, sy);
                if c != CLEAR {
                    self.px(x + sx, y + sy, f(c));
                }
            }
        }
    }

    /// Horizontal bar with an ink border.
    #[allow(clippy::too_many_arguments)]
    pub fn bar(&mut self, x: i32, y: i32, w: i32, h: i32, frac: f32, fill: u8, hi: u8, back: u8) {
        self.frame(x, y, w, h, INK);
        self.rect(x + 1, y + 1, w - 2, h - 2, back);
        let fw = ((w - 2) as f32 * frac.clamp(0.0, 1.0)).round() as i32;
        if fw > 0 {
            self.rect(x + 1, y + 1, fw, h - 2, fill);
            self.rect(x + 1, y + 1, fw, 1, hi);
        }
    }

    /// Big text drawn at an integer scale (titles).
    pub fn text_big(&mut self, x: i32, y: i32, s: &str, scale: i32, c: u8, outline: u8) {
        let mut cx = x;
        for pass in 0..2 {
            cx = x;
            for ch in s.chars() {
                if let Some(g) = self.font.glyph(ch) {
                    for (row, bits) in g.rows.iter().enumerate() {
                        for col in 0..g.w as i32 {
                            if bits & (1 << col) != 0 {
                                let px = cx + col * scale;
                                let py = y + row as i32 * scale;
                                if pass == 0 {
                                    self.rect(px - 1, py - 1, scale + 2, scale + 2, outline);
                                    self.rect(px, py + scale, scale, 1, outline);
                                } else {
                                    self.rect(px, py, scale, scale, c);
                                }
                            }
                        }
                    }
                }
                cx += self.font.advance(ch) * scale;
            }
        }
        let _ = cx;
    }

    pub fn big_width(&self, s: &str, scale: i32) -> i32 {
        self.font.width(s) * scale
    }
}
