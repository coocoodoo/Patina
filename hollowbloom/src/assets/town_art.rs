//! Models for Bramblewick and the road to it: storybook buildings built from a short
//! description, the bus and its shelter, street furniture, and the fittings inside the shops.
//! Local space as in `models`: origin on the ground at the anchor tile's centre, +y up, the
//! front facing +z (towards the camera).

use std::collections::HashMap;
use std::f32::consts::{FRAC_PI_2, PI, TAU};

use glam::{Mat4, Vec2, Vec3};

use super::models::{lathe, skin_box, tiled_box};
use super::tiles;
use crate::game::town::{BUILDINGS, Building, Extra, Facade};
use crate::palette::*;
use crate::render::{Mesh, TexBank, TexId, Texture, UvRect};

const TD: f32 = 16.0;

/// Solid-colour swatches, shared between models.
struct Kit {
    solids: HashMap<u8, TexId>,
    w4: Texture,
}

impl Kit {
    fn new() -> Kit {
        Kit {
            solids: HashMap::new(),
            w4: Texture::new(4, 4, 0),
        }
    }

    fn solid(&mut self, bank: &mut TexBank, c: u8) -> TexId {
        *self
            .solids
            .entry(c)
            .or_insert_with(|| bank.add(tiles::solid(c)))
    }

    /// A box painted one colour.
    fn paint(&mut self, bank: &mut TexBank, m: &mut Mesh, min: Vec3, max: Vec3, c: u8) {
        let t = self.solid(bank, c);
        skin_box(m, min, max, t, &self.w4);
    }
}

/// A quad facing +z (the camera) at depth `z`, from (x0, y0) to (x1, y1).
#[allow(clippy::too_many_arguments)]
fn front_quad(m: &mut Mesh, x0: f32, y0: f32, x1: f32, y1: f32, z: f32, uv: UvRect, t: TexId) {
    m.quad(
        [
            Vec3::new(x0, y0, z),
            Vec3::new(x1, y0, z),
            Vec3::new(x1, y1, z),
            Vec3::new(x0, y1, z),
        ],
        uv,
        t,
    );
}

/// A quad facing -z, for the back of signs.
#[allow(clippy::too_many_arguments)]
fn back_quad(m: &mut Mesh, x0: f32, y0: f32, x1: f32, y1: f32, z: f32, uv: UvRect, t: TexId) {
    m.quad(
        [
            Vec3::new(x1, y0, z),
            Vec3::new(x0, y0, z),
            Vec3::new(x0, y1, z),
            Vec3::new(x1, y1, z),
        ],
        uv,
        t,
    );
}

/// A roof slope from a low edge (a, b) up to a high edge (c, d), tiled at world density.
fn slope(m: &mut Mesh, a: Vec3, b: Vec3, c: Vec3, d: Vec3, t: TexId) {
    let wid = (b - a).length() * TD;
    let len = (c - b).length() * TD;
    m.quad([a, b, c, d], UvRect::new(0.0, 0.0, wid, len), t);
}

pub struct TownArt {
    /// Each building's body, and its windows (drawn glowing at night).
    pub buildings: Vec<(Mesh, Mesh)>,
    pub bus: Mesh,
    pub bus_glass: Mesh,
    pub wheel: Mesh,
    pub shelter: Mesh,
    pub fountain: Mesh,
    pub fountain_water: Mesh,
    pub street_lamp: Mesh,
    pub lamp_glass: Mesh,
    pub board: Mesh,
    /// Empty beds first, then three kinds of flowers.
    pub planters: Vec<Mesh>,
    pub bushes: Vec<Mesh>,
    pub well: Mesh,
    pub stands: Vec<Mesh>,
    pub barrel: Mesh,
    pub counter: Mesh,
    pub shelf: Mesh,
    pub rack: Mesh,
    pub dummy_base: Mesh,
    /// The wooden skin for a mannequin's head.
    pub dummy_wood: TexId,
    pub tables: Vec<Mesh>,
    pub stool: Mesh,
    pub hearth: Mesh,
    /// Oven, anvil, cauldron, gem case, tinker's bench, seed bins, bookcase, desk, taps,
    /// potted plant.
    pub fixtures: Vec<Mesh>,
    pub window: Mesh,
    pub window_glass: Mesh,
    pub paintings: Vec<Mesh>,
    pub sconce: Mesh,
    pub rugs: Vec<TexId>,
    pub bunting: TexId,
}

pub fn build(bank: &mut TexBank, icons: &HashMap<&'static str, TexId>, water: TexId) -> TownArt {
    let mut k = Kit::new();
    let buildings = BUILDINGS
        .iter()
        .map(|b| building(bank, &mut k, icons, b))
        .collect();
    let (bus, bus_glass, wheel) = bus(bank, &mut k);
    let shelter = shelter(bank, &mut k, icons);
    let (fountain, fountain_water) = fountain(bank, &mut k, water);
    let (street_lamp, lamp_glass) = street_lamp(bank, &mut k);
    let board = board(bank, &mut k);
    let planters = planters(bank, &mut k);
    let bushes = bushes(bank, &mut k);
    let well = well(bank, &mut k);
    let stands = stands(bank, &mut k);
    let barrel = barrel(bank, &mut k);
    let fit = fittings(bank, &mut k);
    let rugs = vec![
        bank.add(rug_tex([CRIMSON, GOLD, PLUM])),
        bank.add(rug_tex([TEAL, CREAM, DEEP_TEAL])),
        bank.add(rug_tex([PURPLE, BLUSH, GRAPE])),
        bank.add(rug_tex([GOLD, CREAM, RUST])),
        bank.add(rug_tex([GREEN, LIME, TEAL])),
        bank.add(rug_tex([BLUE, SKY, INDIGO])),
    ];
    let bunting = bank.add(bunting_tex());
    TownArt {
        buildings,
        bus,
        bus_glass,
        wheel,
        shelter,
        fountain,
        fountain_water,
        street_lamp,
        lamp_glass,
        board,
        planters,
        bushes,
        well,
        stands,
        barrel,
        counter: fit.counter,
        shelf: fit.shelf,
        rack: fit.rack,
        dummy_base: fit.dummy_base,
        dummy_wood: fit.dummy_wood,
        tables: fit.tables,
        stool: fit.stool,
        hearth: fit.hearth,
        fixtures: fit.fixtures,
        window: fit.window,
        window_glass: fit.window_glass,
        paintings: fit.paintings,
        sconce: fit.sconce,
        rugs,
        bunting,
    }
}

// ------------------------------------------------------------------------------------------
// Buildings
// ------------------------------------------------------------------------------------------

