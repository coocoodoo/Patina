//! A small vector painter over tiny-skia: rounded rectangles, arcs, strokes, gradients,
//! images, cached blurred shadows, a clip stack and a translation offset used by slide and
//! lift animations.

use std::collections::HashMap;
use std::sync::Arc;

use tiny_skia::{
    BlendMode, FillRule, FilterQuality, GradientStop, LineCap, LineJoin, LinearGradient, Mask,
    Paint, Path, PathBuilder, Pixmap, PixmapMut, PixmapPaint, Point, SpreadMode, Stroke, Transform,
};

use crate::color::Rgba;
use crate::geom::Rect;

const KAPPA: f32 = 0.552_284_8;

/// Build a rounded-rectangle path. Radius is clamped to half the shorter side.
pub fn rrect_path(r: Rect, radius: f32) -> Option<Path> {
    if r.w <= 0.0 || r.h <= 0.0 {
        return None;
    }
    let rad = radius.max(0.0).min(r.w * 0.5).min(r.h * 0.5);
    if rad < 0.5 {
        return Some(PathBuilder::from_rect(r.to_skia()?));
    }
    let k = KAPPA * rad;
    let (x0, y0, x1, y1) = (r.x, r.y, r.right(), r.bottom());
    let mut pb = PathBuilder::new();
    pb.move_to(x0 + rad, y0);
    pb.line_to(x1 - rad, y0);
    pb.cubic_to(x1 - rad + k, y0, x1, y0 + rad - k, x1, y0 + rad);
    pb.line_to(x1, y1 - rad);
    pb.cubic_to(x1, y1 - rad + k, x1 - rad + k, y1, x1 - rad, y1);
    pb.line_to(x0 + rad, y1);
    pb.cubic_to(x0 + rad - k, y1, x0, y1 - rad + k, x0, y1 - rad);
    pb.line_to(x0, y0 + rad);
    pb.cubic_to(x0, y0 + rad - k, x0 + rad - k, y0, x0 + rad, y0);
    pb.close();
    pb.finish()
}

/// An arc of a circle as cubic segments. Angles are degrees, clockwise from 3 o'clock.
pub fn arc_path(cx: f32, cy: f32, r: f32, start_deg: f32, sweep_deg: f32) -> Option<Path> {
    if r <= 0.0 || sweep_deg.abs() < 0.01 {
        return None;
    }
    let segments = ((sweep_deg.abs() / 90.0).ceil() as usize).max(1);
    let step = sweep_deg.to_radians() / segments as f32;
    let k = 4.0 / 3.0 * (step / 4.0).tan();
    let mut a = start_deg.to_radians();
    let mut pb = PathBuilder::new();
    pb.move_to(cx + r * a.cos(), cy + r * a.sin());
    for _ in 0..segments {
        let b = a + step;
        let (sa, ca) = a.sin_cos();
        let (sb, cb) = b.sin_cos();
        pb.cubic_to(
            cx + r * (ca - k * sa),
            cy + r * (sa + k * ca),
            cx + r * (cb + k * sb),
            cy + r * (sb - k * cb),
            cx + r * cb,
            cy + r * sb,
        );
        a = b;
    }
    pb.finish()
}

struct Clip {
    mask: Mask,
    rect: Rect,
}

pub struct Painter<'a> {
    pub pm: PixmapMut<'a>,
    pub scale: f32,
    /// Global alpha multiplier (subtree opacity / disabled state).
    pub alpha: f32,
    /// Translation applied to everything drawn (slide-in and lift animations).
    pub offset: (f32, f32),
    clips: Vec<Clip>,
}

