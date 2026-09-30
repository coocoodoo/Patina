//! The Sunscorch Canyon's own creatures: the cactling, a little cactus that stands among
//! the cacti until you come close, and the sand cobra; and what they leave behind.

use std::collections::HashMap;

use glam::{Mat4, Quat, Vec2, Vec3};

use super::models::{lathe, skin_box};
use crate::palette::*;
use crate::render::{Mesh, TexBank, TexId, Texture};

/// A cactling, standing on the ground and facing +z. Its parts are drawn apart so it can
/// hide its face, wave its arms and waddle.
pub struct Cactling {
    /// Its trunk: ribbed, spiny and stout.
    pub trunk: Mesh,
    /// Glaring eyes under cross brows, and a round open mouth, on the front of the trunk.
    pub face: Mesh,
    /// An arm, from the shoulder out along +x and bending up (turn it round for the other).
    pub arm: Mesh,
    /// A stubby root of a foot.
    pub foot: Mesh,
    /// The flower on its head.
    pub flower: Mesh,
}

/// Where a cactling's arm joins its trunk, its feet stand, and its head tops out.
pub const CACTLING_SHOULDER: Vec3 = Vec3::new(0.17, 0.4, 0.0);
pub const CACTLING_FOOT: Vec3 = Vec3::new(0.1, 0.0, 0.12);
pub const CACTLING_TOP: f32 = 0.8;
/// Where its eyes are, for their glint.
pub const CACTLING_EYE: Vec3 = Vec3::new(0.07, 0.47, 0.185);

/// A sand cobra. Its body is swept along a curve each frame (see `models::tube`), in
/// `scales`; its head, jaw and hood are drawn on the end of it.
pub struct Cobra {
    /// Green scales down its back, and its pale belly down the middle of the texture.
    pub scales: TexId,
    /// Its head, pointing along +z from where the neck joins it.
    pub head: Mesh,
    /// Its lower jaw, hinged at the back of the head, and pink inside.
    pub jaw: Mesh,
    /// Its hood, spread flat and facing +z: banded pale in front, green behind with the
    /// spectacle mark.
    pub hood: Mesh,
    /// Its forked tongue, pointing along +z.
    pub tongue: Mesh,
}

/// Where a cobra's eyes are on its head, for their glow.
pub const COBRA_EYE: Vec3 = Vec3::new(0.05, 0.03, 0.085);

pub struct DesertArt {
    pub cactling: Cactling,
    pub cobra: Cobra,
}

fn v(x: f32, y: f32, z: f32) -> Vec3 {
    Vec3::new(x, y, z)
}

/// A flat colour.
fn flat(bank: &mut TexBank, c: u8) -> TexId {
    bank.add(Texture::new(4, 4, c))
}

/// A little box of flat colour.
fn bx(m: &mut Mesh, min: Vec3, max: Vec3, tex: TexId) {
    skin_box(m, min, max, tex, &Texture::new(4, 4, 0));
}

/// A cone from `at` along `dir`.
fn cone(m: &mut Mesh, at: Vec3, dir: Vec3, len: f32, r: f32, tex: TexId) {
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

/// A block from its eight corners, numbered by bits: 1 for +x, 2 for +y, 4 for +z.
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
        m.quad(p, crate::render::UvRect::new(0.0, 0.0, w, h), tex);
    }
}

/// A block narrowing along z: `back` (half-width, bottom, top) at `z0` to `front` at `z1`.
fn wedge(m: &mut Mesh, (z0, back): (f32, [f32; 3]), (z1, front): (f32, [f32; 3]), tex: TexId) {
    let c = std::array::from_fn(|i| {
        let ([hx, y0, y1], z) = if i & 4 != 0 { (front, z1) } else { (back, z0) };
        v(
            if i & 1 != 0 { hx } else { -hx },
            if i & 2 != 0 { y1 } else { y0 },
            z,
        )
    });
    block(m, c, tex);
}

