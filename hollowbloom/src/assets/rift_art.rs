//! The Starless Rift: black obsidian walls carved with runes and veined with amethyst, dark
//! rock and pale stone underfoot, old stone bridges with red railings, the islands' ragged
//! undersides, and the stars of the void below; and in it amethyst clusters, voidfalls, great
//! chains, eyes in the walls, glowing runes, and the portal you come in by.

use glam::{Mat4, Quat, Vec3};

use super::models::{lathe, skin_box, tiled_box};
use crate::palette::*;
use crate::render::{Mesh, TexBank, TexId, Texture, UvRect};
use crate::util::{Rng, hash2};

const T: i32 = 16;

pub struct RiftArt {
    /// Obsidian faces and tops by look (see `rift::BLOCKS` and so on).
    pub side: [TexId; 5],
    pub top: [TexId; 5],
    /// Dark rock underfoot: plain, with a shard of amethyst, with pebbles.
    pub ground: [TexId; 3],
    /// Pale stone showing through.
    pub pale: [TexId; 2],
    /// The citadel's flagstones, one carved with a rune.
    pub flags: [TexId; 2],
    /// A bridge's slabs, its edge, and its red railing.
    pub deck: TexId,
    pub beam: TexId,
    pub rail: TexId,
    /// The islands' rock faces, ragged along their bottoms (`rift::CLIFF` tall).
    pub cliff: [TexId; 3],
    /// The void: stars and nebulae, a tile to wrap across the screen.
    pub stars: Texture,
    /// Amethyst by `rift::SMALL` and so on.
    pub amethyst: [Mesh; 3],
    /// A voidfall's streaming sheet, by frame (see `FALL_FRAMES`), and its mesh: one tile
    /// wide, hanging from the island's edge down into the void along -y, facing +z.
    pub fall: [TexId; FALL_FRAMES],
    pub fall_sheet: Mesh,
    /// A link of chain, along +x, and a chain's post.
    pub link: Mesh,
    pub post: Mesh,
    /// An eye in the wall: its white, the iris and pupil in front of it, and a lid.
    pub eye_white: Mesh,
    pub iris: Mesh,
    pub lid: Mesh,
    /// Runes on a wall, by which, as a flat panel facing +z.
    pub runes: [Mesh; 4],
    /// The portal, and the swirl within it by frame.
    pub gate: Mesh,
    pub swirl: [TexId; 4],
    /// The labyrinth's columns, cut from obsidian for the citadel.
    pub columns: [Mesh; 3],
}

/// How many frames a voidfall's stream takes to run one texel down.
pub const FALL_FRAMES: usize = 4;