fn building(
    bank: &mut TexBank,
    k: &mut Kit,
    icons: &HashMap<&'static str, TexId>,
    b: &Building,
) -> (Mesh, Mesh) {
    let l = &b.look;
    let (w, d) = (b.w as f32, b.d as f32);
    let [hi, mid, lo] = l.wall;
    let seed = (b.x * 31 + b.z) as u64;
    let wall_t = match l.facade {
        Facade::Plaster | Facade::Timber => bank.add(tiles::plaster(l.wall, seed)),
        Facade::Brick => bank.add(tiles::bricks(mid, hi, lo, seed)),
        Facade::Stone => bank.add(tiles::bricks(mid, hi, lo, seed)),
        Facade::Boards => bank.add(tiles::boards(l.wall, seed)),
    };
    let roof_t = bank.add(tiles::roof(l.roof));
    let stone = bank.add(tiles::cobbles(KHAKI, SAND, SHADOW, seed + 1));
    let door_t = bank.add(tiles::planks(l.door, l.door, MAROON, seed + 2));
    let glass = bank.add(tiles::crystal([CREAM, GOLD, SKY]));
    let beam = k.solid(bank, MAROON);
    let trim = k.solid(bank, l.trim);

    let (x0, x1) = (-0.42, w - 0.58);
    let (z0, z1) = (-(d - 1.0) - 0.42, 0.28);
    let shop = l.awning.is_some();
    let hgt = if l.tall {
        2.5
    } else if shop {
        1.95
    } else {
        1.6
    };
    let mut m = Mesh::new();
    let mut win = Mesh::new();
    // Plinth and walls.
    tiled_box(
        &mut m,
        Vec3::new(x0 - 0.05, 0.0, z0 - 0.05),
        Vec3::new(x1 + 0.05, 0.16, z1 + 0.05),
        stone,
        0,
    );
    tiled_box(
        &mut m,
        Vec3::new(x0, 0.16, z0),
        Vec3::new(x1, hgt, z1),
        wall_t,
        1 << crate::render::mesh::TOP,
    );
    // Timber framing: posts, a sill beam and a band between the storeys.
    if l.facade == Facade::Timber {
        let mut x = x0;
        while x <= x1 + 0.01 {
            k.paint(
                bank,
                &mut m,
                Vec3::new(x - 0.05, 0.16, z1),
                Vec3::new(x + 0.05, hgt, z1 + 0.04),
                MAROON,
            );
            x += 1.0;
        }
        for y in [0.16, hgt - 0.1] {
            tiled_box(
                &mut m,
                Vec3::new(x0, y, z1),
                Vec3::new(x1, y + 0.1, z1 + 0.04),
                beam,
                0,
            );
        }
    }
    if l.tall {
        tiled_box(
            &mut m,
            Vec3::new(x0 - 0.02, 1.42, z0 - 0.02),
            Vec3::new(x1 + 0.02, 1.5, z1 + 0.06),
            trim,
            0,
        );
    }

    // Roof.
    let over = 0.25;
    let (rx0, rx1, rz0, rz1) = (x0 - over, x1 + over, z0 - over, z1 + 0.3);
    let eave = hgt - 0.05;
    if l.gable {
        // Ridge runs front to back; the gable end faces the street.
        let xm = (x0 + x1) * 0.5;
        let rise = ((x1 - x0) * 0.5 * 0.75).min(1.5);
        let top = eave + rise;
        slope(
            &mut m,
            Vec3::new(rx1, eave, rz1),
            Vec3::new(rx1, eave, rz0),
            Vec3::new(xm, top, rz0),
            Vec3::new(xm, top, rz1),
            roof_t,
        );
        slope(
            &mut m,
            Vec3::new(rx0, eave, rz0),
            Vec3::new(rx0, eave, rz1),
            Vec3::new(xm, top, rz1),
            Vec3::new(xm, top, rz0),
            roof_t,
        );
        // Gable triangles (front and back) in the wall material.
        let span = (x1 - x0) * TD;
        let uv = [
            Vec2::new(0.0, rise * TD),
            Vec2::new(span, rise * TD),
            Vec2::new(span * 0.5, 0.0),
        ];
        m.tri(
            [
                Vec3::new(x0, hgt, z1),
                Vec3::new(x1, hgt, z1),
                Vec3::new(xm, top - 0.05, z1),
            ],
            uv,
            wall_t,
        );
        m.tri(
            [
                Vec3::new(x1, hgt, z0),
                Vec3::new(x0, hgt, z0),
                Vec3::new(xm, top - 0.05, z0),
            ],
            uv,
            wall_t,
        );
        // A round attic window.
        let wy = hgt + rise * 0.35;
        k.paint(
            bank,
            &mut m,
            Vec3::new(xm - 0.22, wy - 0.22, z1),
            Vec3::new(xm + 0.22, wy + 0.22, z1 + 0.03),
            l.trim,
        );
        front_quad(
            &mut win,
            xm - 0.16,
            wy - 0.16,
            xm + 0.16,
            wy + 0.16,
            z1 + 0.04,
            UvRect::px(0, 0, 6, 6),
            glass,
        );
    } else {
        let zm = (z0 + z1) * 0.5;
        let rise = ((z1 - z0) * 0.5 * 0.7).min(1.4);
        let top = eave + rise;
        slope(
            &mut m,
            Vec3::new(rx0, eave, rz1),
            Vec3::new(rx1, eave, rz1),
            Vec3::new(rx1, top, zm),
            Vec3::new(rx0, top, zm),
            roof_t,
        );
        slope(
            &mut m,
            Vec3::new(rx1, eave, rz0),
            Vec3::new(rx0, eave, rz0),
            Vec3::new(rx0, top, zm),
            Vec3::new(rx1, top, zm),
            roof_t,
        );
        let depth = (z1 - z0) * TD;
        let uv = [
            Vec2::new(0.0, rise * TD),
            Vec2::new(depth, rise * TD),
            Vec2::new(depth * 0.5, 0.0),
        ];
        m.tri(
            [
                Vec3::new(x0, hgt, z0),
                Vec3::new(x0, hgt, z1),
                Vec3::new(x0, top - 0.05, zm),
            ],
            uv,
            wall_t,
        );
        m.tri(
            [
                Vec3::new(x1, hgt, z1),
                Vec3::new(x1, hgt, z0),
                Vec3::new(x1, top - 0.05, zm),
            ],
            uv,
            wall_t,
        );
    }
    // Chimney.
    if l.chimney {
        let brick = bank.add(tiles::bricks(RUST, CLAY, MAROON, seed + 3));
        let cx = x1 - 0.9;
        let cz = if l.gable {
            z0 + 0.8
        } else {
            (z0 + z1) * 0.5 - 0.3
        };
        tiled_box(
            &mut m,
            Vec3::new(cx - 0.22, hgt, cz - 0.22),
            Vec3::new(cx + 0.22, hgt + 1.45, cz + 0.22),
            brick,
            0,
        );
        k.paint(
            bank,
            &mut m,
            Vec3::new(cx - 0.26, hgt + 1.45, cz - 0.26),
            Vec3::new(cx + 0.26, hgt + 1.55, cz + 0.26),
            SHADOW,
        );
    }

    // The door, with its frame, a little window, a knob and a step.
    let dx = b.door as f32;
    let double = b.w >= 8;
    let dw = if double { 0.46 } else { 0.34 };
    tiled_box(
        &mut m,
        Vec3::new(dx - dw - 0.07, 0.16, z1),
        Vec3::new(dx + dw + 0.07, 1.2, z1 + 0.03),
        trim,
        0,
    );
    tiled_box(
        &mut m,
        Vec3::new(dx - dw, 0.16, z1 + 0.03),
        Vec3::new(dx + dw, 1.12, z1 + 0.06),
        door_t,
        0,
    );
    if double {
        k.paint(
            bank,
            &mut m,
            Vec3::new(dx - 0.02, 0.16, z1 + 0.06),
            Vec3::new(dx + 0.02, 1.12, z1 + 0.07),
            MAROON,
        );
    }
    front_quad(
        &mut win,
        dx - 0.14,
        0.78,
        dx + 0.14,
        1.0,
        z1 + 0.07,
        UvRect::px(0, 0, 5, 4),
        glass,
    );
    k.paint(
        bank,
        &mut m,
        Vec3::new(dx + dw - 0.14, 0.6, z1 + 0.06),
        Vec3::new(dx + dw - 0.08, 0.68, z1 + 0.1),
        GOLD,
    );
    tiled_box(
        &mut m,
        Vec3::new(dx - 0.5, 0.0, z1),
        Vec3::new(dx + 0.5, 0.1, z1 + 0.32),
        stone,
        0,
    );

    // Windows on the ground floor (display windows under an awning for shops) and above.
    let flower_box = bank.add(tiles::planks(CLAY, GOLD, RUST, seed + 4));
    for i in 0..b.w {
        let x = i as f32;
        if l.tall {
            tiled_box(
                &mut m,
                Vec3::new(x - 0.25, 1.62, z1),
                Vec3::new(x + 0.25, 2.18, z1 + 0.03),
                trim,
                0,
            );
            front_quad(
                &mut win,
                x - 0.2,
                1.67,
                x + 0.2,
                2.13,
                z1 + 0.04,
                UvRect::px(0, 0, 7, 8),
                glass,
            );
        }
        if i == b.door || (double && (i - b.door).abs() == 1 && b.w < 9) {
            continue;
        }
        let (hw, y0, y1) = if shop {
            (0.36, 0.34, 1.0)
        } else {
            (0.27, 0.55, 1.05)
        };
        tiled_box(
            &mut m,
            Vec3::new(x - hw - 0.05, y0 - 0.05, z1),
            Vec3::new(x + hw + 0.05, y1 + 0.05, z1 + 0.03),
            trim,
            0,
        );
        front_quad(
            &mut win,
            x - hw,
            y0,
            x + hw,
            y1,
            z1 + 0.04,
            UvRect::px(0, 0, 11, 8),
            glass,
        );
        if !shop {
            // Flower box.
            tiled_box(
                &mut m,
                Vec3::new(x - hw - 0.05, y0 - 0.12, z1),
                Vec3::new(x + hw + 0.05, y0 - 0.02, z1 + 0.15),
                flower_box,
                0,
            );
            for (j, c) in [PINK, CREAM, SKY].into_iter().enumerate() {
                let fx = x - hw + 0.1 + j as f32 * (hw * 2.0 - 0.2) / 2.0;
                k.paint(
                    bank,
                    &mut m,
                    Vec3::new(fx - 0.05, y0 - 0.02, z1 + 0.04),
                    Vec3::new(fx + 0.05, y0 + 0.06, z1 + 0.12),
                    c,
                );
            }
        } else {
            // A sill with goods on show.
            k.paint(
                bank,
                &mut m,
                Vec3::new(x - hw - 0.06, y0 - 0.1, z1),
                Vec3::new(x + hw + 0.06, y0 - 0.04, z1 + 0.1),
                l.trim,
            );
        }
    }
    // A window on each side wall.
    let zm = (z0 + z1) * 0.5;
    for (x, n) in [(x0 - 0.01, -1.0f32), (x1 + 0.01, 1.0)] {
        let p = |dz: f32, y: f32| Vec3::new(x, y, zm + dz);
        let q = if n > 0.0 {
            [p(0.3, 0.6), p(-0.3, 0.6), p(-0.3, 1.05), p(0.3, 1.05)]
        } else {
            [p(-0.3, 0.6), p(0.3, 0.6), p(0.3, 1.05), p(-0.3, 1.05)]
        };
        win.quad(q, UvRect::px(0, 0, 10, 7), glass);
    }

    // An awning over the shop front.
    if let Some((a, c)) = l.awning {
        let cloth = bank.add(tiles::stripes(a, c));
        let (ay0, ay1) = if l.tall { (1.38, 1.18) } else { (1.4, 1.2) };
        let (ax0, ax1) = (x0 + 0.05, x1 - 0.05);
        slope(
            &mut m,
            Vec3::new(ax0, ay1, z1 + 0.5),
            Vec3::new(ax1, ay1, z1 + 0.5),
            Vec3::new(ax1, ay0, z1),
            Vec3::new(ax0, ay0, z1),
            cloth,
        );
        // The scalloped valance.
        front_quad(
            &mut m,
            ax0,
            ay1 - 0.12,
            ax1,
            ay1,
            z1 + 0.5,
            UvRect::new(0.0, 0.0, (ax1 - ax0) * TD, 2.0),
            cloth,
        );
    }

    // The shop sign, above the awning.
    if let Some(icon) = l.sign {
        let sx = (x0 + x1) * 0.5;
        let (sy0, sy1) = (1.45, 1.87);
        let board = bank.add(tiles::planks(CREAM, WHITE, SAND, seed + 5));
        tiled_box(
            &mut m,
            Vec3::new(sx - 0.55, sy0, z1),
            Vec3::new(sx + 0.55, sy1, z1 + 0.05),
            board,
            0,
        );
        for (a, b) in [(sy0 - 0.03, sy0), (sy1, sy1 + 0.03)] {
            k.paint(
                bank,
                &mut m,
                Vec3::new(sx - 0.58, a, z1),
                Vec3::new(sx + 0.58, b, z1 + 0.07),
                l.trim,
            );
        }
        if let Some(&t) = icons.get(icon) {
            let uv = UvRect::new(0.0, 0.0, 16.0, 16.0);
            front_quad(
                &mut m,
                sx - 0.19,
                sy0 + 0.02,
                sx + 0.19,
                sy1 - 0.02,
                z1 + 0.06,
                uv,
                t,
            );
            // A little flourish either side of the picture.
            for side in [-1.0f32, 1.0] {
                k.paint(
                    bank,
                    &mut m,
                    Vec3::new(sx + side * 0.36 - 0.05, 1.63, z1 + 0.05),
                    Vec3::new(sx + side * 0.36 + 0.05, 1.69, z1 + 0.07),
                    l.trim,
                );
            }
        }
    }

    match l.extra {
        Extra::ClockTower => {
            let cx = (x0 + x1) * 0.5;
            let cz = z1 - 1.0;
            let (tw, top) = (0.8, hgt + 2.2);
            tiled_box(
                &mut m,
                Vec3::new(cx - tw, hgt - 0.1, cz - tw),
                Vec3::new(cx + tw, top, cz + tw),
                wall_t,
                0,
            );
            k.paint(
                bank,
                &mut m,
                Vec3::new(cx - tw - 0.05, top, cz - tw - 0.05),
                Vec3::new(cx + tw + 0.05, top + 0.1, cz + tw + 0.05),
                l.trim,
            );
            let face = bank.add(tiles::clock_face());
            front_quad(
                &mut m,
                cx - 0.55,
                top - 1.25,
                cx + 0.55,
                top - 0.15,
                cz + tw + 0.02,
                UvRect::new(0.0, 0.0, 16.0, 16.0),
                face,
            );
            // Belfry openings and a pointed roof with a golden finial.
            let dark = k.solid(bank, INK);
            for s in [-0.4f32, 0.4] {
                front_quad(
                    &mut m,
                    cx + s - 0.14,
                    top - 1.9,
                    cx + s + 0.14,
                    top - 1.45,
                    cz + tw + 0.02,
                    UvRect::px(0, 0, 4, 4),
                    dark,
                );
            }
            lathe(
                &mut m,
                Vec3::new(cx, top + 0.1, cz),
                &[(1.2, 0.0), (0.0, 1.5)],
                4,
                PI / 4.0,
                roof_t,
                true,
            );
            k.paint(
                bank,
                &mut m,
                Vec3::new(cx - 0.07, top + 1.55, cz - 0.07),
                Vec3::new(cx + 0.07, top + 1.72, cz + 0.07),
                GOLD,
            );
        }
        Extra::Turret => {
            let cx = x1 - 0.55;
            let cz = z1 - 0.55;
            lathe(
                &mut m,
                Vec3::new(cx, 0.0, cz),
                &[(0.62, 0.0), (0.6, hgt + 1.3)],
                8,
                0.2,
                wall_t,
                false,
            );
            lathe(
                &mut m,
                Vec3::new(cx, hgt + 1.25, cz),
                &[(0.9, 0.0), (0.5, 0.6), (0.0, 1.5)],
                8,
                0.2,
                roof_t,
                true,
            );
            for (i, y) in [0.7f32, 1.6, 2.4].into_iter().enumerate() {
                let a = 0.4 + i as f32 * 0.35;
                let p = Vec3::new(cx + a.sin() * 0.62, y, cz + a.cos() * 0.62);
                front_quad(
                    &mut win,
                    p.x - 0.1,
                    p.y,
                    p.x + 0.1,
                    p.y + 0.35,
                    p.z + 0.01,
                    UvRect::px(0, 0, 3, 6),
                    glass,
                );
            }
            // A star on the tip.
            let top = hgt + 1.25 + 1.5;
            for (sx, sy) in [(0.18f32, 0.05f32), (0.05, 0.18)] {
                k.paint(
                    bank,
                    &mut m,
                    Vec3::new(cx - sx, top + 0.1 - sy, cz - 0.03),
                    Vec3::new(cx + sx, top + 0.1 + sy, cz + 0.03),
                    GOLD,
                );
            }
        }
        Extra::Banners => {
            let cloth = bank.add(tiles::banner());
            for bx in [dx - 1.6, dx + 1.6] {
                front_quad(
                    &mut m,
                    bx - 0.24,
                    0.7,
                    bx + 0.24,
                    1.35,
                    z1 + 0.05,
                    UvRect::new(0.0, 0.0, 16.0, 16.0),
                    cloth,
                );
                k.paint(
                    bank,
                    &mut m,
                    Vec3::new(bx - 0.3, 1.35, z1),
                    Vec3::new(bx + 0.3, 1.4, z1 + 0.1),
                    GOLD,
                );
            }
        }
        Extra::SnailSign => {
            // A big snail on a board over the door.
            let sy = 1.35;
            let board = bank.add(tiles::planks(GOLD, CREAM, CLAY, seed + 6));
            tiled_box(
                &mut m,
                Vec3::new(dx - 0.7, sy, z1),
                Vec3::new(dx + 0.7, sy + 0.5, z1 + 0.08),
                board,
                0,
            );
            let shell = bank.add(tiles::roof([BLUSH, PINK, CRIMSON]));
            let mut snail = Mesh::new();
            lathe(
                &mut snail,
                Vec3::ZERO,
                &[(0.0, -0.06), (0.22, 0.0), (0.22, 0.08), (0.0, 0.1)],
                10,
                0.0,
                shell,
                false,
            );
            let body = k.solid(bank, SAND);
            skin_box(
                &mut snail,
                Vec3::new(-0.3, -0.22, -0.02),
                Vec3::new(0.34, -0.14, 0.06),
                body,
                &k.w4,
            );
            for ex in [0.24f32, 0.32] {
                k.paint(
                    bank,
                    &mut snail,
                    Vec3::new(ex - 0.012, -0.14, 0.0),
                    Vec3::new(ex + 0.012, -0.02, 0.03),
                    SAND,
                );
                k.paint(
                    bank,
                    &mut snail,
                    Vec3::new(ex - 0.025, -0.03, -0.01),
                    Vec3::new(ex + 0.025, 0.01, 0.04),
                    INK,
                );
            }
            m.append(
                &snail,
                Mat4::from_translation(Vec3::new(dx, sy + 0.3, z1 + 0.12))
                    * Mat4::from_rotation_x(FRAC_PI_2),
            );
        }
        Extra::Greenhouse => {
            // Pots of seedlings for sale along the shop front.
            let clay = k.solid(bank, CLAY);
            for i in 0..(b.w * 2 - 1) {
                let px = x0 + 0.25 + i as f32 * 0.5;
                if (px - dx).abs() < 0.55 {
                    continue;
                }
                let mut pot = Mesh::new();
                lathe(
                    &mut pot,
                    Vec3::ZERO,
                    &[(0.08, 0.0), (0.11, 0.16), (0.0, 0.16)],
                    6,
                    0.0,
                    clay,
                    false,
                );
                let c = [LIME, GREEN, PINK, GOLD][i as usize % 4];
                k.paint(
                    bank,
                    &mut pot,
                    Vec3::new(-0.07, 0.16, -0.07),
                    Vec3::new(0.07, 0.3, 0.07),
                    c,
                );
                m.append(&pot, Mat4::from_translation(Vec3::new(px, 0.1, z1 + 0.15)));
            }
        }
        Extra::Telescope => {
            let brass = bank.add(tiles::stripes(GOLD, CLAY));
            let mut t = Mesh::new();
            lathe(
                &mut t,
                Vec3::ZERO,
                &[(0.09, 0.0), (0.12, 0.9), (0.14, 1.0)],
                6,
                0.0,
                brass,
                true,
            );
            let cx = (x0 + x1) * 0.5 + 0.8;
            let top = hgt + ((z1 - z0) * 0.5 * 0.7).min(1.4);
            m.append(
                &t,
                Mat4::from_translation(Vec3::new(cx, top - 0.3, (z0 + z1) * 0.5))
                    * Mat4::from_rotation_z(-0.7)
                    * Mat4::from_rotation_x(-0.3),
            );
        }
        Extra::Mailbox => {
            let (mx, mz) = (-0.25, 0.42);
            k.paint(
                bank,
                &mut m,
                Vec3::new(mx - 0.03, 0.0, mz - 0.03),
                Vec3::new(mx + 0.03, 0.5, mz + 0.03),
                SHADOW,
            );
            k.paint(
                bank,
                &mut m,
                Vec3::new(mx - 0.1, 0.5, mz - 0.14),
                Vec3::new(mx + 0.1, 0.68, mz + 0.1),
                RED,
            );
            k.paint(
                bank,
                &mut m,
                Vec3::new(mx + 0.1, 0.56, mz - 0.02),
                Vec3::new(mx + 0.13, 0.76, mz + 0.01),
                GOLD,
            );
        }
        Extra::None => {}
    }
    (m, win)
}

