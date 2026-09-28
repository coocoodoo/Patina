//! The low-resolution, palette-indexed framebuffer and its integer upscale to the window.

use crate::palette::INK;

pub struct Frame {
    pub w: usize,
    pub h: usize,
    /// Palette indices.
    pub color: Vec<u8>,
    /// Reciprocal clip-space w (bigger is closer); 0 is the far background.
    pub depth: Vec<f32>,
    /// Object tags for outlines and see-through silhouettes (0 = scenery).
    pub tag: Vec<u8>,
    /// Pixels that give off their own light (flames, lit windows): shadows and ambient
    /// occlusion leave them alone.
    pub glow: Vec<bool>,
}

impl Frame {
    pub fn new(w: usize, h: usize) -> Self {
        let n = w * h;
        Frame {
            w,
            h,
            color: vec![INK; n],
            depth: vec![0.0; n],
            tag: vec![0; n],
            glow: vec![false; n],
        }
    }

    pub fn resize(&mut self, w: usize, h: usize) {
        if w != self.w || h != self.h {
            *self = Frame::new(w, h);
        }
    }

    pub fn clear(&mut self, c: u8) {
        self.color.fill(c);
        self.depth.fill(0.0);
        self.tag.fill(0);
        self.glow.fill(false);
    }

    /// Screen-space ambient occlusion: darkens creases and contact points (where the floor
    /// meets walls, furniture, feet and anything dropped on it), so things stand apart from
    /// a floor of the same colour. It looks only at depth: a pixel is tucked into a crease
    /// when it lies behind the line joining pairs of opposite neighbours (for any flat
    /// surface it lies exactly on it). Up to two steps down each colour's ramp.
    pub fn ambient_occlusion(&mut self, ramp: &[u8; 32], strength: f32) {
        if strength <= 0.0 {
            return;
        }
        // Opposite pairs in four directions at four radii, nearer ones counting more.
        const DIRS: [(isize, isize); 4] = [(1, 0), (0, 1), (1, 1), (1, -1)];
        const RADII: [(isize, f32); 4] = [(2, 1.0), (4, 0.85), (7, 0.6), (11, 0.4)];
        const R: isize = 11;
        // In world units: creases shallower than EPS don't count, FULL is a deep one, and
        // anything more than RANGE in front is something else passing by, not a crease.
        const EPS: f32 = 0.004;
        const FULL: f32 = 0.06;
        const RANGE: f32 = 0.6;
        let (w, h) = (self.w as isize, self.h as isize);
        let mut kernel = Vec::with_capacity(16);
        for &(r, wt) in &RADII {
            for &(dx, dy) in &DIRS {
                kernel.push((dx * r, dy * r, (dy * r) * w + dx * r, wt));
            }
        }
        let total: f32 = kernel.iter().map(|k| k.3).sum();
        let scale = strength / total;
        let (depth, glow) = (&self.depth, &self.glow);
        par_rows(&mut self.color, self.w, |y, row| {
            let y = y as isize;
            let inner_y = y >= R && y < h - R;
            for x in 0..w {
                let i = (y * w + x) as usize;
                let d = depth[i];
                let c = row[x as usize];
                if d <= 0.0 || c >= 32 || glow[i] {
                    continue;
                }
                // Differences in 1/depth to world units, near this pixel's depth.
                let k = 1.0 / (d * d);
                let inner = inner_y && x >= R && x < w - R;
                let mut occ = 0.0;
                for &(dx, dy, off, wt) in &kernel {
                    let (a, b) = if inner {
                        (
                            depth[(i as isize + off) as usize],
                            depth[(i as isize - off) as usize],
                        )
                    } else {
                        let (ax, ay, bx, by) = (x + dx, y + dy, x - dx, y - dy);
                        if ax < 0
                            || bx < 0
                            || ax >= w
                            || bx >= w
                            || ay < 0
                            || by < 0
                            || ay >= h
                            || by >= h
                        {
                            continue;
                        }
                        (depth[(ay * w + ax) as usize], depth[(by * w + bx) as usize])
                    };
                    if a <= 0.0 || b <= 0.0 {
                        continue;
                    }
                    let (da, db) = ((a - d) * k, (b - d) * k);
                    if da > RANGE || db > RANGE {
                        continue;
                    }
                    let crease = (da + db) * 0.5 - EPS;
                    if crease > 0.0 {
                        occ += wt * (crease * (1.0 / FULL)).min(1.0);
                    }
                }
                let amount = occ * scale;
                if amount < 0.12 {
                    continue;
                }
                // Two steps where it's deep, one elsewhere, with a little dither at the edge
                // between them so it never bands.
                let th = crate::palette::BAYER[(((y & 3) << 2) | (x & 3)) as usize];
                let steps = (amount * 2.2 + (th - 0.5) * 0.6 + 0.25) as usize;
                let mut out = c;
                for _ in 0..steps.min(2) {
                    out = ramp[out as usize];
                }
                row[x as usize] = out;
            }
        });
    }

