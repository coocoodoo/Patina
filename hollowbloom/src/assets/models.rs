//! Low-poly models built from boxes and lathes, textured with small pixel-art skins.
//! Local space: origin on the ground at the tile centre, +y up, the front faces +z (towards
//! the camera).

use glam::{Mat4, Vec3};

use super::tiles;
use crate::palette::*;
use crate::render::mesh::{FRONT, TOP};
use crate::render::{BoxUv, Mesh, TexBank, TexId, Texture, UvRect};
use crate::util::Rng;

const TD: f32 = 16.0; // texels per world unit

/// A box whose faces tile a texture at world density.
pub fn tiled_box(m: &mut Mesh, min: Vec3, max: Vec3, tex: TexId, skip: u8) {
    let s = (max - min) * TD;
    let r = |w: f32, h: f32| UvRect::new(0.0, 0.0, w, h);
    let uv = BoxUv([
        r(s.z, s.y),
        r(s.z, s.y),
        r(s.x, s.z),
        r(s.x, s.z),
        r(s.x, s.y),
        r(s.x, s.y),
    ]);
    m.cube(min, max, &uv, tex, skip);
}

/// A box with the whole texture stretched over every face.
pub fn skin_box(m: &mut Mesh, min: Vec3, max: Vec3, tex: TexId, t: &Texture) {
    m.cube(
        min,
        max,
        &BoxUv::all(UvRect::new(0.0, 0.0, t.w as f32, t.h as f32)),
        tex,
        0,
    );
}

pub fn lathe(
    m: &mut Mesh,
    c: Vec3,
    prof: &[(f32, f32)],
    seg: usize,
    phase: f32,
    tex: TexId,
    caps: bool,
) {
    // v runs top to bottom over the profile at world density.
    let top = prof.last().map(|p| p.1).unwrap_or(0.0);
    let p: Vec<(f32, f32, f32)> = prof.iter().map(|&(r, y)| (r, y, (top - y) * TD)).collect();
    let cap = if caps {
        Some(UvRect::new(0.0, 0.0, 16.0, 16.0))
    } else {
        None
    };
    m.lathe(c, &p, seg, TD * 2.0, phase, tex, (cap, cap));
}

// ------------------------------------------------------------------------------------------
// Characters
// ------------------------------------------------------------------------------------------

pub const HEAD: usize = 0;
pub const BODY: usize = 1;
pub const ARM_L: usize = 2;
pub const ARM_R: usize = 3;
pub const LEG_L: usize = 4;
pub const LEG_R: usize = 5;

/// A chibi humanoid split into parts, each built around its pivot:
/// legs hang from the hip, arms from the shoulder, the head sits on the neck.
/// The textures a humanoid is dressed in, so clothes can be swapped in.
#[derive(Clone, Copy, Debug, Default)]
pub struct HumanTex {
    pub head: TexId,
    pub body: TexId,
    pub arm: TexId,
    pub leg: TexId,
}