// ------------------------------------------------------------------------------------------
// The bus and its shelter
// ------------------------------------------------------------------------------------------

/// A chubby little bus, driving towards +z: body, windows and one wheel.
fn bus(bank: &mut TexBank, k: &mut Kit) -> (Mesh, Mesh, Mesh) {
    let paint = bank.add(tiles::plaster([AQUA, TEAL, DEEP_TEAL], 77));
    let cream = bank.add(tiles::plaster([WHITE, CREAM, SAND], 78));
    let roof = bank.add(tiles::plaster([WHITE, WHITE, CREAM], 79));
    let glass = bank.add(tiles::crystal([WHITE, SKY, BLUE]));
    let mut m = Mesh::new();
    let mut g = Mesh::new();
    let (hw, z0, z1) = (0.62, -1.7, 1.7);
    // Lower body, a cream band with the windows, a rounded roof.
    tiled_box(
        &mut m,
        Vec3::new(-hw, 0.28, z0),
        Vec3::new(hw, 0.85, z1),
        paint,
        0,
    );
    tiled_box(
        &mut m,
        Vec3::new(-hw, 0.85, z0),
        Vec3::new(hw, 1.45, z1),
        cream,
        0,
    );
    tiled_box(
        &mut m,
        Vec3::new(-hw + 0.06, 1.45, z0 + 0.06),
        Vec3::new(hw - 0.06, 1.58, z1 - 0.06),
        paint,
        0,
    );
    // A cream stripe down the roof, and little lamps at each corner.
    tiled_box(
        &mut m,
        Vec3::new(-0.2, 1.58, z0 + 0.2),
        Vec3::new(0.2, 1.62, z1 - 0.2),
        roof,
        0,
    );
    for (x, z, c) in [
        (-0.45, z1 - 0.2, GOLD),
        (0.45, z1 - 0.2, GOLD),
        (-0.45, z0 + 0.2, RED),
        (0.45, z0 + 0.2, RED),
    ] {
        k.paint(
            bank,
            &mut m,
            Vec3::new(x - 0.06, 1.58, z - 0.06),
            Vec3::new(x + 0.06, 1.66, z + 0.06),
            c,
        );
    }
    // A stripe along the sides.
    for s in [-1.0f32, 1.0] {
        k.paint(
            bank,
            &mut m,
            Vec3::new(s * hw - 0.012, 0.7, z0 + 0.1),
            Vec3::new(s * hw + 0.012, 0.77, z1 - 0.1),
            GOLD,
        );
    }
    // Side windows (both sides) and the doors.
    for s in [-1.0f32, 1.0] {
        let x = s * (hw + 0.005);
        let mut zz = z0 + 0.25;
        while zz < z1 - 0.7 {
            let (a, b) = (zz, zz + 0.5);
            let q = if s > 0.0 {
                [
                    Vec3::new(x, 0.95, b),
                    Vec3::new(x, 0.95, a),
                    Vec3::new(x, 1.35, a),
                    Vec3::new(x, 1.35, b),
                ]
            } else {
                [
                    Vec3::new(x, 0.95, a),
                    Vec3::new(x, 0.95, b),
                    Vec3::new(x, 1.35, b),
                    Vec3::new(x, 1.35, a),
                ]
            };
            g.quad(q, UvRect::px(0, 0, 8, 6), glass);
            zz += 0.62;
        }
        // Door near the front.
        let (a, b) = (z1 - 0.62, z1 - 0.12);
        let q = if s > 0.0 {
            [
                Vec3::new(x, 0.3, b),
                Vec3::new(x, 0.3, a),
                Vec3::new(x, 1.35, a),
                Vec3::new(x, 1.35, b),
            ]
        } else {
            [
                Vec3::new(x, 0.3, a),
                Vec3::new(x, 0.3, b),
                Vec3::new(x, 1.35, b),
                Vec3::new(x, 1.35, a),
            ]
        };
        g.quad(q, UvRect::px(0, 0, 8, 16), glass);
    }
    // Windscreen, rear window, headlights, bumpers and the destination board.
    let q = |z: f32, flip: bool| {
        if flip {
            [
                Vec3::new(hw - 0.08, 0.92, z),
                Vec3::new(-hw + 0.08, 0.92, z),
                Vec3::new(-hw + 0.08, 1.36, z),
                Vec3::new(hw - 0.08, 1.36, z),
            ]
        } else {
            [
                Vec3::new(-hw + 0.08, 0.92, z),
                Vec3::new(hw - 0.08, 0.92, z),
                Vec3::new(hw - 0.08, 1.36, z),
                Vec3::new(-hw + 0.08, 1.36, z),
            ]
        }
    };
    g.quad(q(z1 + 0.005, false), UvRect::px(0, 0, 16, 8), glass);
    g.quad(q(z0 - 0.005, true), UvRect::px(0, 0, 16, 8), glass);
    for s in [-0.42f32, 0.42] {
        k.paint(
            bank,
            &mut m,
            Vec3::new(s - 0.1, 0.5, z1),
            Vec3::new(s + 0.1, 0.64, z1 + 0.04),
            CREAM,
        );
        k.paint(
            bank,
            &mut m,
            Vec3::new(s - 0.08, 0.5, z0 - 0.04),
            Vec3::new(s + 0.08, 0.6, z0),
            RED,
        );
    }
    for (z, dz) in [(z1, 0.1f32), (z0 - 0.1, 0.1)] {
        k.paint(
            bank,
            &mut m,
            Vec3::new(-hw - 0.03, 0.26, z),
            Vec3::new(hw + 0.03, 0.38, z + dz),
            SHADOW,
        );
    }
    k.paint(
        bank,
        &mut m,
        Vec3::new(-0.4, 1.38, z1),
        Vec3::new(0.4, 1.52, z1 + 0.03),
        INK,
    );
    k.paint(
        bank,
        &mut m,
        Vec3::new(-0.34, 1.41, z1 + 0.03),
        Vec3::new(0.34, 1.49, z1 + 0.04),
        GOLD,
    );
    // Wheel arches.
    for z in [z0 + 0.6, z1 - 0.95] {
        for s in [-1.0f32, 1.0] {
            k.paint(
                bank,
                &mut m,
                Vec3::new(s * hw - 0.02, 0.28, z - 0.3),
                Vec3::new(s * hw + 0.02, 0.36, z + 0.3),
                INK,
            );
        }
    }
    // One wheel, spinning about x.
    let tyre = bank.add(tiles::stripes(INK, SHADOW));
    let mut wheel = Mesh::new();
    lathe(
        &mut wheel,
        Vec3::new(0.0, -0.08, 0.0),
        &[(0.26, 0.0), (0.26, 0.16)],
        8,
        0.0,
        tyre,
        true,
    );
    let hub = k.solid(bank, SAND);
    skin_box(
        &mut wheel,
        Vec3::new(-0.1, -0.1, -0.1),
        Vec3::new(0.1, 0.1, 0.1),
        hub,
        &k.w4,
    );
    let mut w = Mesh::new();
    w.append(&wheel, Mat4::from_rotation_z(FRAC_PI_2));
    (m, g, w)
}