impl<'a> Painter<'a> {
    pub fn new(pm: PixmapMut<'a>, scale: f32) -> Painter<'a> {
        Painter {
            pm,
            scale,
            alpha: 1.0,
            offset: (0.0, 0.0),
            clips: Vec::new(),
        }
    }

    pub fn width(&self) -> f32 {
        self.pm.width() as f32
    }

    pub fn height(&self) -> f32 {
        self.pm.height() as f32
    }

    /// Logical-to-physical helper.
    pub fn s(&self, v: f32) -> f32 {
        v * self.scale
    }

    fn tr(&self) -> Transform {
        Transform::from_translate(self.offset.0, self.offset.1)
    }

    /// A rect moved by the current offset (device space).
    pub fn shifted(&self, r: Rect) -> Rect {
        Rect::new(r.x + self.offset.0, r.y + self.offset.1, r.w, r.h)
    }

    /// The active clip rectangle in device space.
    pub fn clip_rect(&self) -> Rect {
        self.clips.last().map(|c| c.rect).unwrap_or(Rect::new(
            0.0,
            0.0,
            self.width(),
            self.height(),
        ))
    }

    pub fn push_clip(&mut self, r: Rect, radius: f32) {
        let r = self.shifted(r);
        let rect = self.clip_rect().intersect(&r);
        let path = rrect_path(r, radius);
        let mask = match self.clips.last() {
            Some(c) => {
                let mut m = c.mask.clone();
                match &path {
                    Some(p) => m.intersect_path(p, FillRule::Winding, true, Transform::identity()),
                    None => m.clear(),
                }
                m
            }
            None => {
                let mut m = Mask::new(self.pm.width(), self.pm.height()).expect("mask");
                if let Some(p) = &path {
                    m.fill_path(p, FillRule::Winding, true, Transform::identity());
                }
                m
            }
        };
        self.clips.push(Clip { mask, rect });
    }

    pub fn pop_clip(&mut self) {
        self.clips.pop();
    }

    fn paint_for(&self, color: Rgba) -> Paint<'static> {
        let mut p = Paint::default();
        p.set_color(color.mul_alpha(self.alpha).to_skia());
        p.anti_alias = true;
        p
    }

    /// Fill a path given in un-offset coordinates.
    pub fn fill_path(&mut self, path: &Path, color: Rgba) {
        if !color.is_visible() || self.alpha <= 0.0 {
            return;
        }
        let paint = self.paint_for(color);
        let tr = self.tr();
        let mask = self.clips.last().map(|c| &c.mask);
        self.pm.fill_path(path, &paint, FillRule::Winding, tr, mask);
    }

    /// Stroke a path given in un-offset coordinates.
    pub fn stroke_path(&mut self, path: &Path, width: f32, color: Rgba, round: bool) {
        if !color.is_visible() || width <= 0.0 || self.alpha <= 0.0 {
            return;
        }
        let paint = self.paint_for(color);
        let stroke = Stroke {
            width,
            line_cap: if round { LineCap::Round } else { LineCap::Butt },
            line_join: if round {
                LineJoin::Round
            } else {
                LineJoin::Miter
            },
            ..Stroke::default()
        };
        let tr = self.tr();
        let mask = self.clips.last().map(|c| &c.mask);
        self.pm.stroke_path(path, &paint, &stroke, tr, mask);
    }

    pub fn fill_rrect(&mut self, r: Rect, radius: f32, color: Rgba) {
        if let Some(path) = rrect_path(r, radius) {
            self.fill_path(&path, color);
        }
    }

    fn fill_gradient(&mut self, r: Rect, radius: f32, start: Point, end: Point, a: Rgba, b: Rgba) {
        if self.alpha <= 0.0 {
            return;
        }
        let r = self.shifted(r);
        let path = match rrect_path(r, radius) {
            Some(p) => p,
            None => return,
        };
        let shader = LinearGradient::new(
            Point::from_xy(start.x + self.offset.0, start.y + self.offset.1),
            Point::from_xy(end.x + self.offset.0, end.y + self.offset.1),
            vec![
                GradientStop::new(0.0, a.mul_alpha(self.alpha).to_skia()),
                GradientStop::new(1.0, b.mul_alpha(self.alpha).to_skia()),
            ],
            SpreadMode::Pad,
            Transform::identity(),
        );
        let mut paint = Paint::default();
        paint.anti_alias = true;
        match shader {
            Some(s) => paint.shader = s,
            None => paint.set_color(a.mul_alpha(self.alpha).to_skia()),
        }
        let mask = self.clips.last().map(|c| &c.mask);
        self.pm.fill_path(
            &path,
            &paint,
            FillRule::Winding,
            Transform::identity(),
            mask,
        );
    }

    /// Vertical gradient fill (top color to bottom color).
    pub fn fill_rrect_vgradient(&mut self, r: Rect, radius: f32, top: Rgba, bottom: Rgba) {
        let (s, e) = (Point::from_xy(r.x, r.y), Point::from_xy(r.x, r.bottom()));
        self.fill_gradient(r, radius, s, e, top, bottom);
    }

    /// Horizontal gradient fill (left color to right color).
    pub fn fill_rrect_hgradient(&mut self, r: Rect, radius: f32, left: Rgba, right: Rgba) {
        let (s, e) = (Point::from_xy(r.x, r.y), Point::from_xy(r.right(), r.y));
        self.fill_gradient(r, radius, s, e, left, right);
    }

