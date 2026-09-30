//! The Starless Rift's own creatures: the gazer, a winged eyeball trailing tentacles, and
//! the spineback slug with its eyes on stalks; and what they, and the rift's ogres, leave
//! behind.

use std::collections::HashMap;

use glam::{Mat4, Quat, Vec2, Vec3};

use super::models::{lathe, skin_box, tube};
use crate::palette::*;
use crate::render::{Mesh, TexBank, TexId, Texture, UvRect};
use crate::util::hash2;

/// A gazer, its middle at the origin, looking along +z.
pub struct Gazer {
    /// Its round violet body, the great eye in front with its lower lid, and little horns.
    pub body: Mesh,
    /// The iris and slit pupil, on the front of the eyeball (turn it about `GAZER_EYE_C` to
    /// look about).
    pub iris: Mesh,
    /// The upper lid, scowling down over the eye in a V (turn it about `GAZER_EYE_C`, down
    /// over the eye, to blink).
    pub lid: Mesh,
    /// A bat's wing, spreading out along +x from its root (mirror it for the other side).
    pub wing: Mesh,
    /// What its tentacles are skinned in (swept along a curve each frame).
    pub skin: TexId,
}

/// The middle of a gazer's eyeball, how big it is, and the front of it, where it looks out
/// from.
pub const GAZER_EYE_C: Vec3 = Vec3::new(0.0, 0.05, 0.1);
pub const GAZER_EYE_R: f32 = 0.165;
pub const GAZER_EYE: Vec3 = Vec3::new(0.0, 0.122, 0.249);
/// How far up from straight ahead its eye looks (radians): up at you, as you look down on
/// it.
const EYE_TILT: f32 = 0.45;
/// Where its wings join its body, and how big its body is.
pub const GAZER_WING: Vec3 = Vec3::new(0.2, 0.06, -0.04);
pub const GAZER_R: f32 = 0.24;

/// A spineback slug, lying along +z with its head at the front.
pub struct Slug {
    /// Its body, long and low on a violet skirt, a row of bony spines down its back.
    pub body: Mesh,
    /// An eye stalk, standing up along +y from its root, a glowing eye at the top.
    pub stalk: Mesh,
}

/// Where a slug's eye stalks stand (the middle one a little further back, and taller), and
/// how tall they are.
pub const SLUG_STALK: Vec3 = Vec3::new(0.065, 0.2, 0.38);
pub const SLUG_STALK_MID: Vec3 = Vec3::new(0.0, 0.23, 0.31);
pub const STALK_H: f32 = 0.2;
/// The spines down a slug's back: how far along it each stands, and how long it is.
pub const SLUG_SPINES: [(f32, f32); 5] = [
    (0.17, 0.12),
    (0.03, 0.18),
    (-0.11, 0.21),
    (-0.25, 0.17),
    (-0.37, 0.11),
];

/// The slug's body from tail to head: its half-width at each point along it.
const SLUG_BODY: [(f32, f32); 7] = [
    (0.0, -0.47),
    (0.12, -0.4),
    (0.17, -0.2),
    (0.18, 0.1),
    (0.16, 0.28),
    (0.1, 0.41),
    (0.0, 0.45),
];
/// How much lower than wide the slug is, and how high its middle rides.
const SLUG_FLAT: f32 = 0.72;
const SLUG_MID_Y: f32 = 0.1;

/// The height of the top of the slug's back, `z` along it.
pub fn slug_back(z: f32) -> f32 {
    let r = SLUG_BODY
        .windows(2)
        .find(|w| (w[0].1..=w[1].1).contains(&z))
        .map(|w| {
            let t = (z - w[0].1) / (w[1].1 - w[0].1);
            w[0].0 + (w[1].0 - w[0].0) * t
        })
        .unwrap_or(0.0);
    SLUG_MID_Y + r * SLUG_FLAT
}

pub struct VoidArt {
    pub gazer: Gazer,
    pub slug: Slug,
}

fn v(x: f32, y: f32, z: f32) -> Vec3 {
    Vec3::new(x, y, z)
}

