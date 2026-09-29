//! The Sunscorch Canyon: cliffs of layered red sandstone and paler banded rock, drifts of
//! rippled sand, quicksand, the tombs' carved blocks and flagstones; and what's found down
//! there: pillars with sand pouring off them, great bleached skulls, cacti, stone coffins,
//! a wyvern's bones, heaps of gold, and the ancient gateway you come in by.

use glam::{Mat4, Quat, Vec2, Vec3};

use super::models::{lathe, skin_box, tiled_box};
use super::monster_art::paint;
use crate::palette::*;
use crate::render::{Mesh, TexBank, TexId, Texture, UvRect};
use crate::util::{Rng, hash2};

const T: i32 = 16;

pub struct CanyonArt {
    /// Cliff faces by look (see `canyon::STRATA` and so on), a power of two tall so the
    /// terraces' strata run on down them.
    pub side: [TexId; 3],
    pub top: [TexId; 3],
    /// Rippled sand: plain, tracked by little feet, pebbly, and a pale drift.
    pub sand: [TexId; 4],
    /// The tombs' flagstones.
    pub flags: [TexId; 2],
    /// Quicksand, swirling round, by frame.
    pub quicksand: [TexId; 4],
    /// Where quicksand sinks below the sand round it.
    pub bank: TexId,
    /// A sandstone pillar, its sand pile at its foot.
    pub sandfall: Mesh,
    /// Skulls by `canyon::HORNED` and so on.
    pub skulls: [Mesh; 3],
    /// Cacti by `canyon::SAGUARO` and so on.
    pub cacti: [Mesh; 3],
    pub sarcophagus: Mesh,
    /// The wyvern's bones, round the middle of their six tiles.
    pub wyvern: Mesh,
    pub gold: Mesh,
    /// The gateway: posts one tile either side of its middle, the lintel over them.
    pub gateway: Mesh,
    /// The labyrinth's columns, carved from sandstone for the tombs.
    pub columns: [Mesh; 3],
}

/// Layered rock: bands of colour running across, each boundary wandering a little and
/// worn into pits, with sand drifted along the top edge.
fn strata(bands: &[(i32, u8)], seed: u64) -> Texture {
    let mut t = Texture::new(16, 32, bands[0].1);
    let mut r = Rng::new(seed);
    for x in 0..T {
        let wander = (hash2(x / 3, 1, seed as u32) % 3) as i32 - 1;
        let mut y = 0;
        for &(thick, c) in bands.iter().cycle() {
            for k in 0..thick {
                let yy = y + k + wander;
                if (0..32).contains(&yy) {
                    t.set(x, yy, c);
                }
            }
            y += thick;
            if y >= 34 {
                break;
            }
        }
    }
    // Worn pits and chips here and there.
    for _ in 0..14 {
        let (x, y) = (r.range(0, T), r.range(2, 32));
        let c = t.get(x, y);
        t.set(x, y, SHADE[c as usize]);
    }
    // Sand drifted over the top.
    for x in 0..T {
        t.set(x, 0, SAND);
        if (x * 5) % 7 < 3 {
            t.set(x, 1, GOLD);
        }
    }
    t
}

/// The top of the canyon rock: warm stone, sand drifted across it, and a crack or two.
fn rock_top(pal: [u8; 3], seed: u64) -> Texture {
    let [light, mid, dark] = pal;
    let mut t = Texture::new(16, 16, mid);
    let mut r = Rng::new(seed);
    for _ in 0..3 {
        let (cx, cy) = (r.range(0, T), r.range(0, T));
        for k in 0..r.range(4, 8) {
            t.set_wrap(cx + k, cy + (k / 3), light);
            t.set_wrap(cx + k, cy + (k / 3) + 1, GOLD);
        }
    }
    for _ in 0..6 {
        let (x, y) = (r.range(0, T), r.range(0, T));
        t.set(x, y, dark);
    }
    let (mut x, mut y) = (r.range(0, T), r.range(0, T));
    for _ in 0..5 {
        t.set_wrap(x, y, dark);
        x += 1;
        y += r.range(-1, 2);
    }
    t
}

/// The tombs' walls: big carved blocks with glyphs cut into them, under a band.
fn tomb_side() -> Texture {
    let mut t = Texture::new(16, 32, SAND);
    for y in 0..32 {
        for x in 0..T {
            let row = y / 8;
            let off = if row % 2 == 0 { 0 } else { 4 };
            let c = if y % 8 == 7 || (x + off) % 8 == 7 {
                KHAKI
            } else {
                SAND
            };
            t.set(x, y, c);
        }
    }
    // Glyphs: little birds, eyes, reeds and suns, one to a block.
    let glyphs: [&[(i32, i32)]; 4] = [
        &[(0, 0), (1, 0), (2, 1), (1, 2), (0, 2)],
        &[(0, 1), (1, 0), (2, 0), (3, 1), (2, 2), (1, 2), (1, 1)],
        &[(1, 0), (1, 1), (1, 2), (0, 2), (2, 2)],
        &[(1, 0), (0, 1), (2, 1), (1, 2), (1, 1)],
    ];
    for (k, (bx, by)) in [
        (2, 2),
        (10, 2),
        (6, 10),
        (14, 10),
        (2, 18),
        (10, 18),
        (6, 26),
    ]
    .iter()
    .enumerate()
    {
        for &(dx, dy) in glyphs[k % 4] {
            t.set_wrap(bx + dx, by + dy, if k % 3 == 0 { RUST } else { MAROON });
        }
    }
    for x in 0..T {
        t.set(x, 0, CLAY);
        t.set(x, 1, GOLD);
        t.set(x, 2, CLAY);
    }
    t
}

fn tomb_top() -> Texture {
    let mut t = Texture::new(16, 16, SAND);
    for i in 0..T {
        t.set(i, 0, KHAKI);
        t.set(0, i, KHAKI);
    }
    for (x, y) in [(4, 5), (11, 8), (6, 12)] {
        t.set(x, y, GOLD);
    }
    t
}

/// Rippled sand: gold, with pale crests and dashes of shade curving across; now and then
/// the tracks of something small (`kind` 1), pebbles (2), or a soft patch of paler sand
/// blown in (3).
fn sand(kind: usize, seed: u64) -> Texture {
    let mut t = Texture::new(16, 16, GOLD);
    let mut r = Rng::new(seed);
    // A drift's patch, round in the middle of the tile, its edge dithered away.
    let pale = |x: i32, y: i32| {
        let (dx, dy) = ((x as f32 - 7.5) / 6.5, (y as f32 - 7.5) / 5.5);
        let d = dx * dx + dy * dy;
        kind == 3 && (d < 0.6 || d < 1.0 && (x + y) % 2 == 0)
    };
    // Two ripples to a tile, their curve running on seamlessly into the next tile's.
    let phase = (seed % 16) as f32;
    for y in 0..T {
        for x in 0..T {
            let pale = pale(x, y);
            if pale {
                t.set(x, y, SAND);
            }
            let wave = ((x as f32 + phase) / 16.0 * std::f32::consts::TAU).sin() * 1.5;
            let row = y + wave.round() as i32;
            match row.rem_euclid(8) {
                0 if (x + y) % 3 != 0 => t.set(x, y, if pale { WHITE } else { SAND }),
                1 if (x + 4 * row.div_euclid(8)).rem_euclid(8) < 6 => {
                    t.set(x, y, if pale { GOLD } else { CLAY })
                }
                _ => {}
            }
        }
    }
    match kind {
        1 => {
            for (x, y) in [(3, 2), (6, 5), (4, 8), (7, 11), (5, 14)] {
                t.set_wrap(x, y, RUST);
                t.set_wrap(x + 1, y, RUST);
                t.set_wrap(x, y + 1, CLAY);
            }
        }
        2 => {
            for _ in 0..5 {
                let (x, y) = (r.range(0, T), r.range(0, T));
                t.set(x, y, RUST);
                t.set_wrap(x + 1, y, KHAKI);
            }
        }
        _ => {
            for _ in 0..3 {
                let (x, y) = (r.range(0, T), r.range(0, T));
                t.set(x, y, if pale(x, y) { WHITE } else { SAND });
            }
        }
    }
    t
}

