//! The farm's pets and what goes with them: the cat, the jumping spider that hatches from
//! Pip's mysterious egg, the egg itself in its nest of leaves, the candy rocks that buy it,
//! and the puffs of smoke those burst up out of the floor in.
//!
//! The cat and the spider come in parts (body, head, legs, tail) posed by `game::pets`; the
//! joints they hang from are the constants below. Local space as everywhere: the ground under
//! the creature's middle, +y up, facing +z.

use std::collections::HashMap;

use glam::{Mat4, Vec2, Vec3};

use super::models::{lathe, skin_box};
use super::tiles;
use crate::palette::*;
use crate::render::{Mesh, TexBank, TexId, Texture};

/// The cat's body centre stands this high.
pub const CAT_BODY_Y: f32 = 0.2;
/// Where the head sits on the body (from the body's centre).
pub const CAT_NECK: Vec3 = Vec3::new(0.0, 0.11, 0.2);
/// Where the tail leaves the body.
pub const CAT_TAIL_BASE: Vec3 = Vec3::new(0.0, 0.06, -0.2);
/// The tops of the legs (front left, front right, back left, back right), from the body's
/// centre.
pub const CAT_HIPS: [Vec3; 4] = [
    Vec3::new(0.05, -0.02, 0.13),
    Vec3::new(-0.05, -0.02, 0.13),
    Vec3::new(0.056, -0.02, -0.13),
    Vec3::new(-0.056, -0.02, -0.13),
];
/// Hip to the sole of the paw.
pub const CAT_LEG: f32 = 0.18;
/// Each of the tail's segments.
pub const CAT_TAIL_SEG: f32 = 0.075;

/// The jumping spider's legs: thigh and shin lengths, and where the four pairs join its body
/// (how far forward, and how far each fans out from straight out to the side).
pub const SPIDER_THIGH: f32 = 0.12;
pub const SPIDER_SHIN: f32 = 0.16;
pub const SPIDER_HIP_X: f32 = 0.05;
pub const SPIDER_HIP_Y: f32 = 0.075;
pub const SPIDER_LEGS: [(f32, f32); 4] = [(0.085, 1.0), (0.06, 0.4), (0.035, -0.3), (0.01, -0.95)];
/// Where its pedipalps hang, either side of its fangs.
pub const SPIDER_PALP: Vec3 = Vec3::new(0.035, 0.07, 0.125);

pub struct PetArt {
    /// The cat's body, haunches and white bib.
    pub cat_body: Mesh,
    /// Its head: ears, white muzzle and pink nose (the eyes come separately).
    pub cat_head: Mesh,
    pub cat_eyes: Mesh,
    /// Happy (or sleepy) closed eyes.
    pub cat_eyes_shut: Mesh,
    /// A leg hanging down from its hip, a white paw at the bottom.
    pub cat_leg: Mesh,
    /// The tail from its base to its white tip, one mesh per segment.
    pub cat_tail: Vec<Mesh>,
    /// The spider's head and chest: big eyes, fangs and all.
    pub spider_head: Mesh,
    /// Its round abdomen, with a heart on it.
    pub spider_abdomen: Mesh,
    pub spider_thigh: Mesh,
    pub spider_shin: Mesh,
    /// The front pair are stouter, for waving.
    pub spider_front_thigh: Mesh,
    pub spider_front_shin: Mesh,
    pub spider_palp: Mesh,
    pub egg: Mesh,
    /// Nearly hatched: cracks with light showing through.
    pub egg_cracked: Mesh,
    /// The leaves it's tucked into.
    pub nest: Mesh,
    /// Candy rocks: strawberry, cherry, candy corn and grape.
    pub candy: Vec<Mesh>,
    /// Smoke: a fresh, hot puff and a drifting one.
    pub puff_hot: TexId,
    pub puff: TexId,
}

/// Single colours, one texture each.
struct Paint {
    solid: HashMap<u8, TexId>,
    w4: Texture,
}

