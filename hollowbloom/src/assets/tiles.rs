//! Procedural 16x16 tile and material textures, drawn only with palette indices.

use crate::palette::*;
use crate::render::Texture;
use crate::util::Rng;

pub const T: i32 = 16;

fn tex(fill: u8) -> Texture {
    Texture::new(T as u32, T as u32, fill)
}

fn speckle(t: &mut Texture, r: &mut Rng, color: u8, count: usize) {
    for _ in 0..count {
        let x = r.range(0, t.w as i32);
        let y = r.range(0, t.h as i32);
        t.set_wrap(x, y, color);
    }
}

/// A soft blob drawn with wrap-around (for seamless patches).
fn blob(t: &mut Texture, cx: i32, cy: i32, rx: i32, ry: i32, color: u8) {
    for y in -ry..=ry {
        for x in -rx..=rx {
            let fx = x as f32 / (rx as f32 + 0.5);
            let fy = y as f32 / (ry as f32 + 0.5);
            if fx * fx + fy * fy <= 1.0 {
                t.set_wrap(cx + x, cy + y, color);
            }
        }
    }
}

pub fn grass(seed: u64, flowers: bool) -> Texture {
    let mut t = tex(GREEN);
    let mut r = Rng::new(seed);
    // A soft darker clump gives the lawn some depth.
    let (x, y) = (r.range(0, T), r.range(0, T));
    blob(&mut t, x, y, 2, 1, TEAL);
    // Blades: a light tip with a dark root beside it (real 3D tufts grow on top).
    for _ in 0..3 {
        let (x, y) = (r.range(0, T), r.range(0, T));
        t.set_wrap(x, y, LIME);
        t.set_wrap(x + 1, y + 1, TEAL);
    }
    if flowers {
        for _ in 0..2 {
            let (x, y) = (r.range(1, T - 1), r.range(1, T - 1));
            let c = *r.pick(&[WHITE, CREAM, PINK, SKY, BLUSH]);
            t.set_wrap(x, y, c);
            t.set_wrap(x + 1, y + 1, TEAL);
        }
    }
    t
}

/// Winter's lawn: soft snow with drifts in its shadows, and now and then a sprig of
/// grass or a snowdrop poking through.
pub fn snow(seed: u64, sprigs: bool) -> Texture {
    let mut t = tex(WHITE);
    let mut r = Rng::new(seed ^ 0x5A0_5A0);
    // Drifts: blue hollows between the heaps.
    for _ in 0..2 {
        let (x, y) = (r.range(0, T), r.range(0, T));
        blob(&mut t, x, y, 2, 1, SKY);
    }
    // Sparkles catching the light, with a little shadow under each.
    for _ in 0..3 {
        let (x, y) = (r.range(0, T), r.range(0, T));
        t.set_wrap(x, y + 1, SKY);
    }
    if sprigs {
        for i in 0..2 {
            let (x, y) = (r.range(1, T - 1), r.range(2, T - 1));
            if i == 0 {
                // A snowdrop's nodding bell on a green stem.
                t.set_wrap(x, y, WHITE);
                t.set_wrap(x, y + 1, GREEN);
                t.set_wrap(x + 1, y + 1, SKY);
                t.set_wrap(x, y - 1, GREEN);
            } else {
                t.set_wrap(x, y, TEAL);
                t.set_wrap(x + 1, y - 1, GREEN);
                t.set_wrap(x, y + 1, SKY);
            }
        }
    }
    t
}

/// A hedge's top under a cap of snow.
pub fn snowy_hedge(seed: u64) -> Texture {
    let mut t = hedge(seed);
    let mut r = Rng::new(seed ^ 0x51);
    for _ in 0..7 {
        let (x, y) = (r.range(0, T), r.range(0, T));
        blob(&mut t, x, y, 2, 1, WHITE);
    }
    speckle(&mut t, &mut r, SKY, 5);
    t
}

