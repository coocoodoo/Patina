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
    }

    /// Draws a 1px outline around tagged pixels onto untagged pixels behind them.
    pub fn outline(&mut self, color: u8) {
        let (w, h) = (self.w, self.h);
        for y in 0..h {
            for x in 0..w {
                let i = y * w + x;
                if self.tag[i] != 0 {
                    continue;
                }
                let d = self.depth[i];
                let hit = |j: usize| self.tag[j] != 0 && self.depth[j] > d;
                if (x > 0 && hit(i - 1))
                    || (x + 1 < w && hit(i + 1))
                    || (y > 0 && hit(i - w))
                    || (y + 1 < h && hit(i + w))
                {
                    self.color[i] = color;
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
