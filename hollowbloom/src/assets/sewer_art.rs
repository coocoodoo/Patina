//! The Hollow's old sewers: stone walkways either side of murky green channels, their curbs
//! along the water, mossy brick walls, and bridges across of weathered planks or of copper
//! gone green with age. Plus the pipes that drain into them and the grates in the floor.

use std::f32::consts::FRAC_PI_2;

use glam::{Mat4, Vec3};

use super::models::{lathe, skin_box};
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
    /// A barrel of staves and iron hoops, and one brimming over with green goo.
    pub barrel: Mesh,
    pub barrel_goo: Mesh,
    /// Broken planks, three ways.
    pub debris: [Mesh; 3],
    /// The sewer's slime: a brown swirl three coils high with a curl on top, and a face.
    pub swirl: Mesh,
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

/// Murky green water: slow currents, floating scum and the odd bubble. (It glows a little
/// too, see `SEWER_GLOW`, or down here in the dark it would only ever look teal.)
fn water(frame: i32) -> Texture {
    let mut t = Texture::new(16, 16, GREEN);
    let mut r = Rng::new(0x5E0);
    for _ in 0..4 {
        let (x, y) = (r.range(0, T), r.range(0, T));
        blob(&mut t, x, y, 2, 1, TEAL);
    }
    for _ in 0..2 {
        let (x, y) = (r.range(0, T), r.range(0, T));
        blob(&mut t, x, y, 1, 1, DEEP_TEAL);
    }
    // Slow currents, sliding one way on some rows and back the other on the rest.
    let mut s = Rng::new(0x5E1);
    for _ in 0..5 {
        let (x, y) = (s.range(0, T), s.range(0, T));
        let dx = if (y / 4) % 2 == 0 { frame } else { -frame };
        for k in 0..s.range(2, 4) {
            t.set_wrap(x + k + dx, y, LIME);
        }
    }
    // Scum drifting slowly one way.
    let mut g = Rng::new(0x5E2);
    for _ in 0..4 {
        let (x, y) = (g.range(0, T), g.range(0, T));
        t.set_wrap(x + frame / 2, y, CREAM);
        t.set_wrap(x + frame / 2 + 1, y, LIME);
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

/// The sewer slime's skin: over each coil (whose texture runs top to bottom a few texels
/// deep) a glossy highlight, then brown, then the dark underside.
fn swirl_skin() -> Texture {
    let mut t = Texture::new(16, 16, MAROON);
    for x in 0..T {
        t.set(x, 0, if x % 5 == 1 { GOLD } else { CLAY });
        t.set(x, 1, if x % 7 == 3 { CLAY } else { RUST });
        t.set(x, 2, if x % 4 == 0 { MAROON } else { RUST });
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

    // Barrels: staves round a bulge, two iron hoops, and a planked lid, or a lid's worth
    // of glowing goo brimming over and dripping down the side.
    let stave = bank_.add(super::tiles::boards([GOLD, CLAY, RUST], 0xBA2E));
    let hoop = bank_.add(Texture::new(4, 4, SHADOW));
    let lid = bank_.add(planks(false));
    let goo = bank_.add(beam([LIME, GREEN, TEAL], false));
    let mut barrel = Mesh::new();
    lathe(
        &mut barrel,
        Vec3::ZERO,
        &[
            (0.24, 0.0),
            (0.29, 0.16),
            (0.3, 0.32),
            (0.29, 0.48),
            (0.24, 0.64),
        ],
        10,
        0.0,
        stave,
        false,
    );
    for y in [0.09f32, 0.5] {
        lathe(
            &mut barrel,
            Vec3::ZERO,
            &[(0.296, y), (0.296, y + 0.05)],
            10,
            0.0,
            hoop,
            false,
        );
    }
    let mut barrel_goo = barrel.clone();
    lathe(
        &mut barrel,
        Vec3::ZERO,
        &[(0.24, 0.62), (0.0, 0.63)],
        10,
        0.0,
        lid,
        false,
    );
    lathe(
        &mut barrel_goo,
        Vec3::ZERO,
        &[(0.25, 0.62), (0.2, 0.68), (0.0, 0.7)],
        10,
        0.0,
        goo,
        false,
    );
    for (a, len) in [(0.3f32, 0.18f32), (2.4, 0.1), (4.2, 0.26)] {
        let (dx, dz) = (a.cos() * 0.25, a.sin() * 0.25);
        skin_box(
            &mut barrel_goo,
            Vec3::new(dx - 0.03, 0.63 - len, dz - 0.03),
            Vec3::new(dx + 0.03, 0.66, dz + 0.03),
            goo,
            &Texture::new(4, 4, 0),
        );
    }

    // Broken planks: a pair crossed, three scattered, one long and splintered.
    let boards = bank_.add(beam([GOLD, CLAY, RUST], false));
    let dark = bank_.add(Texture::new(4, 4, SHADOW));
    let w4 = Texture::new(4, 4, 0);
    let board = |m: &mut Mesh, len: f32, w: f32, at: Vec3, turn: f32, tex: TexId| {
        let mut b = Mesh::new();
        skin_box(
            &mut b,
            Vec3::new(-len * 0.5, 0.0, -w * 0.5),
            Vec3::new(len * 0.5, 0.045, w * 0.5),
            tex,
            &w4,
        );
        m.append(&b, Mat4::from_translation(at) * Mat4::from_rotation_y(turn));
    };
    let mut d0 = Mesh::new();
    board(&mut d0, 0.62, 0.14, Vec3::ZERO, 0.35, boards);
    board(
        &mut d0,
        0.4,
        0.12,
        Vec3::new(0.02, 0.045, 0.03),
        -0.95,
        boards,
    );
    let mut d1 = Mesh::new();
    board(
        &mut d1,
        0.34,
        0.13,
        Vec3::new(-0.15, 0.0, -0.12),
        0.2,
        boards,
    );
    board(&mut d1, 0.3, 0.12, Vec3::new(0.15, 0.0, 0.02), 1.3, boards);
    board(
        &mut d1,
        0.26,
        0.11,
        Vec3::new(-0.05, 0.0, 0.22),
        -0.4,
        boards,
    );
    let mut d2 = Mesh::new();
    board(&mut d2, 0.74, 0.15, Vec3::ZERO, -0.2, boards);
    // A splinter off the end, and a bent nail.
    board(&mut d2, 0.14, 0.06, Vec3::new(0.4, 0.0, -0.1), 0.7, boards);
    board(
        &mut d2,
        0.025,
        0.025,
        Vec3::new(-0.2, 0.045, 0.02),
        0.0,
        dark,
    );
    let debris = [d0, d1, d2];

    // The swirl: three coils, each smaller, and a curl on top.
    let skin = bank_.add(swirl_skin());
    let mut swirl = Mesh::new();
    for prof in [
        &[
            (0.0, 0.0),
            (0.26, 0.01),
            (0.32, 0.07),
            (0.3, 0.14),
            (0.2, 0.18),
            (0.0, 0.19),
        ][..],
        &[
            (0.0, 0.14),
            (0.22, 0.15),
            (0.25, 0.21),
            (0.22, 0.27),
            (0.12, 0.3),
            (0.0, 0.31),
        ],
        &[
            (0.0, 0.27),
            (0.14, 0.28),
            (0.16, 0.33),
            (0.12, 0.38),
            (0.0, 0.4),
        ],
    ] {
        lathe(&mut swirl, Vec3::ZERO, prof, 10, 0.3, skin, false);
    }
    let mut tip = Mesh::new();
    lathe(
        &mut tip,
        Vec3::ZERO,
        &[(0.07, 0.0), (0.045, 0.07), (0.0, 0.13)],
        8,
        0.0,
        skin,
        false,
    );
    swirl.append(
        &tip,
        Mat4::from_translation(Vec3::new(0.02, 0.36, -0.01)) * Mat4::from_rotation_z(-0.55),
    );
    // A face on the middle coil: big eyes, a grin and rosy cheeks.
    let solid = |b: &mut TexBank, c: u8| b.add(Texture::new(4, 4, c));
    let (white, ink, cheek) = (solid(bank_, WHITE), solid(bank_, INK), solid(bank_, SALMON));
    for sx in [-1.0f32, 1.0] {
        skin_box(
            &mut swirl,
            Vec3::new(sx * 0.075 - 0.045, 0.19, 0.2),
            Vec3::new(sx * 0.075 + 0.045, 0.28, 0.245),
            white,
            &w4,
        );
        skin_box(
            &mut swirl,
            Vec3::new(sx * 0.075 - 0.022, 0.2, 0.24),
            Vec3::new(sx * 0.075 + 0.022, 0.25, 0.252),
            ink,
            &w4,
        );
        skin_box(
            &mut swirl,
            Vec3::new(sx * 0.16 - 0.03, 0.14, 0.235),
            Vec3::new(sx * 0.16 + 0.03, 0.17, 0.26),
            cheek,
            &w4,
        );
    }
    skin_box(
        &mut swirl,
        Vec3::new(-0.06, 0.125, 0.285),
        Vec3::new(0.06, 0.145, 0.3),
        ink,
        &w4,
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
        barrel,
        barrel_goo,
        debris,
        swirl,
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
