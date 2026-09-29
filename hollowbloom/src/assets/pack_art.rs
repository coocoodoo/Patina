//! Backpacks: six styles (knapsack, rucksack, wicker pack, duffel, frame pack and shell pack),
//! each in eight colourways, as models worn on the hero's back and as icons.

use std::collections::HashMap;

use glam::{Mat4, Vec3};

use super::models::{lathe, skin_box};
use super::sprites::art;
use super::tiles;
use crate::palette::*;
use crate::render::{Mesh, TexBank, TexId, Texture};

/// The cloth backpacks, each in `HUES` colourways, and after them the monster backpacks (see
/// `plush_art`), one look each.
pub const CLOTH: usize = 6;
pub const STYLES: usize = CLOTH + super::plush_art::PLUSHES;
pub const HUES: usize = 8;

/// A colourway: the cloth (light, mid, dark) and the leather of its straps (light, dark).
pub struct Hue {
    pub cloth: [u8; 3],
    pub leather: [u8; 2],
}

pub const HUE: [Hue; HUES] = [
    Hue {
        cloth: [SALMON, RED, MAROON],
        leather: [CLAY, RUST],
    },
    Hue {
        cloth: [SKY, BLUE, INDIGO],
        leather: [SAND, KHAKI],
    },
    Hue {
        cloth: [LIME, GREEN, DEEP_TEAL],
        leather: [CLAY, RUST],
    },
    Hue {
        cloth: [SAND, KHAKI, ROSEWOOD],
        leather: [RUST, MAROON],
    },
    Hue {
        cloth: [LAVENDER, PURPLE, GRAPE],
        leather: [GOLD, CLAY],
    },
    Hue {
        cloth: [MINT, TEAL, DEEP_TEAL],
        leather: [SAND, KHAKI],
    },
    Hue {
        cloth: [CREAM, GOLD, CLAY],
        leather: [RUST, MAROON],
    },
    Hue {
        cloth: [BLUSH, PINK, CRIMSON],
        leather: [CREAM, SAND],
    },
];

/// Where a backpack sits: models are made in the space of the hero's body (its base at the
/// hips, facing +z), whose back is this far behind.
pub(super) const BACK: f32 = -0.12;

/// Backpacks are made a little small, then grown this much out from the middle of the back
/// and hung a little low, so they show under the hero's big head (and hat) from the camera
/// up above.
const GROW: f32 = 1.45;
const HANG: Vec3 = Vec3::new(0.0, -0.05, -0.02);

fn grow() -> Mat4 {
    let at = Vec3::new(0.0, 0.1, BACK);
    Mat4::from_translation(at + HANG)
        * Mat4::from_scale(Vec3::splat(GROW))
        * Mat4::from_translation(-at)
}

pub struct PackArt {
    /// Worn on the back, by style and colourway (monster backpacks have just the one).
    pub worn: Vec<Vec<Mesh>>,
    /// Icons, likewise.
    pub icons: Vec<Vec<TexId>>,
}

impl PackArt {
    pub fn mesh(&self, style: usize, hue: usize) -> &Mesh {
        let looks = &self.worn[style % STYLES];
        &looks[hue % looks.len()]
    }

    pub fn icon(&self, style: usize, hue: usize) -> TexId {
        let looks = &self.icons[style % STYLES];
        looks[hue % looks.len()]
    }
}

fn v(x: f32, y: f32, z: f32) -> Vec3 {
    Vec3::new(x, y, z)
}

/// Plain colours, shared.
pub(super) struct Kit {
    solid: HashMap<u8, TexId>,
    pub(super) w4: Texture,
}

impl Kit {
    pub(super) fn c(&mut self, bank: &mut TexBank, c: u8) -> TexId {
        *self
            .solid
            .entry(c)
            .or_insert_with(|| bank.add(tiles::solid(c)))
    }

    pub(super) fn bx(&mut self, bank: &mut TexBank, m: &mut Mesh, a: Vec3, b: Vec3, c: u8) {
        let t = self.c(bank, c);
        skin_box(m, a, b, t, &self.w4);
    }
}

/// Canvas: the cloth's colour with a faint weave of lighter and darker threads.
fn canvas([light, mid, dark]: [u8; 3]) -> Texture {
    let mut t = Texture::new(8, 8, mid);
    for y in 0..8 {
        for x in 0..8 {
            if (x + y * 3) % 7 == 0 {
                t.set(x, y, light);
            } else if (x * 2 + y) % 9 == 4 {
                t.set(x, y, dark);
            }
        }
    }
    t
}