/// A spike, `len` long from a base `r` across at `at`, pointing along `dir`, swelling a
/// little before it narrows: the whole height of `tex` runs up it, its top row at the tip.
fn spike(m: &mut Mesh, at: Vec3, dir: Vec3, len: f32, r: f32, tex: TexId) {
    let mut p = Mesh::new();
    p.lathe(
        Vec3::ZERO,
        &[(r, 0.0, 16.0), (r * 0.62, len * 0.45, 9.0), (0.0, len, 0.0)],
        6,
        16.0,
        0.3,
        tex,
        (Some(UvRect::new(0.0, 14.0, 16.0, 16.0)), None),
    );
    let rot = Quat::from_rotation_arc(Vec3::Y, dir.normalize());
    m.append(&p, Mat4::from_translation(at) * Mat4::from_quat(rot));
}

/// A ball, `r` its radii, round `at`.
fn ball(m: &mut Mesh, at: Vec3, r: Vec3, tex: TexId) {
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
        10,
        0.0,
        tex,
        false,
    );
    m.append(&s, Mat4::from_translation(at) * Mat4::from_scale(r));
}

/// A little knob of a ball, `r` across, round `at`: a knuckle.
fn knob(m: &mut Mesh, at: Vec3, r: f32, tex: TexId) {
    let mut s = Mesh::new();
    lathe(
        &mut s,
        Vec3::ZERO,
        &[(0.0, -1.0), (0.7, -0.7), (1.0, 0.0), (0.7, 0.7), (0.0, 1.0)],
        6,
        0.0,
        tex,
        false,
    );
    m.append(
        &s,
        Mat4::from_translation(at) * Mat4::from_scale(Vec3::splat(r)),
    );
}

/// A strip of a ball's skin, round `c` and `rad` out, over the columns `x` from `x0` to
/// `x1`: in each, from the angle `lo(x)` up to `hi(x)` (from the front, +z, up towards +y).
/// A thin band along one edge (the low one, or with `edge_high` the high one) takes `edge`;
/// the rest `tex`.
#[allow(clippy::too_many_arguments)]
fn skin_strip(
    m: &mut Mesh,
    c: Vec3,
    rad: f32,
    (x0, x1): (f32, f32),
    lo: &dyn Fn(f32) -> f32,
    hi: &dyn Fn(f32) -> f32,
    (tex, edge, edge_high): (TexId, TexId, bool),
) {
    const COLS: usize = 10;
    const BAND: f32 = 0.07;
    // Rows packed close by the edge, where the lid lies over the eye and must keep round
    // enough to stay over the iris (even blinking), and wider towards the back.
    const STEPS: [f32; 5] = [0.0, 0.12, 0.28, 0.55, 1.0];
    let at = |x: f32, th: f32| {
        let rho = (rad * rad - x * x).max(0.0).sqrt();
        c + v(x, rho * th.sin(), rho * th.cos())
    };
    // Each column's angles from bottom to top: the thin band, then the rest.
    let rows = |x: f32| -> Vec<f32> {
        let (a, b) = (lo(x), hi(x).max(lo(x)));
        let span = (b - a).max(0.0);
        let band = BAND.min(span * 0.5);
        let rest = span - band;
        let mut out = Vec::with_capacity(STEPS.len() + 1);
        if edge_high {
            out.extend(STEPS.iter().rev().map(|f| b - band - rest * f));
            out.push(b);
        } else {
            out.push(a);
            out.extend(STEPS.iter().map(|f| a + band + rest * f));
        }
        out
    };
    let solid = UvRect::new(0.0, 0.0, 4.0, 4.0);
    for i in 0..COLS {
        let xa = x0 + (x1 - x0) * i as f32 / COLS as f32;
        let xb = x0 + (x1 - x0) * (i + 1) as f32 / COLS as f32;
        let (ra, rb) = (rows(xa), rows(xb));
        for k in 0..ra.len() - 1 {
            let is_edge = if edge_high { k == ra.len() - 2 } else { k == 0 };
            m.quad(
                [
                    at(xa, ra[k]),
                    at(xb, rb[k]),
                    at(xb, rb[k + 1]),
                    at(xa, ra[k + 1]),
                ],
                solid,
                if is_edge { edge } else { tex },
            );
        }
    }
}

