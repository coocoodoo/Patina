//! The Marble Labyrinth: white marble walls under a gilded band, cracked, mossy, with a
//! niche or a shield here and there, and piers with gilded capitals where they meet; cracked
//! marble flagstones and mosaics underfoot; and what stands about in it: columns whole,
//! broken and fallen, braziers of old green bronze, statues, rubble, a griffin's nest, arches
//! over the doorways, torches in brackets on the walls, and shafts of daylight.

use std::f32::consts::FRAC_PI_2;

use glam::{Mat4, Vec2, Vec3};

use super::models::{lathe, skin_box, tiled_box};
use crate::game::labyrinth::{CELL, CRACKED, MOSSY, NICHE, PIER, PLAIN, SHIELD, WALL_LOOKS};
use crate::palette::*;
use crate::render::{Mesh, TexBank, TexId, Texture, UvRect};
use crate::util::{Rng, hash2};

const T: i32 = 16;
/// A pier's face is taller than a wall's: its capital stands up above them (see
/// `labyrinth::PIER_H`).
pub const PIER_ROWS: i32 = 18;
/// An arch over a doorway: how far it spans either way from the middle, pier to pier; it
/// springs from the piers' tops and rises to its crown, a ring of voussoirs `RING` thick
/// and `DEPTH` either side of the line of the wall.
const SPAN: f32 = 1.5;
const SPRING: f32 = PIER_ROWS as f32 / 16.0;
const CROWN: f32 = 1.72;
const RING: f32 = 0.24;
const DEPTH: f32 = 0.2;

pub struct LabyrinthArt {
    /// Wall faces by look (see `labyrinth::PLAIN` and so on); a pier's is `PIER_ROWS` tall.
    pub side: [TexId; WALL_LOOKS],
    pub top: TexId,
    pub pier_top: TexId,
    /// Marble flagstones: two ways of laying them, one cracked and one broken; and the
    /// same in the sunlit courtyard, paler.
    pub flags: [TexId; 4],
    pub court: [TexId; 4],
    /// Mosaics by motif, and by where the tile falls in its cell (see
    /// `labyrinth::mosaic_at`).
    pub mosaic: [[TexId; 16]; 2],
    /// Columns: standing, broken, fallen.
    pub columns: [Mesh; 3],
    pub brazier: Mesh,
    /// A warrior with spear and shield, and a robed woman with an urn.
    pub statues: [Mesh; 2],
    /// A few chunks, a few more, and a heap.
    pub rubble: [Mesh; 3],
    pub nest: Mesh,
    /// An arch three wide, its front face flush with the walls' faces towards you.
    pub arch: Mesh,
    /// A torch in a bronze bracket on the wall behind (at z = -0.5).
    pub sconce: Mesh,
    /// A shaft of daylight leaning away up out of sight, drawn as added light.
    pub beam: Mesh,
    /// The columns' textures: their fluted shafts, plain marble and gilding (for recarving
    /// them in other stone: see `canyon_art`).
    pub stone: (TexId, TexId, TexId),
}

/// The walls' top rows: pale where the edge catches the light, then the gilded band and
/// the shadow under it.
fn band(t: &mut Texture, y: i32) {
    for x in 0..T {
        t.set(x, y, GOLD);
        t.set(
            x,
            y + 1,
            match x % 4 {
                0 => CLAY,
                2 => CREAM,
                _ => GOLD,
            },
        );
        t.set(x, y + 2, CLAY);
        t.set(x, y + 3, KHAKI);
    }
}

/// Marble blocks in two courses under the gilded band: pale along each block's top and
/// left edges, in pale joints, with a faint vein here and there.
fn marble(seed: u64) -> Texture {
    let mut t = Texture::new(16, 16, SAND);
    let mut r = Rng::new(seed);
    for x in 0..T {
        t.set(x, 0, WHITE);
    }
    band(&mut t, 1);
    for (y0, y1, joints) in [(5, 9, [7, 15]), (10, 15, [3, 11])] {
        for y in y0..=y1 {
            for x in 0..T {
                let c = if y == y1 || joints.contains(&x) {
                    KHAKI
                } else if y == y0 || joints.contains(&(x - 1).rem_euclid(T)) {
                    WHITE
                } else {
                    SAND
                };
                t.set(x, y, c);
            }
        }
    }
    let (mut x, mut y) = (r.range(0, T), r.range(6, 13));
    for _ in 0..r.range(2, 5) {
        if t.get(x.rem_euclid(T), y) == SAND {
            t.set_wrap(x, y, KHAKI);
        }
        x += 1;
        y = (y + r.range(-1, 2)).clamp(6, 14);
    }
    t
}

/// A crack zigzagging down a wall from under the band, and a chip knocked out of a block.
fn cracked(seed: u64) -> Texture {
    let mut t = marble(seed);
    let mut r = Rng::new(seed ^ 0xC4AC);
    let mut x = r.range(4, 12);
    for y in 5..T {
        t.set_wrap(x, y, SHADOW);
        if r.chance(0.4) {
            t.set_wrap(x + 1, y, INK);
        }
        if y % 4 == 2 && r.chance(0.7) {
            t.set_wrap(x - 1, y + 1, SHADOW);
            t.set_wrap(x - 2, y + 2, SHADOW);
        }
        x += r.range(-1, 2);
    }
    let (cx, cy) = (r.range(1, 12), r.range(10, 13));
    for (dx, dy, c) in [
        (0, 0, SHADOW),
        (1, 0, ROSEWOOD),
        (2, 0, ROSEWOOD),
        (0, 1, INK),
        (1, 1, SHADOW),
        (2, 1, ROSEWOOD),
        (1, 2, SHADOW),
    ] {
        t.set_wrap(cx + dx, cy + dy, c);
    }
    t
}