/// The bus shelter: 2 tiles wide, open to the south, with a bench and a round sign.
fn shelter(bank: &mut TexBank, k: &mut Kit, icons: &HashMap<&'static str, TexId>) -> Mesh {
    let wood = bank.add(tiles::boards([GOLD, CLAY, RUST], 88));
    let roof = bank.add(tiles::roof([AQUA, TEAL, DEEP_TEAL]));
    let plank = bank.add(tiles::planks(CLAY, GOLD, RUST, 89));
    let mut m = Mesh::new();
    let (x0, x1) = (-0.45, 1.45);
    // Back wall and posts.
    tiled_box(
        &mut m,
        Vec3::new(x0, 0.0, -0.45),
        Vec3::new(x1, 1.3, -0.35),
        wood,
        0,
    );
    for x in [x0, x1 - 0.08] {
        tiled_box(
            &mut m,
            Vec3::new(x, 0.0, 0.2),
            Vec3::new(x + 0.08, 1.3, 0.28),
            wood,
            0,
        );
        tiled_box(
            &mut m,
            Vec3::new(x, 0.0, -0.4),
            Vec3::new(x + 0.08, 1.3, 0.28),
            wood,
            1 << crate::render::mesh::TOP,
        );
    }
    slope(
        &mut m,
        Vec3::new(x0 - 0.15, 1.28, 0.45),
        Vec3::new(x1 + 0.15, 1.28, 0.45),
        Vec3::new(x1 + 0.15, 1.5, -0.55),
        Vec3::new(x0 - 0.15, 1.5, -0.55),
        roof,
    );
    // The bench.
    tiled_box(
        &mut m,
        Vec3::new(x0 + 0.15, 0.28, -0.32),
        Vec3::new(x1 - 0.15, 0.34, -0.02),
        plank,
        0,
    );
    for x in [x0 + 0.25, x1 - 0.3] {
        k.paint(
            bank,
            &mut m,
            Vec3::new(x, 0.0, -0.28),
            Vec3::new(x + 0.06, 0.28, -0.06),
            RUST,
        );
    }
    // A timetable poster.
    k.paint(
        bank,
        &mut m,
        Vec3::new(0.35, 0.55, -0.35),
        Vec3::new(0.75, 1.05, -0.33),
        CREAM,
    );
    for (i, c) in [TEAL, SHADOW, SHADOW, SHADOW].into_iter().enumerate() {
        let y = 0.95 - i as f32 * 0.1;
        k.paint(
            bank,
            &mut m,
            Vec3::new(0.4, y, -0.33),
            Vec3::new(0.7, y + 0.04, -0.32),
            c,
        );
    }
    // The round bus-stop sign on a pole, at the east end.
    let (sx, sz) = (1.55, 0.35);
    k.paint(
        bank,
        &mut m,
        Vec3::new(sx - 0.03, 0.0, sz - 0.03),
        Vec3::new(sx + 0.03, 1.55, sz + 0.03),
        SHADOW,
    );
    let disc = k.solid(bank, TEAL);
    let mut sign = Mesh::new();
    lathe(
        &mut sign,
        Vec3::ZERO,
        &[(0.24, -0.03), (0.24, 0.03)],
        12,
        0.0,
        disc,
        true,
    );
    m.append(
        &sign,
        Mat4::from_translation(Vec3::new(sx, 1.62, sz)) * Mat4::from_rotation_x(FRAC_PI_2),
    );
    if let Some(&t) = icons.get("bus") {
        let uv = UvRect::new(0.0, 0.0, 16.0, 16.0);
        front_quad(&mut m, sx - 0.16, 1.46, sx + 0.16, 1.78, sz + 0.035, uv, t);
        back_quad(&mut m, sx - 0.16, 1.46, sx + 0.16, 1.78, sz - 0.035, uv, t);
    }
    m
}