/// Dark blocks in courses, their edges catching a little light, with a violet sheen here
/// and there; `look` carves runes in them, cracks them, veins them with amethyst, or makes
/// a tower of them with an arrow slit.
fn obsidian(look: u8, seed: u64) -> Texture {
    let mut t = Texture::new(16, 32, SHADOW);
    let mut r = Rng::new(seed);
    for y in 0..32 {
        for x in 0..T {
            let course = y / 8;
            let off = if course % 2 == 0 { 0 } else { 5 };
            let c = if y % 8 == 7 || (x + off) % 10 == 9 {
                INK
            } else if y % 8 == 0 {
                KHAKI
            } else if (x + off) % 10 == 0 {
                ROSEWOOD
            } else if hash2(x, y, seed as u32) % 13 == 0 {
                GRAPE
            } else {
                SHADOW
            };
            t.set(x, y, c);
        }
    }
    match look {
        1 => {
            // Runes cut into the blocks, glowing faintly.
            let glyphs: [&[(i32, i32)]; 3] = [
                &[(0, 0), (0, 1), (0, 2), (1, 1), (2, 0), (2, 2)],
                &[(1, 0), (0, 1), (2, 1), (1, 2), (1, 1)],
                &[(0, 0), (1, 0), (2, 0), (1, 1), (1, 2), (0, 3), (2, 3)],
            ];
            for (k, (bx, by)) in [(2, 2), (9, 3), (4, 11), (11, 18), (3, 26)]
                .iter()
                .enumerate()
            {
                for &(dx, dy) in glyphs[k % 3] {
                    t.set_wrap(bx + dx, by + dy, if k % 2 == 0 { LAVENDER } else { PURPLE });
                }
            }
        }
        2 => {
            // A crack down through the courses, violet light in it.
            let mut x = r.range(4, 12);
            for y in 1..31 {
                t.set_wrap(x, y, INK);
                if y % 5 == 2 {
                    t.set_wrap(x + 1, y, PURPLE);
                }
                x += r.range(-1, 2);
            }
        }
        3 => {
            // Amethyst breaking out of the stone.
            for (cx, cy) in [(4, 6), (11, 15), (6, 24)] {
                for (dx, dy, c) in [
                    (0, 0, LAVENDER),
                    (1, 0, PURPLE),
                    (0, 1, PURPLE),
                    (1, 1, GRAPE),
                    (-1, 1, LAVENDER),
                    (0, -1, BLUSH),
                    (2, 1, PURPLE),
                ] {
                    t.set_wrap(cx + dx, cy + dy, c);
                }
            }
        }
        4 => {
            // An arrow slit, and a band of carved stone.
            for y in 10..20 {
                t.set(7, y, INK);
                t.set(8, y, INK);
            }
            for y in [9, 20] {
                for x in 6..10 {
                    t.set(x, y, SLATE);
                }
            }
            for x in 0..T {
                t.set(x, 2, GRAPE);
                t.set(x, 3, INK);
            }
        }
        _ => {}
    }
    t
}

/// The top of the obsidian: dark stone, crumbling, and a tower's crenels round its edge.
fn obsidian_top(look: u8) -> Texture {
    let mut t = Texture::new(16, 16, SHADOW);
    for i in 0..T {
        t.set(i, 0, INK);
        t.set(0, i, INK);
        t.set(i, 15, SLATE);
    }
    for (x, y) in [(4, 5), (10, 9), (7, 12), (12, 3)] {
        t.set(x, y, INK);
    }
    if look == 4 {
        for y in 0..T {
            for x in 0..T {
                let rim = !(3..13).contains(&x) || !(3..13).contains(&y);
                let gap = (x / 3 + y / 3) % 2 == 0;
                if rim && gap {
                    t.set(x, y, SLATE);
                }
            }
        }
    }
    t
}

/// The islands' dark rock: flat plates of it with dark cracks between, their edges catching
/// the light; a shard of amethyst in it (`kind` 1), or pebbles (2).
fn ground(kind: usize, seed: u64) -> Texture {
    let mut t = Texture::new(16, 16, SHADOW);
    let mut r = Rng::new(seed);
    // Plates, their bounds wandering a little, and cracks round them.
    let plate = |x: i32, y: i32| {
        let jx = (hash2(y / 4, 7, seed as u32) % 3) as i32;
        let jy = (hash2(x / 4, 9, seed as u32) % 3) as i32;
        ((x + jx).rem_euclid(16) / 6, (y + jy).rem_euclid(16) / 6)
    };
    for y in 0..T {
        for x in 0..T {
            let here = plate(x, y);
            if plate(x, y - 1) != here || plate(x - 1, y) != here {
                t.set(x, y, INK);
            } else if plate(x, y - 2) != here {
                t.set(x, y, KHAKI);
            } else if hash2(x, y, seed as u32) % 19 == 0 {
                t.set(x, y, ROSEWOOD);
            }
        }
    }
    match kind {
        1 => {
            for (dx, dy, c) in [
                (0, 0, LAVENDER),
                (1, 0, PURPLE),
                (0, 1, GRAPE),
                (1, -1, BLUSH),
            ] {
                t.set_wrap(9 + dx, 8 + dy, c);
            }
        }
        2 => {
            for _ in 0..4 {
                let (x, y) = (r.range(0, T), r.range(0, T));
                t.set(x, y, KHAKI);
                t.set_wrap(x + 1, y + 1, INK);
            }
        }
        _ => {}
    }
    t
}