impl Paint {
    fn c(&mut self, bank: &mut TexBank, c: u8) -> TexId {
        *self
            .solid
            .entry(c)
            .or_insert_with(|| bank.add(tiles::solid(c)))
    }

    fn bx(&mut self, bank: &mut TexBank, m: &mut Mesh, a: Vec3, b: Vec3, c: u8) {
        let t = self.c(bank, c);
        skin_box(m, a, b, t, &self.w4);
    }
}

/// A triangle facing away from `inside`.
fn facet(m: &mut Mesh, p: [Vec3; 3], inside: Vec3, tex: TexId) {
    let n = (p[1] - p[0]).cross(p[2] - p[0]);
    let mid = (p[0] + p[1] + p[2]) / 3.0;
    let uv = [
        Vec2::new(0.5, 0.5),
        Vec2::new(3.5, 0.5),
        Vec2::new(2.0, 3.5),
    ];
    if n.dot(mid - inside) >= 0.0 {
        m.tri(p, uv, tex);
    } else {
        m.tri([p[0], p[2], p[1]], [uv[0], uv[2], uv[1]], tex);
    }
}

/// A round lump `r` across and `h` tall, standing at `at`.
fn blob(m: &mut Mesh, at: Vec3, r: f32, h: f32, seg: usize, tex: TexId) {
    lathe(
        m,
        at,
        &[
            (0.0, 0.0),
            (r * 0.7, h * 0.08),
            (r, h * 0.4),
            (r * 0.85, h * 0.75),
            (r * 0.45, h * 0.95),
            (0.0, h),
        ],
        seg,
        0.2,
        tex,
        false,
    );
}

/// A lathe turned on its side: `prof` runs along +z from `z0` (a profile's y becomes z).
fn along_z(m: &mut Mesh, prof: &[(f32, f32)], seg: usize, tex: TexId, place: Mat4) {
    let mut part = Mesh::new();
    lathe(&mut part, Vec3::ZERO, prof, seg, 0.0, tex, false);
    m.append(
        &part,
        place * Mat4::from_rotation_x(std::f32::consts::FRAC_PI_2),
    );
}

// ------------------------------------------------------------------------------------------
// The cat
// ------------------------------------------------------------------------------------------

/// A ginger tabby's coat, wrapped once round the body: stripes over the back and sides, a
/// white belly and chest underneath. Columns go round (the back at 8, the belly at 24), rows
/// run from the shoulders back to the rump.
fn coat() -> Texture {
    let mut t = Texture::new(32, 16, GOLD);
    for v in 0..16 {
        for u in 0..32 {
            let belly = (19..=29).contains(&u);
            let c = if belly {
                WHITE
            } else if v % 3 == 1 && !(17..=31).contains(&u) {
                // Stripes over the back and down the sides, thinning out.
                if (5..=11).contains(&u) { RUST } else { CLAY }
            } else if (7..=9).contains(&u) && v % 3 == 2 {
                CLAY
            } else if (u + v * 5) % 13 == 0 {
                CREAM
            } else {
                GOLD
            };
            t.set(u, v, c);
        }
    }
    t
}

/// Ginger fur with a darker fleck here and there, for the head, legs and haunches.
fn fur() -> Texture {
    let mut t = Texture::new(16, 16, GOLD);
    for v in 0..16 {
        for u in 0..16 {
            if (u * 5 + v * 3) % 11 == 0 {
                t.set(u, v, CLAY);
            } else if (u * 3 + v * 7) % 17 == 0 {
                t.set(u, v, CREAM);
            }
        }
    }
    t
}

