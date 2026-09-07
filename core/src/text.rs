//! Font loading, glyph caching, line layout with word wrapping, and glyph blitting.
//!
//! Fonts come from the operating system (Segoe UI / Helvetica / DejaVu & friends) or from a
//! path supplied through the API or the `PATINA_FONT*` environment variables.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use fontdue::{Font, FontSettings, Metrics};
use tiny_skia::PixmapMut;

use crate::color::Rgba;
use crate::geom::Rect;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Face {
    Regular = 0,
    Semibold = 1,
    Bold = 2,
    Mono = 3,
}

#[derive(Clone, Copy, Debug)]
pub struct LineMetrics {
    pub ascent: f32,
    pub descent: f32,
    pub line_height: f32,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct GlyphKey {
    face: u8,
    glyph: u16,
    px_q: u32,
}

pub struct Glyph {
    pub metrics: Metrics,
    pub bitmap: Vec<u8>,
}

#[derive(Clone, Copy, Debug)]
pub struct PosGlyph {
    pub glyph: u16,
    /// X offset from the line start.
    pub x: f32,
    pub advance: f32,
    /// Byte offset of the character in the source string.
    pub byte: usize,
    pub ch: char,
}

#[derive(Clone, Debug, Default)]
pub struct Line {
    pub glyphs: Vec<PosGlyph>,
    pub width: f32,
    pub start: usize,
    pub end: usize,
}

#[derive(Clone, Debug)]
pub struct TextLayout {
    pub lines: Vec<Line>,
    pub width: f32,
    pub height: f32,
    pub line_height: f32,
    pub ascent: f32,
    pub px: f32,
    pub face: Face,
}

impl TextLayout {
    /// Line index and x offset of a caret placed before byte `byte`.
    pub fn caret(&self, byte: usize) -> (usize, f32) {
        let mut li = 0;
        for (i, line) in self.lines.iter().enumerate() {
            li = i;
            if byte <= line.end {
                break;
            }
        }
        let line = match self.lines.get(li) {
            Some(l) => l,
            None => return (0, 0.0),
        };
        for g in &line.glyphs {
            if g.byte >= byte {
                return (li, g.x);
            }
        }
        (
            li,
            line.glyphs.last().map(|g| g.x + g.advance).unwrap_or(0.0),
        )
    }

    /// Byte offset of the character boundary closest to `x` on line `li`.
    pub fn hit(&self, li: usize, x: f32) -> usize {
        let line = match self.lines.get(li.min(self.lines.len().saturating_sub(1))) {
            Some(l) => l,
            None => return 0,
        };
        let mut best = line.start;
        let mut best_d = f32::INFINITY;
        for g in &line.glyphs {
            let d = (g.x - x).abs();
            if d < best_d {
                best_d = d;
                best = g.byte;
            }
        }
        if let Some(last) = line.glyphs.last() {
            let end_x = last.x + last.advance;
            if (end_x - x).abs() < best_d {
                best = line.end;
            }
        }
        best
    }
}

enum LoadStatus {
    NotLoaded,
    Loaded,
    Failed(String),
}

pub struct TextSystem {
    fonts: [Option<Font>; 4],
    status: LoadStatus,
    glyphs: HashMap<GlyphKey, Arc<Glyph>>,
    lut: [u8; 256],
    pub loaded_from: Vec<String>,
}

impl Default for TextSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl TextSystem {
    pub fn new() -> TextSystem {
        let mut lut = [0u8; 256];
        // Slight gamma boost keeps thin strokes from looking washed out on light backgrounds.
        for (i, v) in lut.iter_mut().enumerate() {
            *v = ((i as f32 / 255.0).powf(1.0 / 1.18) * 255.0).round() as u8;
        }
        TextSystem {
            fonts: [None, None, None, None],
            status: LoadStatus::NotLoaded,
            glyphs: HashMap::new(),
            lut,
            loaded_from: Vec::new(),
        }
    }

    pub fn ready(&self) -> bool {
        matches!(self.status, LoadStatus::Loaded)
    }

    pub fn error(&self) -> Option<&str> {
        match &self.status {
            LoadStatus::Failed(e) => Some(e.as_str()),
            _ => None,
        }
    }

