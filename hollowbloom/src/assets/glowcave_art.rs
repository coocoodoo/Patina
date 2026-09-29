//! The glowcap caves: dark rock in slate and grape strata with glowing lichen, mossy ground
//! sprinkled with spores, the ruins' old flagstones and mossy blocks, glowing pools, and the
//! mushrooms themselves: glowcaps in four colours and three sizes, shelf fungi up the walls,
//! and a giant one.

use std::f32::consts::PI;

use glam::{Vec2, Vec3};

use super::models::lathe;
use super::{ORE_COLORS, tiles};
use crate::palette::*;
use crate::render::{Mesh, TexBank, TexId, Texture, UvRect};
use crate::util::Rng;

const T: i32 = 16;

/// The glowcaps' colours, light to dark: blue, cyan, green and purple.
pub const GLOW: [[u8; 3]; 4] = [
    [SKY, BLUE, INDIGO],
    [MINT, AQUA, TEAL],
    [LIME, GREEN, TEAL],
    [BLUSH, LAVENDER, PURPLE],
];
/// The colour of the glow round each.
pub const HALO: [u8; 4] = [BLUE, AQUA, GREEN, PURPLE];
/// How warm each one's light falls (0 cold .. 8 firelight).
pub const WARMTH: [f32; 4] = [0.5, 1.2, 2.4, 1.6];

pub struct GlowcaveArt {
    /// Mossy cave ground: bare stone, two mossy, and one sprinkled with glowing spores.
    pub floor: [TexId; 4],
    /// The ruins' old flagstones.
    pub flags: [TexId; 2],
    pub rock_side: TexId,
    pub rock_top: TexId,
    /// Ore seams in that rock, by ore.
    pub ore_side: [TexId; 6],
    /// The ruins' walls: old mossy blocks.
    pub ruin_side: TexId,
    pub ruin_top: TexId,
    /// A pool's side, mossy down to the glowing waterline.
    pub bank: TexId,
    /// Glowing pool water, `water[frame][edges]`: four frames, each with a rim along the
    /// sides in `edges` that meet the shore and the corners between them rounded off (see
    /// `shore`).
    pub water: [[TexId; 16]; 4],
    /// Glowcaps by colour and size: pale stems under glowing caps, all drawn unlit.
    pub glowcaps: [[Mesh; 3]; 4],
    /// Shelf fungi by colour, growing out of a wall face at z = 0 along +z (drawn unlit).
    pub shelves: [Mesh; 4],
    /// The giant mushroom by colour: its pale stem (drawn lit) and its cap (unlit).
    pub giant_stem: [Mesh; 4],
    pub giant_cap: [Mesh; 4],
}

fn speckle(t: &mut Texture, r: &mut Rng, c: u8, n: usize) {
    for _ in 0..n {
        let (x, y) = (r.range(0, t.w as i32), r.range(0, t.h as i32));
        t.set_wrap(x, y, c);
    }
}

fn blob(t: &mut Texture, cx: i32, cy: i32, rx: i32, ry: i32, c: u8) {
    for y in -ry..=ry {
        for x in -rx..=rx {
            let (fx, fy) = (x as f32 / (rx as f32 + 0.5), y as f32 / (ry as f32 + 0.5));
            if fx * fx + fy * fy <= 1.0 {
                t.set_wrap(cx + x, cy + y, c);
            }
        }
    }
}