/// The cat's head, which faces +z from its middle: the ginger crown with a tabby's mark on
/// the brow, pointed ears with pink insides, and a white muzzle with a pink nose.
fn cat_head(bank: &mut TexBank, p: &mut Paint, fur_t: TexId) -> Mesh {
    let mut m = Mesh::new();
    // The skull: wider than it is tall, a little flat at the back.
    let mut skull = Mesh::new();
    lathe(
        &mut skull,
        Vec3::ZERO,
        &[
            (0.0, -0.095),
            (0.075, -0.085),
            (0.115, -0.045),
            (0.128, 0.005),
            (0.115, 0.055),
            (0.075, 0.09),
            (0.0, 0.105),
        ],
        10,
        0.0,
        fur_t,
        false,
    );
    m.append(&skull, Mat4::from_scale(Vec3::new(1.08, 1.0, 0.94)));
    // The tabby's "M" on the brow: three short dark stripes.
    for (x, h) in [(-0.03f32, 0.028f32), (0.0, 0.036), (0.03, 0.028)] {
        p.bx(
            bank,
            &mut m,
            Vec3::new(x - 0.008, 0.05, 0.09),
            Vec3::new(x + 0.008, 0.05 + h, 0.105),
            RUST,
        );
    }
    // A white muzzle with round cheeks, a white chin, and a pink nose on top.
    let white = p.c(bank, WHITE);
    let mut cheeks = Mesh::new();
    for sx in [-1.0f32, 1.0] {
        blob(
            &mut cheeks,
            Vec3::new(sx * 0.03, -0.075, 0.075),
            0.042,
            0.066,
            7,
            white,
        );
    }
    m.append(&cheeks, Mat4::IDENTITY);
    p.bx(
        bank,
        &mut m,
        Vec3::new(-0.03, -0.098, 0.06),
        Vec3::new(0.03, -0.07, 0.1),
        WHITE,
    );
    p.bx(
        bank,
        &mut m,
        Vec3::new(-0.015, -0.03, 0.108),
        Vec3::new(0.015, -0.012, 0.124),
        PINK,
    );
    // Ears: a little pyramid each, pink inside, tipped out a touch.
    let fur_c = fur_t;
    let pink = p.c(bank, SALMON);
    for s in [-1.0f32, 1.0] {
        let a = Vec3::new(s * 0.03, 0.075, 0.03);
        let b = Vec3::new(s * 0.118, 0.035, 0.028);
        let c = Vec3::new(s * 0.074, 0.065, -0.045);
        let d = Vec3::new(s * 0.095, 0.185, 0.0);
        let inside = (a + b + c) / 3.0 + Vec3::new(0.0, 0.02, 0.0);
        facet(&mut m, [a, b, d], inside, pink);
        facet(&mut m, [b, c, d], inside, fur_c);
        facet(&mut m, [c, a, d], inside, fur_c);
    }
    m
}

/// Eyes on the cat's face: open (dark and round), or shut in a contented line.
fn cat_eyes(bank: &mut TexBank, p: &mut Paint, shut: bool) -> Mesh {
    let mut m = Mesh::new();
    for s in [-1.0f32, 1.0] {
        let x = s * 0.052;
        if shut {
            p.bx(
                bank,
                &mut m,
                Vec3::new(x - 0.026, 0.004, 0.1),
                Vec3::new(x + 0.026, 0.016, 0.118),
                INK,
            );
        } else {
            p.bx(
                bank,
                &mut m,
                Vec3::new(x - 0.02, -0.014, 0.1),
                Vec3::new(x + 0.02, 0.034, 0.118),
                INK,
            );
            // A green glint in each.
            p.bx(
                bank,
                &mut m,
                Vec3::new(x - s * 0.012 - 0.007, 0.012, 0.116),
                Vec3::new(x - s * 0.012 + 0.007, 0.028, 0.122),
                LIME,
            );
        }
    }
    m
}

/// A foreleg or hind leg hanging from its hip: a little thicker at the top, and a white
/// paw poking forwards at the bottom.
fn cat_leg(bank: &mut TexBank, p: &mut Paint, fur_t: TexId) -> Mesh {
    let mut m = Mesh::new();
    let w4 = Texture::new(4, 4, 0);
    skin_box(
        &mut m,
        Vec3::new(-0.025, -0.09, -0.025),
        Vec3::new(0.025, 0.01, 0.025),
        fur_t,
        &w4,
    );
    skin_box(
        &mut m,
        Vec3::new(-0.02, -0.16, -0.02),
        Vec3::new(0.02, -0.085, 0.02),
        fur_t,
        &w4,
    );
    p.bx(
        bank,
        &mut m,
        Vec3::new(-0.025, -CAT_LEG, -0.026),
        Vec3::new(0.025, -0.15, 0.034),
        WHITE,
    );
    m
}