/// Pale stone: sandy, pitted, crumbling at the edge of the dark.
fn pale(seed: u64) -> Texture {
    let mut t = Texture::new(16, 16, ROSEWOOD);
    for y in 0..T {
        for x in 0..T {
            match hash2(x, y, seed as u32) % 12 {
                0 | 1 => t.set(x, y, KHAKI),
                2 => t.set(x, y, SHADOW),
                3 if (x + y) % 2 == 0 => t.set(x, y, SAND),
                _ => {}
            }
        }
    }
    t
}

/// The citadel's flagstones: big dark slabs, and one with a rune carved in it.
fn flagstones(rune: bool) -> Texture {
    let mut t = Texture::new(16, 16, INK);
    for &(sx, sy, sw, sh, c) in &[
        (0, 0, 8, 8, SHADOW),
        (8, 0, 8, 8, ROSEWOOD),
        (0, 8, 10, 8, ROSEWOOD),
        (10, 8, 6, 8, SHADOW),
    ] {
        for y in 0..sh - 1 {
            for x in 0..sw - 1 {
                let edge = y == 0 || x == 0;
                t.set_wrap(sx + x, sy + y, if edge { KHAKI } else { c });
            }
        }
    }
    if rune {
        for (x, y) in [
            (3, 2),
            (3, 3),
            (3, 4),
            (4, 3),
            (5, 2),
            (5, 4),
            (12, 11),
            (11, 12),
            (13, 12),
            (12, 13),
        ] {
            t.set(x, y, PURPLE);
        }
    }
    t
}

/// A bridge's stone slabs, laid across it.
fn deck() -> Texture {
    let mut t = Texture::new(16, 16, SHADOW);
    for y in 0..T {
        for x in 0..T {
            let (sx, sy) = (x % 8, y % 8);
            let c = if sx == 7 || sy == 7 {
                INK
            } else if sy == 0 || sx == 0 {
                KHAKI
            } else if hash2(x, y, 0xDEC) % 11 == 0 {
                ROSEWOOD
            } else if (x / 8 + y / 8) % 2 == 0 {
                SHADOW
            } else {
                ROSEWOOD
            };
            t.set(x, y, c);
        }
    }
    t
}

/// A slab's edge.
fn beam() -> Texture {
    let mut t = Texture::new(16, 16, INK);
    for x in 0..T {
        t.set(x, 0, KHAKI);
        t.set(x, 1, SHADOW);
        if x % 5 == 2 {
            t.set(x, 3, SLATE);
        }
    }
    t
}

/// The railing's old red lacquer, worn dark.
fn rail() -> Texture {
    let mut t = Texture::new(4, 16, MAROON);
    for y in 0..16 {
        t.set(0, y, CRIMSON);
        t.set(3, y, INK);
    }
    t
}

/// An island's rock face: layers of dark stone with amethyst in them, catching the light
/// along the top, and breaking off ragged below (clear, so the stars show through).
fn cliff(seed: u64) -> Texture {
    let mut t = Texture::new(16, 32, SHADOW);
    let mut r = Rng::new(seed);
    for y in 0..32 {
        for x in 0..T {
            let wobble = (hash2(x / 4, y / 6, seed as u32) % 3) as i32;
            let c = match (y + wobble) % 7 {
                0 => INK,
                3 => SLATE,
                _ if hash2(x, y, seed as u32 ^ 5) % 17 == 0 => GRAPE,
                _ => SHADOW,
            };
            t.set(x, y, c);
        }
    }
    for x in 0..T {
        t.set(x, 0, KHAKI);
        t.set(x, 1, ROSEWOOD);
    }
    // A vein of amethyst.
    let (cx, cy) = (r.range(3, 13), r.range(8, 16));
    for (dx, dy, c) in [
        (0, 0, LAVENDER),
        (1, 0, PURPLE),
        (0, 1, PURPLE),
        (-1, 1, GRAPE),
    ] {
        t.set_wrap(cx + dx, cy + dy, c);
    }
    // The ragged underside: the rock tapers off into points.
    let mut bottom = r.range(20, 26);
    for x in 0..T {
        bottom = (bottom + r.range(-3, 4)).clamp(14, 26);
        let tip = if x % 5 == (seed % 5) as i32 { 4 } else { 0 };
        let cut = (bottom + tip).min(30);
        for y in cut..32 {
            t.set(x, y, CLEAR);
        }
        t.set(x, cut - 1, INK);
    }
    t
}