/// Leaves in blossom: the canopy, sprinkled with little flowers.
pub fn blossom(base: Texture, petals: [u8; 2], seed: u64) -> Texture {
    let mut t = base;
    let mut r = Rng::new(seed ^ 0xB105);
    let [petal, heart] = petals;
    // Little five-petal flowers: a cross of petals round a darker heart.
    for _ in 0..9 {
        let (x, y) = (r.range(0, T), r.range(0, T));
        for (dx, dy) in [(0, -1), (-1, 0), (1, 0), (0, 1)] {
            t.set_wrap(x + dx, y + dy, petal);
        }
        t.set_wrap(x, y, heart);
    }
    speckle(&mut t, &mut r, petal, 6);
    t
}

/// Leaves under snow: big white clumps with blue shadows, the green showing between.
pub fn snowcap(base: Texture, seed: u64) -> Texture {
    let mut t = base;
    let mut r = Rng::new(seed ^ 0x5C4);
    for _ in 0..9 {
        let (x, y) = (r.range(0, T), r.range(0, T));
        blob(&mut t, x, y + 1, 3, 1, SKY);
        blob(&mut t, x, y, 3, 1, WHITE);
    }
    speckle(&mut t, &mut r, WHITE, 8);
    t
}

pub fn dirt_path(seed: u64) -> Texture {
    let mut t = tex(SAND);
    let mut r = Rng::new(seed);
    for _ in 0..4 {
        let (x, y) = (r.range(0, T), r.range(0, T));
        blob(&mut t, x, y, r.range(1, 3), 1, KHAKI);
    }
    for _ in 0..6 {
        let (x, y) = (r.range(0, T), r.range(0, T));
        t.set_wrap(x, y, PEACH);
        t.set_wrap(x, y + 1, KHAKI);
    }
    speckle(&mut t, &mut r, ROSEWOOD, 7);
    speckle(&mut t, &mut r, WHITE, 2);
    t
}

pub fn soil(seed: u64) -> Texture {
    let mut t = tex(RUST);
    let mut r = Rng::new(seed);
    for _ in 0..5 {
        let (x, y) = (r.range(0, T), r.range(0, T));
        blob(&mut t, x, y, r.range(1, 3), 1, CLAY);
    }
    speckle(&mut t, &mut r, MAROON, 10);
    speckle(&mut t, &mut r, GOLD, 3);
    t
}

/// Hoed soil: furrows running across the tile.
pub fn tilled(wet: bool) -> Texture {
    let (base, light, dark, deep) = if wet {
        (MAROON, RUST, GRAPE, INK)
    } else {
        (RUST, CLAY, MAROON, MAROON)
    };
    let mut t = tex(base);
    let mut r = Rng::new(if wet { 71 } else { 70 });
    for y in 0..T {
        for x in 0..T {
            let row = y % 4;
            let c = match row {
                0 => light,
                1 => base,
                2 => dark,
                _ => base,
            };
            t.set(x, y, c);
        }
    }
    // Break the furrows up a little.
    for _ in 0..10 {
        let (x, y) = (r.range(0, T), r.range(0, T));
        t.set(x, y, if r.chance(0.5) { light } else { dark });
    }
    speckle(&mut t, &mut r, deep, 5);
    // Soft border so a field reads as separate beds.
    for i in 0..T {
        t.set(i, 0, dark);
        t.set(0, i, dark);
    }
    t
}

pub fn sand(seed: u64) -> Texture {
    let mut t = tex(PEACH);
    let mut r = Rng::new(seed);
    speckle(&mut t, &mut r, SAND, 20);
    speckle(&mut t, &mut r, SALMON, 5);
    speckle(&mut t, &mut r, WHITE, 3);
    t
}

/// Corner bits for rounded tiles: north-west, north-east, south-west, south-east
/// (texel (0, 0) is the north-west corner).
pub const NW: u8 = 1;
pub const NE: u8 = 2;
pub const SW: u8 = 4;
pub const SE: u8 = 8;

