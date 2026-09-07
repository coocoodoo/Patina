//! SVG support: parsing and rasterization through resvg, `currentColor` resolution and
//! tinting, plus a raster cache keyed by document, size and colors.

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use resvg::usvg;
use tiny_skia::{IntSize, Pixmap};

use crate::color::Rgba;
use crate::smil::Timeline;

static NEXT_ID: AtomicU64 = AtomicU64::new(1);

/// A parsed SVG document. The source is kept so that `currentColor` can be resolved against
/// the theme each time the document is rasterized, and so SMIL animations can be applied.
pub struct SvgDoc {
    pub id: u64,
    source: String,
    /// Intrinsic size in logical pixels (from `width`/`height` or the `viewBox`).
    pub width: f32,
    pub height: f32,
    timeline: Option<Timeline>,
}

impl SvgDoc {
    /// Parse SVG markup. Returns a human-readable error for invalid documents.
    pub fn parse(source: &str) -> Result<SvgDoc, String> {
        let timeline = Timeline::parse(source);
        let first_frame = match &timeline {
            Some(tl) => tl.apply(source, 0.0),
            None => source.to_string(),
        };
        let opt = usvg::Options::default();
        let tree = usvg::Tree::from_str(&resolve_current_color(&first_frame, Rgba::BLACK), &opt)
            .map_err(|e| format!("invalid SVG: {e}"))?;
        let size = tree.size();
        if !(size.width() > 0.0 && size.height() > 0.0) {
            return Err("invalid SVG: the document has no size".into());
        }
        Ok(SvgDoc {
            id: NEXT_ID.fetch_add(1, Ordering::Relaxed),
            source: source.to_string(),
            width: size.width(),
            height: size.height(),
            timeline: timeline.filter(|t| !t.is_empty()),
        })
    }

    /// Whether the document contains SMIL animations.
    pub fn is_animated(&self) -> bool {
        self.timeline.is_some()
    }

    /// Loop length of the animation, when all of its tracks repeat with one period.
    pub fn period(&self) -> Option<f64> {
        self.timeline.as_ref().and_then(|t| t.period())
    }

    /// Time after which the animation stops changing, if it ends at all.
    pub fn end(&self) -> Option<f64> {
        self.timeline.as_ref().and_then(|t| t.end())
    }

    /// Rasterize at `w` x `h` pixels, stretching the document to fill the area. `current` is
    /// the value of `currentColor`; `tint`, when set, recolors every pixel keeping only alpha.
    pub fn render(&self, w: u32, h: u32, current: Rgba, tint: Option<Rgba>) -> Option<Pixmap> {
        self.render_at(w, h, current, tint, 0.0)
    }

    /// Like `render`, at `t` seconds into the animation.
    pub fn render_at(
        &self,
        w: u32,
        h: u32,
        current: Rgba,
        tint: Option<Rgba>,
        t: f64,
    ) -> Option<Pixmap> {
        self.render_frame(w, h, current, tint, t, 0.0, 1.0)
    }

    /// Full-control rendering: `t` seconds into the animation, rotated by `spin_deg` and
    /// scaled by `pulse_k` around the center.
    pub fn render_frame(
        &self,
        w: u32,
        h: u32,
        current: Rgba,
        tint: Option<Rgba>,
        t: f64,
        spin_deg: f32,
        pulse_k: f32,
    ) -> Option<Pixmap> {
        if w == 0 || h == 0 {
            return None;
        }
        let frame = match &self.timeline {
            Some(tl) => tl.apply(&self.source, t),
            None => self.source.clone(),
        };
        let opt = usvg::Options::default();
        let tree = usvg::Tree::from_str(&resolve_current_color(&frame, current), &opt).ok()?;
        let mut target = resvg::tiny_skia::Pixmap::new(w, h)?;
        let (cx, cy) = (w as f32 * 0.5, h as f32 * 0.5);
        let mut transform =
            resvg::tiny_skia::Transform::from_scale(w as f32 / self.width, h as f32 / self.height);
        if spin_deg != 0.0 {
            transform = transform.post_concat(resvg::tiny_skia::Transform::from_rotate_at(
                spin_deg, cx, cy,
            ));
        }
        if (pulse_k - 1.0).abs() > 1e-4 {
            transform = transform.post_concat(
                resvg::tiny_skia::Transform::from_translate(cx, cy)
                    .pre_scale(pulse_k, pulse_k)
                    .pre_translate(-cx, -cy),
            );
        }
        resvg::render(&tree, transform, &mut target.as_mut());
        let mut data = target.take();
        if let Some(c) = tint {
            apply_tint(&mut data, c);
        }
        Pixmap::from_vec(data, IntSize::from_wh(w, h)?)
    }
}

