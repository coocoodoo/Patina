//! The Hollow's walking dead, goblins (fat and skinny), bugs, bones and ghosts. Every family
//! comes in six looks, one for each biome: mossy, crystal, fungal, ember, frost and ruins.

use std::f32::consts::PI;

use glam::{Mat4, Vec2, Vec3};

use super::models::{
    ARM_L, ARM_R, BODY, HEAD, HumanTex, Humanoid, LEG_L, LEG_R, lathe, skin_box, slime_skin,
    tiled_box,
};
use super::tiles;
use crate::palette::*;
use crate::render::{BoxUv, Mesh, TexBank, TexId, Texture, UvRect};

/// Looks per family: one for each biome.
pub const KINDS: usize = 6;

/// A bug: a body, one leg (drawn at every hip, skittering), and wings or claws for some.
pub struct Bug {
    pub body: Mesh,
    pub leg: Mesh,
    /// Moths have wings, drawn on each side and flapping.
    pub wing: Option<Mesh>,
    /// Mantises have a pair of folding claws up front.
    pub claw: Option<Mesh>,
    /// Where each pair of legs meets the body, front to back (z), how far out (x) and up.
    pub hips: Vec<f32>,
    pub hip_x: f32,
    pub hip_y: f32,
    /// How long a leg is, out from the hip.
    pub leg_len: f32,
    /// Flies above the ground instead of scuttling.
    pub flies: bool,
}

pub struct Monsters {
    /// Humanoid families, by biome.
    pub zombie: Vec<Humanoid>,
    /// Fat goblins, and their clubs.
    pub brute: Vec<Humanoid>,
    pub club: Vec<Mesh>,
    /// Skinny goblins, and their daggers.
    pub sneak: Vec<Humanoid>,
    pub dagger: Vec<Mesh>,
    pub skeleton: Vec<Humanoid>,
    /// Ghosts: a sheet per biome, one face, and something on each one's head.
    pub ghost: Vec<Mesh>,
    pub ghost_face: Mesh,
    pub ghost_hat: Vec<Mesh>,
    pub bug: Vec<Bug>,
}

fn v(x: f32, y: f32, z: f32) -> Vec3 {
    Vec3::new(x, y, z)
}

/// A shared cache of plain colours.
struct Kit {
    solid: std::collections::HashMap<u8, TexId>,
    w4: Texture,
}