/// `base` with the corners in `mask` rounded off: texels outside a quarter circle of
/// radius `r` tucked into each corner come from `fill` instead. Rounding a road into
/// grass softens its outer corners; rounding grass into road fills its inner ones.
pub fn round_corners(base: &Texture, fill: &Texture, mask: u8, r: f32) -> Texture {
    let mut t = base.clone();
    let n = T as f32;
    for (bit, cx, cy) in [
        (NW, r, r),
        (NE, n - r, r),
        (SW, r, n - r),
        (SE, n - r, n - r),
    ] {
        if mask & bit == 0 {
            continue;
        }
        for y in 0..T {
            for x in 0..T {
                let (px, py) = (x as f32 + 0.5, y as f32 + 0.5);
                // Only the square between the circle's centre and the corner.
                let inside_x = if cx < n * 0.5 { px < cx } else { px > cx };
                let inside_y = if cy < n * 0.5 { py < cy } else { py > cy };
                if !(inside_x && inside_y) {
                    continue;
                }
                let (dx, dy) = (px - cx, py - cy);
                if dx * dx + dy * dy > r * r {
                    t.set(x, y, fill.get(x, y));
                }
            }
        }
    }
    t
}

/// One frame of animated water; `frame` shifts the ripples.
pub fn water(frame: u32) -> Texture {
    let mut t = tex(BLUE);
    let mut r = Rng::new(900);
    for _ in 0..4 {
        let (x, y) = (r.range(0, T), r.range(0, T));
        blob(&mut t, x, y, 2, 1, INDIGO);
    }
    let shift = frame as i32;
    let mut r2 = Rng::new(901);
    for _ in 0..6 {
        let (x, y) = (r2.range(0, T), r2.range(0, T));
        let len = r2.range(2, 4);
        let dx = if (y / 4) % 2 == 0 { shift } else { -shift };
        for k in 0..len {
            t.set_wrap(x + k + dx, y, SKY);
        }
        t.set_wrap(x + len + dx, y + 1, WHITE);
    }
    t
}

/// Planks running left-right.
pub fn planks(base: u8, light: u8, dark: u8, seed: u64) -> Texture {
    let mut t = tex(base);
    let mut r = Rng::new(seed);
    for y in 0..T {
        let row = y % 4;
        for x in 0..T {
            if row == 3 {
                t.set(x, y, dark);
            } else if row == 0 && r.chance(0.5) {
                t.set(x, y, light);
            }
        }
        if row == 1 {
            // A seam somewhere in each plank.
            let seam = (r.range(0, T) + y * 5) % T;
            t.set(seam, y, dark);
            t.set(seam, y + 1, dark);
        }
    }
    for _ in 0..4 {
        let (x, y) = (r.range(0, T), r.range(0, T));
        if y % 4 != 3 {
            t.set(x, y, dark);
        }
    }
    t
}

/// Irregular cobblestones.
pub fn cobbles(stone: u8, light: u8, gap: u8, seed: u64) -> Texture {
    let mut t = tex(gap);
    let mut r = Rng::new(seed);
    // Place stones on a jittered grid, each a rounded rect.
    for gy in 0..4 {
        for gx in 0..4 {
            let off = if gy % 2 == 0 { 0 } else { 2 };
            let x0 = gx * 4 + off + r.range(0, 1);
            let y0 = gy * 4;
            let w = r.range(3, 4);
            let h = 3;
            for y in 0..h {
                for x in 0..w {
                    let corner = (x == 0 || x == w - 1) && (y == 0 || y == h - 1);
                    if !corner || r.chance(0.3) {
                        let c = if y == 0 && x < w - 1 { light } else { stone };
                        t.set_wrap(x0 + x, y0 + y, c);
                    }
                }
            }
        }
    }
    t
}

/// Brick courses.
pub fn bricks(brick: u8, light: u8, mortar: u8, seed: u64) -> Texture {
    let mut t = tex(brick);
    let mut r = Rng::new(seed);
    for y in 0..T {
        let course = y / 4;
        for x in 0..T {
            let off = if course % 2 == 0 { 0 } else { 4 };
            if y % 4 == 3 || (x + off) % 8 == 7 {
                t.set(x, y, mortar);
            } else if y % 4 == 0 && r.chance(0.6) {
                t.set(x, y, light);
            }
        }
    }
    t
}