/// Violet hide, veined darker, with a sheen.
fn hide() -> Texture {
    let mut t = Texture::new(16, 16, PURPLE);
    for y in 0..16 {
        for x in 0..16 {
            let c = match hash2(x, y, 0x6A2) % 11 {
                0 | 1 => GRAPE,
                2 => LAVENDER,
                _ => PURPLE,
            };
            t.set(x, y, c);
        }
    }
    for (x0, y0) in [(2, 3), (9, 10)] {
        for k in 0..6 {
            t.set_wrap(x0 + k, y0 + (k / 2), GRAPE);
        }
    }
    t
}

/// The white of a gazer's eye, threaded with red veins.
fn eye_white() -> Texture {
    let mut t = Texture::new(16, 16, WHITE);
    for (x0, y0, dx) in [(1, 7, 1), (14, 6, -1), (7, 14, 0)] {
        let (mut x, mut y) = (x0, y0);
        for k in 0..6 {
            t.set_wrap(x, y, if k < 3 { RED } else { SALMON });
            x += dx;
            y += if dx == 0 {
                -1
            } else {
                (k % 2) * if y0 > 8 { -1 } else { 1 }
            };
        }
    }
    t
}

/// A gazer's iris: green, darker round the rim and bright about the middle, a slit of a
/// pupil down it and a glint up in the corner; clear outside the round.
fn iris() -> Texture {
    let mut t = Texture::new(16, 16, CLEAR);
    for y in 0..16 {
        for x in 0..16 {
            let (dx, dy) = (x as f32 - 7.5, y as f32 - 7.5);
            let r = dx.hypot(dy);
            let slit = dy.abs() < 6.2 && dx.abs() < 0.6 + 0.9 * (1.0 - (dy / 6.2).powi(2));
            let c = if r > 7.7 {
                CLEAR
            } else if slit {
                INK
            } else if r > 6.6 {
                DEEP_TEAL
            } else if r > 4.6 || ((x + y) % 2 == 0 && r > 3.4) {
                GREEN
            } else {
                LIME
            };
            t.set(x, y, c);
        }
    }
    for (x, y) in [(4, 3), (5, 3), (4, 4)] {
        t.set(x, y, WHITE);
    }
    t
}

/// A bat's wing membrane, laid across from one finger to the next (u) and out from the
/// wrist to the scalloped edge (v): darker in by the bones, a few veins of blood running
/// out through it, and a thin lit rim along its edge.
fn membrane() -> Texture {
    let mut t = Texture::new(16, 16, GRAPE);
    for y in 0..3 {
        for x in 0..16 {
            if (x + y) % 2 == 0 || y == 0 {
                t.set(x, y, INK);
            }
        }
    }
    // Veins wandering out, forking once.
    for (x0, seed) in [(5, 3), (11, 7)] {
        let mut x = x0;
        for y in 2..13 {
            t.set(x, y, PLUM);
            if y == 7 {
                t.set(x + 1, y + 1, PLUM);
                t.set(x + 2, y + 2, PLUM);
            }
            if hash2(x, y, seed) % 3 == 0 {
                x += if hash2(y, x, seed) % 2 == 0 { 1 } else { -1 };
            }
        }
    }
    for x in 0..16 {
        if x % 2 == 0 {
            t.set(x, 12, PURPLE);
        }
        t.set(x, 13, PURPLE);
        t.set(x, 14, LAVENDER);
        t.set(x, 15, INK);
    }
    t
}

/// A wing's bones: dark, lit along the front.
fn wing_bone() -> Texture {
    let mut t = Texture::new(16, 16, INK);
    for y in 0..16 {
        for x in 0..16 {
            match x {
                0..=2 | 14 | 15 => t.set(x, y, PURPLE),
                3 | 4 | 12 | 13 => t.set(x, y, GRAPE),
                _ => {}
            }
        }
    }
    t
}

/// Bone and horn, pale at the tip and darkening to the root (for `spike`).
fn spike_tex() -> Texture {
    let mut t = Texture::new(16, 16, ROSEWOOD);
    for y in 0..16 {
        let c = match y {
            0..=3 => CREAM,
            4..=8 => SAND,
            9..=12 => KHAKI,
            _ => ROSEWOOD,
        };
        for x in 0..16 {
            t.set(x, y, c);
        }
    }
    for y in 3..13 {
        t.set(12, y, if y < 9 { CREAM } else { SAND });
    }
    t
}