// ------------------------------------------------------------------------------------------
// Street furniture
// ------------------------------------------------------------------------------------------

/// The plaza fountain (3x3 tiles, anchored at its centre) and its water.
fn fountain(bank: &mut TexBank, k: &mut Kit, water: TexId) -> (Mesh, Mesh) {
    let stone = bank.add(tiles::bricks(SAND, WHITE, KHAKI, 90));
    let rim = bank.add(tiles::cobbles(CREAM, WHITE, KHAKI, 91));
    let mut m = Mesh::new();
    lathe(
        &mut m,
        Vec3::ZERO,
        &[(1.35, 0.0), (1.35, 0.42), (1.2, 0.42), (1.2, 0.2)],
        8,
        PI / 8.0,
        stone,
        false,
    );
    lathe(
        &mut m,
        Vec3::ZERO,
        &[(1.42, 0.38), (1.42, 0.48), (1.14, 0.48)],
        8,
        PI / 8.0,
        rim,
        false,
    );
    // The pillar, an upper bowl and a little golden bird on top.
    lathe(
        &mut m,
        Vec3::ZERO,
        &[
            (0.18, 0.0),
            (0.16, 1.0),
            (0.5, 1.12),
            (0.55, 1.26),
            (0.0, 1.22),
        ],
        8,
        0.0,
        stone,
        false,
    );
    lathe(
        &mut m,
        Vec3::ZERO,
        &[(0.12, 1.2), (0.1, 1.55), (0.0, 1.6)],
        6,
        0.0,
        stone,
        false,
    );
    k.paint(
        bank,
        &mut m,
        Vec3::new(-0.08, 1.6, -0.12),
        Vec3::new(0.08, 1.74, 0.12),
        GOLD,
    );
    k.paint(
        bank,
        &mut m,
        Vec3::new(-0.04, 1.7, 0.1),
        Vec3::new(0.04, 1.8, 0.18),
        GOLD,
    );
    // Water in the pool and the upper bowl, in the first water frame (which animates).
    let mut w = Mesh::new();
    disk(&mut w, Vec3::new(0.0, 0.34, 0.0), 1.2, 8, PI / 8.0, water);
    disk(&mut w, Vec3::new(0.0, 1.2, 0.0), 0.46, 8, 0.0, water);
    (m, w)
}

/// A flat, upward-facing polygon of `seg` sides.
fn disk(m: &mut Mesh, c: Vec3, r: f32, seg: usize, phase: f32, t: TexId) {
    let uv = |p: Vec3| Vec2::new(8.0 + (p.x - c.x) * TD, 8.0 + (p.z - c.z) * TD);
    for j in 0..seg {
        let a0 = phase + j as f32 / seg as f32 * TAU;
        let a1 = phase + (j + 1) as f32 / seg as f32 * TAU;
        let p0 = c + Vec3::new(a0.cos() * r, 0.0, a0.sin() * r);
        let p1 = c + Vec3::new(a1.cos() * r, 0.0, a1.sin() * r);
        // Wind so the normal points up.
        let (a, b) = if (p0 - c).cross(p1 - c).y > 0.0 {
            (p0, p1)
        } else {
            (p1, p0)
        };
        m.tri([c, a, b], [uv(c), uv(a), uv(b)], t);
    }
}

/// A street lamp: an iron post with a glass lantern (glowing at night).
fn street_lamp(bank: &mut TexBank, k: &mut Kit) -> (Mesh, Mesh) {
    let iron = k.solid(bank, SHADOW);
    let dark = k.solid(bank, INK);
    let mut m = Mesh::new();
    lathe(
        &mut m,
        Vec3::ZERO,
        &[(0.16, 0.0), (0.12, 0.12), (0.05, 0.2), (0.04, 1.45)],
        6,
        0.0,
        iron,
        false,
    );
    // Lantern frame and cap.
    skin_box(
        &mut m,
        Vec3::new(-0.15, 1.42, -0.15),
        Vec3::new(0.15, 1.47, 0.15),
        dark,
        &k.w4,
    );
    lathe(
        &mut m,
        Vec3::new(0.0, 1.85, 0.0),
        &[(0.22, 0.0), (0.0, 0.18)],
        4,
        PI / 4.0,
        dark,
        true,
    );
    for (x, z) in [(-0.13, -0.13), (0.11, -0.13), (-0.13, 0.11), (0.11, 0.11)] {
        skin_box(
            &mut m,
            Vec3::new(x, 1.47, z),
            Vec3::new(x + 0.02, 1.85, z + 0.02),
            dark,
            &k.w4,
        );
    }
    let glass = bank.add(tiles::crystal([WHITE, CREAM, GOLD]));
    let mut g = Mesh::new();
    tiled_box(
        &mut g,
        Vec3::new(-0.11, 1.47, -0.11),
        Vec3::new(0.11, 1.84, 0.11),
        glass,
        0,
    );
    (m, g)
}

/// The request board: two posts, a little roof and notes pinned all over it.
fn board(bank: &mut TexBank, k: &mut Kit) -> Mesh {
    let wood = bank.add(tiles::planks(CLAY, GOLD, RUST, 92));
    let cork = bank.add(tiles::plaster([GOLD, CLAY, RUST], 93));
    let roof = bank.add(tiles::roof([SALMON, CRIMSON, PLUM]));
    let mut m = Mesh::new();
    for x in [-0.42f32, 0.36] {
        tiled_box(
            &mut m,
            Vec3::new(x, 0.0, -0.05),
            Vec3::new(x + 0.07, 1.5, 0.03),
            wood,
            0,
        );
    }
    tiled_box(
        &mut m,
        Vec3::new(-0.45, 0.55, -0.03),
        Vec3::new(0.45, 1.3, 0.02),
        cork,
        0,
    );
    slope(
        &mut m,
        Vec3::new(-0.55, 1.42, 0.22),
        Vec3::new(0.55, 1.42, 0.22),
        Vec3::new(0.55, 1.62, -0.12),
        Vec3::new(-0.55, 1.62, -0.12),
        roof,
    );
    let notes = [
        (-0.32, 0.95, CREAM),
        (-0.05, 1.05, WHITE),
        (0.2, 0.9, SKY),
        (-0.22, 0.66, BLUSH),
        (0.12, 0.64, CREAM),
    ];
    for (x, y, c) in notes {
        k.paint(
            bank,
            &mut m,
            Vec3::new(x, y, 0.02),
            Vec3::new(x + 0.2, y + 0.2, 0.03),
            c,
        );
        k.paint(
            bank,
            &mut m,
            Vec3::new(x + 0.08, y + 0.16, 0.03),
            Vec3::new(x + 0.12, y + 0.2, 0.045),
            RED,
        );
    }
    m
}