/// An oval, `r` its radii, flat in the xy plane: its front (+z) and back each laid with a
/// square of the texture.
fn oval(m: &mut Mesh, r: Vec2, tex: TexId, front: (f32, f32), back: (f32, f32)) {
    const SEG: usize = 14;
    let rim: Vec<Vec2> = (0..SEG)
        .map(|k| {
            let a = k as f32 / SEG as f32 * std::f32::consts::TAU;
            Vec2::new(a.cos(), a.sin())
        })
        .collect();
    // Planar texture: the oval's box onto a 16-texel square at `at`.
    let uv = |p: Vec2, (u, w): (f32, f32), flip: f32| {
        Vec2::new(
            u + (p.x * flip * 0.5 + 0.5) * 15.0 + 0.5,
            w + (0.5 - p.y * 0.5) * 15.0 + 0.5,
        )
    };
    let c = Vec2::ZERO;
    for k in 0..SEG {
        let (a, b) = (rim[k], rim[(k + 1) % SEG]);
        let (pa, pb) = (
            v(a.x * r.x, a.y * r.y, 0.004),
            v(b.x * r.x, b.y * r.y, 0.004),
        );
        m.tri(
            [v(0.0, 0.0, 0.004), pa, pb],
            [uv(c, front, 1.0), uv(a, front, 1.0), uv(b, front, 1.0)],
            tex,
        );
        let (qa, qb) = (pa - v(0.0, 0.0, 0.008), pb - v(0.0, 0.0, 0.008));
        m.tri(
            [v(0.0, 0.0, -0.004), qb, qa],
            [uv(c, back, -1.0), uv(b, back, -1.0), uv(a, back, -1.0)],
            tex,
        );
    }
}

/// A cobra's scales, laid along its length: olive-green down the back in a lattice of
/// darker scales, and down the middle of the texture its pale, barred belly (which a tube
/// lays opposite its `up`).
fn scales() -> Texture {
    let mut t = Texture::new(16, 16, GREEN);
    for y in 0..16 {
        for x in 0..16 {
            let belly = (6..=10).contains(&x);
            let c = if belly {
                if y % 3 == 0 { KHAKI } else { SAND }
            } else if x == 5 || x == 11 {
                TEAL
            } else if (x + y) % 4 == 0 || (x + 16 - y) % 4 == 0 {
                DEEP_TEAL
            } else if (x * 7 + y * 3) % 11 == 0 {
                LIME
            } else {
                GREEN
            };
            t.set(x, y, c);
        }
    }
    // Dark bands across the back every so often.
    for y in [4, 12] {
        for x in (0..16).filter(|x| !(5..=11).contains(x)) {
            t.set(x, y, DEEP_TEAL);
        }
    }
    t
}

/// A cobra's hood: in front (the left square) pale scales barred with rust; behind (the
/// right square) green, with the pale spectacle mark in the middle.
fn hood_tex() -> Texture {
    let mut t = Texture::new(32, 16, SAND);
    for y in 0..16 {
        for x in 0..16 {
            let c = match y {
                5 | 10 => RUST,
                13 => MAROON,
                _ if (x + y) % 5 == 0 => KHAKI,
                _ if y < 4 => CREAM,
                _ => SAND,
            };
            t.set(x, y, c);
            let back = if (x + y) % 4 == 0 { DEEP_TEAL } else { GREEN };
            t.set(16 + x, y, back);
        }
    }
    // The spectacles: two pale rings joined by a curve, dark in their middles.
    for (cx, cy) in [(21, 6), (26, 6)] {
        for (dx, dy) in [
            (-1, -1),
            (0, -1),
            (1, -1),
            (-1, 0),
            (1, 0),
            (-1, 1),
            (0, 1),
            (1, 1),
        ] {
            t.set(cx + dx, cy + dy, CREAM);
        }
        t.set(cx, cy, INK);
    }
    for x in 22..=25 {
        t.set(x, 9, CREAM);
    }
    t
}

