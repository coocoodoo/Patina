//! Things for going off the beaten path in the Hollow: bombs, cracked floors, the holes they
//! blast open, and the rope back up out of a secret room.

use glam::{Mat4, Vec3};

use super::models::{lathe, skin_box, tiled_box};
use super::tiles;
use crate::palette::*;
use crate::render::{Mesh, TexBank, TexId, Texture};

pub struct DelveArt {
    /// A round black bomb with a fuse (the fuse's tip at `FUSE_TIP`).
    pub bomb: Mesh,
    /// A wooden stake hammered in by a hole, with a rope tied on and hanging down it.
    pub stake: Mesh,
    /// A rope hanging from high above, down to the floor.
    pub rope: Mesh,
    /// Cracks across a floor tile (clear where the floor shows through).
    pub crack: TexId,
    /// A dark hole with a broken rim.
    pub pit: TexId,
}

/// Where a bomb's fuse ends, in the bomb's own space.
pub const FUSE_TIP: Vec3 = Vec3::new(0.05, 0.43, 0.0);

fn crack_tex() -> Texture {
    let mut t = Texture::new(16, 16, CLEAR);
    // Jagged lines running out from near the middle.
    let lines: [&[(i32, i32)]; 4] = [
        &[(7, 7), (6, 6), (5, 6), (4, 5), (3, 5), (2, 4), (1, 3)],
        &[
            (8, 8),
            (9, 9),
            (9, 10),
            (10, 11),
            (11, 11),
            (12, 12),
            (12, 13),
            (13, 14),
        ],
        &[(8, 7), (9, 6), (10, 5), (10, 4), (11, 3), (12, 2)],
        &[(7, 8), (6, 9), (5, 10), (5, 11), (4, 12), (3, 13)],
    ];
    for line in lines {
        for &(x, y) in line {
            t.set(x, y, INK);
        }
    }
    for (x, y) in [(7, 7), (8, 7), (7, 8), (8, 8)] {
        t.set(x, y, INK);
    }
    // Chips of lighter stone knocked loose along the cracks.
    for (x, y) in [(5, 5), (10, 10), (11, 4), (4, 11), (8, 6)] {
        t.set(x, y, SHADOW);
    }
    t
}

fn pit_tex() -> Texture {
    let mut t = Texture::new(16, 16, CLEAR);
    for y in 0..16 {
        for x in 0..16 {
            let (dx, dy) = (x as f32 - 7.5, y as f32 - 7.5);
            let d = (dx * dx + dy * dy).sqrt() + ((x * 7 + y * 3) % 5) as f32 * 0.25;
            let c = if d < 5.2 {
                INK
            } else if d < 6.4 {
                SHADOW
            } else if d < 7.6 {
                if (x + y) % 3 == 0 { KHAKI } else { ROSEWOOD }
            } else {
                continue;
            };
            t.set(x, y, c);
        }
    }
    t
}

pub fn build(bank: &mut TexBank) -> DelveArt {
    let w4 = Texture::new(4, 4, 0);
    // The bomb: a round shell, a brass cap and a curl of fuse.
    let mut shell = Texture::new(8, 8, SHADOW);
    for (x, y) in [(1, 1), (2, 1), (1, 2)] {
        shell.set(x, y, SLATE);
    }
    shell.set(2, 2, WHITE);
    for (x, y) in [(5, 6), (6, 5), (6, 6), (4, 7), (7, 4)] {
        shell.set(x, y, INK);
    }
    let shell = bank.add(shell);
    let brass = bank.add(tiles::solid(GOLD));
    let cord = bank.add(tiles::solid(RUST));
    let mut bomb = Mesh::new();
    lathe(
        &mut bomb,
        Vec3::ZERO,
        &[
            (0.0, 0.0),
            (0.12, 0.03),
            (0.17, 0.13),
            (0.16, 0.24),
            (0.1, 0.31),
            (0.0, 0.33),
        ],
        8,
        0.2,
        shell,
        true,
    );
    skin_box(
        &mut bomb,
        Vec3::new(-0.05, 0.3, -0.05),
        Vec3::new(0.05, 0.36, 0.05),
        brass,
        &w4,
    );
    skin_box(
        &mut bomb,
        Vec3::new(-0.015, 0.36, -0.015),
        Vec3::new(0.015, 0.41, 0.015),
        cord,
        &w4,
    );
    skin_box(
        &mut bomb,
        Vec3::new(0.0, 0.39, -0.015),
        Vec3::new(0.06, 0.42, 0.015),
        cord,
        &w4,
    );

    // A stake by a hole, and the rope tied on and running over the rim.
    let wood = bank.add(tiles::planks(CLAY, GOLD, RUST, 31));
    let rope_tex = {
        let mut t = Texture::new(4, 4, SAND);
        t.set(0, 0, KHAKI);
        t.set(2, 2, KHAKI);
        t.set(1, 3, CREAM);
        bank.add(t)
    };
    let mut stake = Mesh::new();
    tiled_box(
        &mut stake,
        Vec3::new(-0.05, 0.0, -0.05),
        Vec3::new(0.05, 0.42, 0.05),
        wood,
        0,
    );
    skin_box(
        &mut stake,
        Vec3::new(-0.065, 0.28, -0.065),
        Vec3::new(0.065, 0.34, 0.065),
        rope_tex,
        &w4,
    );
    // From the stake's knot, over the rim and down into the dark.
    let mut strand = Mesh::new();
    skin_box(
        &mut strand,
        Vec3::new(-0.018, 0.0, -0.018),
        Vec3::new(0.018, 0.5, 0.018),
        rope_tex,
        &w4,
    );
    stake.append(
        &strand,
        Mat4::from_translation(Vec3::new(0.0, 0.3, 0.0)) * Mat4::from_rotation_x(1.9),
    );

    // The rope back up: a long strand from the roof, knotted every so often.
    let mut rope = Mesh::new();
    skin_box(
        &mut rope,
        Vec3::new(-0.025, 0.0, -0.025),
        Vec3::new(0.025, 2.6, 0.025),
        rope_tex,
        &w4,
    );
    for y in [0.35f32, 0.9, 1.45, 2.0] {
        skin_box(
            &mut rope,
            Vec3::new(-0.045, y, -0.045),
            Vec3::new(0.045, y + 0.07, 0.045),
            rope_tex,
            &w4,
        );
    }
    DelveArt {
        bomb,
        stake,
        rope,
        crack: bank.add(crack_tex()),
        pit: bank.add(pit_tex()),
    }
}
