//! The Hollow's walking dead, goblins (fat and skinny), bugs, bones and ghosts. Every family
//! comes in six looks, one for each biome: mossy, crystal, fungal, ember, frost and ruins;
//! and a seventh for the old sewers, where they're all down to their bones.

use std::f32::consts::PI;

use glam::{Mat4, Vec2, Vec3};

use super::models::{
    ARM_L, ARM_R, BODY, HEAD, HumanTex, Humanoid, LEG_L, LEG_R, lathe, skin_box, slime_skin,
    tiled_box,
};
use super::tiles;
use super::{LOOKS, SEWER_LOOK};
use crate::palette::*;
use crate::render::{BoxUv, Mesh, TexBank, TexId, Texture, UvRect};

/// A bug: a body, jointed legs (a thigh rising from the hip to a knee, a shin reaching down
/// to the floor), and wings or claws for some.
pub struct Bug {
    pub body: Mesh,
    /// A leg's two segments, each running along +x from its joint.
    pub thigh: Mesh,
    pub shin: Mesh,
    pub thigh_len: f32,
    pub shin_len: f32,
    /// Each pair of legs: where it meets the body along its length (z), and how far it
    /// reaches forwards (+) or back (-), in radians.
    pub legs: Vec<(f32, f32)>,
    /// How far out from the middle and how high the hips are.
    pub hip_x: f32,
    pub hip_y: f32,
    /// How steeply the thighs rise to the knees (radians above level).
    pub knee: f32,
    /// Moths have wings, drawn on each side and flapping.
    pub wing: Option<Mesh>,
    /// Mantises have a pair of folding claws up front.
    pub claw: Option<Mesh>,
    /// Flies above the ground instead of scuttling (its legs dangle).
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
    /// The werewolf of the full moon, and its bushy tail (hanging down and back from the
    /// hips).
    pub werewolf: Humanoid,
    pub wolf_tail: Mesh,
    /// The labyrinth's minotaur, its tufted tail (hanging from the hips), and its axe
    /// (gripped at the origin, pointing down the arm).
    pub minotaur: Humanoid,
    pub bull_tail: Mesh,
    pub labrys: Mesh,
    /// The starless rift's ogre, and its imps in violet with their spade-tipped tails
    /// (hanging from the hips; their wings are the gazer's, see `void_art`).
    pub ogre: Humanoid,
    pub void_imp: Humanoid,
    pub imp_tail: Mesh,
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
pub(super) fn paint(t: &mut Texture, x0: i32, y0: i32, rows: &[&str], legend: &[(char, u8)]) {
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

/// Sewer grime: a splat of green slime on top of something, dripping down its front.
fn grime(bank: &mut TexBank, k: &mut Kit, m: &mut Mesh, at: Vec3, w: f32, d: f32) {
    k.bx(
        bank,
        m,
        at + v(-w * 0.65, -0.01, d * 0.2),
        at + v(w * 0.1, 0.02, d + 0.015),
        GREEN,
    );
    for (x, len, c) in [
        (-0.6f32, 0.12f32, LIME),
        (-0.1, 0.07, GREEN),
        (0.3, 0.16, GREEN),
    ] {
        k.bx(
            bank,
            m,
            at + v(x * w - 0.018, -len, d - 0.01),
            at + v(x * w + 0.018, 0.0, d + 0.015),
            c,
        );
    }
}

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
        SEWER_LOOK => {
            grime(bank, k, head, v(0.0, top, 0.0), hx, hx * 0.9);
            moss(bank, k, body, v(-bx * 0.6, by, -0.03), 0.7);
            grime(bank, k, body, v(bx * 0.4, by, 0.0), bx * 0.5, bx * 0.55);
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
        // A bone shambler: nothing left but bones, and rags.
        [WHITE, SAND, KHAKI],
    ];
    let shirts = [
        [SAND, KHAKI, ROSEWOOD],
        [LAVENDER, PURPLE, GRAPE],
        [SAND, KHAKI, ROSEWOOD],
        [RUST, MAROON, INK],
        [SLATE, INDIGO, INK],
        [CREAM, SAND, KHAKI],
        [TEAL, DEEP_TEAL, INK],
    ];
    let glows = [GOLD, MINT, PINK, ORANGE, BLUE, GOLD, LIME];
    let [sl, sm, sd] = skins[biome];
    let [cl, cm, cd] = shirts[biome];
    let g = glows[biome];
    let mummy = biome == 5;
    let bones = biome == SEWER_LOOK;
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
    } else if bones {
        // A cracked skull, one eye still lit, its jaw hanging open.
        &[
            "lllldlll", "slllllls", "skksskks", "skgsskks", "ssskksss", "skwkwkws", "dkkkkkkd",
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
    } else if bones {
        // Ribs, and a rag slung over one shoulder.
        &["lskscm", "kssksc", "lskscm", "kkkkcm", "mdmdmd"]
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
            (
                'k',
                if mummy {
                    sd
                } else if bones {
                    INK
                } else {
                    RUST
                },
            ),
            ('l', sl),
        ],
        cm,
        if mummy { sd } else { RUST },
        cl,
    );
    let arm = if mummy {
        limb(sl, sd, sm)
    } else if bones {
        limb(sm, sd, sl)
    } else {
        limb(sm, sd, sd)
    };
    let leg = if mummy {
        limb(sl, sd, sm)
    } else if bones {
        limb(cm, cd, sm)
    } else {
        limb(if biome == 4 { SLATE } else { INDIGO }, INK, sd)
    };
    // Bones are thinner than the flesh that was on them.
    let thin = if bones { 0.7 } else { 1.0 };
    let b = Build {
        head: v(0.25, 0.44, 0.22),
        body: v(0.17, 0.3, 0.12),
        arm: Vec2::new(0.05 * thin, 0.27),
        leg: Vec2::new(0.055 * thin, 0.2),
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

const GOBLIN_SKINS: [[u8; 3]; LOOKS] = [
    [LIME, GREEN, TEAL],
    [MINT, AQUA, TEAL],
    [BLUSH, PINK, PLUM],
    [ORANGE, RED, MAROON],
    [WHITE, SKY, BLUE],
    [GOLD, CLAY, RUST],
    // Bone.
    [WHITE, SAND, KHAKI],
];

/// A goblin's skull: sockets with a light still in them, a hole for the nose and a grin of
/// teeth over the jaw.
fn goblin_skull(skin: [u8; 3], eye: u8) -> Texture {
    let [sl, sm, sd] = skin;
    head_tex(
        &[
            "ssllllss", "sdssssds", "kkksskkk", "keksskek", "ssskksss", "kwkwwkwk", "sdkkkkds",
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
        [TEAL, DEEP_TEAL, INK],
    ];
    let belts = [GREEN, AQUA, PINK, GOLD, SKY, GOLD, RUST];
    let [hl, hm, hd] = hides[biome];
    let bones = biome == SEWER_LOOK;
    let head = if bones {
        goblin_skull(skin, LIME)
    } else {
        goblin_face(skin, if biome == 5 { CREAM } else { GOLD })
    };
    let front: &[&str] = if bones {
        // A barrel of ribs where the belly was.
        &["lkllkl", "skssks", "lkllkl", "bbbbbb", "hmhmhm"]
    } else {
        &["sslsss", "slllss", "slllss", "bbbbbb", "hmhmhm"]
    };
    let body = body_tex(
        front,
        &[
            ('s', sm),
            ('l', sl),
            ('k', INK),
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
        // Shoulder pads (or leaves, fur, gold, bone) for the biome.
        let pad = [GREEN, AQUA, PINK, INK, WHITE, GOLD, SAND][biome];
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

/// A werewolf: grey fur, a long muzzle full of fangs, tall pointed ears, glowing gold eyes,
/// and claws, in what's left of a pair of trousers. Tall, lean and hunched.
fn werewolf(bank: &mut TexBank, k: &mut Kit) -> (Humanoid, Mesh) {
    let (fur, dark, light) = (SHADOW, INK, KHAKI);
    let (cloth, torn) = (RUST, MAROON);
    let legend = [
        ('f', fur),
        ('d', dark),
        ('l', light),
        ('y', GOLD),
        ('k', INK),
        ('c', cloth),
    ];
    let head = head_tex(
        &[
            "ffffffff", "kkffffkk", "fyykkyyf", "fyklkyyf", "ffllllff", "flllllll", "llllllll",
        ],
        &legend,
        fur,
        dark,
        fur,
        light,
    );
    let body = body_tex(
        &["fllllf", "fllllf", "ffllff", "ffffff", "cccccc"],
        &legend,
        fur,
        cloth,
        fur,
    );
    let arm = limb(fur, dark, dark);
    let leg = limb(cloth, torn, fur);
    let b = Build {
        head: v(0.15, 0.24, 0.15),
        body: v(0.24, 0.36, 0.17),
        arm: Vec2::new(0.07, 0.36),
        leg: Vec2::new(0.08, 0.22),
    };
    let wolf = assemble(bank, [head, body, arm, leg], &b, &mut |bank, parts| {
        let h = &mut parts[HEAD];
        // A long muzzle: fur on top, a pale jaw under it, a black nose on the end, and
        // fangs hanging over the jaw.
        k.bx(bank, h, v(-0.07, 0.045, 0.14), v(0.07, 0.13, 0.33), fur);
        k.bx(bank, h, v(-0.062, 0.0, 0.14), v(0.062, 0.05, 0.3), light);
        k.bx(bank, h, v(-0.034, 0.1, 0.32), v(0.034, 0.145, 0.35), INK);
        k.bx(bank, h, v(-0.068, 0.046, 0.3), v(0.068, 0.054, 0.334), INK);
        for sx in [-1.0f32, 1.0] {
            k.bx(
                bank,
                h,
                v(sx * 0.045 - 0.011, 0.012, 0.3),
                v(sx * 0.045 + 0.011, 0.05, 0.32),
                WHITE,
            );
            // Shaggy tufts at its cheeks.
            let (x0, x1) = if sx < 0.0 {
                (-0.19, -0.13)
            } else {
                (0.13, 0.19)
            };
            k.bx(bank, h, v(x0, 0.02, -0.06), v(x1, 0.12, 0.1), fur);
        }
        // Tall pointed ears, dark at the tips, pink inside.
        let furt = k.c(bank, fur);
        let tip = k.c(bank, dark);
        let inner = k.c(bank, ROSEWOOD);
        for sx in [-1.0f32, 1.0] {
            let mut ear = Mesh::new();
            lathe(
                &mut ear,
                Vec3::ZERO,
                &[(0.06, 0.0), (0.035, 0.1)],
                4,
                0.785,
                furt,
                false,
            );
            lathe(
                &mut ear,
                Vec3::ZERO,
                &[(0.035, 0.1), (0.0, 0.17)],
                4,
                0.785,
                tip,
                false,
            );
            lathe(
                &mut ear,
                v(0.0, 0.0, 0.03),
                &[(0.03, 0.01), (0.0, 0.1)],
                4,
                0.785,
                inner,
                false,
            );
            h.append(
                &ear,
                Mat4::from_translation(v(sx * 0.09, 0.22, -0.03))
                    * Mat4::from_rotation_z(-sx * 0.22),
            );
        }
        // A mane of dark spikes down the back of its head and neck.
        for (y, z) in [(0.2, -0.13), (0.1, -0.15), (0.0, -0.15)] {
            let mut m = Mesh::new();
            lathe(
                &mut m,
                Vec3::ZERO,
                &[(0.05, 0.0), (0.0, 0.11)],
                4,
                0.785,
                tip,
                false,
            );
            h.append(
                &m,
                Mat4::from_translation(v(0.0, y, z)) * Mat4::from_rotation_x(-2.0),
            );
        }
        // Tufts of fur bristling off its shoulders.
        for sx in [-1.0f32, 1.0] {
            for (dx, dz) in [(0.0f32, -0.06f32), (0.05, 0.05)] {
                let mut m = Mesh::new();
                lathe(
                    &mut m,
                    Vec3::ZERO,
                    &[(0.05, 0.0), (0.0, 0.12)],
                    4,
                    0.785,
                    furt,
                    false,
                );
                parts[BODY].append(
                    &m,
                    Mat4::from_translation(v(sx * (0.2 + dx), 0.33, dz))
                        * Mat4::from_rotation_z(-sx * 0.7),
                );
            }
        }
        // Claws on its paws.
        let claw = k.c(bank, CREAM);
        for i in [ARM_L, ARM_R] {
            for x in [-0.04f32, 0.0, 0.04] {
                let mut m = Mesh::new();
                lathe(
                    &mut m,
                    Vec3::ZERO,
                    &[(0.014, 0.0), (0.0, 0.07)],
                    4,
                    0.785,
                    claw,
                    false,
                );
                parts[i].append(
                    &m,
                    Mat4::from_translation(v(x, -0.35, 0.045)) * Mat4::from_rotation_x(2.6),
                );
            }
        }
        // Torn trouser legs, and furry feet with claws.
        for i in [LEG_L, LEG_R] {
            k.bx(
                bank,
                &mut parts[i],
                v(-0.085, -0.22, -0.08),
                v(0.085, -0.17, 0.13),
                fur,
            );
            for x in [-0.045f32, 0.0, 0.045] {
                k.bx(
                    bank,
                    &mut parts[i],
                    v(x - 0.01, -0.22, 0.13),
                    v(x + 0.01, -0.2, 0.165),
                    CREAM,
                );
            }
            // Ragged hems.
            for x in [-0.06f32, 0.02] {
                k.bx(
                    bank,
                    &mut parts[i],
                    v(x, -0.19, -0.085),
                    v(x + 0.04, -0.15, 0.085),
                    torn,
                );
            }
        }
    });
    // A bushy tail, thickest in the middle, a pale tip; it hangs down and back.
    let mut tail = Mesh::new();
    let furt = k.c(bank, fur);
    let pale = k.c(bank, light);
    lathe(
        &mut tail,
        Vec3::ZERO,
        &[(0.04, 0.0), (0.09, 0.12), (0.08, 0.24), (0.05, 0.3)],
        6,
        0.0,
        furt,
        false,
    );
    lathe(
        &mut tail,
        Vec3::ZERO,
        &[(0.05, 0.3), (0.0, 0.36)],
        6,
        0.0,
        pale,
        false,
    );
    let mut hang = Mesh::new();
    hang.append(&tail, Mat4::from_rotation_x(-2.3));
    (wolf, hang)
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
        [KHAKI, SHADOW, INK],
    ];
    let [cl, cm, cd] = cloaks[biome];
    let bones = biome == SEWER_LOOK;
    let mut head = if bones {
        goblin_skull(skin, LIME)
    } else {
        goblin_face(skin, if biome == 5 { WHITE } else { GOLD })
    };
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
    let front: &[&str] = if bones {
        // Ribs showing where the cloak falls open.
        &["cmmmmc", "cmlkmc", "cmklmc", "dkkkkd", "mdmdmd"]
    } else {
        &["cmmmmc", "cmllmc", "cmmmmc", "dkkkkd", "mdmdmd"]
    };
    let body = body_tex(
        front,
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
        if bones {
            grime(bank, k, &mut parts[BODY], v(0.0, 0.27, 0.0), 0.12, 0.09);
        }
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
        // Bleached in the sewers, and slimed.
        [WHITE, SAND, KHAKI],
    ];
    let glows = [LIME, AQUA, PINK, ORANGE, SKY, GOLD, LIME];
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
        SEWER_LOOK => {
            // A skull for a head, sat on the sheet and tipped back to look up at you: a round
            // crown, sockets with a green light in them, a hole for a nose, and a jaw full of
            // teeth.
            let mut skull = Mesh::new();
            let bone = k.c(bank, WHITE);
            lathe(
                &mut skull,
                Vec3::ZERO,
                &[
                    (0.0, 0.0),
                    (0.14, 0.02),
                    (0.175, 0.12),
                    (0.155, 0.22),
                    (0.09, 0.28),
                    (0.0, 0.3),
                ],
                10,
                0.3,
                bone,
                false,
            );
            for sx in [-1.0f32, 1.0] {
                k.bx(
                    bank,
                    &mut skull,
                    v(sx * 0.07 - 0.045, 0.08, 0.14),
                    v(sx * 0.07 + 0.045, 0.17, 0.19),
                    INK,
                );
                k.bx(
                    bank,
                    &mut skull,
                    v(sx * 0.07 - 0.016, 0.105, 0.186),
                    v(sx * 0.07 + 0.016, 0.14, 0.196),
                    LIME,
                );
            }
            k.bx(
                bank,
                &mut skull,
                v(-0.02, 0.03, 0.14),
                v(0.02, 0.065, 0.19),
                INK,
            );
            k.bx(
                bank,
                &mut skull,
                v(-0.12, -0.08, -0.06),
                v(0.12, 0.02, 0.13),
                SAND,
            );
            k.bx(
                bank,
                &mut skull,
                v(-0.11, -0.005, 0.125),
                v(0.11, 0.008, 0.14),
                INK,
            );
            for x in 0..4 {
                let x0 = -0.1 + x as f32 * 0.052;
                k.bx(
                    bank,
                    &mut skull,
                    v(x0, -0.045, 0.125),
                    v(x0 + 0.04, 0.0, 0.142),
                    WHITE,
                );
            }
            m.append(
                &skull,
                Mat4::from_translation(v(0.0, 0.64, 0.02)) * Mat4::from_rotation_x(-0.45),
            );
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

/// One segment of a jointed leg, `len` long along +x and `t` thick, banded in two colours
/// and tapering a little towards its end (with a dark claw on a shin).
fn segment(bank: &mut TexBank, k: &mut Kit, len: f32, t: f32, c: [u8; 2], claw: bool) -> Mesh {
    let mut m = Mesh::new();
    let bands = ((len / 0.07).round() as usize).clamp(1, 5);
    for i in 0..bands {
        let x0 = len * i as f32 / bands as f32;
        let x1 = len * (i + 1) as f32 / bands as f32;
        let w = t * (1.0 - 0.3 * i as f32 / bands as f32);
        k.bx(bank, &mut m, v(x0, -w, -w), v(x1, w, w), c[i % 2]);
    }
    // A knuckle at the joint.
    k.bx(
        bank,
        &mut m,
        v(-t * 1.2, -t * 1.2, -t * 1.2),
        v(t * 1.2, t * 1.2, t * 1.2),
        c[1],
    );
    if claw {
        k.bx(
            bank,
            &mut m,
            v(len - 0.01, -t * 0.6, -t * 0.6),
            v(len + 0.03, t * 0.6, t * 0.6),
            INK,
        );
    }
    m
}

/// A round, fuzzy body part: a lathe `r` across and `h` tall sitting at `at`.
fn blob(m: &mut Mesh, at: Vec3, r: f32, h: f32, tex: TexId) {
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
        8,
        0.2,
        tex,
        false,
    );
}

/// An abdomen's pattern: a fuzzy base with a bright chevron marking down its back.
fn spider_skin(bank: &mut TexBank, pal: [u8; 3], mark: u8) -> TexId {
    let mut t = Texture::new(16, 16, pal[1]);
    for y in 0..16 {
        for x in 0..16 {
            let n = (x * 7 + y * 13) % 11;
            if n == 0 {
                t.set(x, y, pal[0]);
            } else if n == 5 {
                t.set(x, y, pal[2]);
            }
        }
    }
    // Chevrons pointing forwards down the middle of the back.
    for (i, y) in [3, 6, 9, 12].into_iter().enumerate() {
        let w = 4 - i as i32;
        for d in 0..=w {
            t.set(7 - d, y + d / 2, mark);
            t.set(8 + d, y + d / 2, mark);
        }
    }
    bank.add(t)
}

/// A spider: a small head-and-chest up front with a cluster of eyes and a pair of fangs, a
/// big round abdomen behind, and eight long legs arching up to knees as high as its back
/// and down to the floor well out to the sides. `pal` colours it; `legs` bands its legs.
fn spider(bank: &mut TexBank, k: &mut Kit, pal: [u8; 3], mark: u8, legs: [u8; 2], eyes: u8) -> Bug {
    let mut body = Mesh::new();
    let skin = spider_skin(bank, pal, mark);
    let head = speck(bank, [pal[0], pal[1], pal[2]]);
    // Both slung low between the legs, a little oval: the abdomen behind, the head in front.
    let mut part = Mesh::new();
    blob(&mut part, Vec3::ZERO, 0.17, 0.26, skin);
    body.append(
        &part,
        Mat4::from_translation(v(0.0, 0.07, -0.23)) * Mat4::from_scale(v(1.0, 1.0, 1.15)),
    );
    let mut part = Mesh::new();
    blob(&mut part, Vec3::ZERO, 0.1, 0.14, head);
    body.append(
        &part,
        Mat4::from_translation(v(0.0, 0.07, 0.06)) * Mat4::from_scale(v(1.0, 1.0, 1.25)),
    );
    // A waist joining them.
    k.bx(
        bank,
        &mut body,
        v(-0.04, 0.11, -0.08),
        v(0.04, 0.17, -0.02),
        pal[2],
    );
    for sx in [-1.0f32, 1.0] {
        // Eight eyes: a big pair up front with a glint in each, and three little ones
        // either side above them.
        k.bx(
            bank,
            &mut body,
            v(sx * 0.037 - 0.018, 0.145, 0.155),
            v(sx * 0.037 + 0.018, 0.185, 0.18),
            eyes,
        );
        k.bx(
            bank,
            &mut body,
            v(sx * 0.037 - 0.008, 0.172, 0.176),
            v(sx * 0.037 + 0.004, 0.182, 0.182),
            WHITE,
        );
        for (dx, y, z) in [
            (0.016f32, 0.197f32, 0.13f32),
            (0.06, 0.18, 0.14),
            (0.074, 0.155, 0.15),
        ] {
            k.bx(
                bank,
                &mut body,
                v(sx * dx - 0.01, y - 0.01, z - 0.01),
                v(sx * dx + 0.01, y + 0.01, z + 0.012),
                eyes,
            );
        }
        // Fangs, curving down under the eyes.
        k.bx(
            bank,
            &mut body,
            v(sx * 0.03 - 0.017, 0.07, 0.15),
            v(sx * 0.03 + 0.017, 0.125, 0.2),
            INK,
        );
        k.bx(
            bank,
            &mut body,
            v(sx * 0.026 - 0.01, 0.035, 0.175),
            v(sx * 0.026 + 0.01, 0.08, 0.2),
            MAROON,
        );
    }
    // A spinneret at the tip of the abdomen.
    k.bx(
        bank,
        &mut body,
        v(-0.025, 0.13, -0.44),
        v(0.025, 0.17, -0.4),
        pal[2],
    );
    Bug {
        body,
        thigh: segment(bank, k, 0.26, 0.028, legs, false),
        shin: segment(bank, k, 0.42, 0.022, legs, true),
        thigh_len: 0.26,
        shin_len: 0.42,
        // Four pairs round the head-and-chest, fanning from well forwards to well back.
        legs: vec![(0.12, 1.05), (0.08, 0.4), (0.04, -0.35), (0.0, -1.0)],
        hip_x: 0.07,
        hip_y: 0.14,
        knee: 0.95,
        wing: None,
        claw: None,
        flies: false,
    }
}

fn bug(bank: &mut TexBank, k: &mut Kit, biome: usize) -> Bug {
    let mut body = Mesh::new();
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
    match biome {
        SEWER_LOOK => {
            // Bone spider: bleached pale as a skeleton, ribs down its back, eyes lit red.
            let mut b = spider(bank, k, [WHITE, SAND, KHAKI], SHADOW, [SAND, WHITE], RED);
            grime(bank, k, &mut b.body, v(0.0, 0.33, -0.24), 0.08, 0.1);
            b
        }
        0 => {
            // Moss spider: green and fuzzy, with moss growing on its back.
            let mut b = spider(bank, k, [LIME, GREEN, TEAL], GOLD, [GREEN, DEEP_TEAL], RED);
            moss(bank, k, &mut b.body, v(-0.04, 0.31, -0.26), 0.9);
            moss(bank, k, &mut b.body, v(0.05, 0.3, -0.17), 0.6);
            b
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
            Bug {
                body: std::mem::take(&mut body),
                thigh: segment(bank, k, 0.15, 0.015, [AQUA, TEAL], false),
                shin: segment(bank, k, 0.3, 0.012, [AQUA, TEAL], true),
                thigh_len: 0.15,
                shin_len: 0.3,
                legs: vec![(0.02, 0.35), (-0.12, -0.55)],
                hip_x: 0.05,
                hip_y: 0.17,
                knee: 0.7,
                wing: None,
                claw: Some(c),
                flies: false,
            }
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
            Bug {
                body: std::mem::take(&mut body),
                thigh: segment(bank, k, 0.06, 0.01, [PURPLE, GRAPE], false),
                shin: segment(bank, k, 0.09, 0.008, [PURPLE, GRAPE], true),
                thigh_len: 0.06,
                shin_len: 0.09,
                legs: vec![(0.05, 0.4), (0.0, 0.0), (-0.05, -0.4)],
                hip_x: 0.05,
                hip_y: 0.02,
                knee: -0.5,
                wing: Some(w),
                claw: None,
                flies: true,
            }
        }
        3 => {
            // Fire ant: a head with mandibles and elbowed feelers, a chest the legs come
            // from, a pinched waist, and a big glowing tail.
            let shell = speck(bank, [RED, CRIMSON, MAROON]);
            let glow = speck(bank, [GOLD, ORANGE, RED]);
            // The tail, lying back from the waist.
            let mut tail = Mesh::new();
            blob(&mut tail, Vec3::ZERO, 0.13, 0.3, glow);
            body.append(
                &tail,
                Mat4::from_translation(v(0.0, 0.15, -0.1))
                    * Mat4::from_rotation_x(-PI * 0.5 - 0.25),
            );
            // Waist, chest and head.
            k.bx(
                bank,
                &mut body,
                v(-0.025, 0.13, -0.1),
                v(0.025, 0.17, -0.04),
                MAROON,
            );
            blob(&mut body, v(0.0, 0.09, 0.02), 0.07, 0.13, shell);
            blob(&mut body, v(0.0, 0.1, 0.16), 0.08, 0.14, shell);
            eye(bank, k, &mut body, 0.2, 0.2, 0.05, INK);
            for s in [-1.0f32, 1.0] {
                // Mandibles.
                k.bx(
                    bank,
                    &mut body,
                    v(s * 0.04 - 0.015, 0.1, 0.22),
                    v(s * 0.04 + 0.015, 0.13, 0.29),
                    MAROON,
                );
                k.bx(
                    bank,
                    &mut body,
                    v(s * 0.02 - 0.01, 0.1, 0.27),
                    v(s * 0.02 + 0.015, 0.12, 0.3),
                    INK,
                );
                // Elbowed feelers: up, then forward.
                k.bx(
                    bank,
                    &mut body,
                    v(s * 0.035 - 0.008, 0.22, 0.18),
                    v(s * 0.035 + 0.008, 0.33, 0.196),
                    MAROON,
                );
                k.bx(
                    bank,
                    &mut body,
                    v(s * 0.045 - 0.008, 0.32, 0.18),
                    v(s * 0.045 + 0.008, 0.336, 0.3),
                    MAROON,
                );
            }
            Bug {
                body: std::mem::take(&mut body),
                thigh: segment(bank, k, 0.14, 0.014, [CRIMSON, MAROON], false),
                shin: segment(bank, k, 0.26, 0.011, [CRIMSON, MAROON], true),
                thigh_len: 0.14,
                shin_len: 0.26,
                legs: vec![(0.05, 0.75), (0.02, 0.05), (-0.01, -0.7)],
                hip_x: 0.05,
                hip_y: 0.13,
                knee: 0.75,
                wing: None,
                claw: None,
                flies: false,
            }
        }
        4 => {
            // Frost spider: pale as snow, crystals of ice growing from its back, eyes like
            // cold coals.
            let mut b = spider(bank, k, [WHITE, SKY, BLUE], AQUA, [SKY, SLATE], RED);
            for (x, y, z, h) in [
                (-0.06f32, 0.28f32, -0.28f32, 0.08f32),
                (0.05, 0.29, -0.2, 0.07),
                (0.0, 0.29, -0.34, 0.06),
            ] {
                crystal(
                    bank,
                    k,
                    &mut b.body,
                    v(x, y, z),
                    v(x * 3.0, 1.0, (z + 0.23) * 3.0),
                    h,
                    [WHITE, SKY],
                );
            }
            b
        }
        _ => {
            // Scarab: a gold and teal dome, a head with a horn, and six sturdy legs.
            let shell = speck(bank, [GOLD, CLAY, RUST]);
            let jewel = speck(bank, [MINT, TEAL, DEEP_TEAL]);
            lathe(
                &mut body,
                v(0.0, 0.08, -0.04),
                &[
                    (0.0, 0.0),
                    (0.17, 0.02),
                    (0.19, 0.1),
                    (0.13, 0.19),
                    (0.0, 0.22),
                ],
                8,
                0.0,
                shell,
                false,
            );
            // The line where its wing cases meet.
            tiled_box(
                &mut body,
                v(-0.012, 0.12, -0.22),
                v(0.012, 0.31, 0.12),
                jewel,
                0,
            );
            // Head and horn.
            tiled_box(
                &mut body,
                v(-0.07, 0.08, 0.12),
                v(0.07, 0.17, 0.22),
                jewel,
                0,
            );
            crystal(
                bank,
                k,
                &mut body,
                v(0.0, 0.15, 0.2),
                v(0.0, 1.0, 0.8),
                0.14,
                [GOLD, CREAM],
            );
            eye(bank, k, &mut body, 0.14, 0.22, 0.045, INK);
            Bug {
                body: std::mem::take(&mut body),
                thigh: segment(bank, k, 0.11, 0.018, [RUST, MAROON], false),
                shin: segment(bank, k, 0.21, 0.015, [RUST, MAROON], true),
                thigh_len: 0.11,
                shin_len: 0.21,
                legs: vec![(0.08, 0.7), (0.0, 0.05), (-0.08, -0.65)],
                hip_x: 0.12,
                hip_y: 0.12,
                knee: 0.55,
                wing: None,
                claw: None,
                flies: false,
            }
        }
    }
}

/// A horn (or any tapering spike) from `at` along `dir`: `len` long, `r0` thick at its
/// root and `r1` at its end.
#[allow(clippy::too_many_arguments)]
fn taper(m: &mut Mesh, at: Vec3, dir: Vec3, len: f32, r0: f32, r1: f32, tex: TexId) {
    let mut p = Mesh::new();
    lathe(
        &mut p,
        Vec3::ZERO,
        &[(r0, 0.0), (r1, len)],
        6,
        0.5,
        tex,
        false,
    );
    let rot = glam::Quat::from_rotation_arc(Vec3::Y, dir.normalize());
    m.append(&p, Mat4::from_translation(at) * Mat4::from_quat(rot));
}

/// A minotaur: a great bull's head on a hulking body in chestnut hide, pale horns sweeping
/// out and up, a pink muzzle with a gold ring through it, glaring red eyes under a dark
/// forelock, and a shaggy dark mane over its shoulders; a loincloth, bronze armbands, and
/// cloven hooves. Its tufted tail hangs down behind.
fn minotaur(bank: &mut TexBank, k: &mut Kit) -> (Humanoid, Mesh) {
    let (hide, dark, light) = (RUST, MAROON, CLAY);
    let (muzzle, cloth, belt) = (ROSEWOOD, KHAKI, SHADOW);
    let legend = [
        ('h', hide),
        ('d', dark),
        ('l', light),
        ('r', RED),
        ('k', INK),
        ('c', cloth),
        ('b', belt),
    ];
    let head = head_tex(
        &[
            "dddddddd", "hddhhddh", "hrkhhkrh", "hhhhhhhh", "hhhhhhhh", "hhhhhhhh", "hhhhhhhh",
        ],
        &legend,
        hide,
        dark,
        dark,
        light,
    );
    let body = body_tex(
        &["hllllh", "hlhhlh", "hhllhh", "bbbbbb", "cccccc"],
        &legend,
        hide,
        belt,
        dark,
    );
    let arm = limb(hide, dark, dark);
    let leg = limb(hide, dark, INK);
    let b = Build {
        head: v(0.17, 0.25, 0.16),
        body: v(0.3, 0.4, 0.2),
        arm: Vec2::new(0.09, 0.4),
        leg: Vec2::new(0.1, 0.26),
    };
    let bull = assemble(bank, [head, body, arm, leg], &b, &mut |bank, parts| {
        let h = &mut parts[HEAD];
        // A broad pink muzzle, dark nostrils, and a gold ring through its nose.
        k.bx(bank, h, v(-0.12, 0.0, 0.14), v(0.12, 0.13, 0.26), muzzle);
        k.bx(bank, h, v(-0.1, 0.12, 0.14), v(0.1, 0.16, 0.22), hide);
        for sx in [-1.0f32, 1.0] {
            k.bx(
                bank,
                h,
                v(sx * 0.055 - 0.022, 0.065, 0.259),
                v(sx * 0.055 + 0.022, 0.1, 0.263),
                INK,
            );
        }
        // A pale lip over its jaw.
        k.bx(bank, h, v(-0.1, 0.0, 0.2), v(0.1, 0.02, 0.262), SALMON);
        let gold = k.c(bank, GOLD);
        let mut ring = Mesh::new();
        lathe(
            &mut ring,
            Vec3::ZERO,
            &[(0.035, -0.012), (0.05, 0.0), (0.035, 0.012), (0.03, 0.0)],
            8,
            0.0,
            gold,
            false,
        );
        h.append(
            &ring,
            Mat4::from_translation(v(0.0, 0.02, 0.27)) * Mat4::from_rotation_x(PI / 2.0),
        );
        // Horns out from the sides of its head and up, pale, and white at the tips.
        let horn = k.c(bank, SAND);
        let tip = k.c(bank, WHITE);
        for sx in [-1.0f32, 1.0] {
            taper(
                h,
                v(sx * 0.15, 0.2, 0.02),
                v(sx, 0.25, 0.12),
                0.17,
                0.05,
                0.036,
                horn,
            );
            taper(
                h,
                v(sx * 0.31, 0.24, 0.05),
                v(sx * 0.35, 1.0, 0.25),
                0.15,
                0.036,
                0.0,
                tip,
            );
            // Ears sticking out under them, pink inside.
            let (x0, x1) = if sx < 0.0 {
                (-0.26, -0.16)
            } else {
                (0.16, 0.26)
            };
            k.bx(bank, h, v(x0, 0.12, -0.03), v(x1, 0.17, 0.04), hide);
            k.bx(
                bank,
                h,
                v(x0 + 0.015, 0.13, 0.04),
                v(x1 - 0.015, 0.16, 0.045),
                muzzle,
            );
        }
        // A dark forelock tumbling between its horns.
        k.bx(bank, h, v(-0.09, 0.2, 0.1), v(0.09, 0.28, 0.18), dark);
        // The mane: shaggy down the back of its head and over its shoulders.
        k.bx(bank, h, v(-0.15, 0.02, -0.19), v(0.15, 0.26, -0.15), dark);
        k.bx(
            bank,
            &mut parts[BODY],
            v(-0.22, 0.3, -0.23),
            v(0.22, 0.43, 0.06),
            dark,
        );
        // A loincloth hanging front and back from its belt.
        k.bx(
            bank,
            &mut parts[BODY],
            v(-0.13, -0.14, 0.2),
            v(0.13, 0.1, 0.215),
            cloth,
        );
        k.bx(
            bank,
            &mut parts[BODY],
            v(-0.15, -0.12, -0.215),
            v(0.15, 0.1, -0.2),
            cloth,
        );
        // Bronze bands on its arms.
        for i in [ARM_L, ARM_R] {
            k.bx(
                bank,
                &mut parts[i],
                v(-0.097, -0.16, -0.097),
                v(0.097, -0.12, 0.097),
                GOLD,
            );
        }
        // Cloven hooves.
        for i in [LEG_L, LEG_R] {
            for (x0, x1) in [(-0.11f32, -0.012f32), (0.012, 0.11)] {
                k.bx(
                    bank,
                    &mut parts[i],
                    v(x0, -0.27, -0.1),
                    v(x1, -0.2, 0.14),
                    INK,
                );
            }
        }
    });
    // The tail: a rope of hide, hanging down, with a dark tuft at the end.
    let mut tail = Mesh::new();
    k.bx(
        bank,
        &mut tail,
        v(-0.022, -0.3, -0.022),
        v(0.022, 0.0, 0.022),
        hide,
    );
    k.bx(
        bank,
        &mut tail,
        v(-0.05, -0.43, -0.05),
        v(0.05, -0.29, 0.05),
        dark,
    );
    (bull, tail)
}

/// A rift ogre: a hulking brute in violet hide, a small head under a heavy brow with red
/// eyes glaring out, tusks jutting up from its underbite, pointed ears and a topknot; a
/// loincloth, iron bands at its wrists, and fists like boulders.
fn ogre(bank: &mut TexBank, k: &mut Kit) -> Humanoid {
    let (hide, dark, light) = (PURPLE, GRAPE, LAVENDER);
    let legend = [
        ('h', hide),
        ('d', dark),
        ('l', light),
        ('r', RED),
        ('k', INK),
        ('w', WHITE),
        ('c', SHADOW),
        ('b', ROSEWOOD),
    ];
    let head = head_tex(
        &[
            "dddddddd", "hkrhhrkh", "hhhddhhh", "hhhhhhhh", "hwhhhhwh", "hkkkkkkh", "hhhhhhhh",
        ],
        &legend,
        hide,
        dark,
        dark,
        light,
    );
    let body = body_tex(
        &["hllllh", "hlhhlh", "hhllhh", "bbbbbb", "cccccc"],
        &legend,
        hide,
        ROSEWOOD,
        dark,
    );
    let arm = limb(hide, dark, dark);
    let leg = limb(hide, dark, INK);
    let b = Build {
        head: v(0.19, 0.25, 0.18),
        body: v(0.34, 0.4, 0.24),
        arm: Vec2::new(0.11, 0.44),
        leg: Vec2::new(0.12, 0.26),
    };
    assemble(bank, [head, body, arm, leg], &b, &mut |bank, parts| {
        let h = &mut parts[HEAD];
        // A heavy brow jutting out over its eyes (not down over them).
        k.bx(bank, h, v(-0.2, 0.214, 0.15), v(0.2, 0.255, 0.215), dark);
        // Tusks up from its underbite.
        let tusk = k.c(bank, CREAM);
        let ear = k.c(bank, dark);
        for sx in [-1.0f32, 1.0] {
            taper(
                h,
                v(sx * 0.12, 0.04, 0.18),
                v(sx * 0.15, 1.0, 0.25),
                0.13,
                0.032,
                0.0,
                tusk,
            );
            // Pointed ears.
            taper(
                h,
                v(sx * 0.19, 0.16, 0.0),
                v(sx, 0.35, -0.2),
                0.11,
                0.045,
                0.0,
                ear,
            );
        }
        // A topknot.
        k.bx(bank, h, v(-0.045, 0.25, -0.06), v(0.045, 0.34, 0.03), INK);
        // A loincloth, front and back.
        k.bx(
            bank,
            &mut parts[BODY],
            v(-0.15, -0.16, 0.24),
            v(0.15, 0.1, 0.255),
            SHADOW,
        );
        k.bx(
            bank,
            &mut parts[BODY],
            v(-0.17, -0.14, -0.255),
            v(0.17, 0.1, -0.24),
            SHADOW,
        );
        // Iron bands at its wrists, and great fists.
        for i in [ARM_L, ARM_R] {
            k.bx(
                bank,
                &mut parts[i],
                v(-0.12, -0.33, -0.12),
                v(0.12, -0.28, 0.12),
                SLATE,
            );
            k.bx(
                bank,
                &mut parts[i],
                v(-0.14, -0.52, -0.14),
                v(0.14, -0.4, 0.14),
                hide,
            );
        }
    })
}

/// A void imp: a little devil in violet, a big head with horns curling up, pointed ears,
/// orange eyes glaring under cross brows, and a fanged grin. Its tail hangs from its hips,
/// a spade at the end.
fn void_imp(bank: &mut TexBank, k: &mut Kit) -> (Humanoid, Mesh) {
    let (skin, dark, light) = (PURPLE, GRAPE, LAVENDER);
    let legend = [
        ('s', skin),
        ('d', dark),
        ('l', light),
        ('e', ORANGE),
        ('k', INK),
        ('w', WHITE),
    ];
    let head = head_tex(
        &[
            "ssllllss", "dksssskd", "skeddeks", "ssssssss", "sssddsss", "skwkkwks", "sskkkkss",
        ],
        &legend,
        skin,
        dark,
        dark,
        light,
    );
    let body = body_tex(
        &["ssllss", "slssls", "ssssss", "dddddd", "ssssss"],
        &legend,
        skin,
        dark,
        dark,
    );
    let arm = limb(skin, dark, dark);
    let leg = limb(skin, dark, INK);
    let b = Build {
        head: v(0.21, 0.32, 0.19),
        body: v(0.13, 0.2, 0.1),
        arm: Vec2::new(0.045, 0.2),
        leg: Vec2::new(0.05, 0.16),
    };
    let imp = assemble(bank, [head, body, arm, leg], &b, &mut |bank, parts| {
        let h = &mut parts[HEAD];
        let horn = k.c(bank, SHADOW);
        let tip = k.c(bank, KHAKI);
        let ear = k.c(bank, dark);
        for sx in [-1.0f32, 1.0] {
            taper(
                h,
                v(sx * 0.14, 0.3, 0.02),
                v(sx * 0.6, 1.0, 0.1),
                0.14,
                0.05,
                0.03,
                horn,
            );
            taper(
                h,
                v(sx * 0.21, 0.42, 0.04),
                v(sx * 0.1, 1.0, -0.3),
                0.08,
                0.03,
                0.0,
                tip,
            );
            taper(
                h,
                v(sx * 0.2, 0.16, 0.0),
                v(sx, 0.3, -0.1),
                0.12,
                0.05,
                0.0,
                ear,
            );
        }
    });
    let mut tail = Mesh::new();
    let t = k.c(bank, dark);
    taper(
        &mut tail,
        v(0.0, 0.0, 0.0),
        v(0.0, -0.4, -1.0),
        0.28,
        0.022,
        0.016,
        t,
    );
    let spade = v(0.0, -0.1, -0.26);
    k.bx(
        bank,
        &mut tail,
        spade - v(0.05, 0.05, 0.02),
        spade + v(0.05, 0.05, 0.02),
        INK,
    );
    taper(
        &mut tail,
        spade + v(0.0, 0.0, -0.01),
        v(0.0, -0.3, -1.0),
        0.1,
        0.05,
        0.0,
        t,
    );
    (imp, tail)
}

// ------------------------------------------------------------------------------------------
// Weapons
// ------------------------------------------------------------------------------------------

/// A minotaur's axe: a long haft bound in bronze, and a pair of crescent blades at its
/// head, one either way. Gripped at the origin and pointing down the arm.
fn labrys(bank: &mut TexBank, k: &mut Kit) -> Mesh {
    let mut m = Mesh::new();
    let wood = speck(bank, [CLAY, RUST, MAROON]);
    tiled_box(
        &mut m,
        v(-0.026, -0.74, -0.026),
        v(0.026, 0.12, 0.026),
        wood,
        0,
    );
    for y in [0.02f32, -0.58] {
        k.bx(
            bank,
            &mut m,
            v(-0.034, y - 0.04, -0.034),
            v(0.034, y, 0.034),
            GOLD,
        );
    }
    // (Out to either side, so they show whichever way the arm swings.)
    let steel = speck(bank, [WHITE, SKY, SLATE]);
    let mid = -0.66;
    for side in [-1.0f32, 1.0] {
        for (x0, x1, half) in [
            (0.03f32, 0.09f32, 0.07f32),
            (0.09, 0.15, 0.1),
            (0.15, 0.2, 0.13),
        ] {
            let (a, b) = if side > 0.0 { (x0, x1) } else { (-x1, -x0) };
            tiled_box(
                &mut m,
                v(a, mid - half, -0.012),
                v(b, mid + half, 0.012),
                steel,
                0,
            );
        }
    }
    m
}

/// A fat goblin's club, gripped at the origin and pointing down the arm.
fn club(bank: &mut TexBank, k: &mut Kit, biome: usize) -> Mesh {
    let mut m = Mesh::new();
    // In the sewers the club is a great old thigh bone, knobbly end down.
    let wood = speck(
        bank,
        if biome == SEWER_LOOK {
            [WHITE, SAND, KHAKI]
        } else {
            [CLAY, RUST, MAROON]
        },
    );
    tiled_box(&mut m, v(-0.03, -0.3, -0.03), v(0.03, 0.06, 0.03), wood, 0);
    let (head, tip) = match biome {
        SEWER_LOOK => ([WHITE, SAND, KHAKI], SAND),
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
    // (The sewer's are rusty.)
    let grip = [TEAL, INDIGO, GRAPE, MAROON, SLATE, RUST, SHADOW][biome];
    let blade = [WHITE, AQUA, BLUSH, ORANGE, SKY, GOLD, RUST][biome];
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
    let (werewolf, wolf_tail) = werewolf(bank, &mut k);
    let (minotaur, bull_tail) = minotaur(bank, &mut k);
    let labrys = labrys(bank, &mut k);
    let ogre = ogre(bank, &mut k);
    let (void_imp, imp_tail) = void_imp(bank, &mut k);
    let ghost_pals = [
        [MINT, LIME, GREEN],
        [WHITE, SKY, AQUA],
        [BLUSH, PINK, LAVENDER],
        [CREAM, GOLD, ORANGE],
        [WHITE, WHITE, SKY],
        [WHITE, BLUSH, LAVENDER],
        // Sewer gas.
        [LIME, GREEN, TEAL],
    ];
    Monsters {
        zombie: (0..LOOKS).map(|b| zombie(bank, &mut k, b)).collect(),
        brute: (0..LOOKS).map(|b| brute(bank, &mut k, b)).collect(),
        club: (0..LOOKS).map(|b| club(bank, &mut k, b)).collect(),
        sneak: (0..LOOKS).map(|b| sneak(bank, &mut k, b)).collect(),
        dagger: (0..LOOKS).map(|b| dagger(bank, &mut k, b)).collect(),
        skeleton: (0..LOOKS).map(|b| skeleton(bank, &mut k, b)).collect(),
        ghost: ghost_pals.iter().map(|p| ghost_sheet(bank, *p)).collect(),
        ghost_face,
        ghost_hat: (0..LOOKS).map(|b| ghost_hat(bank, &mut k, b)).collect(),
        bug: (0..LOOKS).map(|b| bug(bank, &mut k, b)).collect(),
        werewolf,
        wolf_tail,
        minotaur,
        bull_tail,
        labrys,
        ogre,
        void_imp,
        imp_tail,
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
            assert_eq!(list.len(), LOOKS);
            for h in list.iter() {
                assert!(h.parts.iter().all(|p| !p.tris.is_empty()));
                assert!(h.hip > 0.0 && h.neck > h.hip && h.shoulder_x > 0.0);
            }
        }
        // Fat goblins are wider and shorter in the leg than skinny ones.
        assert!(m.brute[0].shoulder_x > m.sneak[0].shoulder_x);
        assert!(m.brute[0].hip < m.sneak[0].hip);
        assert_eq!(m.bug.len(), LOOKS);
        assert!(m.bug.iter().any(|b| b.flies && b.wing.is_some()));
        assert!(m.bug.iter().any(|b| b.claw.is_some()));
        for b in &m.bug {
            assert!(!b.body.tris.is_empty() && !b.legs.is_empty());
            // A walker's feet reach down to the floor from its knees.
            let knee = b.hip_y + b.thigh_len * b.knee.sin();
            assert!(b.flies || b.shin_len >= knee, "feet off the floor");
        }
        // Spiders (moss and frost) stand on eight legs, knees high, reaching out wide.
        for b in [&m.bug[0], &m.bug[4]] {
            assert_eq!(b.legs.len(), 4);
            assert!(b.hip_y + b.thigh_len * b.knee.sin() > 0.3);
            assert!(b.hip_x + b.thigh_len * b.knee.cos() + b.shin_len * 0.5 > 0.4);
        }
        assert_eq!(m.ghost.len(), LOOKS);
        assert_eq!(m.ghost_hat.len(), LOOKS);
        assert_eq!(m.club.len(), LOOKS);
        assert_eq!(m.dagger.len(), LOOKS);
        // The sewers' spider walks on its bones; it doesn't fly.
        assert!(!m.bug[SEWER_LOOK].flies);
    }
}
