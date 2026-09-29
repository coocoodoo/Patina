//! Folk of the deeper Hollow (floor 11 and down), in a look for every biome: lantern snails
//! under glowing stained-glass shells, and book-worm bibliomancers, bespectacled
//! caterpillars that read from a little floating book and spit glowing ink. The ink dries
//! into runes on the floor, drawn from the textures here.
//!
//! Local space as everywhere: the ground under the creature, +y up, facing +z. The parts
//! that move on their own (a snail's eye stalks, a worm's segments and book) are separate
//! meshes posed by `game::foes`.

use std::collections::HashMap;
use std::f32::consts::{FRAC_PI_2, TAU};

use glam::{Mat4, Vec3};

use super::models::{lathe, skin_box};
use super::tiles;
use super::{LOOKS, SEWER_LOOK};
use crate::palette::*;
use crate::render::{Mesh, TexBank, TexId, Texture, UvRect};

/// Where a snail's shell sits, and how big it is (its radius).
pub const SHELL_AT: Vec3 = Vec3::new(0.0, 0.2, -0.07);
pub const SHELL_R: f32 = 0.19;
/// Where its two eye stalks leave its head, and how long they are.
pub const STALK_AT: Vec3 = Vec3::new(0.035, 0.2, 0.19);
pub const STALK_LEN: f32 = 0.13;

/// A book-worm's segments: how many trail behind its head, how far apart, and how big.
pub const SEGMENTS: usize = 5;
pub const SEG_GAP: f32 = 0.15;
pub const SEG_R: f32 = 0.095;
/// Its head's centre.
pub const HEAD_AT: Vec3 = Vec3::new(0.0, 0.13, 0.0);

pub struct Snail {
    /// The soft foot, head and face (the eyes ride on the stalks).
    pub body: Mesh,
    /// A stalk rising from its base (+y), an eye on top.
    pub stalk: Mesh,
    /// The stained-glass shell, centred on its middle: drawn unlit, it glows.
    pub shell: Mesh,
    /// The shell's colours, lightest first, for its glow and slime trail.
    pub glass: [u8; 3],
}

pub struct Worm {
    /// Its head, spectacles, antennae and all, centred on the head's middle.
    pub head: Mesh,
    /// Body segments in its two colours, centred on their middles, feet below.
    pub segs: [Mesh; 2],
    /// The little book it reads from, open, centred on its spine.
    pub book: Mesh,
    /// Its pages, which glow as it spits.
    pub pages: Mesh,
    /// The ink it spits (light, main).
    pub ink: [u8; 2],
}

pub struct DeepArt {
    pub snails: Vec<Snail>,
    pub worms: Vec<Worm>,
    /// Ink runes on the floor, arrows pointing up the texture, by biome (then the sewers');
    /// the last is the gold one pointing to a secret.
    pub runes: Vec<TexId>,
}

/// A snail's glass by biome (and then the sewers'): three panes' colours, lightest first.
pub const GLASS: [[u8; 3]; LOOKS] = [
    [LIME, GOLD, AQUA],
    [MINT, AQUA, LAVENDER],
    [BLUSH, PINK, LAVENDER],
    [GOLD, ORANGE, RED],
    [WHITE, SKY, MINT],
    [CREAM, GOLD, SALMON],
    [LIME, MINT, GREEN],
];

/// Snail bodies by biome (light, mid, dark); the sewers' are bone.
const SNAIL_SKIN: [[u8; 3]; LOOKS] = [
    [PEACH, SALMON, ROSEWOOD],
    [WHITE, SKY, BLUE],
    [BLUSH, PINK, PLUM],
    [SAND, KHAKI, ROSEWOOD],
    [WHITE, SKY, SLATE],
    [SAND, CLAY, RUST],
    [WHITE, SAND, KHAKI],
];

/// Book-worms by biome: the two segment colours, the spots on them, and spectacle rims.
/// The sewers' are strings of old vertebrae.
const WORM_SKIN: [[u8; 4]; LOOKS] = [
    [LIME, GREEN, GOLD, GOLD],
    [AQUA, TEAL, WHITE, GOLD],
    [LAVENDER, PURPLE, PINK, GOLD],
    [ORANGE, RED, GOLD, INK],
    [WHITE, SKY, BLUE, GOLD],
    [SAND, KHAKI, CREAM, INK],
    [WHITE, SAND, KHAKI, RUST],
];