fn cactling(bank: &mut TexBank, skin: TexId) -> Cactling {
    let ink = flat(bank, INK);
    let red = flat(bank, RED);
    let brow = flat(bank, DEEP_TEAL);
    let mut trunk = Mesh::new();
    lathe(
        &mut trunk,
        Vec3::ZERO,
        &[
            (0.17, 0.0),
            (0.19, 0.1),
            (0.19, 0.5),
            (0.17, 0.66),
            (0.11, 0.77),
            (0.0, CACTLING_TOP),
        ],
        9,
        // A face of it square to the front, for the face to sit on.
        0.1745,
        skin,
        false,
    );
    // Its face: glaring eyes with a red spark in each under cross brows, and a round mouth
    // open in a silent yell.
    let mut face = Mesh::new();
    for sx in [-1.0f32, 1.0] {
        let (x, y, z) = (sx * CACTLING_EYE.x, CACTLING_EYE.y, CACTLING_EYE.z);
        bx(
            &mut face,
            v(x - 0.042, y - 0.04, z - 0.03),
            v(x + 0.042, y + 0.035, z + 0.012),
            ink,
        );
        bx(
            &mut face,
            v(x - 0.014, y - 0.02, z + 0.01),
            v(x + 0.014, y + 0.008, z + 0.017),
            red,
        );
        // The brow, falling towards the nose.
        let c = std::array::from_fn(|i| {
            let inner = (i & 1 != 0) != (sx > 0.0);
            let (bx_, lift) = if inner {
                (sx * 0.015, 0.0)
            } else {
                (sx * 0.12, 0.04)
            };
            v(
                bx_,
                y + 0.035 + lift + if i & 2 != 0 { 0.022 } else { 0.0 },
                z + if i & 4 != 0 { 0.012 } else { -0.04 },
            )
        });
        block(&mut face, c, brow);
    }
    let z = 0.179;
    bx(
        &mut face,
        v(-0.042, 0.27, z - 0.02),
        v(0.042, 0.36, z + 0.012),
        ink,
    );
    bx(
        &mut face,
        v(-0.024, 0.28, z + 0.008),
        v(0.024, 0.3, z + 0.015),
        red,
    );
    // An arm: out from the shoulder, then up, rounded at its tip.
    let mut arm = Mesh::new();
    let mut out = Mesh::new();
    lathe(
        &mut out,
        Vec3::ZERO,
        &[(0.075, 0.0), (0.075, 0.16)],
        7,
        0.0,
        skin,
        false,
    );
    arm.append(&out, Mat4::from_rotation_z(-std::f32::consts::FRAC_PI_2));
    lathe(
        &mut arm,
        v(0.15, -0.02, 0.0),
        &[(0.075, 0.0), (0.072, 0.18), (0.05, 0.24), (0.0, 0.265)],
        7,
        0.0,
        skin,
        false,
    );
    let root = flat(bank, ROSEWOOD);
    let mut foot = Mesh::new();
    lathe(
        &mut foot,
        Vec3::ZERO,
        &[(0.06, 0.0), (0.075, 0.03), (0.06, 0.065), (0.0, 0.075)],
        6,
        0.0,
        root,
        false,
    );
    cone(
        &mut foot,
        v(0.0, 0.015, 0.05),
        v(0.2, -0.1, 1.0),
        0.06,
        0.025,
        root,
    );
    let pink = flat(bank, PINK);
    let gold = flat(bank, GOLD);
    let mut flower = Mesh::new();
    for k in 0..5 {
        let a = k as f32 / 5.0 * std::f32::consts::TAU;
        cone(
            &mut flower,
            v(0.0, 0.0, 0.0),
            v(a.cos(), 0.7, a.sin()),
            0.08,
            0.035,
            pink,
        );
    }
    bx(&mut flower, v(-0.02, 0.0, -0.02), v(0.02, 0.04, 0.02), gold);
    Cactling {
        trunk,
        face,
        arm,
        foot,
        flower,
    }
}