/// Wooden flower beds: bare soil, then three colours of flowers.
fn planters(bank: &mut TexBank, k: &mut Kit) -> Vec<Mesh> {
    let wood = bank.add(tiles::planks(CLAY, GOLD, RUST, 94));
    let soil = bank.add(tiles::soil(95));
    let leaf = k.solid(bank, GREEN);
    let mut out = Vec::new();
    for v in 0..4 {
        let mut m = Mesh::new();
        tiled_box(
            &mut m,
            Vec3::new(-0.44, 0.0, -0.44),
            Vec3::new(0.44, 0.32, 0.44),
            wood,
            1 << crate::render::mesh::TOP,
        );
        m.quad(
            [
                Vec3::new(-0.4, 0.28, 0.4),
                Vec3::new(0.4, 0.28, 0.4),
                Vec3::new(0.4, 0.28, -0.4),
                Vec3::new(-0.4, 0.28, -0.4),
            ],
            UvRect::new(0.0, 0.0, 13.0, 13.0),
            soil,
        );
        if v > 0 {
            let petals = [[PINK, BLUSH], [GOLD, CREAM], [SKY, LAVENDER]][v - 1];
            for i in 0..6 {
                let a = i as f32 * 1.05 + v as f32;
                let r = if i % 2 == 0 { 0.22 } else { 0.1 };
                let (x, z) = (a.cos() * r, a.sin() * r);
                skin_box(
                    &mut m,
                    Vec3::new(x - 0.03, 0.28, z - 0.03),
                    Vec3::new(x + 0.03, 0.46, z + 0.03),
                    leaf,
                    &k.w4,
                );
                k.paint(
                    bank,
                    &mut m,
                    Vec3::new(x - 0.08, 0.46, z - 0.08),
                    Vec3::new(x + 0.08, 0.56, z + 0.08),
                    petals[i % 2],
                );
            }
        } else {
            for i in 0..3 {
                let x = -0.2 + i as f32 * 0.2;
                k.paint(
                    bank,
                    &mut m,
                    Vec3::new(x - 0.02, 0.28, -0.02),
                    Vec3::new(x + 0.02, 0.36, 0.02),
                    KHAKI,
                );
            }
        }
        out.push(m);
    }
    out
}

/// Round bushes: plain, flowering and berried.
fn bushes(bank: &mut TexBank, k: &mut Kit) -> Vec<Mesh> {
    let leaves = bank.add(tiles::canopy([LIME, GREEN, TEAL, DEEP_TEAL], 96));
    let mut out = Vec::new();
    for v in 0..3 {
        let mut m = Mesh::new();
        lathe(
            &mut m,
            Vec3::ZERO,
            &[
                (0.0, 0.0),
                (0.4, 0.05),
                (0.46, 0.3),
                (0.34, 0.55),
                (0.0, 0.62),
            ],
            7,
            v as f32,
            leaves,
            false,
        );
        if v > 0 {
            let c = if v == 1 { PINK } else { RED };
            for i in 0..7 {
                let a = i as f32 * 0.9 + 0.3;
                let y = 0.2 + (i % 3) as f32 * 0.12;
                let r = 0.44 - (y - 0.2) * 0.3;
                let p = Vec3::new(a.cos() * r, y, a.sin() * r);
                k.paint(
                    bank,
                    &mut m,
                    p - Vec3::splat(0.04),
                    p + Vec3::splat(0.04),
                    c,
                );
            }
        }
        out.push(m);
    }
    out
}

fn well(bank: &mut TexBank, k: &mut Kit) -> Mesh {
    let stone = bank.add(tiles::bricks(KHAKI, SAND, SHADOW, 97));
    let wood = bank.add(tiles::planks(CLAY, GOLD, RUST, 98));
    let roof = bank.add(tiles::roof([SALMON, CRIMSON, PLUM]));
    let dark = k.solid(bank, INK);
    let mut m = Mesh::new();
    lathe(
        &mut m,
        Vec3::ZERO,
        &[(0.42, 0.0), (0.42, 0.5), (0.32, 0.5), (0.32, 0.3)],
        8,
        0.2,
        stone,
        false,
    );
    m.quad(
        [
            Vec3::new(-0.3, 0.32, 0.3),
            Vec3::new(0.3, 0.32, 0.3),
            Vec3::new(0.3, 0.32, -0.3),
            Vec3::new(-0.3, 0.32, -0.3),
        ],
        UvRect::px(0, 0, 4, 4),
        dark,
    );
    for x in [-0.38f32, 0.32] {
        tiled_box(
            &mut m,
            Vec3::new(x, 0.3, -0.04),
            Vec3::new(x + 0.06, 1.2, 0.04),
            wood,
            0,
        );
    }
    tiled_box(
        &mut m,
        Vec3::new(-0.38, 0.9, -0.03),
        Vec3::new(0.38, 0.95, 0.03),
        wood,
        0,
    );
    k.paint(
        bank,
        &mut m,
        Vec3::new(-0.08, 0.62, -0.08),
        Vec3::new(0.08, 0.78, 0.08),
        CLAY,
    );
    slope(
        &mut m,
        Vec3::new(-0.5, 1.1, 0.35),
        Vec3::new(0.5, 1.1, 0.35),
        Vec3::new(0.5, 1.35, 0.0),
        Vec3::new(-0.5, 1.35, 0.0),
        roof,
    );
    slope(
        &mut m,
        Vec3::new(0.5, 1.1, -0.35),
        Vec3::new(-0.5, 1.1, -0.35),
        Vec3::new(-0.5, 1.35, 0.0),
        Vec3::new(0.5, 1.35, 0.0),
        roof,
    );
    m
}

/// Little market stands with striped awnings and heaps of produce.
fn stands(bank: &mut TexBank, k: &mut Kit) -> Vec<Mesh> {
    let wood = bank.add(tiles::planks(CLAY, GOLD, RUST, 99));
    let mut out = Vec::new();
    for (i, (a, b)) in [(RED, WHITE), (TEAL, CREAM), (GOLD, WHITE), (PURPLE, BLUSH)]
        .into_iter()
        .enumerate()
    {
        let cloth = bank.add(tiles::stripes(a, b));
        let mut m = Mesh::new();
        tiled_box(
            &mut m,
            Vec3::new(-0.44, 0.0, -0.2),
            Vec3::new(0.44, 0.5, 0.3),
            wood,
            0,
        );
        for x in [-0.42f32, 0.36] {
            tiled_box(
                &mut m,
                Vec3::new(x, 0.5, -0.24),
                Vec3::new(x + 0.06, 1.3, -0.18),
                wood,
                0,
            );
        }
        slope(
            &mut m,
            Vec3::new(-0.55, 1.15, 0.42),
            Vec3::new(0.55, 1.15, 0.42),
            Vec3::new(0.55, 1.35, -0.28),
            Vec3::new(-0.55, 1.35, -0.28),
            cloth,
        );
        let goods = [
            [RED, LIME, GOLD],
            [ORANGE, GREEN, CREAM],
            [PINK, SKY, GOLD],
            [LAVENDER, RED, LIME],
        ][i];
        for (j, c) in goods.into_iter().enumerate() {
            let x = -0.28 + j as f32 * 0.28;
            for (dx, dz, dy) in [(0.0, 0.0, 0.0), (0.07, 0.06, 0.0), (0.03, 0.02, 0.07)] {
                let p = Vec3::new(x + dx, 0.55 + dy, 0.05 + dz);
                k.paint(
                    bank,
                    &mut m,
                    p - Vec3::splat(0.05),
                    p + Vec3::splat(0.05),
                    c,
                );
            }
        }
        out.push(m);
    }
    out
}

fn barrel(bank: &mut TexBank, k: &mut Kit) -> Mesh {
    let wood = bank.add(tiles::boards([GOLD, CLAY, RUST], 100));
    let band = k.solid(bank, SHADOW);
    let mut m = Mesh::new();
    lathe(
        &mut m,
        Vec3::ZERO,
        &[(0.26, 0.0), (0.31, 0.3), (0.26, 0.62)],
        8,
        0.0,
        wood,
        true,
    );
    for y in [0.12f32, 0.48] {
        lathe(
            &mut m,
            Vec3::ZERO,
            &[(0.3, y), (0.3, y + 0.04)],
            8,
            0.0,
            band,
            false,
        );
    }
    m
}

// ------------------------------------------------------------------------------------------
// Shop fittings
// ------------------------------------------------------------------------------------------

struct Fittings {
    counter: Mesh,
    shelf: Mesh,
    rack: Mesh,
    dummy_base: Mesh,
    dummy_wood: TexId,
    tables: Vec<Mesh>,
    stool: Mesh,
    hearth: Mesh,
    fixtures: Vec<Mesh>,
    window: Mesh,
    window_glass: Mesh,
    paintings: Vec<Mesh>,
    sconce: Mesh,
}