/// The caves' ground: cool dark stone, darker in patches, a bluish pebble here and there
/// and a fine crack; moss in patches (`kind` 1 and 2); or a sprinkle of glowing spores (3).
fn ground(kind: usize, seed: u64) -> Texture {
    let mut t = Texture::new(16, 16, SLATE);
    let mut r = Rng::new(seed);
    for _ in 0..3 {
        let (x, y) = (r.range(0, T), r.range(0, T));
        blob(&mut t, x, y, r.range(1, 3), r.range(1, 2), INK);
    }
    for _ in 0..3 {
        let (x, y) = (r.range(0, T), r.range(0, T));
        t.set_wrap(x, y, INDIGO);
        t.set_wrap(x + 1, y, INDIGO);
        t.set_wrap(x, y + 1, INK);
        t.set_wrap(x + 1, y + 1, INK);
    }
    let (mut x, mut y) = (r.range(0, T), r.range(0, T));
    for _ in 0..r.range(3, 6) {
        t.set_wrap(x, y, INK);
        x += 1;
        y += r.range(-1, 2);
    }
    match kind {
        1 | 2 => {
            for _ in 0..2 + kind {
                let (x, y) = (r.range(0, T), r.range(0, T));
                blob(&mut t, x, y, r.range(2, 4), r.range(1, 3), DEEP_TEAL);
            }
            for _ in 0..3 + kind * 2 {
                let (x, y) = (r.range(0, T), r.range(0, T));
                if t.get(x, y) == DEEP_TEAL {
                    t.set(x, y, TEAL);
                }
            }
            if kind == 2 {
                speckle(&mut t, &mut r, GREEN, 2);
            }
        }
        3 => {
            for c in [AQUA, MINT, SKY, LAVENDER, AQUA] {
                let (x, y) = (r.range(0, T), r.range(0, T));
                t.set_wrap(x, y, c);
            }
            speckle(&mut t, &mut r, DEEP_TEAL, 4);
        }
        _ => {}
    }
    t
}

/// Worn flagstones: big slabs, pale along their top edges and dark along the bottom, in dark
/// joints with moss creeping into them.
fn flagstones(seed: u64) -> Texture {
    let mut t = Texture::new(16, 16, INK);
    let mut r = Rng::new(seed);
    // (x, y, w, h), each with its joint along its right and bottom edges.
    let slabs = [(0, 0, 9, 7), (9, 0, 7, 7), (4, 7, 8, 9), (12, 7, 8, 9)];
    for (k, &(sx, sy, sw, sh)) in slabs.iter().enumerate() {
        let stone = [KHAKI, ROSEWOOD, KHAKI, SHADOW][(k + seed as usize) % 4];
        for y in 0..sh - 1 {
            for x in 0..sw - 1 {
                let c = if y == 0 || (x == 0 && y < sh - 2) {
                    if stone == SHADOW { KHAKI } else { SAND }
                } else if y == sh - 2 || x == sw - 2 {
                    if stone == SHADOW { INK } else { SHADOW }
                } else {
                    stone
                };
                t.set_wrap(sx + x, sy + y, c);
            }
        }
        // A crack, or a chip out of it.
        if r.chance(0.6) {
            let (mut x, mut y) = (sx + r.range(1, sw - 2), sy + r.range(1, sh - 2));
            for _ in 0..r.range(2, 4) {
                t.set_wrap(x, y, SHADOW);
                x += r.range(-1, 2);
                y += 1;
            }
        }
    }
    // Moss in the joints.
    for _ in 0..6 {
        let (x, y) = (r.range(0, T), r.range(0, T));
        if t.get(x, y) == INK {
            t.set(x, y, if r.chance(0.4) { TEAL } else { DEEP_TEAL });
        }
    }
    t
}

/// The caves' rock: purple and grape strata, glowing lichen in little clusters, and moss
/// hanging over the top edge.
fn rock_side() -> Texture {
    let mut t = tiles::rock_side([PURPLE, GRAPE, SLATE, INK], 0x6C51);
    let mut r = Rng::new(0x6C52);
    for _ in 0..3 {
        let (x, y) = (r.range(0, T), r.range(3, T - 2));
        t.set_wrap(x, y, AQUA);
        t.set_wrap(x + 1, y, TEAL);
        t.set_wrap(x, y + 1, DEEP_TEAL);
        if r.chance(0.5) {
            t.set_wrap(x - 1, y + 1, MINT);
        }
    }
    for x in 0..T {
        t.set(x, 0, DEEP_TEAL);
        if (x * 7) % 5 != 0 {
            t.set(x, 1, DEEP_TEAL);
        }
        if (x * 3) % 7 == 1 {
            let len = 2 + (x * 5) % 3;
            for y in 2..2 + len {
                t.set(x, y, if y == 1 + len { TEAL } else { DEEP_TEAL });
            }
        }
    }
    t
}

/// The top of the rock: purple, mottled darker, so it stands out from the dark past it
/// and from the ground below.
fn rock_top() -> Texture {
    let mut t = Texture::new(16, 16, PURPLE);
    let mut r = Rng::new(0x6C54);
    for _ in 0..4 {
        let (x, y) = (r.range(0, T), r.range(0, T));
        blob(&mut t, x, y, r.range(1, 3), 1, GRAPE);
    }
    speckle(&mut t, &mut r, SLATE, 6);
    speckle(&mut t, &mut r, DEEP_TEAL, 5);
    t
}

