//! The Hollow's beasts: drakelings (frost drakes of the Frost Caverns and cinder drakes of
//! the Ember Depths), and the leaflings of the Mossy Burrows. Plus the icons for what they,
//! and the werewolves of the full moon, leave behind.

use std::collections::HashMap;

use glam::{Mat4, Quat, Vec2, Vec3};

use super::models::{lathe, skin_box};
use super::tiles;
use crate::palette::*;
use crate::render::{Mesh, TexBank, TexId, Texture, UvRect};

/// A drakeling's colours: its scales (light, mid, dark), its belly plates (light, dark), its
/// wings' skin (light, mid) and bones, its horns, and its eyes.
pub struct DrakeLook {
    pub scales: [u8; 3],
    pub belly: [u8; 2],
    pub wing: [u8; 2],
    pub bone: u8,
    pub horn: u8,
    pub eye: u8,
    /// What it breathes: its colours, hottest (or coldest) first.
    pub breath: [u8; 3],
}

/// The frost drake (blue, like a winter sky, breathing frost) and the cinder drake (red and
/// gold, breathing fire).
pub const DRAKES: [DrakeLook; 2] = [
    DrakeLook {
        scales: [SKY, BLUE, INDIGO],
        belly: [CREAM, SAND],
        wing: [INDIGO, SLATE],
        bone: INK,
        horn: CREAM,
        eye: GOLD,
        breath: [WHITE, SKY, BLUE],
    },
    DrakeLook {
        scales: [SALMON, RED, MAROON],
        belly: [GOLD, CLAY],
        wing: [GOLD, ORANGE],
        bone: MAROON,
        horn: SAND,
        eye: CREAM,
        breath: [CREAM, GOLD, ORANGE],
    },
];

/// Which drakeling lives in a biome: the cinder drake in the Ember Depths, the frost drake
/// everywhere else it turns up.
pub fn drake_look(biome: usize) -> usize {
    usize::from(biome % 6 == 3)
}

/// Where a drakeling's parts go, in its own space (facing +z, on the ground).
pub const WING_AT: Vec3 = Vec3::new(0.12, 0.36, -0.08);
pub const TAIL_AT: Vec3 = Vec3::new(0.0, 0.14, -0.12);
pub const SNOUT_AT: Vec3 = Vec3::new(0.0, 0.47, 0.38);

pub struct Drake {
    /// Body, head, horns, arms and legs.
    pub body: Mesh,
    /// The right wing, out along +x from the shoulder (mirrored for the left).
    pub wing: Mesh,
    /// The tail, back along -z from the rump.
    pub tail: Mesh,
}

pub struct Leafling {
    /// Body, head, face, and the two big leaves on its head.
    pub body: Mesh,
    /// A leaf wing, out along +x from its back (mirrored for the left, one above another).
    pub wing: Mesh,
    /// One of the leaves it throws: its stem at the origin, its point along +y.
    pub leaf: Mesh,
}

/// Where a leafling's wings join its back.
pub const LEAF_WING_AT: Vec3 = Vec3::new(0.05, 0.2, -0.07);

pub struct Beasts {
    /// The frost drake and the cinder drake (see `DRAKES`).
    pub drakes: [Drake; 2],
    pub leafling: Leafling,
}

fn v(x: f32, y: f32, z: f32) -> Vec3 {
    Vec3::new(x, y, z)
}

fn solid(bank: &mut TexBank, cache: &mut HashMap<u8, TexId>, c: u8) -> TexId {
    *cache.entry(c).or_insert_with(|| bank.add(tiles::solid(c)))
}

/// A box in one colour.
fn bx(bank: &mut TexBank, cache: &mut HashMap<u8, TexId>, m: &mut Mesh, a: Vec3, b: Vec3, c: u8) {
    let t = solid(bank, cache, c);
    skin_box(m, a, b, t, &Texture::new(4, 4, c));
}