/// Moss running down from under the band, and creeping up from the foot of the wall.
fn mossy(seed: u64) -> Texture {
    let mut t = marble(seed);
    let mut r = Rng::new(seed ^ 0x3055);
    for _ in 0..4 {
        let x = r.range(0, T);
        let len = r.range(2, 8);
        for k in 0..len {
            let c = if k < 2 { GREEN } else { TEAL };
            t.set_wrap(x, 4 + k, c);
            if k == 0 {
                t.set_wrap(x + 1, 4, GREEN);
            }
        }
    }
    for x in 0..T {
        let h = (hash2(x, 3, seed as u32) % 4) as i32;
        for k in 0..h {
            t.set(x, 15 - k, if k == h - 1 { GREEN } else { DEEP_TEAL });
        }
    }
    t
}

/// An arched niche in the wall, dark within, a gilded keystone over it and a little urn
/// standing in it.
fn niche() -> Texture {
    let mut t = marble(0x1AB0);
    let (cx, spring, rad) = (8.0f32, 9.0f32, 3.5f32);
    for y in 4..T {
        for x in 0..T {
            let (px, py) = (x as f32 + 0.5, y as f32 + 0.5);
            let within =
                |r: f32| (px - cx).abs() < r && (py >= spring || (px - cx).hypot(py - spring) < r);
            if within(rad) {
                t.set(x, y, if py < spring + 1.0 { INK } else { SHADOW });
            } else if within(rad + 1.2) {
                t.set(x, y, if px < cx { WHITE } else { KHAKI });
            }
        }
    }
    for (x, y, c) in [(7, 4, GOLD), (8, 4, GOLD), (7, 5, CLAY), (8, 5, CLAY)] {
        t.set(x, y, c);
    }
    // The urn: a round belly on a foot, two handles.
    for (x, y, c) in [
        (7, 11, CLAY),
        (8, 11, CLAY),
        (6, 12, CLAY),
        (7, 12, GOLD),
        (8, 12, CLAY),
        (9, 12, RUST),
        (6, 13, RUST),
        (7, 13, CLAY),
        (8, 13, RUST),
        (9, 13, RUST),
        (7, 14, RUST),
        (8, 14, RUST),
        (5, 11, CLAY),
        (10, 11, RUST),
    ] {
        t.set(x, y, c);
    }
    t
}

/// A round gilded shield hung on the wall, with a boss in the middle.
fn shield() -> Texture {
    let mut t = marble(0x1AB1);
    let (cx, cy) = (8.0f32, 10.0f32);
    for y in 4..T {
        for x in 0..T {
            let (px, py) = (x as f32 + 0.5, y as f32 + 0.5);
            let d = (px - cx).hypot(py - cy);
            let c = if d < 1.3 {
                RUST
            } else if d < 3.3 {
                if px + py < cx + cy - 2.0 { CREAM } else { GOLD }
            } else if d < 4.4 {
                CLAY
            } else if d < 5.0 && py > cy {
                KHAKI
            } else {
                continue;
            };
            t.set(x, y, c);
        }
    }
    t
}

/// A pier: a plain slab on top, the gilded capital level with the walls' band, a fluted
/// pilaster down the middle between plain edges, and a moulded base.
fn pier() -> Texture {
    // (Textures are a power of two tall: only the top `PIER_ROWS` rows ever show.)
    let mut t = Texture::new(16, 32, SAND);
    for x in 0..T {
        t.set(x, 0, WHITE);
        t.set(x, 1, SAND);
        t.set(x, 2, KHAKI);
    }
    band(&mut t, 3);
    for x in 0..T {
        for y in 7..PIER_ROWS - 2 {
            let c = match x {
                0 | 15 => KHAKI,
                1 => WHITE,
                2 | 14 => SAND,
                13 => KHAKI,
                _ => [WHITE, SAND, KHAKI][((x - 3) % 3) as usize],
            };
            t.set(x, y, c);
        }
        t.set(x, PIER_ROWS - 2, WHITE);
        t.set(x, PIER_ROWS - 1, KHAKI);
    }
    // Volutes curling at the capital's corners.
    for (x, y) in [(1, 4), (2, 5), (1, 6), (14, 4), (13, 5), (14, 6)] {
        t.set(x, y, CLAY);
    }
    t
}

/// The walls' capstones: the palest marble, standing out over the floors below, lit along
/// the edge towards you.
fn top() -> Texture {
    let mut t = Texture::new(16, 16, WHITE);
    let mut r = Rng::new(0x1AB2);
    for _ in 0..14 {
        let (x, y) = (r.range(0, T), r.range(1, T - 2));
        t.set(x, y, SAND);
    }
    for _ in 0..2 {
        let (x, y) = (r.range(0, T), r.range(1, T - 2));
        t.set(x, y, KHAKI);
    }
    for x in 0..T {
        t.set(x, 0, SAND);
        t.set(x, T - 2, WHITE);
        t.set(x, T - 1, SAND);
    }
    for y in 1..T - 2 {
        t.set(0, y, SAND);
    }
    t
}

/// A pier's top: a square slab with a gilded rim.
fn pier_top() -> Texture {
    let mut t = Texture::new(16, 16, WHITE);
    for i in 0..T {
        t.set(i, 0, CLAY);
        t.set(T - 1, i, CLAY);
        t.set(i, T - 1, GOLD);
        t.set(0, i, GOLD);
        if (1..T - 1).contains(&i) {
            t.set(i, 1, SAND);
            t.set(T - 2, i, SAND);
        }
    }
    for (x, y) in [(5, 6), (10, 9), (7, 11), (4, 12)] {
        t.set(x, y, SAND);
    }
    t
}