/// The void below: black, with drifts of violet and blue nebula and stars of every size,
/// wrapping every 256 texels either way.
fn starfield() -> Texture {
    const N: i32 = 256;
    let mut t = Texture::new(N as u32, N as u32, INK);
    // Smooth noise that wraps.
    let lattice = |x: i32, y: i32, cell: i32, seed: u32| {
        let n = N / cell;
        let v =
            |a: i32, b: i32| (hash2(a.rem_euclid(n), b.rem_euclid(n), seed) % 1000) as f32 / 999.0;
        let (gx, gy) = (x as f32 / cell as f32, y as f32 / cell as f32);
        let (ix, iy) = (gx.floor() as i32, gy.floor() as i32);
        let (fx, fy) = (gx - ix as f32, gy - iy as f32);
        let s = |t: f32| t * t * (3.0 - 2.0 * t);
        let (sx, sy) = (s(fx), s(fy));
        let top = v(ix, iy) * (1.0 - sx) + v(ix + 1, iy) * sx;
        let bot = v(ix, iy + 1) * (1.0 - sx) + v(ix + 1, iy + 1) * sx;
        top * (1.0 - sy) + bot * sy
    };
    for y in 0..N {
        for x in 0..N {
            let cloud = lattice(x, y, 64, 11) * 0.65 + lattice(x, y, 16, 12) * 0.35;
            let hue = lattice(x, y, 128, 13);
            let dither = crate::palette::bayer(x as usize, y as usize);
            let c = if cloud > 0.72 + dither * 0.08 {
                if hue > 0.5 { GRAPE } else { INDIGO }
            } else if cloud > 0.6 + dither * 0.1 {
                SLATE
            } else {
                INK
            };
            t.set(x, y, c);
        }
    }
    let mut r = Rng::new(0x57A8);
    for _ in 0..420 {
        let (x, y) = (r.range(0, N), r.range(0, N));
        let c = match r.below(10) {
            0 => WHITE,
            1 | 2 => LAVENDER,
            3 => BLUSH,
            _ => SKY,
        };
        t.set(x, y, c);
    }
    // A few bright ones with a glint round them.
    for _ in 0..16 {
        let (x, y) = (r.range(1, N - 1), r.range(1, N - 1));
        t.set(x, y, WHITE);
        for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
            t.set(x + dx, y + dy, SKY);
        }
    }
    t
}

/// A voidfall's stream: fine streaks of blue and white light running down a darker blue,
/// frame `f` of them having run further, and at the bottom breaking up into the dark.
fn fall(f: usize) -> Texture {
    let mut t = Texture::new(16, 64, INDIGO);
    for y in 0..64 {
        for x in 0..T {
            let h = hash2(x, 0, 0xFA11);
            let len = 4 + (h % 5) as i32;
            let yy = (y - f as i32 * 4 - (h % 16) as i32).rem_euclid(16);
            let c = if yy == len {
                WHITE
            } else if yy < len && yy >= len - 3 {
                SKY
            } else if yy < len || x % 4 == (h % 4) as i32 {
                BLUE
            } else {
                INDIGO
            };
            t.set(x, y, c);
            // Breaking up into drops towards the bottom.
            let fade = (y - 44).max(0) as u32;
            if fade > 0 && hash2(x, y, 0xD20) % 20 < fade {
                t.set(x, y, CLEAR);
            }
        }
    }
    t
}