/// The glowing ink each biome's book-worms spit (light, main).
pub const INK_COLORS: [[u8; 2]; LOOKS] = [
    [WHITE, MINT],
    [WHITE, AQUA],
    [WHITE, PINK],
    [CREAM, GOLD],
    [WHITE, SKY],
    [WHITE, LAVENDER],
    [CREAM, LIME],
];

/// Book covers by biome (the sewers' has gone green).
const COVERS: [u8; LOOKS] = [RED, INDIGO, GRAPE, MAROON, BLUE, RUST, DEEP_TEAL];

struct Kit {
    solid: HashMap<u8, TexId>,
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
        skin_box(m, a.min(b), a.max(b), t, &self.w4);
    }
}

fn v(x: f32, y: f32, z: f32) -> Vec3 {
    Vec3::new(x, y, z)
}

/// A round lump `r` across and `h` tall, standing at `at`.
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

/// A lathe turned on its side: `prof` runs along +z (a profile's y becomes z).
fn along_z(m: &mut Mesh, prof: &[(f32, f32)], seg: usize, tex: TexId, place: Mat4) {
    let mut part = Mesh::new();
    lathe(&mut part, Vec3::ZERO, prof, seg, 0.0, tex, false);
    m.append(&part, place * Mat4::from_rotation_x(FRAC_PI_2));
}

// ------------------------------------------------------------------------------------------
// Stained glass
// ------------------------------------------------------------------------------------------

/// A little hash for picking pane colours.
fn hash(a: i32, b: i32) -> u32 {
    let mut h = (a as u32).wrapping_mul(0x9E37_79B1) ^ (b as u32).wrapping_mul(0x85EB_CA77);
    h ^= h >> 15;
    h = h.wrapping_mul(0x2C1B_3C6D);
    h ^ (h >> 12)
}

/// Stained glass for a shell. The top 32x32 is a round window for its sides: panes in the
/// glass colours, leaded along a spiral that winds out from a bright middle, with a lead
/// ring round the edge. Below it, 32 rows of panes for the rim.
pub fn stained_glass(glass: [u8; 3], lead: u8) -> Texture {
    let mut t = Texture::new(32, 64, lead);
    let pitch = 4.6;
    for y in 0..32 {
        for x in 0..32 {
            let (dx, dy) = (x as f32 - 15.5, y as f32 - 15.5);
            let r = (dx * dx + dy * dy).sqrt();
            let turn = (dy.atan2(dx) / TAU).rem_euclid(1.0);
            // How far out along the spiral: one band per turn.
            let along = r - turn * pitch;
            let band = (along / pitch).floor() as i32;
            let within = along - band as f32 * pitch;
            let c = if r > 15.2 {
                lead
            } else if r < 2.2 {
                WHITE
            } else if within < 1.0 {
                lead
            } else {
                // Each band is cut into panes, more of them further out.
                let panes = 3 + band.max(0) as u32;
                let k = (turn * panes as f32).floor() as i32;
                let edge = (turn * panes as f32).fract();
                if edge < 0.06 + 0.02 * band.max(0) as f32 && r > 4.0 {
                    lead
                } else if within > pitch - 1.8 && edge > 0.5 && edge < 0.62 {
                    // A glint in the corner of the pane.
                    WHITE
                } else {
                    glass[(hash(band, k) % 3) as usize]
                }
            };
            t.set(x, y, c);
        }
    }
    // The rim: rows of panes, staggered like brickwork.
    for y in 32..64 {
        for x in 0..32 {
            let row = (y - 32) / 6;
            let col = (x + row * 3) / 6;
            let c = if (y - 32) % 6 == 0 || (x + row * 3) % 6 == 0 {
                lead
            } else if (x + row * 3) % 6 == 1 && (y - 32) % 6 == 1 {
                WHITE
            } else {
                glass[(hash(row, col) % 3) as usize]
            };
            t.set(x, y, c);
        }
    }
    t
}