/// Layered rock for wall faces: horizontal strata with cracks.
pub fn rock_side(pal: [u8; 4], seed: u64) -> Texture {
    let [light, mid, dark, deep] = pal;
    let mut t = tex(mid);
    let mut r = Rng::new(seed);
    for y in 0..T {
        for x in 0..T {
            let band =
                ((y as f32 + (x as f32 * 0.7 + seed as f32).sin() * 1.2) as i32).rem_euclid(6);
            let c = match band {
                0 => light,
                4 | 5 => dark,
                _ => mid,
            };
            t.set(x, y, c);
        }
    }
    for _ in 0..3 {
        let mut x = r.range(0, T);
        let mut y = r.range(0, T);
        for _ in 0..r.range(2, 5) {
            t.set_wrap(x, y, deep);
            x += r.range(-1, 2);
            y += 1;
        }
    }
    speckle(&mut t, &mut r, light, 4);
    // Top lip catches the light, bottom sinks into shadow.
    for x in 0..T {
        t.set(x, 0, light);
        t.set(x, T - 1, deep);
    }
    t
}

/// Speckled stone for boulders and rocks.
pub fn speckled_stone(pal: [u8; 4], seed: u64) -> Texture {
    let [light, mid, dark, deep] = pal;
    let mut t = tex(mid);
    let mut r = Rng::new(seed);
    for _ in 0..3 {
        let (x, y) = (r.range(0, T), r.range(0, T));
        blob(&mut t, x, y, 2, 1, dark);
    }
    for _ in 0..4 {
        let (x, y) = (r.range(0, T), r.range(0, T));
        t.set_wrap(x, y, light);
        t.set_wrap(x + 1, y, light);
    }
    speckle(&mut t, &mut r, deep, 5);
    t
}

/// Wall tops: dark and quiet so the maze reads clearly from above.
pub fn rock_top(pal: [u8; 4], seed: u64) -> Texture {
    let [light, mid, dark, deep] = pal;
    let _ = light;
    let mut t = tex(deep);
    let mut r = Rng::new(seed);
    for _ in 0..3 {
        let (x, y) = (r.range(0, T), r.range(0, T));
        blob(&mut t, x, y, r.range(1, 3), 1, dark);
    }
    speckle(&mut t, &mut r, mid, 5);
    t
}

/// Cave floor: smooth stone with cracks and a scattering of pebbles.
pub fn cave_floor(pal: [u8; 4], accent: u8, seed: u64) -> Texture {
    let [light, mid, dark, deep] = pal;
    let mut t = tex(mid);
    let mut r = Rng::new(seed);
    for _ in 0..4 {
        let (x, y) = (r.range(0, T), r.range(0, T));
        blob(&mut t, x, y, r.range(1, 4), r.range(1, 2), dark);
    }
    for _ in 0..3 {
        let (x, y) = (r.range(0, T), r.range(0, T));
        t.set_wrap(x, y, light);
        t.set_wrap(x + 1, y, light);
        t.set_wrap(x, y + 1, dark);
        t.set_wrap(x + 1, y + 1, deep);
    }
    for _ in 0..2 {
        let mut x = r.range(0, T);
        let mut y = r.range(0, T);
        for _ in 0..r.range(3, 6) {
            t.set_wrap(x, y, deep);
            x += 1;
            y += r.range(-1, 2);
        }
    }
    if accent != CLEAR {
        for _ in 0..3 {
            let (x, y) = (r.range(0, T), r.range(0, T));
            t.set_wrap(x, y, accent);
            if r.chance(0.6) {
                t.set_wrap(x + 1, y, accent);
            }
            if r.chance(0.4) {
                t.set_wrap(x, y + 1, accent);
            }
        }
    }
    t
}