    /// Stroke a rounded rect with the stroke fully inside `r`.
    pub fn stroke_rrect(&mut self, r: Rect, radius: f32, width: f32, color: Rgba) {
        if !color.is_visible() || width <= 0.0 || self.alpha <= 0.0 {
            return;
        }
        let inner = r.inset(width * 0.5);
        let path = match rrect_path(inner, (radius - width * 0.5).max(0.0)) {
            Some(p) => p,
            None => return,
        };
        self.stroke_path(&path, width, color, false);
    }

    /// Stroke a rounded rect centered on the edge of `r` (used for focus rings).
    pub fn stroke_rrect_outside(&mut self, r: Rect, radius: f32, width: f32, color: Rgba) {
        self.stroke_rrect(r.outset(width), radius + width, width, color);
    }

    pub fn fill_circle(&mut self, cx: f32, cy: f32, radius: f32, color: Rgba) {
        if let Some(path) = PathBuilder::from_circle(cx, cy, radius) {
            self.fill_path(&path, color);
        }
    }

    pub fn stroke_circle(&mut self, cx: f32, cy: f32, radius: f32, width: f32, color: Rgba) {
        if let Some(path) = PathBuilder::from_circle(cx, cy, radius) {
            self.stroke_path(&path, width, color, false);
        }
    }

    pub fn stroke_polyline(&mut self, pts: &[(f32, f32)], width: f32, color: Rgba, round: bool) {
        if pts.len() < 2 {
            return;
        }
        let mut pb = PathBuilder::new();
        pb.move_to(pts[0].0, pts[0].1);
        for p in &pts[1..] {
            pb.line_to(p.0, p.1);
        }
        if let Some(path) = pb.finish() {
            self.stroke_path(&path, width, color, round);
        }
    }

    pub fn draw_pixmap(&mut self, img: &Pixmap, x: i32, y: i32, opacity: f32) {
        let paint = PixmapPaint {
            opacity: (opacity * self.alpha).clamp(0.0, 1.0),
            blend_mode: BlendMode::SourceOver,
            quality: FilterQuality::Nearest,
        };
        let mask = self.clips.last().map(|c| &c.mask);
        let (ox, oy) = (self.offset.0.round() as i32, self.offset.1.round() as i32);
        self.pm.draw_pixmap(
            x + ox,
            y + oy,
            img.as_ref(),
            &paint,
            Transform::identity(),
            mask,
        );
    }

    /// Draw a pixmap through an arbitrary transform (its top-left starts at the origin).
    pub fn draw_pixmap_transformed(&mut self, img: &Pixmap, transform: Transform, opacity: f32) {
        let paint = PixmapPaint {
            opacity: (opacity * self.alpha).clamp(0.0, 1.0),
            blend_mode: BlendMode::SourceOver,
            quality: FilterQuality::Bicubic,
        };
        let mask = self.clips.last().map(|c| &c.mask);
        let t = transform.post_translate(self.offset.0, self.offset.1);
        self.pm.draw_pixmap(0, 0, img.as_ref(), &paint, t, mask);
    }

    /// Draw a soft shadow under `r`.
    pub fn draw_shadow(
        &mut self,
        cache: &mut ShadowCache,
        r: Rect,
        radius: f32,
        blur: f32,
        dy: f32,
        color: Rgba,
    ) {
        if !color.is_visible() || r.is_empty() || self.alpha <= 0.0 {
            return;
        }
        let w = r.w.round().max(1.0) as u32;
        let h = r.h.round().max(1.0) as u32;
        let (pm, pad) = cache.get(w, h, radius, blur, color);
        let x = (r.x - pad as f32).round() as i32;
        let y = (r.y + dy - pad as f32).round() as i32;
        self.draw_pixmap(&pm, x, y, 1.0);
    }