/// Marble flagstones, big slabs in fine joints, lit along their near edges: two to a tile
/// laid one way or the other (`kind` 0 and 1), cracked across (2), or with a corner broken
/// away to the dark earth under it and moss coming through (3). In the sunlit courtyard
/// (`sunlit`) they're paler still.
fn flagstones(kind: usize, sunlit: bool, seed: u64) -> Texture {
    let mut t = Texture::new(16, 16, KHAKI);
    let mut r = Rng::new(seed);
    let slabs: &[(i32, i32, i32, i32)] = if kind % 2 == 0 {
        &[(0, 0, 16, 9), (0, 9, 16, 7)]
    } else {
        &[(0, 0, 9, 16), (9, 0, 7, 16)]
    };
    for &(sx, sy, sw, sh) in slabs {
        let pale = r.chance(if sunlit { 0.45 } else { 0.2 });
        let stone = if pale { WHITE } else { SAND };
        for y in 0..sh - 1 {
            for x in 0..sw - 1 {
                let c = if y == 0 || x == 0 { WHITE } else { stone };
                t.set_wrap(sx + x, sy + y, c);
            }
        }
        for _ in 0..r.range(1, 4) {
            let (x, y) = (sx + r.range(1, sw - 2), sy + r.range(1, sh - 2));
            t.set_wrap(x, y, if pale { SAND } else { KHAKI });
        }
    }
    if kind == 2 {
        // A crack across a slab, forking.
        let (mut x, mut y) = (r.range(3, 8), r.range(2, 5));
        for k in 0..7 {
            t.set(x.clamp(0, T - 1), y.clamp(0, T - 1), SHADOW);
            if k == 3 {
                t.set((x + 1).clamp(0, T - 1), (y + 1).clamp(0, T - 1), KHAKI);
                t.set((x + 2).clamp(0, T - 1), (y + 1).clamp(0, T - 1), SHADOW);
            }
            x += r.range(-1, 2);
            y += 1;
        }
    }
    if kind == 3 {
        let (cx, cy) = (r.range(9, 13), r.range(9, 13));
        for y in cy - 3..=cy + 3 {
            for x in cx - 3..=cx + 3 {
                let d = (x - cx).abs() + (y - cy).abs();
                let c = match d {
                    0..=1 => INK,
                    2 => SHADOW,
                    3 => ROSEWOOD,
                    4 => KHAKI,
                    _ => continue,
                };
                t.set_wrap(x, y, c);
            }
        }
        t.set_wrap(cx, cy, GREEN);
        t.set_wrap(cx + 1, cy - 1, TEAL);
    }
    t
}

/// One cell's mosaic, `CELL * 16` texels square: the cell's own floor (from 16 to the end
/// each way) framed in a dogtooth border round a medallion, a star (motif 0) or a rosette
/// (1), with a diamond in each corner; and the thresholds on the line of the walls between
/// cells (the first 16 each way), a band of checks, with a gilded diamond where they cross.
fn mosaic_canvas(motif: usize) -> Texture {
    let n = CELL * 16;
    let mut t = Texture::new(n as u32, n as u32, SAND);
    let mut r = Rng::new(0x1AB3 + motif as u64);
    let (ink, dark, mid, light) = if motif == 0 {
        (INK, SLATE, INDIGO, BLUE)
    } else {
        (INK, MAROON, RUST, CLAY)
    };
    for y in 0..n {
        for x in 0..n {
            let c = if x >= 16 && y >= 16 {
                // The cell's own floor.
                let (lx, ly) = (x - 16, y - 16);
                let last = n - 17;
                let d = lx.min(ly).min(last - lx).min(last - ly);
                let (along, across) = if ly.min(last - ly) == d {
                    (lx, d)
                } else {
                    (ly, d)
                };
                let (fx, fy) = (lx as f32 - last as f32 * 0.5, ly as f32 - last as f32 * 0.5);
                let rho = fx.hypot(fy);
                let corner = lx.min(last - lx) <= 4 && ly.min(last - ly) <= 4;
                match d {
                    0 | 5 => ink,
                    1..=4 if corner => GOLD,
                    1..=4 => {
                        if (along % 6) < across {
                            RUST
                        } else {
                            WHITE
                        }
                    }
                    _ if rho > 13.2 => {
                        // The corners of the field: a diamond in each.
                        let (ax, ay) = (fx.abs() - 16.0, fy.abs() - 16.0);
                        let dd = ax.abs() + ay.abs();
                        if dd < 1.2 {
                            GOLD
                        } else if dd < 3.6 {
                            mid
                        } else if dd < 4.6 {
                            ink
                        } else {
                            SAND
                        }
                    }
                    _ if rho > 12.0 => ink,
                    _ if rho > 10.8 => GOLD,
                    _ if rho > 10.0 => ink,
                    _ => {
                        let (ax, ay) = (fx.abs(), fy.abs());
                        let inside = if motif == 0 {
                            let d = ax.max(ay).min((ax + ay) * 0.72);
                            d < 7.2
                        } else {
                            let a = fy.atan2(fx);
                            rho < 4.8 + 4.2 * (a * 4.0).cos().abs()
                        };
                        if rho < 2.6 {
                            if rho < 1.0 { CREAM } else { GOLD }
                        } else if inside {
                            let edge = if motif == 0 {
                                ax.max(ay).min((ax + ay) * 0.72) > 6.0
                            } else {
                                let a = fy.atan2(fx);
                                rho > 3.8 + 4.2 * (a * 4.0).cos().abs()
                            };
                            if edge {
                                dark
                            } else if (fx + fy) < -2.0 {
                                light
                            } else {
                                mid
                            }
                        } else {
                            WHITE
                        }
                    }
                }
            } else if x < 16 && y < 16 {
                // Where the thresholds cross: a gilded diamond.
                let dd = (x as f32 - 7.5).abs() + (y as f32 - 7.5).abs();
                if dd < 2.0 {
                    CREAM
                } else if dd < 5.0 {
                    GOLD
                } else if dd < 6.0 {
                    ink
                } else {
                    SAND
                }
            } else {
                // A threshold: a band of checks between two lines.
                let (across, along) = if x < 16 { (x, y) } else { (y, x) };
                match across {
                    2 | 13 => ink,
                    3 | 12 => mid,
                    5..=10 => {
                        if ((across - 5) / 2 + along / 2) % 2 == 0 {
                            ink
                        } else {
                            WHITE
                        }
                    }
                    _ => SAND,
                }
            };
            t.set(x, y, c);
        }
    }
    // Tesserae: a few chips a shade off here and there.
    for _ in 0..n * 2 {
        let (x, y) = (r.range(0, n), r.range(0, n));
        match t.get(x, y) {
            SAND => t.set(x, y, WHITE),
            WHITE => t.set(x, y, SAND),
            _ => {}
        }
    }
    t
}

