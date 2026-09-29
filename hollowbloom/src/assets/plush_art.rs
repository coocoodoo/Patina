//! Monster backpacks: a plush pack in the shape of every creature of the Hollow, its face
//! looking back at whoever's behind you. Made like the cloth backpacks (see `pack_art`), in
//! the space of the hero's body with its back at `BACK`.

use std::f32::consts::{PI, TAU};

use glam::{Mat4, Quat, Vec2, Vec3};

use super::models::{lathe, skin_box};
use super::pack_art::{BACK, Kit};
use super::sprites::art;
use crate::palette::*;
use crate::render::{Mesh, TexBank, Texture};

/// How many monster backpacks there are, in the order of `ICONS` (and of the items).
pub const PLUSHES: usize = 24;

/// Their icons' names, in order.
pub const ICONS: [&str; PLUSHES] = [
    "pack_slime",
    "pack_bat",
    "pack_shroom",
    "pack_crab",
    "pack_wisp",
    "pack_beetle",
    "pack_imp",
    "pack_skeleton",
    "pack_golem",
    "pack_ghost",
    "pack_frog",
    "pack_jelly",
    "pack_puffer",
    "pack_zombie",
    "pack_brute",
    "pack_sneak",
    "pack_bug",
    "pack_snail",
    "pack_bookworm",
    "pack_drake",
    "pack_leafling",
    "pack_werewolf",
    "pack_minotaur",
    "pack_griffin",
];

fn v(x: f32, y: f32, z: f32) -> Vec3 {
    Vec3::new(x, y, z)
}

/// An egg of a ball: `at` its middle, `r` its radii.
fn ball(k: &mut Kit, bank: &mut TexBank, m: &mut Mesh, at: Vec3, r: Vec3, c: u8) {
    let t = k.c(bank, c);
    let mut s = Mesh::new();
    lathe(
        &mut s,
        Vec3::ZERO,
        &[
            (0.0, -1.0),
            (0.5, -0.87),
            (0.87, -0.5),
            (1.0, 0.0),
            (0.87, 0.5),
            (0.5, 0.87),
            (0.0, 1.0),
        ],
        8,
        0.0,
        t,
        false,
    );
    m.append(&s, Mat4::from_translation(at) * Mat4::from_scale(r));
}

/// How far back a ball's face (its -z side) is at (x, y).
fn face(at: Vec3, r: Vec3, x: f32, y: f32) -> f32 {
    let (dx, dy) = ((x - at.x) / r.x, (y - at.y) / r.y);
    at.z - r.z * (1.0 - dx * dx - dy * dy).max(0.0).sqrt()
}

/// A flat dab of colour on a ball's face, centred on (x, y), standing `out` proud of it.
#[allow(clippy::too_many_arguments)]
fn dab(
    k: &mut Kit,
    bank: &mut TexBank,
    m: &mut Mesh,
    ball: (Vec3, Vec3),
    (x, y): (f32, f32),
    (w, h): (f32, f32),
    out: f32,
    c: u8,
) {
    let z = face(ball.0, ball.1, x, y) - out;
    k.bx(
        bank,
        m,
        v(x - w * 0.5, y - h * 0.5, z - 0.008),
        v(x + w * 0.5, y + h * 0.5, z + 0.02),
        c,
    );
}

/// Plush eyes on a ball's face: big and black, a white glint in each.
fn eyes(
    k: &mut Kit,
    bank: &mut TexBank,
    m: &mut Mesh,
    b: (Vec3, Vec3),
    y: f32,
    gap: f32,
    size: f32,
) {
    for sx in [-1.0f32, 1.0] {
        let x = b.0.x + sx * gap;
        dab(k, bank, m, b, (x, y), (size, size * 1.2), 0.0, INK);
        dab(
            k,
            bank,
            m,
            b,
            (x - size * 0.2, y + size * 0.25),
            (size * 0.36, size * 0.36),
            0.006,
            WHITE,
        );
    }
}

/// A cone from `base` along `dir`.
#[allow(clippy::too_many_arguments)]
fn cone(
    k: &mut Kit,
    bank: &mut TexBank,
    m: &mut Mesh,
    base: Vec3,
    dir: Vec3,
    len: f32,
    r: f32,
    c: u8,
) {
    let t = k.c(bank, c);
    let mut p = Mesh::new();
    lathe(
        &mut p,
        Vec3::ZERO,
        &[(r, 0.0), (0.0, len)],
        5,
        0.3,
        t,
        false,
    );
    let rot = Quat::from_rotation_arc(Vec3::Y, dir.normalize());
    m.append(&p, Mat4::from_translation(base) * Mat4::from_quat(rot));
}

/// A square stick `w` thick from `a` to `b`.
fn stick(k: &mut Kit, bank: &mut TexBank, m: &mut Mesh, a: Vec3, b: Vec3, w: f32, c: u8) {
    let t = k.c(bank, c);
    let mut s = Mesh::new();
    let len = (b - a).length();
    skin_box(&mut s, v(-w, 0.0, -w), v(w, len, w), t, &k.w4);
    let rot = Quat::from_rotation_arc(Vec3::Y, (b - a).normalize());
    m.append(&s, Mat4::from_translation(a) * Mat4::from_quat(rot));
}

/// A flat fan of skin (a wing, a leaf, a fin) round its first point, seen from both sides.
fn fan(k: &mut Kit, bank: &mut TexBank, m: &mut Mesh, pts: &[Vec3], c: u8) {
    let t = k.c(bank, c);
    let hub = pts[0];
    let uv = [Vec2::ZERO, Vec2::new(3.0, 0.0), Vec2::new(0.0, 3.0)];
    for w in pts[1..].windows(2) {
        m.tri([hub, w[0], w[1]], uv, t);
        m.tri([hub, w[1], w[0]], uv, t);
    }
}

/// A leaf from `base` to `tip`, `w` wide, lying in the plane facing -z, with a pale rib.
fn leaf(k: &mut Kit, bank: &mut TexBank, m: &mut Mesh, base: Vec3, tip: Vec3, w: f32) {
    let d = tip - base;
    let side = v(-d.y, d.x, 0.0).normalize() * w * 0.5;
    let mid = base + d * 0.45;
    fan(
        k,
        bank,
        m,
        &[mid, base, mid + side, tip, mid - side, base],
        GREEN,
    );
    stick(k, bank, m, base, base + d * 0.85, 0.006, LIME);
}

/// A bat's (or an imp's, or a drakeling's) wing out to the side `sx`, `s` big: skin between
/// finger bones, the edge scalloped between them.
#[allow(clippy::too_many_arguments)]
fn wing(
    k: &mut Kit,
    bank: &mut TexBank,
    m: &mut Mesh,
    at: Vec3,
    sx: f32,
    s: f32,
    skin: u8,
    bone: u8,
) {
    let p = |x: f32, y: f32| at + v(sx * x * s, y * s, 0.0);
    fan(
        k,
        bank,
        m,
        &[
            p(0.0, 0.0),
            p(0.24, 0.1),
            p(0.2, 0.0),
            p(0.27, -0.07),
            p(0.16, -0.09),
            p(0.14, -0.16),
            p(0.02, -0.08),
        ],
        skin,
    );
    for tip in [p(0.24, 0.1), p(0.27, -0.07), p(0.14, -0.16)] {
        stick(k, bank, m, p(0.03, 0.01), tip, 0.008, bone);
    }
}

fn slime(k: &mut Kit, bank: &mut TexBank) -> Mesh {
    let mut m = Mesh::new();
    let b = (v(0.0, 0.12, -0.21), v(0.15, 0.125, 0.11));
    ball(k, bank, &mut m, b.0, b.1, GREEN);
    // A shine up one side, and a bubble inside.
    ball(
        k,
        bank,
        &mut m,
        v(-0.07, 0.19, -0.265),
        v(0.032, 0.022, 0.02),
        LIME,
    );
    ball(
        k,
        bank,
        &mut m,
        v(0.085, 0.06, -0.29),
        v(0.013, 0.013, 0.013),
        MINT,
    );
    eyes(k, bank, &mut m, b, 0.14, 0.05, 0.032);
    dab(k, bank, &mut m, b, (0.0, 0.09), (0.03, 0.011), 0.0, INK);
    for sx in [-1.0f32, 1.0] {
        dab(
            k,
            bank,
            &mut m,
            b,
            (sx * 0.09, 0.1),
            (0.028, 0.012),
            0.0,
            PINK,
        );
    }
    m
}