/// Woven willow, in two tones, with a darker row every so often.
fn wicker() -> Texture {
    let mut t = Texture::new(8, 8, SAND);
    for y in 0..8 {
        for x in 0..8 {
            let over = ((x / 2) + (y / 2)) % 2 == 0;
            if y % 4 == 3 {
                t.set(x, y, CLAY);
            } else if over {
                t.set(x, y, KHAKI);
            }
        }
    }
    t
}

/// A snail's shell: bands running round and round in a spiral (as the lathe wraps it).
fn shell([light, mid, dark]: [u8; 3]) -> Texture {
    let mut t = Texture::new(16, 16, mid);
    for y in 0..16 {
        for x in 0..16 {
            let band = (x + y * 2) % 8;
            let c = match band {
                0 | 1 => dark,
                5 => light,
                _ => mid,
            };
            t.set(x, y, c);
        }
    }
    t
}

/// Straps over both shoulders, down the front of the chest, with a band across.
fn straps(k: &mut Kit, bank: &mut TexBank, m: &mut Mesh, h: &Hue) {
    let [light, dark] = h.leather;
    for sx in [-1.0f32, 1.0] {
        let (x0, x1) = (sx * 0.06, sx * 0.095);
        let (x0, x1) = (x0.min(x1), x0.max(x1));
        k.bx(bank, m, v(x0, 0.03, 0.12), v(x1, 0.285, 0.133), dark);
        k.bx(bank, m, v(x0, 0.27, BACK), v(x1, 0.29, 0.133), dark);
        k.bx(bank, m, v(x0, 0.08, 0.131), v(x1, 0.1, 0.137), light);
    }
    k.bx(bank, m, v(-0.06, 0.17, 0.125), v(0.06, 0.185, 0.134), light);
}

/// A drawstring sack: round and saggy, pinched and tied at the top with a frill of cloth
/// above the knot, and a patch sewn on.
fn knapsack(k: &mut Kit, bank: &mut TexBank, h: &Hue) -> Mesh {
    let mut m = Mesh::new();
    let cloth = bank.add(canvas(h.cloth));
    let at = v(0.0, 0.0, -0.22);
    lathe(
        &mut m,
        at,
        &[
            (0.07, 0.0),
            (0.125, 0.03),
            (0.145, 0.1),
            (0.135, 0.17),
            (0.08, 0.225),
            (0.045, 0.245),
        ],
        8,
        0.2,
        cloth,
        true,
    );
    let tie = k.c(bank, h.leather[0]);
    lathe(
        &mut m,
        at,
        &[(0.05, 0.235), (0.05, 0.262)],
        8,
        0.2,
        tie,
        false,
    );
    let frill = k.c(bank, h.cloth[0]);
    lathe(
        &mut m,
        at,
        &[(0.04, 0.26), (0.075, 0.29), (0.06, 0.3), (0.0, 0.28)],
        8,
        0.2,
        frill,
        false,
    );
    k.bx(
        bank,
        &mut m,
        v(-0.05, 0.07, -0.37),
        v(0.05, 0.15, -0.355),
        h.cloth[0],
    );
    for x in [-0.035f32, 0.0, 0.035] {
        k.bx(
            bank,
            &mut m,
            v(x - 0.006, 0.145, -0.373),
            v(x + 0.006, 0.152, -0.36),
            h.cloth[2],
        );
    }
    m
}