fn cat_body(bank: &mut TexBank, p: &mut Paint, fur_t: TexId) -> Mesh {
    let mut m = Mesh::new();
    let coat_t = bank.add(coat());
    // A long body, deeper at the chest than the waist, lying along z round its centre.
    along_z(
        &mut m,
        &[
            (0.0, 0.0),
            (0.07, 0.015),
            (0.098, 0.07),
            (0.1, 0.16),
            (0.092, 0.25),
            (0.098, 0.32),
            (0.075, 0.385),
            (0.0, 0.41),
        ],
        10,
        coat_t,
        Mat4::from_translation(Vec3::new(0.0, 0.0, -0.205))
            * Mat4::from_scale(Vec3::new(0.86, 1.0, 1.0)),
    );
    // Round haunches over the hind legs.
    for s in [-1.0f32, 1.0] {
        blob(
            &mut m,
            Vec3::new(s * 0.062, -0.1, -0.12),
            0.058,
            0.15,
            8,
            fur_t,
        );
    }
    // A white bib at the front of the chest.
    let white = p.c(bank, WHITE);
    blob(&mut m, Vec3::new(0.0, -0.1, 0.15), 0.06, 0.13, 8, white);
    m
}

/// The tail in five segments, each hanging back along -z from its joint: ginger with a
/// darker ring, and a white tip.
fn cat_tail(bank: &mut TexBank, p: &mut Paint) -> Vec<Mesh> {
    let cols = [GOLD, CLAY, GOLD, CLAY, WHITE];
    cols.iter()
        .enumerate()
        .map(|(i, &c)| {
            let t = 0.022 - i as f32 * 0.0015;
            let mut m = Mesh::new();
            p.bx(
                bank,
                &mut m,
                Vec3::new(-t, -t, -CAT_TAIL_SEG - 0.008),
                Vec3::new(t, t, 0.004),
                c,
            );
            m
        })
        .collect()
}

// ------------------------------------------------------------------------------------------
// The jumping spider
// ------------------------------------------------------------------------------------------

/// Soft fuzz in three shades.
fn fuzz(c: [u8; 3]) -> Texture {
    let mut t = Texture::new(8, 8, c[1]);
    for v in 0..8 {
        for u in 0..8 {
            let n = crate::util::hash2(u, v, 23) % 100;
            if n < 18 {
                t.set(u, v, c[0]);
            } else if n < 30 {
                t.set(u, v, c[2]);
            }
        }
    }
    t
}

/// The abdomen's coat, much finer than usual so the heart shows: 64 columns round (the top
/// at 48) and 96 rows a unit along, from the tail end (row 0) to the front (row 21). Dark
/// fuzz, a white band where it joins the chest, and a big pink heart over its back, lobes
/// towards the tail so it's the right way up seen from in front.
fn heart_back() -> Texture {
    let mut t = Texture::new(64, 32, SHADOW);
    for v in 0..32 {
        for u in 0..64 {
            if crate::util::hash2(u, v, 17) % 100 < 12 {
                t.set(u, v, INK);
            }
        }
    }
    for v in 19..22 {
        for u in 34..=62 {
            t.set(u, v, WHITE);
        }
    }
    let heart = [
        "..XXX...XXX..",
        ".XXXXX.XXXXX.",
        "XXXXXXXXXXXXX",
        "XXXXXXXXXXXXX",
        "XXXXXXXXXXXXX",
        ".XXXXXXXXXXX.",
        "..XXXXXXXXX..",
        "...XXXXXXX...",
        "....XXXXX....",
        ".....XXX.....",
        "......X......",
    ];
    for (row, line) in heart.iter().enumerate() {
        for (col, ch) in line.chars().enumerate() {
            if ch == 'X' {
                t.set(42 + col as i32, 5 + row as i32, PINK);
            }
        }
    }
    for (u, v) in [(44, 6), (45, 6), (43, 7), (44, 7), (43, 8)] {
        t.set(u, v, BLUSH);
    }
    t
}