/// The tombs' old flagstones, dusted with sand.
fn tomb_flags(seed: u64) -> Texture {
    let mut t = Texture::new(16, 16, KHAKI);
    let mut r = Rng::new(seed);
    for &(sx, sy, sw, sh) in &[(0, 0, 9, 8), (9, 0, 7, 8), (0, 8, 6, 8), (6, 8, 10, 8)] {
        let stone = if r.chance(0.3) { GOLD } else { SAND };
        for y in 0..sh - 1 {
            for x in 0..sw - 1 {
                t.set_wrap(sx + x, sy + y, stone);
            }
        }
        // A chipped corner.
        t.set_wrap(
            sx + sw - 2,
            sy + sh - 2,
            if stone == SAND { KHAKI } else { CLAY },
        );
    }
    for _ in 0..8 {
        let (x, y) = (r.range(0, T), r.range(0, T));
        if t.get(x, y) == SAND {
            t.set(x, y, GOLD);
        }
    }
    t
}

/// Quicksand: darker sand swirling slowly round, frame by frame.
fn quicksand(frame: usize) -> Texture {
    let mut t = Texture::new(16, 16, CLAY);
    let turn = frame as f32 * std::f32::consts::FRAC_PI_2 * 0.5;
    for y in 0..T {
        for x in 0..T {
            let (dx, dy) = (x as f32 - 7.5, y as f32 - 7.5);
            let rr = dx.hypot(dy);
            let a = dy.atan2(dx) + turn;
            let k = (rr * 0.9 - a * 1.4).rem_euclid(std::f32::consts::TAU);
            let c = if k < 0.7 {
                GOLD
            } else if k < 1.4 {
                SAND
            } else if k > 5.2 {
                RUST
            } else {
                CLAY
            };
            t.set(x, y, c);
        }
    }
    t
}

/// A sunken edge of sand over quicksand.
fn bank() -> Texture {
    let mut t = Texture::new(16, 16, CLAY);
    for x in 0..T {
        t.set(x, 12, GOLD);
        t.set(x, 13, if x % 3 == 0 { RUST } else { CLAY });
        t.set(x, 14, RUST);
        t.set(x, 15, MAROON);
    }
    t
}

/// A cactus's skin: ribs of green running up it, spines along them.
fn ribs() -> Texture {
    let mut t = Texture::new(16, 16, GREEN);
    for y in 0..T {
        for x in 0..T {
            let c = match x % 4 {
                0 => DEEP_TEAL,
                1 => TEAL,
                _ => GREEN,
            };
            t.set(x, y, c);
            if x % 4 == 2 && y % 3 == 0 {
                t.set(x, y, WHITE);
            }
        }
    }
    t
}

/// Bleached bone: white, yellowed in patches, a crack wandering across it.
fn bone() -> Texture {
    let mut t = Texture::new(16, 16, WHITE);
    for y in 0..T {
        for x in 0..T {
            if hash2(x / 2, y / 2, 0xB0E) % 6 == 0 {
                t.set(x, y, SAND);
            }
        }
    }
    let mut y = 4;
    for x in 2..12 {
        t.set(x, y, KHAKI);
        if x % 3 == 0 {
            y += 1;
        }
    }
    t.set(8, 8, KHAKI);
    t.set(9, 9, KHAKI);
    t
}

/// Older bone, yellowed by the sand and pitted, for jaws and muzzles down in it.
fn worn_bone() -> Texture {
    let mut t = Texture::new(16, 16, SAND);
    for y in 0..T {
        for x in 0..T {
            match hash2(x, y, 0xB0F) % 9 {
                0 | 1 => t.set(x, y, WHITE),
                2 => t.set(x, y, KHAKI),
                _ => {}
            }
        }
    }
    t
}

/// Ridged horn, running (as a tube lays it) from its tip at the top: dark at the point,
/// then rings of grooves and paler crests.
fn horn_ridges() -> Texture {
    let mut t = Texture::new(16, 16, KHAKI);
    for y in 0..T {
        for x in 0..T {
            let c = match y {
                0 | 1 => SHADOW,
                2 | 3 => ROSEWOOD,
                _ => match y % 4 {
                    0 => ROSEWOOD,
                    1 if x % 5 != 0 => SAND,
                    _ => KHAKI,
                },
            };
            t.set(x, y, c);
        }
    }
    t
}

/// Where each picture sits on the features sheet.
const SOCKET: UvRect = UvRect::px(0, 0, 8, 8);
const NOSE: UvRect = UvRect::px(8, 0, 8, 8);
const FACE: UvRect = UvRect::px(0, 8, 8, 8);
/// A glaring socket, its brow falling towards the nose: the left eye as you look at it.
const GLARE: UvRect = UvRect::px(8, 8, 8, 8);
const GLARE_R: UvRect = UvRect::new(16.0, 8.0, 8.0, 16.0);

/// The pictures laid over skulls: a deep round socket, a nose hole, a little skull's whole
/// face, and a glaring socket.
fn features() -> Texture {
    let mut t = Texture::new(16, 16, WHITE);
    let legend = [
        ('w', WHITE),
        ('s', SAND),
        ('k', KHAKI),
        ('i', INK),
        ('r', ROSEWOOD),
    ];
    paint(
        &mut t,
        0,
        0,
        &[
            "wskkkksw", "skiiiiks", "kiiiiiik", "kiiiiiik", "kiiiiiik", "kiiiiiik", "srkiikrs",
            "wsrkkrsw",
        ],
        &legend,
    );
    paint(
        &mut t,
        8,
        0,
        &[
            "wkkwwkkw", "kiikkiik", "kiiiiiik", "wkiiiikw", "wkiiiikw", "wwkiikww", "wwkiikww",
            "wwskksww",
        ],
        &legend,
    );
    paint(
        &mut t,
        0,
        8,
        &[
            "swwwwwws", "wiiwwiiw", "wiiwwiiw", "swwiiwws", "sswwwwss", "swiwiwis", "swiwiwis",
            "sskkkkss",
        ],
        &legend,
    );
    paint(
        &mut t,
        8,
        8,
        &[
            "kkwwwwww", "iikkwwww", "iiiikkww", "iiiiiikw", "iiiiiiik", "iiiiiiik", "riiiiikw",
            "srkkkkww",
        ],
        &legend,
    );
    t
}

/// Old gold, heaped: coins lying every which way, each catching the light on one edge.
fn coins() -> Texture {
    let mut t = Texture::new(16, 16, GOLD);
    let coin = ["gccg", "cggr", "cggr", "grrg"];
    for cy in 0..4 {
        for cx in 0..4 {
            let (x0, y0) = (cx * 4 + (cy % 2) * 2, cy * 4);
            for (dy, row) in coin.iter().enumerate() {
                for (dx, ch) in row.chars().enumerate() {
                    let c = match ch {
                        'c' => CREAM,
                        'r' => CLAY,
                        _ => continue,
                    };
                    t.set_wrap(x0 + dx as i32, y0 + dy as i32, c);
                }
            }
        }
    }
    for (x, y) in [(1, 1), (11, 6), (6, 13)] {
        t.set(x, y, WHITE);
    }
    t
}