/// A canvas rucksack: a buckled flap over the top, a pocket on the back and one on each
/// side, and a loop to hang it up by.
fn rucksack(k: &mut Kit, bank: &mut TexBank, h: &Hue) -> Mesh {
    let mut m = Mesh::new();
    let cloth = bank.add(canvas(h.cloth));
    let [light, _, dark] = h.cloth;
    skin_box(
        &mut m,
        v(-0.13, 0.01, -0.27),
        v(0.13, 0.24, BACK),
        cloth,
        &k.w4,
    );
    // The flap, and its lip hanging down the back.
    k.bx(
        bank,
        &mut m,
        v(-0.135, 0.19, -0.285),
        v(0.135, 0.265, -0.115),
        dark,
    );
    k.bx(
        bank,
        &mut m,
        v(-0.135, 0.12, -0.288),
        v(0.135, 0.2, -0.27),
        dark,
    );
    for sx in [-1.0f32, 1.0] {
        let (x0, x1) = if sx < 0.0 {
            (-0.08, -0.05)
        } else {
            (0.05, 0.08)
        };
        k.bx(
            bank,
            &mut m,
            v(x0, 0.08, -0.296),
            v(x1, 0.268, -0.286),
            h.leather[1],
        );
        k.bx(
            bank,
            &mut m,
            v(x0 - 0.004, 0.085, -0.3),
            v(x1 + 0.004, 0.11, -0.294),
            GOLD,
        );
        // Pockets on the sides.
        let (px0, px1) = if sx < 0.0 {
            (-0.16, -0.13)
        } else {
            (0.13, 0.16)
        };
        k.bx(
            bank,
            &mut m,
            v(px0, 0.03, -0.24),
            v(px1, 0.14, -0.15),
            light,
        );
        k.bx(
            bank,
            &mut m,
            v(px0 - 0.003, 0.12, -0.243),
            v(px1 + 0.003, 0.145, -0.147),
            dark,
        );
    }
    // The back pocket.
    k.bx(
        bank,
        &mut m,
        v(-0.085, 0.025, -0.3),
        v(0.085, 0.11, -0.268),
        light,
    );
    k.bx(
        bank,
        &mut m,
        v(-0.088, 0.1, -0.304),
        v(0.088, 0.118, -0.268),
        dark,
    );
    // A loop at the top.
    k.bx(
        bank,
        &mut m,
        v(-0.03, 0.265, -0.21),
        v(0.03, 0.285, -0.19),
        h.leather[1],
    );
    m
}

/// A willow basket with a cloth bundled over what's inside, and a leek and a loaf of bread
/// poking out behind.
fn wicker_pack(k: &mut Kit, bank: &mut TexBank, h: &Hue) -> Mesh {
    let mut m = Mesh::new();
    let weave = bank.add(wicker());
    skin_box(
        &mut m,
        v(-0.13, 0.0, -0.28),
        v(0.13, 0.215, BACK),
        weave,
        &k.w4,
    );
    k.bx(
        bank,
        &mut m,
        v(-0.14, 0.2, -0.29),
        v(0.14, 0.235, -0.11),
        CLAY,
    );
    let cloth = bank.add(canvas(h.cloth));
    lathe(
        &mut m,
        v(0.0, 0.215, -0.2),
        &[(0.11, 0.0), (0.115, 0.03), (0.08, 0.07), (0.0, 0.085)],
        8,
        0.4,
        cloth,
        false,
    );
    // A leek...
    let leek = Mat4::from_translation(v(0.07, 0.2, -0.26)) * Mat4::from_rotation_z(-0.3);
    let mut l = Mesh::new();
    k.bx(
        bank,
        &mut l,
        v(-0.018, 0.0, -0.018),
        v(0.018, 0.14, 0.018),
        WHITE,
    );
    k.bx(
        bank,
        &mut l,
        v(-0.02, 0.13, -0.02),
        v(0.02, 0.2, 0.02),
        LIME,
    );
    k.bx(
        bank,
        &mut l,
        v(-0.012, 0.19, -0.03),
        v(0.012, 0.27, 0.0),
        GREEN,
    );
    k.bx(
        bank,
        &mut l,
        v(-0.012, 0.19, 0.0),
        v(0.012, 0.25, 0.028),
        GREEN,
    );
    m.append(&l, leek);
    // ...and a loaf.
    let loaf = Mat4::from_translation(v(-0.07, 0.2, -0.25)) * Mat4::from_rotation_z(0.35);
    let mut b = Mesh::new();
    k.bx(
        bank,
        &mut b,
        v(-0.028, 0.0, -0.028),
        v(0.028, 0.2, 0.028),
        GOLD,
    );
    for y in [0.05f32, 0.1, 0.15] {
        k.bx(
            bank,
            &mut b,
            v(-0.02, y, -0.031),
            v(0.02, y + 0.012, -0.026),
            CREAM,
        );
    }
    m.append(&b, loaf);
    m
}