fn spider_head(bank: &mut TexBank, p: &mut Paint) -> Mesh {
    let mut m = Mesh::new();
    let head_t = bank.add(fuzz([KHAKI, ROSEWOOD, SHADOW]));
    // A boxy head-and-chest, longer than wide, sitting up off the ground.
    let mut part = Mesh::new();
    lathe(
        &mut part,
        Vec3::ZERO,
        &[
            (0.0, 0.0),
            (0.07, 0.006),
            (0.088, 0.04),
            (0.086, 0.08),
            (0.068, 0.108),
            (0.0, 0.116),
        ],
        8,
        0.39,
        head_t,
        false,
    );
    m.append(
        &part,
        Mat4::from_translation(Vec3::new(0.0, 0.05, 0.035))
            * Mat4::from_scale(Vec3::new(0.95, 1.0, 1.08)),
    );
    // A fluffy white fringe round the mouth, under the eyes, and a pale mask the eyes sit in.
    let white = p.c(bank, WHITE);
    blob(&mut m, Vec3::new(0.0, 0.052, 0.098), 0.05, 0.05, 7, white);
    p.bx(
        bank,
        &mut m,
        Vec3::new(-0.085, 0.078, 0.112),
        Vec3::new(0.085, 0.162, 0.12),
        PEACH,
    );
    for s in [-1.0f32, 1.0] {
        // The big front eyes, glossy black with a glint, and a smaller one beside each.
        let x = s * 0.037;
        p.bx(
            bank,
            &mut m,
            Vec3::new(x - 0.035, 0.085, 0.118),
            Vec3::new(x + 0.035, 0.155, 0.134),
            INK,
        );
        p.bx(
            bank,
            &mut m,
            Vec3::new(x - 0.022, 0.128, 0.132),
            Vec3::new(x - 0.004, 0.146, 0.138),
            WHITE,
        );
        p.bx(
            bank,
            &mut m,
            Vec3::new(s * 0.075 - 0.012, 0.12, 0.1),
            Vec3::new(s * 0.075 + 0.012, 0.144, 0.116),
            INK,
        );
        // Tufty eyebrows.
        p.bx(
            bank,
            &mut m,
            Vec3::new(x - 0.03, 0.155, 0.108),
            Vec3::new(x + 0.022, 0.17, 0.124),
            WHITE,
        );
        // Shiny teal fangs under the fringe.
        p.bx(
            bank,
            &mut m,
            Vec3::new(s * 0.017 - 0.013, 0.05, 0.128),
            Vec3::new(s * 0.017 + 0.013, 0.078, 0.142),
            AQUA,
        );
        p.bx(
            bank,
            &mut m,
            Vec3::new(s * 0.017 - 0.005, 0.066, 0.141),
            Vec3::new(s * 0.017 + 0.004, 0.076, 0.145),
            MINT,
        );
    }
    m
}

fn spider_abdomen(bank: &mut TexBank) -> Mesh {
    let mut m = Mesh::new();
    let back = bank.add(heart_back());
    // Round and plump, lying back along -z from where it joins the chest, wrapped in its
    // fine coat (see `heart_back`).
    let prof: [(f32, f32); 7] = [
        (0.0, 0.0),
        (0.06, 0.01),
        (0.092, 0.05),
        (0.1, 0.1),
        (0.094, 0.15),
        (0.068, 0.2),
        (0.0, 0.225),
    ];
    let top = 0.225;
    let rows: Vec<(f32, f32, f32)> = prof
        .iter()
        .map(|&(r, y)| (r, y, (top - y) * 96.0))
        .collect();
    let mut part = Mesh::new();
    part.lathe(Vec3::ZERO, &rows, 12, 64.0, 0.0, back, (None, None));
    m.append(
        &part,
        Mat4::from_scale(Vec3::new(1.0, 0.86, 1.0))
            * Mat4::from_rotation_x(-std::f32::consts::FRAC_PI_2),
    );
    m
}