/// Replace every `currentColor` (case-insensitively) with a CSS `rgba()` literal.
fn resolve_current_color(src: &str, c: Rgba) -> String {
    const NEEDLE: &str = "currentcolor";
    let lower = src.to_ascii_lowercase();
    if !lower.contains(NEEDLE) {
        return src.to_string();
    }
    let css = format!(
        "rgba({},{},{},{})",
        (c.r.clamp(0.0, 1.0) * 255.0).round() as u8,
        (c.g.clamp(0.0, 1.0) * 255.0).round() as u8,
        (c.b.clamp(0.0, 1.0) * 255.0).round() as u8,
        c.a.clamp(0.0, 1.0)
    );
    let mut out = String::with_capacity(src.len() + 64);
    let mut last = 0;
    for (i, _) in lower.match_indices(NEEDLE) {
        out.push_str(&src[last..i]);
        out.push_str(&css);
        last = i + NEEDLE.len();
    }
    out.push_str(&src[last..]);
    out
}

/// Recolor premultiplied RGBA8 pixels with `c`, keeping the coverage.
fn apply_tint(data: &mut [u8], c: Rgba) {
    let (r, g, b) = (
        c.r.clamp(0.0, 1.0),
        c.g.clamp(0.0, 1.0),
        c.b.clamp(0.0, 1.0),
    );
    let ca = c.a.clamp(0.0, 1.0);
    for px in data.chunks_exact_mut(4) {
        let a = px[3] as f32 / 255.0 * ca;
        px[0] = (r * a * 255.0 + 0.5) as u8;
        px[1] = (g * a * 255.0 + 0.5) as u8;
        px[2] = (b * a * 255.0 + 0.5) as u8;
        px[3] = (a * 255.0 + 0.5) as u8;
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct Key {
    doc: u64,
    w: u32,
    h: u32,
    current: u32,
    tint: Option<u32>,
    /// Animation frame (60 per second, folded onto the loop period when there is one).
    frame: u32,
    spin: u16,
    pulse: u16,
}

/// Rasterized SVGs, so a document is only re-rendered when its size or colors change.
pub struct SvgCache {
    map: HashMap<Key, Arc<Pixmap>>,
    order: Vec<Key>,
    bytes: usize,
}

impl Default for SvgCache {
    fn default() -> Self {
        Self::new()
    }
}

impl SvgCache {
    const MAX_BYTES: usize = 48 << 20;
    const MAX_ENTRIES: usize = 256;

    pub fn new() -> SvgCache {
        SvgCache {
            map: HashMap::new(),
            order: Vec::new(),
            bytes: 0,
        }
    }

    pub fn get(
        &mut self,
        doc: &SvgDoc,
        w: u32,
        h: u32,
        current: Rgba,
        tint: Option<Rgba>,
    ) -> Option<Arc<Pixmap>> {
        self.get_animated(doc, w, h, current, tint, 0.0, 0.0, 1.0)
    }

    /// A frame of an animated (or spinning / pulsing) document at time `t`.
    #[allow(clippy::too_many_arguments)]
    pub fn get_animated(
        &mut self,
        doc: &SvgDoc,
        w: u32,
        h: u32,
        current: Rgba,
        tint: Option<Rgba>,
        t: f64,
        spin_deg: f32,
        pulse_k: f32,
    ) -> Option<Arc<Pixmap>> {
        let t = if doc.is_animated() {
            match doc.period() {
                Some(p) if p > 0.0 => t % p,
                _ => t,
            }
        } else {
            0.0
        };
        let frame = (t * 60.0).round() as u32;
        let t = frame as f64 / 60.0;
        let spin_q = ((spin_deg.rem_euclid(360.0)) * 4.0).round() as u16;
        let pulse_q = (pulse_k.clamp(0.0, 4.0) * 1000.0).round() as u16;
        let key = Key {
            doc: doc.id,
            w,
            h,
            current: current.to_rgba_u32(),
            tint: tint.map(|t| t.to_rgba_u32()),
            frame,
            spin: spin_q,
            pulse: pulse_q,
        };
        if let Some(p) = self.map.get(&key) {
            return Some(p.clone());
        }
        let pm = Arc::new(doc.render_frame(
            w,
            h,
            current,
            tint,
            t,
            spin_q as f32 / 4.0,
            pulse_q as f32 / 1000.0,
        )?);
        self.bytes += (w as usize) * (h as usize) * 4;
        self.order.push(key);
        self.map.insert(key, pm.clone());
        while (self.bytes > Self::MAX_BYTES || self.order.len() > Self::MAX_ENTRIES)
            && self.order.len() > 1
        {
            let old = self.order.remove(0);
            if let Some(p) = self.map.remove(&old) {
                self.bytes = self
                    .bytes
                    .saturating_sub((p.width() as usize) * (p.height() as usize) * 4);
            }
        }
        Some(pm)
    }

    pub fn clear(&mut self) {
        self.map.clear();
        self.order.clear();
        self.bytes = 0;
    }
}
