//! Home art: every piece of furniture as a little model and an icon, rugs, pictures for the
//! wall, and swatches for the wallpapers and floors. Models stand on the ground at the middle
//! of the tiles they cover, front towards +z.

use std::collections::HashMap;
use std::f32::consts::{FRAC_PI_2, TAU};

use glam::{Mat4, Vec3};

use super::models::{lathe, skin_box, tiled_box};
use super::sprites::{NO, art};
use super::tiles;
use crate::game::home::{FURNS, Furn, RUGS};
use crate::palette::*;
use crate::render::{Mesh, TexBank, TexId, Texture, UvRect};

pub struct HomeArt {
    /// One model per piece of furniture, in `FURNS` order.
    pub furn: Vec<Mesh>,
    /// Drawn see-through, over the fish swimming inside.
    pub tank_glass: Mesh,
    pub tank_water: Mesh,
    pub bowl_glass: Mesh,
    pub bowl_water: Mesh,
    /// The floor lamp's shade, which glows.
    pub lamp_shade: Mesh,
    /// Swings inside the grandfather clock.
    pub pendulum: Mesh,
    /// Spins on the globe's stand.
    pub globe: Mesh,
    /// Rugs by kind (see `home::RUGS`).
    pub rugs: Vec<TexId>,
    /// Pictures for the back wall, by kind.
    pub art: Vec<Mesh>,
}

/// Solid colours and textures shared between models.
struct Kit {
    solids: HashMap<u8, TexId>,
    w4: Texture,
}

impl Kit {
    fn solid(&mut self, bank: &mut TexBank, c: u8) -> TexId {
        *self
            .solids
            .entry(c)
            .or_insert_with(|| bank.add(tiles::solid(c)))
    }

    fn paint(&mut self, bank: &mut TexBank, m: &mut Mesh, min: Vec3, max: Vec3, c: u8) {
        let t = self.solid(bank, c);
        skin_box(m, min, max, t, &self.w4);
    }
}

fn v(x: f32, y: f32, z: f32) -> Vec3 {
    Vec3::new(x, y, z)
}

/// A shaded metal: highlight, base, shadow.
fn metal(bank: &mut TexBank, c: [u8; 3]) -> TexId {
    let mut t = Texture::new(4, 4, c[1]);
    t.set(0, 0, c[0]);
    t.set(1, 1, c[0]);
    t.set(0, 2, c[0]);
    t.set(3, 3, c[2]);
    t.set(2, 3, c[2]);
    t.set(3, 1, c[2]);
    bank.add(t)
}

/// Four table legs inside a rectangle.
fn legs(k: &mut Kit, bank: &mut TexBank, m: &mut Mesh, x: f32, z: f32, h: f32, c: u8) {
    for (sx, sz) in [(-1.0, -1.0), (1.0, -1.0), (-1.0, 1.0), (1.0, 1.0)] {
        let (cx, cz) = (sx * x, sz * z);
        k.paint(
            bank,
            m,
            v(cx - 0.035, 0.0, cz - 0.035),
            v(cx + 0.035, h, cz + 0.035),
            c,
        );
    }
}