impl Kit {
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

/// A 4x4 speckled texture: highlight, base and shadow.
fn speck(bank: &mut TexBank, c: [u8; 3]) -> TexId {
    let mut t = Texture::new(4, 4, c[1]);
    t.set(0, 0, c[0]);
    t.set(2, 1, c[0]);
    t.set(1, 3, c[2]);
    t.set(3, 2, c[2]);
    bank.add(t)
}

/// Paints rows of characters into a texture at (x0, y0) with a legend.
fn paint(t: &mut Texture, x0: i32, y0: i32, rows: &[&str], legend: &[(char, u8)]) {
    for (y, row) in rows.iter().enumerate() {
        for (x, ch) in row.chars().enumerate() {
            if let Some(&(_, c)) = legend.iter().find(|(k, _)| *k == ch) {
                t.set(x0 + x as i32, y0 + y as i32, c);
            }
        }
    }
}

/// A head texture in the humanoid layout: an 8x7 face, sides, back, top and bottom.
fn head_tex(
    face: &[&str],
    legend: &[(char, u8)],
    side: u8,
    back: u8,
    top: u8,
    under: u8,
) -> Texture {
    let mut t = Texture::new(32, 16, side);
    paint(&mut t, 0, 0, face, legend);
    for y in 0..7 {
        for x in 0..8 {
            t.set(15 + x, y, if (x + y) % 5 == 0 { top } else { back });
            t.set(x, 7 + y, if (x * 3 + y) % 7 == 0 { back } else { top });
            t.set(8 + x, 7 + y, under);
        }
    }
    t
}

/// A body texture in the humanoid layout from a 6x5 front picture; the back and sides take
/// `back`, with a band of `belt` low down.
fn body_tex(front: &[&str], legend: &[(char, u8)], back: u8, belt: u8, top: u8) -> Texture {
    let mut t = Texture::new(16, 16, back);
    paint(&mut t, 0, 0, front, legend);
    for x in 0..10 {
        t.set(6 + x, 3, belt);
    }
    for y in 0..4 {
        for x in 0..6 {
            t.set(x, 5 + y, top);
        }
    }
    t
}

/// A 4x4 limb: `main`, darker down one side, ending in `end`.
fn limb(main: u8, dark: u8, end: u8) -> Texture {
    let mut t = Texture::new(4, 4, main);
    t.set(3, 0, dark);
    t.set(3, 1, dark);
    for x in 0..4 {
        t.set(x, 3, end);
    }
    t
}

/// Proportions for a humanoid: half-sizes of the head, body and limbs, and lengths.
struct Build {
    head: Vec3,
    body: Vec3,
    arm: Vec2,
    leg: Vec2,
}

/// Puts a humanoid together from textures and a build; `dress` adds anything extra to its
/// parts (horns, moss, crystals, icicles).
fn assemble(
    bank: &mut TexBank,
    tex: [Texture; 4],
    b: &Build,
    dress: &mut dyn FnMut(&mut TexBank, &mut [Mesh; 6]),
) -> Humanoid {
    let [head_t, body_t, arm_t, leg_t] = tex;
    let head = bank.add(head_t);
    let body = bank.add(body_t);
    let arm = bank.add(arm_t);
    let leg = bank.add(leg_t);
    let mut parts: [Mesh; 6] = Default::default();
    let head_uv = BoxUv([
        UvRect::px(8, 0, 7, 7),
        UvRect::px(23, 0, 7, 7),
        UvRect::px(0, 7, 8, 7),
        UvRect::px(8, 7, 8, 7),
        UvRect::px(0, 0, 8, 7),
        UvRect::px(15, 0, 8, 7),
    ]);
    let h = b.head;
    parts[HEAD].cube(v(-h.x, 0.0, -h.z), v(h.x, h.y, h.z), &head_uv, head, 0);
    let body_uv = BoxUv([
        UvRect::px(12, 0, 4, 5),
        UvRect::px(12, 0, 4, 5),
        UvRect::px(0, 5, 6, 4),
        UvRect::px(6, 5, 6, 4),
        UvRect::px(0, 0, 6, 5),
        UvRect::px(6, 0, 6, 5),
    ]);
    let bd = b.body;
    parts[BODY].cube(v(-bd.x, 0.0, -bd.z), v(bd.x, bd.y, bd.z), &body_uv, body, 0);
    let limb_uv = BoxUv::all(UvRect::px(0, 0, 4, 4));
    for i in [ARM_L, ARM_R] {
        let t = b.arm.x;
        parts[i].cube(v(-t, -b.arm.y, -t), v(t, 0.0, t), &limb_uv, arm, 0);
    }
    for i in [LEG_L, LEG_R] {
        let t = b.leg.x;
        parts[i].cube(v(-t, -b.leg.y, -t), v(t, 0.0, t), &limb_uv, leg, 0);
    }
    dress(bank, &mut parts);
    let hip = b.leg.y;
    let neck = hip + bd.y - 0.05;
    Humanoid {
        head_tucked: parts[HEAD].clone(),
        head_hooded: parts[HEAD].clone(),
        parts,
        tex: HumanTex {
            head,
            body,
            arm,
            leg,
        },
        hip,
        shoulder: neck - 0.01,
        neck,
        shoulder_x: bd.x + b.arm.x,
        hip_x: bd.x * 0.5,
        hand: b.arm.y - 0.03,
    }
}

// ------------------------------------------------------------------------------------------
// Biome trimmings
// ------------------------------------------------------------------------------------------

/// Tufts of moss.
fn moss(bank: &mut TexBank, k: &mut Kit, m: &mut Mesh, at: Vec3, s: f32) {
    for (dx, dz, h, c) in [
        (-0.05f32, 0.0f32, 0.05f32, GREEN),
        (0.04, 0.03, 0.07, LIME),
        (0.0, -0.04, 0.04, TEAL),
    ] {
        let p = at + v(dx, 0.0, dz) * s;
        k.bx(
            bank,
            m,
            p - v(0.035, 0.0, 0.035) * s,
            p + v(0.035, h, 0.035) * s,
            c,
        );
    }
}

/// A crystal growing out along `dir`.
fn crystal(
    bank: &mut TexBank,
    k: &mut Kit,
    m: &mut Mesh,
    at: Vec3,
    dir: Vec3,
    len: f32,
    c: [u8; 2],
) {
    let mut p = Mesh::new();
    k.bx(
        bank,
        &mut p,
        v(-0.035, 0.0, -0.035),
        v(0.035, len * 0.75, 0.035),
        c[0],
    );
    k.bx(
        bank,
        &mut p,
        v(-0.02, len * 0.75, -0.02),
        v(0.02, len, 0.02),
        c[1],
    );
    let rot = glam::Quat::from_rotation_arc(Vec3::Y, dir.normalize());
    m.append(&p, Mat4::from_translation(at) * Mat4::from_quat(rot));
}

/// A little mushroom: a stem and a spotty cap.
fn shroom(bank: &mut TexBank, k: &mut Kit, m: &mut Mesh, at: Vec3, r: f32, cap: [u8; 2]) {
    k.bx(
        bank,
        m,
        at - v(r * 0.25, 0.0, r * 0.25),
        at + v(r * 0.25, r * 0.7, r * 0.25),
        CREAM,
    );
    let mut t = Texture::new(8, 8, cap[0]);
    for (x, y) in [(1, 1), (5, 2), (3, 5), (6, 6)] {
        t.set(x, y, cap[1]);
    }
    let tex = bank.add(t);
    lathe(
        m,
        at + Vec3::Y * (r * 0.6),
        &[(r, 0.0), (r * 0.8, r * 0.35), (0.0, r * 0.55)],
        6,
        0.0,
        tex,
        true,
    );
}

/// An icicle hanging down from `at`.
fn icicle(bank: &mut TexBank, k: &mut Kit, m: &mut Mesh, at: Vec3, len: f32) {
    k.bx(
        bank,
        m,
        at - v(0.025, len * 0.6, 0.025),
        at + v(0.025, 0.0, 0.025),
        WHITE,
    );
    k.bx(
        bank,
        m,
        at - v(0.012, len, 0.012),
        at - v(-0.012, len * 0.6, -0.012),
        SKY,
    );
}

/// Glowing coals stuck on like scales.
fn embers(bank: &mut TexBank, k: &mut Kit, m: &mut Mesh, at: Vec3, s: f32) {
    for (dx, dz, c) in [
        (-0.05f32, 0.02f32, ORANGE),
        (0.04, -0.02, GOLD),
        (0.0, 0.05, RED),
    ] {
        let p = at + v(dx, 0.0, dz) * s;
        k.bx(
            bank,
            m,
            p - v(0.03, 0.0, 0.03) * s,
            p + v(0.03, 0.04, 0.03) * s,
            c,
        );
    }
}

/// A striped pharaoh's headdress over a head `hx` wide and `hy` tall.
fn nemes(bank: &mut TexBank, m: &mut Mesh, hx: f32, hy: f32, hz: f32) {
    let stripes = bank.add(tiles::stripes(GOLD, INDIGO));
    tiled_box(
        m,
        v(-hx - 0.03, hy * 0.35, -hz - 0.03),
        v(hx + 0.03, hy + 0.04, hz * 0.3),
        stripes,
        0,
    );
    for sx in [-1.0f32, 1.0] {
        tiled_box(
            m,
            v(sx * (hx + 0.03) - 0.04, -0.12, -0.04),
            v(sx * (hx + 0.03) + 0.04, hy * 0.5, 0.08),
            stripes,
            0,
        );
    }
    let gold = bank.add(tiles::solid(GOLD));
    skin_box(
        m,
        v(-0.04, hy * 0.72, hz),
        v(0.04, hy * 0.9, hz + 0.04),
        gold,
        &Texture::new(4, 4, 0),
    );
}

/// The biome's trimmings on a head (`head`: its top and half-width) and the shoulders
/// (`body`: half-width and height).
fn trim(
    bank: &mut TexBank,
    k: &mut Kit,
    parts: &mut [Mesh; 6],
    biome: usize,
    (top, hx): (f32, f32),
    (bx, by): (f32, f32),
) {
    let (head, rest) = parts.split_at_mut(1);
    let head = &mut head[0];
    let body = &mut rest[0];
    match biome {
        0 => {
            moss(bank, k, head, v(-hx * 0.4, top, 0.0), 1.0);
            moss(bank, k, body, v(bx * 0.7, by, 0.0), 0.8);
            moss(bank, k, body, v(-bx * 0.6, by, -0.03), 0.7);
        }
        1 => {
            crystal(
                bank,
                k,
                head,
                v(hx * 0.4, top - 0.02, -0.05),
                v(0.3, 1.0, -0.2),
                0.2,
                [AQUA, MINT],
            );
            crystal(
                bank,
                k,
                head,
                v(-hx * 0.3, top - 0.02, 0.0),
                v(-0.4, 1.0, 0.1),
                0.14,
                [SKY, WHITE],
            );
            crystal(
                bank,
                k,
                body,
                v(bx * 0.6, by - 0.02, -0.06),
                v(0.5, 1.0, -0.4),
                0.18,
                [AQUA, MINT],
            );
            crystal(
                bank,
                k,
                body,
                v(-bx * 0.5, by - 0.02, -0.06),
                v(-0.3, 1.0, -0.5),
                0.14,
                [SKY, WHITE],
            );
        }
        2 => {
            shroom(bank, k, head, v(hx * 0.35, top, 0.0), 0.12, [PINK, WHITE]);
            shroom(
                bank,
                k,
                head,
                v(-hx * 0.45, top, -0.05),
                0.08,
                [LAVENDER, WHITE],
            );
            shroom(bank, k, body, v(-bx * 0.7, by, 0.0), 0.08, [PINK, WHITE]);
        }
        3 => {
            embers(bank, k, head, v(0.0, top, 0.0), 1.0);
            embers(bank, k, body, v(bx * 0.6, by, 0.0), 0.8);
        }
        4 => {
            for x in [-0.7f32, -0.2, 0.3, 0.75] {
                icicle(
                    bank,
                    k,
                    body,
                    v(bx * x, by * 0.25, 0.1),
                    0.1 + (x * 7.0).sin().abs() * 0.06,
                );
            }
            k.bx(
                bank,
                head,
                v(-hx - 0.01, top - 0.03, -0.2),
                v(hx + 0.01, top + 0.03, 0.2),
                WHITE,
            );
        }
        _ => {}
    }
}

// ------------------------------------------------------------------------------------------
// Zombies
// ------------------------------------------------------------------------------------------

fn zombie(bank: &mut TexBank, k: &mut Kit, biome: usize) -> Humanoid {
    let skins = [
        [LIME, GREEN, TEAL],
        [SKY, BLUE, INDIGO],
        [LAVENDER, PURPLE, GRAPE],
        [SHADOW, INK, INK],
        [WHITE, SKY, BLUE],
        [CREAM, SAND, KHAKI],
    ];
    let shirts = [
        [SAND, KHAKI, ROSEWOOD],
        [LAVENDER, PURPLE, GRAPE],
        [SAND, KHAKI, ROSEWOOD],
        [RUST, MAROON, INK],
        [SLATE, INDIGO, INK],
        [CREAM, SAND, KHAKI],
    ];
    let glows = [GOLD, MINT, PINK, ORANGE, BLUE, GOLD];
    let [sl, sm, sd] = skins[biome];
    let [cl, cm, cd] = shirts[biome];
    let g = glows[biome];
    let mummy = biome == 5;
    let legend = [
        ('s', sm),
        ('l', sl),
        ('d', sd),
        ('k', INK),
        ('g', g),
        ('w', CREAM),
        ('o', if biome == 3 { ORANGE } else { sd }),
    ];
    let face: &[&str] = if mummy {
        &[
            "llllllll", "sdssssds", "lkglllgk", "ssssssss", "llllllll", "ssdkkdss", "llllllll",
        ]
    } else {
        &[
            "dldlddls", "sllsssls", "kkssoskk", "kgksskgk", "ssosssss", "skwkwkds", "dsssossd",
        ]
    };
    let head = head_tex(
        face,
        &legend,
        sm,
        if mummy { sl } else { sd },
        if mummy { sl } else { sd },
        sd,
    );
    let body_front: &[&str] = if mummy {
        &["llllll", "ssssss", "llllll", "ssssss", "llllll"]
    } else {
        &["cmcccm", "csmcmm", "cmssmc", "kkkkkk", "mdmdmd"]
    };
    let body = body_tex(
        body_front,
        &[
            ('c', cl),
            ('m', cm),
            ('d', cd),
            ('s', sm),
            ('k', if mummy { sd } else { RUST }),
            ('l', sl),
        ],
        cm,
        if mummy { sd } else { RUST },
        cl,
    );
    let arm = if mummy {
        limb(sl, sd, sm)
    } else {
        limb(sm, sd, sd)
    };
    let leg = if mummy {
        limb(sl, sd, sm)
    } else {
        limb(if biome == 4 { SLATE } else { INDIGO }, INK, sd)
    };
    let b = Build {
        head: v(0.25, 0.44, 0.22),
        body: v(0.17, 0.3, 0.12),
        arm: Vec2::new(0.05, 0.27),
        leg: Vec2::new(0.055, 0.2),
    };
    assemble(bank, [head, body, arm, leg], &b, &mut |bank, parts| {
        trim(bank, k, parts, biome, (0.44, 0.25), (0.17, 0.3));
        if mummy {
            // A loose bandage end fluttering off the head.
            k.bx(
                bank,
                &mut parts[HEAD],
                v(0.1, 0.3, -0.26),
                v(0.18, 0.36, -0.22),
                CREAM,
            );
            k.bx(
                bank,
                &mut parts[HEAD],
                v(0.14, 0.18, -0.3),
                v(0.2, 0.32, -0.26),
                SAND,
            );
        }
    })
}

// ------------------------------------------------------------------------------------------
// Goblins
// ------------------------------------------------------------------------------------------

const GOBLIN_SKINS: [[u8; 3]; KINDS] = [
    [LIME, GREEN, TEAL],
    [MINT, AQUA, TEAL],
    [BLUSH, PINK, PLUM],
    [ORANGE, RED, MAROON],
    [WHITE, SKY, BLUE],
    [GOLD, CLAY, RUST],
];

fn goblin_face(skin: [u8; 3], eye: u8) -> Texture {
    let [sl, sm, sd] = skin;
    head_tex(
        &[
            "ssllllss", "sdssssds", "skesseks", "ssssssss", "sssddsss", "skwkkwks", "sdddddds",
        ],
        &[
            ('s', sm),
            ('l', sl),
            ('d', sd),
            ('k', INK),
            ('e', eye),
            ('w', WHITE),
        ],
        sm,
        sd,
        sm,
        sd,
    )
}

/// Big pointed ears and a nose, on a goblin head `hx` wide.
fn goblin_bits(
    bank: &mut TexBank,
    k: &mut Kit,
    head: &mut Mesh,
    skin: [u8; 3],
    hx: f32,
    hz: f32,
    nose: f32,
) {
    for s in [-1.0f32, 1.0] {
        let mut ear = Mesh::new();
        k.bx(
            bank,
            &mut ear,
            v(0.0, -0.035, -0.02),
            v(0.16, 0.035, 0.02),
            skin[1],
        );
        k.bx(
            bank,
            &mut ear,
            v(0.14, -0.02, -0.015),
            v(0.22, 0.02, 0.015),
            skin[0],
        );
        k.bx(
            bank,
            &mut ear,
            v(0.02, -0.02, 0.02),
            v(0.12, 0.02, 0.025),
            skin[2],
        );
        let m = if s < 0.0 {
            Mat4::from_translation(v(-hx, 0.27, 0.0))
                * Mat4::from_rotation_y(PI)
                * Mat4::from_rotation_z(0.45)
        } else {
            Mat4::from_translation(v(hx, 0.27, 0.0)) * Mat4::from_rotation_z(0.45)
        };
        head.append(&ear, m);
    }
    k.bx(
        bank,
        head,
        v(-0.035, 0.1, hz),
        v(0.035, 0.17, hz + nose),
        skin[1],
    );
    k.bx(
        bank,
        head,
        v(-0.025, 0.1, hz + nose - 0.01),
        v(0.025, 0.13, hz + nose + 0.01),
        skin[2],
    );
}

/// A fat goblin: round belly, short legs, a big grin with tusks.
fn brute(bank: &mut TexBank, k: &mut Kit, biome: usize) -> Humanoid {
    let skin = GOBLIN_SKINS[biome];
    let [sl, sm, sd] = skin;
    let hides = [
        [KHAKI, ROSEWOOD, SHADOW],
        [SLATE, INDIGO, INK],
        [KHAKI, ROSEWOOD, SHADOW],
        [SHADOW, INK, INK],
        [WHITE, SAND, KHAKI],
        [SAND, KHAKI, ROSEWOOD],
    ];
    let belts = [GREEN, AQUA, PINK, GOLD, SKY, GOLD];
    let [hl, hm, hd] = hides[biome];
    let head = goblin_face(skin, if biome == 5 { CREAM } else { GOLD });
    let body = body_tex(
        &["sslsss", "slllss", "slllss", "bbbbbb", "hmhmhm"],
        &[
            ('s', sm),
            ('l', sl),
            ('b', belts[biome]),
            ('h', hl),
            ('m', hm),
        ],
        sm,
        belts[biome],
        sl,
    );
    let arm = limb(sm, sd, sd);
    let leg = limb(hm, hd, sd);
    let b = Build {
        head: v(0.25, 0.38, 0.22),
        body: v(0.31, 0.34, 0.25),
        arm: Vec2::new(0.07, 0.26),
        leg: Vec2::new(0.075, 0.14),
    };
    assemble(bank, [head, body, arm, leg], &b, &mut |bank, parts| {
        goblin_bits(bank, k, &mut parts[HEAD], skin, 0.25, 0.22, 0.08);
        // Tusks poking up from the grin.
        for sx in [-1.0f32, 1.0] {
            k.bx(
                bank,
                &mut parts[HEAD],
                v(sx * 0.12 - 0.02, 0.07, 0.22),
                v(sx * 0.12 + 0.02, 0.15, 0.25),
                WHITE,
            );
        }
        // Shoulder pads (or leaves, fur, gold) for the biome.
        let pad = [GREEN, AQUA, PINK, INK, WHITE, GOLD][biome];
        for sx in [-1.0f32, 1.0] {
            k.bx(
                bank,
                &mut parts[BODY],
                v(sx * 0.31 - 0.08, 0.28, -0.14),
                v(sx * 0.31 + 0.08, 0.36, 0.14),
                pad,
            );
        }
        if biome == 3 {
            // Little horns.
            for sx in [-1.0f32, 1.0] {
                crystal(
                    bank,
                    k,
                    &mut parts[HEAD],
                    v(sx * 0.14, 0.38, 0.0),
                    v(sx * 0.4, 1.0, 0.0),
                    0.14,
                    [SAND, WHITE],
                );
            }
        } else {
            trim(bank, k, parts, biome, (0.38, 0.25), (0.31, 0.34));
        }
        if biome == 5 {
            for sx in [-1.0f32, 1.0] {
                k.bx(
                    bank,
                    &mut parts[HEAD],
                    v(sx * 0.3 - 0.02, 0.14, -0.02),
                    v(sx * 0.3 + 0.02, 0.2, 0.02),
                    GOLD,
                );
            }
        }
    })
}

/// A skinny goblin: all elbows and knees, a hood and a long nose.
fn sneak(bank: &mut TexBank, k: &mut Kit, biome: usize) -> Humanoid {
    let skin = GOBLIN_SKINS[biome];
    let [sl, sm, sd] = skin;
    let cloaks = [
        [TEAL, DEEP_TEAL, INK],
        [BLUE, INDIGO, SLATE],
        [PURPLE, GRAPE, INK],
        [RUST, MAROON, INK],
        [SKY, SLATE, INDIGO],
        [CLAY, RUST, MAROON],
    ];
    let [cl, cm, cd] = cloaks[biome];
    let mut head = goblin_face(skin, if biome == 5 { WHITE } else { GOLD });
    // A hood pulled over the top and back.
    for y in 0..7 {
        for x in 0..8 {
            head.set(15 + x, y, if y == 6 { cd } else { cm });
            head.set(x, 7 + y, cm);
        }
        head.set(0, y, cm);
        head.set(7, y, cm);
    }
    for x in 0..8 {
        head.set(x, 0, cl);
    }
    let body = body_tex(
        &["cmmmmc", "cmllmc", "cmmmmc", "dkkkkd", "mdmdmd"],
        &[('c', cl), ('m', cm), ('d', cd), ('k', INK), ('l', sl)],
        cm,
        INK,
        cl,
    );
    let arm = limb(sm, sd, sd);
    let leg = limb(cm, cd, INK);
    let b = Build {
        head: v(0.21, 0.36, 0.19),
        body: v(0.12, 0.27, 0.08),
        arm: Vec2::new(0.035, 0.3),
        leg: Vec2::new(0.035, 0.27),
    };
    assemble(bank, [head, body, arm, leg], &b, &mut |bank, parts| {
        goblin_bits(bank, k, &mut parts[HEAD], skin, 0.21, 0.19, 0.12);
        // The hood's point, flopping back.
        let mut tip = Mesh::new();
        k.bx(bank, &mut tip, v(-0.08, 0.0, -0.1), v(0.08, 0.08, 0.1), cm);
        k.bx(
            bank,
            &mut tip,
            v(-0.04, 0.06, -0.16),
            v(0.04, 0.12, 0.0),
            cd,
        );
        parts[HEAD].append(&tip, Mat4::from_translation(v(0.0, 0.36, -0.05)));
        let _ = sl;
    })
}

// ------------------------------------------------------------------------------------------
// Skeletons
// ------------------------------------------------------------------------------------------

fn skeleton(bank: &mut TexBank, k: &mut Kit, biome: usize) -> Humanoid {
    let bones = [
        [SAND, KHAKI, ROSEWOOD],
        [WHITE, SKY, BLUE],
        [CREAM, SAND, KHAKI],
        [SHADOW, INK, INK],
        [WHITE, SKY, SLATE],
        [CREAM, GOLD, CLAY],
    ];
    let glows = [LIME, AQUA, PINK, ORANGE, SKY, GOLD];
    let [bl, bm, bd] = bones[biome];
    let g = glows[biome];
    let charred = biome == 3;
    let legend = [
        ('b', bm),
        ('l', bl),
        ('d', bd),
        ('k', if charred { MAROON } else { INK }),
        ('g', g),
        ('w', if charred { SHADOW } else { bl }),
    ];
    let head = head_tex(
        &[
            "llllllll", "blbbbbbb", "bkkbbkkb", "bkgbbgkb", "bbbkkbbb", "bwkwkwkb", "dbbwwbbd",
        ],
        &legend,
        bm,
        bm,
        bl,
        bd,
    );
    let rib = if charred { ORANGE } else { bm };
    let body = body_tex(
        &["llllll", "kbkkbk", "kbbbbk", "kbkkbk", "dbbbbd"],
        &[
            ('l', bl),
            ('b', rib),
            ('k', if charred { MAROON } else { INK }),
            ('d', bd),
        ],
        if charred { MAROON } else { INK },
        bm,
        bl,
    );
    let arm = limb(bm, bd, bl);
    let leg = limb(bm, bd, bl);
    let b = Build {
        head: v(0.22, 0.4, 0.2),
        body: v(0.14, 0.3, 0.09),
        arm: Vec2::new(0.032, 0.27),
        leg: Vec2::new(0.035, 0.21),
    };
    assemble(bank, [head, body, arm, leg], &b, &mut |bank, parts| {
        // A bony pelvis and shoulder knobs.
        k.bx(
            bank,
            &mut parts[BODY],
            v(-0.12, -0.02, -0.06),
            v(0.12, 0.05, 0.06),
            bm,
        );
        for sx in [-1.0f32, 1.0] {
            k.bx(
                bank,
                &mut parts[BODY],
                v(sx * 0.15 - 0.04, 0.26, -0.04),
                v(sx * 0.15 + 0.04, 0.32, 0.04),
                bl,
            );
        }
        if biome == 5 {
            nemes(bank, &mut parts[HEAD], 0.22, 0.4, 0.2);
        } else {
            trim(bank, k, parts, biome, (0.4, 0.22), (0.14, 0.3));
        }
    })
}

// ------------------------------------------------------------------------------------------
// Ghosts
// ------------------------------------------------------------------------------------------

fn ghost_sheet(bank: &mut TexBank, pal: [u8; 3]) -> Mesh {
    let sheet = bank.add(slime_skin(pal));
    let mut m = Mesh::new();
    lathe(
        &mut m,
        Vec3::ZERO,
        &[
            (0.3, 0.0),
            (0.34, 0.2),
            (0.3, 0.45),
            (0.18, 0.62),
            (0.0, 0.68),
        ],
        8,
        0.39,
        sheet,
        true,
    );
    m
}

fn ghost_hat(bank: &mut TexBank, k: &mut Kit, biome: usize) -> Mesh {
    let mut m = Mesh::new();
    match biome {
        0 => {
            // A sprig of leaves.
            k.bx(
                bank,
                &mut m,
                v(-0.015, 0.6, -0.015),
                v(0.015, 0.76, 0.015),
                GREEN,
            );
            for s in [-1.0f32, 1.0] {
                k.bx(
                    bank,
                    &mut m,
                    v(s * 0.02, 0.7, -0.03),
                    v(s * 0.12, 0.74, 0.03),
                    LIME,
                );
            }
        }
        1 => {
            for (x, h) in [(-0.12f32, 0.12f32), (0.0, 0.18), (0.12, 0.12)] {
                crystal(
                    bank,
                    k,
                    &mut m,
                    v(x, 0.6, 0.02),
                    v(x * 2.0, 1.0, 0.0),
                    h,
                    [AQUA, WHITE],
                );
            }
        }
        2 => shroom(bank, k, &mut m, v(0.05, 0.6, 0.0), 0.16, [PINK, WHITE]),
        3 => {
            // Flickering flame crest.
            for (x, h, c) in [
                (-0.1f32, 0.12f32, ORANGE),
                (0.0, 0.2, GOLD),
                (0.1, 0.12, RED),
            ] {
                k.bx(
                    bank,
                    &mut m,
                    v(x - 0.04, 0.58, -0.04),
                    v(x + 0.04, 0.58 + h, 0.04),
                    c,
                );
            }
        }
        4 => {
            for k2 in 0..5 {
                let a = k2 as f32 / 5.0 * PI * 2.0;
                let p = v(a.sin() * 0.18, 0.6, a.cos() * 0.16);
                k.bx(
                    bank,
                    &mut m,
                    p - v(0.025, 0.0, 0.025),
                    p + v(0.025, 0.12, 0.025),
                    WHITE,
                );
            }
        }
        _ => nemes(bank, &mut m, 0.22, 0.2, 0.2),
    }
    if biome == 5 {
        // Sit the headdress on the sheet's crown.
        let mut lifted = Mesh::new();
        lifted.append(&m, Mat4::from_translation(v(0.0, 0.46, 0.0)));
        return lifted;
    }
    m
}

// ------------------------------------------------------------------------------------------
// Bugs
// ------------------------------------------------------------------------------------------

fn bug(bank: &mut TexBank, k: &mut Kit, biome: usize) -> Bug {
    let mut body = Mesh::new();
    let mut leg = Mesh::new();
    let mut wing = None;
    let mut claw = None;
    let eye = |bank: &mut TexBank, k: &mut Kit, m: &mut Mesh, y: f32, z: f32, gap: f32, c: u8| {
        for s in [-1.0f32, 1.0] {
            k.bx(
                bank,
                m,
                v(s * gap - 0.03, y - 0.03, z),
                v(s * gap + 0.03, y + 0.03, z + 0.03),
                c,
            );
        }
    };
    let (hips, hip_x, hip_y, flies) = match biome {
        0 => {
            // Moss mite: a round, fuzzy green ball with moss on its back.
            let fuzz = speck(bank, [LIME, GREEN, TEAL]);
            lathe(
                &mut body,
                v(0.0, 0.06, 0.0),
                &[
                    (0.0, 0.0),
                    (0.2, 0.04),
                    (0.24, 0.16),
                    (0.18, 0.28),
                    (0.0, 0.32),
                ],
                8,
                0.0,
                fuzz,
                false,
            );
            moss(bank, k, &mut body, v(0.0, 0.34, -0.02), 1.2);
            eye(bank, k, &mut body, 0.22, 0.2, 0.08, INK);
            k.bx(
                bank,
                &mut leg,
                v(-0.02, -0.02, -0.02),
                v(0.18, 0.02, 0.02),
                TEAL,
            );
            (vec![0.12, 0.0, -0.12], 0.18, 0.1, false)
        }
        1 => {
            // Glass mantis: thin and angular, with folding claws.
            let glass = speck(bank, [MINT, AQUA, TEAL]);
            tiled_box(
                &mut body,
                v(-0.06, 0.12, -0.3),
                v(0.06, 0.22, 0.02),
                glass,
                0,
            );
            tiled_box(&mut body, v(-0.05, 0.16, 0.0), v(0.05, 0.36, 0.1), glass, 0);
            tiled_box(
                &mut body,
                v(-0.08, 0.34, 0.06),
                v(0.08, 0.44, 0.18),
                glass,
                0,
            );
            eye(bank, k, &mut body, 0.41, 0.18, 0.07, INK);
            crystal(
                bank,
                k,
                &mut body,
                v(0.0, 0.2, -0.2),
                v(0.0, 1.0, -0.6),
                0.14,
                [WHITE, SKY],
            );
            for s in [-1.0f32, 1.0] {
                k.bx(
                    bank,
                    &mut body,
                    v(s * 0.03, 0.44, 0.14),
                    v(s * 0.05, 0.56, 0.16),
                    AQUA,
                );
            }
            k.bx(
                bank,
                &mut leg,
                v(-0.015, -0.015, -0.015),
                v(0.22, 0.015, 0.015),
                TEAL,
            );
            let mut c = Mesh::new();
            k.bx(
                bank,
                &mut c,
                v(-0.025, -0.02, 0.0),
                v(0.025, 0.02, 0.18),
                AQUA,
            );
            k.bx(
                bank,
                &mut c,
                v(-0.02, -0.12, 0.15),
                v(0.02, 0.0, 0.19),
                MINT,
            );
            claw = Some(c);
            (vec![0.0, -0.1, -0.2], 0.05, 0.18, false)
        }
        2 => {
            // Spore moth: fuzzy, feathery antennae and big eyespot wings. It flies.
            let fuzz = speck(bank, [BLUSH, LAVENDER, PURPLE]);
            lathe(
                &mut body,
                v(0.0, -0.1, -0.05),
                &[
                    (0.0, 0.0),
                    (0.08, 0.05),
                    (0.1, 0.18),
                    (0.07, 0.28),
                    (0.0, 0.3),
                ],
                6,
                0.0,
                fuzz,
                false,
            );
            eye(bank, k, &mut body, 0.14, 0.05, 0.05, INK);
            for s in [-1.0f32, 1.0] {
                k.bx(
                    bank,
                    &mut body,
                    v(s * 0.03, 0.2, 0.0),
                    v(s * 0.05, 0.34, 0.03),
                    CREAM,
                );
                k.bx(
                    bank,
                    &mut body,
                    v(s * 0.02 - 0.03, 0.3, 0.0),
                    v(s * 0.06 + 0.03, 0.34, 0.03),
                    CREAM,
                );
            }
            let mut t = Texture::new(16, 16, LAVENDER);
            for y in 0..16i32 {
                for x in 0..16i32 {
                    let d = ((x - 9) * (x - 9) + (y - 7) * (y - 7)) as f32;
                    let c = if d < 5.0 {
                        INK
                    } else if d < 12.0 {
                        GOLD
                    } else if d < 22.0 {
                        PINK
                    } else if x < 2 || y > 13 {
                        PURPLE
                    } else {
                        LAVENDER
                    };
                    t.set(x, y, c);
                }
            }
            let wt = bank.add(t);
            let mut w = Mesh::new();
            w.quad(
                [
                    v(0.0, 0.0, 0.12),
                    v(0.36, 0.0, 0.18),
                    v(0.4, 0.0, -0.16),
                    v(0.0, 0.0, -0.14),
                ],
                UvRect::new(0.0, 0.0, 16.0, 16.0),
                wt,
            );
            wing = Some(w);
            k.bx(
                bank,
                &mut leg,
                v(-0.01, -0.01, -0.01),
                v(0.1, 0.01, 0.01),
                PURPLE,
            );
            (vec![0.04, -0.06], 0.06, 0.02, true)
        }
        3 => {
            // Fire ant: head, waist and a big glowing tail, with mandibles.
            let shell = speck(bank, [RED, CRIMSON, MAROON]);
            let glow = speck(bank, [GOLD, ORANGE, RED]);
            lathe(
                &mut body,
                v(0.0, 0.12, -0.2),
                &[(0.0, -0.13), (0.13, -0.06), (0.14, 0.05), (0.0, 0.13)],
                8,
                0.0,
                glow,
                false,
            );
            let mut tail = Mesh::new();
            tail.append(&body, Mat4::IDENTITY);
            body = Mesh::new();
            body.append(
                &tail,
                Mat4::from_translation(v(0.0, 0.0, -0.2))
                    * Mat4::from_rotation_x(PI * 0.5)
                    * Mat4::from_translation(v(0.0, 0.2, -0.12)),
            );
            tiled_box(
                &mut body,
                v(-0.06, 0.1, -0.08),
                v(0.06, 0.2, 0.08),
                shell,
                0,
            );
            tiled_box(
                &mut body,
                v(-0.09, 0.1, 0.08),
                v(0.09, 0.24, 0.24),
                shell,
                0,
            );
            eye(bank, k, &mut body, 0.2, 0.24, 0.06, INK);
            for s in [-1.0f32, 1.0] {
                k.bx(
                    bank,
                    &mut body,
                    v(s * 0.05 - 0.02, 0.1, 0.24),
                    v(s * 0.05 + 0.02, 0.13, 0.33),
                    MAROON,
                );
                k.bx(
                    bank,
                    &mut body,
                    v(s * 0.04, 0.24, 0.18),
                    v(s * 0.05, 0.36, 0.2),
                    MAROON,
                );
            }
            k.bx(
                bank,
                &mut leg,
                v(-0.012, -0.012, -0.012),
                v(0.2, 0.012, 0.012),
                MAROON,
            );
            (vec![0.06, 0.0, -0.06], 0.06, 0.14, false)
        }
        4 => {
            // Frost tick: a flat oval shell studded with ice.
            let shell = speck(bank, [WHITE, SKY, BLUE]);
            lathe(
                &mut body,
                v(0.0, 0.05, 0.0),
                &[
                    (0.0, 0.0),
                    (0.24, 0.02),
                    (0.26, 0.08),
                    (0.16, 0.15),
                    (0.0, 0.17),
                ],
                8,
                0.0,
                shell,
                false,
            );
            for (x, z, h) in [
                (-0.08f32, -0.05f32, 0.1f32),
                (0.08, -0.04, 0.08),
                (0.0, 0.06, 0.12),
                (0.0, -0.14, 0.07),
            ] {
                crystal(
                    bank,
                    k,
                    &mut body,
                    v(x, 0.18, z),
                    v(x * 3.0, 1.0, z * 3.0),
                    h,
                    [WHITE, SKY],
                );
            }
            k.bx(
                bank,
                &mut body,
                v(-0.07, 0.04, 0.2),
                v(0.07, 0.12, 0.3),
                SLATE,
            );
            eye(bank, k, &mut body, 0.1, 0.3, 0.04, RED);
            k.bx(
                bank,
                &mut leg,
                v(-0.015, -0.015, -0.015),
                v(0.16, 0.015, 0.015),
                SLATE,
            );
            (vec![0.12, 0.02, -0.08, -0.18], 0.2, 0.08, false)
        }
        _ => {
            // Scarab: a gold and teal dome with a horn.
            let shell = speck(bank, [GOLD, CLAY, RUST]);
            let jewel = speck(bank, [MINT, TEAL, DEEP_TEAL]);
            lathe(
                &mut body,
                v(0.0, 0.05, -0.02),
                &[
                    (0.0, 0.0),
                    (0.22, 0.02),
                    (0.24, 0.12),
                    (0.16, 0.22),
                    (0.0, 0.25),
                ],
                8,
                0.0,
                shell,
                false,
            );
            tiled_box(
                &mut body,
                v(-0.01, 0.12, -0.24),
                v(0.01, 0.3, 0.2),
                jewel,
                0,
            );
            tiled_box(&mut body, v(-0.1, 0.06, 0.18), v(0.1, 0.16, 0.3), jewel, 0);
            crystal(
                bank,
                k,
                &mut body,
                v(0.0, 0.14, 0.28),
                v(0.0, 1.0, 0.8),
                0.14,
                [GOLD, CREAM],
            );
            eye(bank, k, &mut body, 0.13, 0.3, 0.06, INK);
            k.bx(
                bank,
                &mut leg,
                v(-0.015, -0.015, -0.015),
                v(0.18, 0.015, 0.015),
                RUST,
            );
            (vec![0.1, 0.0, -0.1], 0.18, 0.1, false)
        }
    };
    let leg_len = leg.verts.iter().map(|v| v.pos.x).fold(0.0, f32::max);
    Bug {
        body,
        leg,
        wing,
        claw,
        hips,
        hip_x,
        hip_y,
        leg_len,
        flies,
    }
}

// ------------------------------------------------------------------------------------------
// Weapons
// ------------------------------------------------------------------------------------------

/// A fat goblin's club, gripped at the origin and pointing down the arm.
fn club(bank: &mut TexBank, k: &mut Kit, biome: usize) -> Mesh {
    let mut m = Mesh::new();
    let wood = speck(bank, [CLAY, RUST, MAROON]);
    tiled_box(&mut m, v(-0.03, -0.3, -0.03), v(0.03, 0.06, 0.03), wood, 0);
    let (head, tip) = match biome {
        0 => ([KHAKI, ROSEWOOD, SHADOW], GREEN),
        1 => ([MINT, AQUA, TEAL], WHITE),
        2 => ([BLUSH, PINK, PLUM], WHITE),
        3 => ([SHADOW, INK, INK], ORANGE),
        4 => ([WHITE, SKY, BLUE], WHITE),
        _ => ([CREAM, GOLD, CLAY], CREAM),
    };
    let t = speck(bank, head);
    tiled_box(&mut m, v(-0.08, -0.52, -0.08), v(0.08, -0.28, 0.08), t, 0);
    for (x, z) in [(0.08f32, 0.0f32), (-0.08, 0.0), (0.0, 0.08), (0.0, -0.08)] {
        k.bx(
            bank,
            &mut m,
            v(x - 0.02, -0.44, z - 0.02),
            v(x + 0.02, -0.38, z + 0.02) + v(x * 0.4, 0.0, z * 0.4),
            tip,
        );
    }
    m
}

/// A skinny goblin's dagger.
fn dagger(bank: &mut TexBank, k: &mut Kit, biome: usize) -> Mesh {
    let mut m = Mesh::new();
    let grip = [TEAL, INDIGO, GRAPE, MAROON, SLATE, RUST][biome];
    let blade = [WHITE, AQUA, BLUSH, ORANGE, SKY, GOLD][biome];
    k.bx(
        bank,
        &mut m,
        v(-0.02, -0.06, -0.02),
        v(0.02, 0.05, 0.02),
        grip,
    );
    k.bx(
        bank,
        &mut m,
        v(-0.05, -0.08, -0.02),
        v(0.05, -0.06, 0.02),
        GOLD,
    );
    k.bx(
        bank,
        &mut m,
        v(-0.02, -0.26, -0.01),
        v(0.02, -0.08, 0.01),
        blade,
    );
    m
}

pub fn build(bank: &mut TexBank) -> Monsters {
    let mut k = Kit {
        solid: Default::default(),
        w4: Texture::new(4, 4, 0),
    };
    let mut ghost_face = Mesh::new();
    let eye = k.c(bank, INK);
    for sx in [-1.0f32, 1.0] {
        let c = v(sx * 0.1, 0.42, 0.305);
        skin_box(
            &mut ghost_face,
            c - v(0.045, 0.07, 0.0),
            c + v(0.045, 0.07, 0.03),
            eye,
            &k.w4,
        );
    }
    let ghost_pals = [
        [MINT, LIME, GREEN],
        [WHITE, SKY, AQUA],
        [BLUSH, PINK, LAVENDER],
        [CREAM, GOLD, ORANGE],
        [WHITE, WHITE, SKY],
        [WHITE, BLUSH, LAVENDER],
    ];
    Monsters {
        zombie: (0..KINDS).map(|b| zombie(bank, &mut k, b)).collect(),
        brute: (0..KINDS).map(|b| brute(bank, &mut k, b)).collect(),
        club: (0..KINDS).map(|b| club(bank, &mut k, b)).collect(),
        sneak: (0..KINDS).map(|b| sneak(bank, &mut k, b)).collect(),
        dagger: (0..KINDS).map(|b| dagger(bank, &mut k, b)).collect(),
        skeleton: (0..KINDS).map(|b| skeleton(bank, &mut k, b)).collect(),
        ghost: ghost_pals.iter().map(|p| ghost_sheet(bank, *p)).collect(),
        ghost_face,
        ghost_hat: (0..KINDS).map(|b| ghost_hat(bank, &mut k, b)).collect(),
        bug: (0..KINDS).map(|b| bug(bank, &mut k, b)).collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_family_has_a_look_for_every_biome() {
        let mut bank = TexBank::default();
        let m = build(&mut bank);
        for list in [&m.zombie, &m.brute, &m.sneak, &m.skeleton] {
            assert_eq!(list.len(), KINDS);
            for h in list.iter() {
                assert!(h.parts.iter().all(|p| !p.tris.is_empty()));
                assert!(h.hip > 0.0 && h.neck > h.hip && h.shoulder_x > 0.0);
            }
        }
        // Fat goblins are wider and shorter in the leg than skinny ones.
        assert!(m.brute[0].shoulder_x > m.sneak[0].shoulder_x);
        assert!(m.brute[0].hip < m.sneak[0].hip);
        assert_eq!(m.bug.len(), KINDS);
        assert!(m.bug.iter().any(|b| b.flies && b.wing.is_some()));
        assert!(m.bug.iter().any(|b| b.claw.is_some()));
        assert!(
            m.bug
                .iter()
                .all(|b| !b.body.tris.is_empty() && !b.hips.is_empty())
        );
        assert_eq!(m.ghost.len(), KINDS);
        assert_eq!(m.ghost_hat.len(), KINDS);
        assert_eq!(m.club.len(), KINDS);
        assert_eq!(m.dagger.len(), KINDS);
    }
}