/// A long canvas roll strapped across the back, darker at the ends, with leather bands and
/// a handle on top.
fn duffel(k: &mut Kit, bank: &mut TexBank, h: &Hue) -> Mesh {
    let mut m = Mesh::new();
    let cloth = bank.add(canvas(h.cloth));
    let mut roll = Mesh::new();
    lathe(
        &mut roll,
        Vec3::ZERO,
        &[(0.085, -0.17), (0.1, -0.15), (0.1, 0.15), (0.085, 0.17)],
        8,
        0.4,
        cloth,
        true,
    );
    let end = k.c(bank, h.cloth[2]);
    let band = k.c(bank, h.leather[1]);
    for sy in [-1.0f32, 1.0] {
        // Darker ends, and a leather band round each.
        lathe(
            &mut roll,
            v(0.0, sy * 0.15 - 0.01, 0.0),
            &[(0.102, 0.0), (0.102, 0.02)],
            8,
            0.4,
            end,
            false,
        );
        lathe(
            &mut roll,
            v(0.0, sy * 0.08 - 0.012, 0.0),
            &[(0.103, 0.0), (0.103, 0.024)],
            8,
            0.4,
            band,
            false,
        );
    }
    m.append(
        &roll,
        Mat4::from_translation(v(0.0, 0.15, -0.22))
            * Mat4::from_rotation_z(std::f32::consts::FRAC_PI_2),
    );
    k.bx(
        bank,
        &mut m,
        v(-0.05, 0.245, -0.225),
        v(0.05, 0.262, -0.205),
        h.leather[1],
    );
    for sx in [-0.05f32, 0.04] {
        k.bx(
            bank,
            &mut m,
            v(sx, 0.235, -0.225),
            v(sx + 0.01, 0.262, -0.205),
            h.leather[1],
        );
    }
    m
}

/// An explorer's pack on a wooden frame that stands up past the shoulders, a bedroll lashed
/// across the top and a tin mug hanging off the side.
fn frame_pack(k: &mut Kit, bank: &mut TexBank, h: &Hue) -> Mesh {
    let mut m = Mesh::new();
    let cloth = bank.add(canvas(h.cloth));
    for sx in [-1.0f32, 1.0] {
        let (x0, x1) = if sx < 0.0 {
            (-0.15, -0.125)
        } else {
            (0.125, 0.15)
        };
        k.bx(
            bank,
            &mut m,
            v(x0, -0.03, -0.31),
            v(x1, 0.43, -0.285),
            ROSEWOOD,
        );
        k.bx(
            bank,
            &mut m,
            v(x0 - 0.004, 0.42, -0.314),
            v(x1 + 0.004, 0.44, -0.281),
            CLAY,
        );
    }
    k.bx(
        bank,
        &mut m,
        v(-0.14, 0.39, -0.305),
        v(0.14, 0.41, -0.29),
        ROSEWOOD,
    );
    k.bx(
        bank,
        &mut m,
        v(-0.14, 0.0, -0.305),
        v(0.14, 0.02, -0.29),
        ROSEWOOD,
    );
    skin_box(
        &mut m,
        v(-0.12, 0.02, -0.285),
        v(0.12, 0.23, BACK),
        cloth,
        &k.w4,
    );
    k.bx(
        bank,
        &mut m,
        v(-0.123, 0.17, -0.29),
        v(0.123, 0.235, -0.115),
        h.cloth[2],
    );
    for x in [-0.05f32, 0.05] {
        k.bx(
            bank,
            &mut m,
            v(x - 0.012, 0.1, -0.292),
            v(x + 0.012, 0.18, -0.286),
            h.leather[1],
        );
        k.bx(
            bank,
            &mut m,
            v(x - 0.016, 0.1, -0.295),
            v(x + 0.016, 0.12, -0.29),
            GOLD,
        );
    }
    // The bedroll.
    let roll_t = bank.add(canvas([h.cloth[0], h.cloth[0], h.cloth[1]]));
    let mut roll = Mesh::new();
    lathe(
        &mut roll,
        Vec3::ZERO,
        &[(0.055, -0.15), (0.065, -0.13), (0.065, 0.13), (0.055, 0.15)],
        8,
        0.4,
        roll_t,
        true,
    );
    let tie = k.c(bank, h.leather[1]);
    for sy in [-1.0f32, 1.0] {
        lathe(
            &mut roll,
            v(0.0, sy * 0.08 - 0.01, 0.0),
            &[(0.068, 0.0), (0.068, 0.02)],
            8,
            0.4,
            tie,
            false,
        );
    }
    m.append(
        &roll,
        Mat4::from_translation(v(0.0, 0.33, -0.33))
            * Mat4::from_rotation_z(std::f32::consts::FRAC_PI_2),
    );
    // A tin mug on the side.
    k.bx(
        bank,
        &mut m,
        v(0.15, 0.06, -0.24),
        v(0.2, 0.12, -0.19),
        SLATE,
    );
    k.bx(
        bank,
        &mut m,
        v(0.152, 0.115, -0.238),
        v(0.198, 0.12, -0.192),
        SKY,
    );
    m
}