fn fittings(bank: &mut TexBank, k: &mut Kit) -> Fittings {
    let plank = bank.add(tiles::planks(CLAY, GOLD, RUST, 110));
    let dark = bank.add(tiles::planks(RUST, CLAY, MAROON, 111));
    let panel = bank.add(tiles::boards([CLAY, RUST, MAROON], 112));
    let stone = bank.add(tiles::bricks(KHAKI, SAND, SHADOW, 113));
    let brick = bank.add(tiles::bricks(RUST, CLAY, MAROON, 114));
    let glass = bank.add(tiles::crystal([WHITE, SKY, AQUA]));

    // Counter: a panelled front with a polished top; tiles join into one long counter.
    let mut counter = Mesh::new();
    tiled_box(
        &mut counter,
        Vec3::new(-0.5, 0.0, -0.3),
        Vec3::new(0.5, 0.66, 0.3),
        panel,
        0,
    );
    tiled_box(
        &mut counter,
        Vec3::new(-0.5, 0.66, -0.34),
        Vec3::new(0.5, 0.74, 0.36),
        plank,
        0,
    );

    // Shelves: a tall open case against the back wall with three boards.
    let mut shelf = Mesh::new();
    tiled_box(
        &mut shelf,
        Vec3::new(-0.46, 0.0, -0.46),
        Vec3::new(0.46, 1.62, -0.42),
        dark,
        0,
    );
    for x in [-0.46f32, 0.4] {
        tiled_box(
            &mut shelf,
            Vec3::new(x, 0.0, -0.46),
            Vec3::new(x + 0.06, 1.62, -0.06),
            dark,
            0,
        );
    }
    for y in [0.0f32, 0.5, 1.0, 1.52] {
        tiled_box(
            &mut shelf,
            Vec3::new(-0.46, y, -0.46),
            Vec3::new(0.46, y + 0.06, -0.06),
            plank,
            0,
        );
    }

    // Weapon rack: a frame with pegs; the weapons are drawn into it.
    let mut rack = Mesh::new();
    for x in [-0.44f32, 0.38] {
        tiled_box(
            &mut rack,
            Vec3::new(x, 0.0, -0.4),
            Vec3::new(x + 0.06, 1.3, -0.3),
            dark,
            0,
        );
    }
    for y in [0.15f32, 1.1] {
        tiled_box(
            &mut rack,
            Vec3::new(-0.44, y, -0.42),
            Vec3::new(0.44, y + 0.07, -0.3),
            plank,
            0,
        );
    }

    // Mannequin stand.
    let mut dummy_base = Mesh::new();
    lathe(
        &mut dummy_base,
        Vec3::ZERO,
        &[(0.26, 0.0), (0.26, 0.05), (0.04, 0.08), (0.03, 0.2)],
        8,
        0.0,
        dark,
        true,
    );
    let dummy_wood = bank.add(tiles::planks(GOLD, CREAM, CLAY, 115));

    // Tables: a round table with a cloth, and a plain tavern table with mugs.
    let cloth = bank.add(tiles::checker(WHITE, SALMON, BLUSH));
    let mut t0 = Mesh::new();
    lathe(
        &mut t0,
        Vec3::ZERO,
        &[(0.06, 0.0), (0.06, 0.52)],
        6,
        0.0,
        dark,
        false,
    );
    lathe(
        &mut t0,
        Vec3::ZERO,
        &[(0.42, 0.42), (0.44, 0.56), (0.0, 0.56)],
        8,
        0.0,
        cloth,
        false,
    );
    let mut t1 = Mesh::new();
    tiled_box(
        &mut t1,
        Vec3::new(-0.42, 0.5, -0.36),
        Vec3::new(0.42, 0.58, 0.36),
        plank,
        0,
    );
    for (x, z) in [(-0.34, -0.28), (0.3, -0.28), (-0.34, 0.24), (0.3, 0.24)] {
        tiled_box(
            &mut t1,
            Vec3::new(x, 0.0, z),
            Vec3::new(x + 0.06, 0.5, z + 0.06),
            dark,
            0,
        );
    }
    for (x, z) in [(-0.15, 0.05), (0.18, -0.1)] {
        lathe(
            &mut t1,
            Vec3::new(x, 0.58, z),
            &[(0.06, 0.0), (0.06, 0.14)],
            6,
            0.0,
            k.solid(bank, SAND),
            true,
        );
        k.paint(
            bank,
            &mut t1,
            Vec3::new(x - 0.05, 0.72, z - 0.05),
            Vec3::new(x + 0.05, 0.76, z + 0.05),
            WHITE,
        );
    }
    let mut stool = Mesh::new();
    lathe(
        &mut stool,
        Vec3::ZERO,
        &[(0.14, 0.0), (0.08, 0.08), (0.07, 0.28)],
        8,
        0.0,
        plank,
        false,
    );
    let cushion = bank.add(tiles::checker(SALMON, CRIMSON, PLUM));
    lathe(
        &mut stool,
        Vec3::ZERO,
        &[(0.2, 0.28), (0.22, 0.36), (0.0, 0.4)],
        8,
        0.0,
        cushion,
        false,
    );

    // Hearth: a stone fireplace set into the back wall (flames are added when drawn).
    let mut hearth = Mesh::new();
    tiled_box(
        &mut hearth,
        Vec3::new(-0.5, 0.0, -0.5),
        Vec3::new(-0.3, 1.3, -0.1),
        stone,
        0,
    );
    tiled_box(
        &mut hearth,
        Vec3::new(0.3, 0.0, -0.5),
        Vec3::new(0.5, 1.3, -0.1),
        stone,
        0,
    );
    tiled_box(
        &mut hearth,
        Vec3::new(-0.55, 1.0, -0.5),
        Vec3::new(0.55, 1.3, -0.05),
        stone,
        0,
    );
    tiled_box(
        &mut hearth,
        Vec3::new(-0.6, 1.3, -0.5),
        Vec3::new(0.6, 1.38, 0.0),
        dark,
        0,
    );
    k.paint(
        bank,
        &mut hearth,
        Vec3::new(-0.3, 0.0, -0.5),
        Vec3::new(0.3, 1.0, -0.45),
        INK,
    );
    tiled_box(
        &mut hearth,
        Vec3::new(-0.25, 0.0, -0.35),
        Vec3::new(0.25, 0.08, -0.15),
        dark,
        0,
    );

    let mut fixtures = Vec::new();
    // 0: the bakery oven, a brick dome with a glowing mouth.
    let mut oven = Mesh::new();
    lathe(
        &mut oven,
        Vec3::new(0.0, 0.0, -0.1),
        &[
            (0.45, 0.0),
            (0.45, 0.5),
            (0.36, 0.85),
            (0.18, 1.0),
            (0.0, 1.04),
        ],
        8,
        0.0,
        brick,
        false,
    );
    k.paint(
        bank,
        &mut oven,
        Vec3::new(-0.18, 0.2, 0.26),
        Vec3::new(0.18, 0.5, 0.36),
        ORANGE,
    );
    k.paint(
        bank,
        &mut oven,
        Vec3::new(-0.12, 0.2, 0.36),
        Vec3::new(0.12, 0.3, 0.37),
        GOLD,
    );
    fixtures.push(oven);
    // 1: an anvil on a stump.
    let mut anvil = Mesh::new();
    lathe(
        &mut anvil,
        Vec3::ZERO,
        &[(0.26, 0.0), (0.24, 0.4)],
        7,
        0.0,
        bank.add(tiles::bark()),
        true,
    );
    let steel = k.solid(bank, SLATE);
    skin_box(
        &mut anvil,
        Vec3::new(-0.12, 0.4, -0.1),
        Vec3::new(0.12, 0.56, 0.1),
        steel,
        &k.w4,
    );
    skin_box(
        &mut anvil,
        Vec3::new(-0.3, 0.56, -0.13),
        Vec3::new(0.26, 0.7, 0.13),
        steel,
        &k.w4,
    );
    skin_box(
        &mut anvil,
        Vec3::new(0.26, 0.6, -0.06),
        Vec3::new(0.4, 0.7, 0.06),
        steel,
        &k.w4,
    );
    fixtures.push(anvil);
    // 2: a bubbling cauldron.
    let mut pot = Mesh::new();
    let iron = k.solid(bank, INK);
    lathe(
        &mut pot,
        Vec3::ZERO,
        &[
            (0.2, 0.05),
            (0.38, 0.2),
            (0.4, 0.4),
            (0.34, 0.55),
            (0.3, 0.5),
        ],
        8,
        0.0,
        iron,
        false,
    );
    let brew = bank.add(tiles::crystal([MINT, AQUA, TEAL]));
    lathe(
        &mut pot,
        Vec3::new(0.0, 0.5, 0.0),
        &[(0.3, 0.0), (0.0, 0.02)],
        8,
        0.0,
        brew,
        false,
    );
    for a in [0.3f32, 2.4, 4.4] {
        skin_box(
            &mut pot,
            Vec3::new(a.cos() * 0.25 - 0.04, 0.0, a.sin() * 0.25 - 0.04),
            Vec3::new(a.cos() * 0.25 + 0.04, 0.1, a.sin() * 0.25 + 0.04),
            iron,
            &k.w4,
        );
    }
    fixtures.push(pot);
    // 3: a glass case of gems.
    let mut case = Mesh::new();
    tiled_box(
        &mut case,
        Vec3::new(-0.45, 0.0, -0.3),
        Vec3::new(0.45, 0.55, 0.3),
        dark,
        0,
    );
    tiled_box(
        &mut case,
        Vec3::new(-0.42, 0.55, -0.27),
        Vec3::new(0.42, 0.82, 0.27),
        glass,
        0,
    );
    for (i, c) in [RED, SKY, LIME, LAVENDER, GOLD].into_iter().enumerate() {
        let x = -0.3 + i as f32 * 0.15;
        let z = if i % 2 == 0 { -0.08 } else { 0.08 };
        k.paint(
            bank,
            &mut case,
            Vec3::new(x - 0.04, 0.56, z - 0.04),
            Vec3::new(x + 0.04, 0.64, z + 0.04),
            c,
        );
    }
    fixtures.push(case);
    // 4: a tinker's bench with a vice and cogs.
    let mut bench = Mesh::new();
    tiled_box(
        &mut bench,
        Vec3::new(-0.45, 0.5, -0.4),
        Vec3::new(0.45, 0.6, 0.1),
        plank,
        0,
    );
    for x in [-0.4f32, 0.34] {
        tiled_box(
            &mut bench,
            Vec3::new(x, 0.0, -0.35),
            Vec3::new(x + 0.06, 0.5, 0.05),
            dark,
            0,
        );
    }
    let cog = k.solid(bank, GOLD);
    for (x, r) in [(-0.2f32, 0.12f32), (0.15, 0.08)] {
        let mut c = Mesh::new();
        lathe(
            &mut c,
            Vec3::ZERO,
            &[(r, -0.02), (r, 0.02)],
            8,
            0.0,
            cog,
            true,
        );
        bench.append(
            &c,
            Mat4::from_translation(Vec3::new(x, 0.6 + r, -0.2)) * Mat4::from_rotation_x(FRAC_PI_2),
        );
    }
    skin_box(
        &mut bench,
        Vec3::new(0.25, 0.6, -0.05),
        Vec3::new(0.4, 0.75, 0.08),
        steel,
        &k.w4,
    );
    fixtures.push(bench);
    // 5: seed bins, three open crates of seeds.
    let mut bins = Mesh::new();
    for (i, c) in [GOLD, RUST, LIME].into_iter().enumerate() {
        let x = -0.3 + i as f32 * 0.3;
        tiled_box(
            &mut bins,
            Vec3::new(x - 0.14, 0.0, -0.2),
            Vec3::new(x + 0.14, 0.45 - i as f32 * 0.05, 0.2),
            plank,
            1 << crate::render::mesh::TOP,
        );
        let seeds = bank.add(tiles::plaster([CREAM, c, SHADOW], 120 + i as u64));
        let top = 0.4 - i as f32 * 0.05;
        bins.quad(
            [
                Vec3::new(x - 0.13, top, 0.19),
                Vec3::new(x + 0.13, top, 0.19),
                Vec3::new(x + 0.13, top, -0.19),
                Vec3::new(x - 0.13, top, -0.19),
            ],
            UvRect::new(0.0, 0.0, 4.0, 6.0),
            seeds,
        );
    }
    fixtures.push(bins);
    // 6: a bookcase full of books.
    let mut books = Mesh::new();
    books.append(&shelf, Mat4::IDENTITY);
    let spines = [RED, BLUE, GREEN, GOLD, PURPLE, TEAL, CRIMSON, INDIGO];
    for (row, y) in [0.06f32, 0.56, 1.06].into_iter().enumerate() {
        let mut x = -0.4;
        let mut i = row * 3;
        while x < 0.36 {
            let wdt = 0.07 + (i % 3) as f32 * 0.015;
            let tall = 0.3 + (i % 4) as f32 * 0.04;
            k.paint(
                bank,
                &mut books,
                Vec3::new(x, y, -0.42),
                Vec3::new(x + wdt, y + tall, -0.12),
                spines[i % spines.len()],
            );
            x += wdt + 0.01;
            i += 1;
        }
    }
    fixtures.push(books);
    // 7: the mayor's desk with papers and a quill.
    let mut desk = Mesh::new();
    tiled_box(
        &mut desk,
        Vec3::new(-0.5, 0.0, -0.32),
        Vec3::new(0.5, 0.62, 0.32),
        panel,
        0,
    );
    tiled_box(
        &mut desk,
        Vec3::new(-0.52, 0.62, -0.34),
        Vec3::new(0.52, 0.68, 0.34),
        plank,
        0,
    );
    k.paint(
        bank,
        &mut desk,
        Vec3::new(-0.3, 0.68, -0.15),
        Vec3::new(0.0, 0.7, 0.12),
        WHITE,
    );
    k.paint(
        bank,
        &mut desk,
        Vec3::new(0.2, 0.68, -0.1),
        Vec3::new(0.24, 0.9, -0.06),
        WHITE,
    );
    fixtures.push(desk);
    // 8: barrels with taps behind the bar.
    let mut taps = Mesh::new();
    let cask = bank.add(tiles::boards([GOLD, CLAY, RUST], 116));
    let mut c = Mesh::new();
    lathe(
        &mut c,
        Vec3::ZERO,
        &[(0.2, -0.3), (0.26, 0.0), (0.2, 0.3)],
        8,
        0.0,
        cask,
        true,
    );
    for y in [0.28f32, 0.78] {
        taps.append(
            &c,
            Mat4::from_translation(Vec3::new(0.0, y, -0.2)) * Mat4::from_rotation_x(FRAC_PI_2),
        );
        k.paint(
            bank,
            &mut taps,
            Vec3::new(-0.03, y - 0.06, 0.1),
            Vec3::new(0.03, y, 0.18),
            GOLD,
        );
    }
    tiled_box(
        &mut taps,
        Vec3::new(-0.4, 0.0, -0.45),
        Vec3::new(0.4, 0.05, 0.05),
        dark,
        0,
    );
    fixtures.push(taps);
    // 9: a potted plant.
    let mut plant = Mesh::new();
    let clay = k.solid(bank, CLAY);
    lathe(
        &mut plant,
        Vec3::ZERO,
        &[(0.14, 0.0), (0.2, 0.3), (0.0, 0.3)],
        6,
        0.0,
        clay,
        false,
    );
    let leaves = bank.add(tiles::canopy([LIME, GREEN, TEAL, DEEP_TEAL], 117));
    lathe(
        &mut plant,
        Vec3::ZERO,
        &[(0.0, 0.3), (0.3, 0.5), (0.34, 0.8), (0.2, 1.05), (0.0, 1.1)],
        6,
        0.4,
        leaves,
        false,
    );
    fixtures.push(plant);

    // A window in the back wall, a few paintings and a wall lamp.
    let mut window = Mesh::new();
    let frame = k.solid(bank, CREAM);
    skin_box(
        &mut window,
        Vec3::new(-0.36, 0.75, -0.02),
        Vec3::new(0.36, 1.55, 0.02),
        frame,
        &k.w4,
    );
    let curtain = bank.add(tiles::stripes(PINK, BLUSH));
    for x in [-0.42f32, 0.3] {
        tiled_box(
            &mut window,
            Vec3::new(x, 0.7, 0.0),
            Vec3::new(x + 0.12, 1.62, 0.04),
            curtain,
            0,
        );
    }
    let mut window_glass = Mesh::new();
    front_quad(
        &mut window_glass,
        -0.3,
        0.8,
        0.3,
        1.5,
        0.025,
        UvRect::px(0, 0, 10, 11),
        bank.add(tiles::crystal([WHITE, SKY, BLUE])),
    );
    let mut paintings = Vec::new();
    for pal in [
        [SKY, GREEN, LIME],
        [INDIGO, CREAM, GOLD],
        [PEACH, SALMON, CRIMSON],
    ] {
        let mut p = Mesh::new();
        let gold = k.solid(bank, GOLD);
        skin_box(
            &mut p,
            Vec3::new(-0.32, 0.95, -0.02),
            Vec3::new(0.32, 1.45, 0.02),
            gold,
            &k.w4,
        );
        let art = bank.add(landscape(pal));
        front_quad(
            &mut p,
            -0.27,
            1.0,
            0.27,
            1.4,
            0.025,
            UvRect::new(0.0, 0.0, 16.0, 16.0),
            art,
        );
        paintings.push(p);
    }
    let mut sconce = Mesh::new();
    k.paint(
        bank,
        &mut sconce,
        Vec3::new(-0.06, 1.1, -0.02),
        Vec3::new(0.06, 1.3, 0.08),
        SHADOW,
    );
    k.paint(
        bank,
        &mut sconce,
        Vec3::new(-0.08, 1.3, 0.0),
        Vec3::new(0.08, 1.46, 0.14),
        CREAM,
    );
    Fittings {
        counter,
        shelf,
        rack,
        dummy_base,
        dummy_wood,
        tables: vec![t0, t1],
        stool,
        hearth,
        fixtures,
        window,
        window_glass,
        paintings,
        sconce,
    }
}