/// The shell: a plump wheel of stained glass leaded with `lead`, standing on edge, its
/// round windows facing left and right, centred on its middle.
pub fn shell_mesh(bank: &mut TexBank, glass: [u8; 3], lead: u8) -> Mesh {
    let tex = bank.add(stained_glass(glass, lead));
    let r = SHELL_R;
    let w = r * 0.5;
    // The rim, round-shouldered, textured with the rows of panes (v 32..64).
    let prof = [
        (r * 0.84, -w, 32.0),
        (r * 0.96, -w * 0.72, 38.0),
        (r, -w * 0.2, 44.0),
        (r, w * 0.2, 52.0),
        (r * 0.96, w * 0.72, 58.0),
        (r * 0.84, w, 63.9),
    ];
    let mut wheel = Mesh::new();
    let window = UvRect::new(0.0, 0.0, 32.0, 32.0);
    wheel.lathe(
        Vec3::ZERO,
        &prof,
        14,
        32.0,
        0.0,
        tex,
        (Some(window), Some(window)),
    );
    let mut m = Mesh::new();
    // Stood on edge (its axis across the body), leaning back a touch.
    m.append(
        &wheel,
        Mat4::from_rotation_x(-0.18) * Mat4::from_rotation_z(FRAC_PI_2),
    );
    m
}

fn snail(bank: &mut TexBank, k: &mut Kit, biome: usize) -> Snail {
    let [light, mid, dark] = SNAIL_SKIN[biome];
    let glass = GLASS[biome];
    // A bone snail: all vertebrae, its shell a lattice of bone round green glass, and its
    // eyes lit like lamps.
    let bones = biome == SEWER_LOOK;
    // Soft skin with pale freckles (or bones with dark joints between them).
    let mut skin = Texture::new(16, 16, mid);
    for y in 0..16 {
        for x in 0..16 {
            if bones {
                skin.set(
                    x,
                    y,
                    match x % 4 {
                        3 => SHADOW,
                        0 => light,
                        _ => mid,
                    },
                );
            } else if (x * 5 + y * 3) % 13 == 0 {
                skin.set(x, y, light);
            } else if (x * 3 + y * 7) % 19 == 0 {
                skin.set(x, y, dark);
            }
        }
    }
    let skin = bank.add(skin);
    let mut body = Mesh::new();
    // The foot: long, low and rounded, tapering to a tail behind.
    let mut foot = Mesh::new();
    along_z(
        &mut foot,
        &[
            (0.0, -0.36),
            (0.04, -0.33),
            (0.075, -0.24),
            (0.098, -0.1),
            (0.105, 0.05),
            (0.1, 0.15),
            (0.08, 0.22),
            (0.0, 0.26),
        ],
        9,
        skin,
        Mat4::IDENTITY,
    );
    body.append(
        &foot,
        Mat4::from_translation(v(0.0, 0.056, 0.0)) * Mat4::from_scale(v(1.05, 0.55, 1.0)),
    );
    // The head, raised at the front.
    blob(&mut body, v(0.0, 0.03, 0.17), 0.085, 0.2, skin);
    // A little smile and rosy cheeks.
    k.bx(
        bank,
        &mut body,
        v(-0.018, 0.115, 0.248),
        v(0.018, 0.126, 0.258),
        INK,
    );
    k.bx(
        bank,
        &mut body,
        v(-0.012, 0.106, 0.246),
        v(0.012, 0.115, 0.256),
        dark,
    );
    for s in [-1.0f32, 1.0] {
        k.bx(
            bank,
            &mut body,
            v(s * 0.043, 0.13, 0.236),
            v(s * 0.068, 0.145, 0.246),
            if bones { dark } else { PINK },
        );
    }
    // A stalk and its eye: white with a big dark pupil and a sparkle (a dark socket with a
    // glow in it, on a bone snail).
    let (white, pupil) = if bones { (INK, LIME) } else { (WHITE, INK) };
    let mut stalk = Mesh::new();
    k.bx(
        bank,
        &mut stalk,
        v(-0.011, 0.0, -0.011),
        v(0.011, STALK_LEN, 0.011),
        mid,
    );
    k.bx(
        bank,
        &mut stalk,
        v(-0.03, STALK_LEN - 0.01, -0.028),
        v(0.03, STALK_LEN + 0.05, 0.03),
        white,
    );
    k.bx(
        bank,
        &mut stalk,
        v(-0.018, STALK_LEN + 0.002, 0.024),
        v(0.018, STALK_LEN + 0.042, 0.034),
        pupil,
    );
    k.bx(
        bank,
        &mut stalk,
        v(0.004, STALK_LEN + 0.026, 0.031),
        v(0.014, STALK_LEN + 0.036, 0.037),
        WHITE,
    );
    Snail {
        body,
        stalk,
        shell: shell_mesh(bank, glass, if bones { SAND } else { INK }),
        glass,
    }
}