fn bat(k: &mut Kit, bank: &mut TexBank) -> Mesh {
    let mut m = Mesh::new();
    let b = (v(0.0, 0.14, -0.2), v(0.105, 0.11, 0.085));
    ball(k, bank, &mut m, b.0, b.1, PURPLE);
    for sx in [-1.0f32, 1.0] {
        cone(
            k,
            bank,
            &mut m,
            v(sx * 0.05, 0.22, -0.2),
            v(sx * 0.35, 1.0, 0.0),
            0.09,
            0.035,
            PURPLE,
        );
        cone(
            k,
            bank,
            &mut m,
            v(sx * 0.05, 0.225, -0.232),
            v(sx * 0.35, 1.0, 0.0),
            0.06,
            0.018,
            LAVENDER,
        );
        wing(
            k,
            bank,
            &mut m,
            v(sx * 0.08, 0.16, -0.2),
            sx,
            1.1,
            GRAPE,
            INK,
        );
    }
    dab(k, bank, &mut m, b, (0.0, 0.08), (0.08, 0.05), 0.0, LAVENDER);
    eyes(k, bank, &mut m, b, 0.16, 0.042, 0.026);
    for sx in [-1.0f32, 1.0] {
        dab(
            k,
            bank,
            &mut m,
            b,
            (sx * 0.016, 0.112),
            (0.01, 0.02),
            0.004,
            WHITE,
        );
    }
    m
}

fn shroom(k: &mut Kit, bank: &mut TexBank) -> Mesh {
    let mut m = Mesh::new();
    let stem = k.c(bank, SAND);
    let at = v(0.0, 0.0, -0.21);
    lathe(
        &mut m,
        at,
        &[(0.085, 0.0), (0.095, 0.08), (0.085, 0.16)],
        8,
        0.3,
        stem,
        true,
    );
    // A big spotted cap.
    let cap = k.c(bank, RED);
    let top = v(0.0, 0.14, -0.23);
    lathe(
        &mut m,
        top,
        &[
            (0.17, 0.0),
            (0.175, 0.025),
            (0.155, 0.07),
            (0.105, 0.11),
            (0.045, 0.13),
            (0.0, 0.135),
        ],
        10,
        0.3,
        cap,
        true,
    );
    for (a, rad, h) in [
        (0.0f32, 0.13f32, 0.07f32),
        (1.2, 0.14, 0.055),
        (-1.2, 0.14, 0.055),
        (0.6, 0.07, 0.115),
        (-0.7, 0.075, 0.11),
        (2.2, 0.12, 0.08),
        (-2.3, 0.12, 0.08),
    ] {
        let at = top + v(a.sin() * rad, h, -a.cos() * rad);
        ball(k, bank, &mut m, at, v(0.024, 0.014, 0.024), WHITE);
    }
    // A sleepy little face on the stem.
    let z = -0.21 - 0.094;
    for sx in [-1.0f32, 1.0] {
        k.bx(
            bank,
            &mut m,
            v(sx * 0.035 - 0.011, 0.08, z - 0.01),
            v(sx * 0.035 + 0.011, 0.11, z + 0.01),
            INK,
        );
        k.bx(
            bank,
            &mut m,
            v(sx * 0.062 - 0.012, 0.065, z - 0.008),
            v(sx * 0.062 + 0.012, 0.075, z + 0.01),
            PINK,
        );
    }
    k.bx(
        bank,
        &mut m,
        v(-0.012, 0.055, z - 0.01),
        v(0.012, 0.062, z + 0.01),
        INK,
    );
    m
}

fn crab(k: &mut Kit, bank: &mut TexBank) -> Mesh {
    let mut m = Mesh::new();
    let b = (v(0.0, 0.11, -0.21), v(0.15, 0.09, 0.1));
    ball(
        k,
        bank,
        &mut m,
        v(0.0, 0.08, -0.21),
        v(0.155, 0.055, 0.105),
        TEAL,
    );
    ball(k, bank, &mut m, b.0, b.1, AQUA);
    // Crystals growing out of its shell.
    cone(
        k,
        bank,
        &mut m,
        v(0.0, 0.17, -0.24),
        v(0.0, 1.0, -0.4),
        0.1,
        0.032,
        SKY,
    );
    cone(
        k,
        bank,
        &mut m,
        v(-0.07, 0.16, -0.23),
        v(-0.3, 1.0, -0.3),
        0.075,
        0.025,
        WHITE,
    );
    cone(
        k,
        bank,
        &mut m,
        v(0.07, 0.16, -0.22),
        v(0.3, 1.0, -0.3),
        0.075,
        0.025,
        MINT,
    );
    for sx in [-1.0f32, 1.0] {
        // Claws out at the sides...
        stick(
            k,
            bank,
            &mut m,
            v(sx * 0.13, 0.08, -0.22),
            v(sx * 0.22, 0.13, -0.26),
            0.02,
            AQUA,
        );
        ball(
            k,
            bank,
            &mut m,
            v(sx * 0.25, 0.15, -0.27),
            v(0.05, 0.04, 0.035),
            AQUA,
        );
        cone(
            k,
            bank,
            &mut m,
            v(sx * 0.27, 0.18, -0.27),
            v(sx * 0.2, 1.0, 0.0),
            0.05,
            0.022,
            AQUA,
        );
        cone(
            k,
            bank,
            &mut m,
            v(sx * 0.21, 0.18, -0.27),
            v(-sx * 0.1, 1.0, 0.0),
            0.04,
            0.018,
            TEAL,
        );
        // ...and eyes on stalks.
        stick(
            k,
            bank,
            &mut m,
            v(sx * 0.045, 0.16, -0.28),
            v(sx * 0.06, 0.25, -0.29),
            0.01,
            TEAL,
        );
        ball(
            k,
            bank,
            &mut m,
            v(sx * 0.06, 0.26, -0.29),
            v(0.024, 0.024, 0.024),
            WHITE,
        );
        k.bx(
            bank,
            &mut m,
            v(sx * 0.06 - 0.009, 0.25, -0.319),
            v(sx * 0.06 + 0.009, 0.27, -0.305),
            INK,
        );
    }
    dab(k, bank, &mut m, b, (0.0, 0.1), (0.05, 0.012), 0.0, INK);
    m
}

fn wisp(k: &mut Kit, bank: &mut TexBank) -> Mesh {
    let mut m = Mesh::new();
    let flame = k.c(bank, MINT);
    lathe(
        &mut m,
        v(0.0, 0.0, -0.21),
        &[
            (0.0, 0.0),
            (0.09, 0.03),
            (0.13, 0.09),
            (0.12, 0.16),
            (0.08, 0.22),
            (0.035, 0.28),
            (0.0, 0.33),
        ],
        8,
        0.3,
        flame,
        false,
    );
    for sx in [-1.0f32, 1.0] {
        cone(
            k,
            bank,
            &mut m,
            v(sx * 0.07, 0.17, -0.21),
            v(sx * 0.5, 1.0, 0.0),
            0.13,
            0.04,
            MINT,
        );
    }
    cone(
        k,
        bank,
        &mut m,
        v(0.02, 0.25, -0.24),
        v(0.2, 1.0, -0.2),
        0.11,
        0.03,
        AQUA,
    );
    // A bright heart, and a face in it.
    let b = (v(0.0, 0.12, -0.3), v(0.08, 0.075, 0.045));
    ball(k, bank, &mut m, b.0, b.1, WHITE);
    for sx in [-1.0f32, 1.0] {
        dab(
            k,
            bank,
            &mut m,
            b,
            (sx * 0.03, 0.135),
            (0.018, 0.035),
            0.0,
            INK,
        );
    }
    dab(k, bank, &mut m, b, (0.0, 0.09), (0.03, 0.012), 0.0, TEAL);
    m
}

fn beetle(k: &mut Kit, bank: &mut TexBank) -> Mesh {
    let mut m = Mesh::new();
    for sx in [-1.0f32, 1.0] {
        let b = (v(sx * 0.062, 0.12, -0.21), v(0.075, 0.13, 0.095));
        ball(k, bank, &mut m, b.0, b.1, LAVENDER);
        for (x, y) in [(0.03f32, 0.17f32), (0.07, 0.08), (0.02, 0.03)] {
            dab(
                k,
                bank,
                &mut m,
                b,
                (sx * x + b.0.x * 0.4, y),
                (0.03, 0.03),
                0.0,
                PURPLE,
            );
        }
    }
    k.bx(
        bank,
        &mut m,
        v(-0.006, 0.0, -0.275),
        v(0.006, 0.245, -0.255),
        INK,
    );
    // Its head and feelers on top.
    let h = (v(0.0, 0.255, -0.21), v(0.07, 0.045, 0.062));
    ball(k, bank, &mut m, h.0, h.1, GRAPE);
    for sx in [-1.0f32, 1.0] {
        dab(
            k,
            bank,
            &mut m,
            h,
            (sx * 0.028, 0.265),
            (0.015, 0.015),
            0.0,
            WHITE,
        );
        stick(
            k,
            bank,
            &mut m,
            v(sx * 0.025, 0.285, -0.23),
            v(sx * 0.085, 0.37, -0.25),
            0.007,
            INK,
        );
        ball(
            k,
            bank,
            &mut m,
            v(sx * 0.085, 0.37, -0.25),
            v(0.014, 0.014, 0.014),
            INK,
        );
    }
    m
}