fn furniture(bank: &mut TexBank, k: &mut Kit, f: Furn) -> Mesh {
    let mut m = Mesh::new();
    let wood = bank.add(tiles::planks(CLAY, GOLD, RUST, 140));
    let dark = bank.add(tiles::planks(RUST, CLAY, MAROON, 141));
    match f {
        Furn::Bed | Furn::CanopyBed => {
            let (hw, hd) = if f == Furn::Bed {
                (0.44, 0.94)
            } else {
                (0.9, 0.92)
            };
            let quilt = if f == Furn::Bed {
                bank.add(tiles::checker(PINK, SALMON, BLUSH))
            } else {
                bank.add(tiles::checker(LAVENDER, PURPLE, GOLD))
            };
            let frame = if f == Furn::Bed { wood } else { dark };
            tiled_box(&mut m, v(-hw, 0.1, -hd), v(hw, 0.34, hd), frame, 0);
            tiled_box(
                &mut m,
                v(-hw, 0.0, -hd - 0.04),
                v(hw, 0.92, -hd + 0.08),
                frame,
                0,
            );
            tiled_box(
                &mut m,
                v(-hw, 0.0, hd - 0.06),
                v(hw, 0.52, hd + 0.02),
                frame,
                0,
            );
            k.paint(
                bank,
                &mut m,
                v(-hw + 0.04, 0.34, -hd + 0.08),
                v(hw - 0.04, 0.46, hd - 0.06),
                WHITE,
            );
            tiled_box(
                &mut m,
                v(-hw + 0.02, 0.42, -hd * 0.3),
                v(hw - 0.02, 0.5, hd - 0.04),
                quilt,
                0,
            );
            for px in if f == Furn::Bed {
                vec![0.0f32]
            } else {
                vec![-0.42, 0.42]
            } {
                k.paint(
                    bank,
                    &mut m,
                    v(px - 0.3, 0.46, -hd + 0.1),
                    v(px + 0.3, 0.6, -hd + 0.36),
                    WHITE,
                );
            }
            if f == Furn::CanopyBed {
                // Four posts holding up an open frame, a frilly valance round it and curtains
                // tied back at the corners (open on top, so the quilt still shows).
                let gold = k.solid(bank, GOLD);
                let drape = bank.add(tiles::stripes(LAVENDER, BLUSH));
                let top = 1.5;
                for (sx, sz) in [(-1.0f32, -1.0f32), (1.0, -1.0), (-1.0, 1.0), (1.0, 1.0)] {
                    let (cx, cz) = (sx * (hw - 0.03), sz * (hd - 0.02));
                    skin_box(
                        &mut m,
                        v(cx - 0.04, 0.0, cz - 0.04),
                        v(cx + 0.04, top, cz + 0.04),
                        gold,
                        &k.w4,
                    );
                    tiled_box(
                        &mut m,
                        v(cx - 0.08, 0.7, cz - 0.08),
                        v(cx + 0.08, top - 0.1, cz + 0.08),
                        drape,
                        0,
                    );
                }
                let (x0, x1, z0, z1) = (-hw - 0.05, hw + 0.05, -hd - 0.05, hd + 0.05);
                for (a, b) in [
                    (v(x0, top - 0.2, z0), v(x1, top, z0 + 0.05)),
                    (v(x0, top - 0.2, z1 - 0.05), v(x1, top, z1)),
                    (v(x0, top - 0.2, z0), v(x0 + 0.05, top, z1)),
                    (v(x1 - 0.05, top - 0.2, z0), v(x1, top, z1)),
                ] {
                    tiled_box(&mut m, a, b, drape, 0);
                    skin_box(
                        &mut m,
                        v(a.x, top, a.z),
                        v(b.x, top + 0.04, b.z),
                        gold,
                        &k.w4,
                    );
                }
            }
        }
        Furn::Stove | Furn::Range => {
            let wide = f == Furn::Range;
            let hw = if wide { 0.9 } else { 0.38 };
            let body = if wide {
                metal(bank, [GOLD, CLAY, RUST])
            } else {
                metal(bank, [SLATE, SHADOW, INK])
            };
            tiled_box(&mut m, v(-hw, 0.1, -0.36), v(hw, 0.64, 0.34), body, 0);
            k.paint(
                bank,
                &mut m,
                v(-hw - 0.02, 0.64, -0.38),
                v(hw + 0.02, 0.7, 0.36),
                SHADOW,
            );
            legs(k, bank, &mut m, hw - 0.06, 0.28, 0.1, SHADOW);
            let doors = if wide {
                vec![-0.45f32, 0.45]
            } else {
                vec![0.0]
            };
            for dx in doors {
                k.paint(
                    bank,
                    &mut m,
                    v(dx - 0.24, 0.16, 0.34),
                    v(dx + 0.24, 0.5, 0.36),
                    SHADOW,
                );
                k.paint(
                    bank,
                    &mut m,
                    v(dx - 0.16, 0.24, 0.36),
                    v(dx + 0.16, 0.4, 0.37),
                    ORANGE,
                );
                k.paint(
                    bank,
                    &mut m,
                    v(dx - 0.1, 0.44, 0.36),
                    v(dx + 0.1, 0.47, 0.4),
                    GOLD,
                );
            }
            // A pot bubbling on top.
            let pot = k.solid(bank, if wide { CRIMSON } else { RED });
            let px = if wide { -0.4 } else { 0.0 };
            lathe(
                &mut m,
                v(px, 0.7, 0.02),
                &[(0.16, 0.0), (0.18, 0.12), (0.16, 0.2)],
                8,
                0.0,
                pot,
                true,
            );
            k.paint(
                bank,
                &mut m,
                v(px - 0.12, 0.9, -0.08),
                v(px + 0.12, 0.93, 0.12),
                SHADOW,
            );
            if wide {
                // A kettle and a tiled backsplash.
                let kettle = k.solid(bank, SKY);
                lathe(
                    &mut m,
                    v(0.45, 0.7, 0.0),
                    &[(0.12, 0.0), (0.14, 0.1), (0.06, 0.2), (0.0, 0.22)],
                    8,
                    0.0,
                    kettle,
                    true,
                );
                let tile = bank.add(tiles::checker(WHITE, SKY, BLUE));
                tiled_box(&mut m, v(-hw, 0.7, -0.38), v(hw, 1.3, -0.3), tile, 0);
            } else {
                // The chimney pipe.
                let pipe = metal(bank, [SLATE, SHADOW, INK]);
                tiled_box(&mut m, v(-0.08, 0.7, -0.32), v(0.08, 1.9, -0.18), pipe, 0);
            }
        }
        Furn::Counter => {
            let panel = bank.add(tiles::boards([CLAY, RUST, MAROON], 142));
            tiled_box(&mut m, v(-0.48, 0.0, -0.38), v(0.48, 0.66, 0.34), panel, 0);
            let top = bank.add(tiles::checker(WHITE, SAND, KHAKI));
            tiled_box(&mut m, v(-0.5, 0.66, -0.4), v(0.5, 0.74, 0.38), top, 0);
            k.paint(bank, &mut m, v(-0.3, 0.74, -0.1), v(0.05, 0.77, 0.2), SAND);
            let jar = k.solid(bank, SKY);
            lathe(
                &mut m,
                v(0.25, 0.74, -0.1),
                &[(0.07, 0.0), (0.08, 0.14), (0.05, 0.18), (0.0, 0.19)],
                6,
                0.0,
                jar,
                true,
            );
            k.paint(bank, &mut m, v(-0.04, 0.4, 0.34), v(0.04, 0.46, 0.38), GOLD);
        }
        Furn::Icebox => {
            let body = metal(bank, [WHITE, SKY, BLUE]);
            tiled_box(&mut m, v(-0.36, 0.06, -0.34), v(0.36, 1.34, 0.3), body, 0);
            k.paint(bank, &mut m, v(-0.36, 0.84, 0.3), v(0.36, 0.87, 0.32), BLUE);
            for (y0, y1) in [(0.95f32, 1.2f32), (0.3, 0.7)] {
                k.paint(bank, &mut m, v(0.22, y0, 0.3), v(0.27, y1, 0.36), SLATE);
            }
            legs(k, bank, &mut m, 0.3, 0.28, 0.06, SHADOW);
        }
        Furn::RoundTable => {
            lathe(
                &mut m,
                Vec3::ZERO,
                &[(0.2, 0.0), (0.08, 0.06), (0.06, 0.5)],
                8,
                0.0,
                dark,
                false,
            );
            lathe(
                &mut m,
                Vec3::ZERO,
                &[(0.42, 0.5), (0.44, 0.58), (0.0, 0.58)],
                10,
                0.0,
                wood,
                false,
            );
            let cup = k.solid(bank, WHITE);
            lathe(
                &mut m,
                v(0.12, 0.58, 0.05),
                &[(0.05, 0.0), (0.06, 0.08)],
                6,
                0.0,
                cup,
                true,
            );
            let pot = k.solid(bank, SKY);
            lathe(
                &mut m,
                v(-0.12, 0.58, -0.05),
                &[(0.09, 0.0), (0.11, 0.08), (0.06, 0.16), (0.0, 0.18)],
                8,
                0.0,
                pot,
                true,
            );
        }
        Furn::DiningTable => {
            tiled_box(&mut m, v(-0.92, 0.5, -0.4), v(0.92, 0.58, 0.4), wood, 0);
            legs(k, bank, &mut m, 0.84, 0.32, 0.5, RUST);
            let runner = bank.add(tiles::stripes(CREAM, SALMON));
            tiled_box(&mut m, v(-0.8, 0.58, -0.14), v(0.8, 0.6, 0.14), runner, 0);
            k.paint(
                bank,
                &mut m,
                v(-0.03, 0.6, -0.03),
                v(0.03, 0.78, 0.03),
                CREAM,
            );
            for x in [-0.5f32, 0.5] {
                let plate = k.solid(bank, WHITE);
                lathe(
                    &mut m,
                    v(x, 0.6, 0.0),
                    &[(0.12, 0.0), (0.13, 0.02)],
                    8,
                    0.0,
                    plate,
                    true,
                );
            }
        }
        Furn::Chair => {
            k.paint(
                bank,
                &mut m,
                v(-0.22, 0.3, -0.22),
                v(0.22, 0.36, 0.22),
                CLAY,
            );
            legs(k, bank, &mut m, 0.18, 0.18, 0.3, RUST);
            tiled_box(&mut m, v(-0.22, 0.36, -0.24), v(0.22, 0.86, -0.18), wood, 0);
            k.paint(
                bank,
                &mut m,
                v(-0.14, 0.5, -0.18),
                v(0.14, 0.72, -0.17),
                GOLD,
            );
        }
        Furn::Armchair | Furn::Sofa => {
            let hw = if f == Furn::Sofa { 0.92 } else { 0.4 };
            let fabric = if f == Furn::Sofa {
                bank.add(tiles::plaster([SKY, BLUE, INDIGO], 143))
            } else {
                bank.add(tiles::plaster([SALMON, CRIMSON, PLUM], 144))
            };
            tiled_box(&mut m, v(-hw, 0.08, -0.36), v(hw, 0.4, 0.3), fabric, 0);
            tiled_box(&mut m, v(-hw, 0.4, -0.38), v(hw, 0.92, -0.2), fabric, 0);
            for sx in [-1.0f32, 1.0] {
                tiled_box(
                    &mut m,
                    v(sx * hw - 0.1, 0.4, -0.36),
                    v(sx * hw + 0.1 * sx.signum() * 0.0 + 0.1, 0.62, 0.3),
                    fabric,
                    0,
                );
            }
            let cushion = if f == Furn::Sofa { WHITE } else { BLUSH };
            if f == Furn::Sofa {
                for x in [-0.45f32, 0.45] {
                    k.paint(
                        bank,
                        &mut m,
                        v(x - 0.4, 0.4, -0.2),
                        v(x + 0.4, 0.5, 0.28),
                        SKY,
                    );
                }
                for x in [-0.62f32, 0.62] {
                    k.paint(
                        bank,
                        &mut m,
                        v(x - 0.12, 0.5, -0.22),
                        v(x + 0.12, 0.72, -0.14),
                        GOLD,
                    );
                }
            } else {
                k.paint(bank, &mut m, v(-0.3, 0.4, -0.2), v(0.3, 0.5, 0.28), cushion);
            }
            legs(k, bank, &mut m, hw - 0.06, 0.28, 0.08, RUST);
        }
        Furn::Bookshelf => {
            tiled_box(&mut m, v(-0.46, 0.0, -0.44), v(0.46, 1.62, -0.4), dark, 0);
            for x in [-0.46f32, 0.4] {
                tiled_box(&mut m, v(x, 0.0, -0.44), v(x + 0.06, 1.62, -0.04), dark, 0);
            }
            for y in [0.0f32, 0.52, 1.04, 1.56] {
                tiled_box(
                    &mut m,
                    v(-0.46, y, -0.44),
                    v(0.46, y + 0.06, -0.04),
                    wood,
                    0,
                );
            }
            let spines = [
                RED, BLUE, GREEN, GOLD, PURPLE, TEAL, CRIMSON, INDIGO, SALMON,
            ];
            for (row, y) in [0.06f32, 0.58, 1.1].into_iter().enumerate() {
                let mut x = -0.4;
                let mut i = row * 3;
                while x < 0.36 {
                    let wdt = 0.07 + (i % 3) as f32 * 0.015;
                    let tall = 0.3 + (i % 4) as f32 * 0.04;
                    k.paint(
                        bank,
                        &mut m,
                        v(x, y, -0.4),
                        v(x + wdt, y + tall, -0.1),
                        spines[i % spines.len()],
                    );
                    x += wdt + 0.01;
                    i += 1;
                }
            }
        }
        Furn::Wardrobe => {
            tiled_box(&mut m, v(-0.44, 0.08, -0.36), v(0.44, 1.72, 0.3), dark, 0);
            tiled_box(&mut m, v(-0.48, 1.72, -0.4), v(0.48, 1.82, 0.34), wood, 0);
            for sx in [-1.0f32, 1.0] {
                tiled_box(
                    &mut m,
                    v(sx * 0.22 - 0.19, 0.16, 0.3),
                    v(sx * 0.22 + 0.19, 1.64, 0.33),
                    wood,
                    0,
                );
                k.paint(
                    bank,
                    &mut m,
                    v(sx * 0.05 - 0.02, 0.84, 0.33),
                    v(sx * 0.05 + 0.02, 0.96, 0.36),
                    GOLD,
                );
            }
            legs(k, bank, &mut m, 0.38, 0.26, 0.08, MAROON);
        }
        Furn::Dresser => {
            tiled_box(&mut m, v(-0.44, 0.06, -0.32), v(0.44, 0.7, 0.3), wood, 0);
            for (i, y) in [0.12f32, 0.32, 0.52].into_iter().enumerate() {
                k.paint(
                    bank,
                    &mut m,
                    v(-0.4, y, 0.3),
                    v(0.4, y + 0.16, 0.32),
                    if i % 2 == 0 { CLAY } else { GOLD },
                );
                k.paint(
                    bank,
                    &mut m,
                    v(-0.04, y + 0.06, 0.32),
                    v(0.04, y + 0.1, 0.35),
                    CREAM,
                );
            }
            // A little round mirror.
            k.paint(
                bank,
                &mut m,
                v(-0.03, 0.7, -0.2),
                v(0.03, 0.82, -0.16),
                RUST,
            );
            let glass = bank.add(tiles::crystal([WHITE, SKY, AQUA]));
            let mut mirror = Mesh::new();
            lathe(
                &mut mirror,
                Vec3::ZERO,
                &[(0.18, -0.02), (0.18, 0.02)],
                10,
                0.0,
                glass,
                true,
            );
            m.append(
                &mirror,
                Mat4::from_translation(v(0.0, 1.0, -0.18)) * Mat4::from_rotation_x(FRAC_PI_2),
            );
        }
        Furn::FloorLamp => {
            lathe(
                &mut m,
                Vec3::ZERO,
                &[(0.2, 0.0), (0.18, 0.05), (0.03, 0.08)],
                8,
                0.0,
                dark,
                true,
            );
            k.paint(
                bank,
                &mut m,
                v(-0.025, 0.05, -0.025),
                v(0.025, 1.2, 0.025),
                GOLD,
            );
        }
        Furn::Candelabra => {
            let brass = metal(bank, [CREAM, GOLD, CLAY]);
            lathe(
                &mut m,
                Vec3::ZERO,
                &[(0.18, 0.0), (0.14, 0.06), (0.04, 0.1)],
                8,
                0.0,
                brass,
                true,
            );
            skin_box(
                &mut m,
                v(-0.03, 0.08, -0.03),
                v(0.03, 0.8, 0.03),
                brass,
                &k.w4,
            );
            skin_box(
                &mut m,
                v(-0.24, 0.78, -0.03),
                v(0.24, 0.84, 0.03),
                brass,
                &k.w4,
            );
            for (x, h) in [(-0.22f32, 0.22f32), (0.0, 0.3), (0.22, 0.22)] {
                skin_box(
                    &mut m,
                    v(x - 0.05, 0.84, -0.05),
                    v(x + 0.05, 0.88, 0.05),
                    brass,
                    &k.w4,
                );
                k.paint(
                    bank,
                    &mut m,
                    v(x - 0.03, 0.88, -0.03),
                    v(x + 0.03, 0.88 + h, 0.03),
                    CREAM,
                );
            }
        }
        Furn::Fireplace => {
            let stone = bank.add(tiles::bricks(KHAKI, SAND, SHADOW, 145));
            for sx in [-1.0f32, 1.0] {
                tiled_box(
                    &mut m,
                    v(sx * 0.7 - 0.24, 0.0, -0.46),
                    v(sx * 0.7 + 0.24, 1.2, -0.06),
                    stone,
                    0,
                );
            }
            tiled_box(&mut m, v(-0.96, 0.9, -0.46), v(0.96, 1.2, -0.02), stone, 0);
            tiled_box(&mut m, v(-1.0, 1.2, -0.48), v(1.0, 1.3, 0.06), dark, 0);
            k.paint(bank, &mut m, v(-0.46, 0.0, -0.46), v(0.46, 0.9, -0.42), INK);
            tiled_box(&mut m, v(-0.5, 0.0, -0.1), v(0.5, 0.06, 0.2), stone, 0);
            // Logs.
            let log = bank.add(tiles::bark());
            for (x, a) in [(-0.12f32, 0.5f32), (0.12, -0.5)] {
                let mut l = Mesh::new();
                lathe(
                    &mut l,
                    Vec3::ZERO,
                    &[(0.06, -0.22), (0.06, 0.22)],
                    6,
                    0.0,
                    log,
                    true,
                );
                m.append(
                    &l,
                    Mat4::from_translation(v(x, 0.08, -0.24))
                        * Mat4::from_rotation_y(a)
                        * Mat4::from_rotation_x(FRAC_PI_2),
                );
            }
            // A few things on the mantel.
            k.paint(bank, &mut m, v(-0.7, 1.3, -0.3), v(-0.6, 1.5, -0.2), CREAM);
            k.paint(
                bank,
                &mut m,
                v(0.55, 1.3, -0.32),
                v(0.75, 1.45, -0.26),
                GOLD,
            );
        }
        Furn::Fern | Furn::Cactus => {
            let pot = k.solid(bank, if f == Furn::Fern { CLAY } else { SALMON });
            lathe(
                &mut m,
                Vec3::ZERO,
                &[(0.14, 0.0), (0.2, 0.3), (0.0, 0.3)],
                8,
                0.0,
                pot,
                false,
            );
            if f == Furn::Fern {
                let leaves = bank.add(tiles::canopy([LIME, GREEN, TEAL, DEEP_TEAL], 146));
                lathe(
                    &mut m,
                    Vec3::ZERO,
                    &[
                        (0.0, 0.3),
                        (0.34, 0.46),
                        (0.4, 0.66),
                        (0.24, 0.86),
                        (0.0, 0.9),
                    ],
                    7,
                    0.4,
                    leaves,
                    false,
                );
            } else {
                let green = bank.add(tiles::stripes(GREEN, LIME));
                lathe(
                    &mut m,
                    Vec3::ZERO,
                    &[(0.1, 0.3), (0.11, 0.8), (0.07, 0.92), (0.0, 0.95)],
                    6,
                    0.0,
                    green,
                    false,
                );
                for (sx, y) in [(-1.0f32, 0.5f32), (1.0, 0.62)] {
                    skin_box(
                        &mut m,
                        v(sx * 0.1, y, -0.04),
                        v(sx * 0.24, y + 0.07, 0.04),
                        green,
                        &k.w4,
                    );
                    skin_box(
                        &mut m,
                        v(sx * 0.18 - 0.04, y, -0.04),
                        v(sx * 0.18 + 0.04, y + 0.22, 0.04),
                        green,
                        &k.w4,
                    );
                }
                k.paint(
                    bank,
                    &mut m,
                    v(-0.04, 0.95, -0.04),
                    v(0.04, 1.02, 0.04),
                    PINK,
                );
            }
        }
        Furn::Vase => {
            tiled_box(&mut m, v(-0.26, 0.5, -0.26), v(0.26, 0.56, 0.26), wood, 0);
            legs(k, bank, &mut m, 0.2, 0.2, 0.5, RUST);
            let vase = metal(bank, [SKY, BLUE, INDIGO]);
            lathe(
                &mut m,
                v(0.0, 0.56, 0.0),
                &[(0.08, 0.0), (0.12, 0.12), (0.06, 0.24), (0.07, 0.3)],
                8,
                0.0,
                vase,
                true,
            );
            for (x, z, c) in [
                (-0.08f32, 0.0f32, PINK),
                (0.07, 0.04, GOLD),
                (0.0, -0.07, WHITE),
                (0.04, 0.08, SALMON),
            ] {
                k.paint(
                    bank,
                    &mut m,
                    v(x * 0.4 - 0.01, 0.86, z * 0.4 - 0.01),
                    v(x + 0.01, 1.0, z + 0.01),
                    GREEN,
                );
                k.paint(
                    bank,
                    &mut m,
                    v(x - 0.04, 1.0, z - 0.04),
                    v(x + 0.04, 1.07, z + 0.04),
                    c,
                );
            }
        }
        Furn::Clock => {
            tiled_box(&mut m, v(-0.24, 0.0, -0.22), v(0.24, 1.92, 0.14), dark, 0);
            tiled_box(&mut m, v(-0.28, 1.92, -0.26), v(0.28, 2.02, 0.18), wood, 0);
            // The face.
            let face = k.solid(bank, CREAM);
            let mut dial = Mesh::new();
            lathe(
                &mut dial,
                Vec3::ZERO,
                &[(0.16, -0.01), (0.16, 0.01)],
                10,
                0.0,
                face,
                true,
            );
            m.append(
                &dial,
                Mat4::from_translation(v(0.0, 1.58, 0.15)) * Mat4::from_rotation_x(FRAC_PI_2),
            );
            k.paint(bank, &mut m, v(-0.01, 1.58, 0.16), v(0.01, 1.7, 0.17), INK);
            k.paint(bank, &mut m, v(0.0, 1.57, 0.16), v(0.09, 1.59, 0.17), INK);
            // A dark window where the pendulum swings.
            k.paint(
                bank,
                &mut m,
                v(-0.14, 0.4, 0.14),
                v(0.14, 1.3, 0.15),
                SHADOW,
            );
        }
        Furn::Piano => {
            let lacquer = bank.add(tiles::planks(MAROON, RUST, INK, 147));
            tiled_box(&mut m, v(-0.92, 0.0, -0.4), v(0.92, 1.24, 0.02), lacquer, 0);
            tiled_box(&mut m, v(-0.9, 0.56, 0.02), v(0.9, 0.66, 0.34), lacquer, 0);
            let keys = bank.add(piano_keys());
            m.quad(
                [
                    v(-0.84, 0.665, 0.32),
                    v(0.84, 0.665, 0.32),
                    v(0.84, 0.665, 0.04),
                    v(-0.84, 0.665, 0.04),
                ],
                UvRect::new(0.0, 0.0, 32.0, 4.0),
                keys,
            );
            k.paint(
                bank,
                &mut m,
                v(-0.24, 0.9, 0.02),
                v(0.24, 1.14, 0.05),
                WHITE,
            );
            for x in [-0.8f32, 0.8] {
                k.paint(
                    bank,
                    &mut m,
                    v(x - 0.06, 0.0, 0.2),
                    v(x + 0.06, 0.56, 0.3),
                    MAROON,
                );
            }
            k.paint(
                bank,
                &mut m,
                v(0.6, 1.24, -0.2),
                v(0.66, 1.42, -0.14),
                CREAM,
            );
        }
        Furn::Globe => {
            lathe(
                &mut m,
                Vec3::ZERO,
                &[(0.18, 0.0), (0.12, 0.06), (0.04, 0.12), (0.04, 0.4)],
                8,
                0.0,
                dark,
                true,
            );
            let brass = metal(bank, [CREAM, GOLD, CLAY]);
            for k2 in 0..9 {
                let a = -0.3 + k2 as f32 / 8.0 * 3.8;
                let p = v(0.0, 0.66, 0.0) + v(a.cos() * 0.29, a.sin() * 0.29, 0.0);
                skin_box(
                    &mut m,
                    p - Vec3::splat(0.022),
                    p + Vec3::splat(0.022),
                    brass,
                    &k.w4,
                );
            }
        }
        Furn::Telescope => {
            let brass = metal(bank, [CREAM, GOLD, CLAY]);
            for k2 in 0..3 {
                let a = k2 as f32 / 3.0 * TAU;
                let mut leg = Mesh::new();
                k.paint(
                    bank,
                    &mut leg,
                    v(-0.025, 0.0, -0.025),
                    v(0.025, 0.8, 0.025),
                    RUST,
                );
                m.append(
                    &leg,
                    Mat4::from_translation(v(a.cos() * 0.22, 0.0, a.sin() * 0.22))
                        * Mat4::from_rotation_z(a.cos() * 0.25)
                        * Mat4::from_rotation_x(-a.sin() * 0.25),
                );
            }
            let mut tube = Mesh::new();
            lathe(
                &mut tube,
                Vec3::ZERO,
                &[(0.07, -0.4), (0.08, 0.1), (0.1, 0.4)],
                8,
                0.0,
                brass,
                true,
            );
            m.append(
                &tube,
                Mat4::from_translation(v(0.0, 0.92, 0.0)) * Mat4::from_rotation_x(-0.8),
            );
        }
        Furn::Plush => {
            let fluff = k.solid(bank, WHITE);
            lathe(
                &mut m,
                Vec3::ZERO,
                &[
                    (0.0, 0.0),
                    (0.16, 0.02),
                    (0.18, 0.14),
                    (0.12, 0.24),
                    (0.0, 0.26),
                ],
                8,
                0.0,
                fluff,
                true,
            );
            lathe(
                &mut m,
                v(0.0, 0.24, 0.02),
                &[(0.0, 0.0), (0.12, 0.04), (0.12, 0.14), (0.0, 0.2)],
                8,
                0.0,
                fluff,
                true,
            );
            for sx in [-1.0f32, 1.0] {
                k.paint(
                    bank,
                    &mut m,
                    v(sx * 0.05 - 0.03, 0.4, -0.02),
                    v(sx * 0.05 + 0.03, 0.62, 0.03),
                    WHITE,
                );
                k.paint(
                    bank,
                    &mut m,
                    v(sx * 0.05 - 0.015, 0.44, 0.03),
                    v(sx * 0.05 + 0.015, 0.58, 0.035),
                    BLUSH,
                );
                k.paint(
                    bank,
                    &mut m,
                    v(sx * 0.045 - 0.015, 0.32, 0.12),
                    v(sx * 0.045 + 0.015, 0.35, 0.14),
                    INK,
                );
            }
            k.paint(
                bank,
                &mut m,
                v(-0.02, 0.29, 0.13),
                v(0.02, 0.31, 0.15),
                PINK,
            );
        }
        Furn::FishBowl => {
            tiled_box(&mut m, v(-0.28, 0.46, -0.28), v(0.28, 0.52, 0.28), wood, 0);
            legs(k, bank, &mut m, 0.22, 0.22, 0.46, RUST);
            let pebbles = bank.add(tiles::plaster([SAND, KHAKI, PINK], 148));
            lathe(
                &mut m,
                v(0.0, 0.52, 0.0),
                &[(0.14, 0.0), (0.18, 0.05), (0.0, 0.05)],
                8,
                0.0,
                pebbles,
                false,
            );
            let weed = k.solid(bank, GREEN);
            skin_box(
                &mut m,
                v(0.06, 0.55, -0.04),
                v(0.09, 0.72, -0.01),
                weed,
                &k.w4,
            );
        }
        Furn::FishTank => {
            tiled_box(&mut m, v(-0.92, 0.0, -0.34), v(0.92, 0.5, 0.3), dark, 0);
            for x in [-0.46f32, 0.46] {
                k.paint(
                    bank,
                    &mut m,
                    v(x - 0.4, 0.08, 0.3),
                    v(x + 0.4, 0.42, 0.32),
                    RUST,
                );
                k.paint(
                    bank,
                    &mut m,
                    v(x - 0.04, 0.22, 0.32),
                    v(x + 0.04, 0.28, 0.35),
                    GOLD,
                );
            }
            let sand = bank.add(tiles::plaster([CREAM, SAND, KHAKI], 149));
            tiled_box(&mut m, v(-0.88, 0.5, -0.3), v(0.88, 0.58, 0.26), sand, 0);
            // Weeds, a pebble and a tiny sunken chest.
            let weed = bank.add(tiles::stripes(GREEN, LIME));
            for (x, h) in [(-0.7f32, 0.36f32), (-0.6, 0.26), (0.62, 0.42), (0.72, 0.3)] {
                skin_box(
                    &mut m,
                    v(x - 0.025, 0.58, -0.18),
                    v(x + 0.025, 0.58 + h, -0.13),
                    weed,
                    &k.w4,
                );
            }
            k.paint(bank, &mut m, v(0.1, 0.58, -0.12), v(0.3, 0.7, 0.02), RUST);
            k.paint(bank, &mut m, v(0.1, 0.7, -0.12), v(0.3, 0.73, 0.02), GOLD);
            k.paint(bank, &mut m, v(-0.3, 0.58, 0.0), v(-0.16, 0.64, 0.1), SLATE);
            // A wooden rim round the open top, and a little lamp at the back.
            for (a, b) in [
                (v(-0.92, 1.22, -0.34), v(0.92, 1.28, -0.28)),
                (v(-0.92, 1.22, 0.24), v(0.92, 1.28, 0.3)),
                (v(-0.92, 1.22, -0.34), v(-0.86, 1.28, 0.3)),
                (v(0.86, 1.22, -0.34), v(0.92, 1.28, 0.3)),
            ] {
                tiled_box(&mut m, a, b, dark, 0);
            }
            k.paint(
                bank,
                &mut m,
                v(-0.2, 1.22, -0.34),
                v(0.2, 1.32, -0.24),
                SHADOW,
            );
        }
    }
    m
}