fn cobra(bank: &mut TexBank) -> Cobra {
    let skin = scales();
    let scales = bank.add(skin);
    let belly = flat(bank, SAND);
    let ink = flat(bank, INK);
    let red = flat(bank, RED);
    let pink = flat(bank, SALMON);
    let white = flat(bank, WHITE);
    let dark = flat(bank, DEEP_TEAL);
    let mut head = Mesh::new();
    wedge(
        &mut head,
        (0.0, [0.068, -0.02, 0.045]),
        (0.15, [0.04, -0.02, 0.02]),
        scales,
    );
    // Heavy brows over its eyes, and the eyes, red and burning.
    for sx in [-1.0f32, 1.0] {
        let e = v(sx * COBRA_EYE.x, COBRA_EYE.y, COBRA_EYE.z);
        bx(
            &mut head,
            e - v(0.014, 0.012, 0.018),
            e + v(0.014, 0.012, 0.018),
            red,
        );
        let c = std::array::from_fn(|i| {
            let outer = (i & 1 != 0) == (sx > 0.0);
            let x = if outer { sx * 0.066 } else { sx * 0.02 };
            let lift = if outer { 0.012 } else { 0.0 };
            v(
                x,
                0.038 + lift + if i & 2 != 0 { 0.016 } else { 0.0 },
                if i & 4 != 0 { 0.11 } else { 0.05 },
            )
        });
        block(&mut head, c, dark);
        // Nostrils.
        bx(
            &mut head,
            v(sx * 0.018 - 0.006, 0.012, 0.146),
            v(sx * 0.018 + 0.006, 0.02, 0.153),
            ink,
        );
        // Fangs, folded up inside its mouth until it strikes.
        cone(
            &mut head,
            v(sx * 0.022, -0.018, 0.12),
            v(0.0, -1.0, 0.25),
            0.026,
            0.008,
            white,
        );
    }
    let mut jaw = Mesh::new();
    wedge(
        &mut jaw,
        (0.0, [0.058, -0.05, -0.025]),
        (0.14, [0.034, -0.044, -0.025]),
        belly,
    );
    wedge(
        &mut jaw,
        (0.01, [0.046, -0.025, -0.021]),
        (0.13, [0.026, -0.025, -0.021]),
        pink,
    );
    let hood_t = bank.add(hood_tex());
    let mut hood = Mesh::new();
    oval(
        &mut hood,
        Vec2::new(0.17, 0.23),
        hood_t,
        (0.0, 0.0),
        (16.0, 0.0),
    );
    let mut tongue = Mesh::new();
    bx(
        &mut tongue,
        v(-0.006, -0.004, 0.0),
        v(0.006, 0.004, 0.07),
        red,
    );
    for sx in [-1.0f32, 1.0] {
        cone(
            &mut tongue,
            v(0.0, 0.0, 0.066),
            v(sx * 0.6, 0.0, 1.0),
            0.035,
            0.006,
            red,
        );
    }
    Cobra {
        scales,
        head,
        jaw,
        hood,
        tongue,
    }
}

pub fn build(bank: &mut TexBank, cactus_skin: TexId) -> DesertArt {
    DesertArt {
        cactling: cactling(bank, cactus_skin),
        cobra: cobra(bank),
    }
}

/// A cactling's needle: long, pale and sharp.
const NEEDLE: &[&str] = &[
    "................",
    "...........KK...",
    "..........KwK...",
    ".........KwyK...",
    "........KwyK....",
    ".......KwyK.....",
    "......KwyK......",
    ".....KynK.......",
    "....KynK........",
    "...KnhK.........",
    "..KghK..........",
    ".KggK...........",
    ".KgtK...........",
    "..KK............",
    "................",
    "................",
];

/// A cobra's shed skin, curled round on itself: pale and patterned, the hood at its end.
const SKIN: &[&str] = &[
    "................",
    ".....KKKKK......",
    "...KKnhnhnKK....",
    "..KnhKKKKKhnK...",
    ".KnhK.....KhnK..",
    ".KhK..KKK..KhK..",
    ".KnK.KnhnK.KnK..",
    ".KhK.KhKhK.KhK..",
    ".KnK..KKnK.KnK..",
    ".KhnK..KhK.KhK..",
    "..KhnKKnhK.KnK..",
    "...KKhnhKKKhnK..",
    "....KKKKnhnhK...",
    "........KKKK....",
    "................",
    "................",
];

/// Icons for what the canyon's creatures leave behind.
pub fn icons(bank: &mut TexBank, m: &mut HashMap<&'static str, TexId>) {
    use super::sprites::art;
    m.insert("cactling_needle", bank.add(art(NEEDLE, [CLEAR; 4])));
    m.insert("cobra_skin", bank.add(art(SKIN, [CLEAR; 4])));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_cobras_belly_runs_down_the_middle() {
        let t = scales();
        for y in 0..16 {
            assert!(matches!(t.get(8, y), SAND | KHAKI), "row {y}");
            assert!(!matches!(t.get(1, y), SAND | KHAKI), "row {y}");
        }
    }

    #[test]
    fn the_icons_are_tidy() {
        for rows in [NEEDLE, SKIN] {
            assert_eq!(rows.len(), 16);
            for r in rows {
                assert_eq!(r.chars().count(), 16, "{r}");
            }
        }
    }
}