/// Coins in a stack, seen edge on, a coin to each pair of texels from row 8 down; the top
/// four rows are a coin's face, for the top of the stack.
fn coin_edges() -> Texture {
    let mut t = Texture::new(16, 16, GOLD);
    for y in 8..T {
        for x in 0..T {
            let c = if y % 2 == 1 {
                CLAY
            } else if (x * 3 + y) % 7 == 0 {
                CREAM
            } else {
                GOLD
            };
            t.set(x, y, c);
        }
    }
    for (x, y, c) in [
        (0, 0, CLAY),
        (3, 0, CLAY),
        (0, 3, CLAY),
        (3, 3, CLAY),
        (1, 1, CREAM),
        (2, 2, CLAY),
    ] {
        t.set(x, y, c);
    }
    t
}

/// A carved mummy's wrappings, lying along +x: bands of gold across, and down the middle
/// a strip of glyphs between gilded lines.
fn mummy_wraps() -> Texture {
    let mut t = Texture::new(16, 16, SAND);
    for x in 0..T {
        t.set(x, 0, KHAKI);
        t.set(x, 15, KHAKI);
        t.set(x, 5, GOLD);
        t.set(x, 10, GOLD);
        for y in 6..10 {
            let glyph = hash2(x, y, 0x61F) % 3 == 0 && x % 4 != 3;
            t.set(
                x,
                y,
                if glyph {
                    if y % 2 == 0 { RUST } else { MAROON }
                } else {
                    CREAM
                },
            );
        }
    }
    for x in [2, 13] {
        for y in (0..T).filter(|y| !(5..=10).contains(y)) {
            t.set(x, y, GOLD);
        }
    }
    t
}

/// A broad collar of beads: rows of turquoise, gold and carnelian across the chest.
fn jewelled_collar() -> Texture {
    let mut t = Texture::new(16, 16, GOLD);
    for y in 0..T {
        for x in 0..T {
            let c = match x % 5 {
                0 => AQUA,
                2 => RED,
                4 => TEAL,
                _ => GOLD,
            };
            t.set(x, y, if y % 4 == 3 && x % 5 != 1 { CREAM } else { c });
        }
    }
    t
}

/// The striped headdress: gold and lapis.
fn nemes_stripes() -> Texture {
    let mut t = Texture::new(16, 16, GOLD);
    for y in 0..T {
        for x in 0..T {
            if y % 3 == 2 {
                t.set(x, y, INDIGO);
            }
        }
    }
    t
}

/// A golden face, lying looking up with its brow towards +x: kohl-lined eyes, a nose, and
/// a small mouth.
fn golden_mask() -> Texture {
    let mut t = Texture::new(8, 8, GOLD);
    paint(
        &mut t,
        0,
        0,
        &[
            "gggggggg", "gggggrgg", "ggggrkgg", "grgcgggg", "grgrgggg", "ggggrkgg", "gggggrgg",
            "gggggggg",
        ],
        &[('g', GOLD), ('r', CLAY), ('k', INK), ('c', CREAM)],
    );
    t
}

/// A stick of bone from `a` to `b`.
fn stick(m: &mut Mesh, a: Vec3, b: Vec3, w: f32, tex: TexId) {
    let len = (b - a).length();
    let mut s = Mesh::new();
    skin_box(
        &mut s,
        Vec3::new(-w, 0.0, -w),
        Vec3::new(w, len, w),
        tex,
        &Texture::new(4, 4, 0),
    );
    let rot = Quat::from_rotation_arc(Vec3::Y, (b - a).normalize());
    m.append(&s, Mat4::from_translation(a) * Mat4::from_quat(rot));
}

/// A horn (or a fang): a cone from `at` along `dir`.
fn horn(m: &mut Mesh, at: Vec3, dir: Vec3, len: f32, r: f32, tex: TexId) {
    let mut p = Mesh::new();
    lathe(
        &mut p,
        Vec3::ZERO,
        &[(r, 0.0), (0.0, len)],
        5,
        0.4,
        tex,
        false,
    );
    let rot = Quat::from_rotation_arc(Vec3::Y, dir.normalize());
    m.append(&p, Mat4::from_translation(at) * Mat4::from_quat(rot));
}

/// A tube swept along `pts`, as thick as `radii` at each: horns, fangs and curved bones.
/// Its texture runs along it from the far end, so a horn's tip takes the top of the texture.
fn tube(m: &mut Mesh, pts: &[Vec3], radii: &[f32], seg: usize, tex: TexId) {
    let n = pts.len();
    let mut v = vec![0.0f32; n];
    for i in (0..n - 1).rev() {
        v[i] = v[i + 1] + (pts[i + 1] - pts[i]).length() * 16.0;
    }
    let along = |i: usize| (pts[(i + 1).min(n - 1)] - pts[i.saturating_sub(1)]).normalize();
    // Carry a frame along the curve so the tube never twists.
    let mut a = along(0).any_orthonormal_vector();
    let rings: Vec<Vec<Vec3>> = (0..n)
        .map(|i| {
            let t = along(i);
            a = (a - t * a.dot(t)).normalize();
            let b = t.cross(a);
            (0..seg)
                .map(|j| {
                    let th = j as f32 / seg as f32 * std::f32::consts::TAU;
                    pts[i] + (a * th.cos() + b * th.sin()) * radii[i].max(0.003)
                })
                .collect()
        })
        .collect();
    for i in 0..n - 1 {
        for j in 0..seg {
            let k = (j + 1) % seg;
            let (u0, u1) = (
                j as f32 * 16.0 / seg as f32,
                (j + 1) as f32 * 16.0 / seg as f32,
            );
            m.quad(
                [rings[i][j], rings[i][k], rings[i + 1][k], rings[i + 1][j]],
                UvRect::new(u0, v[i + 1], u1, v[i]),
                tex,
            );
        }
    }
    // Close the ends.
    let dot = [Vec2::new(8.5, 8.5); 3];
    for j in 0..seg {
        let k = (j + 1) % seg;
        m.tri([pts[0], rings[0][k], rings[0][j]], dot, tex);
        m.tri([pts[n - 1], rings[n - 1][j], rings[n - 1][k]], dot, tex);
    }
}

/// A block from its eight corners, numbered by bits: 1 for +x, 2 for +y, 4 for +z (as
/// `Mesh::cube` lays its faces), textured at world density.
fn block(m: &mut Mesh, c: [Vec3; 8], tex: TexId) {
    const FACES: [[usize; 4]; 6] = [
        [5, 1, 3, 7],
        [0, 4, 6, 2],
        [6, 7, 3, 2],
        [0, 1, 5, 4],
        [4, 5, 7, 6],
        [1, 0, 2, 3],
    ];
    for f in FACES {
        let p = f.map(|i| c[i]);
        let (w, h) = ((p[1] - p[0]).length() * 16.0, (p[3] - p[0]).length() * 16.0);
        m.quad(p, UvRect::new(0.0, 0.0, w, h), tex);
    }
}

/// A picture off the features sheet laid over a face looking along +z.
fn decal(m: &mut Mesh, c: Vec3, half: Vec2, r: UvRect, tex: TexId) {
    let (x0, x1, y0, y1) = (c.x - half.x, c.x + half.x, c.y - half.y, c.y + half.y);
    m.quad(
        [
            Vec3::new(x0, y0, c.z),
            Vec3::new(x1, y0, c.z),
            Vec3::new(x1, y1, c.z),
            Vec3::new(x0, y1, c.z),
        ],
        r,
        tex,
    );
}

/// A decal on a face looking along +z turned `yaw` about y.
fn decal_turned(m: &mut Mesh, c: Vec3, half: Vec2, r: UvRect, tex: TexId, yaw: f32) {
    let mut d = Mesh::new();
    decal(&mut d, Vec3::ZERO, half, r, tex);
    m.append(&d, Mat4::from_translation(c) * Mat4::from_rotation_y(yaw));
}

/// What skulls are made of.
#[derive(Clone, Copy)]
struct Bones {
    bone: TexId,
    worn: TexId,
    horn: TexId,
    features: TexId,
}