    /// Load system fonts on first use. Returns whether text can be rendered.
    pub fn ensure_loaded(&mut self) -> bool {
        match self.status {
            LoadStatus::Loaded => true,
            LoadStatus::Failed(_) => false,
            LoadStatus::NotLoaded => match self.load_system() {
                Ok(()) => {
                    self.status = LoadStatus::Loaded;
                    true
                }
                Err(e) => {
                    self.status = LoadStatus::Failed(e);
                    false
                }
            },
        }
    }

    fn load_font(path: &Path, index: u32) -> Result<Font, String> {
        let data = std::fs::read(path).map_err(|e| format!("{}: {}", path.display(), e))?;
        let settings = FontSettings {
            collection_index: index,
            scale: 40.0,
            load_substitutions: true,
        };
        Font::from_bytes(data, settings).map_err(|e| format!("{}: {}", path.display(), e))
    }

    /// Load explicit font files. `regular` is required; others fall back to it.
    pub fn load_files(
        &mut self,
        regular: (&Path, u32),
        semibold: Option<(&Path, u32)>,
        bold: Option<(&Path, u32)>,
        mono: Option<(&Path, u32)>,
    ) -> Result<(), String> {
        let reg = Self::load_font(regular.0, regular.1)?;
        let mut from = vec![regular.0.display().to_string()];
        let mut optional = |spec: Option<(&Path, u32)>| -> Option<Font> {
            let (p, i) = spec?;
            match Self::load_font(p, i) {
                Ok(f) => {
                    from.push(p.display().to_string());
                    Some(f)
                }
                Err(_) => None,
            }
        };
        let sb = optional(semibold);
        let b = optional(bold);
        let m = optional(mono);
        self.fonts = [Some(reg), sb, b, m];
        self.glyphs.clear();
        self.loaded_from = from;
        self.status = LoadStatus::Loaded;
        Ok(())
    }

    fn load_system(&mut self) -> Result<(), String> {
        if let Ok(p) = std::env::var("PATINA_FONT") {
            let bold = std::env::var("PATINA_FONT_BOLD").ok().map(PathBuf::from);
            let semi = std::env::var("PATINA_FONT_SEMIBOLD")
                .ok()
                .map(PathBuf::from);
            let mono = std::env::var("PATINA_FONT_MONO").ok().map(PathBuf::from);
            return self.load_files(
                (Path::new(&p), 0),
                semi.as_deref().map(|p| (p, 0)),
                bold.as_deref().map(|p| (p, 0)),
                mono.as_deref().map(|p| (p, 0)),
            );
        }
        let mut tried = Vec::new();
        for set in system_candidates() {
            let (reg_path, reg_idx) = &set[0];
            if !reg_path.exists() {
                tried.push(reg_path.display().to_string());
                continue;
            }
            let opt = |i: usize| -> Option<(&Path, u32)> {
                let (p, idx) = &set[i];
                if p.as_os_str().is_empty() || !p.exists() {
                    None
                } else {
                    Some((p.as_path(), *idx))
                }
            };
            match self.load_files((reg_path.as_path(), *reg_idx), opt(1), opt(2), opt(3)) {
                Ok(()) => return Ok(()),
                Err(e) => tried.push(e),
            }
        }
        Err(format!(
            "no usable system font found (set PATINA_FONT to a .ttf path). Tried: {}",
            tried.join(", ")
        ))
    }

    fn font(&self, face: Face) -> &Font {
        let idx = face as usize;
        let chain: &[usize] = match face {
            Face::Regular => &[0],
            Face::Semibold => &[1, 2, 0],
            Face::Bold => &[2, 1, 0],
            Face::Mono => &[3, 0],
        };
        for &i in chain {
            if let Some(f) = &self.fonts[i] {
                return f;
            }
        }
        let _ = idx;
        self.fonts[0].as_ref().expect("fonts not loaded")
    }

    /// Which slot a face actually resolves to (for cache keys).
    fn face_slot(&self, face: Face) -> u8 {
        let chain: &[usize] = match face {
            Face::Regular => &[0],
            Face::Semibold => &[1, 2, 0],
            Face::Bold => &[2, 1, 0],
            Face::Mono => &[3, 0],
        };
        for &i in chain {
            if self.fonts[i].is_some() {
                return i as u8;
            }
        }
        0
    }