/// A pane of glass: clear, with bright edges and a diagonal glint.
fn glass_pane() -> Texture {
    let mut t = Texture::clear(32, 32);
    for i in 0..32 {
        t.set(i, 0, WHITE);
        t.set(i, 31, SKY);
        t.set(0, i, SKY);
        t.set(31, i, WHITE);
    }
    for i in 4..12 {
        t.set(i + 2, 20 - i, WHITE);
        t.set(i + 3, 20 - i, WHITE);
    }
    t
}

/// Mostly clear, with a few glints (the fish bowl).
fn glints() -> Texture {
    let mut t = Texture::clear(16, 16);
    for (x, y) in [(3, 3), (4, 3), (3, 4), (11, 9), (12, 10), (7, 13)] {
        t.set(x, y, WHITE);
    }
    t
}

/// Still, clear water with a few ripples of light.
fn calm_water() -> Texture {
    let mut t = Texture::new(16, 16, AQUA);
    for y in 0..16 {
        for x in 0..16 {
            if (x + y * 3) % 11 == 0 {
                t.set(x, y, SKY);
            }
        }
    }
    for (x, y) in [(2, 3), (3, 3), (9, 7), (10, 7), (5, 12), (13, 13)] {
        t.set(x, y, WHITE);
    }
    t
}