/// A little cone from `at` pointing along `dir`.
fn spike(m: &mut Mesh, at: Vec3, dir: Vec3, len: f32, r: f32, tex: TexId) {
    let mut p = Mesh::new();
    lathe(
        &mut p,
        Vec3::ZERO,
        &[(r, 0.0), (0.0, len)],
        4,
        0.4,
        tex,
        false,
    );
    let rot = Quat::from_rotation_arc(Vec3::Y, dir.normalize());
    m.append(&p, Mat4::from_translation(at) * Mat4::from_quat(rot));
}

/// Scales: rows of little overlapping scallops, a bright edge along the top of each.
fn scales(pal: [u8; 3]) -> Texture {
    let [light, mid, dark] = pal;
    let mut t = Texture::new(16, 16, mid);
    for y in 0..16 {
        let off = if (y / 4) % 2 == 0 { 0 } else { 2 };
        for x in 0..16 {
            let (sx, sy) = ((x + off) % 4, y % 4);
            let c = match (sx, sy) {
                (_, 3) => dark,
                (0, 2) | (3, 2) => dark,
                (1, 0) | (2, 0) => light,
                _ => mid,
            };
            t.set(x, y, c);
        }
    }
    t
}

/// A drakeling's belly: plates in bands, the joins darker.
fn belly([light, dark]: [u8; 2]) -> Texture {
    let mut t = Texture::new(16, 16, light);
    for y in 0..16 {
        if y % 4 == 3 {
            for x in 0..16 {
                t.set(x, y, dark);
            }
        }
    }
    t
}

/// A wing's skin: its colour, paler in streaks towards its trailing edge, with the finger
/// bones showing through.
fn membrane([light, mid]: [u8; 2], bone: u8) -> Texture {
    let mut t = Texture::new(16, 16, mid);
    for y in 0..16 {
        for x in 0..16 {
            if y > 10 && (x + y) % 4 == 0 {
                t.set(x, y, light);
            } else if x % 5 == 0 && y > 2 {
                t.set(x, y, bone);
            }
        }
    }
    t
}