/// A great horned beast's skull, `s` big: its long face tipped back to look up at you and
/// its muzzle sunk in the sand, deep sockets out at the sides under a heavy brow, a crack
/// down its forehead, and ridged horns sweeping out and up.
fn horned_skull(s: f32, t: Bones) -> Mesh {
    let v = |x: f32, y: f32, z: f32| Vec3::new(x, y, z) * s;
    // Built standing up, looking along +z, then tipped back.
    let mut k = Mesh::new();
    // A block narrowing from `hx1` wide at `y1` to `hx0` at `y0`.
    let narrowing = |y0: f32, hx0: f32, y1: f32, hx1: f32, z0: f32, z1: f32| -> [Vec3; 8] {
        std::array::from_fn(|i| {
            let (y, hx) = if i & 2 != 0 { (y1, hx1) } else { (y0, hx0) };
            v(
                if i & 1 != 0 { hx } else { -hx },
                y,
                if i & 4 != 0 { z1 } else { z0 },
            )
        })
    };
    // The poll between the horns, and the knob atop it.
    tiled_box(&mut k, v(-0.19, 0.52, -0.2), v(0.19, 0.63, 0.04), t.bone, 0);
    tiled_box(
        &mut k,
        v(-0.13, 0.63, -0.16),
        v(0.13, 0.665, 0.0),
        t.bone,
        0,
    );
    // The forehead narrowing down to the muzzle, and the muzzle to its flared nose.
    block(
        &mut k,
        narrowing(0.26, 0.125, 0.53, 0.17, -0.18, 0.06),
        t.bone,
    );
    block(
        &mut k,
        narrowing(-0.02, 0.085, 0.28, 0.12, -0.15, 0.035),
        t.worn,
    );
    tiled_box(
        &mut k,
        v(-0.1, -0.08, -0.13),
        v(0.1, 0.015, 0.045),
        t.worn,
        0,
    );
    // A crack running down the forehead, forking.
    for (x, y0, y1) in [
        (0.0f32, 0.36f32, 0.6f32),
        (0.03, 0.3, 0.37),
        (-0.035, 0.44, 0.5),
    ] {
        tiled_box(
            &mut k,
            v(x - 0.008, y0, 0.05),
            v(x + 0.008, y1, 0.066),
            t.horn,
            0,
        );
    }
    for sx in [-1.0f32, 1.0] {
        // The orbits standing out at the sides, their sockets deep, a heavy brow over them.
        let (x0, x1) = (sx * 0.13, sx * 0.255);
        tiled_box(
            &mut k,
            v(x0.min(x1), 0.33, -0.12),
            v(x0.max(x1), 0.47, 0.07),
            t.bone,
            0,
        );
        let (b0, b1) = (sx * 0.115, sx * 0.27);
        tiled_box(
            &mut k,
            v(b0.min(b1), 0.45, -0.12),
            v(b0.max(b1), 0.505, 0.09),
            t.bone,
            0,
        );
        decal(
            &mut k,
            v(sx * 0.1925, 0.395, 0.0725),
            Vec2::new(0.055, 0.058) * s,
            SOCKET,
            t.features,
        );
        // The cheekbones below them.
        let (c0, c1) = (sx * 0.1, sx * 0.19);
        tiled_box(
            &mut k,
            v(c0.min(c1), 0.25, -0.14),
            v(c0.max(c1), 0.335, 0.03),
            t.worn,
            0,
        );
        // Nostrils.
        decal(
            &mut k,
            v(sx * 0.045, -0.035, 0.0475),
            Vec2::new(0.03, 0.03) * s,
            SOCKET,
            t.features,
        );
        // The horns.
        let h = |x: f32, y: f32, z: f32| v(sx * x, y, z);
        tube(
            &mut k,
            &[
                h(0.15, 0.575, -0.08),
                h(0.28, 0.585, -0.08),
                h(0.39, 0.62, -0.05),
                h(0.46, 0.7, 0.0),
                h(0.49, 0.8, 0.05),
                h(0.475, 0.9, 0.09),
            ],
            &[0.075, 0.068, 0.058, 0.045, 0.028, 0.006].map(|r| r * s),
            7,
            t.horn,
        );
    }
    // The hollow of the nose between the nostrils.
    decal(
        &mut k,
        v(0.0, 0.08, 0.0375),
        Vec2::new(0.045, 0.07) * s,
        NOSE,
        t.features,
    );
    let mut m = Mesh::new();
    m.append(
        &k,
        Mat4::from_translation(v(0.0, 0.0, 0.2)) * Mat4::from_rotation_x(-0.8),
    );
    m
}