/// White keys and black ones.
fn piano_keys() -> Texture {
    let mut t = Texture::new(32, 4, WHITE);
    for x in 0..32 {
        if x % 2 == 1 {
            t.set(x, 3, SAND);
        }
        let b = x % 7;
        if [1, 2, 4, 5, 6].contains(&b) && x % 2 == 0 {
            t.set(x, 0, INK);
            t.set(x, 1, INK);
        }
    }
    t
}

/// A rug: a border, a woven middle, round or stitched with a star or a fish.
fn rug_tex(kind: usize, pal: [u8; 3]) -> Texture {
    let [base, pattern, edge] = pal;
    let mut t = Texture::clear(16, 16);
    for y in 0..16i32 {
        for x in 0..16i32 {
            let (dx, dy) = (x as f32 - 7.5, y as f32 - 7.5);
            let r = (dx * dx + dy * dy).sqrt();
            let c = match kind {
                0 => {
                    if r > 7.8 {
                        continue;
                    } else if r > 6.6 {
                        edge
                    } else if (r as i32) % 3 == 0 {
                        pattern
                    } else {
                        base
                    }
                }
                1 => {
                    if x == 0 || x == 15 || y == 0 || y == 15 {
                        edge
                    } else if (y / 2) % 2 == 0 {
                        pattern
                    } else {
                        base
                    }
                }
                2 => {
                    let star = (dx.abs() + dy.abs() * 0.6 < 4.0 && dy.abs() < 1.5)
                        || (dy.abs() + dx.abs() * 0.6 < 4.0 && dx.abs() < 1.5);
                    if x == 0 || x == 15 || y == 0 || y == 15 {
                        edge
                    } else if star || (x * 7 + y * 3) % 17 == 0 {
                        pattern
                    } else {
                        base
                    }
                }
                _ => {
                    // A happy flat fish.
                    let body = (dx / 6.0).powi(2) + (dy / 4.0).powi(2) <= 1.0 && dx > -5.0;
                    let tail = dx <= -5.0 && dx > -7.6 && dy.abs() <= (-5.0 - dx) * 1.3 + 0.5;
                    if !(body || tail) {
                        continue;
                    } else if (dx - 3.0).abs() < 0.8 && (dy + 1.0).abs() < 0.8 {
                        INK
                    } else if tail || (x % 3 == 0 && body) {
                        pattern
                    } else if (dx / 6.0).powi(2) + (dy / 4.0).powi(2) > 0.7 {
                        edge
                    } else {
                        base
                    }
                }
            };
            t.set(x, y, c);
        }
    }
    t
}