    /// Draw an image into `dst` with the given fit (0 contain, 1 cover, 2 fill), clipped to a
    /// rounded rect.
    pub fn draw_image(&mut self, img: &Pixmap, dst: Rect, fit: i64, radius: f32) {
        if dst.is_empty() || img.width() == 0 || img.height() == 0 {
            return;
        }
        let (iw, ih) = (img.width() as f32, img.height() as f32);
        let (sx, sy) = match fit {
            2 => (dst.w / iw, dst.h / ih),
            1 => {
                let s = (dst.w / iw).max(dst.h / ih);
                (s, s)
            }
            _ => {
                let s = (dst.w / iw).min(dst.h / ih);
                (s, s)
            }
        };
        let dw = iw * sx;
        let dh = ih * sy;
        let tx = dst.x + (dst.w - dw) * 0.5;
        let ty = dst.y + (dst.h - dh) * 0.5;
        self.push_clip(dst, radius);
        let paint = PixmapPaint {
            opacity: self.alpha.clamp(0.0, 1.0),
            blend_mode: BlendMode::SourceOver,
            quality: if (sx - 1.0).abs() < 1e-3 && (sy - 1.0).abs() < 1e-3 {
                FilterQuality::Nearest
            } else {
                FilterQuality::Bicubic
            },
        };
        let t = Transform::from_row(sx, 0.0, 0.0, sy, tx, ty)
            .post_translate(self.offset.0, self.offset.1);
        let mask = self.clips.last().map(|c| &c.mask);
        self.pm.draw_pixmap(0, 0, img.as_ref(), &paint, t, mask);
        self.pop_clip();
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct ShadowKey {
    w: u32,
    h: u32,
    radius: u32,
    blur: u32,
    color: u32,
}

/// Pre-blurred shadow sprites keyed by size, radius, blur and color.
pub struct ShadowCache {
    map: HashMap<ShadowKey, (Arc<Pixmap>, u32)>,
    order: Vec<ShadowKey>,
}

impl Default for ShadowCache {
    fn default() -> Self {
        Self::new()
    }
}

impl ShadowCache {
    pub fn new() -> ShadowCache {
        ShadowCache {
            map: HashMap::new(),
            order: Vec::new(),
        }
    }

    pub fn get(
        &mut self,
        w: u32,
        h: u32,
        radius: f32,
        blur: f32,
        color: Rgba,
    ) -> (Arc<Pixmap>, u32) {
        let key = ShadowKey {
            w,
            h,
            radius: (radius * 4.0).round() as u32,
            blur: (blur * 4.0).round() as u32,
            color: color.to_rgba_u32(),
        };
        if let Some((pm, pad)) = self.map.get(&key) {
            return (pm.clone(), *pad);
        }
        let (pm, pad) = build_shadow(w, h, radius, blur, color);
        let pm = Arc::new(pm);
        if self.order.len() >= 160 {
            let old = self.order.remove(0);
            self.map.remove(&old);
        }
        self.order.push(key);
        self.map.insert(key, (pm.clone(), pad));
        (pm, pad)
    }
}

fn build_shadow(w: u32, h: u32, radius: f32, blur: f32, color: Rgba) -> (Pixmap, u32) {
    let pad = (blur * 1.5).ceil() as u32 + 1;
    let pw = w + 2 * pad;
    let ph = h + 2 * pad;
    let mut pm = Pixmap::new(pw, ph).expect("shadow pixmap");
    if let Some(path) = rrect_path(
        Rect::new(pad as f32, pad as f32, w as f32, h as f32),
        radius,
    ) {
        let mut paint = Paint::default();
        paint.set_color(color.to_skia());
        paint.anti_alias = true;
        pm.fill_path(
            &path,
            &paint,
            FillRule::Winding,
            Transform::identity(),
            None,
        );
    }
    let r = ((blur * 0.5).round() as usize).max(1);
    for _ in 0..3 {
        box_blur(&mut pm, r);
    }
    (pm, pad)
}

/// Separable box blur over premultiplied RGBA8 (uniform color, so channels blur independently).
fn box_blur(pm: &mut Pixmap, r: usize) {
    let w = pm.width() as usize;
    let h = pm.height() as usize;
    let data = pm.data_mut();
    let mut tmp = vec![0u8; data.len()];
    let mut prefix = vec![0u32; (w.max(h) + 1) * 4];

    // Horizontal pass: data -> tmp
    for y in 0..h {
        let row = &data[y * w * 4..(y + 1) * w * 4];
        prefix[..4].fill(0);
        for x in 0..w {
            for c in 0..4 {
                prefix[(x + 1) * 4 + c] = prefix[x * 4 + c] + row[x * 4 + c] as u32;
            }
        }
        for x in 0..w {
            let lo = x.saturating_sub(r);
            let hi = (x + r + 1).min(w);
            let n = (hi - lo) as u32;
            for c in 0..4 {
                let s = prefix[hi * 4 + c] - prefix[lo * 4 + c];
                tmp[(y * w + x) * 4 + c] = ((s + n / 2) / n) as u8;
            }
        }
    }
    // Vertical pass: tmp -> data
    for x in 0..w {
        prefix[..4].fill(0);
        for y in 0..h {
            for c in 0..4 {
                prefix[(y + 1) * 4 + c] = prefix[y * 4 + c] + tmp[(y * w + x) * 4 + c] as u32;
            }
        }
        for y in 0..h {
            let lo = y.saturating_sub(r);
            let hi = (y + r + 1).min(h);
            let n = (hi - lo) as u32;
            for c in 0..4 {
                let s = prefix[hi * 4 + c] - prefix[lo * 4 + c];
                data[(y * w + x) * 4 + c] = ((s + n / 2) / n) as u8;
            }
        }
    }
}