/// The portal's swirl, turning by frame.
fn swirl(frame: usize) -> Texture {
    let mut t = Texture::new(16, 16, GRAPE);
    let turn = frame as f32 * std::f32::consts::FRAC_PI_2 * 0.5;
    for y in 0..T {
        for x in 0..T {
            let (dx, dy) = (x as f32 - 7.5, y as f32 - 7.5);
            let rr = dx.hypot(dy);
            let a = dy.atan2(dx) - turn;
            let k = (rr * 0.8 + a * 1.6).rem_euclid(std::f32::consts::TAU);
            let c = if rr < 1.8 {
                WHITE
            } else if k < 0.9 {
                LAVENDER
            } else if k < 1.8 {
                PURPLE
            } else if k > 5.4 {
                BLUSH
            } else {
                GRAPE
            };
            t.set(x, y, c);
        }
    }
    t
}

/// A rune, pale violet, to glow on a wall (the rest clear).
fn rune(k: usize) -> Texture {
    let shapes: [&[&str]; 4] = [
        &[
            "................",
            "......vvvv......",
            ".....v....v.....",
            "....v..vv..v....",
            "....v.v..v.v....",
            "....v..vv..v....",
            ".....v....v.....",
            "......vvvv......",
            ".......vv.......",
            ".......vv.......",
            ".....vvvvvv.....",
            ".......vv.......",
            ".......vv.......",
            "......v..v......",
            ".....v....v.....",
            "................",
        ],
        &[
            "................",
            "...v........v...",
            "....v......v....",
            ".....v....v.....",
            "......v..v......",
            ".......vv.......",
            "......vvvv......",
            ".....v.vv.v.....",
            "....v..vv..v....",
            ".......vv.......",
            ".......vv.......",
            "....vvvvvvvv....",
            ".......vv.......",
            "......vvvv......",
            ".......vv.......",
            "................",
        ],
        &[
            "................",
            ".......vv.......",
            "......v..v......",
            ".....v....v.....",
            "....vvvvvvvv....",
            "................",
            "..vv........vv..",
            "...vv......vv...",
            "....vv....vv....",
            ".....vv..vv.....",
            "......vvvv......",
            ".......vv.......",
            ".......vv.......",
            ".......vv.......",
            ".....vvvvvv.....",
            "................",
        ],
        &[
            "................",
            "....vvvvvvvv....",
            "....v......v....",
            "....v.vvvv.v....",
            "....v.v..v.v....",
            "....v.v..v.v....",
            "....v.vvvv.v....",
            "....v......v....",
            "....vvvvvvvv....",
            ".......vv.......",
            "...vvvvvvvvvv...",
            ".......vv.......",
            "......v..v......",
            ".....v....v.....",
            "....v......v....",
            "................",
        ],
    ];
    Texture::from_art(shapes[k % 4], &[('v', LAVENDER)])
}

/// A crystal: a six-sided prism with a point, `h` tall and `r` round, leaning `lean`.
fn crystal(m: &mut Mesh, at: Vec3, h: f32, r: f32, lean: Vec3, tex: TexId) {
    let mut p = Mesh::new();
    lathe(
        &mut p,
        Vec3::ZERO,
        &[(r * 0.7, 0.0), (r, h * 0.15), (r, h * 0.72), (0.0, h)],
        6,
        0.3,
        tex,
        false,
    );
    let rot = Quat::from_rotation_arc(Vec3::Y, lean.normalize());
    m.append(&p, Mat4::from_translation(at) * Mat4::from_quat(rot));
}

/// Amethyst: pale at the tips, deep violet at the root, with a bright edge.
fn amethyst_tex() -> Texture {
    let mut t = Texture::new(16, 16, PURPLE);
    for y in 0..T {
        for x in 0..T {
            let c = match y {
                0..=2 => BLUSH,
                3..=6 => LAVENDER,
                13..=15 => GRAPE,
                _ => PURPLE,
            };
            t.set(x, y, if x % 8 == 0 { LAVENDER } else { c });
        }
    }
    t
}