/// A horn swept along a curve (for `tube`, whose texture runs at world density from the
/// tip): pale at the tip, darkening to the root within a few texels.
fn horn_tex() -> Texture {
    let mut t = Texture::new(16, 16, ROSEWOOD);
    for (y, c) in [(0, CREAM), (1, SAND), (2, KHAKI), (3, KHAKI)] {
        for x in 0..16 {
            t.set(x, y, c);
        }
    }
    for y in 1..4 {
        t.set(3, y, SAND);
    }
    t
}

/// The slug's dark, slimy skin, spotted violet, with a glistening line along it.
fn slug_skin() -> Texture {
    let mut t = Texture::new(16, 16, INK);
    for y in 0..16 {
        for x in 0..16 {
            match hash2(x / 2, y / 2, 0x51) % 7 {
                0 => t.set(x, y, GRAPE),
                1 if (x + y) % 2 == 0 => t.set(x, y, SLATE),
                _ => {}
            }
        }
    }
    for x in 0..16 {
        t.set(x, 2, PURPLE);
    }
    t
}

/// A bat's wing, spreading out along +x from its root at the origin, its skin facing
/// forwards and back: the arm bowing up to the elbow and on to the wrist, a hooked thumb
/// there, and four long fingers fanning out from it, curving back towards their tips, with
/// the skin stretched between them sagging a little and scalloped between the tips, and on
/// back from the last finger to the body.
fn bat_wing(bank: &mut TexBank) -> Mesh {
    let skin = bank.add(membrane());
    let bone = bank.add(wing_bone());
    let claw = bank.add(spike_tex());
    let mut m = Mesh::new();
    let wrist = v(0.19, 0.12, -0.06);
    let arm = [
        Vec3::ZERO,
        v(0.045, 0.047, -0.015),
        v(0.095, 0.078, -0.035),
        v(0.145, 0.108, -0.05),
        wrist,
    ];
    tube(
        &mut m,
        &arm,
        &[0.022, 0.02, 0.019, 0.015, 0.014],
        5,
        bone,
        Vec3::Z,
    );
    knob(&mut m, arm[2], 0.021, bone);
    knob(&mut m, wrist, 0.018, bone);
    spike(
        &mut m,
        wrist + v(0.0, 0.012, 0.004),
        v(-0.25, 1.0, 0.45),
        0.055,
        0.012,
        claw,
    );
    // The fingers, each bowing out towards the leading edge so its tip sweeps back, the
    // first ending in a little claw.
    const ALONG: [f32; 5] = [0.0, 0.3, 0.55, 0.8, 1.0];
    let tips = [
        v(0.45, 0.15, -0.1),
        v(0.44, -0.02, -0.11),
        v(0.35, -0.16, -0.09),
        v(0.2, -0.22, -0.06),
    ];
    let fingers: Vec<[Vec3; 5]> = tips
        .iter()
        .enumerate()
        .map(|(i, &tip)| {
            let d = tip - wrist;
            let lead = v(-d.y, d.x, 0.0).normalize_or_zero();
            let bow = [0.032, 0.028, 0.024, 0.018][i];
            ALONG.map(|t| wrist + d * t + lead * (bow * 4.0 * t * (1.0 - t)))
        })
        .collect();
    for (i, f) in fingers.iter().enumerate() {
        let thick = [0.012, 0.011, 0.0105, 0.01][i];
        // A knuckle standing out a little, a third of the way along.
        let radii: [f32; 5] = std::array::from_fn(|k| {
            thick * (1.0 - ALONG[k] * 0.62) + [0.0, 0.002, 0.0, 0.0, 0.0][k]
        });
        tube(&mut m, f, &radii, 4, bone, Vec3::Z);
    }
    let d = fingers[0][4] - fingers[0][3];
    spike(&mut m, fingers[0][4], d, 0.035, 0.007, claw);
    // The skin: between each finger and the next, then from the last back to the body.
    let body = v(0.015, -0.13, 0.0);
    let to_body = ALONG.map(|t| wrist.lerp(body, t));
    let mut panels: Vec<([Vec3; 5], [Vec3; 5], f32)> =
        fingers.windows(2).map(|w| (w[0], w[1], 0.2)).collect();
    panels.push((fingers[3], to_body, 0.17));
    for (a, b, dip) in panels {
        const COLS: usize = 4;
        let edge = |c: usize| {
            let f = c as f32 / COLS as f32;
            let chord = a[4].lerp(b[4], f);
            chord + (wrist - chord) * (dip * (f * std::f32::consts::PI).sin())
        };
        let grid: Vec<Vec<(Vec3, Vec2)>> = (0..5)
            .map(|r| {
                (0..=COLS)
                    .map(|c| {
                        let f = c as f32 / COLS as f32;
                        let chord = a[4].lerp(b[4], f);
                        let pull = (edge(c) - chord) * (ALONG[r] * ALONG[r]);
                        let sag = -0.02 * (f * std::f32::consts::PI).sin() * ALONG[r];
                        (
                            a[r].lerp(b[r], f) + pull + Vec3::Z * sag,
                            Vec2::new(f * 16.0, ALONG[r] * 16.0),
                        )
                    })
                    .collect()
            })
            .collect();
        for r in 0..4 {
            for c in 0..COLS {
                let (p00, p01) = (grid[r][c], grid[r][c + 1]);
                let (p10, p11) = (grid[r + 1][c], grid[r + 1][c + 1]);
                if r > 0 {
                    m.tri([p00.0, p01.0, p11.0], [p00.1, p01.1, p11.1], skin);
                }
                m.tri([p00.0, p11.0, p10.0], [p00.1, p11.1, p10.1], skin);
            }
        }
    }
    // And between the arm and the body, back to where the skin meets it.
    for (k, pair) in arm.windows(2).enumerate() {
        let (u0, u1) = (k as f32 * 4.0, (k + 1) as f32 * 4.0);
        m.tri(
            [body, pair[0], pair[1]],
            [Vec2::new(8.0, 13.0), Vec2::new(u0, 1.0), Vec2::new(u1, 1.0)],
            skin,
        );
    }
    m
}

