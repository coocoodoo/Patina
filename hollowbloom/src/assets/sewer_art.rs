//! The Hollow's old sewers: stone walkways either side of murky green channels, their curbs
//! along the water, mossy brick walls, and bridges across of weathered planks or of copper
//! gone green with age. Plus the pipes that drain into them and the grates in the floor.

use std::f32::consts::FRAC_PI_2;

use glam::{Mat4, Vec3};

use super::models::lathe;
use crate::palette::*;
use crate::render::{Mesh, TexBank, TexId, Texture, UvRect};
use crate::util::Rng;

const T: i32 = 16;

/// Which sides of a walkway tile meet the water, for its curb (`walk[mask]`).
pub const NORTH: u8 = 1;
pub const EAST: u8 = 2;
pub const SOUTH: u8 = 4;
pub const WEST: u8 = 8;

pub struct SewerArt {
    /// Walkway slabs, with a curb along the sides that meet the water (by side bits).
    pub walk: [TexId; 16],
    pub wall_side: TexId,
    pub wall_top: TexId,
    /// The channel's side below the walkways, slimy green at the waterline.
    pub bank: TexId,
    /// Murky green water, four frames.
    pub water: [TexId; 4],
    /// Bridge decks: weathered planks and riveted copper, boards running along z (0) or
    /// along x (1).
    pub wood: [TexId; 2],
    pub copper: [TexId; 2],
    /// The deck's edges, and its posts and rails.
    pub wood_beam: TexId,
    pub copper_beam: TexId,
    /// A drain pipe's mouth sticking out of a wall: the wall at z = 0, the pipe along +z.
    pub drain: Mesh,
    /// A round grate, lying flat on the floor.
    pub grate: Mesh,
}