/// The ruins' walls: big old blocks, two courses to a tile, moss hanging from the top and
/// in the joints.
fn ruin_side() -> Texture {
    let mut t = Texture::new(16, 16, SHADOW);
    let mut r = Rng::new(0x6C5E);
    for y in 0..T {
        let course = y / 8;
        let off = if course % 2 == 0 { 0 } else { 5 };
        for x in 0..T {
            let bx = (x + off) % 10;
            let c = if y % 8 == 7 || bx == 9 {
                INK
            } else if y % 8 == 0 {
                KHAKI
            } else if y % 8 == 6 || bx == 8 {
                if (x + y) % 3 == 0 { INK } else { SHADOW }
            } else if ((x + off) / 10 + course) % 2 == 0 {
                ROSEWOOD
            } else {
                SHADOW
            };
            t.set(x, y, c);
        }
    }
    speckle(&mut t, &mut r, KHAKI, 5);
    for x in 0..T {
        t.set(x, 0, if x % 4 == 1 { TEAL } else { DEEP_TEAL });
        if x % 3 != 0 {
            t.set(x, 1, DEEP_TEAL);
        }
    }
    for _ in 0..4 {
        let (x, y) = (r.range(0, T), [7, 15][r.below(2)]);
        t.set(x, y, DEEP_TEAL);
    }
    t
}

fn ruin_top() -> Texture {
    let mut t = tiles::cobbles(SHADOW, ROSEWOOD, INK, 0x6C5F);
    let mut r = Rng::new(0x6C60);
    speckle(&mut t, &mut r, DEEP_TEAL, 5);
    t
}

/// A pool's side: stone, moss, and the glow of the water where they meet.
fn bank() -> Texture {
    let mut t = Texture::new(16, 16, SLATE);
    for x in 0..T {
        t.set(x, 12, if x % 6 == 2 { SHADOW } else { SLATE });
        t.set(x, 13, if x % 4 == 1 { TEAL } else { DEEP_TEAL });
        t.set(x, 14, if x % 3 == 0 { AQUA } else { TEAL });
        t.set(x, 15, AQUA);
    }
    t
}

/// Glowing pool water: bright cyan, slow ripples sliding across, and motes of light
/// winking in it.
fn water(frame: i32) -> Texture {
    let mut t = Texture::new(16, 16, TEAL);
    let mut r = Rng::new(0x6CA);
    for _ in 0..5 {
        let (x, y) = (r.range(0, T), r.range(0, T));
        blob(&mut t, x, y, 2, 1, AQUA);
    }
    for _ in 0..2 {
        let (x, y) = (r.range(0, T), r.range(0, T));
        blob(&mut t, x, y, 1, 1, DEEP_TEAL);
    }
    let mut s = Rng::new(0x6CB);
    for _ in 0..5 {
        let (x, y) = (s.range(0, T), s.range(0, T));
        let dx = if (y / 4) % 2 == 0 { frame } else { -frame };
        for k in 0..s.range(2, 4) {
            t.set_wrap(x + k + dx, y, MINT);
        }
    }
    let m = [(4, 11), (12, 3), (7, 14), (10, 7)];
    let (mx, my) = m[frame as usize % 4];
    t.set(mx, my, WHITE);
    t.set((mx + 7) % T, (my + 5) % T, MINT);
    t
}

