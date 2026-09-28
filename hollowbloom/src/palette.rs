//! The Resurrect 32 palette by Kerrie Lake (<https://lospec.com/palette-list/resurrect-32>)
//! and the lighting tables that keep every pixel the game ever shows inside it.
//!
//! Nothing is rendered in RGB. Textures, sprites, the UI and the framebuffer all hold palette
//! indices; light is applied through precomputed colour maps (in the spirit of the old
//! software renderers) that map `(light level, warmth, index)` to another index, with ordered
//! dithering between neighbouring levels. The only place RGB appears is the final blit.

/// The 32 colours, in lospec order.
pub const PALETTE: [u32; 32] = [
    0xffffff, 0xfb6b1d, 0xe83b3b, 0x831c5d, 0xc32454, 0xf04f78, 0xf68181, 0xfca790, 0xe3c896,
    0xab947a, 0x966c6c, 0x625565, 0x3e3546, 0x0b5e65, 0x0b8a8f, 0x1ebc73, 0x91db69, 0xfbff86,
    0xfbb954, 0xcd683d, 0x9e4539, 0x7a3045, 0x6b3e75, 0x905ea9, 0xa884f3, 0xeaaded, 0x8fd3ff,
    0x4d9be6, 0x4d65b4, 0x484a77, 0x30e1b9, 0x8ff8e2,
];

pub const WHITE: u8 = 0;
pub const ORANGE: u8 = 1;
pub const RED: u8 = 2;
pub const PLUM: u8 = 3;
pub const CRIMSON: u8 = 4;
pub const PINK: u8 = 5;
pub const SALMON: u8 = 6;
pub const PEACH: u8 = 7;
pub const SAND: u8 = 8;
pub const KHAKI: u8 = 9;
pub const ROSEWOOD: u8 = 10;
pub const SHADOW: u8 = 11;
pub const INK: u8 = 12;
pub const DEEP_TEAL: u8 = 13;
pub const TEAL: u8 = 14;
pub const GREEN: u8 = 15;
pub const LIME: u8 = 16;
pub const CREAM: u8 = 17;
pub const GOLD: u8 = 18;
pub const CLAY: u8 = 19;
pub const RUST: u8 = 20;
pub const MAROON: u8 = 21;
pub const GRAPE: u8 = 22;
pub const PURPLE: u8 = 23;
pub const LAVENDER: u8 = 24;
pub const BLUSH: u8 = 25;
pub const SKY: u8 = 26;
pub const BLUE: u8 = 27;
pub const INDIGO: u8 = 28;
pub const SLATE: u8 = 29;
pub const AQUA: u8 = 30;
pub const MINT: u8 = 31;

/// Texel value that is never drawn (cut-out textures and sprites).
pub const CLEAR: u8 = 255;

/// Number of light levels in the colour maps. Level 16 is the texture's own colour.
pub const LEVELS: usize = 32;
/// The light level that reproduces texture colours exactly.
pub const FULL: f32 = 16.0;
/// Number of warmth steps: 0 is cold moonlight, 4 neutral, 8 firelight.
pub const WARMTHS: usize = 9;
pub const NEUTRAL: f32 = 4.0;
/// Opacity steps in the blend tables: step `k` covers `(k + 1) / (BLENDS + 1)`. A pixel's
/// opacity dithers between neighbouring steps, from fully clear to fully solid.
pub const BLENDS: usize = 8;

/// 4x4 ordered-dither thresholds in [0, 1).
pub const BAYER: [f32; 16] = [
    0.0 / 16.0,
    8.0 / 16.0,
    2.0 / 16.0,
    10.0 / 16.0,
    12.0 / 16.0,
    4.0 / 16.0,
    14.0 / 16.0,
    6.0 / 16.0,
    3.0 / 16.0,
    11.0 / 16.0,
    1.0 / 16.0,
    9.0 / 16.0,
    15.0 / 16.0,
    7.0 / 16.0,
    13.0 / 16.0,
    5.0 / 16.0,
];

#[inline]
pub fn bayer(x: usize, y: usize) -> f32 {
    BAYER[((y & 3) << 2) | (x & 3)]
}