// ------------------------------------------------------------------------------------------
// Book-worms
// ------------------------------------------------------------------------------------------

/// A body segment's coat: its colour, a band round its middle and a spot on top.
fn worm_coat(main: u8, band: u8, spot: u8) -> Texture {
    let mut t = Texture::new(16, 16, main);
    for y in 0..16 {
        for x in 0..16 {
            if (6..=7).contains(&y) {
                t.set(x, y, band);
            }
            // Spots on the back, either side of the ridge.
            if y < 5 && (x % 8 == 2 || x % 8 == 3) && y > 1 {
                t.set(x, y, spot);
            }
        }
    }
    t
}

/// A ring of spectacle rim, `r` across, facing +z.
fn rim(m: &mut Mesh, tex: TexId, at: Vec3, r: f32) {
    let mut ring = Mesh::new();
    let t = 0.012;
    lathe(
        &mut ring,
        Vec3::ZERO,
        &[
            (r - t, -0.006),
            (r, -0.006),
            (r + 0.002, 0.0),
            (r, 0.006),
            (r - t, 0.006),
            (r - t - 0.002, 0.0),
            (r - t, -0.006),
        ],
        12,
        0.0,
        tex,
        false,
    );
    m.append(
        &ring,
        Mat4::from_translation(at) * Mat4::from_rotation_x(FRAC_PI_2),
    );
}