fn imp(k: &mut Kit, bank: &mut TexBank) -> Mesh {
    let mut m = Mesh::new();
    let b = (v(0.0, 0.14, -0.2), v(0.13, 0.12, 0.095));
    ball(k, bank, &mut m, b.0, b.1, RED);
    for sx in [-1.0f32, 1.0] {
        cone(
            k,
            bank,
            &mut m,
            v(sx * 0.06, 0.24, -0.21),
            v(sx * 0.35, 1.0, -0.1),
            0.11,
            0.032,
            INK,
        );
        cone(
            k,
            bank,
            &mut m,
            v(sx * 0.12, 0.15, -0.2),
            v(sx, 0.35, 0.0),
            0.08,
            0.03,
            RED,
        );
        wing(
            k,
            bank,
            &mut m,
            v(sx * 0.09, 0.19, -0.18),
            sx,
            0.7,
            MAROON,
            INK,
        );
        dab(
            k,
            bank,
            &mut m,
            b,
            (sx * 0.05, 0.165),
            (0.042, 0.036),
            0.0,
            GOLD,
        );
        dab(
            k,
            bank,
            &mut m,
            b,
            (sx * 0.05, 0.165),
            (0.012, 0.03),
            0.004,
            INK,
        );
        dab(
            k,
            bank,
            &mut m,
            b,
            (sx * 0.03, 0.096),
            (0.014, 0.02),
            0.005,
            WHITE,
        );
    }
    dab(k, bank, &mut m, b, (0.0, 0.1), (0.1, 0.024), 0.0, INK);
    // A tail with a spade on the end.
    stick(
        k,
        bank,
        &mut m,
        v(0.05, 0.04, -0.26),
        v(0.15, -0.04, -0.27),
        0.012,
        RED,
    );
    fan(
        k,
        bank,
        &mut m,
        &[
            v(0.15, -0.04, -0.27),
            v(0.13, -0.06, -0.27),
            v(0.2, -0.08, -0.27),
            v(0.17, -0.02, -0.27),
        ],
        INK,
    );
    m
}

fn skeleton(k: &mut Kit, bank: &mut TexBank) -> Mesh {
    let mut m = Mesh::new();
    // Crossed bones behind the skull.
    for sx in [-1.0f32, 1.0] {
        let (a, b) = (v(-sx * 0.19, 0.03, -0.17), v(sx * 0.19, 0.3, -0.17));
        stick(k, bank, &mut m, a, b, 0.017, SAND);
        for end in [a, b] {
            let across = v(0.022, -sx * 0.016, 0.0);
            ball(k, bank, &mut m, end + across, v(0.022, 0.022, 0.022), WHITE);
            ball(k, bank, &mut m, end - across, v(0.022, 0.022, 0.022), WHITE);
        }
    }
    let b = (v(0.0, 0.19, -0.21), v(0.105, 0.09, 0.09));
    ball(k, bank, &mut m, b.0, b.1, WHITE);
    k.bx(
        bank,
        &mut m,
        v(-0.065, 0.085, -0.28),
        v(0.065, 0.13, -0.17),
        SAND,
    );
    for x in [-0.036f32, 0.0, 0.036] {
        k.bx(
            bank,
            &mut m,
            v(x - 0.011, 0.1, -0.287),
            v(x + 0.011, 0.126, -0.278),
            WHITE,
        );
    }
    for sx in [-1.0f32, 1.0] {
        dab(
            k,
            bank,
            &mut m,
            b,
            (sx * 0.042, 0.2),
            (0.045, 0.045),
            0.0,
            INK,
        );
    }
    dab(k, bank, &mut m, b, (0.0, 0.155), (0.02, 0.022), 0.0, INK);
    m
}

/// A golem's stone: grey-brown with darker cracks and pale flecks.
fn stone() -> Texture {
    let mut t = Texture::new(8, 8, KHAKI);
    for y in 0..8 {
        for x in 0..8 {
            if (x * 3 + y * 5) % 11 == 0 {
                t.set(x, y, SAND);
            } else if (x + y * 2) % 9 == 0 {
                t.set(x, y, SHADOW);
            }
        }
    }
    t
}

fn golem(k: &mut Kit, bank: &mut TexBank) -> Mesh {
    let mut m = Mesh::new();
    let rock = bank.add(stone());
    skin_box(
        &mut m,
        v(-0.13, 0.0, -0.3),
        v(0.13, 0.24, BACK),
        rock,
        &k.w4,
    );
    k.bx(
        bank,
        &mut m,
        v(-0.13, 0.235, -0.3),
        v(0.04, 0.255, -0.15),
        GREEN,
    );
    k.bx(
        bank,
        &mut m,
        v(-0.1, 0.25, -0.26),
        v(-0.04, 0.27, -0.2),
        LIME,
    );
    cone(
        k,
        bank,
        &mut m,
        v(0.08, 0.235, -0.24),
        v(0.2, 1.0, -0.2),
        0.11,
        0.035,
        SKY,
    );
    cone(
        k,
        bank,
        &mut m,
        v(0.11, 0.235, -0.18),
        v(0.4, 1.0, 0.0),
        0.075,
        0.025,
        MINT,
    );
    // One glowing crystal eye under a heavy brow, and cracks.
    k.bx(
        bank,
        &mut m,
        v(-0.035, 0.115, -0.312),
        v(0.035, 0.18, -0.295),
        SKY,
    );
    k.bx(
        bank,
        &mut m,
        v(-0.022, 0.155, -0.316),
        v(-0.008, 0.172, -0.31),
        WHITE,
    );
    k.bx(
        bank,
        &mut m,
        v(-0.065, 0.183, -0.319),
        v(0.065, 0.205, -0.295),
        SHADOW,
    );
    for (a, b) in [
        (v(-0.11, 0.03, -0.302), v(-0.06, 0.09, -0.302)),
        (v(0.07, 0.06, -0.302), v(0.11, 0.02, -0.302)),
        (v(0.06, 0.2, -0.302), v(0.1, 0.16, -0.302)),
    ] {
        stick(k, bank, &mut m, a, b, 0.005, SHADOW);
    }
    m
}

fn ghost(k: &mut Kit, bank: &mut TexBank) -> Mesh {
    let mut m = Mesh::new();
    let sheet = k.c(bank, WHITE);
    let at = v(0.0, -0.02, -0.21);
    lathe(
        &mut m,
        at,
        &[
            (0.13, 0.0),
            (0.14, 0.06),
            (0.135, 0.15),
            (0.1, 0.23),
            (0.05, 0.27),
            (0.0, 0.28),
        ],
        8,
        0.3,
        sheet,
        false,
    );
    // A wavy hem.
    for i in 0..7 {
        let a = i as f32 / 7.0 * TAU + 0.2;
        let base = at + v(a.sin() * 0.115, 0.01, -a.cos() * 0.115);
        cone(k, bank, &mut m, base, v(0.0, -1.0, 0.0), 0.05, 0.035, WHITE);
    }
    let z = -0.21 - 0.135;
    for sx in [-1.0f32, 1.0] {
        k.bx(
            bank,
            &mut m,
            v(sx * 0.045 - 0.014, 0.13, z - 0.01),
            v(sx * 0.045 + 0.014, 0.19, z + 0.012),
            INK,
        );
        k.bx(
            bank,
            &mut m,
            v(sx * 0.085 - 0.014, 0.1, z - 0.004),
            v(sx * 0.085 + 0.014, 0.112, z + 0.012),
            BLUSH,
        );
    }
    k.bx(
        bank,
        &mut m,
        v(-0.018, 0.065, z - 0.01),
        v(0.018, 0.1, z + 0.012),
        INK,
    );
    m
}

fn frog(k: &mut Kit, bank: &mut TexBank) -> Mesh {
    let mut m = Mesh::new();
    let b = (v(0.0, 0.11, -0.2), v(0.15, 0.11, 0.1));
    ball(k, bank, &mut m, b.0, b.1, GREEN);
    ball(
        k,
        bank,
        &mut m,
        v(0.0, 0.075, -0.26),
        v(0.1, 0.065, 0.05),
        LIME,
    );
    for sx in [-1.0f32, 1.0] {
        ball(
            k,
            bank,
            &mut m,
            v(sx * 0.075, 0.21, -0.23),
            v(0.048, 0.048, 0.048),
            GREEN,
        );
        ball(
            k,
            bank,
            &mut m,
            v(sx * 0.075, 0.215, -0.262),
            v(0.032, 0.032, 0.02),
            WHITE,
        );
        k.bx(
            bank,
            &mut m,
            v(sx * 0.075 - 0.012, 0.203, -0.285),
            v(sx * 0.075 + 0.012, 0.228, -0.275),
            INK,
        );
        dab(
            k,
            bank,
            &mut m,
            b,
            (sx * 0.1, 0.14),
            (0.03, 0.014),
            0.0,
            PINK,
        );
        // Little feet.
        k.bx(
            bank,
            &mut m,
            v(sx * 0.12 - 0.035, 0.0, -0.3),
            v(sx * 0.12 + 0.035, 0.02, -0.24),
            GREEN,
        );
    }
    dab(k, bank, &mut m, b, (0.0, 0.14), (0.13, 0.012), 0.0, INK);
    m
}