#[derive(Clone)]
pub struct Humanoid {
    pub parts: [Mesh; 6],
    /// The head with its hair tucked under a hat (nothing poking through the crown), and
    /// with no loose hair at all, for hoods.
    pub head_tucked: Mesh,
    pub head_hooded: Mesh,
    pub tex: HumanTex,
    pub hip: f32,
    pub shoulder: f32,
    pub neck: f32,
    pub shoulder_x: f32,
    pub hip_x: f32,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Hair {
    Fluffy,
    Bald,
    /// Falls down the back.
    Long,
    /// Tied up on top.
    Bun,
    /// Two little tails.
    Pigtails,
    /// A rounded bob down to the chin.
    Bob,
    /// Tufts sticking up.
    Spiky,
}

#[derive(Clone, Copy)]
pub struct Look {
    pub hair: [u8; 3],
    pub skin: [u8; 3],
    pub eyes: u8,
    pub cheeks: u8,
    pub shirt: [u8; 3],
    pub belt: u8,
    pub pants: u8,
    pub boots: u8,
    pub style: Hair,
    pub beard: bool,
    pub scale: f32,
}

/// Where the arms hang from, before scaling: height and distance out from the middle.
pub const SHOULDER: f32 = 0.43;
pub const SHOULDER_X: f32 = 0.22;

pub const HERO: Look = Look {
    hair: [GOLD, CLAY, RUST],
    skin: [PEACH, PEACH, SALMON],
    eyes: INK,
    cheeks: SALMON,
    shirt: [AQUA, TEAL, DEEP_TEAL],
    belt: SAND,
    pants: INDIGO,
    boots: RUST,
    style: Hair::Fluffy,
    beard: false,
    scale: 1.0,
};

pub fn head_tex(l: &Look) -> Texture {
    let [hl, hm, hd] = l.hair;
    let [sl, sm, sd] = l.skin;
    let mut t = Texture::new(32, 16, hm);
    let bald = matches!(l.style, Hair::Bald);
    if bald {
        t = Texture::new(32, 16, sm);
    }
    // Front 8x7 at (0,0).
    for y in 0..7 {
        for x in 0..8 {
            t.set(x, y, sm);
        }
    }
    match l.style {
        Hair::Fluffy | Hair::Long | Hair::Bun | Hair::Pigtails | Hair::Bob | Hair::Spiky => {
            for x in 0..8 {
                t.set(x, 0, hm);
                t.set(x, 1, if x % 3 == 1 { hl } else { hm });
            }
            for (x, y) in [(0, 2), (1, 2), (3, 2), (6, 2), (7, 2), (0, 3), (7, 3)] {
                t.set(x, y, hm);
            }
            t.set(4, 2, hd);
        }
        Hair::Bald => {
            for x in 0..8 {
                t.set(x, 0, sl);
            }
        }
    }
    // Eyes and cheeks.
    for (x, y) in [(2, 3), (2, 4), (5, 3), (5, 4)] {
        t.set(x, y, l.eyes);
    }
    t.set(1, 5, l.cheeks);
    t.set(6, 5, l.cheeks);
    t.set(3, 6, sd);
    t.set(4, 6, sd);
    // Sides 7x7 at (8,0) and (23,0): hair with skin low at the front edge.
    for y in 0..7 {
        for x in 0..7 {
            let c = if bald || (y >= 4 && x <= 1) {
                sm
            } else if y == 0 {
                hl
            } else {
                hm
            };
            t.set(8 + x, y, c);
            t.set(23 + 6 - x, y, c);
        }
    }
    // Back 8x7 at (15,0).
    for y in 0..7 {
        for x in 0..8 {
            let c = if bald {
                sm
            } else if y == 6 {
                hd
            } else if (x + y) % 5 == 0 {
                hl
            } else {
                hm
            };
            t.set(15 + x, y, c);
        }
    }
    // Top 8x7 at (0,7), bottom at (8,7).
    for y in 0..7 {
        for x in 0..8 {
            let top = if bald {
                sl
            } else if (x == 3 || x == 4) && y > 1 {
                hl
            } else {
                hm
            };
            t.set(x, 7 + y, top);
            t.set(8 + x, 7 + y, sd);
        }
    }
    t
}

pub fn body_tex(l: &Look) -> Texture {
    let [cl, cm, cd] = l.shirt;
    let mut t = Texture::new(16, 16, cm);
    // Front 6x5 at (0,0): collar, tunic, belt with buckle.
    for x in 0..6 {
        t.set(x, 0, cl);
        t.set(x, 3, l.belt);
        t.set(x, 4, cd);
    }
    t.set(2, 3, GOLD);
    t.set(3, 3, GOLD);
    t.set(0, 1, cd);
    t.set(5, 1, cd);
    // Back 6x5 at (6,0).
    for x in 0..6 {
        t.set(6 + x, 3, l.belt);
        t.set(6 + x, 4, cd);
        t.set(6 + x, 0, cd);
    }
    // Sides 4x5 at (12,0).
    for x in 0..4 {
        t.set(12 + x, 3, l.belt);
        t.set(12 + x, 4, cd);
    }
    // Top/bottom 6x4 at (0,5) and (6,5).
    for y in 0..4 {
        for x in 0..6 {
            t.set(x, 5 + y, cl);
            t.set(6 + x, 5 + y, cd);
        }
    }
    t
}

pub fn limb_tex(main: u8, dark: u8, end: u8) -> Texture {
    let mut t = Texture::new(4, 4, main);
    for x in 0..4 {
        t.set(x, 3, end);
    }
    t.set(3, 0, dark);
    t.set(3, 1, dark);
    t
}

pub fn humanoid(bank: &mut TexBank, l: &Look) -> Humanoid {
    let s = l.scale;
    let head_t = head_tex(l);
    let body_t = body_tex(l);
    let arm_t = limb_tex(l.shirt[1], l.shirt[2], l.skin[1]);
    let leg_t = limb_tex(l.pants, INK, l.boots);
    let head = bank.add(head_t);
    let body = bank.add(body_t);
    let arm = bank.add(arm_t);
    let leg = bank.add(leg_t);

    let v = |x: f32, y: f32, z: f32| Vec3::new(x, y, z) * s;
    let mut parts: [Mesh; 6] = Default::default();
    let head_uv = BoxUv([
        UvRect::px(8, 0, 7, 7),
        UvRect::px(23, 0, 7, 7),
        UvRect::px(0, 7, 8, 7),
        UvRect::px(8, 7, 8, 7),
        UvRect::px(0, 0, 8, 7),
        UvRect::px(15, 0, 8, 7),
    ]);
    parts[HEAD].cube(v(-0.26, 0.0, -0.23), v(0.26, 0.46, 0.23), &head_uv, head, 0);
    let mut head_tucked = parts[HEAD].clone();
    let mut head_hooded = parts[HEAD].clone();
    hair_extras(bank, &mut parts[HEAD], l, HairFit::Loose);
    hair_extras(bank, &mut head_tucked, l, HairFit::Tucked);
    hair_extras(bank, &mut head_hooded, l, HairFit::Hooded);
    let body_uv = BoxUv([
        UvRect::px(12, 0, 4, 5),
        UvRect::px(12, 0, 4, 5),
        UvRect::px(0, 5, 6, 4),
        UvRect::px(6, 5, 6, 4),
        UvRect::px(0, 0, 6, 5),
        UvRect::px(6, 0, 6, 5),
    ]);
    parts[BODY].cube(v(-0.17, 0.0, -0.12), v(0.17, 0.28, 0.12), &body_uv, body, 0);
    let limb_uv = BoxUv::all(UvRect::px(0, 0, 4, 4));
    for (i, sx) in [(ARM_L, -1.0f32), (ARM_R, 1.0)] {
        let _ = sx;
        parts[i].cube(v(-0.05, -0.24, -0.05), v(0.05, 0.0, 0.05), &limb_uv, arm, 0);
    }
    for i in [LEG_L, LEG_R] {
        parts[i].cube(v(-0.06, -0.18, -0.06), v(0.06, 0.0, 0.06), &limb_uv, leg, 0);
    }
    Humanoid {
        parts,
        head_tucked,
        head_hooded,
        tex: HumanTex {
            head,
            body,
            arm,
            leg,
        },
        hip: 0.18 * s,
        shoulder: SHOULDER * s,
        neck: 0.43 * s,
        shoulder_x: SHOULDER_X * s,
        hip_x: 0.08 * s,
    }
}

/// How loose hair sits: all of it, tucked under a hat's brim, or hidden by a hood.
#[derive(Clone, Copy, PartialEq, Eq)]
enum HairFit {
    Loose,
    Tucked,
    Hooded,
}

/// Where a hat's brim sits on the head: tucked hair stops just below it.
const HAT_LINE: f32 = 0.33;

/// Hair beyond the head box itself, and beards.
fn hair_extras(bank: &mut TexBank, m: &mut Mesh, l: &Look, fit: HairFit) {
    let s = l.scale;
    let v = |x: f32, y: f32, z: f32| Vec3::new(x, y, z) * s;
    let [hl, hm, hd] = l.hair;
    let mid = bank.add(tiles::solid(hm));
    let dark = bank.add(tiles::solid(hd));
    let light = bank.add(tiles::solid(hl));
    let w4 = Texture::new(4, 4, 0);
    // Under a hat, anything that would reach the crown is trimmed at the brim.
    let skin_box = |m: &mut Mesh, a: Vec3, b: Vec3, t: TexId, w: &Texture| {
        if fit == HairFit::Tucked {
            let top = HAT_LINE * s;
            if a.y >= top {
                return;
            }
            skin_box(m, a, Vec3::new(b.x, b.y.min(top), b.z), t, w);
        } else {
            skin_box(m, a, b, t, w);
        }
    };
    let style = if fit == HairFit::Hooded {
        Hair::Bald
    } else {
        l.style
    };
    match style {
        Hair::Long => {
            skin_box(m, v(-0.27, -0.22, -0.27), v(0.27, 0.42, -0.18), mid, &w4);
            skin_box(m, v(-0.29, -0.1, -0.2), v(-0.25, 0.4, 0.02), mid, &w4);
            skin_box(m, v(0.25, -0.1, -0.2), v(0.29, 0.4, 0.02), mid, &w4);
        }
        Hair::Bun => {
            skin_box(m, v(-0.11, 0.44, -0.2), v(0.11, 0.62, 0.0), mid, &w4);
            skin_box(m, v(-0.05, 0.6, -0.14), v(0.05, 0.66, -0.06), light, &w4);
        }
        Hair::Pigtails => {
            for sx in [-1.0f32, 1.0] {
                let (a, b) = if sx < 0.0 {
                    (-0.37, -0.25)
                } else {
                    (0.25, 0.37)
                };
                skin_box(m, v(a, 0.02, -0.12), v(b, 0.34, 0.04), mid, &w4);
                skin_box(
                    m,
                    v(a + 0.02, 0.32, -0.08),
                    v(b - 0.02, 0.37, 0.0),
                    dark,
                    &w4,
                );
            }
        }
        Hair::Bob => {
            skin_box(m, v(-0.29, 0.06, -0.26), v(0.29, 0.47, -0.2), mid, &w4);
            skin_box(m, v(-0.29, 0.06, -0.2), v(-0.25, 0.47, 0.14), mid, &w4);
            skin_box(m, v(0.25, 0.06, -0.2), v(0.29, 0.47, 0.14), mid, &w4);
        }
        Hair::Spiky if fit == HairFit::Tucked => {}
        Hair::Spiky => {
            for (x, z, t) in [
                (-0.14f32, 0.02f32, -0.5f32),
                (0.0, -0.06, 0.0),
                (0.14, 0.04, 0.5),
            ] {
                let mut tuft = Mesh::new();
                skin_box(
                    &mut tuft,
                    Vec3::new(-0.06, 0.0, -0.06) * s,
                    Vec3::new(0.06, 0.14, 0.06) * s,
                    if x == 0.0 { light } else { mid },
                    &w4,
                );
                m.append(
                    &tuft,
                    Mat4::from_translation(v(x, 0.44, z)) * Mat4::from_rotation_z(t * 0.6),
                );
            }
        }
        Hair::Fluffy | Hair::Bald => {}
    }
    if l.beard {
        skin_box(m, v(-0.21, -0.06, 0.17), v(0.21, 0.13, 0.27), mid, &w4);
        skin_box(m, v(-0.12, -0.13, 0.18), v(0.12, -0.05, 0.26), dark, &w4);
        // A moustache.
        skin_box(m, v(-0.14, 0.12, 0.23), v(0.14, 0.17, 0.27), dark, &w4);
    }
}

/// The little sprout on the hero's head.
pub fn sprout(bank: &mut TexBank) -> Mesh {
    let leaf = bank.add(Texture::new(4, 4, LIME));
    let stem = bank.add(Texture::new(4, 4, GREEN));
    let mut m = Mesh::new();
    skin_box(
        &mut m,
        Vec3::new(-0.015, 0.0, -0.015),
        Vec3::new(0.015, 0.12, 0.015),
        stem,
        &Texture::new(4, 4, 0),
    );
    let mut l = Mesh::new();
    skin_box(
        &mut l,
        Vec3::new(0.0, -0.015, -0.035),
        Vec3::new(0.12, 0.015, 0.035),
        leaf,
        &Texture::new(4, 4, 0),
    );
    m.append(
        &l,
        Mat4::from_translation(Vec3::new(0.01, 0.11, 0.0)) * Mat4::from_rotation_z(0.5),
    );
    m.append(
        &l,
        Mat4::from_translation(Vec3::new(-0.01, 0.1, 0.0))
            * Mat4::from_rotation_z(std::f32::consts::PI - 0.5),
    );
    m
}

/// Two small dark eyes on a front surface at height `y`, `z` in front.
fn eyes(m: &mut Mesh, tex: TexId, y: f32, z: f32, gap: f32, size: f32) {
    let w = Texture::new(4, 4, 0);
    for sx in [-1.0, 1.0] {
        let c = Vec3::new(sx * gap, y, z);
        skin_box(
            m,
            c - Vec3::new(size * 0.45, size * 0.7, 0.0),
            c + Vec3::new(size * 0.45, size * 0.7, 0.03),
            tex,
            &w,
        );
    }
}

pub struct Critters {
    /// The see-through jelly body.
    pub slime: Mesh,
    /// What floats inside it.
    pub slime_core: Mesh,
    /// Eyes and mouth, drawn solid on the surface.
    pub slime_face: Mesh,
    pub bat_body: Mesh,
    pub bat_wing: Mesh,
    pub shroom: Mesh,
    pub crab: Mesh,
    pub wisp: Mesh,
    pub beetle: Mesh,
    pub golem: Mesh,
    pub ghost: Mesh,
    pub ghost_face: Mesh,
    pub imp: Humanoid,
    pub skeleton: Humanoid,
    pub mole: Mesh,
    pub cat: Mesh,
    pub cat_tail: Mesh,
    /// Water folk: a bog frog (and a back leg), a drift jelly and a puffer.
    pub frog: Mesh,
    pub frog_leg: Mesh,
    pub jelly_bell: Mesh,
    pub jelly_core: Mesh,
    pub jelly_threads: Mesh,
    pub puffer: Mesh,
    pub puffer_spikes: Mesh,
    pub puffer_fin: Mesh,
    pub frog_tex: TexId,
    pub puffer_tex: TexId,
    /// Texture ids that palette variants swap.
    pub slime_tex: TexId,
    pub slime_core_tex: TexId,
    pub cap_tex: TexId,
    pub shell_tex: TexId,
    pub wisp_tex: TexId,
    pub fur_tex: TexId,
    pub wing_tex: TexId,
    pub beetle_tex: TexId,
    pub golem_tex: TexId,
}

fn slime_skin(pal: [u8; 3]) -> Texture {
    let [hi, mid, lo] = pal;
    let mut t = Texture::new(16, 16, mid);
    for y in 0..16 {
        for x in 0..16 {
            if y > 11 {
                t.set(x, y, lo);
            } else if y < 3 && (x % 8) < 3 {
                t.set(x, y, hi);
            }
        }
    }
    t.set(3, 4, WHITE);
    t.set(11, 4, WHITE);
    t
}

/// Jelly: a bright crown, a clear body and a deeper colour where it pools at the bottom.
fn jelly_skin(pal: [u8; 3]) -> Texture {
    let [hi, mid, lo] = pal;
    let mut t = Texture::new(16, 8, mid);
    for y in 0..8 {
        for x in 0..16 {
            let c = match y {
                0 => hi,
                1 if (x + y) % 3 != 0 => hi,
                2 if x % 4 == 0 => hi,
                6 if x % 2 == 0 => lo,
                7 => lo,
                _ => mid,
            };
            t.set(x, y, c);
        }
    }
    // A couple of bubbles caught in the surface.
    t.set(4, 3, hi);
    t.set(12, 4, hi);
    t
}

pub fn critters(bank: &mut TexBank) -> Critters {
    let eye = bank.add(Texture::new(4, 4, INK));
    let white = bank.add(Texture::new(4, 4, WHITE));
    let w4 = Texture::new(4, 4, 0);

    // Slime: a squat dome of jelly with something floating in the middle, and a face.
    let slime_tex = bank.add(jelly_skin([LIME, GREEN, TEAL]));
    let mut slime = Mesh::new();
    lathe(
        &mut slime,
        Vec3::ZERO,
        &[
            (0.0, 0.0),
            (0.34, 0.0),
            (0.41, 0.07),
            (0.43, 0.16),
            (0.39, 0.27),
            (0.3, 0.37),
            (0.16, 0.43),
            (0.0, 0.45),
        ],
        10,
        0.31,
        slime_tex,
        true,
    );
    // The nucleus: a dark, knobbly heart that shows through the jelly.
    let mut core_t = Texture::new(8, 8, TEAL);
    for (x, y) in [(1, 1), (2, 1), (5, 4)] {
        core_t.set(x, y, GREEN);
    }
    let slime_core_tex = bank.add(core_t);
    let mut slime_core = Mesh::new();
    lathe(
        &mut slime_core,
        Vec3::ZERO,
        &[
            (0.0, -0.12),
            (0.13, -0.08),
            (0.17, 0.0),
            (0.12, 0.09),
            (0.0, 0.13),
        ],
        6,
        0.2,
        slime_core_tex,
        false,
    );
    // Big eyes that sit proud of the jelly, each with a shine, and a little smile.
    let mut slime_face = Mesh::new();
    eyes(&mut slime_face, eye, 0.22, 0.418, 0.125, 0.12);
    for sx in [-1.0f32, 1.0] {
        let c = Vec3::new(sx * 0.125 - 0.018, 0.25, 0.45);
        skin_box(
            &mut slime_face,
            c - Vec3::new(0.017, 0.017, 0.0),
            c + Vec3::new(0.017, 0.017, 0.012),
            white,
            &w4,
        );
    }
    skin_box(
        &mut slime_face,
        Vec3::new(-0.045, 0.115, 0.425),
        Vec3::new(0.045, 0.14, 0.45),
        eye,
        &w4,
    );

    // Bat.
    let fur = bank.add(Texture::new(4, 4, GRAPE));
    let fur_bat = fur;
    let mut membrane = Texture::new(16, 8, PURPLE);
    for x in 0..16 {
        membrane.set(x, 0, GRAPE);
        if x % 5 == 0 {
            for y in 0..8 {
                membrane.set(x, y, GRAPE);
            }
        }
    }
    for y in 5..8 {
        for x in 0..16 {
            if x % 5 > 7 - y {
                membrane.set(x, y, CLEAR);
            }
        }
    }
    let wing_t = bank.add(membrane);
    let mut bat_body = Mesh::new();
    lathe(
        &mut bat_body,
        Vec3::ZERO,
        &[
            (0.0, -0.16),
            (0.14, -0.12),
            (0.18, 0.0),
            (0.14, 0.12),
            (0.0, 0.16),
        ],
        6,
        0.3,
        fur,
        false,
    );
    skin_box(
        &mut bat_body,
        Vec3::new(-0.12, 0.1, -0.03),
        Vec3::new(-0.05, 0.24, 0.03),
        fur,
        &w4,
    );
    skin_box(
        &mut bat_body,
        Vec3::new(0.05, 0.1, -0.03),
        Vec3::new(0.12, 0.24, 0.03),
        fur,
        &w4,
    );
    let red_eye = bank.add(Texture::new(4, 4, CREAM));
    eyes(&mut bat_body, red_eye, 0.03, 0.15, 0.06, 0.06);
    let mut bat_wing = Mesh::new();
    bat_wing.quad(
        [
            Vec3::new(0.0, 0.0, 0.08),
            Vec3::new(0.42, -0.06, 0.08),
            Vec3::new(0.42, 0.0, -0.12),
            Vec3::new(0.0, 0.0, -0.1),
        ],
        UvRect::px(0, 0, 16, 8),
        wing_t,
    );

    // Shroomling: stem with a face under a spotted cap.
    let stem = bank.add(tiles::solid(SAND));
    let mut cap_t = Texture::new(16, 16, RED);
    for (x, y) in [
        (2, 3),
        (3, 3),
        (9, 5),
        (10, 5),
        (9, 6),
        (5, 10),
        (13, 11),
        (14, 11),
    ] {
        cap_t.set(x, y, WHITE);
    }
    for x in 0..16 {
        cap_t.set(x, 15, CRIMSON);
        cap_t.set(x, 14, CRIMSON);
    }
    let cap_tex = bank.add(cap_t);
    let mut shroom = Mesh::new();
    lathe(
        &mut shroom,
        Vec3::ZERO,
        &[(0.14, 0.0), (0.17, 0.12), (0.15, 0.3)],
        6,
        0.0,
        stem,
        false,
    );
    eyes(&mut shroom, eye, 0.17, 0.16, 0.06, 0.06);
    lathe(
        &mut shroom,
        Vec3::ZERO,
        &[
            (0.18, 0.26),
            (0.42, 0.3),
            (0.4, 0.42),
            (0.26, 0.54),
            (0.0, 0.58),
        ],
        8,
        0.39,
        cap_tex,
        true,
    );

    // Crystal crab.
    let shell_tex = bank.add(tiles::crystal([MINT, AQUA, TEAL]));
    let claw = bank.add(tiles::solid(TEAL));
    let mut crab = Mesh::new();
    lathe(
        &mut crab,
        Vec3::ZERO,
        &[
            (0.0, 0.08),
            (0.34, 0.08),
            (0.36, 0.2),
            (0.22, 0.32),
            (0.0, 0.34),
        ],
        6,
        0.52,
        shell_tex,
        true,
    );
    for sx in [-1.0f32, 1.0] {
        skin_box(
            &mut crab,
            Vec3::new(sx * 0.3 - 0.08, 0.06, 0.18),
            Vec3::new(sx * 0.3 + 0.08, 0.18, 0.36),
            claw,
            &w4,
        );
        for k in 0..3 {
            let z = -0.14 + k as f32 * 0.14;
            skin_box(
                &mut crab,
                Vec3::new(sx * 0.3 - 0.1, 0.0, z - 0.02),
                Vec3::new(sx * 0.3 + 0.1, 0.06, z + 0.02),
                claw,
                &w4,
            );
        }
        skin_box(
            &mut crab,
            Vec3::new(sx * 0.08 - 0.02, 0.3, 0.12),
            Vec3::new(sx * 0.08 + 0.02, 0.42, 0.16),
            claw,
            &w4,
        );
        skin_box(
            &mut crab,
            Vec3::new(sx * 0.08 - 0.035, 0.42, 0.1),
            Vec3::new(sx * 0.08 + 0.035, 0.48, 0.17),
            eye,
            &w4,
        );
    }

    // Wisp: a glowing octahedron.
    let wisp_tex = bank.add(tiles::crystal([WHITE, MINT, AQUA]));
    let mut wisp = Mesh::new();
    lathe(
        &mut wisp,
        Vec3::ZERO,
        &[(0.0, -0.22), (0.2, 0.0), (0.0, 0.22)],
        4,
        0.0,
        wisp_tex,
        false,
    );
    eyes(&mut wisp, eye, 0.02, 0.15, 0.06, 0.06);

    // Beetle: shiny dome and little legs.
    let shell2 = bank.add(tiles::crystal([LAVENDER, PURPLE, GRAPE]));
    let mut beetle = Mesh::new();
    lathe(
        &mut beetle,
        Vec3::ZERO,
        &[
            (0.0, 0.06),
            (0.32, 0.06),
            (0.34, 0.16),
            (0.24, 0.28),
            (0.0, 0.32),
        ],
        6,
        0.52,
        shell2,
        true,
    );
    let leg = bank.add(tiles::solid(INK));
    for sx in [-1.0f32, 1.0] {
        for k in 0..3 {
            let z = -0.14 + k as f32 * 0.14;
            skin_box(
                &mut beetle,
                Vec3::new(sx * 0.3 - 0.08, 0.0, z - 0.02),
                Vec3::new(sx * 0.3 + 0.08, 0.08, z + 0.02),
                leg,
                &w4,
            );
        }
    }
    skin_box(
        &mut beetle,
        Vec3::new(-0.04, 0.12, 0.28),
        Vec3::new(0.04, 0.2, 0.42),
        leg,
        &w4,
    );
    eyes(&mut beetle, white, 0.14, 0.3, 0.1, 0.06);

    // Golem: stacked stone blocks with glowing eyes.
    let rock = bank.add(tiles::rock_side([KHAKI, ROSEWOOD, SHADOW, INK], 3));
    let glow = bank.add(Texture::new(4, 4, AQUA));
    let mut golem = Mesh::new();
    tiled_box(
        &mut golem,
        Vec3::new(-0.3, 0.0, -0.2),
        Vec3::new(-0.1, 0.3, 0.2),
        rock,
        0,
    );
    tiled_box(
        &mut golem,
        Vec3::new(0.1, 0.0, -0.2),
        Vec3::new(0.3, 0.3, 0.2),
        rock,
        0,
    );
    tiled_box(
        &mut golem,
        Vec3::new(-0.42, 0.28, -0.3),
        Vec3::new(0.42, 0.8, 0.3),
        rock,
        0,
    );
    tiled_box(
        &mut golem,
        Vec3::new(-0.62, 0.3, -0.14),
        Vec3::new(-0.42, 0.78, 0.14),
        rock,
        0,
    );
    tiled_box(
        &mut golem,
        Vec3::new(0.42, 0.3, -0.14),
        Vec3::new(0.62, 0.78, 0.14),
        rock,
        0,
    );
    tiled_box(
        &mut golem,
        Vec3::new(-0.22, 0.8, -0.2),
        Vec3::new(0.22, 1.12, 0.2),
        rock,
        0,
    );
    eyes(&mut golem, glow, 0.98, 0.2, 0.09, 0.08);

    // Ghost: a soft teardrop.
    let sheet = bank.add(slime_skin([WHITE, BLUSH, LAVENDER]));
    let mut ghost = Mesh::new();
    lathe(
        &mut ghost,
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
    let mut ghost_face = Mesh::new();
    eyes(&mut ghost_face, eye, 0.42, 0.305, 0.1, 0.1);

    let imp = humanoid(
        bank,
        &Look {
            hair: [RED, CRIMSON, PLUM],
            skin: [SALMON, RED, CRIMSON],
            eyes: CREAM,
            cheeks: CRIMSON,
            shirt: [GRAPE, GRAPE, INK],
            belt: GOLD,
            pants: INK,
            boots: INK,
            style: Hair::Bald,
            beard: false,
            scale: 0.85,
        },
    );
    let skeleton = humanoid(
        bank,
        &Look {
            hair: [WHITE, SAND, KHAKI],
            skin: [WHITE, SAND, KHAKI],
            eyes: INK,
            cheeks: SAND,
            shirt: [SAND, KHAKI, ROSEWOOD],
            belt: RUST,
            pants: KHAKI,
            boots: ROSEWOOD,
            style: Hair::Bald,
            beard: false,
            scale: 0.95,
        },
    );

    // Mole merchant: round and soft, pink nose, a lamp helmet.
    let moleskin = bank.add(slime_skin([ROSEWOOD, SHADOW, INK]));
    let helmet = bank.add(tiles::solid(GOLD));
    let nose = bank.add(tiles::solid(PINK));
    let lampglass = bank.add(tiles::solid(CREAM));
    let apron = bank.add(tiles::solid(TEAL));
    let mut mole = Mesh::new();
    lathe(
        &mut mole,
        Vec3::ZERO,
        &[
            (0.0, 0.0),
            (0.3, 0.0),
            (0.36, 0.25),
            (0.32, 0.55),
            (0.2, 0.75),
            (0.0, 0.8),
        ],
        8,
        0.39,
        moleskin,
        true,
    );
    skin_box(
        &mut mole,
        Vec3::new(-0.2, 0.08, 0.26),
        Vec3::new(0.2, 0.42, 0.34),
        apron,
        &w4,
    );
    skin_box(
        &mut mole,
        Vec3::new(-0.06, 0.5, 0.28),
        Vec3::new(0.06, 0.6, 0.4),
        nose,
        &w4,
    );
    eyes(&mut mole, eye, 0.64, 0.25, 0.1, 0.05);
    lathe(
        &mut mole,
        Vec3::new(0.0, 0.7, 0.0),
        &[(0.26, 0.0), (0.24, 0.1), (0.0, 0.16)],
        8,
        0.39,
        helmet,
        true,
    );
    skin_box(
        &mut mole,
        Vec3::new(-0.05, 0.74, 0.2),
        Vec3::new(0.05, 0.84, 0.28),
        lampglass,
        &w4,
    );

    // Cat.
    let fur = bank.add(slime_skin([SAND, GOLD, CLAY]));
    let mut cat = Mesh::new();
    skin_box(
        &mut cat,
        Vec3::new(-0.12, 0.08, -0.22),
        Vec3::new(0.12, 0.26, 0.16),
        fur,
        &Texture::new(16, 16, 0),
    );
    skin_box(
        &mut cat,
        Vec3::new(-0.14, 0.18, 0.1),
        Vec3::new(0.14, 0.42, 0.34),
        fur,
        &Texture::new(16, 16, 0),
    );
    for sx in [-1.0f32, 1.0] {
        skin_box(
            &mut cat,
            Vec3::new(sx * 0.09 - 0.04, 0.42, 0.18),
            Vec3::new(sx * 0.09 + 0.04, 0.5, 0.24),
            fur,
            &Texture::new(16, 16, 0),
        );
        for z in [-0.16f32, 0.1] {
            skin_box(
                &mut cat,
                Vec3::new(sx * 0.07 - 0.03, 0.0, z - 0.03),
                Vec3::new(sx * 0.07 + 0.03, 0.1, z + 0.03),
                fur,
                &Texture::new(16, 16, 0),
            );
        }
    }
    eyes(&mut cat, eye, 0.32, 0.34, 0.06, 0.05);
    skin_box(
        &mut cat,
        Vec3::new(-0.025, 0.26, 0.34),
        Vec3::new(0.025, 0.29, 0.36),
        nose,
        &w4,
    );
    let mut cat_tail = Mesh::new();
    skin_box(
        &mut cat_tail,
        Vec3::new(-0.025, 0.0, -0.3),
        Vec3::new(0.025, 0.05, 0.0),
        fur,
        &Texture::new(16, 16, 0),
    );

    // Bog frog: a squat body with a pale belly, bulging eyes on top and a wide smile.
    let mut frog_t = Texture::new(16, 16, GREEN);
    for y in 0..16 {
        for x in 0..16 {
            if y > 10 {
                frog_t.set(x, y, LIME);
            } else if y < 4 || (x * 7 + y * 3) % 11 == 0 {
                frog_t.set(x, y, TEAL);
            }
        }
    }
    let frog_tex = bank.add(frog_t);
    let mut frog = Mesh::new();
    lathe(
        &mut frog,
        Vec3::ZERO,
        &[
            (0.0, 0.0),
            (0.24, 0.02),
            (0.3, 0.12),
            (0.27, 0.23),
            (0.17, 0.3),
            (0.0, 0.32),
        ],
        8,
        0.39,
        frog_tex,
        true,
    );
    for sx in [-1.0f32, 1.0] {
        let c = Vec3::new(sx * 0.12, 0.3, 0.1);
        skin_box(
            &mut frog,
            c - Vec3::new(0.065, 0.05, 0.065),
            c + Vec3::new(0.065, 0.06, 0.065),
            frog_tex,
            &w4,
        );
        skin_box(
            &mut frog,
            c + Vec3::new(-0.04, -0.03, 0.06),
            c + Vec3::new(0.04, 0.04, 0.075),
            white,
            &w4,
        );
        skin_box(
            &mut frog,
            c + Vec3::new(-0.02, -0.02, 0.074),
            c + Vec3::new(0.02, 0.03, 0.085),
            eye,
            &w4,
        );
        // Front feet.
        skin_box(
            &mut frog,
            Vec3::new(sx * 0.16 - 0.05, 0.0, 0.14),
            Vec3::new(sx * 0.16 + 0.05, 0.05, 0.28),
            frog_tex,
            &w4,
        );
    }
    skin_box(
        &mut frog,
        Vec3::new(-0.14, 0.13, 0.27),
        Vec3::new(0.14, 0.15, 0.29),
        eye,
        &w4,
    );
    let mut frog_leg = Mesh::new();
    skin_box(
        &mut frog_leg,
        Vec3::new(-0.05, -0.06, -0.18),
        Vec3::new(0.05, 0.06, 0.04),
        frog_tex,
        &w4,
    );
    skin_box(
        &mut frog_leg,
        Vec3::new(-0.06, -0.08, -0.26),
        Vec3::new(0.06, -0.04, -0.14),
        frog_tex,
        &w4,
    );

    // Drift jelly: a glassy bell, a soft glowing heart, and trailing threads.
    let jelly_t = bank.add(jelly_skin([WHITE, LAVENDER, PURPLE]));
    let mut jelly_bell = Mesh::new();
    lathe(
        &mut jelly_bell,
        Vec3::ZERO,
        &[
            (0.31, -0.02),
            (0.33, 0.07),
            (0.29, 0.19),
            (0.18, 0.28),
            (0.0, 0.31),
        ],
        10,
        0.31,
        jelly_t,
        false,
    );
    let mut heart_t = Texture::new(8, 8, PINK);
    for (x, y) in [(1, 1), (5, 2), (3, 5)] {
        heart_t.set(x, y, BLUSH);
    }
    let heart = bank.add(heart_t);
    let mut jelly_core = Mesh::new();
    lathe(
        &mut jelly_core,
        Vec3::ZERO,
        &[(0.0, 0.03), (0.14, 0.07), (0.12, 0.15), (0.0, 0.19)],
        6,
        0.2,
        heart,
        false,
    );
    let mut thread_t = Texture::new(4, 16, LAVENDER);
    for y in 0..16 {
        thread_t.set(1, y, if y % 4 == 0 { WHITE } else { BLUSH });
        if y > 11 {
            thread_t.set(0, y, CLEAR);
            thread_t.set(3, y, CLEAR);
        }
    }
    let thread = bank.add(thread_t);
    let mut jelly_threads = Mesh::new();
    for k in 0..6 {
        let a = k as f32 / 6.0 * std::f32::consts::TAU;
        let r = if k % 2 == 0 { 0.2 } else { 0.12 };
        let (x, z) = (a.cos() * r, a.sin() * r);
        let (dx, dz) = (-a.sin() * 0.04, a.cos() * 0.04);
        let len = if k % 2 == 0 { 0.5 } else { 0.38 };
        jelly_threads.quad(
            [
                Vec3::new(x - dx, -len, z - dz),
                Vec3::new(x + dx, -len, z + dz),
                Vec3::new(x + dx, 0.0, z + dz),
                Vec3::new(x - dx, 0.0, z - dz),
            ],
            UvRect::px(0, 0, 4, 16),
            thread,
        );
    }

    // Puffer: a round fish with big eyes, spines for when it blows itself up, and fins.
    let mut puff_t = Texture::new(16, 16, SAND);
    for y in 0..16 {
        for x in 0..16 {
            if y > 10 {
                puff_t.set(x, y, CREAM);
            } else if (x + y * 5) % 7 == 0 {
                puff_t.set(x, y, KHAKI);
            }
        }
    }
    let puffer_tex = bank.add(puff_t);
    let mut puffer = Mesh::new();
    lathe(
        &mut puffer,
        Vec3::ZERO,
        &[
            (0.0, -0.24),
            (0.17, -0.2),
            (0.26, -0.06),
            (0.25, 0.08),
            (0.17, 0.2),
            (0.0, 0.25),
        ],
        8,
        0.39,
        puffer_tex,
        false,
    );
    eyes(&mut puffer, white, 0.06, 0.235, 0.1, 0.1);
    for sx in [-1.0f32, 1.0] {
        skin_box(
            &mut puffer,
            Vec3::new(sx * 0.1 - 0.02, 0.03, 0.262),
            Vec3::new(sx * 0.1 + 0.02, 0.08, 0.272),
            eye,
            &w4,
        );
    }
    skin_box(
        &mut puffer,
        Vec3::new(-0.03, -0.06, 0.24),
        Vec3::new(0.03, -0.02, 0.27),
        bank.add(tiles::solid(CRIMSON)),
        &w4,
    );
    let tailfin = bank.add(tiles::solid(AQUA));
    skin_box(
        &mut puffer,
        Vec3::new(-0.02, -0.08, -0.36),
        Vec3::new(0.02, 0.08, -0.22),
        tailfin,
        &w4,
    );
    let spine = bank.add(tiles::solid(CREAM));
    let mut puffer_spikes = Mesh::new();
    for k in 0..14 {
        let a = k as f32 * 2.4;
        let up = ((k as f32 * 0.7).sin()) * 0.9;
        let dir = Vec3::new(a.cos() * up.cos(), up.sin(), a.sin() * up.cos());
        let mut sp = Mesh::new();
        skin_box(
            &mut sp,
            Vec3::new(-0.012, 0.0, -0.012),
            Vec3::new(0.012, 0.1, 0.012),
            spine,
            &w4,
        );
        let rot = glam::Quat::from_rotation_arc(Vec3::Y, dir.normalize_or_zero());
        puffer_spikes.append(
            &sp,
            Mat4::from_translation(dir * 0.24) * Mat4::from_quat(rot),
        );
    }
    let mut puffer_fin = Mesh::new();
    puffer_fin.quad(
        [
            Vec3::new(0.0, -0.05, -0.02),
            Vec3::new(0.0, -0.05, 0.08),
            Vec3::new(0.0, 0.05, 0.08),
            Vec3::new(0.0, 0.05, -0.02),
        ],
        UvRect::new(0.0, 0.0, 4.0, 4.0),
        tailfin,
    );

    Critters {
        frog,
        frog_leg,
        jelly_bell,
        jelly_core,
        jelly_threads,
        puffer,
        puffer_spikes,
        puffer_fin,
        frog_tex,
        puffer_tex,
        slime,
        slime_core,
        slime_face,
        bat_body,
        bat_wing,
        shroom,
        crab,
        wisp,
        beetle,
        golem,
        ghost,
        ghost_face,
        imp,
        skeleton,
        mole,
        cat,
        cat_tail,
        slime_tex,
        slime_core_tex,
        cap_tex,
        shell_tex,
        wisp_tex,
        fur_tex: fur_bat,
        wing_tex: wing_t,
        beetle_tex: shell2,
        golem_tex: rock,
    }
}

// ------------------------------------------------------------------------------------------
// Props
// ------------------------------------------------------------------------------------------

pub struct Props {
    pub trees: Vec<Mesh>,
    pub pines: Vec<Mesh>,
    pub stump: Mesh,
    pub log: Mesh,
    pub rocks: Vec<Mesh>,
    pub boulder: Mesh,
    pub crystal: Vec<Mesh>,
    pub stalagmite: Vec<Mesh>,
    pub mushroom: Vec<Mesh>,
    pub bones: Mesh,
    pub pot: Mesh,
    pub crate_: Mesh,
    pub chest: Mesh,
    pub chest_open: Mesh,
    pub loot_chest: Mesh,
    pub stairs: Mesh,
    pub waystone: Mesh,
    pub waystone_rune: Mesh,
    pub campfire: Mesh,
    pub torch_stick: Mesh,
    pub lamp: Mesh,
    pub fence_post: Mesh,
    pub fence_rail_e: Mesh,
    pub fence_rail_s: Mesh,
    pub sprinkler: Vec<Mesh>,
    pub workbench: Mesh,
    pub flower_pot: Mesh,
    pub bench: Mesh,
    pub house: Mesh,
    pub house_windows: Mesh,
    pub bin: Mesh,
    pub hollow: Mesh,
    pub stall: Mesh,
    pub sign: Mesh,
    pub enchant_table: Mesh,
    pub enchant_book: Mesh,
    /// Little clumps of grass scattered over the lawns; the last `TALL_TUFTS` are tall.
    pub tufts: Vec<Mesh>,
    /// A leafy weed with dandelions, and a dry one gone to seed.
    pub weeds: Vec<Mesh>,
    /// Wild bushes on the farm: leafy, blueberry and bramble.
    pub shrubs: Vec<Mesh>,
    /// The Ward spell's bubble of light.
    pub bubble: Mesh,
}

/// How many of `Props::tufts` (at the end) are tall meadow grass.
pub const TALL_TUFTS: usize = 2;

/// A strip shading from a light tip down to a dark root, for blades and leaves.
fn blade_tex(tip: u8, mid: u8, root: u8) -> Texture {
    let mut t = Texture::new(4, 16, mid);
    for y in 0..16 {
        for x in 0..4 {
            let c = if y < 7 {
                tip
            } else if y > 11 {
                root
            } else {
                mid
            };
            t.set(x, y, c);
        }
    }
    t
}

/// Fans `n` blades around a centre: `h` tall, `width` at the root, tips leaning out by
/// `lean`. Every blade is lit like the ground it grows from, a little brighter when it
/// leans towards the sun.
#[allow(clippy::too_many_arguments)]
fn blades(
    m: &mut Mesh,
    tex: TexId,
    n: usize,
    h: f32,
    lean: f32,
    width: f32,
    spread: f32,
    r: &mut Rng,
) {
    use std::f32::consts::TAU;
    for j in 0..n {
        let a = j as f32 / n as f32 * TAU + r.range_f(-0.4, 0.4);
        let dir = Vec3::new(a.cos(), 0.0, a.sin());
        let side = Vec3::new(-a.sin(), 0.0, a.cos());
        let c = dir * r.range_f(0.0, spread);
        let w = width * r.range_f(0.8, 1.2) * 0.5;
        let tip = c + dir * (lean * r.range_f(0.6, 1.3)) + Vec3::Y * (h * r.range_f(0.7, 1.1));
        let first = m.verts.len();
        m.tri(
            [c - side * w, c + side * w, tip],
            [
                glam::Vec2::new(0.3, 15.7),
                glam::Vec2::new(3.7, 15.7),
                glam::Vec2::new(2.0, 0.3),
            ],
            tex,
        );
        let normal = (Vec3::Y + dir * 0.5).normalize();
        for v in &mut m.verts[first..] {
            v.n = normal;
        }
    }
}

fn foliage(bank: &mut TexBank) -> (Vec<Mesh>, Vec<Mesh>, Vec<Mesh>) {
    let w4 = Texture::new(4, 4, 0);
    let mut r = Rng::new(0x6A55);
    let lawn = bank.add(blade_tex(LIME, GREEN, TEAL));
    let deep = bank.add(blade_tex(LIME, TEAL, DEEP_TEAL));
    let mut tufts = Vec::new();
    // Short tufts first, then the tall meadow clumps (see `TALL_TUFTS`).
    for (n, h, lean, width, tex) in [
        (5, 0.36, 0.1, 0.085, lawn),
        (7, 0.3, 0.13, 0.075, lawn),
        (4, 0.42, 0.08, 0.075, deep),
        (6, 0.33, 0.15, 0.08, lawn),
        (9, 0.62, 0.14, 0.09, lawn),
        (8, 0.54, 0.18, 0.085, deep),
    ] {
        let mut m = Mesh::new();
        blades(&mut m, tex, n, h, lean, width, 0.06, &mut r);
        tufts.push(m);
    }

    // Weeds: broad, messy rosettes that plainly want pulling.
    let leafy = bank.add(blade_tex(LIME, GREEN, DEEP_TEAL));
    let dry = bank.add(blade_tex(SAND, KHAKI, ROSEWOOD));
    let stalk = bank.add(tiles::solid(GREEN));
    let straw = bank.add(tiles::solid(KHAKI));
    let bloom = bank.add(tiles::solid(GOLD));
    let fluff = bank.add(tiles::solid(WHITE));
    let mut weeds = Vec::new();
    for (tex, stem, head) in [(leafy, stalk, bloom), (dry, straw, fluff)] {
        let mut m = Mesh::new();
        blades(&mut m, tex, 7, 0.3, 0.2, 0.11, 0.06, &mut r);
        blades(&mut m, tex, 5, 0.4, 0.1, 0.06, 0.03, &mut r);
        for (x, z, h) in [(0.05f32, 0.02f32, 0.46f32), (-0.06, -0.03, 0.38)] {
            skin_box(
                &mut m,
                Vec3::new(x - 0.012, 0.0, z - 0.012),
                Vec3::new(x + 0.012, h, z + 0.012),
                stem,
                &w4,
            );
            skin_box(
                &mut m,
                Vec3::new(x - 0.04, h, z - 0.04),
                Vec3::new(x + 0.04, h + 0.07, z + 0.04),
                head,
                &w4,
            );
        }
        weeds.push(m);
    }

    // Wild bushes.
    let leaves = bank.add(tiles::canopy([LIME, GREEN, TEAL, DEEP_TEAL], 61));
    let dark = bank.add(tiles::canopy([GREEN, TEAL, DEEP_TEAL, INK], 62));
    let berry = bank.add(tiles::solid(BLUE));
    let berry_hi = bank.add(tiles::solid(SKY));
    let twig = bank.add(tiles::solid(RUST));
    let petal = bank.add(tiles::solid(BLUSH));
    let mut shrubs = Vec::new();
    for v in 0..3 {
        let mut m = Mesh::new();
        let tex = if v == 2 { dark } else { leaves };
        lathe(
            &mut m,
            Vec3::ZERO,
            &[
                (0.0, 0.0),
                (0.34, 0.04),
                (0.4, 0.24),
                (0.3, 0.46),
                (0.0, 0.54),
            ],
            7,
            v as f32 * 0.7,
            tex,
            false,
        );
        lathe(
            &mut m,
            Vec3::new(0.16, 0.0, -0.1),
            &[(0.0, 0.3), (0.22, 0.36), (0.2, 0.56), (0.0, 0.64)],
            6,
            v as f32,
            tex,
            false,
        );
        match v {
            1 => {
                for i in 0..9 {
                    let a = i as f32 * 0.8 + 0.2;
                    let y = 0.14 + (i % 3) as f32 * 0.12;
                    let rad = 0.4 - (y - 0.14) * 0.35;
                    let p = Vec3::new(a.cos() * rad, y, a.sin() * rad);
                    let t = if i % 3 == 0 { berry_hi } else { berry };
                    skin_box(
                        &mut m,
                        p - Vec3::splat(0.035),
                        p + Vec3::splat(0.035),
                        t,
                        &w4,
                    );
                }
            }
            2 => {
                for i in 0..5 {
                    let a = i as f32 * 1.3;
                    let d = Vec3::new(a.cos(), 0.0, a.sin());
                    let p = d * 0.3 + Vec3::Y * (0.3 + (i % 2) as f32 * 0.14);
                    let end = p + d * 0.16 + Vec3::Y * 0.06;
                    skin_box(
                        &mut m,
                        p.min(end) - Vec3::splat(0.015),
                        p.max(end) + Vec3::splat(0.015),
                        twig,
                        &w4,
                    );
                    let q = p + d * 0.06 + Vec3::Y * 0.12;
                    skin_box(
                        &mut m,
                        q - Vec3::splat(0.03),
                        q + Vec3::splat(0.03),
                        petal,
                        &w4,
                    );
                }
            }
            _ => {}
        }
        shrubs.push(m);
    }
    (tufts, weeds, shrubs)
}

pub fn props(bank: &mut TexBank) -> Props {
    let (tufts, weeds, shrubs) = foliage(bank);
    let shimmer = bank.add(tiles::crystal([WHITE, MINT, AQUA]));
    let mut bubble = Mesh::new();
    lathe(
        &mut bubble,
        Vec3::ZERO,
        &[
            (0.0, -0.02),
            (0.42, 0.12),
            (0.6, 0.55),
            (0.56, 0.95),
            (0.36, 1.28),
            (0.0, 1.4),
        ],
        12,
        0.0,
        shimmer,
        false,
    );
    let w4 = Texture::new(4, 4, 0);
    let bark = bank.add(tiles::bark());
    let rings = bank.add(tiles::rings());
    let leaves = [
        bank.add(tiles::canopy([LIME, GREEN, TEAL, DEEP_TEAL], 1)),
        bank.add(tiles::leaves([CREAM, LIME, GREEN, TEAL], 2)),
        bank.add(tiles::leaves([BLUSH, PINK, CRIMSON, PLUM], 3)),
        bank.add(tiles::leaves([GOLD, CLAY, RUST, MAROON], 4)),
    ];
    let pine_leaves = bank.add(tiles::canopy([GREEN, TEAL, DEEP_TEAL, INK], 5));

    // Round trees: trunk plus two or three leafy blobs.
    let mut trees = Vec::new();
    for (i, &lt) in leaves.iter().enumerate() {
        let mut m = Mesh::new();
        lathe(
            &mut m,
            Vec3::ZERO,
            &[(0.16, 0.0), (0.13, 0.5), (0.12, 0.9)],
            6,
            0.3,
            bark,
            false,
        );
        lathe(
            &mut m,
            Vec3::new(0.0, 0.0, 0.0),
            &[
                (0.0, 0.62),
                (0.62, 0.7),
                (0.74, 1.05),
                (0.6, 1.4),
                (0.0, 1.55),
            ],
            8,
            0.2 + i as f32,
            lt,
            true,
        );
        lathe(
            &mut m,
            Vec3::new(0.12, 0.0, -0.05),
            &[
                (0.0, 1.3),
                (0.4, 1.36),
                (0.44, 1.6),
                (0.3, 1.8),
                (0.0, 1.86),
            ],
            7,
            0.5 + i as f32,
            lt,
            true,
        );
        if i == 2 || i == 1 {
            let fruit = bank.add(tiles::solid(if i == 1 { RED } else { WHITE }));
            for k in 0..5 {
                let a = k as f32 * 1.3;
                let p = Vec3::new(
                    a.cos() * 0.66,
                    0.95 + (k % 2) as f32 * 0.2,
                    a.sin().abs() * 0.5 + 0.1,
                );
                skin_box(
                    &mut m,
                    p - Vec3::splat(0.05),
                    p + Vec3::splat(0.05),
                    fruit,
                    &w4,
                );
            }
        }
        trees.push(m);
    }
    let mut pines = Vec::new();
    for i in 0..2 {
        let mut m = Mesh::new();
        lathe(
            &mut m,
            Vec3::ZERO,
            &[(0.14, 0.0), (0.1, 0.5)],
            6,
            0.3,
            bark,
            false,
        );
        for (k, (r, y0, h)) in [(0.66, 0.35, 0.7), (0.52, 0.8, 0.6), (0.36, 1.2, 0.6)]
            .into_iter()
            .enumerate()
        {
            lathe(
                &mut m,
                Vec3::ZERO,
                &[(0.0, y0), (r, y0 + 0.06), (0.0, y0 + h)],
                7,
                (i * 3 + k) as f32 * 0.7,
                pine_leaves,
                true,
            );
        }
        pines.push(m);
    }
    let mut stump = Mesh::new();
    lathe(
        &mut stump,
        Vec3::ZERO,
        &[(0.24, 0.0), (0.2, 0.18)],
        7,
        0.1,
        bark,
        false,
    );
    lathe(
        &mut stump,
        Vec3::new(0.0, 0.18, 0.0),
        &[(0.2, 0.0), (0.0, 0.001)],
        7,
        0.1,
        rings,
        false,
    );
    let mut log = Mesh::new();
    {
        let mut l = Mesh::new();
        lathe(
            &mut l,
            Vec3::ZERO,
            &[(0.16, -0.36), (0.16, 0.36)],
            6,
            0.0,
            bark,
            false,
        );
        lathe(
            &mut l,
            Vec3::new(0.0, 0.36, 0.0),
            &[(0.16, 0.0), (0.0, 0.001)],
            6,
            0.0,
            rings,
            false,
        );
        lathe(
            &mut l,
            Vec3::new(0.0, -0.36, 0.0),
            &[(0.0, -0.001), (0.16, 0.0)],
            6,
            0.0,
            rings,
            false,
        );
        log.append(
            &l,
            Mat4::from_translation(Vec3::new(0.0, 0.15, 0.0))
                * Mat4::from_rotation_z(std::f32::consts::FRAC_PI_2),
        );
    }

    // Rocks.
    let stone = bank.add(tiles::speckled_stone([SAND, KHAKI, ROSEWOOD, SHADOW], 21));
    let mut rocks = Vec::new();
    let mut r = Rng::new(9);
    for _ in 0..3 {
        let mut m = Mesh::new();
        let s = r.range_f(0.85, 1.1);
        lathe(
            &mut m,
            Vec3::ZERO,
            &[
                (0.0, 0.0),
                (0.3 * s, 0.0),
                (0.34 * s, 0.12),
                (0.28 * s, 0.24 * s),
                (0.14 * s, 0.3 * s),
                (0.0, 0.32 * s),
            ],
            6,
            r.range_f(0.0, 3.0),
            stone,
            true,
        );
        rocks.push(m);
    }
    let mut boulder = Mesh::new();
    lathe(
        &mut boulder,
        Vec3::ZERO,
        &[
            (0.0, 0.0),
            (0.46, 0.0),
            (0.5, 0.25),
            (0.42, 0.5),
            (0.2, 0.66),
            (0.0, 0.68),
        ],
        6,
        0.4,
        stone,
        true,
    );

    // Crystals in a few colours.
    let crystal_tex = vec![
        bank.add(tiles::crystal([MINT, AQUA, TEAL])),
        bank.add(tiles::crystal([WHITE, SKY, BLUE])),
        bank.add(tiles::crystal([BLUSH, LAVENDER, PURPLE])),
        bank.add(tiles::crystal([CREAM, GOLD, ORANGE])),
        bank.add(tiles::crystal([WHITE, MINT, SKY])),
        bank.add(tiles::crystal([CREAM, GOLD, CLAY])),
    ];
    let mut crystal = Vec::new();
    for &ct in &crystal_tex {
        let mut m = Mesh::new();
        for (x, z, h, tilt) in [
            (0.0, 0.0, 0.7, 0.0),
            (0.18, 0.1, 0.45, 0.35),
            (-0.16, 0.08, 0.4, -0.4),
        ] {
            let mut c = Mesh::new();
            lathe(
                &mut c,
                Vec3::ZERO,
                &[(0.1, 0.0), (0.11, h * 0.75), (0.0, h)],
                5,
                0.3,
                ct,
                false,
            );
            m.append(
                &c,
                Mat4::from_translation(Vec3::new(x, 0.0, z)) * Mat4::from_rotation_z(tilt),
            );
        }
        crystal.push(m);
    }
    let mut stalagmite = Vec::new();
    for (i, pal) in [
        [KHAKI, ROSEWOOD, SHADOW, INK],
        [SKY, INDIGO, SLATE, INK],
        [LAVENDER, PURPLE, GRAPE, INK],
    ]
    .into_iter()
    .enumerate()
    {
        let t = bank.add(tiles::rock_side(pal, 40 + i as u64));
        let mut m = Mesh::new();
        lathe(
            &mut m,
            Vec3::ZERO,
            &[(0.3, 0.0), (0.18, 0.4), (0.0, 0.95)],
            5,
            0.2,
            t,
            false,
        );
        lathe(
            &mut m,
            Vec3::new(0.24, 0.0, 0.12),
            &[(0.14, 0.0), (0.0, 0.42)],
            5,
            0.6,
            t,
            false,
        );
        stalagmite.push(m);
    }
    let mushroom_caps = vec![
        bank.add(tiles::crystal([MINT, AQUA, TEAL])),
        bank.add(tiles::crystal([BLUSH, PINK, CRIMSON])),
        bank.add(tiles::crystal([CREAM, GOLD, CLAY])),
    ];
    let stem = bank.add(tiles::solid(SAND));
    let mut mushroom = Vec::new();
    for &cap in &mushroom_caps {
        let mut m = Mesh::new();
        for (x, z, s) in [
            (0.0f32, 0.0f32, 1.0f32),
            (0.2, 0.14, 0.6),
            (-0.18, 0.1, 0.5),
        ] {
            lathe(
                &mut m,
                Vec3::new(x, 0.0, z),
                &[(0.06 * s, 0.0), (0.05 * s, 0.3 * s)],
                5,
                0.0,
                stem,
                false,
            );
            lathe(
                &mut m,
                Vec3::new(x, 0.0, z),
                &[
                    (0.05 * s, 0.26 * s),
                    (0.22 * s, 0.3 * s),
                    (0.18 * s, 0.4 * s),
                    (0.0, 0.44 * s),
                ],
                6,
                0.3,
                cap,
                true,
            );
        }
        mushroom.push(m);
    }
    let bone_t = bank.add(tiles::solid(SAND));
    let bone_w = bank.add(tiles::solid(WHITE));
    let mut bones = Mesh::new();
    skin_box(
        &mut bones,
        Vec3::new(-0.22, 0.0, -0.03),
        Vec3::new(0.18, 0.05, 0.03),
        bone_t,
        &w4,
    );
    skin_box(
        &mut bones,
        Vec3::new(-0.05, 0.0, -0.2),
        Vec3::new(0.0, 0.05, 0.15),
        bone_w,
        &w4,
    );
    lathe(
        &mut bones,
        Vec3::new(0.12, 0.0, 0.12),
        &[(0.1, 0.0), (0.12, 0.08), (0.0, 0.18)],
        6,
        0.0,
        bone_w,
        false,
    );

    let clay = bank.add(tiles::planks(CLAY, GOLD, RUST, 5));
    let mut pot = Mesh::new();
    lathe(
        &mut pot,
        Vec3::ZERO,
        &[
            (0.0, 0.0),
            (0.18, 0.0),
            (0.28, 0.18),
            (0.24, 0.36),
            (0.14, 0.44),
            (0.17, 0.5),
        ],
        7,
        0.2,
        clay,
        true,
    );
    let plank = bank.add(tiles::planks(CLAY, GOLD, RUST, 6));
    let plank_dark = bank.add(tiles::planks(RUST, CLAY, MAROON, 7));
    let metal = bank.add(tiles::solid(GOLD));
    let mut crate_ = Mesh::new();
    tiled_box(
        &mut crate_,
        Vec3::new(-0.3, 0.0, -0.3),
        Vec3::new(0.3, 0.6, 0.3),
        plank,
        0,
    );

    let mut chest = Mesh::new();
    tiled_box(
        &mut chest,
        Vec3::new(-0.32, 0.0, -0.22),
        Vec3::new(0.32, 0.3, 0.22),
        plank,
        0,
    );
    let mut lid = Mesh::new();
    tiled_box(
        &mut lid,
        Vec3::new(-0.34, 0.0, -0.24),
        Vec3::new(0.34, 0.14, 0.24),
        plank_dark,
        0,
    );
    skin_box(
        &mut lid,
        Vec3::new(-0.05, -0.06, 0.24),
        Vec3::new(0.05, 0.06, 0.27),
        metal,
        &w4,
    );
    let mut chest_open = chest.clone();
    chest.append(&lid, Mat4::from_translation(Vec3::new(0.0, 0.3, 0.0)));
    chest_open.append(
        &lid,
        Mat4::from_translation(Vec3::new(0.0, 0.3, -0.24))
            * Mat4::from_rotation_x(-1.9)
            * Mat4::from_translation(Vec3::new(0.0, 0.0, 0.24)),
    );
    let gold_trim = bank.add(tiles::planks(CRIMSON, PINK, PLUM, 8));
    let mut loot_chest = Mesh::new();
    tiled_box(
        &mut loot_chest,
        Vec3::new(-0.34, 0.0, -0.24),
        Vec3::new(0.34, 0.32, 0.24),
        gold_trim,
        0,
    );
    tiled_box(
        &mut loot_chest,
        Vec3::new(-0.36, 0.32, -0.26),
        Vec3::new(0.36, 0.48, 0.26),
        gold_trim,
        0,
    );
    skin_box(
        &mut loot_chest,
        Vec3::new(-0.36, 0.3, -0.26),
        Vec3::new(0.36, 0.34, 0.27),
        metal,
        &w4,
    );
    skin_box(
        &mut loot_chest,
        Vec3::new(-0.06, 0.22, 0.26),
        Vec3::new(0.06, 0.36, 0.29),
        metal,
        &w4,
    );

    // Stairs down: steps sinking into a pit.
    let step = bank.add(tiles::cobbles(KHAKI, SAND, SHADOW, 9));
    let dark = bank.add(tiles::solid(INK));
    let mut stairs = Mesh::new();
    for k in 0..4 {
        let y = -0.2 * (k + 1) as f32;
        let z0 = 0.5 - 0.25 * (k + 1) as f32;
        tiled_box(
            &mut stairs,
            Vec3::new(-0.4, y - 0.2, z0),
            Vec3::new(0.4, y, z0 + 0.25),
            step,
            0,
        );
    }
    // Pit walls and a dark bottom.
    tiled_box(
        &mut stairs,
        Vec3::new(-0.5, -1.0, -0.5),
        Vec3::new(-0.4, 0.0, 0.5),
        step,
        0,
    );
    tiled_box(
        &mut stairs,
        Vec3::new(0.4, -1.0, -0.5),
        Vec3::new(0.5, 0.0, 0.5),
        step,
        0,
    );
    tiled_box(
        &mut stairs,
        Vec3::new(-0.5, -1.0, -0.5),
        Vec3::new(0.5, 0.0, -0.42),
        step,
        0,
    );
    stairs.quad(
        [
            Vec3::new(-0.4, -0.99, 0.5),
            Vec3::new(0.4, -0.99, 0.5),
            Vec3::new(0.4, -0.99, -0.42),
            Vec3::new(-0.4, -0.99, -0.42),
        ],
        UvRect::px(0, 0, 4, 4),
        dark,
    );
    for v in &mut stairs.verts {
        v.ao = (1.0 + v.pos.y * 0.85).max(0.15);
    }

    // Waystone: a carved pillar with a glowing rune (the rune is drawn unlit separately).
    let block = bank.add(tiles::bricks(SAND, WHITE, KHAKI, 10));
    let mut waystone = Mesh::new();
    tiled_box(
        &mut waystone,
        Vec3::new(-0.42, 0.0, -0.42),
        Vec3::new(0.42, 0.14, 0.42),
        block,
        0,
    );
    tiled_box(
        &mut waystone,
        Vec3::new(-0.24, 0.14, -0.2),
        Vec3::new(0.24, 1.3, 0.2),
        block,
        0,
    );
    tiled_box(
        &mut waystone,
        Vec3::new(-0.3, 1.3, -0.26),
        Vec3::new(0.3, 1.42, 0.26),
        block,
        0,
    );
    let mut rune_t = Texture::clear(8, 16);
    for (x, y) in [
        (3, 1),
        (4, 1),
        (2, 2),
        (5, 2),
        (3, 3),
        (4, 3),
        (3, 4),
        (4, 5),
        (3, 6),
        (2, 7),
        (5, 7),
        (3, 8),
        (4, 8),
        (1, 10),
        (6, 10),
        (2, 11),
        (5, 11),
        (3, 12),
        (4, 12),
        (3, 13),
        (4, 14),
    ] {
        rune_t.set(x, y, MINT);
    }
    let rune = bank.add(rune_t);
    let mut waystone_rune = Mesh::new();
    waystone_rune.quad(
        [
            Vec3::new(-0.16, 0.3, 0.205),
            Vec3::new(0.16, 0.3, 0.205),
            Vec3::new(0.16, 1.2, 0.205),
            Vec3::new(-0.16, 1.2, 0.205),
        ],
        UvRect::px(0, 0, 8, 16),
        rune,
    );

    let mut campfire = Mesh::new();
    for a in [0.3f32, 1.9, 3.4] {
        let mut l = Mesh::new();
        tiled_box(
            &mut l,
            Vec3::new(-0.3, 0.0, -0.06),
            Vec3::new(0.3, 0.12, 0.06),
            bark,
            0,
        );
        campfire.append(&l, Mat4::from_rotation_y(a));
    }
    for k in 0..7 {
        let a = k as f32 / 7.0 * std::f32::consts::TAU;
        let p = Vec3::new(a.cos() * 0.4, 0.0, a.sin() * 0.4);
        let mut s = Mesh::new();
        lathe(
            &mut s,
            Vec3::ZERO,
            &[(0.09, 0.0), (0.07, 0.1), (0.0, 0.12)],
            5,
            a,
            stone,
            false,
        );
        campfire.append(&s, Mat4::from_translation(p));
    }
    let mut torch_stick = Mesh::new();
    tiled_box(
        &mut torch_stick,
        Vec3::new(-0.04, 0.0, -0.04),
        Vec3::new(0.04, 0.7, 0.04),
        bark,
        0,
    );
    skin_box(
        &mut torch_stick,
        Vec3::new(-0.07, 0.62, -0.07),
        Vec3::new(0.07, 0.72, 0.07),
        metal,
        &w4,
    );

    let post = bank.add(tiles::rock_side([SAND, KHAKI, ROSEWOOD, SHADOW], 22));
    let mut lamp = Mesh::new();
    tiled_box(
        &mut lamp,
        Vec3::new(-0.16, 0.0, -0.16),
        Vec3::new(0.16, 0.12, 0.16),
        post,
        0,
    );
    tiled_box(
        &mut lamp,
        Vec3::new(-0.07, 0.12, -0.07),
        Vec3::new(0.07, 0.9, 0.07),
        post,
        0,
    );

    let fence_wood = bank.add(tiles::planks(SAND, WHITE, KHAKI, 11));
    let mut fence_post = Mesh::new();
    tiled_box(
        &mut fence_post,
        Vec3::new(-0.07, 0.0, -0.07),
        Vec3::new(0.07, 0.62, 0.07),
        fence_wood,
        0,
    );
    skin_box(
        &mut fence_post,
        Vec3::new(-0.08, 0.62, -0.08),
        Vec3::new(0.08, 0.66, 0.08),
        fence_wood,
        &w4,
    );
    let mut fence_rail_e = Mesh::new();
    for y in [0.22, 0.44] {
        tiled_box(
            &mut fence_rail_e,
            Vec3::new(0.0, y, -0.035),
            Vec3::new(1.0, y + 0.08, 0.035),
            fence_wood,
            0,
        );
    }
    let mut fence_rail_s = Mesh::new();
    for y in [0.22, 0.44] {
        tiled_box(
            &mut fence_rail_s,
            Vec3::new(-0.035, y, 0.0),
            Vec3::new(0.035, y + 0.08, 1.0),
            fence_wood,
            0,
        );
    }

    let mut sprinkler = Vec::new();
    for pal in [
        [GOLD, ORANGE, RUST],
        [WHITE, SKY, INDIGO],
        [WHITE, AQUA, TEAL],
    ] {
        let t = bank.add(tiles::crystal(pal));
        let mut m = Mesh::new();
        lathe(
            &mut m,
            Vec3::ZERO,
            &[(0.2, 0.0), (0.22, 0.08), (0.12, 0.16), (0.05, 0.22)],
            6,
            0.3,
            t,
            false,
        );
        lathe(
            &mut m,
            Vec3::ZERO,
            &[(0.04, 0.22), (0.04, 0.34), (0.0, 0.38)],
            5,
            0.0,
            t,
            false,
        );
        for a in [0.0f32, std::f32::consts::FRAC_PI_2] {
            let mut arm = Mesh::new();
            skin_box(
                &mut arm,
                Vec3::new(-0.16, 0.3, -0.02),
                Vec3::new(0.16, 0.34, 0.02),
                t,
                &w4,
            );
            m.append(&arm, Mat4::from_rotation_y(a));
        }
        sprinkler.push(m);
    }

    let mut workbench = Mesh::new();
    tiled_box(
        &mut workbench,
        Vec3::new(-0.42, 0.46, -0.3),
        Vec3::new(0.42, 0.56, 0.3),
        plank,
        0,
    );
    for (x, z) in [(-0.34, -0.22), (0.34, -0.22), (-0.34, 0.22), (0.34, 0.22)] {
        tiled_box(
            &mut workbench,
            Vec3::new(x - 0.05, 0.0, z - 0.05),
            Vec3::new(x + 0.05, 0.46, z + 0.05),
            plank_dark,
            0,
        );
    }
    let iron = bank.add(tiles::solid(SKY));
    skin_box(
        &mut workbench,
        Vec3::new(-0.3, 0.56, -0.1),
        Vec3::new(-0.1, 0.62, 0.05),
        iron,
        &w4,
    );
    skin_box(
        &mut workbench,
        Vec3::new(0.05, 0.56, -0.02),
        Vec3::new(0.3, 0.6, 0.04),
        plank_dark,
        &w4,
    );

    let mut flower_pot = Mesh::new();
    lathe(
        &mut flower_pot,
        Vec3::ZERO,
        &[(0.14, 0.0), (0.2, 0.26), (0.0, 0.26)],
        6,
        0.2,
        clay,
        true,
    );

    let mut bench = Mesh::new();
    tiled_box(
        &mut bench,
        Vec3::new(-0.45, 0.24, -0.16),
        Vec3::new(0.45, 0.3, 0.16),
        plank,
        0,
    );
    tiled_box(
        &mut bench,
        Vec3::new(-0.45, 0.3, -0.18),
        Vec3::new(0.45, 0.62, -0.13),
        plank,
        0,
    );
    for x in [-0.38f32, 0.38] {
        tiled_box(
            &mut bench,
            Vec3::new(x - 0.04, 0.0, -0.14),
            Vec3::new(x + 0.04, 0.24, 0.14),
            plank_dark,
            0,
        );
    }

    let (house, house_windows) = house(bank);

    let mut bin = Mesh::new();
    tiled_box(
        &mut bin,
        Vec3::new(-0.45, 0.0, -0.32),
        Vec3::new(0.45, 0.46, 0.32),
        plank,
        0,
    );
    tiled_box(
        &mut bin,
        Vec3::new(-0.48, 0.46, -0.35),
        Vec3::new(0.48, 0.56, 0.35),
        plank_dark,
        0,
    );
    skin_box(
        &mut bin,
        Vec3::new(-0.2, 0.2, 0.32),
        Vec3::new(0.2, 0.3, 0.34),
        metal,
        &w4,
    );

    let hill = bank.add(tiles::speckled_stone([KHAKI, ROSEWOOD, SHADOW, INK], 23));
    let hollow = hollow(bank, hill, plank_dark, dark);

    let awning = bank.add(tiles::stripes(RED, WHITE));
    let mut stall = Mesh::new();
    tiled_box(
        &mut stall,
        Vec3::new(-0.9, 0.0, 0.1),
        Vec3::new(0.9, 0.55, 0.45),
        plank,
        0,
    );
    tiled_box(
        &mut stall,
        Vec3::new(-0.95, 0.55, 0.05),
        Vec3::new(0.95, 0.62, 0.5),
        plank_dark,
        0,
    );
    for x in [-0.88f32, 0.88] {
        tiled_box(
            &mut stall,
            Vec3::new(x - 0.05, 0.0, -0.45),
            Vec3::new(x + 0.05, 1.5, -0.35),
            plank_dark,
            0,
        );
        tiled_box(
            &mut stall,
            Vec3::new(x - 0.05, 0.62, 0.3),
            Vec3::new(x + 0.05, 1.44, 0.4),
            plank_dark,
            0,
        );
    }
    stall.quad(
        [
            Vec3::new(-1.05, 1.42, 0.35),
            Vec3::new(1.05, 1.42, 0.35),
            Vec3::new(1.05, 1.6, -0.5),
            Vec3::new(-1.05, 1.6, -0.5),
        ],
        UvRect::new(0.0, 0.0, 32.0, 16.0),
        awning,
    );
    // Goods on the counter.
    for (i, c) in [RED, GOLD, LIME, SKY].into_iter().enumerate() {
        let t = bank.add(tiles::solid(c));
        let x = -0.6 + i as f32 * 0.4;
        skin_box(
            &mut stall,
            Vec3::new(x - 0.1, 0.62, 0.2),
            Vec3::new(x + 0.1, 0.76, 0.36),
            t,
            &w4,
        );
    }

    let mut sign = Mesh::new();
    tiled_box(
        &mut sign,
        Vec3::new(-0.04, 0.0, -0.04),
        Vec3::new(0.04, 0.5, 0.04),
        plank_dark,
        0,
    );
    tiled_box(
        &mut sign,
        Vec3::new(-0.3, 0.36, 0.04),
        Vec3::new(0.3, 0.66, 0.1),
        plank,
        0,
    );

    // The enchanting table: a carved pedestal under a starry cloth, with two candles.
    let mut enchant_table = Mesh::new();
    let carved = bank.add(tiles::bricks(SLATE, LAVENDER, INK, 51));
    let cloth = bank.add(tiles::stripes(PURPLE, GRAPE));
    let wax = bank.add(tiles::solid(CREAM));
    let flame_t = bank.add(tiles::solid(GOLD));
    lathe(
        &mut enchant_table,
        Vec3::ZERO,
        &[
            (0.34, 0.0),
            (0.34, 0.08),
            (0.2, 0.14),
            (0.16, 0.5),
            (0.26, 0.56),
            (0.0, 0.56),
        ],
        8,
        0.0,
        carved,
        true,
    );
    tiled_box(
        &mut enchant_table,
        Vec3::new(-0.36, 0.56, -0.3),
        Vec3::new(0.36, 0.64, 0.3),
        cloth,
        0,
    );
    // The cloth hangs down at the front and back.
    for z in [-0.31, 0.29] {
        tiled_box(
            &mut enchant_table,
            Vec3::new(-0.3, 0.4, z),
            Vec3::new(0.3, 0.6, z + 0.02),
            cloth,
            0,
        );
    }
    for x in [-0.28, 0.28] {
        skin_box(
            &mut enchant_table,
            Vec3::new(x - 0.035, 0.64, -0.2),
            Vec3::new(x + 0.035, 0.8, -0.13),
            wax,
            &w4,
        );
        skin_box(
            &mut enchant_table,
            Vec3::new(x - 0.015, 0.8, -0.18),
            Vec3::new(x + 0.015, 0.86, -0.15),
            flame_t,
            &w4,
        );
    }
    // A little open book of spells.
    let mut enchant_book = Mesh::new();
    let page = bank.add(tiles::stripes(WHITE, CREAM));
    let cover = bank.add(tiles::solid(CRIMSON));
    for side in [-1.0f32, 1.0] {
        let mut leaf = Mesh::new();
        skin_box(
            &mut leaf,
            Vec3::new(0.0, -0.01, -0.09),
            Vec3::new(0.15, 0.0, 0.09),
            cover,
            &w4,
        );
        skin_box(
            &mut leaf,
            Vec3::new(0.01, 0.0, -0.08),
            Vec3::new(0.14, 0.03, 0.08),
            page,
            &w4,
        );
        let m = if side > 0.0 {
            Mat4::from_rotation_z(0.25)
        } else {
            Mat4::from_rotation_y(std::f32::consts::PI) * Mat4::from_rotation_z(0.25)
        };
        enchant_book.append(&leaf, m);
    }

    Props {
        enchant_table,
        enchant_book,
        tufts,
        weeds,
        shrubs,
        bubble,
        trees,
        pines,
        stump,
        log,
        rocks,
        boulder,
        crystal,
        stalagmite,
        mushroom,
        bones,
        pot,
        crate_,
        chest,
        chest_open,
        loot_chest,
        stairs,
        waystone,
        waystone_rune,
        campfire,
        torch_stick,
        lamp,
        fence_post,
        fence_rail_e,
        fence_rail_s,
        sprinkler,
        workbench,
        flower_pot,
        bench,
        house,
        house_windows,
        bin,
        hollow,
        stall,
        sign,
    }
}

/// The farmhouse, 4 tiles wide and 3 deep, anchored at its front-left tile's centre.
/// Returns the house and its windows (drawn glowing at night).
fn house(bank: &mut TexBank) -> (Mesh, Mesh) {
    let wall = bank.add(tiles::cottage_wall());
    let roof_t = bank.add(tiles::roof([SALMON, CRIMSON, PLUM]));
    let door_t = bank.add(tiles::planks(RUST, CLAY, MAROON, 12));
    let stone = bank.add(tiles::cobbles(KHAKI, SAND, SHADOW, 13));
    let brick = bank.add(tiles::bricks(RUST, CLAY, MAROON, 14));
    let glass = bank.add(tiles::crystal([CREAM, GOLD, SKY]));
    let frame = bank.add(tiles::solid(RUST));
    let w4 = Texture::new(4, 4, 0);

    // Footprint x in [-0.5, 3.5], z in [-2.5, 0.5] (relative to the front-left tile centre).
    let (x0, x1, z0, z1) = (-0.45, 3.45, -2.45, 0.3);
    let mut m = Mesh::new();
    tiled_box(
        &mut m,
        Vec3::new(x0 - 0.05, 0.0, z0 - 0.05),
        Vec3::new(x1 + 0.05, 0.18, z1 + 0.05),
        stone,
        0,
    );
    tiled_box(
        &mut m,
        Vec3::new(x0, 0.18, z0),
        Vec3::new(x1, 1.5, z1),
        wall,
        1 << TOP,
    );
    // Gabled roof running east-west.
    let (ry0, ry1) = (1.45, 2.55);
    let zm = (z0 + z1) * 0.5;
    let (rx0, rx1, rz0, rz1) = (x0 - 0.25, x1 + 0.25, z0 - 0.3, z1 + 0.35);
    let slope = |a: Vec3, b: Vec3, c: Vec3, d: Vec3, m: &mut Mesh| {
        let len = (c - b).length() * TD;
        let wid = (b - a).length() * TD;
        m.quad([a, b, c, d], UvRect::new(0.0, 0.0, wid, len), roof_t);
    };
    // South slope (faces the camera).
    slope(
        Vec3::new(rx0, ry0, rz1),
        Vec3::new(rx1, ry0, rz1),
        Vec3::new(rx1, ry1, zm),
        Vec3::new(rx0, ry1, zm),
        &mut m,
    );
    // North slope.
    slope(
        Vec3::new(rx1, ry0, rz0),
        Vec3::new(rx0, ry0, rz0),
        Vec3::new(rx0, ry1, zm),
        Vec3::new(rx1, ry1, zm),
        &mut m,
    );
    // Roof underside (visible eaves) and gable ends.
    for (x, flip) in [(x0, true), (x1, false)] {
        let a = Vec3::new(x, 1.5, z1);
        let b = Vec3::new(x, 1.5, z0);
        let c = Vec3::new(x, ry1 - 0.05, zm);
        let uv = [
            glam::Vec2::new(0.0, 16.0),
            glam::Vec2::new(40.0, 16.0),
            glam::Vec2::new(20.0, 0.0),
        ];
        if flip {
            m.tri([b, a, c], [uv[0], uv[1], uv[2]], wall);
        } else {
            m.tri([a, b, c], uv, wall);
        }
    }
    // Chimney.
    tiled_box(
        &mut m,
        Vec3::new(2.4, 1.9, -1.6),
        Vec3::new(2.85, 2.9, -1.15),
        brick,
        0,
    );
    // Door (front centre-left) with a step and a little lamp.
    let door_x = 1.0;
    tiled_box(
        &mut m,
        Vec3::new(door_x - 0.34, 0.18, z1),
        Vec3::new(door_x + 0.34, 1.12, z1 + 0.04),
        door_t,
        0,
    );
    skin_box(
        &mut m,
        Vec3::new(door_x + 0.18, 0.6, z1 + 0.04),
        Vec3::new(door_x + 0.24, 0.68, z1 + 0.08),
        frame,
        &w4,
    );
    tiled_box(
        &mut m,
        Vec3::new(door_x - 0.5, 0.0, z1),
        Vec3::new(door_x + 0.5, 0.12, z1 + 0.35),
        stone,
        0,
    );
    // Window frames.
    let mut windows = Mesh::new();
    for wx in [-0.05f32, 2.3] {
        tiled_box(
            &mut m,
            Vec3::new(wx - 0.05, 0.55, z1),
            Vec3::new(wx + 0.75, 1.15, z1 + 0.03),
            frame,
            0,
        );
        windows.quad(
            [
                Vec3::new(wx, 0.6, z1 + 0.04),
                Vec3::new(wx + 0.7, 0.6, z1 + 0.04),
                Vec3::new(wx + 0.7, 1.1, z1 + 0.04),
                Vec3::new(wx, 1.1, z1 + 0.04),
            ],
            UvRect::px(0, 0, 11, 8),
            glass,
        );
        // Flower boxes.
        let fb = bank.add(tiles::planks(CLAY, GOLD, RUST, 15));
        tiled_box(
            &mut m,
            Vec3::new(wx - 0.05, 0.45, z1),
            Vec3::new(wx + 0.75, 0.56, z1 + 0.16),
            fb,
            0,
        );
        for k in 0..4 {
            let c = [PINK, CREAM, SKY, PINK][k];
            let t = bank.add(tiles::solid(c));
            let x = wx + 0.08 + k as f32 * 0.18;
            skin_box(
                &mut m,
                Vec3::new(x - 0.05, 0.56, z1 + 0.05),
                Vec3::new(x + 0.05, 0.64, z1 + 0.13),
                t,
                &w4,
            );
        }
    }
    (m, windows)
}

/// The Hollow: a mossy rock mound with a timber-framed opening down into the dungeon.
/// Anchored at the entrance tile; covers 3x2 tiles.
fn hollow(bank: &mut TexBank, stone: TexId, beam: TexId, dark: TexId) -> Mesh {
    let moss = bank.add(tiles::leaves([LIME, GREEN, TEAL, DEEP_TEAL], 30));
    let mut m = Mesh::new();
    lathe(
        &mut m,
        Vec3::new(0.0, 0.0, -0.75),
        &[(1.5, 0.0), (1.42, 0.5), (1.1, 1.0), (0.7, 1.3), (0.0, 1.4)],
        7,
        0.3,
        stone,
        false,
    );
    lathe(
        &mut m,
        Vec3::new(0.05, 0.0, -0.8),
        &[(1.18, 0.98), (0.74, 1.34), (0.0, 1.48)],
        7,
        0.6,
        moss,
        false,
    );
    // Opening: a timber portal with a dark passage behind it.
    tiled_box(
        &mut m,
        Vec3::new(-0.5, 0.0, 0.3),
        Vec3::new(0.5, 1.0, 0.62),
        stone,
        1 << FRONT,
    );
    m.quad(
        [
            Vec3::new(-0.4, 0.0, 0.64),
            Vec3::new(0.4, 0.0, 0.64),
            Vec3::new(0.4, 0.88, 0.64),
            Vec3::new(-0.4, 0.88, 0.64),
        ],
        UvRect::px(0, 0, 4, 4),
        dark,
    );
    for x in [-0.5f32, 0.5] {
        tiled_box(
            &mut m,
            Vec3::new(x - 0.09, 0.0, 0.6),
            Vec3::new(x + 0.09, 1.0, 0.78),
            beam,
            0,
        );
    }
    tiled_box(
        &mut m,
        Vec3::new(-0.68, 0.96, 0.56),
        Vec3::new(0.68, 1.14, 0.8),
        beam,
        0,
    );
    m
}