    pub fn face_for(weight: u16, family: i64) -> Face {
        if family == 1 {
            Face::Mono
        } else if weight >= 700 {
            Face::Bold
        } else if weight >= 500 {
            Face::Semibold
        } else {
            Face::Regular
        }
    }

    pub fn line_metrics(&self, face: Face, px: f32) -> LineMetrics {
        let font = self.font(face);
        match font.horizontal_line_metrics(px) {
            Some(m) => {
                let ascent = m.ascent;
                let descent = -m.descent;
                let line_height = (m.new_line_size).max(ascent + descent).max(px * 1.15);
                LineMetrics {
                    ascent,
                    descent,
                    line_height: line_height.round(),
                }
            }
            None => LineMetrics {
                ascent: px * 0.8,
                descent: px * 0.2,
                line_height: (px * 1.3).round(),
            },
        }
    }

    fn glyph(&mut self, face: Face, glyph: u16, px: f32) -> Arc<Glyph> {
        let key = GlyphKey {
            face: self.face_slot(face),
            glyph,
            px_q: (px * 4.0).round() as u32,
        };
        if let Some(g) = self.glyphs.get(&key) {
            return g.clone();
        }
        let px_r = key.px_q as f32 / 4.0;
        let (metrics, bitmap) = self.font(face).rasterize_indexed(glyph, px_r);
        let g = Arc::new(Glyph { metrics, bitmap });
        if self.glyphs.len() > 4096 {
            self.glyphs.clear();
        }
        self.glyphs.insert(key, g.clone());
        g
    }

    /// Lay out `text` at `px` pixels. `max_width` may be infinite. When `wrap` is false the
    /// text is a single line per paragraph. A non-positive finite `max_width` wraps at every
    /// word boundary (min-content measurement) without breaking words.
    pub fn layout(
        &mut self,
        text: &str,
        face: Face,
        px: f32,
        max_width: f32,
        wrap: bool,
    ) -> TextLayout {
        let lm = self.line_metrics(face, px);
        let font = self.font(face);
        let allow_char_break = max_width > 0.0;
        let space_adv = font
            .metrics_indexed(font.lookup_glyph_index(' '), px)
            .advance_width;
        let mut lines: Vec<Line> = Vec::new();
        let mut offset = 0usize;

        for para in text.split('\n') {
            let start = offset;
            let mut line = Line {
                start,
                end: start,
                ..Default::default()
            };
            let mut x = 0.0f32;
            let mut prev: Option<u16> = None;
            // (glyph index after the break, byte after the whitespace)
            let mut last_break: Option<(usize, usize)> = None;

            for (i, ch) in para.char_indices() {
                let byte = start + i;
                let gid = font.lookup_glyph_index(ch);
                let mut adv = font.metrics_indexed(gid, px).advance_width;
                if ch == '\t' {
                    adv = space_adv * 4.0;
                }
                let mut kern = prev
                    .and_then(|p| font.horizontal_kern_indexed(p, gid, px))
                    .unwrap_or(0.0);
                let mut gx = x + kern;

                if wrap
                    && max_width.is_finite()
                    && gx + adv > max_width
                    && !line.glyphs.is_empty()
                    && !ch.is_whitespace()
                {
                    if let Some((gi, bbyte)) = last_break {
                        let rest = line.glyphs.split_off(gi);
                        line.end = bbyte;
                        finish_line(&mut line);
                        lines.push(std::mem::take(&mut line));
                        line.start = rest.first().map(|g| g.byte).unwrap_or(byte);
                        let shift = rest.first().map(|g| g.x).unwrap_or(0.0);
                        line.glyphs = rest
                            .into_iter()
                            .map(|mut g| {
                                g.x -= shift;
                                g
                            })
                            .collect();
                        x = line.glyphs.last().map(|g| g.x + g.advance).unwrap_or(0.0);
                        prev = line.glyphs.last().map(|g| g.glyph);
                        last_break = None;
                        kern = prev
                            .and_then(|p| font.horizontal_kern_indexed(p, gid, px))
                            .unwrap_or(0.0);
                        gx = x + kern;
                    } else if allow_char_break {
                        line.end = byte;
                        finish_line(&mut line);
                        lines.push(std::mem::take(&mut line));
                        line.start = byte;
                        gx = 0.0;
                    }
                }

                line.glyphs.push(PosGlyph {
                    glyph: gid,
                    x: gx,
                    advance: adv,
                    byte,
                    ch,
                });
                x = gx + adv;
                prev = Some(gid);
                if ch.is_whitespace() {
                    last_break = Some((line.glyphs.len(), byte + ch.len_utf8()));
                }
            }
            line.end = start + para.len();
            finish_line(&mut line);
            lines.push(line);
            offset += para.len() + 1;
        }

        let width = lines.iter().map(|l| l.width).fold(0.0, f32::max);
        let height = lines.len() as f32 * lm.line_height;
        TextLayout {
            lines,
            width,
            height,
            line_height: lm.line_height,
            ascent: lm.ascent,
            px,
            face,
        }
    }