fn speckle(t: &mut Texture, r: &mut Rng, c: u8, n: usize) {
    for _ in 0..n {
        let (x, y) = (r.range(0, T), r.range(0, T));
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

/// Walkway slabs laid like brickwork, worn pale at the edges, moss in the joints.
fn slabs(seed: u64) -> Texture {
    let mut t = Texture::new(16, 16, KHAKI);
    let mut r = Rng::new(seed);
    for y in 0..T {
        let off = if (y / 4) % 2 == 0 { 0 } else { 4 };
        for x in 0..T {
            let c = if y % 4 == 3 || (x + off) % 8 == 7 {
                SHADOW
            } else if y % 4 == 0 && (x + off) % 8 < 3 {
                SAND
            } else {
                KHAKI
            };
            t.set(x, y, c);
        }
    }
    speckle(&mut t, &mut r, ROSEWOOD, 9);
    // Moss creeping along the joints.
    for _ in 0..5 {
        let x = r.range(0, T);
        let y = r.range(0, 4) * 4 + 3;
        t.set_wrap(x, y, GREEN);
        t.set_wrap(x + 1, y, TEAL);
        if r.chance(0.5) {
            t.set_wrap(x, y - 1, LIME);
        }
    }
    t
}

/// Slabs with a curb of pale stones along the sides that meet the water.
fn curbed(base: &Texture, mask: u8) -> Texture {
    let mut t = base.clone();
    for i in 0..T {
        let joint = i % 5 == 4;
        for (bit, at) in [
            (NORTH, [(i, 0), (i, 1), (i, 2)]),
            (SOUTH, [(i, 15), (i, 14), (i, 13)]),
            (WEST, [(0, i), (1, i), (2, i)]),
            (EAST, [(15, i), (14, i), (13, i)]),
        ] {
            if mask & bit == 0 {
                continue;
            }
            let [edge, stone, lip] = at;
            t.set(edge.0, edge.1, if joint { SAND } else { CREAM });
            t.set(stone.0, stone.1, if joint { KHAKI } else { SAND });
            t.set(lip.0, lip.1, SHADOW);
        }
    }
    t
}

/// Brick courses in cool dark stone, moss dripping from the top, grime at the foot.
fn wall_side() -> Texture {
    let mut t = Texture::new(16, 16, SHADOW);
    let mut r = Rng::new(0x5E3E);
    for y in 0..T {
        let off = if (y / 4) % 2 == 0 { 0 } else { 4 };
        for x in 0..T {
            let brick = ((x + off) / 8, y / 4);
            let c = if y % 4 == 3 || (x + off) % 8 == 7 {
                INK
            } else {
                match (brick.0 * 7 + brick.1 * 3 + 1) % 5 {
                    0 => ROSEWOOD,
                    1 => KHAKI,
                    _ => SHADOW,
                }
            };
            t.set(x, y, c);
        }
    }
    // Moss hanging down from the top.
    for _ in 0..5 {
        let x = r.range(0, T);
        let len = r.range(1, 5);
        for y in 0..len {
            t.set(x, y, if y + 1 == len { TEAL } else { GREEN });
        }
    }
    for x in 0..T {
        if r.chance(0.5) {
            t.set(x, 15, DEEP_TEAL);
        }
    }
    t
}

/// The top of a wall: old bricks mostly under moss.
fn wall_top() -> Texture {
    let mut t = Texture::new(16, 16, GREEN);
    let mut r = Rng::new(0x70B);
    for y in 0..T {
        let off = if (y / 4) % 2 == 0 { 0 } else { 4 };
        for x in 0..T {
            if y % 4 == 3 || (x + off) % 8 == 7 {
                t.set(x, y, SHADOW);
            } else {
                t.set(x, y, KHAKI);
            }
        }
    }
    for _ in 0..7 {
        let (x, y) = (r.range(0, T), r.range(0, T));
        blob(&mut t, x, y, r.range(1, 3), 1, GREEN);
    }
    speckle(&mut t, &mut r, TEAL, 10);
    speckle(&mut t, &mut r, LIME, 8);
    t
}

/// The channel wall between walkway and water (only its bottom four rows show): a course
/// of brick, and green slime where the water laps.
fn bank() -> Texture {
    let mut t = Texture::new(16, 16, KHAKI);
    for x in 0..T {
        t.set(x, 12, if x % 8 == 7 { SHADOW } else { KHAKI });
        t.set(x, 13, if x % 3 == 0 { TEAL } else { GREEN });
        t.set(x, 14, if x % 4 == 1 { GREEN } else { TEAL });
        t.set(x, 15, DEEP_TEAL);
    }
    t
}

/// Murky green water: slow currents, floating scum and the odd bubble.
fn water(frame: i32) -> Texture {
    let mut t = Texture::new(16, 16, DEEP_TEAL);
    let mut r = Rng::new(0x5E0);
    for _ in 0..4 {
        let (x, y) = (r.range(0, T), r.range(0, T));
        blob(&mut t, x, y, 2, 1, TEAL);
    }
    let mut s = Rng::new(0x5E1);
    for _ in 0..5 {
        let (x, y) = (s.range(0, T), s.range(0, T));
        let dx = if (y / 4) % 2 == 0 { frame } else { -frame };
        for k in 0..s.range(2, 4) {
            t.set_wrap(x + k + dx, y, GREEN);
        }
    }
    // Scum drifting slowly one way.
    let mut g = Rng::new(0x5E2);
    for _ in 0..4 {
        let (x, y) = (g.range(0, T), g.range(0, T));
        t.set_wrap(x + frame / 2, y, LIME);
        t.set_wrap(x + frame / 2 + 1, y, GREEN);
    }
    // A bubble rising in its own spot each frame.
    let b = [(3, 11), (12, 4), (7, 13), (10, 8)][frame as usize % 4];
    t.set(b.0, b.1, MINT);
    t
}

/// Boards laid side by side along z (or x), with gaps, pale edges and a nail at each end.
fn planks(along_z: bool) -> Texture {
    let mut t = Texture::new(16, 16, CLAY);
    let mut r = Rng::new(0xB0A2);
    for a in 0..T {
        let board = a / 4;
        let col = a % 4;
        for b in 0..T {
            let c = match col {
                3 => INK,
                0 => GOLD,
                _ if board % 2 == 1 => RUST,
                _ => CLAY,
            };
            let (x, y) = if along_z { (a, b) } else { (b, a) };
            t.set(x, y, c);
        }
        // Nails near both ends.
        if col == 1 {
            for b in [1, 14] {
                let (x, y) = if along_z { (a, b) } else { (b, a) };
                t.set(x, y, SHADOW);
            }
        }
    }
    // Grain and weathering.
    for _ in 0..8 {
        let a = r.range(0, 4) * 4 + r.range(1, 3);
        let b = r.range(2, 13);
        for k in 0..r.range(2, 4) {
            let (x, y) = if along_z { (a, b + k) } else { (b + k, a) };
            t.set_wrap(x, y, MAROON);
        }
    }
    t
}

/// Copper plate gone green: verdigris over riveted plates, rust where the green has
/// worn through, and raised studs for grip.
fn copper(along_z: bool) -> Texture {
    let mut t = Texture::new(16, 16, TEAL);
    let mut r = Rng::new(0xC0FF);
    for y in 0..T {
        for x in 0..T {
            let (a, b) = if along_z { (x, y) } else { (y, x) };
            let c = if a % 8 == 7 || b == 15 {
                DEEP_TEAL
            } else if a % 8 == 0 || b == 0 {
                AQUA
            } else if a % 4 == 2 && b % 4 == 2 {
                MINT
            } else {
                TEAL
            };
            t.set(x, y, c);
        }
    }
    // Rivets at the plates' corners.
    for (a, b) in [
        (1, 1),
        (5, 1),
        (9, 1),
        (13, 1),
        (1, 13),
        (5, 13),
        (9, 13),
        (13, 13),
    ] {
        let (x, y) = if along_z { (a, b) } else { (b, a) };
        t.set(x, y, INK);
    }
    speckle(&mut t, &mut r, AQUA, 10);
    // Rust coming through.
    for _ in 0..3 {
        let (x, y) = (r.range(0, T), r.range(0, T));
        blob(&mut t, x, y, 1, 1, RUST);
        t.set_wrap(x, y, ORANGE);
    }
    t
}

/// A beam's grain (for edges, posts and rails).
fn beam(pal: [u8; 3], rust: bool) -> Texture {
    let [hi, mid, lo] = pal;
    let mut t = Texture::new(16, 16, mid);
    let mut r = Rng::new(0xBEA);
    for y in 0..T {
        for x in 0..T {
            let c = match y % 4 {
                0 => hi,
                3 => lo,
                _ => mid,
            };
            t.set(x, y, c);
        }
    }
    if rust {
        for _ in 0..4 {
            let (x, y) = (r.range(0, T), r.range(0, T));
            t.set_wrap(x, y, RUST);
            t.set_wrap(x + 1, y, ORANGE);
        }
    }
    t
}

/// A round grate: bars across a dark hole, in a stone ring.
fn grate_tex() -> Texture {
    let mut t = Texture::new(16, 16, CLEAR);
    for y in 0..T {
        for x in 0..T {
            let (dx, dy) = (x as f32 - 7.5, y as f32 - 7.5);
            let d = (dx * dx + dy * dy).sqrt();
            let c = if d > 7.6 {
                continue;
            } else if d > 6.2 {
                if (x + y) % 3 == 0 { SAND } else { KHAKI }
            } else if d > 5.2 {
                SLATE
            } else if x % 3 == 1 {
                SHADOW
            } else {
                INK
            };
            t.set(x, y, c);
        }
    }
    t
}

pub fn build(bank_: &mut TexBank) -> SewerArt {
    let base = slabs(0x5AB);
    let walk = std::array::from_fn(|m| bank_.add(curbed(&base, m as u8)));
    let wall_side = bank_.add(wall_side());
    let wall_top = bank_.add(wall_top());
    let bank = bank_.add(bank());
    let water = std::array::from_fn(|f| bank_.add(water(f as i32)));
    let wood = [bank_.add(planks(true)), bank_.add(planks(false))];
    let copper = [bank_.add(copper(true)), bank_.add(copper(false))];
    let wood_beam = bank_.add(beam([GOLD, CLAY, RUST], false));
    let copper_beam = bank_.add(beam([AQUA, TEAL, DEEP_TEAL], true));

    // The drain: a verdigris pipe with a lip at its mouth, dark inside.
    let pipe = bank_.add(beam([AQUA, TEAL, DEEP_TEAL], true));
    let hole = bank_.add(Texture::new(4, 4, INK));
    let mut part = Mesh::new();
    lathe(
        &mut part,
        Vec3::ZERO,
        &[
            (0.13, 0.0),
            (0.13, 0.2),
            (0.16, 0.21),
            (0.16, 0.27),
            (0.1, 0.275),
        ],
        10,
        0.0,
        pipe,
        false,
    );
    lathe(
        &mut part,
        Vec3::Y * 0.27,
        &[(0.1, 0.0), (0.0, 0.002)],
        10,
        0.0,
        hole,
        false,
    );
    let mut drain = Mesh::new();
    drain.append(&part, Mat4::from_rotation_x(FRAC_PI_2));

    let gt = bank_.add(grate_tex());
    let mut grate = Mesh::new();
    let s = 0.36;
    grate.quad(
        [
            Vec3::new(-s, 0.012, s),
            Vec3::new(s, 0.012, s),
            Vec3::new(s, 0.012, -s),
            Vec3::new(-s, 0.012, -s),
        ],
        UvRect::new(0.0, 0.0, 16.0, 16.0),
        gt,
    );

    SewerArt {
        walk,
        wall_side,
        wall_top,
        bank,
        water,
        wood,
        copper,
        wood_beam,
        copper_beam,
        drain,
        grate,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn curbs_run_along_the_water_sides_only() {
        let base = slabs(1);
        let north = curbed(&base, NORTH);
        assert_eq!(north.get(3, 0), CREAM);
        assert_eq!(north.get(3, 15), base.get(3, 15));
        let all = curbed(&base, NORTH | EAST | SOUTH | WEST);
        for (x, y) in [(3, 0), (15, 3), (3, 15), (0, 3)] {
            assert_eq!(all.get(x, y), CREAM, "({x}, {y})");
        }
    }

    #[test]
    fn decks_turn_with_the_bridge() {
        // Boards along z have their gaps in columns; along x, in rows.
        let z = planks(true);
        let x = planks(false);
        assert!((0..16).all(|y| z.get(3, y) == INK));
        assert!((0..16).all(|i| x.get(i, 3) == INK));
    }
}