fn worm(bank: &mut TexBank, k: &mut Kit, biome: usize) -> Worm {
    let [a, b, spot, specs] = WORM_SKIN[biome];
    // A bone worm: vertebrae with dark joints between, and a skull for a head.
    let bones = biome == SEWER_LOOK;
    let coats = if bones {
        [
            bank.add(worm_coat(a, SHADOW, spot)),
            bank.add(worm_coat(b, SHADOW, spot)),
        ]
    } else {
        [
            bank.add(worm_coat(a, b, spot)),
            bank.add(worm_coat(b, a, spot)),
        ]
    };
    let (white, pupil) = if bones { (INK, LIME) } else { (WHITE, INK) };
    let segs = [0, 1].map(|i| {
        let mut m = Mesh::new();
        blob(&mut m, v(0.0, -SEG_R, 0.0), SEG_R, SEG_R * 2.0, coats[i]);
        // Stubby feet underneath.
        for s in [-1.0f32, 1.0] {
            k.bx(
                bank,
                &mut m,
                v(s * 0.045, -SEG_R, -0.02),
                v(s * 0.07, -SEG_R + 0.03, 0.02),
                if i == 0 { b } else { a },
            );
        }
        m
    });
    let mut head = Mesh::new();
    let hr = 0.12;
    let face = bank.add(tiles::solid(a));
    blob(&mut head, v(0.0, -hr * 0.95, 0.0), hr, hr * 1.9, face);
    // Big round spectacles, the eyes peering through, and the bridge between.
    let rims = k.c(bank, specs);
    for s in [-1.0f32, 1.0] {
        let c = v(s * 0.05, 0.02, hr * 0.84);
        rim(&mut head, rims, c, 0.044);
        k.bx(
            bank,
            &mut head,
            c + v(-0.03, -0.028, -0.03),
            c + v(0.03, 0.03, -0.006),
            white,
        );
        k.bx(
            bank,
            &mut head,
            c + v(-0.016, -0.018, -0.01),
            c + v(0.016, 0.018, -0.002),
            pupil,
        );
        k.bx(
            bank,
            &mut head,
            c + v(0.002, 0.004, -0.006),
            c + v(0.012, 0.014, 0.0),
            WHITE,
        );
        // A hint of glass across the lens, catching the light.
        k.bx(
            bank,
            &mut head,
            c + v(-0.03, 0.018, -0.004),
            c + v(-0.018, 0.03, 0.0),
            SKY,
        );
        // Arms of the spectacles, back over the ears.
        k.bx(
            bank,
            &mut head,
            v(s * 0.093, 0.03, 0.0),
            v(s * 0.099, 0.036, hr * 0.84),
            specs,
        );
        // Antennae with bobbles.
        k.bx(
            bank,
            &mut head,
            v(s * 0.035, hr * 0.7, -0.01),
            v(s * 0.047, hr * 1.35, 0.002),
            b,
        );
        k.bx(
            bank,
            &mut head,
            v(s * 0.02, hr * 1.35, -0.022),
            v(s * 0.062, hr * 1.35 + 0.04, 0.016),
            spot,
        );
        // Rosy cheeks (just bone, on a skull).
        k.bx(
            bank,
            &mut head,
            v(s * 0.06, -0.045, hr * 0.8),
            v(s * 0.085, -0.03, hr * 0.84),
            if bones { b } else { PINK },
        );
    }
    k.bx(
        bank,
        &mut head,
        v(-0.008, 0.02, hr * 0.88),
        v(0.008, 0.028, hr * 0.92),
        specs,
    );
    // A small, clever smile (a skull's grin of teeth).
    k.bx(
        bank,
        &mut head,
        v(-0.02, -0.058, hr * 0.84),
        v(0.02, -0.048, hr * 0.9),
        INK,
    );
    if bones {
        for x in [-0.014f32, 0.0, 0.014] {
            k.bx(
                bank,
                &mut head,
                v(x - 0.004, -0.058, hr * 0.89),
                v(x + 0.004, -0.05, hr * 0.92),
                WHITE,
            );
        }
    }

    // The book: two covers open in a shallow V, pages fanned on top.
    let cover = COVERS[biome];
    let mut book = Mesh::new();
    let mut pages = Mesh::new();
    for s in [-1.0f32, 1.0] {
        let mut half = Mesh::new();
        k.bx(
            bank,
            &mut half,
            v(0.0, -0.006, -0.06),
            v(0.1, 0.0, 0.06),
            cover,
        );
        k.bx(
            bank,
            &mut half,
            v(0.08, 0.0, -0.06),
            v(0.1, 0.004, 0.06),
            GOLD,
        );
        book.append(
            &half,
            Mat4::from_scale(v(s, 1.0, 1.0)) * Mat4::from_rotation_z(0.25),
        );
        let mut leaf = Mesh::new();
        k.bx(
            bank,
            &mut leaf,
            v(0.004, 0.0, -0.052),
            v(0.088, 0.014, 0.052),
            CREAM,
        );
        // Lines of writing.
        for row in 0..4 {
            let z = -0.036 + row as f32 * 0.022;
            k.bx(
                bank,
                &mut leaf,
                v(0.018, 0.014, z),
                v(0.074, 0.017, z + 0.008),
                INDIGO,
            );
        }
        pages.append(
            &leaf,
            Mat4::from_scale(v(s, 1.0, 1.0)) * Mat4::from_rotation_z(0.25),
        );
    }
    Worm {
        head,
        segs,
        book,
        pages,
        ink: INK_COLORS[biome],
    }
}

// ------------------------------------------------------------------------------------------
// Ink runes
// ------------------------------------------------------------------------------------------

/// A rune of ink on the floor: a ring of little marks round an arrow pointing up the
/// texture, or for a secret, round a star.
pub fn rune(light: u8, main: u8, secret: bool) -> Texture {
    let mut t = Texture::new(16, 16, CLEAR);
    // The ring: dashes round a circle.
    for i in 0..20 {
        let a = i as f32 / 20.0 * TAU;
        if i % 5 == 4 {
            continue;
        }
        let (x, y) = (7.5 + a.cos() * 7.0, 7.5 + a.sin() * 7.0);
        t.set(x.round() as i32, y.round() as i32, main);
    }
    // The arrow: a shaft and a broad head, lit down the middle.
    for y in 4..13 {
        t.set(7, y, main);
        t.set(8, y, light);
    }
    for i in 0..4 {
        let y = 3 + i;
        t.set(7 - i, y + 1, main);
        t.set(8 + i, y + 1, main);
        t.set(7 - i, y + 2, main);
        t.set(8 + i, y + 2, main);
    }
    t.set(7, 2, light);
    t.set(8, 2, light);
    if secret {
        // A little star in the tail.
        for (x, y) in [(7, 12), (8, 12), (6, 13), (9, 13), (7, 14), (8, 14)] {
            t.set(x, y, light);
        }
        t.set(5, 12, main);
        t.set(10, 12, main);
    }
    t
}