/// A little landscape, or a starry night, for a painting.
fn painting_tex(kind: usize) -> Texture {
    let mut t = Texture::new(16, 16, SKY);
    match kind {
        0 | 1 => {
            let (sky, hill, sun) = if kind == 0 {
                (SKY, GREEN, GOLD)
            } else {
                (PEACH, PLUM, GOLD)
            };
            for y in 0..16 {
                for x in 0..16 {
                    t.set(x, y, if kind == 1 && y < 5 { SALMON } else { sky });
                }
            }
            for x in 0..16 {
                let h = 10 + ((x as f32 * 0.6).sin() * 2.0) as i32;
                for y in h..16 {
                    t.set(
                        x,
                        y,
                        if y == h {
                            if kind == 0 { LIME } else { CRIMSON }
                        } else {
                            hill
                        },
                    );
                }
            }
            for (x, y) in [(11, 3), (12, 3), (11, 4), (12, 4), (10, 4), (13, 4)] {
                t.set(x, y, sun);
            }
        }
        _ => {
            for y in 0..16 {
                for x in 0..16 {
                    let swirl = ((x as f32 * 0.8 + (y as f32 * 0.5).sin() * 2.0) as i32) % 4 == 0;
                    t.set(x, y, if swirl { BLUE } else { INDIGO });
                }
            }
            for (x, y) in [(3, 3), (9, 2), (13, 5), (6, 7), (11, 9)] {
                t.set(x, y, GOLD);
                t.set(x + 1, y, CREAM);
            }
            for x in 0..16 {
                for y in 12..16 {
                    t.set(x, y, if (x + y) % 3 == 0 { TEAL } else { DEEP_TEAL });
                }
            }
        }
    }
    t
}

/// A picture for the back wall: a gold frame around a painting, or a fish on a plaque.
fn wall_art(bank: &mut TexBank, k: &mut Kit, kind: usize, fish: TexId) -> Mesh {
    let mut m = Mesh::new();
    let front = |m: &mut Mesh, x0: f32, y0: f32, x1: f32, y1: f32, z: f32, t: TexId| {
        m.quad(
            [v(x0, y0, z), v(x1, y0, z), v(x1, y1, z), v(x0, y1, z)],
            UvRect::new(0.0, 0.0, 16.0, 16.0),
            t,
        );
    };
    if kind == 3 {
        let plaque = bank.add(tiles::planks(RUST, CLAY, MAROON, 150));
        skin_box(
            &mut m,
            v(-0.36, 0.95, -0.02),
            v(0.36, 1.45, 0.03),
            plaque,
            &Texture::new(16, 16, 0),
        );
        k.paint(bank, &mut m, v(-0.1, 0.98, 0.03), v(0.1, 1.02, 0.04), GOLD);
        front(&mut m, -0.3, 1.02, 0.3, 1.42, 0.035, fish);
    } else {
        let gold = k.solid(bank, GOLD);
        skin_box(
            &mut m,
            v(-0.34, 0.92, -0.02),
            v(0.34, 1.48, 0.02),
            gold,
            &k.w4,
        );
        let canvas = bank.add(painting_tex(kind));
        front(&mut m, -0.29, 0.97, 0.29, 1.43, 0.025, canvas);
    }
    m
}

const BED: &[&str] = &[
    "................",
    "................",
    "..KKKK..........",
    ".KCnnCK.........",
    ".KCnnCKKKKKKKKK.",
    ".KCwwwwKPbPbPbPK",
    ".KCwwwwKbPbPbPbK",
    ".KCKKKKKPbPbPbPK",
    ".KCKPsPsPsPsPsPK",
    ".KCKsPsPsPsPsPsK",
    ".KCKPsPsPsPsPsPK",
    ".KuKKKKKKKKKKKKK",
    ".KuCCCCCCCCCCCuK",
    ".KuKKKKKKKKKKKuK",
    ".KKK.........KKK",
    "................",
];

const CANOPY_BED: &[&str] = &[
    "KKKKKKKKKKKKKKKK",
    "KVLVLVLVLVLVLVLK",
    "KKKKKKKKKKKKKKKK",
    "KYKL........LKYK",
    "KYKL.KKKKKK.LKYK",
    "KYKL.KwwwwK.LKYK",
    "KYKLKKKKKKKKLKYK",
    "KYKKLLLLLLLLKKYK",
    "KYKLVLVLVLVLVKYK",
    "KYKVLVLVLVLVLKYK",
    "KYKLVLVLVLVLVKYK",
    "KYKKKKKKKKKKKKYK",
    "KYYYYYYYYYYYYYYK",
    "KuKKKKKKKKKKKKuK",
    "KKK..........KKK",
    "................",
];

const STOVE: &[&str] = &[
    "...........KK...",
    "...........KkK..",
    "...KKKK....KkK..",
    "..KrrrrK...KkK..",
    ".KKrccrKK..KkK..",
    ".KDDDDDDDDDDkDK.",
    ".KDkkkkkkkkkkDK.",
    ".KDKKKKKKKKKKDK.",
    ".KDKoYooYoooKDK.",
    ".KDKooYoooYoKDK.",
    ".KDKKKKKKKKKKDK.",
    ".KDDDDDDDDDDDDK.",
    ".KkkkkkkkkkkkkK.",
    "..KkK......KkK..",
    "..KKK......KKK..",
    "................",
];