fn gazer(bank: &mut TexBank) -> Gazer {
    let skin = bank.add(hide());
    let white = bank.add(eye_white());
    let lid_t = bank.add(Texture::new(4, 4, GRAPE));
    let lash = bank.add(Texture::new(4, 4, INK));
    let horn = bank.add(horn_tex());
    let mut body = Mesh::new();
    ball(&mut body, Vec3::ZERO, Vec3::splat(GAZER_R), skin);
    // The great eye bulging out in front, looking up a little, and a lower lid narrowing it.
    // The eye's parts are made looking straight ahead, then tipped up about its middle.
    let tilt = Mat4::from_translation(GAZER_EYE_C)
        * Mat4::from_rotation_x(-EYE_TILT)
        * Mat4::from_translation(-GAZER_EYE_C);
    ball(&mut body, GAZER_EYE_C, Vec3::splat(GAZER_EYE_R), white);
    let lid_r = GAZER_EYE_R + 0.018;
    let span = (-lid_r * 0.98, lid_r * 0.98);
    let angle = move |y: f32, x: f32| {
        let rho = (lid_r * lid_r - x * x).max(1e-6).sqrt();
        (y / rho).clamp(-1.0, 1.0).asin()
    };
    let mut lower = Mesh::new();
    skin_strip(
        &mut lower,
        GAZER_EYE_C,
        lid_r,
        span,
        &|_| -2.2,
        &move |x| angle(-0.075 - 0.03 * (x / 0.17).powi(2), x),
        (lid_t, lash, true),
    );
    body.append(&lower, tilt);
    // Little horns, curling back.
    for sx in [-1.0f32, 1.0] {
        tube(
            &mut body,
            &[
                v(sx * 0.1, 0.19, -0.02),
                v(sx * 0.13, 0.27, -0.04),
                v(sx * 0.15, 0.31, -0.09),
                v(sx * 0.15, 0.3, -0.15),
            ],
            &[0.03, 0.022, 0.014, 0.004],
            6,
            horn,
            Vec3::Z,
        );
    }
    // The iris, on the front of the eyeball, laid on flat.
    let iris_t = bank.add(iris());
    let mut flat = Mesh::new();
    let (ir, half, n) = (GAZER_EYE_R + 0.005, 0.085f32, 6);
    let at = |x: f32, y: f32| GAZER_EYE_C + v(x, y, (ir * ir - x * x - y * y).max(0.0).sqrt());
    for j in 0..n {
        for i in 0..n {
            let (xa, xb) = (
                -half + 2.0 * half * i as f32 / n as f32,
                -half + 2.0 * half * (i + 1) as f32 / n as f32,
            );
            let (ya, yb) = (
                -half + 2.0 * half * j as f32 / n as f32,
                -half + 2.0 * half * (j + 1) as f32 / n as f32,
            );
            let tex = |x: f32, y: f32| ((x / half + 1.0) * 8.0, (1.0 - y / half) * 8.0);
            let (u0, v1) = tex(xa, ya);
            let (u1, v0) = tex(xb, yb);
            flat.quad(
                [at(xa, ya), at(xb, ya), at(xb, yb), at(xa, yb)],
                UvRect::new(u0, v0, u1, v1),
                iris_t,
            );
        }
    }
    let mut iris = Mesh::new();
    iris.append(&flat, tilt);
    // The upper lid, scowling down to a point over the middle of the eye, cutting into the
    // top of its iris.
    let mut upper = Mesh::new();
    skin_strip(
        &mut upper,
        GAZER_EYE_C,
        lid_r,
        span,
        &move |x| angle(0.05 + 0.5 * x.abs(), x),
        &|_| 2.3,
        (lid_t, lash, false),
    );
    let mut lid = Mesh::new();
    lid.append(&upper, tilt);
    Gazer {
        body,
        iris,
        lid,
        wing: bat_wing(bank),
        skin,
    }
}