/// Ore nuggets pressed into a rock texture.
pub fn with_ore(base: &Texture, ore: [u8; 3], seed: u64) -> Texture {
    let [hi, mid, lo] = ore;
    let mut t = base.clone();
    let mut r = Rng::new(seed);
    let spots = [(2, 3), (9, 2), (5, 9), (11, 10)];
    for (i, (sx, sy)) in spots.iter().enumerate() {
        let x = sx + r.range(-1, 2);
        let y = sy + r.range(-1, 2);
        for (dx, dy) in [
            (-1, 0),
            (0, -1),
            (1, -1),
            (2, 0),
            (2, 1),
            (1, 2),
            (0, 2),
            (-1, 1),
        ] {
            t.set(x + dx, y + dy, INK);
        }
        t.set(x, y, hi);
        t.set(x + 1, y, mid);
        t.set(x, y + 1, mid);
        t.set(x + 1, y + 1, lo);
        if i % 2 == 0 {
            t.set(x + 1, y, hi);
        }
    }
    t
}

pub fn leaves(pal: [u8; 4], seed: u64) -> Texture {
    let [hi, mid, dark, deep] = pal;
    let mut t = tex(mid);
    let mut r = Rng::new(seed);
    for _ in 0..7 {
        let (x, y) = (r.range(0, T), r.range(0, T));
        blob(&mut t, x, y, 2, 1, dark);
    }
    for _ in 0..9 {
        let (x, y) = (r.range(0, T), r.range(0, T));
        t.set_wrap(x, y, hi);
        t.set_wrap(x + 1, y, hi);
        t.set_wrap(x, y + 1, mid);
    }
    speckle(&mut t, &mut r, deep, 8);
    t
}

/// Leaf canopy: a darker body than the lawn with bright clumps of light on top.
pub fn canopy(pal: [u8; 4], seed: u64) -> Texture {
    let [hi, mid, dark, deep] = pal;
    let mut t = tex(dark);
    let mut r = Rng::new(seed);
    for _ in 0..6 {
        let (x, y) = (r.range(0, T), r.range(0, T));
        blob(&mut t, x, y, 2, 1, mid);
    }
    for _ in 0..6 {
        let (x, y) = (r.range(0, T), r.range(0, T));
        t.set_wrap(x, y, hi);
        t.set_wrap(x + 1, y, hi);
        t.set_wrap(x, y + 1, mid);
    }
    speckle(&mut t, &mut r, deep, 7);
    t
}

pub fn bark() -> Texture {
    let mut t = tex(RUST);
    let mut r = Rng::new(77);
    for x in 0..T {
        let c = match x % 5 {
            0 => MAROON,
            2 => CLAY,
            _ => RUST,
        };
        for y in 0..T {
            t.set(x, y, c);
        }
    }
    speckle(&mut t, &mut r, MAROON, 10);
    speckle(&mut t, &mut r, CLAY, 4);
    t
}

/// Cross-section rings for cut logs and stumps.
pub fn rings() -> Texture {
    let mut t = tex(SAND);
    for y in 0..T {
        for x in 0..T {
            let dx = x as f32 - 7.5;
            let dy = y as f32 - 7.5;
            let d = (dx * dx + dy * dy).sqrt();
            let c = if d > 7.0 {
                RUST
            } else if (d as i32) % 3 == 0 {
                KHAKI
            } else {
                SAND
            };
            t.set(x, y, c);
        }
    }
    t
}

/// Clay roof tiles: overlapping scallops.
pub fn roof(pal: [u8; 3]) -> Texture {
    let [hi, mid, dark] = pal;
    let mut t = tex(mid);
    for y in 0..T {
        for x in 0..T {
            let row = y / 4;
            let off = if row % 2 == 0 { 0 } else { 2 };
            let lx = (x + off) % 4;
            let ly = y % 4;
            let c = if ly == 3 || (ly == 2 && (lx == 0 || lx == 3)) {
                dark
            } else if ly == 0 && lx == 1 {
                hi
            } else {
                mid
            };
            t.set(x, y, c);
        }
    }
    t
}