pub fn build(bank: &mut TexBank, marble: (TexId, TexId, TexId), columns: &[Mesh; 3]) -> RiftArt {
    let side = std::array::from_fn(|k| bank.add(obsidian(k as u8, 0x0B5 + k as u64)));
    let top = std::array::from_fn(|k| bank.add(obsidian_top(k as u8)));
    let ground_t = std::array::from_fn(|k| bank.add(ground(k, 0x6D0 + k as u64)));
    let pale_t = [bank.add(pale(0x9A1)), bank.add(pale(0x9A2))];
    let flags = [bank.add(flagstones(false)), bank.add(flagstones(true))];
    let deck_t = bank.add(deck());
    let beam_t = bank.add(beam());
    let rail_t = bank.add(rail());
    let cliff_t = std::array::from_fn(|k| bank.add(cliff(0xC11F + k as u64)));
    let fall_t = std::array::from_fn(|f| bank.add(fall(f)));
    let swirl_t = std::array::from_fn(|f| bank.add(swirl(f)));
    let w4 = Texture::new(4, 4, 0);
    let flat = |bank: &mut TexBank, c: u8| bank.add(Texture::new(4, 4, c));

    // Amethyst: a few crystals, a bigger bunch, and a great one towering over its fellows,
    // each on a lump of dark rock.
    let gem = bank.add(amethyst_tex());
    let rock = side[0];
    // Each crystal: where it stands (x, z), how tall and thick, and which way it leans.
    type Crystal = (f32, f32, f32, f32, f32, f32);
    let clusters: [&[Crystal]; 3] = [
        &[
            (0.0, 0.0, 0.26, 0.05, 0.0, 0.0),
            (0.08, 0.05, 0.18, 0.04, 0.5, 0.3),
            (-0.07, 0.04, 0.16, 0.035, -0.5, 0.2),
        ],
        &[
            (0.0, 0.0, 0.5, 0.08, 0.0, 0.0),
            (0.13, 0.05, 0.34, 0.06, 0.5, 0.2),
            (-0.12, 0.06, 0.3, 0.055, -0.5, 0.3),
            (0.03, -0.12, 0.26, 0.05, 0.1, -0.5),
        ],
        &[
            (0.0, 0.0, 1.15, 0.15, 0.0, 0.0),
            (0.2, 0.08, 0.62, 0.1, 0.45, 0.2),
            (-0.2, 0.06, 0.56, 0.09, -0.5, 0.25),
            (0.06, -0.2, 0.46, 0.08, 0.15, -0.5),
            (-0.1, 0.2, 0.36, 0.07, -0.2, 0.5),
        ],
    ];
    let amethyst = std::array::from_fn(|k| {
        let mut m = Mesh::new();
        let base = [0.12f32, 0.2, 0.34][k];
        tiled_box(
            &mut m,
            Vec3::new(-base, 0.0, -base * 0.8),
            Vec3::new(base, base * 0.5, base * 0.8),
            rock,
            0,
        );
        for &(x, z, h, r, lx, lz) in clusters[k] {
            crystal(
                &mut m,
                Vec3::new(x, base * 0.3, z),
                h,
                r,
                Vec3::new(lx, 1.0, lz),
                gem,
            );
        }
        m
    });

    // A voidfall's sheet: narrow at the lip, spreading a little as it falls, and standing
    // just out from the rock face behind it.
    let mut fall_sheet = Mesh::new();
    let depth = 3.2;
    fall_sheet.quad(
        [
            Vec3::new(-0.48, -depth, 0.07),
            Vec3::new(0.48, -depth, 0.07),
            Vec3::new(0.32, 0.0, 0.07),
            Vec3::new(-0.32, 0.0, 0.07),
        ],
        UvRect::new(0.0, 0.0, 16.0, 64.0),
        fall_t[0],
    );

    // A link of chain: a long oval of dark iron.
    let iron = bank.add({
        let mut t = Texture::new(4, 4, KHAKI);
        t.set(0, 0, SAND);
        t.set(3, 3, SHADOW);
        t.set(3, 2, SHADOW);
        t
    });
    let mut link = Mesh::new();
    let (lw, lh, t) = (0.1, 0.045, 0.018);
    for s in [-1.0f32, 1.0] {
        skin_box(
            &mut link,
            Vec3::new(-lw, s * lh - t, -t),
            Vec3::new(lw, s * lh + t, t),
            iron,
            &w4,
        );
        skin_box(
            &mut link,
            Vec3::new(s * lw - t, -lh, -t),
            Vec3::new(s * lw + t, lh, t),
            iron,
            &w4,
        );
    }
    // A chain's post: a squat pillar of obsidian with an iron ring atop it.
    let mut post = Mesh::new();
    tiled_box(
        &mut post,
        Vec3::new(-0.16, 0.0, -0.16),
        Vec3::new(0.16, 0.72, 0.16),
        side[4],
        0,
    );
    tiled_box(
        &mut post,
        Vec3::new(-0.2, 0.72, -0.2),
        Vec3::new(0.2, 0.8, 0.2),
        side[0],
        0,
    );
    skin_box(
        &mut post,
        Vec3::new(-0.05, 0.8, -0.05),
        Vec3::new(0.05, 0.9, 0.05),
        iron,
        &w4,
    );

    // An eye: a ball bulging out of a dark socket, a red-gold iris with a slit pupil in
    // front of it, and a lid of dark stone to close over it.
    let white = flat(bank, WHITE);
    let socket = flat(bank, INK);
    let mut eye_white = Mesh::new();
    let mut ball = Mesh::new();
    lathe(
        &mut ball,
        Vec3::ZERO,
        &[
            (0.0, -0.1),
            (0.07, -0.085),
            (0.1, -0.04),
            (0.1, 0.04),
            (0.07, 0.085),
            (0.0, 0.1),
        ],
        8,
        0.0,
        white,
        false,
    );
    let mut rim = Mesh::new();
    lathe(
        &mut rim,
        Vec3::ZERO,
        &[
            (0.0, -0.13),
            (0.1, -0.11),
            (0.13, 0.0),
            (0.1, 0.11),
            (0.0, 0.13),
        ],
        8,
        0.0,
        socket,
        false,
    );
    eye_white.append(&rim, Mat4::from_scale(Vec3::new(1.5, 0.95, 0.25)));
    eye_white.append(
        &ball,
        Mat4::from_translation(Vec3::Z * 0.01) * Mat4::from_scale(Vec3::new(1.35, 0.8, 0.5)),
    );
    let iris_t = bank.add({
        let mut t = Texture::new(8, 8, ORANGE);
        for y in 0..8 {
            for x in 0..8 {
                let (dx, dy) = (x as f32 - 3.5, y as f32 - 3.5);
                let rr = dx.hypot(dy);
                let c = if dx.abs() < 1.0 {
                    INK
                } else if rr > 3.2 {
                    RED
                } else if rr < 2.0 {
                    GOLD
                } else {
                    ORANGE
                };
                t.set(x, y, c);
            }
        }
        t
    });
    let mut iris = Mesh::new();
    iris.quad(
        [
            Vec3::new(-0.075, -0.075, 0.0),
            Vec3::new(0.075, -0.075, 0.0),
            Vec3::new(0.075, 0.075, 0.0),
            Vec3::new(-0.075, 0.075, 0.0),
        ],
        UvRect::new(0.0, 0.0, 8.0, 8.0),
        iris_t,
    );
    let mut lid = Mesh::new();
    tiled_box(
        &mut lid,
        Vec3::new(-0.17, -0.1, -0.01),
        Vec3::new(0.17, 0.1, 0.07),
        side[0],
        0,
    );
    skin_box(
        &mut lid,
        Vec3::new(-0.15, -0.01, 0.07),
        Vec3::new(0.15, 0.01, 0.075),
        socket,
        &w4,
    );

    // Runes: glowing glyphs on a flat panel.
    let runes = std::array::from_fn(|k| {
        let rt = rune(k);
        let tex = bank.add(rt);
        let mut m = Mesh::new();
        m.quad(
            [
                Vec3::new(-0.34, -0.34, 0.0),
                Vec3::new(0.34, -0.34, 0.0),
                Vec3::new(0.34, 0.34, 0.0),
                Vec3::new(-0.34, 0.34, 0.0),
            ],
            UvRect::new(0.0, 0.0, 16.0, 16.0),
            tex,
        );
        m
    });

    // The portal: two pillars of obsidian and a lintel, runes up them, and the swirl.
    let mut gate = Mesh::new();
    for x in [-0.36f32, 0.36] {
        tiled_box(
            &mut gate,
            Vec3::new(x - 0.12, 0.0, -0.14),
            Vec3::new(x + 0.12, 1.55, 0.14),
            side[1],
            0,
        );
    }
    tiled_box(
        &mut gate,
        Vec3::new(-0.56, 1.55, -0.17),
        Vec3::new(0.56, 1.8, 0.17),
        side[4],
        0,
    );
    let mut disc = Mesh::new();
    lathe(
        &mut disc,
        Vec3::ZERO,
        &[(0.0, -0.02), (0.24, 0.0), (0.0, 0.02)],
        12,
        0.0,
        swirl_t[0],
        false,
    );
    gate.append(
        &disc,
        Mat4::from_translation(Vec3::new(0.0, 0.78, 0.0))
            * Mat4::from_rotation_x(std::f32::consts::FRAC_PI_2)
            * Mat4::from_scale(Vec3::new(1.0, 1.0, 2.7)),
    );

    let (flute, stone, trim) = marble;
    let columns = columns.clone().map(|m| {
        m.retexture(flute, side[0])
            .retexture(stone, top[0])
            .retexture(trim, rail_t)
    });

    RiftArt {
        side,
        top,
        ground: ground_t,
        pale: pale_t,
        flags,
        deck: deck_t,
        beam: beam_t,
        rail: rail_t,
        cliff: cliff_t,
        stars: starfield(),
        amethyst,
        fall: fall_t,
        fall_sheet,
        link,
        post,
        eye_white,
        iris,
        lid,
        runes,
        gate,
        swirl: swirl_t,
        columns,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn everything_stays_in_the_palette() {
        let mut all = vec![
            obsidian_top(0),
            obsidian_top(4),
            pale(1),
            flagstones(true),
            deck(),
            beam(),
            rail(),
            amethyst_tex(),
            starfield(),
        ];
        all.extend((0..5).map(|k| obsidian(k, k as u64)));
        all.extend((0..3).map(|k| ground(k, k as u64)));
        all.extend((0..3).map(|k| cliff(k as u64)));
        all.extend((0..FALL_FRAMES).map(fall));
        all.extend((0..4).map(swirl));
        all.extend((0..4).map(rune));
        for t in all {
            for y in 0..t.h as i32 {
                for x in 0..t.w as i32 {
                    let c = t.get(x, y);
                    assert!(c < 32 || c == CLEAR, "({x}, {y})");
                }
            }
        }
    }

    #[test]
    fn island_faces_break_off_ragged_below() {
        for k in 0..3 {
            let t = cliff(k);
            assert_ne!(t.get(3, 2), CLEAR, "solid up top");
            assert!(
                (0..16).all(|x| t.get(x, 31) == CLEAR),
                "clear at the very bottom"
            );
            let lowest: Vec<i32> = (0..16)
                .map(|x| (0..32).rev().find(|&y| t.get(x, y) != CLEAR).unwrap_or(0))
                .collect();
            assert!(
                lowest.iter().max() != lowest.iter().min(),
                "not a straight edge"
            );
        }
    }

    #[test]
    fn the_stars_wrap_round() {
        let t = starfield();
        let stars = t.data.iter().filter(|&&c| c == WHITE || c == SKY).count();
        assert!(stars > 200, "{stars}");
        let clouds = t.data.iter().filter(|&&c| c == SLATE).count();
        assert!(clouds > 1000, "nebulae: {clouds}");
    }
}