fn slug(bank: &mut TexBank) -> Slug {
    let skin = bank.add(slug_skin());
    let spine = bank.add(spike_tex());
    let skirt = bank.add({
        let mut t = Texture::new(16, 16, PURPLE);
        for x in 0..16 {
            t.set(x, 0, GRAPE);
            t.set(x, 1, if x % 2 == 0 { GRAPE } else { PURPLE });
            t.set(x, 15, GRAPE);
        }
        t
    });
    let collar = bank.add(Texture::new(4, 4, GRAPE));
    let mut body = Mesh::new();
    // Long and low: a lathe laid along z, flattened.
    let mut b = Mesh::new();
    lathe(&mut b, Vec3::ZERO, &SLUG_BODY, 9, 0.0, skin, false);
    body.append(
        &b,
        Mat4::from_translation(v(0.0, SLUG_MID_Y, 0.0))
            * Mat4::from_rotation_x(std::f32::consts::FRAC_PI_2)
            * Mat4::from_scale(v(1.0, 1.0, SLUG_FLAT)),
    );
    // Its head, lifted a little at the front.
    ball(&mut body, v(0.0, 0.14, 0.33), v(0.12, 0.1, 0.13), skin);
    // Its skirt, a violet fringe spreading out round it on the ground.
    ball(&mut body, v(0.0, 0.03, -0.03), v(0.215, 0.03, 0.49), skirt);
    // Bony spines down its back, biggest in the middle, leaning back, each rising from a
    // dark collar; and smaller ones along its flanks, between them.
    for (k, &(z, len)) in SLUG_SPINES.iter().enumerate() {
        let y = slug_back(z) - 0.012;
        spike(
            &mut body,
            v(0.0, y, z),
            v(0.0, 1.0, -0.25),
            len,
            0.04,
            spine,
        );
        ball(&mut body, v(0.0, y, z), v(0.045, 0.018, 0.045), collar);
        if k + 1 < SLUG_SPINES.len() {
            let zf = (z + SLUG_SPINES[k + 1].0) * 0.5;
            for sx in [-1.0f32, 1.0] {
                let yf = slug_back(zf) - 0.05;
                spike(
                    &mut body,
                    v(sx * 0.11, yf, zf),
                    v(sx * 0.9, 0.75, -0.3),
                    len * 0.55,
                    0.024,
                    spine,
                );
            }
        }
    }
    // Two little feelers down in front.
    for sx in [-1.0f32, 1.0] {
        spike(
            &mut body,
            v(sx * 0.05, 0.06, 0.41),
            v(sx * 0.45, -0.25, 1.0),
            0.07,
            0.016,
            skin,
        );
    }
    let eye = bank.add(Texture::new(4, 4, CREAM));
    let pupil = bank.add(Texture::new(4, 4, INK));
    let mut stalk = Mesh::new();
    lathe(
        &mut stalk,
        Vec3::ZERO,
        &[(0.025, 0.0), (0.018, STALK_H)],
        5,
        0.0,
        skin,
        false,
    );
    ball(
        &mut stalk,
        v(0.0, STALK_H + 0.03, 0.0),
        v(0.04, 0.04, 0.04),
        eye,
    );
    skin_box(
        &mut stalk,
        v(-0.008, STALK_H + 0.008, 0.03),
        v(0.008, STALK_H + 0.052, 0.045),
        pupil,
        &Texture::new(4, 4, 0),
    );
    Slug { body, stalk }
}