/// Plaster wall with timber framing.
pub fn cottage_wall() -> Texture {
    let mut t = tex(PEACH);
    let mut r = Rng::new(31);
    speckle(&mut t, &mut r, SAND, 14);
    speckle(&mut t, &mut r, WHITE, 5);
    for i in 0..T {
        t.set(i, 0, RUST);
        t.set(i, T - 1, RUST);
        t.set(0, i, RUST);
        t.set(T - 1, i, MAROON);
        t.set(i, T - 2, SALMON);
    }
    t
}

/// Horizontal stripes for awnings and cloth.
pub fn stripes(a: u8, b: u8) -> Texture {
    let mut t = tex(a);
    for y in 0..T {
        for x in 0..T {
            if (x / 4) % 2 == 1 {
                t.set(x, y, b);
            }
        }
    }
    t
}

pub fn solid(c: u8) -> Texture {
    Texture::new(4, 4, c)
}

/// A soft round disc for blob shadows.
pub fn disk() -> Texture {
    let mut t = Texture::clear(16, 16);
    for y in 0..16 {
        for x in 0..16 {
            let dx = x as f32 - 7.5;
            let dy = y as f32 - 7.5;
            if dx * dx + dy * dy < 56.0 {
                t.set(x, y, INK);
            }
        }
    }
    t
}

/// Ground cursor frame (corners only).
pub fn cursor(color: u8) -> Texture {
    let mut t = Texture::clear(16, 16);
    for i in 0..4 {
        for (x, y) in [
            (i, 0),
            (0, i),
            (15 - i, 0),
            (15, i),
            (i, 15),
            (0, 15 - i),
            (15 - i, 15),
            (15, 15 - i),
        ] {
            t.set(x, y, color);
        }
    }
    t
}

/// Flickering flame texture (unlit).
pub fn flame(frame: u32) -> Texture {
    let mut t = Texture::clear(8, 8);
    let mut r = Rng::new(500 + frame as u64);
    for y in 0..8 {
        for x in 0..8 {
            let dx = (x as f32 - 3.5).abs();
            let top = 1.0 + dx * 1.4 + r.f32() * 1.5;
            if (y as f32) >= top {
                let c = if dx < 1.2 && y > 4 {
                    CREAM
                } else if dx < 2.2 && y > 2 {
                    GOLD
                } else {
                    ORANGE
                };
                t.set(x, y, c);
            }
        }
    }
    t
}

/// Glowing crystal facets (unlit).
pub fn crystal(pal: [u8; 3]) -> Texture {
    let [hi, mid, lo] = pal;
    let mut t = tex(mid);
    for y in 0..T {
        for x in 0..T {
            let c = if (x + y) % 7 == 0 {
                hi
            } else if (x * 2 + y) % 11 == 0 {
                lo
            } else {
                mid
            };
            t.set(x, y, c);
        }
    }
    t
}

/// Cliff face for the farm border: rock with a grassy lip.
pub fn cliff_side() -> Texture {
    let mut t = rock_side([KHAKI, ROSEWOOD, SHADOW, INK], 12);
    for x in 0..T {
        t.set(x, 0, LIME);
        t.set(x, 1, GREEN);
        if x % 3 != 1 {
            t.set(x, 2, TEAL);
        }
        if x % 5 == 0 {
            t.set(x, 3, TEAL);
        }
    }
    t
}

/// Rounded town cobbles in warm stone.
pub fn street(seed: u64) -> Texture {
    let mut t = tex(ROSEWOOD);
    let mut r = Rng::new(seed);
    for gy in 0..4 {
        for gx in 0..4 {
            let off = if gy % 2 == 0 { 0 } else { 2 };
            let x0 = gx * 4 + off;
            let y0 = gy * 4;
            let c = *r.pick(&[SAND, SAND, KHAKI, PEACH]);
            for y in 0..3 {
                for x in 0..3 {
                    let corner = (x == 0 || x == 2) && (y == 0 || y == 2);
                    if !corner {
                        let px = if y == 0 && x == 1 { PEACH } else { c };
                        t.set_wrap(x0 + x, y0 + y, px);
                    } else if r.chance(0.5) {
                        t.set_wrap(x0 + x, y0 + y, c);
                    }
                }
            }
        }
    }
    t
}