/// The wyvern's skull, `s` big, lying on its jaw with its mouth a little open: a domed crown
/// with a crest along it, glaring sockets under slanted brows (looking out to the sides as
/// well as ahead), a long snout, teeth, two great fangs, and horns sweeping back.
fn wyvern_skull(s: f32, t: Bones) -> Mesh {
    use std::f32::consts::PI;
    let v = |x: f32, y: f32, z: f32| Vec3::new(x, y, z) * s;
    let mut m = Mesh::new();
    // The lower jaw on the sand: two long bones meeting at the chin, teeth along them.
    let bar =
        |sx: f32, (xb, zb): (f32, f32), (xf, zf): (f32, f32), hw: f32, y1: f32| -> [Vec3; 8] {
            std::array::from_fn(|i| {
                let (x, z) = if i & 4 != 0 { (xf, zf) } else { (xb, zb) };
                let side = if (i & 1 != 0) == (sx > 0.0) { hw } else { -hw };
                v(sx * x + sx * side, if i & 2 != 0 { y1 } else { 0.0 }, z)
            })
        };
    for sx in [-1.0f32, 1.0] {
        block(
            &mut m,
            bar(sx, (0.11, -0.16), (0.06, 0.22), 0.024, 0.05),
            t.worn,
        );
        for k in 0..4 {
            let f = k as f32 / 3.0;
            let (x, z) = (0.105 - f * 0.04, -0.04 + f * 0.22);
            horn(
                &mut m,
                v(sx * x, 0.045, z),
                Vec3::Y,
                0.035 * s,
                0.012 * s,
                t.bone,
            );
        }
    }
    tiled_box(&mut m, v(-0.08, 0.0, 0.19), v(0.08, 0.05, 0.25), t.worn, 0);
    // Everything else, lifted a little about the hinge of the jaw.
    let mut up = Mesh::new();
    let mut dome = Mesh::new();
    lathe(
        &mut dome,
        Vec3::ZERO,
        &[
            (0.13, 0.0),
            (0.175, 0.05),
            (0.18, 0.12),
            (0.145, 0.19),
            (0.075, 0.235),
            (0.0, 0.25),
        ]
        .map(|(r, y)| (r * s, y * s)),
        8,
        PI / 8.0,
        t.bone,
        false,
    );
    up.append(
        &dome,
        Mat4::from_translation(v(0.0, 0.05, -0.11)) * Mat4::from_scale(Vec3::new(1.0, 1.0, 1.15)),
    );
    // The crest along the crown.
    tiled_box(
        &mut up,
        v(-0.018, 0.25, -0.33),
        v(0.018, 0.325, -0.05),
        t.bone,
        0,
    );
    // The snout, narrowing and sloping down to the nose.
    let snout: [Vec3; 8] = std::array::from_fn(|i| {
        let (hx, y0, y1, z) = if i & 4 != 0 {
            (0.07, 0.065, 0.13, 0.36)
        } else {
            (0.1, 0.06, 0.17, 0.0)
        };
        v(
            if i & 1 != 0 { hx } else { -hx },
            if i & 2 != 0 { y1 } else { y0 },
            z,
        )
    });
    block(&mut up, snout, t.bone);
    decal(
        &mut up,
        v(0.0, 0.1, 0.3625),
        Vec2::new(0.042, 0.03) * s,
        NOSE,
        t.features,
    );
    // Little teeth down the upper jaw, and a row across the front.
    for sx in [-1.0f32, 1.0] {
        for z in [0.1f32, 0.18, 0.26] {
            let hx = 0.1 - z / 0.36 * 0.03;
            horn(
                &mut up,
                v(sx * (hx - 0.012), 0.07, z),
                -Vec3::Y,
                0.04 * s,
                0.013 * s,
                t.bone,
            );
        }
    }
    tiled_box(
        &mut up,
        v(-0.035, 0.035, 0.33),
        v(0.035, 0.068, 0.352),
        t.bone,
        0,
    );
    for sx in [-1.0f32, 1.0] {
        // The orbits, their glaring sockets, and the slanted brows over them.
        let (x0, x1) = (sx * 0.07, sx * 0.185);
        tiled_box(
            &mut up,
            v(x0.min(x1), 0.1, -0.05),
            v(x0.max(x1), 0.2, 0.1),
            t.bone,
            0,
        );
        decal(
            &mut up,
            v(sx * 0.1275, 0.15, 0.1025),
            Vec2::new(0.05, 0.043) * s,
            if sx < 0.0 { GLARE } else { GLARE_R },
            t.features,
        );
        let brow: [Vec3; 8] = std::array::from_fn(|i| {
            let outer = (i & 1 != 0) == (sx > 0.0);
            let (x, lift) = if outer { (0.2, 0.03) } else { (0.055, 0.0) };
            v(
                sx * x,
                0.19 + lift + if i & 2 != 0 { 0.04 } else { 0.0 },
                if i & 4 != 0 { 0.113 } else { -0.03 },
            )
        });
        block(&mut up, brow, t.bone);
        decal_turned(
            &mut up,
            v(sx * 0.1875, 0.15, 0.03),
            Vec2::new(0.06, 0.042) * s,
            if sx < 0.0 { GLARE } else { GLARE_R },
            t.features,
            sx * std::f32::consts::FRAC_PI_2,
        );
        // The fangs.
        let f = |x: f32, y: f32, z: f32| v(sx * x, y, z);
        tube(
            &mut up,
            &[
                f(0.05, 0.09, 0.325),
                f(0.053, 0.03, 0.332),
                f(0.056, -0.04, 0.316),
                f(0.052, -0.1, 0.29),
            ],
            &[0.028, 0.024, 0.016, 0.004].map(|r| r * s),
            6,
            t.bone,
        );
        // Horns sweeping back off the crown, and spikes off the cheeks.
        let h = |x: f32, y: f32, z: f32| v(sx * x, y, z);
        tube(
            &mut up,
            &[
                h(0.09, 0.23, -0.19),
                h(0.14, 0.27, -0.29),
                h(0.17, 0.32, -0.38),
                h(0.18, 0.38, -0.44),
                h(0.165, 0.43, -0.47),
            ],
            &[0.05, 0.042, 0.031, 0.018, 0.004].map(|r| r * s),
            6,
            t.horn,
        );
        tube(
            &mut up,
            &[
                h(0.15, 0.1, -0.06),
                h(0.21, 0.1, -0.1),
                h(0.26, 0.11, -0.16),
            ],
            &[0.03, 0.02, 0.004].map(|r| r * s),
            5,
            t.horn,
        );
    }
    let hinge = v(0.0, 0.05, -0.14);
    m.append(
        &up,
        Mat4::from_translation(hinge)
            * Mat4::from_rotation_x(-0.14)
            * Mat4::from_translation(-hinge),
    );
    m
}

/// A fanged monster's skull, `s` big, tipped back on its jaw to glare up at you: a domed
/// crown, glaring sockets under slanted brows, a heavy muzzle with a nose hole and a row of
/// teeth, two great fangs, spiked cheeks, and horns sweeping out and up.
fn fanged_skull(s: f32, t: Bones) -> Mesh {
    use std::f32::consts::PI;
    let v = |x: f32, y: f32, z: f32| Vec3::new(x, y, z) * s;
    // Built standing up, looking along +z, then tipped back.
    let mut k = Mesh::new();
    // A block narrowing from `hx1` wide at `y1` to `hx0` at `y0`.
    let narrowing = |y0: f32, hx0: f32, y1: f32, hx1: f32, z0: f32, z1: f32| -> [Vec3; 8] {
        std::array::from_fn(|i| {
            let (y, hx) = if i & 2 != 0 { (y1, hx1) } else { (y0, hx0) };
            v(
                if i & 1 != 0 { hx } else { -hx },
                y,
                if i & 4 != 0 { z1 } else { z0 },
            )
        })
    };
    let mut dome = Mesh::new();
    lathe(
        &mut dome,
        Vec3::ZERO,
        &[
            (0.17, 0.0),
            (0.2, 0.08),
            (0.19, 0.18),
            (0.14, 0.27),
            (0.07, 0.32),
            (0.0, 0.335),
        ]
        .map(|(r, y)| (r * s, y * s)),
        8,
        PI / 8.0,
        t.bone,
        false,
    );
    k.append(
        &dome,
        Mat4::from_translation(v(0.0, 0.2, -0.1)) * Mat4::from_scale(Vec3::new(1.0, 1.0, 1.1)),
    );
    // The face, and the heavy muzzle out in front of it.
    block(
        &mut k,
        narrowing(0.2, 0.15, 0.44, 0.17, -0.05, 0.12),
        t.bone,
    );
    block(
        &mut k,
        narrowing(0.02, 0.11, 0.25, 0.13, -0.06, 0.2),
        t.worn,
    );
    decal(
        &mut k,
        v(0.0, 0.165, 0.2025),
        Vec2::new(0.045, 0.05) * s,
        NOSE,
        t.features,
    );
    // The lower jaw, dropped open, and teeth in both.
    tiled_box(
        &mut k,
        v(-0.11, -0.07, -0.08),
        v(0.11, -0.01, 0.12),
        t.worn,
        0,
    );
    for x in [-0.075f32, -0.037, 0.0, 0.037, 0.075] {
        horn(
            &mut k,
            v(x, 0.025, 0.18),
            -Vec3::Y,
            0.05 * s,
            0.015 * s,
            t.bone,
        );
    }
    for x in [-0.05f32, 0.05] {
        horn(
            &mut k,
            v(x, -0.015, 0.1),
            Vec3::Y,
            0.04 * s,
            0.013 * s,
            t.bone,
        );
    }
    for sx in [-1.0f32, 1.0] {
        // The orbits, their glaring sockets, and the slanted brows over them.
        let (x0, x1) = (sx * 0.04, sx * 0.18);
        tiled_box(
            &mut k,
            v(x0.min(x1), 0.25, 0.0),
            v(x0.max(x1), 0.37, 0.14),
            t.bone,
            0,
        );
        decal(
            &mut k,
            v(sx * 0.11, 0.31, 0.1425),
            Vec2::new(0.062, 0.052) * s,
            if sx < 0.0 { GLARE } else { GLARE_R },
            t.features,
        );
        let brow: [Vec3; 8] = std::array::from_fn(|i| {
            let outer = (i & 1 != 0) == (sx > 0.0);
            let (x, lift) = if outer { (0.2, 0.04) } else { (0.03, 0.0) };
            v(
                sx * x,
                0.35 + lift + if i & 2 != 0 { 0.045 } else { 0.0 },
                if i & 4 != 0 { 0.155 } else { -0.02 },
            )
        });
        block(&mut k, brow, t.bone);
        // The cheekbones, spiked.
        let (c0, c1) = (sx * 0.13, sx * 0.22);
        tiled_box(
            &mut k,
            v(c0.min(c1), 0.19, -0.06),
            v(c0.max(c1), 0.27, 0.08),
            t.worn,
            0,
        );
        let h = |x: f32, y: f32, z: f32| v(sx * x, y, z);
        tube(
            &mut k,
            &[h(0.2, 0.235, 0.0), h(0.28, 0.22, -0.04), h(0.33, 0.2, -0.1)],
            &[0.03, 0.02, 0.004].map(|r| r * s),
            5,
            t.horn,
        );
        // The fangs.
        tube(
            &mut k,
            &[
                h(0.095, 0.05, 0.16),
                h(0.1, -0.02, 0.17),
                h(0.095, -0.1, 0.16),
                h(0.08, -0.17, 0.13),
            ],
            &[0.034, 0.028, 0.017, 0.004].map(|r| r * s),
            6,
            t.bone,
        );
        // The horns.
        tube(
            &mut k,
            &[
                h(0.11, 0.46, -0.1),
                h(0.2, 0.53, -0.12),
                h(0.26, 0.62, -0.12),
                h(0.28, 0.72, -0.08),
                h(0.26, 0.8, -0.02),
            ],
            &[0.055, 0.047, 0.036, 0.022, 0.004].map(|r| r * s),
            7,
            t.horn,
        );
    }
    let mut m = Mesh::new();
    m.append(
        &k,
        Mat4::from_translation(v(0.0, 0.06, 0.15)) * Mat4::from_rotation_x(-0.6),
    );
    m
}