    /// Draws a 1px outline around tagged pixels onto untagged pixels behind them: in
    /// `color`, or in `glow` round things tagged `glow_tag` (full-moon creatures).
    pub fn outline(&mut self, color: u8) {
        self.outline_with(color, u8::MAX, color);
    }

    pub fn outline_with(&mut self, color: u8, glow_tag: u8, glow: u8) {
        let (w, h) = (self.w, self.h);
        for y in 0..h {
            for x in 0..w {
                let i = y * w + x;
                if self.tag[i] != 0 {
                    continue;
                }
                let d = self.depth[i];
                let hit = |j: usize| self.tag[j] != 0 && self.depth[j] > d;
                let mut found = None;
                for (ok, j) in [
                    (x > 0, i.wrapping_sub(1)),
                    (x + 1 < w, i + 1),
                    (y > 0, i.wrapping_sub(w)),
                    (y + 1 < h, i + w),
                ] {
                    if ok && hit(j) {
                        found = Some(self.tag[j]);
                        if self.tag[j] == glow_tag {
                            break;
                        }
                    }
                }
                if let Some(t) = found {
                    self.color[i] = if t == glow_tag { glow } else { color };
                }
            }
        }
    }

    /// Nearest-neighbour upscale into a 0x00RRGGBB buffer of `dw * dh` pixels.
    pub fn blit_scaled(
        &self,
        dst: &mut [u32],
        dw: usize,
        dh: usize,
        scale: usize,
        rgb: &[u32; 256],
    ) {
        let scale = scale.max(1);
        let ink = rgb[INK as usize];
        let mut row = vec![ink; dw];
        let mut built_for = usize::MAX;
        for y in 0..dh {
            let sy = y / scale;
            let out = &mut dst[y * dw..(y + 1) * dw];
            if sy >= self.h {
                out.fill(ink);
                continue;
            }
            if built_for != sy {
                built_for = sy;
                let src = &self.color[sy * self.w..(sy + 1) * self.w];
                let mut x = 0;
                for &c in src {
                    let px = rgb[c as usize];
                    let end = (x + scale).min(dw);
                    row[x..end].fill(px);
                    x = end;
                    if x >= dw {
                        break;
                    }
                }
                row[x..].fill(ink);
            }
            out.copy_from_slice(&row);
        }
    }
}

/// How many threads the post passes share rows between.
fn workers() -> usize {
    static N: std::sync::OnceLock<usize> = std::sync::OnceLock::new();
    *N.get_or_init(|| {
        std::thread::available_parallelism()
            .map_or(1, |n| n.get())
            .clamp(1, 8)
    })
}

/// Runs `f(y, row)` over every row of a colour buffer `w` wide, shared out between threads.
pub fn par_rows(color: &mut [u8], w: usize, f: impl Fn(usize, &mut [u8]) + Sync) {
    let h = color.len() / w.max(1);
    let threads = workers().min(h.max(1));
    if threads <= 1 {
        for (y, row) in color.chunks_mut(w).enumerate() {
            f(y, row);
        }
        return;
    }
    let band = h.div_ceil(threads);
    std::thread::scope(|s| {
        for (k, chunk) in color.chunks_mut(band * w).enumerate() {
            let f = &f;
            s.spawn(move || {
                for (j, row) in chunk.chunks_mut(w).enumerate() {
                    f(k * band + j, row);
                }
            });
        }
    });
}