fn jelly(k: &mut Kit, bank: &mut TexBank) -> Mesh {
    let mut m = Mesh::new();
    let bell = k.c(bank, LAVENDER);
    let at = v(0.0, 0.12, -0.21);
    lathe(
        &mut m,
        at,
        &[
            (0.14, 0.0),
            (0.15, 0.03),
            (0.13, 0.09),
            (0.08, 0.135),
            (0.0, 0.15),
        ],
        8,
        0.3,
        bell,
        true,
    );
    let rim = k.c(bank, BLUSH);
    lathe(
        &mut m,
        at - v(0.0, 0.01, 0.0),
        &[(0.152, 0.0), (0.152, 0.022)],
        8,
        0.3,
        rim,
        false,
    );
    // Streaming threads.
    for i in 0..5 {
        let x = -0.1 + i as f32 * 0.05;
        let z = -0.28 + (x.abs() * 0.4);
        let bend = if i % 2 == 0 { 0.025 } else { -0.025 };
        let (a, b, c) = (v(x, 0.11, z), v(x + bend, 0.02, z - 0.01), v(x, -0.08, z));
        stick(k, bank, &mut m, a, b, 0.011, PINK);
        stick(k, bank, &mut m, b, c, 0.01, BLUSH);
    }
    let z = -0.21 - 0.14;
    for sx in [-1.0f32, 1.0] {
        k.bx(
            bank,
            &mut m,
            v(sx * 0.04 - 0.011, 0.17, z - 0.008),
            v(sx * 0.04 + 0.011, 0.195, z + 0.012),
            INK,
        );
    }
    k.bx(
        bank,
        &mut m,
        v(-0.02, 0.15, z - 0.006),
        v(0.02, 0.158, z + 0.012),
        GRAPE,
    );
    m
}

fn puffer(k: &mut Kit, bank: &mut TexBank) -> Mesh {
    let mut m = Mesh::new();
    let b = (v(0.0, 0.13, -0.21), v(0.14, 0.13, 0.11));
    ball(k, bank, &mut m, b.0, b.1, GOLD);
    ball(
        k,
        bank,
        &mut m,
        v(0.0, 0.07, -0.275),
        v(0.09, 0.05, 0.045),
        CREAM,
    );
    for (ex, ey) in [
        (-0.7f32, 0.35f32),
        (0.7, 0.35),
        (-0.4, 0.75),
        (0.4, 0.75),
        (0.0, 0.93),
        (-0.88, -0.1),
        (0.88, -0.1),
        (-0.65, -0.55),
        (0.65, -0.55),
        (-0.3, 0.55),
        (0.3, 0.55),
    ] {
        let dir = v(ex, ey, -(1.0 - ex * ex - ey * ey).max(0.05).sqrt());
        let base = b.0 + dir * b.1;
        cone(k, bank, &mut m, base, dir, 0.055, 0.018, SAND);
    }
    for sx in [-1.0f32, 1.0] {
        fan(
            k,
            bank,
            &mut m,
            &[
                v(sx * 0.13, 0.12, -0.21),
                v(sx * 0.21, 0.18, -0.22),
                v(sx * 0.2, 0.06, -0.22),
            ],
            ORANGE,
        );
    }
    eyes(k, bank, &mut m, b, 0.16, 0.055, 0.036);
    ball(
        k,
        bank,
        &mut m,
        v(0.0, 0.105, -0.322),
        v(0.02, 0.016, 0.012),
        PINK,
    );
    m
}

fn zombie(k: &mut Kit, bank: &mut TexBank) -> Mesh {
    let mut m = Mesh::new();
    k.bx(
        bank,
        &mut m,
        v(-0.12, 0.02, -0.3),
        v(0.12, 0.25, BACK),
        GREEN,
    );
    k.bx(
        bank,
        &mut m,
        v(-0.125, 0.245, -0.305),
        v(0.125, 0.275, -0.14),
        SHADOW,
    );
    for (x, dir) in [(-0.07f32, -0.3f32), (0.0, 0.1), (0.06, 0.4)] {
        cone(
            k,
            bank,
            &mut m,
            v(x, 0.27, -0.23),
            v(dir, 1.0, -0.2),
            0.06,
            0.025,
            SHADOW,
        );
    }
    k.bx(
        bank,
        &mut m,
        v(0.03, 0.12, -0.304),
        v(0.11, 0.24, -0.298),
        TEAL,
    );
    // Stitches down the middle.
    k.bx(
        bank,
        &mut m,
        v(-0.004, 0.03, -0.308),
        v(0.004, 0.24, -0.3),
        INK,
    );
    for y in [0.06f32, 0.11, 0.16, 0.21] {
        k.bx(
            bank,
            &mut m,
            v(-0.02, y, -0.31),
            v(0.02, y + 0.007, -0.3),
            INK,
        );
    }
    // One round staring eye, one sewn shut.
    k.bx(
        bank,
        &mut m,
        v(-0.085, 0.14, -0.31),
        v(-0.03, 0.195, -0.3),
        WHITE,
    );
    k.bx(
        bank,
        &mut m,
        v(-0.066, 0.152, -0.315),
        v(-0.046, 0.18, -0.309),
        INK,
    );
    for s in [-1.0f32, 1.0] {
        stick(
            k,
            bank,
            &mut m,
            v(0.035, 0.14 + s * 0.0, -0.312),
            v(0.085, 0.14 + 0.05, -0.312),
            0.006,
            INK,
        );
        stick(
            k,
            bank,
            &mut m,
            v(0.035, 0.19, -0.312),
            v(0.085, 0.14, -0.312),
            0.006,
            INK,
        );
    }
    k.bx(
        bank,
        &mut m,
        v(-0.06, 0.07, -0.308),
        v(0.06, 0.085, -0.3),
        INK,
    );
    k.bx(
        bank,
        &mut m,
        v(0.02, 0.058, -0.31),
        v(0.036, 0.075, -0.303),
        WHITE,
    );
    m
}

fn brute(k: &mut Kit, bank: &mut TexBank) -> Mesh {
    let mut m = Mesh::new();
    // The club, strapped on behind.
    stick(
        k,
        bank,
        &mut m,
        v(0.12, -0.03, -0.17),
        v(0.19, 0.3, -0.19),
        0.022,
        ROSEWOOD,
    );
    ball(
        k,
        bank,
        &mut m,
        v(0.195, 0.31, -0.19),
        v(0.045, 0.065, 0.045),
        RUST,
    );
    let b = (v(0.0, 0.13, -0.2), v(0.14, 0.12, 0.1));
    ball(k, bank, &mut m, b.0, b.1, GREEN);
    for sx in [-1.0f32, 1.0] {
        cone(
            k,
            bank,
            &mut m,
            v(sx * 0.13, 0.16, -0.2),
            v(sx, 0.3, 0.0),
            0.11,
            0.035,
            GREEN,
        );
        dab(
            k,
            bank,
            &mut m,
            b,
            (sx * 0.045, 0.15),
            (0.03, 0.022),
            0.0,
            GOLD,
        );
        dab(
            k,
            bank,
            &mut m,
            b,
            (sx * 0.045, 0.148),
            (0.012, 0.014),
            0.004,
            INK,
        );
        cone(
            k,
            bank,
            &mut m,
            v(sx * 0.042, 0.07, -0.3),
            v(0.0, 1.0, -0.3),
            0.055,
            0.016,
            WHITE,
        );
    }
    dab(k, bank, &mut m, b, (0.0, 0.185), (0.19, 0.03), 0.0, TEAL);
    dab(k, bank, &mut m, b, (0.0, 0.08), (0.12, 0.028), 0.0, INK);
    m
}