fn drake(bank: &mut TexBank, look: &DrakeLook) -> Drake {
    let mut cache = HashMap::new();
    let skin = bank.add(scales(look.scales));
    let plates = bank.add(belly(look.belly));
    let horn = solid(bank, &mut cache, look.horn);
    let dark = look.scales[2];
    let ridge = solid(bank, &mut cache, dark);
    let w4 = Texture::new(4, 4, 0);
    let mut body = Mesh::new();
    // Haunches, sat on, with claws poking out in front.
    for sx in [-1.0f32, 1.0] {
        skin_box(
            &mut body,
            v(sx * 0.115 - 0.055, 0.0, -0.11),
            v(sx * 0.115 + 0.055, 0.15, 0.06),
            skin,
            &w4,
        );
        for k in [-1.0f32, 0.0, 1.0] {
            bx(
                bank,
                &mut cache,
                &mut body,
                v(sx * 0.115 + k * 0.032 - 0.009, 0.0, 0.06),
                v(sx * 0.115 + k * 0.032 + 0.009, 0.026, 0.095),
                look.horn,
            );
        }
    }
    // A chubby body sitting up, plates down its front, and a neck leaning forward.
    skin_box(
        &mut body,
        v(-0.14, 0.1, -0.12),
        v(0.14, 0.4, 0.12),
        skin,
        &w4,
    );
    skin_box(
        &mut body,
        v(-0.1, 0.12, 0.12),
        v(0.1, 0.38, 0.15),
        plates,
        &w4,
    );
    skin_box(
        &mut body,
        v(-0.075, 0.34, -0.02),
        v(0.075, 0.47, 0.13),
        skin,
        &w4,
    );
    skin_box(
        &mut body,
        v(-0.06, 0.36, 0.13),
        v(0.06, 0.45, 0.145),
        plates,
        &w4,
    );
    // The head, and a long snout with a pale jaw under it.
    skin_box(
        &mut body,
        v(-0.11, 0.42, 0.0),
        v(0.11, 0.6, 0.22),
        skin,
        &w4,
    );
    skin_box(
        &mut body,
        v(-0.075, 0.43, 0.22),
        v(0.075, 0.53, 0.37),
        skin,
        &w4,
    );
    bx(
        bank,
        &mut cache,
        &mut body,
        v(-0.07, 0.415, 0.19),
        v(0.07, 0.445, 0.36),
        look.belly[0],
    );
    for sx in [-1.0f32, 1.0] {
        // Nostrils on top of the snout, fangs over the jaw.
        bx(
            bank,
            &mut cache,
            &mut body,
            v(sx * 0.035 - 0.014, 0.53, 0.33),
            v(sx * 0.035 + 0.014, 0.536, 0.355),
            INK,
        );
        bx(
            bank,
            &mut cache,
            &mut body,
            v(sx * 0.055 - 0.01, 0.405, 0.33),
            v(sx * 0.055 + 0.01, 0.44, 0.345),
            WHITE,
        );
        // Big eyes over the snout, slit pupils, and a fierce brow over each.
        bx(
            bank,
            &mut cache,
            &mut body,
            v(sx * 0.068 - 0.036, 0.51, 0.215),
            v(sx * 0.068 + 0.036, 0.575, 0.228),
            look.eye,
        );
        bx(
            bank,
            &mut cache,
            &mut body,
            v(sx * 0.06 - 0.009, 0.515, 0.226),
            v(sx * 0.06 + 0.009, 0.57, 0.232),
            INK,
        );
        let mut brow = Mesh::new();
        bx(
            bank,
            &mut cache,
            &mut brow,
            v(-0.045, -0.012, -0.03),
            v(0.045, 0.012, 0.012),
            dark,
        );
        body.append(
            &brow,
            Mat4::from_translation(v(sx * 0.066, 0.588, 0.215)) * Mat4::from_rotation_z(sx * 0.35),
        );
        // Horns sweeping back off the top of its head, and frills at its cheeks.
        spike(
            &mut body,
            v(sx * 0.075, 0.585, 0.06),
            v(sx * 0.3, 0.55, -0.78),
            0.22,
            0.038,
            horn,
        );
        spike(
            &mut body,
            v(sx * 0.11, 0.5, 0.08),
            v(sx * 1.0, 0.15, -0.45),
            0.1,
            0.03,
            ridge,
        );
        // Stubby arms reaching forward.
        skin_box(
            &mut body,
            v(sx * 0.15 - 0.035, 0.22, 0.02),
            v(sx * 0.15 + 0.035, 0.29, 0.18),
            skin,
            &w4,
        );
        for k in [-1.0f32, 1.0] {
            bx(
                bank,
                &mut cache,
                &mut body,
                v(sx * 0.15 + k * 0.018 - 0.008, 0.22, 0.18),
                v(sx * 0.15 + k * 0.018 + 0.008, 0.24, 0.205),
                look.horn,
            );
        }
    }
    // Spikes down its neck and back.
    for (y, z) in [
        (0.6, 0.02),
        (0.5, -0.03),
        (0.4, -0.12),
        (0.28, -0.13),
        (0.17, -0.13),
    ] {
        spike(
            &mut body,
            v(0.0, y, z),
            v(0.0, 0.55, -1.0),
            0.075,
            0.03,
            ridge,
        );
    }
    // The wing: bones out from the shoulder and the skin stretched between them, facing
    // forwards, with a scalloped trailing edge.
    let skin_w = bank.add(membrane(look.wing, look.bone));
    let bone = solid(bank, &mut cache, look.bone);
    let mut wing = Mesh::new();
    let shoulder = Vec3::ZERO;
    let wrist = v(0.2, 0.18, 0.0);
    let tips = [
        v(0.46, 0.2, -0.02),
        v(0.48, 0.02, -0.03),
        v(0.34, -0.1, -0.03),
    ];
    let root = v(0.06, -0.1, -0.02);
    let uv = |p: Vec3| Vec2::new(p.x / 0.48 * 15.0, (0.2 - p.y) / 0.3 * 15.0);
    let mut edge = vec![wrist, tips[0]];
    for (a, b) in [(tips[0], tips[1]), (tips[1], tips[2]), (tips[2], root)] {
        // Between two fingers the skin sags in towards the shoulder.
        edge.push(a.lerp(b, 0.5) * 0.8);
        edge.push(b);
    }
    for w in edge.windows(2) {
        let (a, b) = (w[0], w[1]);
        wing.tri([shoulder, a, b], [uv(shoulder), uv(a), uv(b)], skin_w);
    }
    for (from, to) in [
        (shoulder, wrist),
        (wrist, tips[0]),
        (wrist, tips[1]),
        (wrist, tips[2]),
    ] {
        let mut b = Mesh::new();
        let len = (to - from).length();
        skin_box(
            &mut b,
            v(-0.013, 0.0, -0.013),
            v(0.013, len, 0.013),
            bone,
            &w4,
        );
        let rot = Quat::from_rotation_arc(Vec3::Y, (to - from).normalize());
        wing.append(&b, Mat4::from_translation(from) * Mat4::from_quat(rot));
    }
    // A claw at the wrist.
    spike(&mut wing, wrist, v(0.1, 1.0, 0.2), 0.06, 0.02, horn);
    // The tail: getting thinner as it goes, curling round to one side, with spikes along
    // it and a spade at the end.
    let mut tail = Mesh::new();
    let mut at = Vec3::ZERO;
    let mut dir = v(0.0, -0.25, -1.0).normalize();
    for (len, w) in [
        (0.14f32, 0.075f32),
        (0.13, 0.058),
        (0.12, 0.045),
        (0.1, 0.035),
    ] {
        let mut seg = Mesh::new();
        skin_box(
            &mut seg,
            v(-w, 0.0, -w * 0.8),
            v(w, len, w * 0.8),
            skin,
            &w4,
        );
        let rot = Quat::from_rotation_arc(Vec3::Y, dir);
        tail.append(&seg, Mat4::from_translation(at) * Mat4::from_quat(rot));
        spike(
            &mut tail,
            at + dir * len * 0.5 + Vec3::Y * w,
            v(0.0, 1.0, -0.5),
            0.05,
            0.02,
            ridge,
        );
        at += dir * len;
        // Level off, and curl round.
        dir = (Quat::from_rotation_y(0.45) * v(dir.x, dir.y * 0.3, dir.z)).normalize();
    }
    let spade = solid(bank, &mut cache, dark);
    let (s, h) = (0.075f32, 0.012f32);
    let side = v(dir.z, 0.0, -dir.x) * s;
    let tip = at + dir * 0.13 + Vec3::Y * 0.02;
    for (a, b) in [(at - side, tip), (tip, at + side)] {
        tail.tri(
            [a + Vec3::Y * h, b + Vec3::Y * h, at + Vec3::Y * h],
            [Vec2::ZERO, Vec2::new(3.0, 0.0), Vec2::new(0.0, 3.0)],
            spade,
        );
        tail.tri(
            [b - Vec3::Y * h, a - Vec3::Y * h, at - Vec3::Y * h],
            [Vec2::ZERO, Vec2::new(3.0, 0.0), Vec2::new(0.0, 3.0)],
            spade,
        );
    }
    Drake { body, wing, tail }
}