/// The 16 square at (`u`, `v`) (in tiles) out of a bigger texture.
fn piece(t: &Texture, u: i32, v: i32) -> Texture {
    let mut p = Texture::new(16, 16, SAND);
    for y in 0..T {
        for x in 0..T {
            p.set(x, y, t.get(u * T + x, v * T + y));
        }
    }
    p
}

/// A column's fluted shaft: pale ridges between shaded flutes.
fn fluted() -> Texture {
    let mut t = Texture::new(16, 16, SAND);
    for y in 0..T {
        for x in 0..T {
            t.set(x, y, [WHITE, SAND, SAND, KHAKI][(x % 4) as usize]);
        }
    }
    t
}

/// Plain marble for plinths, statues and rubble.
fn slab() -> Texture {
    let mut t = Texture::new(16, 16, SAND);
    let mut r = Rng::new(0x1AB4);
    for _ in 0..14 {
        let (x, y) = (r.range(0, T), r.range(0, T));
        t.set(x, y, WHITE);
    }
    for _ in 0..6 {
        let (x, y) = (r.range(0, T), r.range(0, T));
        t.set(x, y, KHAKI);
    }
    t
}

/// Old bronze gone green: verdigris streaked pale, and a glint of the metal here and there.
fn bronze() -> Texture {
    let mut t = Texture::new(16, 16, TEAL);
    let mut r = Rng::new(0x1AB5);
    for y in 0..T {
        for x in 0..T {
            if (x * 3 + y) % 7 == 0 {
                t.set(x, y, AQUA);
            } else if (x + y * 5) % 11 == 0 {
                t.set(x, y, DEEP_TEAL);
            }
        }
    }
    for _ in 0..4 {
        let (x, y) = (r.range(0, T), r.range(0, T));
        t.set(x, y, MINT);
    }
    for _ in 0..2 {
        let (x, y) = (r.range(0, T), r.range(0, T));
        t.set(x, y, CLAY);
    }
    t
}

/// Glowing coals.
fn coals() -> Texture {
    let mut t = Texture::new(8, 8, MAROON);
    for (x, y, c) in [
        (1, 1, ORANGE),
        (4, 2, GOLD),
        (6, 5, ORANGE),
        (2, 5, RED),
        (5, 0, RED),
        (0, 7, ORANGE),
        (3, 3, RUST),
    ] {
        t.set(x, y, c);
    }
    t
}

/// Twigs and straw, woven into a nest.
fn straw() -> Texture {
    let mut t = Texture::new(16, 16, ROSEWOOD);
    for y in 0..T {
        for x in 0..T {
            let c = match (x * 5 + y * 3 + (x * y) % 7) % 11 {
                0 => KHAKI,
                1 | 2 => SHADOW,
                3 => CLAY,
                4 => SAND,
                5 => INK,
                _ => continue,
            };
            t.set(x, y, c);
        }
    }
    t
}

/// The shaft of daylight: soft and pale, a faint streak or two running up it.
fn beam_tex() -> Texture {
    let mut t = Texture::new(16, 32, SAND);
    for y in 0..32 {
        for x in 0..T {
            let c = match (x * 7 + y / 8) % 9 {
                0 => WHITE,
                1 | 2 => CREAM,
                _ => SAND,
            };
            t.set(x, y, c);
        }
    }
    t
}