const RANGE: &[&str] = &[
    "................",
    "................",
    "KKKKKKKKKKKKKKKK",
    "KwSwSwSwSwSwSwSK",
    "KKKKKKKKKKKKKKKK",
    "KYkkYYYYYYYYkkYK",
    "KYYYYYYYYYYYYYYK",
    "KCKKKKKCCKKKKKCK",
    "KCKooooKKooooKCK",
    "KCKoYoYKKoYoYKCK",
    "KCKKKKKCCKKKKKCK",
    "KCCCCCCCCCCCCCCK",
    "KuuuuuuuuuuuuuuK",
    "KKuKKKKKKKKKKuKK",
    ".KKK........KKK.",
    "................",
];

const COUNTER: &[&str] = &[
    "................",
    "................",
    "................",
    "......KK........",
    ".....KgK..KKK...",
    "....KKgKKKSSK...",
    "KKKKKKKKKKKKKKKK",
    "KwwwwwwwwwwwwwwK",
    "KnnnnnnnnnnnnnnK",
    "KKKKKKKKKKKKKKKK",
    "KCuKCCCCCCKCCCuK",
    "KCuKCCYCCCKCYCuK",
    "KCuKCCCCCCKCCCuK",
    "KCuKCCCCCCKCCCuK",
    "KKKKKKKKKKKKKKKK",
    "................",
];

const ICEBOX: &[&str] = &[
    "....KKKKKKKK....",
    "...KwwwwwwwwK...",
    "...KwSSSSSSwK...",
    "...KwSwwwwSwK...",
    "...KwSwwwDSwK...",
    "...KwSwwwDSwK...",
    "...KwSSSSSSwK...",
    "...KKBBBBBBKK...",
    "...KwSSSSSSwK...",
    "...KwSwwwDSwK...",
    "...KwSwwwDSwK...",
    "...KwSwwwwSwK...",
    "...KwSSSSSSwK...",
    "...KKKKKKKKKK...",
    "....KDK..KDK....",
    "....KKK..KKK....",
];

const ROUND_TABLE: &[&str] = &[
    "................",
    "................",
    "................",
    ".......KKK......",
    "..KK..KSSSK.....",
    ".KwwK.KSSSK.....",
    "..KKKKKKKKKKK...",
    ".KnnnnnnnnnnnK..",
    "KnnnnnnnnnnnnnK.",
    ".KCCCCCCCCCCCK..",
    "..KKKKKuKKKKK...",
    "......KuK.......",
    "......KuK.......",
    ".....KuuuK......",
    "....KKKKKKK.....",
    "................",
];

const DINING_TABLE: &[&str] = &[
    "................",
    "................",
    "................",
    "................",
    ".......KK.......",
    ".......KyK......",
    "..KKKKKKYKKKKK..",
    ".KnnnnnnnnnnnnK.",
    "KnnssssssssssnnK",
    "KCCCCCCCCCCCCCCK",
    "KuKKKKKKKKKKKKuK",
    "KuK..........KuK",
    "KuK..........KuK",
    "KuK..........KuK",
    "KKK..........KKK",
    "................",
];

const CHAIR: &[&str] = &[
    "................",
    "....KKKKKKK.....",
    "....KCCCCCK.....",
    "....KCKYKCK.....",
    "....KCKYKCK.....",
    "....KCKYKCK.....",
    "....KCCCCCK.....",
    "....KCK.KCK.....",
    "...KKKKKKKKK....",
    "...KnnnnnnnK....",
    "...KCCCCCCCK....",
    "...KuKKKKKuK....",
    "...KuK...KuK....",
    "...KuK...KuK....",
    "...KKK...KKK....",
    "................",
];

const ARMCHAIR: &[&str] = &[
    "................",
    "................",
    "....KKKKKKKK....",
    "...KssssssssK...",
    "...KsccccccsK...",
    "...KsccccccsK...",
    ".KKKsccccccsKKK.",
    ".KssKKKKKKKKssK.",
    ".KscKbbbbbbKcsK.",
    ".KscKbbbbbbKcsK.",
    ".KscKKKKKKKKcsK.",
    ".KccccccccccccK.",
    ".KppppppppppppK.",
    ".KKuKKKKKKKKuKK.",
    "..KKK......KKK..",
    "................",
];

const SOFA: &[&str] = &[
    "................",
    "................",
    "................",
    "..KKKKKKKKKKKK..",
    ".KSSSSSSSSSSSSK.",
    ".KSBBBBKKBBBBSK.",
    "KKSBBBBKKBBBBSKK",
    "KSKYYKSKKSKYYKSK",
    "KBKYYKSSSSKYYKBK",
    "KBKKKKKKKKKKKKBK",
    "KBKSSSSSSSSSSKBK",
    "KBBBBBBBBBBBBBBK",
    "KIIIIIIIIIIIIIIK",
    "KKuKKKKKKKKKKuKK",
    ".KKK........KKK.",
    "................",
];

const BOOKSHELF: &[&str] = &[
    "..KKKKKKKKKKKK..",
    "..KuuuuuuuuuuK..",
    "..KuKKKKKKKKuK..",
    "..KuKrBgYVcKuK..",
    "..KuKrBgYVcKuK..",
    "..KuKrBgYVcKuK..",
    "..KuKKKKKKKKuK..",
    "..KuKtYrKBgKuK..",
    "..KuKtYrKBgKuK..",
    "..KuKtYrKBgKuK..",
    "..KuKKKKKKKKuK..",
    "..KuKgcBVYrKuK..",
    "..KuKgcBVYrKuK..",
    "..KuKKKKKKKKuK..",
    "..KuuuuuuuuuuK..",
    "..KKKKKKKKKKKK..",
];

const WARDROBE: &[&str] = &[
    "..KKKKKKKKKKKK..",
    ".KCCCCCCCCCCCCK.",
    ".KKKKKKKKKKKKKK.",
    ".KuuuuuKKuuuuuK.",
    ".KuCCCuKKuCCCuK.",
    ".KuCuCuKKuCuCuK.",
    ".KuCCCuKKuCCCuK.",
    ".KuuuuYKKYuuuuK.",
    ".KuCCCuKKuCCCuK.",
    ".KuCuCuKKuCuCuK.",
    ".KuCCCuKKuCCCuK.",
    ".KuuuuuKKuuuuuK.",
    ".KKKKKKKKKKKKKK.",
    ".KmmmmmmmmmmmmK.",
    ".KmKK......KKmK.",
    ".KKK........KKK.",
];

const DRESSER: &[&str] = &[
    "................",
    "......KKKK......",
    ".....KSSSSK.....",
    ".....KSwSSK.....",
    "......KSSK......",
    ".......KK.......",
    "..KKKKKKKKKKKK..",
    ".KCCCCCCCCCCCCK.",
    ".KYYYYYyyYYYYYK.",
    ".KKKKKKKKKKKKKK.",
    ".KCCCCCyyCCCCCK.",
    ".KKKKKKKKKKKKKK.",
    ".KYYYYYyyYYYYYK.",
    ".KuKKKKKKKKKKuK.",
    ".KKK........KKK.",
    "................",
];

const FLOOR_LAMP: &[&str] = &[
    ".....KKKKKK.....",
    "....KyyyyyyK....",
    "...KyyYyyYyyK...",
    "..KYYYYYYYYYYK..",
    "..KKKKKKKKKKKK..",
    "......KyyK......",
    ".......KK.......",
    ".......KYK......",
    ".......KYK......",
    ".......KYK......",
    ".......KYK......",
    ".......KYK......",
    ".......KYK......",
    ".....KKuuKK.....",
    "....KuuuuuuK....",
    "....KKKKKKKK....",
];

const CANDELABRA: &[&str] = &[
    "..y....y....y...",
    ".KoK..KoK..KoK..",
    ".KwK..KwK..KwK..",
    ".KwK..KwK..KwK..",
    ".KYKKKKYKKKKYK..",
    "..KYYYYYYYYYK...",
    "...KKKKYKKKK....",
    "......KYK.......",
    "......KYK.......",
    "......KYK.......",
    "......KYK.......",
    ".....KYYYK......",
    "....KYYYYYK.....",
    "....KKKKKKK.....",
    "................",
    "................",
];