/// A great snail shell, hollowed out and worn on the back, its spiral facing out.
fn shell_pack(k: &mut Kit, bank: &mut TexBank, h: &Hue) -> Mesh {
    let mut m = Mesh::new();
    let skin = bank.add(shell(h.cloth));
    let mut s = Mesh::new();
    lathe(
        &mut s,
        Vec3::ZERO,
        &[
            (0.15, 0.0),
            (0.16, 0.035),
            (0.15, 0.08),
            (0.115, 0.125),
            (0.065, 0.155),
            (0.0, 0.165),
        ],
        10,
        0.0,
        skin,
        false,
    );
    // Its lip, round the rim against the back.
    let lip = k.c(bank, h.cloth[0]);
    lathe(
        &mut s,
        Vec3::ZERO,
        &[(0.13, -0.01), (0.155, 0.005)],
        10,
        0.0,
        lip,
        false,
    );
    m.append(
        &s,
        Mat4::from_translation(v(0.0, 0.14, BACK - 0.01))
            * Mat4::from_rotation_x(-std::f32::consts::FRAC_PI_2),
    );
    m
}

const KNAPSACK: &[&str] = &[
    "................",
    ".....KK..KK.....",
    "....K00KK00K....",
    ".....K1111K.....",
    "......3YY3......",
    "....KK1111KK....",
    "...K011111111K..",
    "..K0111111112K..",
    ".K011111111112K.",
    ".K011100001112K.",
    ".K111100001122K.",
    ".K111100001122K.",
    ".K211111111122K.",
    "..K2211111122K..",
    "...KK222222KK...",
    ".....KKKKKK.....",
];

const RUCKSACK: &[&str] = &[
    "................",
    "......KKKK......",
    ".....K....K.....",
    "..KKKKKKKKKKKK..",
    ".K222222222222K.",
    ".K222222222222K.",
    ".K223222222322K.",
    ".KKK3KKKKKK3KKK.",
    ".K113111111311K.",
    "KK11Y111111Y11KK",
    "K1K1111111111K1K",
    "K1K1KKKKKKKK1K1K",
    "K1K1K000000K1K1K",
    "KKK1K000000K1KKK",
    ".K11KKKKKKKK11K.",
    "..KKKKKKKKKKKK..",
];

const WICKER: &[&str] = &[
    "..........KK....",
    "..Kg.....KeeK...",
    "..Kgg...KeeK....",
    "...Kg.KKKeK.....",
    "...KyK0000KK....",
    "..KKyK1111110K..",
    ".KKKKKKKKKKKKKK.",
    ".KuuuuuuuuuuuuK.",
    ".KnhnhnhnhnhnhK.",
    ".KhnhnhnhnhnhnK.",
    ".KnhnhnhnhnhnhK.",
    ".KhnhnhnhnhnhnK.",
    ".KnhnhnhnhnhnhK.",
    ".KuuuuuuuuuuuuK.",
    "..KKKKKKKKKKKK..",
    "................",
];

const DUFFEL: &[&str] = &[
    "................",
    "................",
    ".....KKKKKK.....",
    "....K......K....",
    "..KKKKKKKKKKKK..",
    ".K2K00000000K2K.",
    "K22K11111111K22K",
    "K22K13111131K22K",
    "K22K13111131K22K",
    "K22K13111131K22K",
    "K22K11111111K22K",
    ".K2K22222222K2K.",
    "..KKKKKKKKKKKK..",
    "................",
    "................",
    "................",
];