    pub fn measure(
        &mut self,
        text: &str,
        face: Face,
        px: f32,
        max_width: f32,
        wrap: bool,
    ) -> (f32, f32) {
        let l = self.layout(text, face, px, max_width, wrap);
        (l.width, l.height)
    }

    /// Draw a layout with its top-left at (x, y). `box_w` is used for center/right alignment.
    pub fn draw(
        &mut self,
        pm: &mut PixmapMut,
        layout: &TextLayout,
        x: f32,
        y: f32,
        box_w: f32,
        align: i64,
        color: Rgba,
        clip: Rect,
    ) {
        if !color.is_visible() {
            return;
        }
        let lut = self.lut;
        for (li, line) in layout.lines.iter().enumerate() {
            let lx = match align {
                1 => x + (box_w - line.width) * 0.5,
                2 => x + box_w - line.width,
                _ => x,
            };
            let baseline = y + li as f32 * layout.line_height + layout.ascent;
            let by = baseline.round() as i32;
            if (by as f32) < clip.y - layout.line_height
                || by as f32 > clip.bottom() + layout.line_height
            {
                continue;
            }
            for g in &line.glyphs {
                if g.ch.is_whitespace() {
                    continue;
                }
                let glyph = self.glyph(layout.face, g.glyph, layout.px);
                if glyph.metrics.width == 0 || glyph.metrics.height == 0 {
                    continue;
                }
                let gx = (lx + g.x).round() as i32 + glyph.metrics.xmin;
                let gy = by - (glyph.metrics.height as i32 + glyph.metrics.ymin);
                blit_glyph(pm, &glyph, gx, gy, color, clip, &lut);
            }
        }
    }
}

fn finish_line(line: &mut Line) {
    let mut w = 0.0f32;
    for g in line.glyphs.iter().rev() {
        if !g.ch.is_whitespace() {
            w = g.x + g.advance;
            break;
        }
    }
    line.width = w;
}

/// Premultiplied source-over of a coverage bitmap onto RGBA8 pixels.
fn blit_glyph(
    pm: &mut PixmapMut,
    g: &Glyph,
    x0: i32,
    y0: i32,
    color: Rgba,
    clip: Rect,
    lut: &[u8; 256],
) {
    let pw = pm.width() as i32;
    let ph = pm.height() as i32;
    let cx0 = clip.x.max(0.0).floor() as i32;
    let cy0 = clip.y.max(0.0).floor() as i32;
    let cx1 = clip.right().min(pw as f32).ceil() as i32;
    let cy1 = clip.bottom().min(ph as f32).ceil() as i32;
    if cx1 <= cx0 || cy1 <= cy0 {
        return;
    }
    let data = pm.data_mut();
    let (cr, cg, cb, ca) = (
        color.r * 255.0,
        color.g * 255.0,
        color.b * 255.0,
        color.a.clamp(0.0, 1.0),
    );
    let gw = g.metrics.width;
    for row in 0..g.metrics.height {
        let py = y0 + row as i32;
        if py < cy0 || py >= cy1 {
            continue;
        }
        let src_row = &g.bitmap[row * gw..(row + 1) * gw];
        for (col, &cov) in src_row.iter().enumerate() {
            let px = x0 + col as i32;
            if px < cx0 || px >= cx1 || cov == 0 {
                continue;
            }
            let a = lut[cov as usize] as f32 / 255.0 * ca;
            if a <= 0.0 {
                continue;
            }
            let inv = 1.0 - a;
            let i = ((py * pw + px) * 4) as usize;
            data[i] = (cr * a + data[i] as f32 * inv + 0.5) as u8;
            data[i + 1] = (cg * a + data[i + 1] as f32 * inv + 0.5) as u8;
            data[i + 2] = (cb * a + data[i + 2] as f32 * inv + 0.5) as u8;
            data[i + 3] = (255.0 * a + data[i + 3] as f32 * inv + 0.5) as u8;
        }
    }
}

/// Candidate font sets: [regular, semibold, bold, mono] as (path, collection index).
fn system_candidates() -> Vec<[(PathBuf, u32); 4]> {
    let mut out = Vec::new();
    let p = |s: &str| (PathBuf::from(s), 0u32);
    let none = || (PathBuf::new(), 0u32);

    #[cfg(target_os = "windows")]
    {
        let dir = std::env::var("WINDIR")
            .map(|w| format!("{}\\Fonts", w))
            .unwrap_or_else(|_| "C:\\Windows\\Fonts".into());
        let f = |n: &str| (PathBuf::from(format!("{}\\{}", dir, n)), 0u32);
        out.push([
            f("segoeui.ttf"),
            f("seguisb.ttf"),
            f("segoeuib.ttf"),
            f("consola.ttf"),
        ]);
        out.push([f("arial.ttf"), none(), f("arialbd.ttf"), f("cour.ttf")]);
        out.push([f("tahoma.ttf"), none(), f("tahomabd.ttf"), f("lucon.ttf")]);
    }
    #[cfg(target_os = "macos")]
    {
        out.push([
            (PathBuf::from("/System/Library/Fonts/Helvetica.ttc"), 0),
            none(),
            (PathBuf::from("/System/Library/Fonts/Helvetica.ttc"), 1),
            (PathBuf::from("/System/Library/Fonts/Menlo.ttc"), 0),
        ]);
        out.push([
            p("/System/Library/Fonts/Supplemental/Arial.ttf"),
            none(),
            p("/System/Library/Fonts/Supplemental/Arial Bold.ttf"),
            p("/System/Library/Fonts/Supplemental/Courier New.ttf"),
        ]);
        out.push([
            p("/Library/Fonts/Arial.ttf"),
            none(),
            p("/Library/Fonts/Arial Bold.ttf"),
            none(),
        ]);
    }
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    {
        let dirs = [
            "/usr/share/fonts/truetype/dejavu",
            "/usr/share/fonts/TTF",
            "/usr/share/fonts/dejavu",
            "/usr/share/fonts/truetype/noto",
            "/usr/share/fonts/noto",
            "/usr/share/fonts/truetype/liberation",
            "/usr/share/fonts/liberation",
            "/usr/share/fonts/truetype/ubuntu",
            "/usr/share/fonts/ubuntu",
            "/usr/share/fonts/truetype/freefont",
            "/usr/local/share/fonts",
        ];
        for d in dirs {
            let f = |n: &str| (PathBuf::from(format!("{}/{}", d, n)), 0u32);
            out.push([
                f("DejaVuSans.ttf"),
                none(),
                f("DejaVuSans-Bold.ttf"),
                f("DejaVuSansMono.ttf"),
            ]);
            out.push([
                f("NotoSans-Regular.ttf"),
                f("NotoSans-SemiBold.ttf"),
                f("NotoSans-Bold.ttf"),
                f("NotoSansMono-Regular.ttf"),
            ]);
            out.push([
                f("LiberationSans-Regular.ttf"),
                none(),
                f("LiberationSans-Bold.ttf"),
                f("LiberationMono-Regular.ttf"),
            ]);
            out.push([
                f("Ubuntu-R.ttf"),
                f("Ubuntu-M.ttf"),
                f("Ubuntu-B.ttf"),
                f("UbuntuMono-R.ttf"),
            ]);
            out.push([
                f("FreeSans.ttf"),
                none(),
                f("FreeSansBold.ttf"),
                f("FreeMono.ttf"),
            ]);
        }
    }
    let _ = p;
    out
}