/// Pool water meeting the shore on the sides in `edges` (see `sewer_art::NORTH` and so on):
/// a glowing rim along each of them, and where two meet, the corner rounded off into the
/// ground (`ground`) with the rim curving round it.
pub fn shore(water: &Texture, ground: &Texture, edges: u8) -> Texture {
    use super::sewer_art::{EAST, NORTH, SOUTH, WEST};
    let mut t = water.clone();
    let last = T - 1;
    for i in 0..T {
        for (bit, at, inner) in [
            (NORTH, (i, 0), (i, 1)),
            (SOUTH, (i, last), (i, last - 1)),
            (WEST, (0, i), (1, i)),
            (EAST, (last, i), (last - 1, i)),
        ] {
            if edges & bit != 0 {
                t.set(at.0, at.1, MINT);
                if t.get(inner.0, inner.1) != MINT {
                    t.set(inner.0, inner.1, AQUA);
                }
            }
        }
    }
    let r = 8.0f32;
    let n = T as f32;
    // Each corner: the sides that meet there, and which way it lies from the middle.
    for (a, b, sx, sy) in [
        (NORTH, WEST, -1.0, -1.0),
        (NORTH, EAST, 1.0, -1.0),
        (SOUTH, WEST, -1.0, 1.0),
        (SOUTH, EAST, 1.0, 1.0),
    ] {
        if edges & a == 0 || edges & b == 0 {
            continue;
        }
        let (cx, cy) = (n * 0.5 + sx * (n * 0.5 - r), n * 0.5 + sy * (n * 0.5 - r));
        for y in 0..T {
            for x in 0..T {
                let (px, py) = (x as f32 + 0.5, y as f32 + 0.5);
                if (px - cx) * sx <= 0.0 || (py - cy) * sy <= 0.0 {
                    continue;
                }
                let d = (px - cx).hypot(py - cy);
                if d > r + 1.0 {
                    t.set(x, y, ground.get(x, y));
                } else if d > r {
                    t.set(x, y, MINT);
                } else if d > r - 1.0 {
                    t.set(x, y, AQUA);
                }
            }
        }
    }
    t
}

/// A cap's skin, crown (top row) to gills (bottom): a bright crown, spots on the dome, a
/// glowing rim, and the gills underneath.
fn cap_skin(pal: [u8; 3], w: u32, h: u32, seed: u64) -> Texture {
    let [light, mid, dark] = pal;
    let (wi, hi) = (w as i32, h as i32);
    let mut t = Texture::new(w, h, mid);
    let (crown, rim, gills) = (hi / 8, hi * 5 / 8, hi * 3 / 4);
    for x in 0..wi {
        for y in 0..crown {
            t.set(x, y, light);
        }
        for y in rim..gills {
            t.set(x, y, light);
        }
        for y in gills..hi {
            t.set(x, y, if x % 2 == 0 { light } else { dark });
        }
    }
    let mut r = Rng::new(seed);
    let spots = (wi * hi / 60).max(3);
    for _ in 0..spots {
        let (x, y) = (r.range(0, wi), r.range(crown + 1, rim - 1));
        t.set_wrap(x, y, light);
        if hi > 16 {
            for (dx, dy) in [(1, 0), (0, 1), (-1, 0), (0, -1)] {
                t.set_wrap(x + dx, y + dy, light);
            }
        }
        t.set_wrap(x, y, WHITE);
    }
    t
}

/// A glowing shroomling's cap: a bright crown ringed with glowing spots (it's seen from
/// above, so they're all near the top), and a dark rim underneath.
pub fn shroom_cap([light, mid, dark]: [u8; 3]) -> Texture {
    let mut t = Texture::new(16, 16, mid);
    for x in 0..T {
        if x % 4 == 0 {
            t.set(x, 0, light);
        }
        t.set(x, 14, dark);
        t.set(x, 15, dark);
    }
    for (x, y) in [(1, 2), (7, 1), (12, 2), (4, 4), (10, 4), (14, 5)] {
        t.set(x, y, WHITE);
        t.set_wrap(x + 1, y, light);
        t.set_wrap(x - 1, y, light);
    }
    t
}

/// A mushroom cap round `c`: the gills from the stem out to the rim, then the dome up to
/// the crown, with the whole of its skin (`rows` of it) from the crown down to the gills.
#[allow(clippy::too_many_arguments)]
fn cap(
    m: &mut Mesh,
    c: Vec3,
    stem: f32,
    rad: f32,
    tall: f32,
    seg: usize,
    (cols, rows): (f32, f32),
    tex: TexId,
) {
    let v = |k: f32| k * rows;
    // Little caps are too small to show a rim of their own: just gills and a dome.
    let prof: &[(f32, f32, f32)] = if rad < 0.1 {
        &[
            (stem, 0.05 * tall, v(0.999)),
            (rad, 0.0, v(0.7)),
            (rad * 0.85, 0.6 * tall, v(0.3)),
            (0.0, tall, 0.0),
        ]
    } else {
        &[
            (stem, 0.05 * tall, v(0.999)),
            (rad, 0.0, v(0.75)),
            (rad * 1.04, 0.2 * tall, v(0.64)),
            (rad * 0.92, 0.55 * tall, v(0.4)),
            (rad * 0.6, 0.86 * tall, v(0.18)),
            (0.0, tall, 0.0),
        ]
    };
    m.lathe(c, prof, seg, cols, 0.3, tex, (None, None));
}