/// A little skull out of the warrens' heaps, `s` big: a round crown, its face with sockets,
/// a nose and teeth, and a jaw.
fn little_skull(s: f32, t: Bones) -> Mesh {
    use std::f32::consts::PI;
    let v = |x: f32, y: f32, z: f32| Vec3::new(x, y, z) * s;
    let mut m = Mesh::new();
    let mut dome = Mesh::new();
    lathe(
        &mut dome,
        Vec3::ZERO,
        &[
            (0.1, 0.0),
            (0.155, 0.06),
            (0.165, 0.14),
            (0.14, 0.22),
            (0.08, 0.275),
            (0.0, 0.295),
        ]
        .map(|(r, y)| (r * s, y * s)),
        8,
        PI / 8.0,
        t.bone,
        false,
    );
    m.append(
        &dome,
        Mat4::from_translation(v(0.0, 0.02, -0.02)) * Mat4::from_scale(Vec3::new(1.0, 1.0, 1.12)),
    );
    tiled_box(&mut m, v(-0.115, 0.0, 0.0), v(0.115, 0.17, 0.16), t.bone, 0);
    decal(
        &mut m,
        v(0.0, 0.085, 0.163),
        Vec2::new(0.115, 0.085) * s,
        FACE,
        t.features,
    );
    tiled_box(
        &mut m,
        v(-0.085, -0.02, 0.02),
        v(0.085, 0.03, 0.145),
        t.worn,
        0,
    );
    m
}