/// Basket-weave brick pavers for the plaza.
pub fn pavers() -> Texture {
    let mut t = tex(ROSEWOOD);
    for by in 0..4 {
        for bx in 0..4 {
            let flip = (bx + by) % 2 == 0;
            let (x0, y0) = (bx * 4, by * 4);
            for k in 0..2 {
                for a in 0..3 {
                    for b in 0..1 {
                        let (x, y) = if flip {
                            (x0 + a, y0 + k * 2 + b)
                        } else {
                            (x0 + k * 2 + b, y0 + a)
                        };
                        let c = if a == 0 { PEACH } else { SALMON };
                        t.set(x, y, c);
                    }
                }
            }
        }
    }
    t
}

/// Checkered shop tiles.
pub fn checker(a: u8, b: u8, line: u8) -> Texture {
    let mut t = tex(a);
    for y in 0..T {
        for x in 0..T {
            let c = if ((x / 8) + (y / 8)) % 2 == 0 { a } else { b };
            t.set(x, y, c);
            if x % 8 == 7 || y % 8 == 7 {
                t.set(x, y, line);
            }
        }
    }
    t
}

/// A soft carpet with a small diamond pattern.
pub fn carpet(base: u8, pattern: u8, dot: u8) -> Texture {
    let mut t = tex(base);
    for y in 0..T {
        for x in 0..T {
            let dx = (x % 8 - 4).abs();
            let dy = (y % 8 - 4).abs();
            if dx + dy == 3 {
                t.set(x, y, pattern);
            }
            if dx + dy == 0 {
                t.set(x, y, dot);
            }
        }
    }
    t
}

/// Clipped hedge leaves.
pub fn hedge(seed: u64) -> Texture {
    let mut t = tex(GREEN);
    let mut r = Rng::new(seed);
    for _ in 0..10 {
        let (x, y) = (r.range(0, T), r.range(0, T));
        blob(&mut t, x, y, 1, 1, TEAL);
    }
    for _ in 0..14 {
        let (x, y) = (r.range(0, T), r.range(0, T));
        t.set_wrap(x, y, LIME);
    }
    speckle(&mut t, &mut r, DEEP_TEAL, 6);
    t
}

/// Painted plaster in a palette of (light, main, dark).
pub fn plaster(pal: [u8; 3], seed: u64) -> Texture {
    let [hi, mid, lo] = pal;
    let mut t = tex(mid);
    let mut r = Rng::new(seed);
    speckle(&mut t, &mut r, hi, 12);
    speckle(&mut t, &mut r, lo, 6);
    t
}

/// Upright boards.
pub fn boards(pal: [u8; 3], seed: u64) -> Texture {
    let [hi, mid, lo] = pal;
    let mut t = tex(mid);
    let mut r = Rng::new(seed);
    for x in 0..T {
        let col = x % 4;
        for y in 0..T {
            if col == 3 {
                t.set(x, y, lo);
            } else if col == 0 && r.chance(0.5) {
                t.set(x, y, hi);
            }
        }
    }
    for _ in 0..4 {
        let (x, y) = (r.range(0, T), r.range(0, T));
        if x % 4 != 3 {
            t.set(x, y, lo);
        }
    }
    t
}

/// A clock face with hands at ten past ten.
pub fn clock_face() -> Texture {
    let mut t = Texture::clear(16, 16);
    for y in 0..16 {
        for x in 0..16 {
            let (dx, dy) = (x as f32 - 7.5, y as f32 - 7.5);
            let d = (dx * dx + dy * dy).sqrt();
            if d < 7.6 {
                t.set(x, y, if d > 6.4 { GOLD } else { CREAM });
            }
        }
    }
    for (x, y) in [(7, 1), (7, 13), (1, 7), (13, 7)] {
        t.set(x, y, RUST);
    }
    for k in 0..5 {
        t.set(7, 7 - k, INK);
    }
    for k in 0..4 {
        t.set(7 + k, 7 - k / 2, INK);
    }
    t
}