const FRAME: &[&str] = &[
    "..KK........KK..",
    "..KRK......KRK..",
    ".KKRKKKKKKKKRKK.",
    ".K0R000330000RK.",
    ".K1R111331111RK.",
    ".KKRKKKKKKKKRKK.",
    "..KR22222222RK..",
    "..KR21111112RK..",
    "..KR21111112RK..",
    "..KR21Y11Y12RK..",
    "..KR21111112RK..",
    "..KR22222222RK..",
    "..KRKKKKKKKKRK..",
    "..KR........RK..",
    "..KK........KK..",
    "................",
];

/// The shell's icon: a spiral, drawn outright.
fn shell_icon([light, mid, dark]: [u8; 3]) -> Texture {
    let mut t = Texture::clear(16, 16);
    let (cx, cy) = (7.5f32, 8.0f32);
    for y in 0..16 {
        for x in 0..16 {
            let (dx, dy) = (x as f32 - cx, y as f32 - cy);
            let r = (dx * dx + dy * dy).sqrt();
            if r > 7.2 {
                continue;
            }
            if r > 6.3 {
                t.set(x, y, INK);
                continue;
            }
            let turn = (dy.atan2(dx) / std::f32::consts::TAU + 0.5) * 2.2;
            let band = ((r + turn) / 2.2).floor() as i32;
            let c = match band.rem_euclid(3) {
                0 => light,
                1 => mid,
                _ => dark,
            };
            t.set(x, y, if r < 1.0 { dark } else { c });
        }
    }
    t
}

fn icon(style: usize, h: &Hue) -> Texture {
    let [light, mid, dark] = h.cloth;
    let slots = [light, mid, dark, h.leather[1]];
    match style {
        0 => art(KNAPSACK, slots),
        1 => art(RUCKSACK, slots),
        2 => art(WICKER, slots),
        3 => art(DUFFEL, slots),
        4 => art(FRAME, slots),
        _ => shell_icon(h.cloth),
    }
}

/// The icon names the cloth backpacks go by (in their first colourway), in style order.
pub const ICONS: [&str; CLOTH] = [
    "pack_knapsack",
    "pack_rucksack",
    "pack_wicker",
    "pack_duffel",
    "pack_frame",
    "pack_shell",
];

pub fn build(bank: &mut TexBank, icons: &mut HashMap<&'static str, TexId>) -> PackArt {
    let mut k = Kit {
        solid: HashMap::new(),
        w4: Texture::new(4, 4, 0),
    };
    let mut worn = Vec::new();
    let mut pics = Vec::new();
    // Grown out from the back; the straps stay put.
    let wear = |k: &mut Kit, bank: &mut TexBank, bag: Mesh, h: &Hue| {
        let mut m = Mesh::new();
        m.append(&bag, grow());
        straps(k, bank, &mut m, h);
        m
    };
    for (style, name) in ICONS.iter().enumerate() {
        let make = [
            knapsack,
            rucksack,
            wicker_pack,
            duffel,
            frame_pack,
            shell_pack,
        ][style];
        let looks = HUE
            .iter()
            .map(|h| {
                let bag = make(&mut k, bank, h);
                wear(&mut k, bank, bag, h)
            })
            .collect();
        worn.push(looks);
        let row: Vec<TexId> = HUE.iter().map(|h| bank.add(icon(style, h))).collect();
        icons.insert(name, row[0]);
        pics.push(row);
    }
    // The monster backpacks, with plain leather straps.
    let strap = Hue {
        cloth: [SAND, KHAKI, ROSEWOOD],
        leather: [KHAKI, ROSEWOOD],
    };
    for p in 0..super::plush_art::PLUSHES {
        let bag = super::plush_art::plush(p, &mut k, bank);
        worn.push(vec![wear(&mut k, bank, bag, &strap)]);
        let t = bank.add(super::plush_art::icon(p));
        icons.insert(super::plush_art::ICONS[p], t);
        pics.push(vec![t]);
    }
    PackArt { worn, icons: pics }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pack_icons_fit_their_frames() {
        for rows in [KNAPSACK, RUCKSACK, WICKER, DUFFEL, FRAME] {
            assert_eq!(rows.len(), 16);
            for r in rows {
                assert_eq!(r.chars().count(), 16, "{r}");
            }
        }
        let t = shell_icon(HUE[0].cloth);
        assert_eq!(t.get(0, 0), CLEAR);
        assert_ne!(t.get(8, 8), CLEAR);
    }
}