fn sneak(k: &mut Kit, bank: &mut TexBank) -> Mesh {
    let mut m = Mesh::new();
    // A dagger strapped at its side.
    stick(
        k,
        bank,
        &mut m,
        v(-0.14, 0.0, -0.2),
        v(-0.15, 0.1, -0.2),
        0.012,
        ROSEWOOD,
    );
    k.bx(
        bank,
        &mut m,
        v(-0.185, 0.095, -0.21),
        v(-0.115, 0.11, -0.19),
        GOLD,
    );
    stick(
        k,
        bank,
        &mut m,
        v(-0.15, 0.1, -0.2),
        v(-0.16, 0.27, -0.2),
        0.015,
        WHITE,
    );
    let hood = k.c(bank, SHADOW);
    lathe(
        &mut m,
        v(0.0, 0.0, -0.21),
        &[
            (0.13, 0.0),
            (0.14, 0.08),
            (0.12, 0.16),
            (0.08, 0.23),
            (0.03, 0.29),
            (0.0, 0.32),
        ],
        8,
        0.3,
        hood,
        true,
    );
    // Its face, peering out of the dark of the hood.
    ball(
        k,
        bank,
        &mut m,
        v(0.0, 0.14, -0.33),
        v(0.085, 0.075, 0.012),
        INK,
    );
    let f = (v(0.0, 0.14, -0.335), v(0.066, 0.056, 0.022));
    ball(k, bank, &mut m, f.0, f.1, LIME);
    for sx in [-1.0f32, 1.0] {
        dab(
            k,
            bank,
            &mut m,
            f,
            (sx * 0.028, 0.155),
            (0.022, 0.016),
            0.0,
            GOLD,
        );
        dab(
            k,
            bank,
            &mut m,
            f,
            (sx * 0.028, 0.155),
            (0.008, 0.012),
            0.004,
            INK,
        );
    }
    cone(
        k,
        bank,
        &mut m,
        v(0.0, 0.132, -0.35),
        v(0.0, -0.3, -1.0),
        0.06,
        0.017,
        GREEN,
    );
    m
}

fn bug(k: &mut Kit, bank: &mut TexBank) -> Mesh {
    let mut m = Mesh::new();
    for sx in [-1.0f32, 1.0] {
        for i in 0..4 {
            let y0 = 0.05 + i as f32 * 0.055;
            let knee = v(sx * 0.24, y0 + 0.07, -0.23);
            let foot = v(sx * 0.3, y0 - 0.07, -0.24);
            stick(k, bank, &mut m, v(sx * 0.07, y0, -0.21), knee, 0.01, INK);
            stick(k, bank, &mut m, knee, foot, 0.009, INK);
        }
    }
    let b = (v(0.0, 0.12, -0.21), v(0.12, 0.12, 0.1));
    ball(k, bank, &mut m, b.0, b.1, GREEN);
    dab(k, bank, &mut m, b, (0.0, 0.1), (0.05, 0.14), 0.0, TEAL);
    for (x, y) in [(-0.06f32, 0.16f32), (0.07, 0.07), (-0.05, 0.05)] {
        dab(k, bank, &mut m, b, (x, y), (0.025, 0.025), 0.0, LIME);
    }
    let h = (v(0.0, 0.25, -0.22), v(0.062, 0.05, 0.055));
    ball(k, bank, &mut m, h.0, h.1, DEEP_TEAL);
    for (x, y, s) in [
        (-0.02f32, 0.26f32, 0.018f32),
        (0.02, 0.26, 0.018),
        (-0.042, 0.245, 0.011),
        (0.042, 0.245, 0.011),
    ] {
        dab(k, bank, &mut m, h, (x, y), (s, s), 0.0, RED);
    }
    m
}

/// A lantern snail's shell: panes of coloured glass in dark leading.
fn stained() -> Texture {
    let panes = [GOLD, AQUA, PINK, LIME, SKY, ORANGE];
    let mut t = Texture::new(16, 16, INK);
    for y in 0..16 {
        for x in 0..16 {
            let (px, py) = ((x + y) / 4, y / 5);
            if (x + y) % 4 == 0 || y % 5 == 4 {
                continue;
            }
            t.set(x, y, panes[(px * 2 + py * 3) as usize % panes.len()]);
        }
    }
    t
}

fn snail(k: &mut Kit, bank: &mut TexBank) -> Mesh {
    let mut m = Mesh::new();
    let glass = bank.add(stained());
    let mut s = Mesh::new();
    lathe(
        &mut s,
        Vec3::ZERO,
        &[
            (0.14, 0.0),
            (0.15, 0.035),
            (0.14, 0.08),
            (0.105, 0.12),
            (0.06, 0.15),
            (0.0, 0.16),
        ],
        10,
        0.0,
        glass,
        false,
    );
    let lip = k.c(bank, INK);
    lathe(
        &mut s,
        Vec3::ZERO,
        &[(0.125, -0.01), (0.152, 0.005)],
        10,
        0.0,
        lip,
        false,
    );
    m.append(
        &s,
        Mat4::from_translation(v(0.0, 0.13, BACK - 0.01)) * Mat4::from_rotation_x(-PI * 0.5),
    );
    // The snail itself peeping over the top, eyes on stalks.
    ball(
        k,
        bank,
        &mut m,
        v(0.0, 0.29, -0.19),
        v(0.05, 0.035, 0.05),
        SAND,
    );
    for sx in [-1.0f32, 1.0] {
        stick(
            k,
            bank,
            &mut m,
            v(sx * 0.02, 0.31, -0.2),
            v(sx * 0.04, 0.38, -0.21),
            0.007,
            SAND,
        );
        ball(
            k,
            bank,
            &mut m,
            v(sx * 0.04, 0.385, -0.21),
            v(0.014, 0.014, 0.014),
            INK,
        );
    }
    m
}

fn bookworm(k: &mut Kit, bank: &mut TexBank) -> Mesh {
    let mut m = Mesh::new();
    // A thick old book...
    k.bx(
        bank,
        &mut m,
        v(-0.12, 0.0, -0.25),
        v(0.12, 0.26, -0.13),
        MAROON,
    );
    k.bx(
        bank,
        &mut m,
        v(0.1, 0.012, -0.243),
        v(0.126, 0.248, -0.137),
        CREAM,
    );
    for (x, y) in [(-0.1f32, 0.02f32), (0.08, 0.02), (-0.1, 0.22), (0.08, 0.22)] {
        k.bx(
            bank,
            &mut m,
            v(x, y, -0.256),
            v(x + 0.04, y + 0.03, -0.248),
            GOLD,
        );
    }
    k.bx(
        bank,
        &mut m,
        v(-0.04, 0.09, -0.258),
        v(0.04, 0.17, -0.249),
        GOLD,
    );
    k.bx(
        bank,
        &mut m,
        v(-0.022, 0.108, -0.262),
        v(0.022, 0.152, -0.255),
        RED,
    );
    // ...and the book-worm curled over the top of it, in its spectacles.
    for (i, x) in [-0.09f32, -0.045, 0.0].into_iter().enumerate() {
        let r = 0.034 + i as f32 * 0.003;
        ball(k, bank, &mut m, v(x, 0.285, -0.19), v(r, r, r), LIME);
    }
    let h = (v(0.06, 0.3, -0.2), v(0.048, 0.045, 0.045));
    ball(k, bank, &mut m, h.0, h.1, LIME);
    for sx in [-1.0f32, 1.0] {
        let x = 0.06 + sx * 0.02;
        dab(k, bank, &mut m, h, (x, 0.305), (0.026, 0.026), 0.0, INK);
        dab(k, bank, &mut m, h, (x, 0.305), (0.016, 0.016), 0.004, CREAM);
        stick(
            k,
            bank,
            &mut m,
            v(x, 0.34, -0.2),
            v(x + sx * 0.02, 0.39, -0.21),
            0.006,
            INK,
        );
    }
    m
}

fn drake(k: &mut Kit, bank: &mut TexBank) -> Mesh {
    let mut m = Mesh::new();
    let b = (v(0.0, 0.13, -0.2), v(0.12, 0.12, 0.09));
    for sx in [-1.0f32, 1.0] {
        wing(
            k,
            bank,
            &mut m,
            v(sx * 0.09, 0.19, -0.2),
            sx,
            1.25,
            SLATE,
            INK,
        );
        cone(
            k,
            bank,
            &mut m,
            v(sx * 0.05, 0.235, -0.19),
            v(sx * 0.3, 1.0, 0.25),
            0.1,
            0.028,
            CREAM,
        );
    }
    ball(k, bank, &mut m, b.0, b.1, SKY);
    for (x, y) in [
        (-0.05f32, 0.08f32),
        (0.06, 0.12),
        (-0.02, 0.2),
        (0.05, 0.04),
    ] {
        dab(k, bank, &mut m, b, (x, y), (0.03, 0.02), 0.0, BLUE);
    }
    for y in [0.05f32, 0.11, 0.17] {
        let z = face(b.0, b.1, 0.0, y);
        cone(
            k,
            bank,
            &mut m,
            v(0.0, y, z + 0.01),
            v(0.0, 0.3, -1.0),
            0.05,
            0.02,
            INDIGO,
        );
    }
    for sx in [-1.0f32, 1.0] {
        dab(
            k,
            bank,
            &mut m,
            b,
            (sx * 0.045, 0.215),
            (0.03, 0.022),
            0.0,
            GOLD,
        );
        dab(
            k,
            bank,
            &mut m,
            b,
            (sx * 0.045, 0.215),
            (0.008, 0.02),
            0.004,
            INK,
        );
    }
    // A tail curling down, a spade at its end.
    let pts = [
        v(0.03, 0.03, -0.25),
        v(0.08, -0.03, -0.26),
        v(0.14, -0.05, -0.25),
        v(0.18, -0.02, -0.24),
    ];
    for w in pts.windows(2) {
        stick(k, bank, &mut m, w[0], w[1], 0.016, SKY);
    }
    fan(
        k,
        bank,
        &mut m,
        &[
            v(0.18, -0.02, -0.24),
            v(0.17, 0.0, -0.24),
            v(0.24, 0.02, -0.24),
            v(0.2, -0.04, -0.24),
        ],
        INDIGO,
    );
    m
}