/// A tiny landscape for a painting: sky, hills and a sun.
fn landscape(pal: [u8; 3]) -> Texture {
    let [sky, hill, sun] = pal;
    let mut t = Texture::new(16, 16, sky);
    for x in 0..16 {
        let h = 10 + ((x as f32 * 0.6).sin() * 2.0) as i32;
        for y in h..16 {
            t.set(x, y, if y == h { WHITE } else { hill });
        }
    }
    for (x, y) in [(11, 3), (12, 3), (11, 4), (12, 4)] {
        t.set(x, y, sun);
    }
    t
}

/// A rug with a border and a woven centre.
fn rug_tex(pal: [u8; 3]) -> Texture {
    let [base, pattern, edge] = pal;
    let mut t = Texture::new(16, 16, base);
    for y in 0..16 {
        for x in 0..16 {
            let border = x < 2 || y < 2 || x > 13 || y > 13;
            if border {
                t.set(x, y, if (x + y) % 2 == 0 { edge } else { pattern });
            } else if (x - 8i32).abs() + (y - 8i32).abs() == 3 {
                t.set(x, y, pattern);
            }
        }
    }
    t
}

/// Festival bunting: a string of little triangular flags.
fn bunting_tex() -> Texture {
    let mut t = Texture::clear(16, 8);
    let cols = [RED, GOLD, SKY, LIME];
    for x in 0..16 {
        t.set(x, 0, SHADOW);
    }
    for (f, &c) in cols.iter().enumerate() {
        for y in 1..6 {
            let half = (6 - y) / 2;
            for x in (f as i32 * 4 + 2 - half)..=(f as i32 * 4 + 1 + half) {
                t.set(x, y, c);
            }
        }
    }
    t
}

/// Where the wheels sit under the bus, in its own space.
pub const WHEELS: [Vec3; 4] = [
    Vec3::new(-0.62, 0.26, 1.1),
    Vec3::new(0.62, 0.26, 1.1),
    Vec3::new(-0.62, 0.26, -1.1),
    Vec3::new(0.62, 0.26, -1.1),
];

/// A turn of the wheel for a distance travelled.
pub fn wheel_turn(dist: f32) -> f32 {
    (dist / 0.26) % TAU
}