pub fn build(bank_: &mut TexBank, marble: (TexId, TexId, TexId), columns: &[Mesh; 3]) -> CanyonArt {
    let side = [
        bank_.add(strata(
            &[
                (3, CLAY),
                (1, MAROON),
                (4, RUST),
                (2, ORANGE),
                (3, CLAY),
                (1, GOLD),
                (4, RUST),
                (2, MAROON),
            ],
            0xCA70,
        )),
        bank_.add(strata(
            &[
                (3, GOLD),
                (2, SAND),
                (1, CLAY),
                (4, PEACH),
                (2, GOLD),
                (1, RUST),
                (3, SAND),
                (2, CLAY),
            ],
            0xCA71,
        )),
        bank_.add(tomb_side()),
    ];
    let top = [
        bank_.add(rock_top([ORANGE, CLAY, RUST], 0xCA72)),
        bank_.add(rock_top([SAND, GOLD, CLAY], 0xCA73)),
        bank_.add(tomb_top()),
    ];
    let sand_t = std::array::from_fn(|k| bank_.add(sand(k, 0xCA80 + k as u64)));
    let flags = [bank_.add(tomb_flags(0xCA90)), bank_.add(tomb_flags(0xCA91))];
    let quicksand_t = std::array::from_fn(|f| bank_.add(quicksand(f)));
    let bank_t = bank_.add(bank());

    let w4 = Texture::new(4, 4, 0);
    let rockside = side[0];
    let gold = bank_.add(Texture::new(4, 4, GOLD));
    let bone_t = bank_.add(bone());
    let bones = Bones {
        bone: bone_t,
        worn: bank_.add(worn_bone()),
        horn: bank_.add(horn_ridges()),
        features: bank_.add(features()),
    };

    // The sandfall pillar: a column of layered rock narrowing as it goes up, and the sand
    // piled round its foot.
    let mut sandfall = Mesh::new();
    tiled_box(
        &mut sandfall,
        Vec3::new(-0.36, 0.0, -0.36),
        Vec3::new(0.36, 1.4, 0.36),
        rockside,
        0,
    );
    tiled_box(
        &mut sandfall,
        Vec3::new(-0.3, 1.4, -0.3),
        Vec3::new(0.3, 2.5, 0.3),
        rockside,
        0,
    );
    let heap = bank_.add(sand(3, 0xCA84));
    lathe(
        &mut sandfall,
        Vec3::new(0.0, 0.0, 0.18),
        &[(0.5, 0.0), (0.38, 0.12), (0.2, 0.26), (0.0, 0.34)],
        10,
        0.0,
        heap,
        false,
    );

    // Skulls: a great horned beast's sunk in the sand, a heap of little ones among bones,
    // and a fanged monster's.
    let horned = horned_skull(1.3, bones);
    let mut pile = Mesh::new();
    for (at, turn, tip) in [
        (Vec3::new(-0.14, 0.0, 0.07), 0.35f32, 0.0f32),
        (Vec3::new(0.15, 0.0, 0.03), -0.45, 0.0),
        (Vec3::new(0.0, 0.13, -0.08), 0.1, -0.25),
    ] {
        pile.append(
            &little_skull(0.62, bones),
            Mat4::from_translation(at) * Mat4::from_rotation_y(turn) * Mat4::from_rotation_x(tip),
        );
    }
    stick(
        &mut pile,
        Vec3::new(-0.34, 0.02, 0.26),
        Vec3::new(0.24, 0.02, 0.32),
        0.02,
        bones.worn,
    );
    stick(
        &mut pile,
        Vec3::new(0.22, 0.02, -0.3),
        Vec3::new(0.36, 0.02, 0.12),
        0.018,
        bones.worn,
    );
    let fanged = fanged_skull(1.15, bones);

    // Cacti: a tall one with arms, a squat ribbed barrel with a flower, a clump of paddles
    // with fruit on them.
    let rib = bank_.add(ribs());
    let pink = bank_.add(Texture::new(4, 4, PINK));
    let blush = bank_.add(Texture::new(4, 4, BLUSH));
    let mut saguaro = Mesh::new();
    lathe(
        &mut saguaro,
        Vec3::ZERO,
        &[(0.14, 0.0), (0.15, 0.6), (0.13, 1.05), (0.07, 1.15)],
        8,
        0.0,
        rib,
        false,
    );
    for (sx, y, reach, up) in [(1.0f32, 0.45f32, 0.26f32, 0.36f32), (-1.0, 0.62, 0.22, 0.3)] {
        let mut arm = Mesh::new();
        lathe(
            &mut arm,
            Vec3::ZERO,
            &[(0.075, 0.0), (0.075, reach)],
            6,
            0.0,
            rib,
            false,
        );
        saguaro.append(
            &arm,
            Mat4::from_translation(Vec3::new(0.0, y, 0.0))
                * Mat4::from_rotation_z(-sx * std::f32::consts::FRAC_PI_2),
        );
        lathe(
            &mut saguaro,
            Vec3::new(sx * reach, y, 0.0),
            &[(0.075, 0.0), (0.07, up), (0.035, up + 0.06)],
            6,
            0.0,
            rib,
            false,
        );
    }
    lathe(
        &mut saguaro,
        Vec3::new(0.0, 1.12, 0.0),
        &[(0.02, 0.0), (0.06, 0.04), (0.0, 0.07)],
        5,
        0.0,
        pink,
        false,
    );
    let mut barrel = Mesh::new();
    lathe(
        &mut barrel,
        Vec3::ZERO,
        &[
            (0.2, 0.0),
            (0.27, 0.12),
            (0.26, 0.3),
            (0.16, 0.42),
            (0.0, 0.46),
        ],
        10,
        0.0,
        rib,
        false,
    );
    for k in 0..3 {
        let a = k as f32 * 2.1;
        lathe(
            &mut barrel,
            Vec3::new(a.cos() * 0.06, 0.44, a.sin() * 0.06),
            &[(0.02, 0.0), (0.05, 0.04), (0.0, 0.06)],
            5,
            0.0,
            gold,
            false,
        );
    }
    let mut paddle = Mesh::new();
    for (x, y, z, s, tilt) in [
        (0.0f32, 0.0f32, 0.0f32, 1.0f32, 0.0f32),
        (-0.14, 0.28, 0.02, 0.8, 0.5),
        (0.13, 0.3, -0.02, 0.75, -0.4),
        (0.02, 0.5, 0.0, 0.6, 0.1),
    ] {
        let mut p = Mesh::new();
        lathe(
            &mut p,
            Vec3::ZERO,
            &[
                (0.0, 0.0),
                (0.13, 0.08),
                (0.14, 0.2),
                (0.08, 0.3),
                (0.0, 0.34),
            ],
            8,
            0.0,
            rib,
            false,
        );
        paddle.append(
            &p,
            Mat4::from_translation(Vec3::new(x, y, z))
                * Mat4::from_rotation_z(tilt)
                * Mat4::from_scale(Vec3::new(s, s, s * 0.35)),
        );
    }
    for (x, y) in [(-0.2f32, 0.55f32), (0.22, 0.56), (0.05, 0.84)] {
        skin_box(
            &mut paddle,
            Vec3::new(x - 0.03, y, -0.03),
            Vec3::new(x + 0.03, y + 0.07, 0.03),
            blush,
            &w4,
        );
    }

    // The sarcophagus: a coffin of carved blocks under a slab, and on the slab its lid
    // carved as the one inside: wrapped, arms crossed on a jewelled collar holding a crook and
    // a flail, in a striped headdress with a golden face.
    let slab = bank_.add(tomb_top());
    let clay = bank_.add(Texture::new(4, 4, CLAY));
    let indigo = bank_.add(Texture::new(4, 4, INDIGO));
    let mut sarcophagus = Mesh::new();
    tiled_box(
        &mut sarcophagus,
        Vec3::new(-0.44, 0.0, -0.24),
        Vec3::new(0.44, 0.28, 0.24),
        side[2],
        0,
    );
    tiled_box(
        &mut sarcophagus,
        Vec3::new(-0.46, 0.28, -0.26),
        Vec3::new(0.46, 0.33, 0.26),
        slab,
        0,
    );
    let wraps = mummy_wraps();
    let wraps_t = bank_.add(wraps.clone());
    skin_box(
        &mut sarcophagus,
        Vec3::new(-0.38, 0.33, -0.16),
        Vec3::new(0.2, 0.41, 0.16),
        wraps_t,
        &wraps,
    );
    // The feet, standing up at the end.
    skin_box(
        &mut sarcophagus,
        Vec3::new(-0.43, 0.33, -0.1),
        Vec3::new(-0.36, 0.45, 0.1),
        wraps_t,
        &wraps,
    );
    let collar = jewelled_collar();
    let collar_t = bank_.add(collar.clone());
    skin_box(
        &mut sarcophagus,
        Vec3::new(0.08, 0.33, -0.19),
        Vec3::new(0.21, 0.425, 0.19),
        collar_t,
        &collar,
    );
    // The crook and the flail, crossed.
    for (turn, tex) in [(0.7f32, gold), (-0.7, indigo)] {
        let mut rod = Mesh::new();
        skin_box(
            &mut rod,
            Vec3::new(-0.12, 0.0, -0.018),
            Vec3::new(0.12, 0.03, 0.018),
            tex,
            &w4,
        );
        sarcophagus.append(
            &rod,
            Mat4::from_translation(Vec3::new(0.0, 0.41, 0.0)) * Mat4::from_rotation_y(turn),
        );
    }
    let nemes = nemes_stripes();
    let nemes_t = bank_.add(nemes.clone());
    skin_box(
        &mut sarcophagus,
        Vec3::new(0.2, 0.33, -0.15),
        Vec3::new(0.42, 0.47, 0.15),
        nemes_t,
        &nemes,
    );
    let mask = golden_mask();
    let mask_t = bank_.add(mask.clone());
    skin_box(
        &mut sarcophagus,
        Vec3::new(0.24, 0.47, -0.08),
        Vec3::new(0.38, 0.495, 0.08),
        mask_t,
        &mask,
    );
    // The beard at its chin, and the cobra rearing on its brow.
    skin_box(
        &mut sarcophagus,
        Vec3::new(0.17, 0.4, -0.025),
        Vec3::new(0.25, 0.46, 0.025),
        indigo,
        &w4,
    );
    skin_box(
        &mut sarcophagus,
        Vec3::new(0.395, 0.46, -0.018),
        Vec3::new(0.43, 0.52, 0.018),
        gold,
        &w4,
    );
    skin_box(
        &mut sarcophagus,
        Vec3::new(0.4, 0.5, -0.01),
        Vec3::new(0.435, 0.525, 0.01),
        clay,
        &w4,
    );

    // The wyvern: its long spine curving over the six tiles, ribs arching up off it, its
    // horned skull at one end and its tail at the other, and the long bones of its wings
    // spread out either side.
    let mut wyvern = Mesh::new();
    let spine: Vec<Vec3> = (0..13)
        .map(|k| {
            let t = k as f32 / 12.0;
            Vec3::new(
                0.95 - t * 2.2,
                0.07 + (t * 3.0).sin() * 0.05,
                (t * 5.0).sin() * 0.17,
            )
        })
        .collect();
    let thick: Vec<f32> = (0..13).map(|k| 0.052 - k as f32 * 0.0022).collect();
    tube(&mut wyvern, &spine, &thick, 6, bone_t);
    // Knobs of vertebrae along the spine.
    for (k, &p) in spine.iter().enumerate().skip(1).step_by(2) {
        let r = 0.05 - k as f32 * 0.002;
        skin_box(
            &mut wyvern,
            p + Vec3::new(-r, 0.02, -r * 1.3),
            p + Vec3::new(r, 0.06 + r * 1.4, r * 1.3),
            bone_t,
            &w4,
        );
    }
    // Ribs arching up off the spine and down to the sand, biggest in the middle.
    for (k, &p) in spine.iter().enumerate().take(9).skip(3) {
        let size = 1.0 - ((k as f32 - 5.5) / 3.0).powi(2) * 0.45;
        for side in [-1.0f32, 1.0] {
            let at =
                |dx: f32, y: f32, out: f32| Vec3::new(p.x + dx * size, y, p.z + side * out * size);
            tube(
                &mut wyvern,
                &[
                    p,
                    at(0.0, p.y + 0.2 * size, 0.1),
                    at(0.02, p.y + 0.27 * size, 0.26),
                    at(0.04, p.y + 0.19 * size, 0.4),
                    at(0.06, 0.03, 0.46),
                ],
                &[0.03, 0.028, 0.025, 0.021, 0.017],
                5,
                bone_t,
            );
        }
    }
    let head = spine[0];
    wyvern.append(
        &wyvern_skull(1.2, bones),
        Mat4::from_translation(head + Vec3::new(0.12, -0.02, 0.0))
            * Mat4::from_rotation_y(std::f32::consts::FRAC_PI_2)
            * Mat4::from_rotation_x(0.06),
    );
    for side in [-1.0f32, 1.0] {
        // The wings: the arm bone up to the elbow, and the long fingers spreading from it,
        // bowed, down to the sand.
        let shoulder = spine[3] + Vec3::new(0.0, 0.12, side * 0.14);
        let elbow = shoulder + Vec3::new(-0.15, 0.2, side * 0.45);
        tube(
            &mut wyvern,
            &[shoulder, (shoulder + elbow) * 0.5 + Vec3::Y * 0.04, elbow],
            &[0.045, 0.038, 0.036],
            5,
            bone_t,
        );
        for (dx, reach) in [(0.45f32, 0.5f32), (-0.1, 0.72), (-0.65, 0.58)] {
            let tip = elbow + Vec3::new(dx, -elbow.y + 0.03, side * reach);
            let bow = (elbow + tip) * 0.5 + Vec3::new(0.0, 0.07, side * 0.05);
            tube(
                &mut wyvern,
                &[elbow, bow, tip],
                &[0.028, 0.022, 0.012],
                5,
                bone_t,
            );
        }
        // A leg, bent, and its claws.
        let hip = spine[8] + Vec3::new(0.0, 0.0, side * 0.1);
        let knee = hip + Vec3::new(0.14, 0.04, side * 0.32);
        let foot = knee + Vec3::new(0.24, -0.08, side * 0.06);
        tube(
            &mut wyvern,
            &[hip, knee, foot],
            &[0.04, 0.034, 0.03],
            5,
            bone_t,
        );
        for turn in [-0.5f32, 0.0, 0.5] {
            let d = Vec3::new(turn.cos(), -0.1, side * turn.sin());
            horn(&mut wyvern, foot, d, 0.1, 0.018, bone_t);
        }
    }
    // The tail, curling off to a spiked end.
    let end = spine[12];
    let tail = [
        end,
        end + Vec3::new(-0.16, -0.01, 0.12),
        end + Vec3::new(-0.24, -0.02, 0.3),
        end + Vec3::new(-0.2, -0.03, 0.46),
    ];
    tube(&mut wyvern, &tail, &[0.026, 0.022, 0.018, 0.014], 5, bone_t);
    horn(
        &mut wyvern,
        tail[3],
        Vec3::new(0.2, 0.0, 1.0),
        0.16,
        0.06,
        bone_t,
    );

    // A heap of old gold: coins piled up, stacks of them standing round it, a few rolled
    // loose, a goblet tipped on its side and a jewel or two.
    let coin_t = bank_.add(coins());
    let edges_t = bank_.add(coin_edges());
    let ruby = bank_.add(Texture::new(4, 4, RED));
    let turquoise = bank_.add(Texture::new(4, 4, AQUA));
    let mut gold_pile = Mesh::new();
    lathe(
        &mut gold_pile,
        Vec3::ZERO,
        &[
            (0.33, 0.0),
            (0.29, 0.05),
            (0.21, 0.12),
            (0.12, 0.18),
            (0.04, 0.22),
            (0.0, 0.23),
        ],
        10,
        0.3,
        coin_t,
        false,
    );
    // Stacks of coins, one coin to a texel down their sides.
    let face = Some(UvRect::px(0, 0, 4, 4));
    for (x, z, h) in [
        (0.3f32, 0.2f32, 0.19f32),
        (0.37, 0.0, 0.12),
        (-0.33, 0.18, 0.15),
    ] {
        gold_pile.lathe(
            Vec3::new(x, 0.0, z),
            &[(0.055, 0.0, 8.0 + h * 32.0), (0.055, h, 8.0)],
            7,
            16.0,
            0.0,
            edges_t,
            (None, face),
        );
    }
    for (x, z) in [
        (0.06f32, 0.4f32),
        (-0.24, 0.34),
        (0.12, -0.38),
        (-0.42, -0.1),
        (0.42, -0.24),
    ] {
        gold_pile.lathe(
            Vec3::new(x, 0.0, z),
            &[(0.05, 0.0, 9.0), (0.05, 0.016, 8.0)],
            7,
            16.0,
            0.0,
            edges_t,
            (None, face),
        );
    }
    let mut goblet = Mesh::new();
    lathe(
        &mut goblet,
        Vec3::ZERO,
        &[
            (0.05, 0.0),
            (0.015, 0.03),
            (0.015, 0.08),
            (0.06, 0.12),
            (0.055, 0.16),
        ],
        6,
        0.0,
        gold,
        false,
    );
    gold_pile.append(
        &goblet,
        Mat4::from_translation(Vec3::new(-0.16, 0.1, 0.12)) * Mat4::from_rotation_z(1.2),
    );
    for (at, tex) in [
        (Vec3::new(0.08, 0.16, 0.08), ruby),
        (Vec3::new(-0.12, 0.13, -0.08), turquoise),
        (Vec3::new(0.2, 0.06, -0.16), ruby),
    ] {
        skin_box(
            &mut gold_pile,
            at - Vec3::splat(0.03),
            at + Vec3::splat(0.03),
            tex,
            &w4,
        );
    }

    // The gateway: two square posts of carved blocks and a lintel across them.
    let tomb_t = side[2];
    let mut gateway = Mesh::new();
    for x in [-1.0f32, 1.0] {
        tiled_box(
            &mut gateway,
            Vec3::new(x - 0.26, 0.0, -0.26),
            Vec3::new(x + 0.26, 2.0, 0.26),
            tomb_t,
            0,
        );
    }
    tiled_box(
        &mut gateway,
        Vec3::new(-1.4, 2.0, -0.3),
        Vec3::new(1.4, 2.45, 0.3),
        tomb_t,
        0,
    );
    skin_box(
        &mut gateway,
        Vec3::new(-0.2, 2.1, 0.3),
        Vec3::new(0.2, 2.35, 0.32),
        gold,
        &w4,
    );

    // The labyrinth's columns, recarved in sandstone.
    let (flute, stone, trim) = marble;
    let sand_flute = bank_.add(strata(&[(2, SAND), (1, PEACH), (1, KHAKI)], 0xCAA0));
    let sand_slab = bank_.add(tomb_top());
    let columns = columns.clone().map(|m| {
        m.retexture(flute, sand_flute)
            .retexture(stone, sand_slab)
            .retexture(trim, clay)
    });

    CanyonArt {
        side,
        top,
        sand: sand_t,
        flags,
        quicksand: quicksand_t,
        bank: bank_t,
        sandfall,
        skulls: [horned, pile, fanged],
        cacti: [saguaro, barrel, paddle],
        sarcophagus,
        wyvern,
        gold: gold_pile,
        gateway,
        columns,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn everything_stays_in_the_palette() {
        let mut all = vec![
            strata(&[(3, CLAY), (1, MAROON), (4, RUST)], 1),
            rock_top([ORANGE, CLAY, RUST], 2),
            tomb_side(),
            tomb_top(),
            tomb_flags(3),
            bank(),
            ribs(),
            bone(),
            worn_bone(),
            horn_ridges(),
            features(),
            coins(),
            coin_edges(),
            mummy_wraps(),
            jewelled_collar(),
            nemes_stripes(),
            golden_mask(),
        ];
        all.extend((0..4).map(|k| sand(k, k as u64)));
        all.extend((0..4).map(quicksand));
        for t in all {
            for y in 0..t.h as i32 {
                for x in 0..t.w as i32 {
                    assert!(t.get(x, y) < 32, "({x}, {y})");
                }
            }
        }
    }

    #[test]
    fn quicksand_swirls_round() {
        assert_ne!(quicksand(0).get(3, 5), quicksand(2).get(3, 5));
    }
}