fn leafling(k: &mut Kit, bank: &mut TexBank) -> Mesh {
    let mut m = Mesh::new();
    for sx in [-1.0f32, 1.0] {
        leaf(
            k,
            bank,
            &mut m,
            v(sx * 0.08, 0.16, -0.22),
            v(sx * 0.33, 0.3, -0.22),
            0.1,
        );
        leaf(
            k,
            bank,
            &mut m,
            v(sx * 0.08, 0.12, -0.22),
            v(sx * 0.3, 0.02, -0.22),
            0.08,
        );
        leaf(
            k,
            bank,
            &mut m,
            v(sx * 0.04, 0.22, -0.21),
            v(sx * 0.13, 0.39, -0.21),
            0.07,
        );
    }
    let b = (v(0.0, 0.13, -0.2), v(0.11, 0.11, 0.085));
    ball(k, bank, &mut m, b.0, b.1, LIME);
    for sx in [-1.0f32, 1.0] {
        dab(
            k,
            bank,
            &mut m,
            b,
            (sx * 0.04, 0.14),
            (0.024, 0.026),
            0.0,
            RED,
        );
        dab(
            k,
            bank,
            &mut m,
            b,
            (sx * 0.04, 0.14),
            (0.01, 0.014),
            0.004,
            INK,
        );
        let z = face(b.0, b.1, sx * 0.04, 0.175) - 0.01;
        stick(
            k,
            bank,
            &mut m,
            v(sx * 0.015, 0.165, z),
            v(sx * 0.07, 0.185, z),
            0.007,
            DEEP_TEAL,
        );
    }
    dab(
        k,
        bank,
        &mut m,
        b,
        (0.0, 0.09),
        (0.04, 0.01),
        0.0,
        DEEP_TEAL,
    );
    m
}

fn werewolf(k: &mut Kit, bank: &mut TexBank) -> Mesh {
    let mut m = Mesh::new();
    // A bushy tail hanging down behind.
    let fur = k.c(bank, SHADOW);
    let pale = k.c(bank, KHAKI);
    let mut tail = Mesh::new();
    lathe(
        &mut tail,
        Vec3::ZERO,
        &[(0.03, 0.0), (0.07, 0.08), (0.06, 0.16), (0.03, 0.21)],
        6,
        0.0,
        fur,
        false,
    );
    lathe(
        &mut tail,
        Vec3::ZERO,
        &[(0.03, 0.21), (0.0, 0.25)],
        6,
        0.0,
        pale,
        false,
    );
    m.append(
        &tail,
        Mat4::from_translation(v(0.05, 0.03, -0.25))
            * Mat4::from_rotation_z(-2.6)
            * Mat4::from_rotation_x(0.3),
    );
    let b = (v(0.0, 0.13, -0.2), v(0.13, 0.12, 0.1));
    ball(k, bank, &mut m, b.0, b.1, SHADOW);
    for sx in [-1.0f32, 1.0] {
        cone(
            k,
            bank,
            &mut m,
            v(sx * 0.07, 0.22, -0.2),
            v(sx * 0.3, 1.0, 0.0),
            0.11,
            0.045,
            SHADOW,
        );
        cone(
            k,
            bank,
            &mut m,
            v(sx * 0.07, 0.225, -0.235),
            v(sx * 0.3, 1.0, 0.0),
            0.07,
            0.022,
            ROSEWOOD,
        );
        cone(
            k,
            bank,
            &mut m,
            v(sx * 0.12, 0.09, -0.22),
            v(sx, -0.3, -0.2),
            0.065,
            0.03,
            SHADOW,
        );
        dab(
            k,
            bank,
            &mut m,
            b,
            (sx * 0.048, 0.165),
            (0.03, 0.022),
            0.0,
            GOLD,
        );
        dab(
            k,
            bank,
            &mut m,
            b,
            (sx * 0.048, 0.163),
            (0.01, 0.016),
            0.004,
            INK,
        );
        let z = face(b.0, b.1, sx * 0.05, 0.19) - 0.012;
        stick(
            k,
            bank,
            &mut m,
            v(sx * 0.02, 0.2, z),
            v(sx * 0.075, 0.185, z),
            0.008,
            INK,
        );
    }
    let muzzle = (v(0.0, 0.1, -0.28), v(0.062, 0.046, 0.045));
    ball(k, bank, &mut m, muzzle.0, muzzle.1, KHAKI);
    ball(
        k,
        bank,
        &mut m,
        v(0.0, 0.125, -0.325),
        v(0.02, 0.014, 0.012),
        INK,
    );
    for sx in [-1.0f32, 1.0] {
        cone(
            k,
            bank,
            &mut m,
            v(sx * 0.022, 0.085, -0.32),
            v(0.0, -1.0, -0.2),
            0.03,
            0.009,
            WHITE,
        );
    }
    m
}

fn minotaur(k: &mut Kit, bank: &mut TexBank) -> Mesh {
    let mut m = Mesh::new();
    let b = (v(0.0, 0.13, -0.2), v(0.13, 0.12, 0.1));
    ball(k, bank, &mut m, b.0, b.1, RUST);
    // A dark forelock, and horns out to the sides and up, pale to their tips.
    dab(k, bank, &mut m, b, (0.0, 0.22), (0.1, 0.035), 0.0, MAROON);
    for sx in [-1.0f32, 1.0] {
        cone(
            k,
            bank,
            &mut m,
            v(sx * 0.1, 0.2, -0.2),
            v(sx, 0.35, 0.0),
            0.1,
            0.03,
            SAND,
        );
        cone(
            k,
            bank,
            &mut m,
            v(sx * 0.19, 0.23, -0.2),
            v(sx * 0.3, 1.0, 0.0),
            0.09,
            0.022,
            WHITE,
        );
        // Floppy ears under them.
        cone(
            k,
            bank,
            &mut m,
            v(sx * 0.12, 0.15, -0.19),
            v(sx, -0.2, -0.1),
            0.07,
            0.028,
            RUST,
        );
    }
    eyes(k, bank, &mut m, b, 0.165, 0.05, 0.03);
    // A big pink muzzle with nostrils, and a gold ring through its nose.
    let muzzle = (v(0.0, 0.08, -0.29), v(0.08, 0.05, 0.04));
    ball(k, bank, &mut m, muzzle.0, muzzle.1, SALMON);
    for sx in [-1.0f32, 1.0] {
        dab(
            k,
            bank,
            &mut m,
            muzzle,
            (sx * 0.03, 0.09),
            (0.018, 0.014),
            0.0,
            INK,
        );
    }
    let gold = k.c(bank, GOLD);
    let mut ring = Mesh::new();
    lathe(
        &mut ring,
        Vec3::ZERO,
        &[(0.022, -0.006), (0.03, 0.0), (0.022, 0.006), (0.018, 0.0)],
        8,
        0.0,
        gold,
        false,
    );
    m.append(
        &ring,
        Mat4::from_translation(v(0.0, 0.045, -0.335)) * Mat4::from_rotation_x(PI / 2.0),
    );
    m
}