/// A column standing `h` tall: a square plinth, a moulded base, the fluted shaft, and a
/// gilded capital under its square top. Broken (`broken`), it stops short in a jagged
/// break with a chunk lying at its foot.
fn column(m: &mut Mesh, h: f32, broken: bool, (flute, marble, gold): (TexId, TexId, TexId)) {
    tiled_box(
        m,
        Vec3::new(-0.3, 0.0, -0.3),
        Vec3::new(0.3, 0.14, 0.3),
        marble,
        0,
    );
    lathe(
        m,
        Vec3::Y * 0.14,
        &[(0.26, 0.0), (0.26, 0.05), (0.21, 0.09)],
        10,
        0.0,
        marble,
        false,
    );
    if broken {
        lathe(
            m,
            Vec3::Y * 0.23,
            &[(0.2, 0.0), (0.19, h - 0.23)],
            10,
            0.0,
            flute,
            false,
        );
        // The break: a few teeth of marble left standing round a rough top.
        lathe(
            m,
            Vec3::Y * h,
            &[(0.19, 0.0), (0.0, 0.02)],
            10,
            0.0,
            marble,
            false,
        );
        for (a, rise) in [(0.3f32, 0.16f32), (1.9, 0.09), (3.6, 0.2), (5.0, 0.07)] {
            let (x, z) = (a.cos() * 0.13, a.sin() * 0.13);
            tiled_box(
                m,
                Vec3::new(x - 0.06, h - 0.02, z - 0.06),
                Vec3::new(x + 0.06, h + rise, z + 0.06),
                flute,
                0,
            );
        }
        let mut chunk = Mesh::new();
        tiled_box(
            &mut chunk,
            Vec3::new(-0.08, 0.0, -0.07),
            Vec3::new(0.08, 0.1, 0.07),
            marble,
            0,
        );
        m.append(
            &chunk,
            Mat4::from_translation(Vec3::new(0.34, 0.0, 0.26)) * Mat4::from_rotation_y(0.6),
        );
        return;
    }
    lathe(
        m,
        Vec3::Y * 0.23,
        &[(0.2, 0.0), (0.18, h - 0.45)],
        10,
        0.0,
        flute,
        false,
    );
    lathe(
        m,
        Vec3::Y * (h - 0.22),
        &[(0.18, 0.0), (0.2, 0.03), (0.27, 0.09)],
        10,
        0.0,
        gold,
        false,
    );
    tiled_box(
        m,
        Vec3::new(-0.29, h - 0.13, -0.29),
        Vec3::new(0.29, h, 0.29),
        marble,
        0,
    );
}

/// A drum of a fallen column lying along x, `len` long.
fn drum(m: &mut Mesh, at: Vec3, len: f32, turn: f32, flute: TexId) {
    let mut d = Mesh::new();
    lathe(
        &mut d,
        Vec3::ZERO,
        &[(0.19, 0.0), (0.19, len)],
        10,
        0.0,
        flute,
        true,
    );
    m.append(
        &d,
        Mat4::from_translation(at + Vec3::Y * 0.19)
            * Mat4::from_rotation_y(turn)
            * Mat4::from_rotation_z(-FRAC_PI_2)
            * Mat4::from_translation(Vec3::new(0.0, -len * 0.5, 0.0)),
    );
}

/// A marble chunk, `s` across, tipped over a little.
fn chunk(m: &mut Mesh, at: Vec3, s: Vec3, turn: f32, tip: f32, tex: TexId) {
    let mut c = Mesh::new();
    tiled_box(
        &mut c,
        -s * Vec3::new(0.5, 0.0, 0.5),
        s * Vec3::new(0.5, 1.0, 0.5),
        tex,
        0,
    );
    m.append(
        &c,
        Mat4::from_translation(at) * Mat4::from_rotation_y(turn) * Mat4::from_rotation_z(tip),
    );
}

/// A point on an arch's ring at `a` (0 at its east foot, PI at its west), on the inside of
/// the ring (`out` 0) or the outside (1).
pub fn arch_point(a: f32, out: f32) -> Vec2 {
    let grow = RING * out;
    Vec2::new(
        (SPAN + grow) * a.cos(),
        SPRING + (CROWN - SPRING + grow) * a.sin(),
    )
}

/// An arch's voussoirs, laid out along the ring from its west foot to its east: pale on
/// the outer edge, shaded on the inner, in fine joints, with a gilded keystone at the crown.
fn voussoirs() -> Texture {
    let mut t = Texture::new(64, 8, SAND);
    for x in 0..64 {
        let key = (30..34).contains(&x);
        for y in 0..8 {
            let c = if key {
                match (x, y) {
                    (30 | 33, _) | (_, 0) => CLAY,
                    (_, 1) => CREAM,
                    _ => GOLD,
                }
            } else if x % 6 == 5 {
                KHAKI
            } else {
                match y {
                    0 => WHITE,
                    1 | 2 => SAND,
                    3 => KHAKI,
                    _ => SAND,
                }
            };
            t.set(x, y, c);
        }
    }
    t
}

/// Who a statue shows: a hero with a sword held high and a shield, or a woman in a long
/// robe bearing an urn.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Carving {
    Hero,
    Bearer,
}

/// How tall the plinths stand.
const PLINTH: f32 = 0.3;