/// A leaf with a pale midrib and darker veins and edge; clear round its shape (so it can be
/// drawn on a flat quad).
fn leaf([light, mid, dark]: [u8; 3]) -> Texture {
    let mut t = Texture::clear(16, 16);
    for y in 0..16 {
        // Widest a little below the middle, coming to a point at the top (y = 0).
        let k = y as f32 / 15.0;
        let half = (k * std::f32::consts::PI).sin().powf(0.8) * 7.0 * (0.35 + 0.65 * k);
        for x in 0..16 {
            let d = (x as f32 + 0.5 - 8.0).abs();
            if d <= half {
                let c = if d > half - 1.0 {
                    dark
                } else if d < 0.8 {
                    light
                } else if (y + x / 2) % 5 == 0 {
                    dark
                } else {
                    mid
                };
                t.set(x, y, c);
            }
        }
    }
    t
}

/// A leaf quad `len` long and `w` wide, its stem at the origin and its point along +y.
fn leaf_quad(m: &mut Mesh, len: f32, w: f32, tex: TexId) {
    m.quad(
        [
            v(-w * 0.5, 0.0, 0.0),
            v(w * 0.5, 0.0, 0.0),
            v(w * 0.5, len, 0.0),
            v(-w * 0.5, len, 0.0),
        ],
        UvRect::new(0.0, 0.0, 16.0, 16.0),
        tex,
    );
}