fn griffin(k: &mut Kit, bank: &mut TexBank) -> Mesh {
    let mut m = Mesh::new();
    // Russet wings spread out behind, a tawny lion's body with a tufted tail, and a white
    // eagle's head on top with a hooked gold beak.
    for sx in [-1.0f32, 1.0] {
        let p = |x: f32, y: f32| v(sx * x, y, -0.19);
        fan(
            k,
            bank,
            &mut m,
            &[
                p(0.08, 0.14),
                p(0.14, 0.24),
                p(0.3, 0.2),
                p(0.34, 0.12),
                p(0.3, 0.05),
                p(0.22, 0.02),
                p(0.12, 0.06),
            ],
            CLAY,
        );
        for (x, y) in [(0.26f32, 0.14f32), (0.2, 0.09)] {
            stick(k, bank, &mut m, p(0.1, 0.14), p(x, y), 0.006, RUST);
        }
    }
    let body = (v(0.0, 0.07, -0.2), v(0.12, 0.08, 0.09));
    ball(k, bank, &mut m, body.0, body.1, SAND);
    let pts = [
        v(-0.05, 0.02, -0.24),
        v(-0.12, 0.0, -0.25),
        v(-0.17, 0.04, -0.25),
    ];
    for w in pts.windows(2) {
        stick(k, bank, &mut m, w[0], w[1], 0.012, SAND);
    }
    ball(
        k,
        bank,
        &mut m,
        v(-0.18, 0.06, -0.25),
        v(0.025, 0.025, 0.025),
        RUST,
    );
    let head = (v(0.0, 0.2, -0.21), v(0.09, 0.085, 0.08));
    ball(k, bank, &mut m, head.0, head.1, WHITE);
    eyes(k, bank, &mut m, head, 0.215, 0.04, 0.026);
    let z = face(head.0, head.1, 0.0, 0.185);
    cone(
        k,
        bank,
        &mut m,
        v(0.0, 0.185, z + 0.01),
        v(0.0, -0.4, -1.0),
        0.06,
        0.025,
        GOLD,
    );
    // Its crest.
    cone(
        k,
        bank,
        &mut m,
        v(0.0, 0.27, -0.2),
        v(0.0, 1.0, 0.5),
        0.06,
        0.022,
        SAND,
    );
    m
}

/// Makes monster backpack `p` (see `ICONS`), as it hangs on the back before it's grown.
pub(super) fn plush(p: usize, k: &mut Kit, bank: &mut TexBank) -> Mesh {
    let make = [
        slime, bat, shroom, crab, wisp, beetle, imp, skeleton, golem, ghost, frog, jelly, puffer,
        zombie, brute, sneak, bug, snail, bookworm, drake, leafling, werewolf, minotaur, griffin,
    ];
    make[p](k, bank)
}

const SLIME: &[&str] = &[
    "................",
    "................",
    "......KKKK......",
    "....KKggggKK....",
    "...KgllgggggK...",
    "..KglwlgggggtK..",
    "..KglggggggggK..",
    ".KggKKggggKKggK.",
    ".KggKwggggKwggK.",
    ".KggKKggggKKggK.",
    ".KgggggggggggtK.",
    ".KggggPgKgPgttK.",
    ".KtgggggKKgtttK.",
    "..KttttttttttK..",
    "...KKKKKKKKKK...",
    "................",
];

const BAT: &[&str] = &[
    "................",
    "....K......K....",
    "...KVK....KVK...",
    "...KVVKKKKVVK...",
    "K..KVVVVVVVVK..K",
    "vK.KVKKVVKKVK.Kv",
    "vvKKVKwVVKwVKKvv",
    "vvvKVVVVVVVVKvvv",
    "vvvKVVwVVwVVKvvv",
    "vvvvKVVKKVVKvvvv",
    "vvK.KVLLLLVK.Kvv",
    "vK...KVVVVK...Kv",
    "K.....KKKK.....K",
    "................",
    "................",
    "................",
];

const SHROOM: &[&str] = &[
    "................",
    ".....KKKKKK.....",
    "...KKrrwwrrKK...",
    "..KrrrrwwrrrrK..",
    ".KrwwrrrrrrwwrK.",
    ".KrwwrrrrrrwwrK.",
    "KrrrrrrwwrrrrrrK",
    "KccrrrrwwrrrrccK",
    ".KKKKKKKKKKKKKK.",
    "...KnnnnnnnnK...",
    "...KnKKnnKKnK...",
    "...KnKwnnKwnK...",
    "...KnPnKKnPnK...",
    "...KnnnnnnnnK...",
    "....KKKKKKKK....",
    "................",
];

const CRAB: &[&str] = &[
    "................",
    "...K........K...",
    "..KwK......KwK..",
    "..KKK.KSSK.KKK..",
    "...K.KSwwSK.K...",
    "KK.KKaSaaSaKK.KK",
    "KaKKaaaaaaaaKKaK",
    "KaaKaKKaaKKaKaaK",
    "KaaKaKwaaKwaKaaK",
    "KKaKaaaaaaaaKaKK",
    ".KKKtaaKKaatKKK.",
    "...KttaaaattK...",
    "...KtttttttK....",
    "....KKKKKKK.....",
    "...K.K...K.K....",
    "................",
];

const WISP: &[&str] = &[
    ".......K........",
    "......KMK..K....",
    ".....KMMK.KMK...",
    "....KMMMMKMMK...",
    "...KMMaMMMMMK...",
    "...KMaaaaaMMK...",
    "..KMaawwwaaMMK..",
    "..KMawwwwwaaMK..",
    "..KMaKwwwKaaMK..",
    "..KMaKwwwKaaMK..",
    "..KMawwwwwwaMK..",
    "..KMaawKKwaaMK..",
    "...KMaawwaaMK...",
    "....KMMaaMMK....",
    ".....KKKKKK.....",
    "................",
];

const BEETLE: &[&str] = &[
    "................",
    "...K........K...",
    "....K......K....",
    ".....KKKKKK.....",
    "....KvwvvwvK....",
    "...KKKKKKKKKK...",
    "..KLLLLKKLLLLK..",
    ".KLLVLLKKLLVLLK.",
    ".KLVVLLKKLLVVLK.",
    ".KLLLLVKKVLLLLK.",
    ".KVLLLLKKLLLLVK.",
    ".KVVLLLKKLLLVVK.",
    "..KVVVVKKVVVVK..",
    "...KVVVKKVVVK...",
    "....KKKK.KKKK...",
    "................",
];

const IMP: &[&str] = &[
    "................",
    "..KK........KK..",
    "..KKK......KKK..",
    "...KKK.KK.KKK...",
    "...KrKKrrKKrK...",
    "..KrrrrrrrrrrK..",
    ".KrrrrrrrrrrrrK.",
    "KrrKYYKrrKYYKrrK",
    "KrrKYKKrrKKYKrrK",
    ".KrrrrrrrrrrrrK.",
    ".KrrKKKKKKKKrrK.",
    "..KrKwKwKKwKrK..",
    "..KrrKKKKKKrrK..",
    "...KKrrrrrrKK...",
    ".....KKKKKK.....",
    "................",
];

const SKELETON: &[&str] = &[
    "................",
    ".....KKKKKK.....",
    "...KKwwwwwwKK...",
    "..KwwwwwwwwwwK..",
    "..KwwwwwwwwwnK..",
    ".KwwKKKwwKKKwnK.",
    ".KwwKKKwwKKKwnK.",
    ".KwwKKKwwKKKwnK.",
    ".KwwwwwKKwwwwnK.",
    "..KwwwwKKwwwnK..",
    "...KwwwwwwwnK...",
    "...KKwKwKwKKK...",
    "...KwKwKwKnK....",
    "....KKKKKKKK....",
    "................",
    "................",
];

const GOLEM: &[&str] = &[
    "................",
    "...K..KKKK..K...",
    "..KSK.KggK.KSK..",
    "..KSKKKKKKKKSK..",
    "..KhhhhhhhhhhK..",
    ".KhhnhhhhhhnhhK.",
    ".KhhhhkkkkhhhhK.",
    ".KhhhhKSSKhhhhK.",
    ".KhkhhKSwKhhkhK.",
    ".KhkhhKSSKhhkhK.",
    ".KhhhhkkkkhhhhK.",
    ".KhhnhhhhhhhnhK.",
    ".KkhhhhhhhhhhkK.",
    ".KkkkkkkkkkkkkK.",
    "..KKKKKKKKKKKK..",
    "................",
];

const GHOST: &[&str] = &[
    "................",
    "......KKKK......",
    "....KKwwwwKK....",
    "...KwwwwwwwwK...",
    "..KwwwwwwwwwwK..",
    "..KwwKKwwKKwwK..",
    "..KwwKKwwKKwwK..",
    "..KwwKKwwKKwwK..",
    "..KwbwwwwwwbwK..",
    "..KwwwwKKwwwwK..",
    "..KwwwwKKwwwwK..",
    "..KwwwwwwwwwSK..",
    "..KwwwwwwwwSSK..",
    "..KwSKwSKwSKSK..",
    "...KK.KK.KK.K...",
    "................",
];

const FROG: &[&str] = &[
    "................",
    "...KKK....KKK...",
    "..KwwwK..KwwwK..",
    "..KwKwKKKKwKwK..",
    "..KwKKggggKKwK..",
    "..KggggggggggK..",
    ".KggggggggggggK.",
    ".KPPgggggggPPgK.",
    ".KgKKKKKKKKKKgK.",
    ".KggKrrrrrrKggK.",
    ".KgggKKKKKKgggK.",
    ".KglllllllllgK..",
    "..KllllllllllK..",
    "...KgKKKKKKgK...",
    "...KK......KK...",
    "................",
];