/// A glowcap cluster's mushrooms: (x, z, height, cap radius).
const CLUSTERS: [&[(f32, f32, f32, f32)]; 3] = [
    &[
        (0.0, 0.02, 0.17, 0.08),
        (0.14, 0.1, 0.12, 0.06),
        (-0.13, 0.11, 0.1, 0.055),
        (0.06, -0.14, 0.13, 0.065),
        (-0.16, -0.08, 0.08, 0.045),
    ],
    &[
        (0.0, 0.0, 0.38, 0.17),
        (0.21, 0.13, 0.25, 0.12),
        (-0.19, 0.1, 0.21, 0.1),
        (0.1, -0.2, 0.12, 0.06),
        (-0.22, -0.12, 0.09, 0.05),
    ],
    &[
        (0.0, 0.0, 0.78, 0.31),
        (0.27, 0.17, 0.26, 0.11),
        (-0.24, 0.16, 0.18, 0.09),
        (0.12, -0.25, 0.11, 0.055),
    ],
];

/// A shelf fungus bracket: a flat half-disc sticking out of a wall face (z = 0) along +z
/// at `at`, banded on top like a tree's rings and glowing round its edge.
fn bracket(m: &mut Mesh, at: Vec3, rad: f32, thick: f32, top: TexId, edge: TexId) {
    const N: usize = 7;
    let rim = |i: usize, y: f32| {
        let a = i as f32 / N as f32 * PI;
        Vec3::new(at.x + rad * a.cos(), y, at.z + rad * 0.85 * a.sin())
    };
    let (lo, hi) = (at.y, at.y + thick);
    let c = Vec3::new(at.x, hi, at.z);
    for i in 0..N {
        let (p0, p1) = (rim(i, hi), rim(i + 1, hi));
        let uv = |i: usize| {
            let a = i as f32 / N as f32 * PI;
            Vec2::new(8.0 + a.cos() * 7.0, 15.5)
        };
        m.tri([c, p1, p0], [Vec2::new(8.0, 0.5), uv(i + 1), uv(i)], top);
        m.quad(
            [rim(i + 1, lo), rim(i, lo), p0, p1],
            UvRect::new(0.0, 0.0, 4.0, 4.0),
            edge,
        );
    }
}

/// A shelf fungus's top: dark by the wall, then its colour, a dark ring, and a glowing rim.
fn shelf_skin(pal: [u8; 3]) -> Texture {
    let [light, mid, dark] = pal;
    let mut t = Texture::new(16, 16, mid);
    for x in 0..T {
        for y in 0..T {
            let c = match y {
                0..=4 => dark,
                11 | 12 => dark,
                13..=15 => light,
                _ if (x * 5 + y * 3) % 11 == 0 => light,
                _ => mid,
            };
            t.set(x, y, c);
        }
    }
    t
}

/// A pale stem glowing faintly in its cap's colour, streaked with fibres and darker at the
/// foot.
fn stem_skin([light, mid, _]: [u8; 3]) -> Texture {
    let mut t = Texture::new(16, 16, WHITE);
    for y in 0..T {
        for x in 0..T {
            if x % 3 == 1 || y >= T - 3 && (x + y) % 2 == 0 {
                t.set(x, y, light);
            }
        }
    }
    for x in 0..T {
        t.set(x, T - 1, mid);
    }
    t
}