// ------------------------------------------------------------------------------------------
// Icons
// ------------------------------------------------------------------------------------------

/// A shard of stained glass (slots: light, mid and dark panes).
const SHARD: &[&str] = &[
    "................",
    "........K.......",
    ".......K0K......",
    "......K0w0K.....",
    ".....K000K1K....",
    "....K000K111K...",
    "...K000K11111K..",
    "..KKKKK111111K..",
    "..K2222KKKK11K..",
    "..K22222222KKK..",
    "..K2w2222222K...",
    "...K22222222K...",
    "....KK22222K....",
    "......KKKKK.....",
    "................",
    "................",
];

/// A bottle of glowing ink (slots: the ink's light and main colours).
const INK_BOTTLE: &[&str] = &[
    "................",
    "......KKKK......",
    "......KCCK......",
    ".....KKuuKK.....",
    "......KSSK......",
    ".....KS00SK.....",
    "....KS0w00SK....",
    "...KS001100SK...",
    "...KS011110SK...",
    "...KS111111SK...",
    "...KS111111SK...",
    "...KS011110SK...",
    "....KS1111SK....",
    ".....KKKKKK.....",
    "................",
    "................",
];

/// A map drawn in glowing ink: a dotted way to an arrow (slots: the ink's light and main).
const INK_MAP: &[&str] = &[
    "................",
    "..KKKKKKKKKKKK..",
    ".KyyyyyyyyyyyyK.",
    ".KynnnnnnnnnnyK.",
    "..KnnnnnnK0nnK..",
    "..Kn1nnnK00KnK..",
    "..KnnnnK0w00KK..",
    "..Kn1nnnnK0nnK..",
    "..KnnnnnnK1nnK..",
    "..Kn1n1n11nnnK..",
    "..KnnnnnnnnnnK..",
    "..K1nnnnnnnnnK..",
    ".KyyyyyyyyyyyyK.",
    ".KyKKKKKKKKKKyK.",
    "..KK........KK..",
    "................",
];

/// Icons for what the deeper folk leave behind, and what's made from it.
pub fn icons(bank: &mut TexBank, m: &mut HashMap<&'static str, TexId>) {
    use super::sprites::art;
    m.insert(
        "stained_glass",
        bank.add(art(SHARD, [AQUA, PINK, GOLD, CLEAR])),
    );
    m.insert(
        "glow_ink",
        bank.add(art(INK_BOTTLE, [MINT, AQUA, CLEAR, CLEAR])),
    );
    m.insert(
        "ink_map",
        bank.add(art(INK_MAP, [MINT, AQUA, CLEAR, CLEAR])),
    );
}

pub fn build(bank: &mut TexBank) -> DeepArt {
    let mut k = Kit {
        solid: HashMap::new(),
        w4: Texture::new(4, 4, 0),
    };
    let snails = (0..LOOKS).map(|b| snail(bank, &mut k, b)).collect();
    let worms = (0..LOOKS).map(|b| worm(bank, &mut k, b)).collect();
    let mut runes: Vec<TexId> = INK_COLORS
        .iter()
        .map(|&[l, m]| bank.add(rune(l, m, false)))
        .collect();
    runes.push(bank.add(rune(CREAM, GOLD, true)));
    DeepArt {
        snails,
        worms,
        runes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stained_glass_is_leaded_glass() {
        for g in GLASS {
            let t = stained_glass(g, INK);
            let mut counts = [0usize; 32];
            for y in 0..64 {
                for x in 0..32 {
                    counts[t.get(x, y) as usize] += 1;
                }
            }
            // Every pane colour shows, divided by plenty of lead.
            for c in g {
                assert!(counts[c as usize] > 40, "{c} barely shows");
            }
            assert!(counts[INK as usize] > 200);
        }
    }

    #[test]
    fn icons_are_tidy() {
        for rows in [SHARD, INK_BOTTLE, INK_MAP] {
            assert_eq!(rows.len(), 16);
            for r in rows {
                assert_eq!(r.chars().count(), 16, "{r}");
            }
        }
    }

    #[test]
    fn runes_point_up() {
        let t = rune(WHITE, MINT, false);
        // The arrowhead is at the top, wider than the shaft below it.
        let row = |y: i32| (0..16).filter(|&x| t.get(x, y) != CLEAR).count();
        assert!(row(5) > row(10));
    }
}