fn leafling(bank: &mut TexBank) -> Leafling {
    let mut cache = HashMap::new();
    let (light, mid, dark) = (LIME, GREEN, DEEP_TEAL);
    let skin = solid(bank, &mut cache, mid);
    let pale = solid(bank, &mut cache, light);
    let leaf_t = bank.add(leaf([light, mid, dark]));
    let mut body = Mesh::new();
    // A little body with a skirt of leaves, and a big round head.
    lathe(
        &mut body,
        Vec3::ZERO,
        &[(0.02, 0.02), (0.07, 0.06), (0.075, 0.13), (0.05, 0.19)],
        6,
        0.0,
        skin,
        true,
    );
    lathe(
        &mut body,
        Vec3::ZERO,
        &[
            (0.03, 0.16),
            (0.1, 0.19),
            (0.13, 0.26),
            (0.12, 0.33),
            (0.07, 0.38),
            (0.0, 0.39),
        ],
        8,
        0.0,
        pale,
        false,
    );
    // A cross little face: red eyes under knitted brows, and a small frown.
    for sx in [-1.0f32, 1.0] {
        bx(
            bank,
            &mut cache,
            &mut body,
            v(sx * 0.045 - 0.018, 0.265, 0.115),
            v(sx * 0.045 + 0.018, 0.3, 0.13),
            RED,
        );
        bx(
            bank,
            &mut cache,
            &mut body,
            v(sx * 0.045 - 0.006, 0.272, 0.128),
            v(sx * 0.045 + 0.006, 0.29, 0.134),
            INK,
        );
        let mut brow = Mesh::new();
        bx(
            bank,
            &mut cache,
            &mut brow,
            v(-0.028, -0.007, -0.006),
            v(0.028, 0.007, 0.006),
            dark,
        );
        body.append(
            &brow,
            Mat4::from_translation(v(sx * 0.045, 0.312, 0.122)) * Mat4::from_rotation_z(sx * 0.45),
        );
    }
    bx(
        bank,
        &mut cache,
        &mut body,
        v(-0.022, 0.235, 0.117),
        v(0.022, 0.245, 0.128),
        dark,
    );
    // Two big leaves standing up off its head like ears, and a sprig on top.
    for sx in [-1.0f32, 1.0] {
        let mut l = Mesh::new();
        leaf_quad(&mut l, 0.24, 0.13, leaf_t);
        body.append(
            &l,
            Mat4::from_translation(v(sx * 0.08, 0.33, -0.01))
                * Mat4::from_rotation_z(-sx * 0.75)
                * Mat4::from_rotation_y(sx * 0.25),
        );
    }
    let mut sprig = Mesh::new();
    leaf_quad(&mut sprig, 0.13, 0.07, leaf_t);
    body.append(
        &sprig,
        Mat4::from_translation(v(0.0, 0.38, 0.0)) * Mat4::from_rotation_x(-0.3),
    );
    // Leaves round its middle for a skirt, and little arms.
    for k in 0..5 {
        let a = k as f32 / 5.0 * std::f32::consts::TAU;
        let mut l = Mesh::new();
        leaf_quad(&mut l, 0.1, 0.07, leaf_t);
        body.append(
            &l,
            Mat4::from_translation(v(a.sin() * 0.06, 0.11, a.cos() * 0.06))
                * Mat4::from_rotation_y(a)
                * Mat4::from_rotation_x(2.6),
        );
    }
    for sx in [-1.0f32, 1.0] {
        bx(
            bank,
            &mut cache,
            &mut body,
            v(sx * 0.075 - 0.015, 0.1, 0.0),
            v(sx * 0.075 + 0.015, 0.16, 0.03),
            mid,
        );
    }
    // A wing: a pair of pale, glassy leaves fanning out from its back.
    let wing_t = bank.add(leaf([WHITE, MINT, AQUA]));
    let mut wing = Mesh::new();
    for (tilt, len) in [(1.0f32, 0.32f32), (1.95, 0.24)] {
        let mut l = Mesh::new();
        leaf_quad(&mut l, len, 0.16, wing_t);
        wing.append(&l, Mat4::from_rotation_z(-tilt));
    }
    let mut leaf = Mesh::new();
    leaf_quad(&mut leaf, 0.2, 0.12, leaf_t);
    Leafling { body, wing, leaf }
}