pub fn build(bank_: &mut TexBank) -> GlowcaveArt {
    let floor = std::array::from_fn(|k| bank_.add(ground(k, 0x6C10 + k as u64)));
    let flags = [bank_.add(flagstones(0x6CF1)), bank_.add(flagstones(0x6CF2))];
    let side = rock_side();
    let rock_side = bank_.add(side.clone());
    let rock_top = bank_.add(rock_top());
    let ore_side = std::array::from_fn(|o| {
        bank_.add(tiles::with_ore(&side, ORE_COLORS[o], 0x6C70 + o as u64))
    });
    let ruin_side = bank_.add(ruin_side());
    let ruin_top = bank_.add(ruin_top());
    let bank = bank_.add(bank());
    let bare = ground(0, 0x6C10);
    let water = std::array::from_fn(|f| {
        let frame = water(f as i32);
        std::array::from_fn(|edges| bank_.add(shore(&frame, &bare, edges as u8)))
    });

    // Glowcaps: pale stems, and glowing caps in every colour.
    let glowcaps = std::array::from_fn(|c| {
        let stem = bank_.add(stem_skin(GLOW[c]));
        let skin = bank_.add(cap_skin(GLOW[c], 16, 16, 0x6CC0 + c as u64));
        std::array::from_fn(|size| {
            let mut m = Mesh::new();
            for &(x, z, h, r) in CLUSTERS[size] {
                let sr = (r * 0.28).max(0.018);
                let seg = if r > 0.2 {
                    10
                } else if r > 0.09 {
                    7
                } else {
                    5
                };
                lathe(
                    &mut m,
                    Vec3::new(x, 0.0, z),
                    &[(sr * 1.15, 0.0), (sr * 0.9, h - r * 0.4)],
                    seg.min(6),
                    0.0,
                    stem,
                    false,
                );
                cap(
                    &mut m,
                    Vec3::new(x, h - r * 0.5, z),
                    sr,
                    r,
                    r * 0.72,
                    seg,
                    (16.0, 16.0),
                    skin,
                );
            }
            m
        })
    });

    // Shelf fungi: a few brackets stacked up the wall.
    let shelves = std::array::from_fn(|c| {
        let top = bank_.add(shelf_skin(GLOW[c]));
        let edge = bank_.add(Texture::new(4, 4, GLOW[c][0]));
        let mut m = Mesh::new();
        for (x, y, r) in [
            (-0.2, 0.6, 0.17),
            (0.1, 0.42, 0.23),
            (0.29, 0.72, 0.12),
            (-0.04, 0.84, 0.1),
        ] {
            bracket(&mut m, Vec3::new(x, y, 0.0), r, 0.045, top, edge);
        }
        m
    });

    // The giant: a thick pale stem flaring out at its foot, under a great cap.
    let giant_stem = std::array::from_fn(|c| {
        let stem = bank_.add(stem_skin(GLOW[c]));
        let mut m = Mesh::new();
        lathe(
            &mut m,
            Vec3::ZERO,
            &[
                (0.95, 0.0),
                (0.72, 0.14),
                (0.56, 0.5),
                (0.5, 1.0),
                (0.53, 1.45),
                (0.62, 1.72),
            ],
            12,
            0.0,
            stem,
            false,
        );
        m
    });
    let giant_cap = std::array::from_fn(|c| {
        let skin = bank_.add(cap_skin(GLOW[c], 32, 32, 0x6CE0 + c as u64));
        let mut m = Mesh::new();
        cap(
            &mut m,
            Vec3::Y * 1.62,
            0.6,
            2.0,
            1.12,
            16,
            (64.0, 32.0),
            skin,
        );
        m
    });

    GlowcaveArt {
        floor,
        flags,
        rock_side,
        rock_top,
        ore_side,
        ruin_side,
        ruin_top,
        bank,
        water,
        glowcaps,
        shelves,
        giant_stem,
        giant_cap,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn caps_glow_crown_rim_and_gills() {
        let t = cap_skin(GLOW[1], 16, 16, 1);
        assert_eq!(t.get(3, 0), MINT, "a bright crown");
        assert_eq!(t.get(3, 10), MINT, "a glowing rim");
        assert_eq!(t.get(2, 13), MINT);
        assert_eq!(t.get(3, 13), TEAL, "gills");
    }

    #[test]
    fn everything_stays_in_the_palette() {
        for t in [
            ground(0, 1),
            ground(1, 2),
            ground(2, 3),
            ground(3, 4),
            flagstones(5),
            rock_side(),
            rock_top(),
            ruin_side(),
            ruin_top(),
            bank(),
            water(2),
            shelf_skin(GLOW[3]),
            shroom_cap(GLOW[0]),
            stem_skin(GLOW[2]),
            shore(&water(1), &ground(0, 7), 15),
            cap_skin(GLOW[0], 32, 32, 9),
        ] {
            for y in 0..t.h as i32 {
                for x in 0..t.w as i32 {
                    assert!(t.get(x, y) < 32, "({x}, {y})");
                }
            }
        }
    }
}