const FIREPLACE: &[&str] = &[
    "KKKKKKKKKKKKKKKK",
    "KuuuuuuuuuuuuuuK",
    "KKKKKKKKKKKKKKKK",
    ".KnhnhnhnhnhnhK.",
    ".KhnKKKKKKKKnhK.",
    ".KnhK......KhnK.",
    ".KhnK..Y...KnhK.",
    ".KnhK.YoY..KhnK.",
    ".KhnK.YoYo.KnhK.",
    ".KnhKYoroYoKhnK.",
    ".KhnKorrrooKnhK.",
    ".KnhKuCuCuCKhnK.",
    ".KhnKKKKKKKKnhK.",
    ".KnhnhnhnhnhnhK.",
    ".KKKKKKKKKKKKKK.",
    "................",
];

const FERN: &[&str] = &[
    "......l.........",
    "...l..gl..l.....",
    "..lgl.gg.lgl....",
    "...lgggglg..l...",
    ".l..gtggtg.lgl..",
    ".lgl.ggtgggg.l..",
    "..lggtggggtgg...",
    "...gtggtggtg....",
    ".....gggtgg.....",
    "....KKKKKKKK....",
    "....KCuCCCuK....",
    "....KCCCCCCK....",
    ".....KCCuCK.....",
    ".....KCCCCK.....",
    ".....KKKKKK.....",
    "................",
];

const CACTUS: &[&str] = &[
    "................",
    "......KPK.......",
    ".....KPbPK......",
    "......KgK.......",
    ".....KglgK..KK..",
    ".KK..KgggK.KgK..",
    ".KgK.KglgK.KgK..",
    ".KgKKKgggKKKgK..",
    ".KgggggglgggK...",
    "..KKKKKglgKK....",
    ".....KgggK......",
    "....KKKKKKK.....",
    "....KsssssK.....",
    "....KscsscK.....",
    ".....KsssK......",
    ".....KKKKK......",
];

const VASE: &[&str] = &[
    "....P..Y..P.....",
    "...PbP.g.PbP....",
    "....P.KgK.P.....",
    "...g.K.g.K.g....",
    "....gKgggKg.....",
    ".....KBBBK......",
    "....KBSSSBK.....",
    "....KBSBBBK.....",
    "....KBBBBBK.....",
    ".....KBBBK......",
    "...KKKKKKKKK....",
    "...KnnnnnnnK....",
    "....KuKKKuK.....",
    "....KuK.KuK.....",
    "....KKK.KKK.....",
    "................",
];

const CLOCK: &[&str] = &[
    ".....KKKKKK.....",
    "....KuuuuuuK....",
    "....KuKKKKuK....",
    "....KKyyyyKK....",
    "....KyyKyyyK....",
    "....KyyKKKyK....",
    "....KKyyyyKK....",
    "....KuKKKKuK....",
    "....KuKkkKuK....",
    "....KuKkYkuK....",
    "....KuKkYKuK....",
    "....KuKYYYuK....",
    "....KuKKKKuK....",
    "....KuuuuuuK....",
    "...KKuKKKKuKK...",
    "...KKKK..KKKK...",
];

const PIANO: &[&str] = &[
    "................",
    "KKKKKKKKKKKKKKKK",
    "KmmmmmmmmmmmmmmK",
    "KmKKKKKKKKKKKKmK",
    "KmKwwwwKKwwwwKmK",
    "KmKwkwwKKwkwwKmK",
    "KmKKKKKKKKKKKKmK",
    "KmmmmmmmmmmmmmmK",
    "KKKKKKKKKKKKKKKK",
    "KwKwKwwKwKwKwwKK",
    "KwwwwwwwwwwwwwwK",
    "KKKKKKKKKKKKKKKK",
    "KmmmmmmmmmmmmmmK",
    "KmK..........KmK",
    "KKK..........KKK",
    "................",
];

const GLOBE: &[&str] = &[
    "................",
    ".....KKKKK......",
    "...KKSBgBSKK....",
    "..KSBggBBSBSK...",
    "..KBgggBSSBBYK..",
    ".KSBggBBSSgBBYK.",
    ".KBBBBSSSgggBYK.",
    ".KSSBBSSggggSYK.",
    "..KBBSSBgggBYK..",
    "..KSBBSSBBBSYK..",
    "...KKSBBBSKYK...",
    ".....KKKKKYK....",
    "......KuuK......",
    ".....KuuuuK.....",
    "....KuuuuuuK....",
    "....KKKKKKKK....",
];

const TELESCOPE: &[&str] = &[
    "............KK..",
    "...........KyYK.",
    "..........KYYYK.",
    ".........KYYYK..",
    "........KYYYK...",
    ".......KYYYK....",
    "......KCYYK.....",
    ".....KYKKK......",
    "....KYKKuK......",
    "....KuK.KuK.....",
    "...KuK..KuK.....",
    "...KuK...KuK....",
    "..KuK....KuK....",
    "..KuK.....KuK...",
    "..KKK.....KKK...",
    "................",
];

const PLUSH: &[&str] = &[
    "....KK...KK.....",
    "...KwbK.KbwK....",
    "...KwbK.KbwK....",
    "...KwbK.KbwK....",
    "....KwKKKwK.....",
    "...KwwwwwwwK....",
    "..KwwKwwwKwwK...",
    "..KwwwwPwwwwK...",
    "...KwwwwwwwK....",
    "..KKKwwwwwKKK...",
    ".KwwKwwwwwKwwK..",
    ".KwwKwwwwwKwwK..",
    "..KKwwwwwwwKK...",
    "...KbwwwwwbK....",
    "....KKKKKKK.....",
    "................",
];

const FISH_BOWL: &[&str] = &[
    "................",
    "................",
    ".....KKKKKK.....",
    "....K.w....K....",
    "...KaaaaaaaaK...",
    "..KaaaaaaoaaaK..",
    "..KaaoaaaoooaK..",
    "..KaoooaaaoaaK..",
    "..KaaoaaaaaaaK..",
    "...KangaanaaK...",
    "....KKKKKKKK....",
    "...KKKKKKKKKK...",
    "...KnnnnnnnnK...",
    "....KuK..KuK....",
    "....KKK..KKK....",
    "................",
];

const FISH_TANK: &[&str] = &[
    "KKKKKKKKKKKKKKKK",
    "KkkkkkkkkkkkkkkK",
    "KaaaaaaaaaaaaaaK",
    "KawaaoaaaaaaaaaK",
    "KaaaoooaaaaPaaaK",
    "KaaaaoaaaaPPPaaK",
    "KaagaaaaaaaPaaaK",
    "KaggaaaaaaaaagaK",
    "KngnnnYnnnnnggnK",
    "KKKKKKKKKKKKKKKK",
    "KuCCCCCCCCCCCCuK",
    "KuCCCCYCCYCCCCuK",
    "KuCCCCCCCCCCCCuK",
    "KuuuuuuuuuuuuuuK",
    "KKuKKKKKKKKKKuKK",
    ".KKK........KKK.",
];

/// A roll of wallpaper: 0-1 the paper, 2 its fleck.
const WALLPAPER: &[&str] = &[
    "................",
    "................",
    "...KKKKKKKKKK...",
    "..K0010010010K..",
    "..K0120120120KK.",
    "..K0010010010KyK",
    "..K0120120120KyK",
    "..K0010010010KyK",
    "..K0120120120KKK",
    "..K0010010010K..",
    "..K0120120120K..",
    "..KKKKKKKKKKKK..",
    "...KyyyyyyyyK...",
    "....KKKKKKKK....",
];

fn furniture_icon(f: Furn) -> &'static [&'static str] {
    match f {
        Furn::Bed => BED,
        Furn::CanopyBed => CANOPY_BED,
        Furn::Stove => STOVE,
        Furn::Range => RANGE,
        Furn::Counter => COUNTER,
        Furn::Icebox => ICEBOX,
        Furn::RoundTable => ROUND_TABLE,
        Furn::DiningTable => DINING_TABLE,
        Furn::Chair => CHAIR,
        Furn::Armchair => ARMCHAIR,
        Furn::Sofa => SOFA,
        Furn::Bookshelf => BOOKSHELF,
        Furn::Wardrobe => WARDROBE,
        Furn::Dresser => DRESSER,
        Furn::FloorLamp => FLOOR_LAMP,
        Furn::Candelabra => CANDELABRA,
        Furn::Fireplace => FIREPLACE,
        Furn::Fern => FERN,
        Furn::Cactus => CACTUS,
        Furn::Vase => VASE,
        Furn::Clock => CLOCK,
        Furn::Piano => PIANO,
        Furn::Globe => GLOBE,
        Furn::Telescope => TELESCOPE,
        Furn::Plush => PLUSH,
        Furn::FishBowl => FISH_BOWL,
        Furn::FishTank => FISH_TANK,
    }
}