const JELLY: &[&str] = &[
    "................",
    "......KKKK......",
    "....KKLLLLKK....",
    "...KLLbLLLLLK...",
    "..KLLbLLLLLLLK..",
    "..KLLKLLLLKLLK..",
    "..KLLLLKKLLLLK..",
    "..KbbbbbbbbbbK..",
    "...KbKbKKbKbK...",
    "...KPK.KK.KPK...",
    "....KP.KP.KP....",
    "...KP..KP..KP...",
    "....KP..KP.KP...",
    "...KP..KP...K...",
    "....K...K.......",
    "................",
];

const PUFFER: &[&str] = &[
    "................",
    "...n..n..n..n...",
    "....KKKKKKKK....",
    "..nKYYYYYYYYKn..",
    "..KYYYnYYnYYYK..",
    ".nKYKKYYYYKKYKn.",
    "..KYKwYYYYKwYK..",
    ".nKYYYYYYYYYYKn.",
    "..KYYYYKKYYYYK..",
    ".nKYYYKPPKYYYKn.",
    "..KYYYYKKYYYYK..",
    "..nKyyyyyyyyKn..",
    "...KKyyyyyyKK...",
    "....nKKKKKKn....",
    "................",
    "................",
];

const ZOMBIE: &[&str] = &[
    "................",
    "....K.K.K.......",
    "...KkKkKkKKK....",
    "..KKKKKKKKKKKK..",
    "..KggggKggggggK.",
    "..KggggKgggggKK.",
    "..KgKwwKgKgKgK..",
    "..KgKwKKggKggK..",
    "..KgKKKKgKgKgK..",
    "..KggggKggggggK.",
    "..KggKKKKKKKggK.",
    "..KgggKwKgggggK.",
    "..KggggggggggtK.",
    "..KtttttttttttK.",
    "...KKKKKKKKKKK..",
    "................",
];

const BRUTE: &[&str] = &[
    "................",
    "................",
    "K....KKKKKK....K",
    "gK.KKggggggKK.Kg",
    "KggKggggggggKggK",
    ".KgKTTTTTTTTKgK.",
    "..KgKYKggKYKgK..",
    "..KggggggggggK..",
    "..KggggKKggggK..",
    "..KgKKKKKKKKgK..",
    "..KgKwKKKKwKgK..",
    "..KggwggggwggK..",
    "...KggggggggK...",
    "....KKKKKKKK....",
    "................",
    "................",
];

const SNEAK: &[&str] = &[
    ".......K........",
    "......KkK.......",
    ".....KkkkK......",
    "....KkkkkkK.....",
    "...KkkkkkkkK....",
    "..KkkKKKKKkkK...",
    "..KkKllllllKkK..",
    "..KkKYKllKYKkK..",
    ".KkKlllllllllKK.",
    ".KkKllllKKlllKkK",
    ".KkKlllllKKlKkK.",
    ".KkkKllllllKkkK.",
    ".KkkkKKKKKKkkkK.",
    ".KkkkkkkkkkkkkK.",
    "..KKKKKKKKKKKK..",
    "................",
];

const BUG: &[&str] = &[
    "................",
    "..K..........K..",
    "...K..KKKK..K...",
    "....KKTTTTKK....",
    "...K.KrTTrK.K...",
    "KK..KgKKKKgK..KK",
    "..KKgggggggtKK..",
    "KKKggllgggggtKKK",
    "..KgglgggggggK..",
    "KKKgggKKKKgggKKK",
    "..KgggKttKgggK..",
    "KK.KggKKKKggK.KK",
    "...KKggggggKK...",
    "..K..KKKKKK..K..",
    "................",
    "................",
];

const SNAIL: &[&str] = &[
    "................",
    ".....KKKKKK.....",
    "...KKYYKaaaKK...",
    "..KYYYKaaaaaaK..",
    "..KYYKPPPKaaaK..",
    ".KKKKPKKKPKKKKK.",
    ".KaaKPKYYKPKlK..",
    ".KaaKPKYKKPKlK..",
    ".KaaKPPKKPPKllK.",
    ".KaaaKKPPKKlllK.",
    "..KaaaKKKKlllK..",
    "..KKKaaaKlllKK..",
    "....KKKKKKKK....",
    "................",
    "................",
    "................",
];

const BOOKWORM: &[&str] = &[
    "................",
    "...KKKK.........",
    "..KllllK........",
    ".KwwKwwK.KKKKK..",
    ".KwKKKwKKlllllK.",
    ".KllllllllgllgK.",
    "..KKKKKKKKKKKK..",
    "..KmmmmmmmmmmKK.",
    "..KmYYmmmmYYmKyK",
    "..KmYmmYYmmYmKyK",
    "..KmmmYmmYmmmKyK",
    "..KmYmmYYmmYmKyK",
    "..KmYYmmmmYYmKyK",
    "..KKKKKKKKKKKKKK",
    "................",
    "................",
];

const DRAKE: &[&str] = &[
    "................",
    ".....y....y.....",
    "KK...Ky..yK...KK",
    "KDK..KSKKSK..KDK",
    "KDDKKSSSSSSKKDDK",
    "KDDDKSYSSYSKDDDK",
    "KIDDKSKSSKSKDDIK",
    ".KIDKSSSSSSKDIK.",
    ".KIIKBSnnSBKIIK.",
    "..KIKBBSSBBKIK..",
    "...KKBBBBBBKK...",
    ".....KBBBBK.....",
    "......KBBKK.....",
    ".......KBBK.....",
    "........KIIK....",
    ".........KK.....",
];

const LEAFLING: &[&str] = &[
    "......K..K......",
    "...K.KgKKgK.K...",
    "..KgKKggggKKgK..",
    "..KggKllllKggK..",
    ".KggKllllllKggK.",
    ".KgKllllllllKgK.",
    "KggKlTllllTlKggK",
    "KgKllrTllTrllKgK",
    "KgKllrrllrrllKgK",
    ".KKllllllllllKK.",
    "..KllllKKllllK..",
    "..KlllKllKlllK..",
    "...KllllllllK...",
    "....KKggggKK....",
    ".....KgKKgK.....",
    "......K..K......",
];

const WEREWOLF: &[&str] = &[
    "................",
    "..KK........KK..",
    "..KkK......KkK..",
    "..KkkK....KkkK..",
    "..KkRkKKKKkRkK..",
    ".KkkkkkkkkkkkkK.",
    ".KkKKkkkkkkKKkK.",
    ".KkkYKkkkkKYkkK.",
    "KkkkkkkhhkkkkkkK",
    "KkkkkkhhhhkkkkkK",
    ".KkkkhhKKhhkkkK.",
    ".KkkkhKhhKhkkkK.",
    "..KkkhwhhwhkkK..",
    "...KkkhhhhkkK...",
    "....KKKKKKKK....",
    "................",
];
const MINOTAUR: &[&str] = &[
    "................",
    ".Kw..........wK.",
    ".Kyw........wyK.",
    "..Kny......ynK..",
    "...KnKKKKKKnK...",
    "..KuummmmmmuuK..",
    ".KuuuuuuuuuuuuK.",
    "KuKuKKuuuuKKuKuK",
    "KuKuKwuuuuKwuKuK",
    ".KuuuuuuuuuuuuK.",
    "..KussssssssuK..",
    "..KsKsssssKssK..",
    "..KssssssssssK..",
    "...KssYYYYssK...",
    "....KKYKKYKK....",
    "......KYYK......",
];

const GRIFFIN: &[&str] = &[
    "................",
    "......KnnK......",
    ".....KwwwwK.....",
    "....KwwwwwwK....",
    "KK.KwKwwwwKwK.KK",
    "KCKKwKYwwYKwKKCK",
    "KCCKwwwYYwwwKCCK",
    "KCuKwwwYYwwwKuCK",
    "KCuuKwwoowwKuuCK",
    ".KCuKKwwwwKKuCK.",
    ".KCuKnnnnnnKuCK.",
    "..KKnnnnnnnnKK..",
    "...KnnhnnhnnK...",
    "...KnnnnnnnnKuK.",
    "....KYK..KYK.Ku.",
    "....KK....KK..K.",
];

const ART: [&[&str]; PLUSHES] = [
    SLIME, BAT, SHROOM, CRAB, WISP, BEETLE, IMP, SKELETON, GOLEM, GHOST, FROG, JELLY, PUFFER,
    ZOMBIE, BRUTE, SNEAK, BUG, SNAIL, BOOKWORM, DRAKE, LEAFLING, WEREWOLF, MINOTAUR, GRIFFIN,
];

/// Monster backpack `p`'s icon.
pub(super) fn icon(p: usize) -> Texture {
    art(ART[p], [CLEAR; 4])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_monster_backpack_has_a_tidy_icon() {
        for rows in ART {
            assert_eq!(rows.len(), 16);
            for r in rows {
                assert_eq!(r.chars().count(), 16, "{r}");
            }
        }
    }
}