/// A leg segment `len` long along +x and `t` thick, tapering, in fuzzy `c[0]` with a pale
/// `c[1]` ring towards its far end, a knuckle at its joint and (for a shin) a pale tip.
fn leg_part(bank: &mut TexBank, p: &mut Paint, len: f32, t: f32, c: [u8; 2], tip: bool) -> Mesh {
    let mut m = Mesh::new();
    p.bx(
        bank,
        &mut m,
        Vec3::new(0.0, -t, -t),
        Vec3::new(len * 0.5, t, t),
        c[0],
    );
    let w = t * 0.85;
    p.bx(
        bank,
        &mut m,
        Vec3::new(len * 0.5, -w, -w),
        Vec3::new(len, w, w),
        c[0],
    );
    let r = t * 1.05;
    p.bx(
        bank,
        &mut m,
        Vec3::new(len * 0.62, -r, -r),
        Vec3::new(len * 0.76, r, r),
        c[1],
    );
    p.bx(
        bank,
        &mut m,
        Vec3::splat(-t * 1.2),
        Vec3::splat(t * 1.2),
        c[0],
    );
    if tip {
        p.bx(
            bank,
            &mut m,
            Vec3::new(len - 0.02, -t * 0.8, -t * 0.8),
            Vec3::new(len + 0.01, t * 0.8, t * 0.8),
            c[1],
        );
    }
    m
}

/// A pedipalp: a fuzzy little arm hanging from its joint, with a pale mitten at the end.
fn spider_palp(bank: &mut TexBank, p: &mut Paint) -> Mesh {
    let mut m = Mesh::new();
    p.bx(
        bank,
        &mut m,
        Vec3::new(-0.01, -0.035, -0.01),
        Vec3::new(0.01, 0.0, 0.012),
        KHAKI,
    );
    p.bx(
        bank,
        &mut m,
        Vec3::new(-0.014, -0.058, -0.012),
        Vec3::new(0.014, -0.032, 0.018),
        WHITE,
    );
    m
}

// ------------------------------------------------------------------------------------------
// The egg, and the candy rocks
// ------------------------------------------------------------------------------------------

/// Pale violet with darker speckles, and a few spots of mint that glow after dark.
fn egg_shell(cracked: bool) -> Texture {
    let mut t = Texture::new(16, 8, LAVENDER);
    for v in 0..8 {
        for u in 0..16 {
            let n = crate::util::hash2(u, v, 41) % 100;
            if v == 0 && u % 3 != 0 {
                t.set(u, v, BLUSH);
            } else if n < 12 {
                t.set(u, v, PURPLE);
            } else if n < 17 && v > 1 {
                t.set(u, v, GRAPE);
            }
        }
    }
    for (u, v) in [(3, 2), (11, 4), (7, 5), (14, 2)] {
        t.set(u, v, MINT);
    }
    if cracked {
        // A jagged crack round its middle, a little light leaking out.
        let ys = [3, 3, 2, 2, 3, 4, 4, 3, 3, 2, 3, 4, 3, 3, 2, 3];
        for (u, &v) in ys.iter().enumerate() {
            t.set(u as i32, v, INK);
        }
        for u in [2, 9] {
            t.set(u, ys[u as usize] + 1, MINT);
        }
    }
    t
}

fn egg(bank: &mut TexBank, cracked: bool) -> Mesh {
    let tex = bank.add(egg_shell(cracked));
    let mut m = Mesh::new();
    lathe(
        &mut m,
        Vec3::ZERO,
        &[
            (0.0, 0.0),
            (0.085, 0.012),
            (0.13, 0.06),
            (0.145, 0.13),
            (0.135, 0.2),
            (0.105, 0.27),
            (0.06, 0.325),
            (0.0, 0.35),
        ],
        10,
        0.3,
        tex,
        false,
    );
    m
}