/// A statue: one of the Hollow's own folk (see `models::humanoid`) carved in pale marble,
/// larger than life, posed and set on a plinth.
fn statue(bank: &mut TexBank, who: Carving, marble: TexId, gold: TexId) -> Mesh {
    use super::models::{ARM_L, ARM_R, BODY, HEAD, Hair, LEG_L, LEG_R, Look, humanoid};
    let look = Look {
        hair: [WHITE, SAND, KHAKI],
        skin: [WHITE, WHITE, SAND],
        // Carved eyes, a faint shade in the stone.
        eyes: KHAKI,
        cheeks: WHITE,
        shirt: [WHITE, SAND, KHAKI],
        belt: KHAKI,
        pants: SAND,
        boots: KHAKI,
        style: if who == Carving::Hero {
            Hair::Spiky
        } else {
            Hair::Long
        },
        beard: who == Carving::Hero,
        scale: 1.3,
    };
    let h = humanoid(bank, &look);
    let s = look.scale;
    let mut m = Mesh::new();
    tiled_box(
        &mut m,
        Vec3::new(-0.32, 0.0, -0.32),
        Vec3::new(0.32, PLINTH - 0.05, 0.32),
        marble,
        0,
    );
    tiled_box(
        &mut m,
        Vec3::new(-0.35, PLINTH - 0.06, -0.35),
        Vec3::new(0.35, PLINTH, 0.35),
        marble,
        0,
    );
    skin_box(
        &mut m,
        Vec3::new(-0.12, 0.08, 0.32),
        Vec3::new(0.12, 0.17, 0.335),
        gold,
        &Texture::new(4, 4, 0),
    );
    let root = Mat4::from_translation(Vec3::Y * PLINTH);
    let at = |x: f32, y: f32| root * Mat4::from_translation(Vec3::new(x, y, 0.0));
    for (i, side) in [(LEG_L, -1.0f32), (LEG_R, 1.0)] {
        m.append(&h.parts[i], at(side * h.hip_x, h.hip));
    }
    m.append(&h.parts[BODY], at(0.0, h.hip - 0.02));
    m.append(
        &h.parts[HEAD],
        at(0.0, h.neck) * Mat4::from_rotation_x(-0.12),
    );
    // The hand that's free is on -x, the other on +x (see `draw::HAND`).
    let (right, left) = match who {
        Carving::Hero => (
            at(-h.shoulder_x, h.shoulder) * Mat4::from_rotation_x(-2.75),
            at(h.shoulder_x, h.shoulder) * Mat4::from_rotation_x(-0.55),
        ),
        Carving::Bearer => (
            at(-h.shoulder_x, h.shoulder)
                * Mat4::from_rotation_x(-1.05)
                * Mat4::from_rotation_z(0.35),
            at(h.shoulder_x, h.shoulder)
                * Mat4::from_rotation_x(-1.05)
                * Mat4::from_rotation_z(-0.35),
        ),
    };
    m.append(&h.parts[ARM_R], right);
    m.append(&h.parts[ARM_L], left);
    let v = |x: f32, y: f32, z: f32| Vec3::new(x, y, z);
    match who {
        Carving::Hero => {
            // A sword held high...
            let mut sword = Mesh::new();
            tiled_box(
                &mut sword,
                v(-0.03, -0.72, -0.012),
                v(0.03, -0.26, 0.012),
                marble,
                0,
            );
            tiled_box(
                &mut sword,
                v(-0.1, -0.28, -0.03),
                v(0.1, -0.24, 0.03),
                gold,
                0,
            );
            m.append(
                &sword,
                right * Mat4::from_translation(v(0.0, -h.hand + 0.2, 0.03)),
            );
            // ...and a round shield on the other arm, facing out.
            let mut shield = Mesh::new();
            lathe(
                &mut shield,
                Vec3::ZERO,
                &[(0.2, 0.0), (0.2, 0.03), (0.08, 0.06), (0.0, 0.07)],
                10,
                0.0,
                marble,
                false,
            );
            m.append(
                &shield,
                left * Mat4::from_translation(v(0.07, -0.16, 0.04))
                    * Mat4::from_rotation_y(-0.8)
                    * Mat4::from_rotation_z(-FRAC_PI_2),
            );
        }
        Carving::Bearer => {
            // A long robe over the legs, and an urn held out in both hands.
            tiled_box(
                &mut m,
                v(-0.2 * s, PLINTH, -0.15 * s),
                v(0.2 * s, PLINTH + h.hip + 0.04, 0.15 * s),
                marble,
                0,
            );
            lathe(
                &mut m,
                v(0.0, PLINTH + h.shoulder - 0.2 * s, 0.3 * s),
                &[
                    (0.05, 0.0),
                    (0.1, 0.05),
                    (0.11, 0.12),
                    (0.06, 0.18),
                    (0.07, 0.22),
                ],
                8,
                0.0,
                gold,
                false,
            );
        }
    }
    m
}