fn srgb_to_linear(c: f32) -> f32 {
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

pub fn linear_rgb(i: u8) -> [f32; 3] {
    let c = PALETTE[i as usize];
    [
        srgb_to_linear(((c >> 16) & 0xff) as f32 / 255.0),
        srgb_to_linear(((c >> 8) & 0xff) as f32 / 255.0),
        srgb_to_linear((c & 0xff) as f32 / 255.0),
    ]
}

fn oklab([r, g, b]: [f32; 3]) -> [f32; 3] {
    let l = 0.412_221_46 * r + 0.536_332_55 * g + 0.051_445_995 * b;
    let m = 0.211_903_5 * r + 0.680_699_5 * g + 0.107_396_96 * b;
    let s = 0.088_302_46 * r + 0.281_718_85 * g + 0.629_978_7 * b;
    let (l, m, s) = (l.max(0.0).cbrt(), m.max(0.0).cbrt(), s.max(0.0).cbrt());
    [
        0.210_454_26 * l + 0.793_617_8 * m - 0.004_072_047 * s,
        1.977_998_5 * l - 2.428_592_2 * m + 0.450_593_7 * s,
        0.025_904_037 * l + 0.782_771_77 * m - 0.808_675_77 * s,
    ]
}

/// Precomputed lighting tables.
pub struct Shading {
    /// `map[(warmth * LEVELS + level) * 32 + index]`.
    pub map: Vec<u8>,
    /// Shadow tables: `darken[k][index]`, each step darker than the last.
    pub darken: [[u8; 32]; 3],
    /// Highlight table, one step brighter.
    pub lighten: [u8; 32],
    /// Translucency: `glass[(step * 32 + src) * 32 + dst]` is `src` laid over `dst` like
    /// tinted jelly, at the step's opacity (see `BLENDS`).
    pub glass: Vec<u8>,
    /// Emitted light: `glow[(step * 32 + src) * 32 + dst]` is `dst` with `src` added on top.
    pub glow: Vec<u8>,
    /// Palette as 0x00RRGGBB for presentation.
    pub rgb: [u32; 256],
    lab: [[f32; 3]; 32],
}

impl Default for Shading {
    fn default() -> Self {
        Self::new()
    }
}

impl Shading {
    pub fn new() -> Self {
        let mut lab = [[0.0; 3]; 32];
        for (i, l) in lab.iter_mut().enumerate() {
            *l = oklab(linear_rgb(i as u8));
        }
        let mut rgb = [PALETTE[INK as usize]; 256];
        rgb[..32].copy_from_slice(&PALETTE);
        let mut s = Shading {
            map: vec![0; WARMTHS * LEVELS * 32],
            darken: [[0; 32]; 3],
            lighten: [0; 32],
            glass: vec![0; BLENDS * 32 * 32],
            glow: vec![0; BLENDS * 32 * 32],
            rgb,
            lab,
        };
        for k in 0..BLENDS {
            let alpha = (k + 1) as f32 / (BLENDS + 1) as f32;
            for src in 0..32u8 {
                let a = linear_rgb(src);
                for dst in 0..32u8 {
                    let b = linear_rgb(dst);
                    let i = (k * 32 + src as usize) * 32 + dst as usize;
                    // Light through jelly takes on its colour (a filter), and some of the
                    // surface's own colour scatters back towards the eye.
                    let mut over = [0.0; 3];
                    let mut add = [0.0; 3];
                    for c in 0..3 {
                        let filtered = b[c] * (0.12 + 0.88 * a[c]);
                        over[c] = filtered * (1.0 - alpha) + a[c] * alpha;
                        add[c] = (b[c] + a[c] * alpha).min(1.0);
                    }
                    s.glass[i] = s.nearest(over);
                    s.glow[i] = s.nearest(add);
                }
            }
        }
        for w in 0..WARMTHS {
            let tint = tint(w as f32);
            for lv in 0..LEVELS {
                let level = lv as f32 / FULL;
                for i in 0..32u8 {
                    let out = if lv == FULL as usize && w == NEUTRAL as usize {
                        i
                    } else {
                        s.nearest(lit(linear_rgb(i), tint, level))
                    };
                    s.map[(w * LEVELS + lv) * 32 + i as usize] = out;
                }
            }
        }
        let neutral = NEUTRAL as usize * LEVELS;
        for (k, level) in [10usize, 6, 3].into_iter().enumerate() {
            let row = &s.map[(neutral + level) * 32..(neutral + level + 1) * 32];
            s.darken[k].copy_from_slice(row);
        }
        let row = &s.map[(neutral + 22) * 32..(neutral + 23) * 32];
        s.lighten.copy_from_slice(row);
        s
    }

    /// Nearest palette entry to a linear RGB colour, in OKLab with chroma weighted up so that
    /// darkening keeps its hue (the way a pixel artist builds a ramp).
    pub fn nearest(&self, c: [f32; 3]) -> u8 {
        let t = oklab(c);
        let mut best = 0;
        let mut best_d = f32::MAX;
        for (i, p) in self.lab.iter().enumerate() {
            let dl = t[0] - p[0];
            let da = t[1] - p[1];
            let db = t[2] - p[2];
            let d = dl * dl + 1.6 * (da * da + db * db);
            if d < best_d {
                best_d = d;
                best = i as u8;
            }
        }
        best
    }

    #[inline]
    pub fn shade(&self, warmth: usize, level: usize, index: u8) -> u8 {
        self.map[(warmth * LEVELS + level) * 32 + index as usize]
    }
}

/// Light colour for a warmth step.
fn tint(w: f32) -> [f32; 3] {
    const COLD: [f32; 3] = [0.55, 0.68, 1.0];
    const WARM: [f32; 3] = [1.0, 0.8, 0.58];
    let t = (w - NEUTRAL) / NEUTRAL;
    let (a, k) = if t < 0.0 { (COLD, -t) } else { (WARM, t) };
    [
        1.0 + (a[0] - 1.0) * k,
        1.0 + (a[1] - 1.0) * k,
        1.0 + (a[2] - 1.0) * k,
    ]
}

fn lit(base: [f32; 3], tint: [f32; 3], level: f32) -> [f32; 3] {
    let mut out = [0.0; 3];
    // Above full brightness a colour is scaled up (keeping its hue, so it climbs its own
    // ramp) with a little warm sunlight added, which nudges greens towards yellow.
    const SUN: [f32; 3] = [1.0, 0.9, 0.45];
    let over = (level - 1.0).max(0.0);
    for c in 0..3 {
        let v = base[c] * tint[c] * level.min(1.0);
        out[c] = (v * (1.0 + over * 0.9) + over * 0.1 * SUN[c] * tint[c]).clamp(0.0, 1.0);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn neutral_full_light_is_identity() {
        let s = Shading::new();
        for i in 0..32u8 {
            assert_eq!(s.shade(NEUTRAL as usize, FULL as usize, i), i);
        }
    }

    #[test]
    fn every_entry_is_in_palette() {
        let s = Shading::new();
        assert!(s.map.iter().all(|&i| i < 32));
        assert!(s.darken.iter().flatten().all(|&i| i < 32));
    }

    #[test]
    fn darkness_goes_to_ink() {
        let s = Shading::new();
        assert_eq!(s.shade(NEUTRAL as usize, 0, WHITE), INK);
    }

    #[test]
    fn blends_stay_in_palette_and_lean_the_right_way() {
        let s = Shading::new();
        assert!(s.glass.iter().chain(s.glow.iter()).all(|&i| i < 32));
        let at = |k: usize, src: u8, dst: u8| s.glass[(k * 32 + src as usize) * 32 + dst as usize];
        // Jelly over itself is itself; half-clear jelly shows what's behind it.
        assert_eq!(at(BLENDS - 1, GREEN, GREEN), GREEN);
        let lum = |i: u8| s.lab[i as usize][0];
        for src in [GREEN, BLUE, PINK, ORANGE] {
            let (dark, light) = (at(BLENDS / 2, src, INK), at(BLENDS / 2, src, WHITE));
            assert!(lum(light) > lum(dark), "jelly {src} hides the floor");
        }
        // Adding light never darkens.
        for dst in 0..32u8 {
            let g = s.glow[((BLENDS - 1) * 32 + GOLD as usize) * 32 + dst as usize];
            assert!(lum(g) + 0.02 >= lum(dst), "glow darkened {dst}");
        }
    }
}