/// A ring of fallen leaves and a few twigs round where the egg sits.
fn nest(bank: &mut TexBank, p: &mut Paint) -> Mesh {
    let mut m = Mesh::new();
    let cols = [ORANGE, GOLD, RUST, CLAY, ORANGE, RUST, GOLD, CLAY, RED];
    for (k, &c) in cols.iter().enumerate() {
        let tex = p.c(bank, c);
        let a = k as f32 / cols.len() as f32 * std::f32::consts::TAU + (k % 2) as f32 * 0.3;
        let r = 0.1 + (k % 3) as f32 * 0.035;
        let dir = Vec3::new(a.cos(), 0.0, a.sin());
        let side = Vec3::new(-a.sin(), 0.0, a.cos());
        let base = dir * r + Vec3::Y * (0.012 + (k % 2) as f32 * 0.01);
        let len = 0.14 + (k % 4) as f32 * 0.015;
        // A diamond leaf lying on the ground, pointing outwards, its tip curling up.
        let tip = base + dir * len + Vec3::Y * 0.03;
        let mid = base + dir * (len * 0.5);
        let l = mid + side * 0.045;
        let rr = mid - side * 0.045;
        let up = Vec3::new(0.0, -1.0, 0.0) + base;
        facet(&mut m, [base, l, tip], up, tex);
        facet(&mut m, [base, tip, rr], up, tex);
    }
    for (a, len) in [(0.4f32, 0.36f32), (2.1, 0.32), (4.0, 0.34)] {
        let d = Vec3::new(a.cos(), 0.0, a.sin());
        let mut twig = Mesh::new();
        p.bx(
            bank,
            &mut twig,
            Vec3::new(-len * 0.5, 0.0, -0.008),
            Vec3::new(len * 0.5, 0.016, 0.008),
            RUST,
        );
        m.append(
            &twig,
            Mat4::from_translation(d * 0.02) * Mat4::from_rotation_y(-a),
        );
    }
    m
}

/// Bold candy-cane stripes twisting round: white, and the flavour's colour with a lighter
/// edge (wide enough to see from across the room).
fn candy_stripes(c: [u8; 3]) -> Texture {
    let mut t = Texture::new(16, 16, c[0]);
    for v in 0..16 {
        for u in 0..16 {
            let k = (u + v) % 16;
            if k >= 8 {
                t.set(u, v, if k == 8 || k == 15 { c[1] } else { c[2] });
            }
        }
    }
    t
}

/// A cluster of rock-candy crystals growing out of a lump of the Hollow's floor.
fn candy_rock(bank: &mut TexBank, c: [u8; 3], seed: u32) -> Mesh {
    let mut m = Mesh::new();
    let stone = bank.add(fuzz([KHAKI, ROSEWOOD, SHADOW]));
    let sugar = bank.add(candy_stripes(c));
    lathe(
        &mut m,
        Vec3::ZERO,
        &[
            (0.0, 0.0),
            (0.22, 0.0),
            (0.2, 0.05),
            (0.13, 0.1),
            (0.0, 0.12),
        ],
        7,
        seed as f32,
        stone,
        false,
    );
    let crystals: [(f32, f32, f32, f32, f32, f32); 5] = [
        (0.0, 0.0, 0.0, 0.0, 0.44, 0.07),
        (0.09, 0.04, -0.5, 0.15, 0.3, 0.055),
        (-0.085, 0.05, 0.45, 0.2, 0.26, 0.05),
        (0.03, -0.09, -0.1, -0.5, 0.28, 0.052),
        (-0.04, 0.1, 0.25, 0.55, 0.2, 0.045),
    ];
    for (i, &(x, z, rz, rx, h, r)) in crystals.iter().enumerate() {
        let mut part = Mesh::new();
        lathe(
            &mut part,
            Vec3::ZERO,
            &[(r * 0.8, 0.0), (r, h * 0.2), (r, h * 0.74), (0.0, h)],
            6,
            (seed as usize + i) as f32 * 0.7,
            sugar,
            false,
        );
        m.append(
            &part,
            Mat4::from_translation(Vec3::new(x, 0.04, z))
                * Mat4::from_rotation_z(rz)
                * Mat4::from_rotation_x(rx),
        );
    }
    m
}