/// A guild banner: blue cloth, a gold lantern and a fringe.
pub fn banner() -> Texture {
    let mut t = tex(BLUE);
    for y in 0..T {
        t.set(0, y, INDIGO);
        t.set(T - 1, y, INDIGO);
    }
    for x in 0..T {
        t.set(x, 0, GOLD);
        if x % 2 == 0 {
            t.set(x, T - 1, GOLD);
        } else {
            t.set(x, T - 1, CLEAR);
        }
    }
    for y in 5..11 {
        for x in 6..10 {
            t.set(x, y, if y == 5 || y == 10 { CLAY } else { GOLD });
        }
    }
    t.set(7, 4, CLAY);
    t.set(8, 4, CLAY);
    t.set(7, 7, CREAM);
    t.set(8, 8, CREAM);
    t
}

/// Inside walls, 16x32 texels for a wall two units tall: crown moulding, the paper
/// pattern, a chair rail and wainscot panels.
pub fn wallpaper(style: u8) -> Texture {
    let mut t = Texture::new(16, 32, CREAM);
    let (paper, fleck, wood, wood_dark) = match style {
        0 => ([SAND, KHAKI], SHADOW, CLAY, RUST),
        1 => ([RUST, MAROON], CLAY, KHAKI, SHADOW),
        2 => ([MINT, AQUA], WHITE, CLAY, RUST),
        3 => ([PURPLE, GRAPE], CREAM, INDIGO, INK),
        4 => ([GOLD, CLAY], CREAM, RUST, MAROON),
        5 => ([CREAM, PEACH], PINK, SALMON, ROSEWOOD),
        6 => ([TEAL, DEEP_TEAL], GOLD, INDIGO, INK),
        7 => ([PEACH, SAND], PINK, CLAY, RUST),
        8 => ([SAND, PEACH], CLAY, RUST, MAROON),
        9 => ([SLATE, INDIGO], SKY, RUST, MAROON),
        _ => ([CREAM, SAND], GOLD, CRIMSON, PLUM),
    };
    for y in 0..32 {
        for x in 0..16 {
            let c = match y {
                0 => wood_dark,
                1..=2 => wood,
                3 => wood_dark,
                4..=19 => {
                    // Paper: stripes, stones or a sprinkle of motifs.
                    match style {
                        0 | 1 | 9 => {
                            // Stone or brick courses.
                            let course = (y - 4) / 4;
                            let off = if course % 2 == 0 { 0 } else { 4 };
                            if (y - 4) % 4 == 3 || (x + off) % 8 == 7 {
                                fleck
                            } else {
                                paper[(((x + off) / 8 + course) % 2) as usize]
                            }
                        }
                        4 => {
                            if x % 4 == 3 {
                                wood_dark
                            } else {
                                paper[0]
                            }
                        }
                        2 | 8 => paper[(x / 4 % 2) as usize],
                        _ => {
                            let motif = (x + y * 3) % 11 == 0;
                            if motif {
                                fleck
                            } else {
                                paper[((y / 8) % 2) as usize]
                            }
                        }
                    }
                }
                20 => wood_dark,
                21 => wood,
                22..=30 => {
                    if x % 8 == 0 || y == 22 || y == 30 {
                        wood_dark
                    } else {
                        wood
                    }
                }
                _ => wood_dark,
            };
            t.set(x, y, c);
        }
    }
    t
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tiles_only_use_palette() {
        let all = [
            grass(1, true),
            dirt_path(2),
            tilled(true),
            water(3),
            cobbles(KHAKI, SAND, SHADOW, 4),
            rock_side([KHAKI, ROSEWOOD, SHADOW, INK], 5),
            flame(1),
        ];
        for t in &all {
            assert!(t.data.iter().all(|&c| c < 32 || c == CLEAR));
        }
    }
}