/// A shimmering drake scale (slots: its light, mid and dark).
const SCALE: &[&str] = &[
    "................",
    "......KKKK......",
    "....KK0000KK....",
    "...K00w00000K...",
    "..K00w0000011K..",
    "..K0000001111K..",
    "..K0000111111K..",
    "..K1111111112K..",
    "...K11111122K...",
    "...K11111222K...",
    "....K111222K....",
    ".....K1222K.....",
    "......K22K......",
    ".......KK.......",
    "................",
    "................",
];

/// A wolf's fang, white and curved, dark at the root.
const FANG: &[&str] = &[
    "................",
    "....KKKKKKK.....",
    "...KRRRRRRRK....",
    "...KRkkkkkRK....",
    "....KwwwwwK.....",
    "....KwwwwyK.....",
    ".....KwwwyK.....",
    ".....KwwwyK.....",
    "......KwwyK.....",
    "......KwwyK.....",
    ".......KwyK.....",
    ".......KwyK.....",
    "........KyK.....",
    "........KK......",
    "................",
    "................",
];

/// Icons for what the beasts leave behind.
pub fn icons(bank: &mut TexBank, m: &mut HashMap<&'static str, TexId>) {
    use super::sprites::art;
    for (key, look) in [("frost_scale", &DRAKES[0]), ("cinder_scale", &DRAKES[1])] {
        let [light, mid, dark] = look.scales;
        m.insert(key, bank.add(art(SCALE, [light, mid, dark, CLEAR])));
    }
    m.insert("wolf_fang", bank.add(art(FANG, [CLEAR; 4])));
}

pub fn build(bank: &mut TexBank) -> Beasts {
    Beasts {
        drakes: [drake(bank, &DRAKES[0]), drake(bank, &DRAKES[1])],
        leafling: leafling(bank),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_leaf_is_leaf_shaped() {
        let t = leaf([LIME, GREEN, DEEP_TEAL]);
        assert_eq!(t.get(0, 8), CLEAR, "clear round its edge");
        assert_ne!(t.get(8, 9), CLEAR, "solid down the middle");
        assert_eq!(t.get(8, 12), LIME, "a pale midrib");
    }

    #[test]
    fn drakes_live_where_they_breathe() {
        assert_eq!(drake_look(3), 1, "the cinder drake in the Ember Depths");
        assert_eq!(drake_look(4), 0, "the frost drake in the Frost Caverns");
        assert_eq!(DRAKES[1].breath[2], ORANGE);
        assert_eq!(DRAKES[0].breath[1], SKY);
    }
}