pub fn build(bank_: &mut TexBank) -> LabyrinthArt {
    let side = std::array::from_fn(|look| {
        let t = match look as u8 {
            PLAIN => marble(0x1A00),
            CRACKED => cracked(0x1A01),
            MOSSY => mossy(0x1A02),
            NICHE => niche(),
            SHIELD => shield(),
            PIER => pier(),
            _ => marble(0x1A03),
        };
        bank_.add(t)
    });
    let top = bank_.add(top());
    let pier_top = bank_.add(pier_top());
    let flags = std::array::from_fn(|k| bank_.add(flagstones(k, false, 0x1AF0 + k as u64)));
    let court = std::array::from_fn(|k| bank_.add(flagstones(k, true, 0x1AF8 + k as u64)));
    let mosaic = std::array::from_fn(|motif| {
        let canvas = mosaic_canvas(motif);
        std::array::from_fn(|k| {
            let (u, v) = (k as i32 % CELL, k as i32 / CELL);
            bank_.add(piece(&canvas, u, v))
        })
    });

    let flute = bank_.add(fluted());
    let marble = bank_.add(slab());
    let gold = bank_.add(Texture::new(4, 4, GOLD));
    let stone = (flute, marble, gold);
    let mut standing = Mesh::new();
    column(&mut standing, 1.55, false, stone);
    let mut broken = Mesh::new();
    column(&mut broken, 0.78, true, stone);
    let mut fallen = Mesh::new();
    drum(&mut fallen, Vec3::new(-0.08, 0.0, -0.12), 0.52, 0.25, flute);
    drum(&mut fallen, Vec3::new(0.18, 0.0, 0.2), 0.3, -0.5, flute);
    chunk(
        &mut fallen,
        Vec3::new(-0.3, 0.0, 0.26),
        Vec3::new(0.3, 0.12, 0.3),
        0.4,
        0.15,
        marble,
    );

    // The brazier: a round foot, a slender stem and a wide bowl of old bronze on three
    // splayed legs, heaped with coals.
    let bronze = bank_.add(bronze());
    let coals = bank_.add(coals());
    let mut brazier = Mesh::new();
    lathe(
        &mut brazier,
        Vec3::ZERO,
        &[
            (0.2, 0.0),
            (0.19, 0.05),
            (0.08, 0.1),
            (0.06, 0.4),
            (0.1, 0.46),
        ],
        8,
        0.0,
        bronze,
        false,
    );
    lathe(
        &mut brazier,
        Vec3::Y * 0.44,
        &[(0.1, 0.0), (0.24, 0.08), (0.32, 0.2), (0.35, 0.3)],
        10,
        0.0,
        bronze,
        false,
    );
    lathe(
        &mut brazier,
        Vec3::Y * 0.73,
        &[(0.37, 0.0), (0.37, 0.05)],
        10,
        0.0,
        gold,
        false,
    );
    lathe(
        &mut brazier,
        Vec3::Y * 0.7,
        &[(0.33, 0.0), (0.2, 0.05), (0.0, 0.07)],
        10,
        0.0,
        coals,
        false,
    );
    for k in 0..3 {
        let a = k as f32 * std::f32::consts::TAU / 3.0 + 0.5;
        let mut leg = Mesh::new();
        skin_box(
            &mut leg,
            Vec3::new(-0.025, 0.0, -0.025),
            Vec3::new(0.025, 0.56, 0.025),
            bronze,
            &Texture::new(16, 16, 0),
        );
        brazier.append(
            &leg,
            Mat4::from_rotation_y(a)
                * Mat4::from_translation(Vec3::new(0.3, 0.0, 0.0))
                * Mat4::from_rotation_z(0.28),
        );
    }

    // Statues: folk of the Hollow carved in marble, on plinths with a gilded plaque.
    let statues = [
        statue(bank_, Carving::Hero, marble, gold),
        statue(bank_, Carving::Bearer, marble, gold),
    ];

    // Rubble: chunks of marble lying about, and a heap of them round a fallen drum.
    let mut rubble: [Mesh; 3] = Default::default();
    let mut r = Rng::new(0x1AB6);
    for (k, m) in rubble.iter_mut().enumerate() {
        let n = [4, 6, 9][k];
        for i in 0..n {
            let s = if k == 2 {
                r.range_f(0.14, 0.28)
            } else {
                r.range_f(0.07, 0.15)
            };
            let spread = if k == 2 { 0.28 } else { 0.34 };
            let lift = if k == 2 && i > 4 { 0.12 } else { 0.0 };
            chunk(
                m,
                Vec3::new(r.range_f(-spread, spread), lift, r.range_f(-spread, spread)),
                Vec3::new(s, s * r.range_f(0.5, 0.9), s * r.range_f(0.7, 1.1)),
                r.f32() * 3.0,
                r.range_f(-0.3, 0.3),
                if i % 3 == 0 { flute } else { marble },
            );
        }
        if k == 2 {
            drum(m, Vec3::new(0.05, 0.0, 0.1), 0.42, 0.7, flute);
        }
    }

    // The griffin's nest: a broken column with a nest of straw and twigs on top, and two
    // eggs in it.
    let straw = bank_.add(straw());
    let shell = bank_.add(Texture::new(4, 4, CREAM));
    let mut nest = Mesh::new();
    column(&mut nest, 0.95, false, stone);
    lathe(
        &mut nest,
        Vec3::Y * 0.93,
        &[
            (0.2, 0.0),
            (0.36, 0.05),
            (0.4, 0.13),
            (0.34, 0.2),
            (0.26, 0.16),
            (0.0, 0.1),
        ],
        10,
        0.0,
        straw,
        false,
    );
    for (a, len) in [
        (0.2f32, 0.2f32),
        (1.4, 0.16),
        (2.6, 0.22),
        (3.9, 0.18),
        (5.1, 0.2),
    ] {
        let mut twig = Mesh::new();
        skin_box(
            &mut twig,
            Vec3::new(0.0, -0.012, -0.012),
            Vec3::new(len, 0.012, 0.012),
            straw,
            &Texture::new(16, 16, 0),
        );
        nest.append(
            &twig,
            Mat4::from_translation(Vec3::new(a.cos() * 0.3, 1.08, -a.sin() * 0.3))
                * Mat4::from_rotation_y(a)
                * Mat4::from_rotation_z(0.25),
        );
    }
    for (x, z) in [(-0.07f32, 0.03f32), (0.08, -0.04)] {
        lathe(
            &mut nest,
            Vec3::new(x, 1.04, z),
            &[(0.0, 0.0), (0.06, 0.03), (0.055, 0.09), (0.0, 0.13)],
            6,
            0.0,
            shell,
            false,
        );
    }

    // The arch: a ring of voussoirs springing from the tops of the piers either side of
    // the doorway, high over it, with a gilded keystone at its crown.
    let ring = bank_.add(voussoirs());
    let mut arch = Mesh::new();
    const STEPS: usize = 14;
    for k in 0..STEPS {
        let (a0, a1) = (
            std::f32::consts::PI * (1.0 - k as f32 / STEPS as f32),
            std::f32::consts::PI * (1.0 - (k + 1) as f32 / STEPS as f32),
        );
        let (i0, i1, o0, o1) = (
            arch_point(a0, 0.0),
            arch_point(a1, 0.0),
            arch_point(a0, 1.0),
            arch_point(a1, 1.0),
        );
        let (u0, u1) = (
            64.0 * k as f32 / STEPS as f32,
            64.0 * (k + 1) as f32 / STEPS as f32,
        );
        let at = |p: Vec2, z: f32| Vec3::new(p.x, p.y, z);
        // Its face towards you...
        let face = [at(i0, DEPTH), at(i1, DEPTH), at(o1, DEPTH), at(o0, DEPTH)];
        let uv = [
            Vec2::new(u0, 4.0),
            Vec2::new(u1, 4.0),
            Vec2::new(u1, 0.0),
            Vec2::new(u0, 0.0),
        ];
        arch.tri([face[0], face[1], face[2]], [uv[0], uv[1], uv[2]], ring);
        arch.tri([face[0], face[2], face[3]], [uv[0], uv[2], uv[3]], ring);
        // ...and over the top of it (its back and underside never face you).
        arch.quad(
            [at(o0, DEPTH), at(o1, DEPTH), at(o1, -DEPTH), at(o0, -DEPTH)],
            UvRect::new(u0, 0.0, u1, 2.0),
            ring,
        );
    }

    // The sconce: a bronze plate on the wall, an arm out from it and a cup holding a torch.
    let wood = bank_.add(Texture::new(4, 4, CLAY));
    let w4 = Texture::new(4, 4, 0);
    let mut sconce = Mesh::new();
    skin_box(
        &mut sconce,
        Vec3::new(-0.07, 0.42, -0.5),
        Vec3::new(0.07, 0.66, -0.46),
        bronze,
        &Texture::new(16, 16, 0),
    );
    skin_box(
        &mut sconce,
        Vec3::new(-0.025, 0.5, -0.46),
        Vec3::new(0.025, 0.54, -0.3),
        bronze,
        &w4,
    );
    lathe(
        &mut sconce,
        Vec3::new(0.0, 0.5, -0.28),
        &[(0.025, 0.0), (0.06, 0.08)],
        6,
        0.0,
        bronze,
        false,
    );
    skin_box(
        &mut sconce,
        Vec3::new(-0.025, 0.54, -0.305),
        Vec3::new(0.025, 0.76, -0.255),
        wood,
        &w4,
    );

    // The shaft of daylight: a square beam leaning away and up out of sight.
    let light = bank_.add(beam_tex());
    let mut beam = Mesh::new();
    let lean = Vec3::new(1.3, 3.6, -1.0);
    let s = 0.5;
    let foot = [
        Vec3::new(-s, 0.0, s),
        Vec3::new(s, 0.0, s),
        Vec3::new(s, 0.0, -s),
        Vec3::new(-s, 0.0, -s),
    ];
    for k in 0..4 {
        let (a, b) = (foot[k], foot[(k + 1) % 4]);
        beam.quad(
            [a, b, b + lean, a + lean],
            UvRect::new(0.0, 0.0, 16.0, 32.0),
            light,
        );
    }

    LabyrinthArt {
        side,
        top,
        pier_top,
        flags,
        court,
        mosaic,
        columns: [standing, broken, fallen],
        brazier,
        statues,
        rubble,
        nest,
        arch,
        sconce,
        beam,
        stone,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn everything_stays_in_the_palette() {
        let mut all = vec![
            marble(1),
            cracked(2),
            mossy(3),
            niche(),
            shield(),
            pier(),
            top(),
            pier_top(),
            fluted(),
            slab(),
            bronze(),
            coals(),
            straw(),
            beam_tex(),
            voussoirs(),
            mosaic_canvas(0),
            mosaic_canvas(1),
        ];
        all.extend((0..8).map(|k| flagstones(k % 4, k > 3, k as u64)));
        for t in all {
            for y in 0..t.h as i32 {
                for x in 0..t.w as i32 {
                    assert!(t.get(x, y) < 32, "({x}, {y})");
                }
            }
        }
    }

    #[test]
    fn the_band_runs_level_from_wall_to_pier_to_arch() {
        // A pier stands two texels taller, so its band sits two rows lower on its face.
        assert_eq!(
            PIER_ROWS as f32 / 16.0,
            crate::game::labyrinth::PIER_H,
            "a pier's face fits it exactly"
        );
        let (wall, pier) = (marble(1), pier());
        for x in 0..T {
            for y in 1..5 {
                // (Bar the volutes at the capital's corners.)
                if (3..13).contains(&x) {
                    assert_eq!(wall.get(x, y), pier.get(x, y + 2), "({x}, {y})");
                }
            }
            assert_eq!(wall.get(x, 1), GOLD);
        }
    }

    #[test]
    fn an_arch_springs_from_the_piers_and_rises_over_the_doorway() {
        use std::f32::consts::PI;
        // Its feet stand on the piers' tops either side of the doorway, three wide.
        for a in [0.0, PI] {
            let (inner, outer) = (arch_point(a, 0.0), arch_point(a, 1.0));
            assert!((inner.y - crate::game::labyrinth::PIER_H).abs() < 1e-5);
            assert!((inner.x.abs() - 1.5).abs() < 1e-5);
            assert!(outer.x.abs() > 1.5 && outer.x.abs() < 2.5, "on the pier");
        }
        // And the crown is well over anyone's head.
        assert!((arch_point(FRAC_PI_2, 0.0).y - CROWN).abs() < 1e-5);
        const { assert!(CROWN > 1.6) };
    }
}