/// A swatch of a texture with an ink border (floor icons, rug icons).
fn swatch(src: &Texture, inset: i32, rug: bool) -> Texture {
    let mut t = Texture::clear(16, 16);
    for y in inset..16 - inset {
        for x in 1..15 {
            let (sx, sy) = if rug {
                (((x - 1) * 16 / 14), ((y - inset) * 16 / (16 - inset * 2)))
            } else {
                (x, y)
            };
            let c = src.get(sx, sy);
            if c != CLEAR {
                t.set(x, y, c);
            }
        }
    }
    let copy = t.clone();
    for y in 0..16 {
        for x in 0..16 {
            if copy.get(x, y) != CLEAR {
                continue;
            }
            let near = [(1, 0), (-1, 0), (0, 1), (0, -1)].iter().any(|(dx, dy)| {
                let (nx, ny) = (x + dx, y + dy);
                (0..16).contains(&nx) && (0..16).contains(&ny) && copy.get(nx, ny) != CLEAR
            });
            if near {
                t.set(x, y, INK);
            }
        }
    }
    t
}

/// Wallpaper colours for each style (see `tiles::wallpaper`).
fn paper_colors(style: u8) -> [u8; 4] {
    match style {
        2 => [MINT, AQUA, WHITE, CLEAR],
        3 => [PURPLE, GRAPE, CREAM, CLEAR],
        4 => [GOLD, CLAY, RUST, CLEAR],
        5 => [CREAM, PEACH, PINK, CLEAR],
        6 => [TEAL, DEEP_TEAL, GOLD, CLEAR],
        7 => [PEACH, SAND, PINK, CLEAR],
        _ => [SAND, PEACH, CLAY, CLEAR],
    }
}

pub fn build(bank: &mut TexBank, icons: &mut HashMap<&'static str, TexId>) -> HomeArt {
    let mut k = Kit {
        solids: HashMap::new(),
        w4: Texture::new(4, 4, 0),
    };
    let furn: Vec<Mesh> = FURNS.iter().map(|f| furniture(bank, &mut k, *f)).collect();
    for f in FURNS {
        icons.insert(f.item().def().icon, bank.add(art(furniture_icon(f), NO)));
    }
    // Glass and water for the tanks, drawn see-through: clear panes with bright edges and a
    // glint, over calm water.
    let pane = bank.add(glass_pane());
    let glass = bank.add(glints());
    let water = bank.add(calm_water());
    let mut tank_glass = Mesh::new();
    tank_glass.cube(
        v(-0.9, 0.5, -0.32),
        v(0.9, 1.24, 0.28),
        &crate::render::mesh::BoxUv::all(UvRect::new(0.0, 0.0, 32.0, 32.0)),
        pane,
        1 << crate::render::mesh::TOP,
    );
    let mut tank_water = Mesh::new();
    tiled_box(
        &mut tank_water,
        v(-0.86, 0.58, -0.28),
        v(0.86, 1.1, 0.24),
        water,
        0,
    );
    let mut bowl_glass = Mesh::new();
    lathe(
        &mut bowl_glass,
        v(0.0, 0.52, 0.0),
        &[
            (0.14, 0.0),
            (0.24, 0.1),
            (0.25, 0.22),
            (0.18, 0.34),
            (0.14, 0.36),
        ],
        10,
        0.0,
        glass,
        false,
    );
    let mut bowl_water = Mesh::new();
    lathe(
        &mut bowl_water,
        v(0.0, 0.54, 0.0),
        &[(0.13, 0.0), (0.22, 0.09), (0.22, 0.2), (0.0, 0.26)],
        10,
        0.0,
        water,
        false,
    );
    let mut lamp_shade = Mesh::new();
    let shade = bank.add(tiles::stripes(CREAM, GOLD));
    lathe(
        &mut lamp_shade,
        v(0.0, 1.14, 0.0),
        &[(0.26, 0.0), (0.2, 0.18), (0.14, 0.3), (0.0, 0.3)],
        8,
        0.0,
        shade,
        false,
    );
    let mut pendulum = Mesh::new();
    k.paint(
        bank,
        &mut pendulum,
        v(-0.01, -0.6, -0.01),
        v(0.01, 0.0, 0.01),
        GOLD,
    );
    let brass = metal(bank, [CREAM, GOLD, CLAY]);
    lathe(
        &mut pendulum,
        v(0.0, -0.68, 0.0),
        &[(0.0, -0.06), (0.07, 0.0), (0.0, 0.06)],
        8,
        0.0,
        brass,
        false,
    );
    let mut globe = Mesh::new();
    let mut map = Texture::new(16, 16, BLUE);
    for y in 0..16 {
        for x in 0..16 {
            let land = ((x as f32 * 0.9).sin() + (y as f32 * 0.7 + x as f32 * 0.2).cos()) > 0.8;
            if land {
                map.set(x, y, if (x + y) % 5 == 0 { LIME } else { GREEN });
            } else if (x * 3 + y) % 13 == 0 {
                map.set(x, y, SKY);
            }
        }
    }
    let map = bank.add(map);
    lathe(
        &mut globe,
        Vec3::ZERO,
        &[
            (0.0, -0.24),
            (0.17, -0.17),
            (0.24, 0.0),
            (0.17, 0.17),
            (0.0, 0.24),
        ],
        10,
        0.0,
        map,
        false,
    );
    // Rugs, pictures, wallpaper and floors.
    let mut rugs = Vec::new();
    for (i, (_, pal)) in RUGS.iter().enumerate() {
        let t = rug_tex(i, *pal);
        let icon = swatch(&t, 3, true);
        rugs.push(bank.add(t));
        if let Some(item) = crate::game::home::rug_item(i as u8) {
            icons.insert(item.def().icon, bank.add(icon));
        }
    }
    let fish = icons.get("golden_carp").copied().unwrap_or(0);
    let mut art_meshes = Vec::new();
    for kind in 0..4 {
        art_meshes.push(wall_art(bank, &mut k, kind, fish));
        if let Some(item) = crate::game::home::art_item(kind as u8) {
            let icon = if kind == 3 {
                let mut t = bank.get(fish).clone();
                for x in 0..16 {
                    t.set(x, 15, RUST);
                    t.set(x, 14, if x % 3 == 0 { GOLD } else { RUST });
                }
                t
            } else {
                let mut t = Texture::new(16, 16, GOLD);
                let p = painting_tex(kind);
                for y in 2..14 {
                    for x in 2..14 {
                        t.set(x, y, p.get(x + 1, y + 1));
                    }
                }
                for i in 0..16 {
                    t.set(i, 0, INK);
                    t.set(i, 15, INK);
                    t.set(0, i, INK);
                    t.set(15, i, INK);
                }
                t
            };
            icons.insert(item.def().icon, bank.add(icon));
        }
    }
    for style in [2u8, 3, 4, 5, 6, 7, 8] {
        if let Some(item) = crate::game::home::wallpaper_item(style) {
            icons.insert(
                item.def().icon,
                bank.add(art(WALLPAPER, paper_colors(style))),
            );
        }
    }
    let floors = [
        tiles::planks(CLAY, GOLD, RUST, 20),
        tiles::checker(CREAM, PEACH, SAND),
        tiles::carpet(CRIMSON, PLUM, GOLD),
        tiles::carpet(GRAPE, INDIGO, LAVENDER),
        tiles::carpet(DEEP_TEAL, INDIGO, GOLD),
        tiles::cobbles(KHAKI, SAND, SHADOW, 21),
    ];
    for (i, t) in floors.iter().enumerate() {
        if let Some(item) = crate::game::home::flooring_item(i as u8) {
            icons.insert(item.def().icon, bank.add(swatch(t, 1, false)));
        }
    }
    HomeArt {
        furn,
        tank_glass,
        tank_water,
        bowl_glass,
        bowl_water,
        lamp_shade,
        pendulum,
        globe,
        rugs,
        art: art_meshes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn home_art_is_tidy() {
        for f in FURNS {
            let rows = furniture_icon(f);
            assert!(rows.len() <= 16, "{f:?}");
            for r in rows {
                assert_eq!(r.chars().count(), 16, "{f:?}: {r}");
            }
        }
        for r in WALLPAPER {
            assert!(r.chars().count() <= 16);
        }
        let mut bank = TexBank::default();
        let mut icons = HashMap::new();
        crate::assets::fish_art::build(&mut bank, &mut icons);
        let h = build(&mut bank, &mut icons);
        assert_eq!(h.furn.len(), FURNS.len());
        assert!(h.furn.iter().all(|m| !m.tris.is_empty()));
        assert_eq!(h.rugs.len(), RUGS.len());
        for id in icons.values() {
            assert!(bank.get(*id).data.iter().all(|&c| c < 32 || c == CLEAR));
        }
    }
}