pub fn build(bank: &mut TexBank) -> VoidArt {
    VoidArt {
        gazer: gazer(bank),
        slug: slug(bank),
    }
}

/// A gazer's lens: its eye's glassy green heart.
const LENS: &[&str] = &[
    "................",
    "................",
    ".....KKKKKK.....",
    "...KKwwwwwwKK...",
    "..KwwwtttwwwwK..",
    "..KwwtgggtwwwK..",
    ".KwwtgllKgtwwwK.",
    ".KwwtglKKgtwwwK.",
    ".KwwtglKKgtwwwK.",
    ".KwwtgggKgtwwwK.",
    "..KwwtgggtwwwK..",
    "..KrwwtttwwwrK..",
    "...KKrwwwwrKK...",
    ".....KKKKKK.....",
    "................",
    "................",
];

/// A spineback slug's spine: long, bone-white, curved.
const SPINE: &[&str] = &[
    "................",
    "..........KK....",
    ".........KwK....",
    "........KwwK....",
    ".......KwwnK....",
    "......KwwnK.....",
    ".....KwwnK......",
    ".....KwnnK......",
    "....KwnnK.......",
    "....KwnhK.......",
    "...KnnhK........",
    "...KnhhK........",
    "..KVhhK.........",
    "..KVVVK.........",
    "...KKK..........",
    "................",
];

/// An ogre's tusk: yellowed, chipped, dark at the root.
const TUSK: &[&str] = &[
    "................",
    "................",
    "...KK...........",
    "..KyK...........",
    "..KyyK..........",
    "..KnyyK.........",
    "...KnyyK........",
    "...KnnyyK.......",
    "....KnnyyK......",
    "....KhnnyyK.....",
    ".....KhnnyK.....",
    "......KhhnK.....",
    "......KRhhK.....",
    ".......KRRK.....",
    "........KK......",
    "................",
];

/// Icons for what the rift's creatures leave behind.
pub fn icons(bank: &mut TexBank, m: &mut HashMap<&'static str, TexId>) {
    use super::sprites::art;
    m.insert("gazer_lens", bank.add(art(LENS, [CLEAR; 4])));
    m.insert("slug_spine", bank.add(art(SPINE, [CLEAR; 4])));
    m.insert("ogre_tusk", bank.add(art(TUSK, [CLEAR; 4])));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_icons_are_tidy() {
        for rows in [LENS, SPINE, TUSK] {
            assert_eq!(rows.len(), 16);
            for r in rows {
                assert_eq!(r.chars().count(), 16, "{r}");
            }
        }
    }

    #[test]
    fn the_gazer_glares_through_a_round_iris_on_light_wings() {
        let t = iris();
        assert_eq!(t.data[0], CLEAR, "clear in the corners");
        assert_eq!(t.data[8 * 16 + 7], INK, "a slit of a pupil down the middle");
        let mut bank = TexBank::default();
        let wing = bat_wing(&mut bank);
        // Every gazer beats two of these, so they're kept light for the handhelds.
        assert!(wing.tris.len() < 500, "{} triangles", wing.tris.len());
        assert!(
            wing.verts.iter().all(|v| v.pos.x > -0.03),
            "it spreads out from its root"
        );
        let span = wing.verts.iter().map(|v| v.pos.x).fold(0.0, f32::max);
        assert!((0.4..0.5).contains(&span), "{span}");
        // The slug's spines stand on its back.
        for (z, _) in SLUG_SPINES {
            assert!(slug_back(z) > SLUG_MID_Y, "{z}");
        }
    }

    #[test]
    fn everything_stays_in_the_palette() {
        for t in [
            hide(),
            eye_white(),
            slug_skin(),
            iris(),
            membrane(),
            wing_bone(),
            spike_tex(),
            horn_tex(),
        ] {
            assert!(t.data.iter().all(|&c| c < 32 || c == CLEAR));
        }
    }
}