/// A soft round puff of smoke: `c` from the middle out, ragged at the edge.
fn puff(c: [u8; 3]) -> Texture {
    let mut t = Texture::clear(16, 16);
    for y in 0..16 {
        for x in 0..16 {
            let (dx, dy) = (x as f32 - 7.5, y as f32 - 7.5);
            let d = (dx * dx + dy * dy).sqrt();
            // Lit from the top left.
            let lit = d + (dx + dy) * 0.25;
            let col = if d > 7.6 || (d > 6.4 && (x + y) % 2 == 0) {
                continue;
            } else if lit < 3.2 {
                c[0]
            } else if lit < 6.2 {
                c[1]
            } else {
                c[2]
            };
            t.set(x, y, col);
        }
    }
    t
}

pub fn build(bank: &mut TexBank) -> PetArt {
    let mut p = Paint {
        solid: HashMap::new(),
        w4: Texture::new(4, 4, 0),
    };
    let fur_t = bank.add(fur());
    let banded = [SHADOW, SAND];
    let front = [INK, SAND];
    PetArt {
        cat_body: cat_body(bank, &mut p, fur_t),
        cat_head: cat_head(bank, &mut p, fur_t),
        cat_eyes: cat_eyes(bank, &mut p, false),
        cat_eyes_shut: cat_eyes(bank, &mut p, true),
        cat_leg: cat_leg(bank, &mut p, fur_t),
        cat_tail: cat_tail(bank, &mut p),
        spider_head: spider_head(bank, &mut p),
        spider_abdomen: spider_abdomen(bank),
        spider_thigh: leg_part(bank, &mut p, SPIDER_THIGH, 0.019, banded, false),
        spider_shin: leg_part(bank, &mut p, SPIDER_SHIN, 0.016, banded, true),
        spider_front_thigh: leg_part(bank, &mut p, SPIDER_THIGH, 0.025, front, false),
        spider_front_shin: leg_part(bank, &mut p, SPIDER_SHIN, 0.021, front, true),
        spider_palp: spider_palp(bank, &mut p),
        egg: egg(bank, false),
        egg_cracked: egg(bank, true),
        nest: nest(bank, &mut p),
        candy: vec![
            candy_rock(bank, [WHITE, BLUSH, PINK], 1),
            candy_rock(bank, [WHITE, SALMON, RED], 2),
            candy_rock(bank, [WHITE, GOLD, ORANGE], 3),
            candy_rock(bank, [WHITE, LAVENDER, PURPLE], 4),
        ],
        puff_hot: bank.add(puff([WHITE, CREAM, GOLD])),
        puff: bank.add(puff([WHITE, SKY, SLATE])),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pets_are_built_in_parts() {
        let mut bank = TexBank::default();
        let a = build(&mut bank);
        assert_eq!(a.cat_tail.len(), 5);
        assert_eq!(a.candy.len(), 4);
        for m in [
            &a.cat_body,
            &a.cat_head,
            &a.cat_eyes,
            &a.cat_eyes_shut,
            &a.cat_leg,
            &a.spider_head,
            &a.spider_abdomen,
            &a.egg,
            &a.nest,
        ] {
            assert!(!m.tris.is_empty());
        }
        // The legs reach the ground from their hips.
        let low = a
            .cat_leg
            .verts
            .iter()
            .map(|v| v.pos.y)
            .fold(f32::MAX, f32::min);
        assert!((low + CAT_LEG).abs() < 1e-4, "{low}");
        let hip = CAT_BODY_Y + CAT_HIPS[0].y;
        assert!(hip >= CAT_LEG, "legs {CAT_LEG} can't reach down from {hip}");
    }
}
