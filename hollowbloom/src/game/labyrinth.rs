//! The Marble Labyrinth. Now and then a floor of the Hollow, from floor 8 down, is an old
//! maze of white marble: corridors three wide between walls banded in gold, a pier at every
//! corner, arches over the doorways, and mosaics and cracked flagstones underfoot. Braziers
//! of old green bronze and torches on the walls light the way; broken columns, statues and
//! rubble lie about, and the dead ends keep their treasure. Somewhere in it an open
//! courtyard lies in shafts of daylight, with a griffin's nest on a broken column in the
//! middle; minotaurs roam the corridors; and the stairs down are as long a walk from where
//! you come in as the maze allows. Past its outer walls there's nothing but the dark.

use super::dungeon::{
    CRACK_CHANCE, Foe, GLEAM_CHANCE, Level, Spawn, floor_foes, is_waystone_floor, too_tough,
};
use super::glowcave::{open, place_solid, walk};
use super::world::{Area, Floor, Obj, Wall, World};
use crate::util::{Rng, hash2};

/// How often a floor is the labyrinth. Never the first few floors, a guardian's floor, the
/// sewers or the glowcap caves.
pub const LABYRINTH_CHANCE: f32 = 0.08;
pub const FIRST_LABYRINTH: u32 = 8;

/// The maze is laid out in cells three tiles square, with walls a tile thick between them
/// and all round them.
pub const CELL: i32 = 4;
/// The dark round the maze.
pub const MARGIN: i32 = 2;

/// How a tile of marble wall looks (see `Wall::Marble`): plain blocks, cracked, mossy, a
/// niche, a gilded shield, or a pier where walls meet.
pub const PLAIN: u8 = 0;
pub const CRACKED: u8 = 1;
pub const MOSSY: u8 = 2;
pub const NICHE: u8 = 3;
pub const SHIELD: u8 = 4;
pub const PIER: u8 = 5;
pub const WALL_LOOKS: usize = 6;
/// A pier stands a little taller than the walls, with its capital on top.
pub const PIER_H: f32 = 1.125;

/// How a column stands (see `Obj::Column`): whole, broken off, or fallen in pieces.
pub const STANDING: u8 = 0;
pub const BROKEN: u8 = 1;
pub const FALLEN: u8 = 2;
/// Rubble: a few chunks you can walk over (0 and 1), or a heap you can't (2).
pub const HEAP: u8 = 2;

/// From this floor down, a pair of griffins keeps the courtyard.
pub const GRIFFIN_PAIR: u32 = 30;

/// How many minotaurs roam a labyrinth: more the deeper it lies.
pub fn minotaurs(depth: u32) -> usize {
    (1 + depth as usize / 25).min(3)
}

/// Is this floor the labyrinth?
pub fn is_labyrinth(seed: u64, depth: u32, biome: usize) -> bool {
    depth >= FIRST_LABYRINTH
        && !is_waystone_floor(depth)
        && !super::sewer::is_sewer(seed, depth)
        && !super::glowcave::is_glowcave(seed, depth, biome)
        && Rng::new(seed ^ (depth as u64).wrapping_mul(0xC2B2_AE35) ^ 0x1AB7)
            .chance(LABYRINTH_CHANCE)
}

/// Where a tile falls in the pattern of the mosaic it's part of, `v * CELL + u`: `u` and
/// `v` run 1 to 3 across a cell's own floor, and are 0 on the line of the walls between
/// cells (see `labyrinth_art::mosaic`).
pub fn mosaic_at(x: i32, z: i32) -> usize {
    let (u, v) = ((x - MARGIN).rem_euclid(CELL), (z - MARGIN).rem_euclid(CELL));
    (v * CELL + u) as usize
}

/// Set on a tile of mosaic laid in the second of the two designs (see `World::flag`): a
/// room's floor is all one design, and the corridors' change every few cells.
pub const SECOND_MOTIF: u8 = 64;
/// Set on the flagstones of the sunlit courtyard, paler than the rest.
pub const COURTYARD: u8 = 32;

const DIRS: [(i32, i32); 4] = [(1, 0), (-1, 0), (0, 1), (0, -1)];

/// The maze, cell by cell: `east[j * cols + i]` is the way through from cell (i, j) to the
/// one east of it, and `south` to the one south of it.
struct Maze {
    cols: i32,
    rows: i32,
    east: Vec<bool>,
    south: Vec<bool>,
}

impl Maze {
    fn new(cols: i32, rows: i32) -> Maze {
        let n = (cols * rows) as usize;
        Maze {
            cols,
            rows,
            east: vec![false; n],
            south: vec![false; n],
        }
    }

    fn at(&self, i: i32, j: i32) -> usize {
        (j * self.cols + i) as usize
    }

    fn inside(&self, i: i32, j: i32) -> bool {
        i >= 0 && j >= 0 && i < self.cols && j < self.rows
    }

    /// Is there a way from cell (i, j) to the next one along `(di, dj)`?
    fn open(&self, i: i32, j: i32, (di, dj): (i32, i32)) -> bool {
        if !self.inside(i, j) || !self.inside(i + di, j + dj) {
            return false;
        }
        match (di, dj) {
            (1, 0) => self.east[self.at(i, j)],
            (-1, 0) => self.east[self.at(i - 1, j)],
            (0, 1) => self.south[self.at(i, j)],
            (0, -1) => self.south[self.at(i, j - 1)],
            _ => false,
        }
    }

    fn set(&mut self, i: i32, j: i32, (di, dj): (i32, i32), way: bool) {
        if !self.inside(i, j) || !self.inside(i + di, j + dj) {
            return;
        }
        let k = match (di, dj) {
            (1, 0) | (0, 1) => self.at(i, j),
            _ => self.at(i + di, j + dj),
        };
        if di != 0 {
            self.east[k] = way;
        } else {
            self.south[k] = way;
        }
    }

    fn exits(&self, i: i32, j: i32) -> usize {
        DIRS.iter().filter(|&&d| self.open(i, j, d)).count()
    }

    /// Opens every wall inside a block of cells.
    fn clear(&mut self, (i0, j0, cw, ch): (i32, i32, i32, i32)) {
        for j in j0..j0 + ch {
            for i in i0..i0 + cw {
                if i + 1 < i0 + cw {
                    self.set(i, j, (1, 0), true);
                }
                if j + 1 < j0 + ch {
                    self.set(i, j, (0, 1), true);
                }
            }
        }
    }

    /// Steps from a cell to every cell.
    fn steps(&self, from: (i32, i32)) -> Vec<u32> {
        let mut dist = vec![u32::MAX; (self.cols * self.rows) as usize];
        let mut q = std::collections::VecDeque::from([from]);
        dist[self.at(from.0, from.1)] = 0;
        while let Some((i, j)) = q.pop_front() {
            let d = dist[self.at(i, j)];
            for dir in DIRS {
                let (ni, nj) = (i + dir.0, j + dir.1);
                if self.open(i, j, dir) && dist[self.at(ni, nj)] == u32::MAX {
                    dist[self.at(ni, nj)] = d + 1;
                    q.push_back((ni, nj));
                }
            }
        }
        dist
    }
}

/// Carves a maze through every cell from `from`, running on straight more often than not,
/// for long corridors a minotaur can charge down.
fn carve(m: &mut Maze, from: (i32, i32), r: &mut Rng) {
    let mut seen = vec![false; (m.cols * m.rows) as usize];
    seen[m.at(from.0, from.1)] = true;
    let mut stack = vec![(from, (1, 0))];
    while let Some(&((i, j), came)) = stack.last() {
        let ways: Vec<(i32, i32)> = DIRS
            .iter()
            .copied()
            .filter(|&(di, dj)| m.inside(i + di, j + dj) && !seen[m.at(i + di, j + dj)])
            .collect();
        if ways.is_empty() {
            stack.pop();
            continue;
        }
        let d = if ways.contains(&came) && r.chance(0.35) {
            came
        } else {
            ways[r.below(ways.len())]
        };
        m.set(i, j, d, true);
        let next = (i + d.0, j + d.1);
        seen[m.at(next.0, next.1)] = true;
        stack.push((next, d));
    }
}

/// Does a block of cells `(i, j, w, h)` touch another, or come within `pad` cells of it?
fn near(
    (ai, aj, aw, ah): (i32, i32, i32, i32),
    (bi, bj, bw, bh): (i32, i32, i32, i32),
    pad: i32,
) -> bool {
    ai - pad < bi + bw && bi - pad < ai + aw && aj - pad < bj + bh && bj - pad < aj + ah
}

fn inside_block((i0, j0, w, h): (i32, i32, i32, i32), i: i32, j: i32) -> bool {
    i >= i0 && j >= j0 && i < i0 + w && j < j0 + h
}

/// The tile at a cell's north-west corner.
fn corner(i: i32, j: i32) -> (i32, i32) {
    (MARGIN + 1 + i * CELL, MARGIN + 1 + j * CELL)
}

/// The tile in the middle of a cell.
fn middle(i: i32, j: i32) -> (i32, i32) {
    let (x, z) = corner(i, j);
    (x + 1, z + 1)
}

/// A tile of marble wall: mostly plain blocks, some cracked, some mossy.
fn wall_look(r: &mut Rng) -> u8 {
    match r.below(100) {
        0..=63 => PLAIN,
        64..=81 => CRACKED,
        _ => MOSSY,
    }
}

pub fn generate(seed: u64, depth: u32, biome: usize) -> Level {
    let mut r = Rng::new(seed ^ (depth as u64).wrapping_mul(0x9E37_79B9) ^ 0x1AB7_1AB7);
    let grow = depth.min(40) as i32;
    let (cols, rows) = (9 + grow / 7, 6 + grow / 10);
    let mut m = Maze::new(cols, rows);

    // In on one side (west or east), and the courtyard over in the far half.
    let flip = r.chance(0.5);
    let side = |i: i32| if flip { cols - 1 - i } else { i };
    let start_cell = (side(0), r.range(1, rows - 1));
    let hi = r.range(cols / 2, cols - 2);
    let hall = (
        if flip { cols - 3 - hi } else { hi },
        r.range(0, rows - 2),
        3,
        3,
    );
    carve(&mut m, start_cell, &mut r);
    m.clear(hall);

    // Rooms: a few blocks of cells opened up into one, never in the way of the courtyard or
    // where you come in.
    let mut rooms: Vec<(i32, i32, i32, i32)> = Vec::new();
    let want = (cols * rows / 50).max(1) as usize + r.below(2);
    for _ in 0..80 {
        if rooms.len() >= want {
            break;
        }
        let (rw, rh) = match r.below(10) {
            0..=5 => (2, 2),
            6 | 7 => (3, 2),
            _ => (2, 3),
        };
        let room = (r.range(0, cols - rw + 1), r.range(0, rows - rh + 1), rw, rh);
        if near(room, hall, 1)
            || rooms.iter().any(|&o| near(room, o, 1))
            || inside_block(room, start_cell.0, start_cell.1)
        {
            continue;
        }
        m.clear(room);
        rooms.push(room);
    }
    let in_room = |i: i32, j: i32| rooms.iter().any(|&b| inside_block(b, i, j));
    let in_hall = |i: i32, j: i32| inside_block(hall, i, j);

    // A few loops, so there's more than one way round; and now and then a dead end broken
    // through into the next corridor.
    for j in 0..rows {
        for i in 0..cols {
            for d in [(1, 0), (0, 1)] {
                if m.inside(i + d.0, j + d.1) && !m.open(i, j, d) && r.chance(0.05) {
                    m.set(i, j, d, true);
                }
            }
        }
    }
    for j in 0..rows {
        for i in 0..cols {
            if m.exits(i, j) == 1 && (i, j) != start_cell && r.chance(0.15) {
                let shut: Vec<(i32, i32)> = DIRS
                    .iter()
                    .copied()
                    .filter(|&d| m.inside(i + d.0, j + d.1) && !m.open(i, j, d))
                    .collect();
                if !shut.is_empty() {
                    m.set(i, j, shut[r.below(shut.len())], true);
                }
            }
        }
    }

    // Down the stairs in the cell furthest from where you come in (never the courtyard).
    let steps = m.steps(start_cell);
    let mut far = (0, start_cell);
    for j in 0..rows {
        for i in 0..cols {
            let d = steps[m.at(i, j)];
            if d != u32::MAX && d > far.0 && !in_hall(i, j) {
                far = (d, (i, j));
            }
        }
    }
    let stairs_cell = far.1;

    // Mosaics in the rooms and here and there along the corridors; flagstones everywhere
    // else, and in the sunlit courtyard.
    let patch = hash2(depth as i32, 0x1AB, seed as u32);
    let mosaic: Vec<bool> = (0..rows)
        .flat_map(|j| (0..cols).map(move |i| (i, j)))
        .map(|(i, j)| !in_hall(i, j) && (in_room(i, j) || hash2(i / 2, j / 2, patch) % 100 < 22))
        .collect();
    let tiled = |i: i32, j: i32| m.inside(i, j) && mosaic[m.at(i, j)];
    let motif = |i: i32, j: i32| match rooms.iter().position(|&b| inside_block(b, i, j)) {
        Some(k) => hash2(k as i32, 7, patch) % 2 == 1,
        None => hash2(i / 3, j / 2, patch ^ 0x5EC) % 2 == 1,
    };

    let (w, h) = (MARGIN * 2 + cols * CELL + 1, MARGIN * 2 + rows * CELL + 1);
    let mut world = World::new(w, h, Area::Hollow { depth }, biome);
    world.labyrinth = true;
    let lay = |world: &mut World, x: i32, z: i32, tiles: bool| {
        world.set_wall(x, z, Wall::None);
        world.set_floor(x, z, if tiles { Floor::Tiles } else { Floor::Walkway });
    };
    for j in 0..rows {
        for i in 0..cols {
            let (x0, z0) = corner(i, j);
            for dz in -1..3 {
                for dx in -1..3 {
                    // (The flag goes on the line of the walls round the cell too, for
                    // wherever the maze goes through.)
                    world.set_flag(x0 + dx, z0 + dz, SECOND_MOTIF, motif(i, j));
                    if dx >= 0 && dz >= 0 {
                        lay(&mut world, x0 + dx, z0 + dz, tiled(i, j));
                    }
                }
            }
        }
    }
    // The walls between cells running north and south, and a way through where the maze
    // goes.
    for i in 0..=cols {
        let x = MARGIN + i * CELL;
        for j in 0..rows {
            let z0 = MARGIN + 1 + j * CELL;
            if m.open(i - 1, j, (1, 0)) {
                for k in 0..3 {
                    lay(&mut world, x, z0 + k, tiled(i - 1, j) && tiled(i, j));
                }
            } else {
                for k in 0..3 {
                    world.set_wall(x, z0 + k, Wall::Marble(wall_look(&mut r)));
                }
            }
        }
    }
    // And east and west, their faces towards you: here and there a niche or a gilded
    // shield in the middle.
    for j in 0..=rows {
        let z = MARGIN + j * CELL;
        for i in 0..cols {
            let x0 = MARGIN + 1 + i * CELL;
            if m.open(i, j - 1, (0, 1)) {
                for k in 0..3 {
                    lay(&mut world, x0 + k, z, tiled(i, j - 1) && tiled(i, j));
                }
            } else {
                for k in 0..3 {
                    let look = match r.below(100) {
                        0..=6 if k == 1 && j < rows => NICHE,
                        7..=10 if k == 1 && j < rows => SHIELD,
                        _ => wall_look(&mut r),
                    };
                    world.set_wall(x0 + k, z, Wall::Marble(look));
                }
            }
        }
    }
    // Piers where the walls meet. One standing on its own, with the ways all round it
    // open, is a column instead.
    let mut posts: Vec<(i32, i32, i32, i32)> = Vec::new();
    for j in 0..=rows {
        for i in 0..=cols {
            let (x, z) = (MARGIN + i * CELL, MARGIN + j * CELL);
            let free = m.open(i - 1, j - 1, (1, 0))
                && m.open(i - 1, j, (1, 0))
                && m.open(i - 1, j - 1, (0, 1))
                && m.open(i, j - 1, (0, 1));
            if free {
                let tiles =
                    tiled(i - 1, j - 1) && tiled(i, j - 1) && tiled(i - 1, j) && tiled(i, j);
                lay(&mut world, x, z, tiles);
                posts.push((x, z, i, j));
            } else {
                world.set_wall(x, z, Wall::Marble(PIER));
            }
        }
    }

    let start = middle(start_cell.0, start_cell.1);
    let stairs = middle(stairs_cell.0, stairs_cell.1);
    world.set_obj(stairs.0, stairs.1, Some(Obj::StairsDown));
    let keep_clear = |x: i32, z: i32| {
        (x - start.0).abs() <= 1 && (z - start.1).abs() <= 1
            || (x - stairs.0).abs() <= 1 && (z - stairs.1).abs() <= 1
    };
    let free = |world: &World, x: i32, z: i32| {
        open(world, x, z) && world.obj(x, z).is_none() && !keep_clear(x, z)
    };

    // The courtyard: columns round a griffin's nest on a broken column in the middle,
    // shafts of daylight, and broken marble lying about.
    let (hx, hz) = (hall.0, hall.1);
    let nest = middle(hx + 1, hz + 1);
    world.set_obj(nest.0, nest.1, Some(Obj::Nest));
    for &(x, z, i, j) in &posts {
        if inside_block(hall, i, j) && inside_block(hall, i - 1, j - 1) {
            let var = if r.chance(0.7) { STANDING } else { BROKEN };
            world.set_obj(x, z, Some(Obj::Column { var }));
        }
    }
    let (ax, az) = corner(hx, hz);
    let (bx, bz) = (ax + 3 * CELL - 2, az + 3 * CELL - 2);
    for z in az..=bz {
        for x in ax..=bx {
            world.set_flag(x, z, COURTYARD, true);
        }
    }
    let mut beams = 0;
    for _ in 0..80 {
        if beams >= 4 {
            break;
        }
        let (x, z) = (r.range(ax, bx + 1), r.range(az, bz + 1));
        let by_nest = (x - nest.0).abs() <= 1 && (z - nest.1).abs() <= 1;
        let by_beam = (-2..=2).any(|dz: i32| {
            (-2..=2).any(|dx: i32| matches!(world.obj(x + dx, z + dz), Some(Obj::Sunbeam)))
        });
        if !by_nest && !by_beam && free(&world, x, z) {
            world.set_obj(x, z, Some(Obj::Sunbeam));
            beams += 1;
        }
    }
    for z in az..=bz {
        for x in ax..=bx {
            if free(&world, x, z) && r.chance(0.04) {
                world.set_obj(
                    x,
                    z,
                    Some(Obj::Rubble {
                        var: r.below(2) as u8,
                    }),
                );
            }
        }
    }

    // Wherever else a pier would stand but for the ways open all round it: in the rooms a
    // statue, a brazier or a column, and a column anywhere else.
    for &(x, z, i, j) in &posts {
        if world.obj(x, z).is_some() {
            continue;
        }
        let column = Obj::Column {
            var: if r.chance(0.65) { STANDING } else { BROKEN },
        };
        let o = if in_room(i, j) && in_room(i - 1, j - 1) {
            match r.below(100) {
                0..=39 => Obj::Statue {
                    var: r.below(2) as u8,
                },
                40..=74 => Obj::Brazier,
                _ => column,
            }
        } else {
            column
        };
        world.set_obj(x, z, Some(o));
    }

    // Arches over the doorways through the walls across your way, and torches on the
    // walls facing you.
    for j in 1..rows {
        let z = MARGIN + j * CELL;
        for i in 0..cols {
            let x = MARGIN + 2 + i * CELL;
            let roomy = |i: i32, j: i32| in_room(i, j) || in_hall(i, j);
            let inside = roomy(i, j - 1) && roomy(i, j);
            // Only ever from pier to pier.
            let piers = world.wall(x - 2, z) == Wall::Marble(PIER)
                && world.wall(x + 2, z) == Wall::Marble(PIER);
            if m.open(i, j - 1, (0, 1)) && !inside && piers && r.chance(0.5) && free(&world, x, z) {
                world.set_obj(x, z, Some(Obj::Arch));
            }
        }
    }
    for j in 0..rows {
        let z = MARGIN + j * CELL;
        for i in 0..cols {
            let x = MARGIN + 2 + i * CELL;
            let wall = world.wall(x, z);
            let plain = matches!(wall, Wall::Marble(l) if l != NICHE && l != SHIELD);
            if plain && r.chance(0.26) && free(&world, x, z + 1) && !in_hall(i, j) {
                world.set_obj(x, z + 1, Some(Obj::Sconce));
            }
        }
    }

    // Braziers at some of the crossings.
    for j in 0..rows {
        for i in 0..cols {
            if m.exits(i, j) >= 3 && !in_room(i, j) && !in_hall(i, j) && r.chance(0.3) {
                let (x0, z0) = corner(i, j);
                let (dx, dz) = [(0, 0), (2, 0), (0, 2), (2, 2)][r.below(4)];
                if free(&world, x0 + dx, z0 + dz) {
                    place_solid(&mut world, x0 + dx, z0 + dz, Obj::Brazier, start);
                }
            }
        }
    }

    // Treasure at the back of the dead ends (and if the maze has too few of them, in a
    // corner somewhere quiet), or a statue keeping watch, or a heap of rubble where the
    // roof came down.
    let mut ends: Vec<(i32, i32)> = (0..rows)
        .flat_map(|j| (0..cols).map(move |i| (i, j)))
        .filter(|&(i, j)| {
            m.exits(i, j) == 1
                && (i, j) != start_cell
                && (i, j) != stairs_cell
                && !in_room(i, j)
                && !in_hall(i, j)
        })
        .collect();
    r.shuffle(&mut ends);
    let mut chests = 2 + r.below(2) + depth as usize / 25;
    for &(i, j) in &ends {
        let Some(&d) = DIRS.iter().find(|&&d| m.open(i, j, d)) else {
            continue;
        };
        let (cx, cz) = middle(i, j);
        let back = (cx - d.0, cz - d.1);
        let o = if chests > 0 {
            chests -= 1;
            Obj::LootChest {
                opened: false,
                gleam: r.chance(GLEAM_CHANCE * 1.5),
            }
        } else {
            match r.below(10) {
                0..=3 => Obj::Statue {
                    var: r.below(2) as u8,
                },
                4 | 5 => Obj::Rubble { var: HEAP },
                _ => continue,
            }
        };
        if free(&world, back.0, back.1) {
            place_solid(&mut world, back.0, back.1, o, start);
        }
    }
    let far_off = walk(&world, start);
    let mut corners: Vec<(u32, i32, i32)> = Vec::new();
    for j in 0..rows {
        for i in 0..cols {
            if in_hall(i, j) || (i, j) == start_cell {
                continue;
            }
            let (x0, z0) = corner(i, j);
            for (dx, dz) in [(0, 0), (2, 0), (0, 2), (2, 2)] {
                let (x, z) = (x0 + dx, z0 + dz);
                let d = far_off[world.idx(x, z)];
                if d != u32::MAX && d > 15 && free(&world, x, z) {
                    corners.push((d.min(60) + r.below(30) as u32, x, z));
                }
            }
        }
    }
    corners.sort_by(|a, b| b.0.cmp(&a.0));
    for (_, x, z) in corners {
        if chests == 0 {
            break;
        }
        let chest = Obj::LootChest {
            opened: false,
            gleam: r.chance(GLEAM_CHANCE),
        };
        if free(&world, x, z) && place_solid(&mut world, x, z, chest, start) {
            chests -= 1;
        }
    }

    // Rooms: old pots along the walls, and a chest in some of them.
    for &(i0, j0, cw, ch) in &rooms {
        let (x0, z0) = corner(i0, j0);
        let (x1, z1) = (x0 + cw * CELL - 2, z0 + ch * CELL - 2);
        let mut spots: Vec<(i32, i32)> = Vec::new();
        for z in z0..=z1 {
            for x in x0..=x1 {
                let by_wall = DIRS
                    .iter()
                    .any(|&(dx, dz)| world.wall(x + dx, z + dz) != Wall::None);
                if by_wall && free(&world, x, z) {
                    spots.push((x, z));
                }
            }
        }
        r.shuffle(&mut spots);
        let mut spots = spots.into_iter();
        if r.chance(0.35) {
            for (x, z) in spots.by_ref() {
                let chest = Obj::LootChest {
                    opened: false,
                    gleam: r.chance(GLEAM_CHANCE),
                };
                if place_solid(&mut world, x, z, chest, start) {
                    break;
                }
            }
        }
        for _ in 0..r.range(1, 4) {
            let Some((x, z)) = spots.next() else { break };
            place_solid(&mut world, x, z, Obj::Pot { hp: 1 }, start);
        }
        if let Some((x, z)) = spots.next() {
            if r.chance(0.5) {
                world.set_obj(
                    x,
                    z,
                    Some(Obj::Rubble {
                        var: r.below(2) as u8,
                    }),
                );
            }
        }
    }

    // Along the corridors: fallen columns, rubble under the cracked walls, and bones.
    for j in 0..rows {
        for i in 0..cols {
            if in_room(i, j) || in_hall(i, j) || (i, j) == start_cell {
                continue;
            }
            let (x0, z0) = corner(i, j);
            if r.chance(0.05) {
                let (x, z) = (x0 + r.range(0, 3), z0 + r.range(0, 3));
                if free(&world, x, z) {
                    place_solid(&mut world, x, z, Obj::Column { var: FALLEN }, start);
                }
            }
            if r.chance(0.04) {
                let (x, z) = (x0 + r.range(0, 3), z0 + r.range(0, 3));
                if free(&world, x, z) {
                    world.set_obj(x, z, Some(Obj::Bones));
                }
            }
        }
    }
    for z in 1..h - 1 {
        for x in 1..w - 1 {
            if world.wall(x, z - 1) == Wall::Marble(CRACKED) && free(&world, x, z) && r.chance(0.3)
            {
                world.set_obj(
                    x,
                    z,
                    Some(Obj::Rubble {
                        var: r.below(2) as u8,
                    }),
                );
            }
        }
    }

    // Now and then, a cracked flagstone over a secret room.
    let dist = walk(&world, start);
    let mut crack = None;
    if r.chance(CRACK_CHANCE) {
        for _ in 0..400 {
            let (x, z) = (r.range(1, w - 1), r.range(1, h - 1));
            let clear = (-1..=1).all(|dz| {
                (-1..=1)
                    .all(|dx| open(&world, x + dx, z + dz) && world.obj(x + dx, z + dz).is_none())
            });
            if clear && !keep_clear(x, z) && dist[world.idx(x, z)] != u32::MAX {
                world.set_obj(x, z, Some(Obj::Crack));
                crack = Some((x, z));
                break;
            }
        }
    }

    // Who lives here: fewer of the biome's folk than most floors, for the minotaurs roaming
    // the corridors, and the griffin on its nest.
    let foes = floor_foes(biome, depth);
    let weights: Vec<f32> = foes.iter().map(|f| f.1).collect();
    let cap = (4 + depth as usize / 3).min(20);
    let dist = walk(&world, start);
    let mut spawns = Vec::new();
    for _ in 0..cap * 40 {
        if spawns.len() >= cap {
            break;
        }
        let (x, z) = (r.range(1, w - 1), r.range(1, h - 1));
        let d = dist[world.idx(x, z)];
        if d == u32::MAX || d < 12 || world.blocked(x, z) || keep_clear(x, z) {
            continue;
        }
        let mut foe = foes[r.weighted(&weights)].0;
        if too_tough(foe, depth) {
            foe = Foe::Sneak;
        }
        spawns.push(Spawn {
            foe,
            x: x as f32 + 0.5,
            z: z as f32 + 0.5,
            boss: false,
        });
    }
    // Minotaurs out in the corridors, well away from where you come in and from each
    // other: more of them deeper down.
    let bulls = minotaurs(depth);
    let mut lairs: Vec<(i32, i32)> = Vec::new();
    for _ in 0..400 {
        if lairs.len() >= bulls {
            break;
        }
        let (i, j) = (r.range(0, cols), r.range(0, rows));
        let (x, z) = middle(i, j);
        let d = dist[world.idx(x, z)];
        let spaced = lairs
            .iter()
            .all(|&(lx, lz)| (lx - x).abs() + (lz - z).abs() >= 12);
        if d != u32::MAX
            && d >= 22
            && !in_hall(i, j)
            && !world.blocked(x, z)
            && !keep_clear(x, z)
            && spaced
        {
            lairs.push((x, z));
        }
    }
    for (x, z) in lairs {
        spawns.push(Spawn {
            foe: Foe::Minotaur,
            x: x as f32 + 0.5,
            z: z as f32 + 0.5,
            boss: false,
        });
    }
    // The griffin on its nest (and deeper down, its mate somewhere in the courtyard).
    spawns.push(Spawn {
        foe: Foe::Griffin,
        x: nest.0 as f32 + 0.5,
        z: nest.1 as f32 + 0.5,
        boss: false,
    });
    if depth >= GRIFFIN_PAIR {
        for _ in 0..40 {
            let (x, z) = (r.range(ax, bx + 1), r.range(az, bz + 1));
            if !world.blocked(x, z) && (x - nest.0).abs() + (z - nest.1).abs() >= 3 {
                spawns.push(Spawn {
                    foe: Foe::Griffin,
                    x: x as f32 + 0.5,
                    z: z as f32 + 0.5,
                    boss: false,
                });
                break;
            }
        }
    }

    Level {
        world,
        start,
        stairs,
        waystone: None,
        spawns,
        lair: (start.0 as f32 + 0.5, start.1 as f32 + 0.5),
        crack,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::dungeon::biome_for;

    /// Some labyrinths to look at, whatever the chance says.
    fn mazes() -> Vec<Level> {
        [(1u64, 8u32), (7, 12), (42, 25), (9, 47), (123, 68), (5, 99)]
            .into_iter()
            .map(|(seed, depth)| generate(seed, depth, biome_for(seed, depth)))
            .collect()
    }

    fn count(l: &Level, f: impl Fn(&World, i32, i32) -> bool) -> usize {
        let w = &l.world;
        (0..w.h)
            .flat_map(|z| (0..w.w).map(move |x| (x, z)))
            .filter(|&(x, z)| f(w, x, z))
            .count()
    }

    #[test]
    fn the_stairs_are_a_long_walk_through_the_maze() {
        for l in mazes() {
            let w = &l.world;
            assert!(w.labyrinth);
            let dist = walk(w, l.start);
            let d = dist[w.idx(l.stairs.0, l.stairs.1)];
            assert_ne!(d, u32::MAX, "the stairs are out of reach");
            assert!(d > 30, "only {d} steps");
            assert!(matches!(
                w.floor(l.start.0, l.start.1),
                Floor::Walkway | Floor::Tiles
            ));
        }
    }

    #[test]
    fn marble_walls_piers_mosaics_and_a_courtyard() {
        for l in mazes() {
            let marble = count(&l, |w, x, z| matches!(w.wall(x, z), Wall::Marble(_)));
            let piers = count(&l, |w, x, z| w.wall(x, z) == Wall::Marble(PIER));
            let mosaic = count(&l, |w, x, z| w.floor(x, z) == Floor::Tiles);
            let flags = count(&l, |w, x, z| w.floor(x, z) == Floor::Walkway);
            let nests = count(&l, |w, x, z| matches!(w.obj(x, z), Some(Obj::Nest)));
            let beams = count(&l, |w, x, z| matches!(w.obj(x, z), Some(Obj::Sunbeam)));
            let columns = count(&l, |w, x, z| {
                matches!(w.obj(x, z), Some(Obj::Column { .. }))
            });
            let lights = count(&l, |w, x, z| {
                matches!(w.obj(x, z), Some(Obj::Brazier | Obj::Sconce))
            });
            let chests = count(&l, |w, x, z| {
                matches!(w.obj(x, z), Some(Obj::LootChest { .. }))
            });
            let other = count(&l, |w, x, z| {
                !matches!(w.wall(x, z), Wall::None | Wall::Marble(_))
            });
            assert!(marble > 150, "{marble} marble walls");
            assert!(piers > 40, "{piers} piers");
            assert!(mosaic > 40, "{mosaic} tiles of mosaic");
            assert!(flags > 100, "{flags} flagstones");
            assert_eq!(nests, 1);
            assert!(beams >= 2, "{beams} shafts of daylight");
            assert!(columns >= 4, "{columns} columns");
            assert!(lights >= 6, "{lights} braziers and torches");
            assert!(chests >= 2, "{chests} chests");
            assert_eq!(other, 0, "nothing but marble");
        }
    }

    /// Where you could walk if nothing were in the way: everywhere but walls and the dark.
    fn open_ground(w: &World, from: (i32, i32)) -> Vec<bool> {
        let mut seen = vec![false; (w.w * w.h) as usize];
        let mut q = std::collections::VecDeque::from([from]);
        seen[w.idx(from.0, from.1)] = true;
        while let Some((x, z)) = q.pop_front() {
            for (dx, dz) in DIRS {
                let (nx, nz) = (x + dx, z + dz);
                if open(w, nx, nz) && !seen[w.idx(nx, nz)] {
                    seen[w.idx(nx, nz)] = true;
                    q.push_back((nx, nz));
                }
            }
        }
        seen
    }

    #[test]
    fn nothing_shuts_the_way_anywhere() {
        for seed in 0..30u64 {
            for depth in [8u32, 17, 33, 52, 76] {
                let seed = seed * 7919 + 11;
                let l = generate(seed, depth, biome_for(seed, depth));
                let w = &l.world;
                let dist = walk(w, l.start);
                assert_ne!(dist[w.idx(l.stairs.0, l.stairs.1)], u32::MAX);
                // Every tile of the maze is part of it: you can walk everywhere that
                // nothing stands.
                let ground = open_ground(w, l.start);
                for z in 0..w.h {
                    for x in 0..w.w {
                        let i = w.idx(x, z);
                        if open(w, x, z) {
                            assert!(ground[i], "({x}, {z}) is cut off on seed {seed}");
                        }
                        assert!(
                            !ground[i] || w.blocked(x, z) || dist[i] != u32::MAX,
                            "({x}, {z}) is shut off on seed {seed} floor {depth}"
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn deterministic() {
        let a = generate(77, 23, 2);
        let b = generate(77, 23, 2);
        assert_eq!(a.start, b.start);
        assert_eq!(a.stairs, b.stairs);
        assert_eq!(a.world.floor, b.world.floor);
        assert_eq!(a.world.wall, b.world.wall);
    }

    #[test]
    fn now_and_then_but_never_too_soon() {
        let n = (0..1000u64)
            .filter(|&seed| is_labyrinth(seed, 23, biome_for(seed, 23)))
            .count();
        assert!((40..130).contains(&n), "{n} in 1000");
        assert!(
            !(0..400u64).any(|seed| is_labyrinth(seed, 20, 2)),
            "never a guardian's"
        );
        assert!(
            !(0..400u64).any(|seed| is_labyrinth(seed, 7, 2)),
            "not so soon"
        );
        // Never where the sewers or the glowcaps are.
        for seed in 0..400u64 {
            for depth in 8..60 {
                let biome = biome_for(seed, depth);
                if is_labyrinth(seed, depth, biome) {
                    assert!(!super::super::sewer::is_sewer(seed, depth));
                    assert!(!super::super::glowcave::is_glowcave(seed, depth, biome));
                }
            }
        }
    }

    #[test]
    fn mosaics_line_up_with_the_cells() {
        // A cell's own floor runs 1 to 3 across, and the walls between are 0.
        let (x0, z0) = corner(2, 3);
        assert_eq!(mosaic_at(x0, z0), (CELL + 1) as usize);
        assert_eq!(mosaic_at(x0 + 2, z0 + 2), (3 * CELL + 3) as usize);
        assert_eq!(mosaic_at(x0 - 1, z0 + 1), CELL as usize * 2);
        assert_eq!(mosaic_at(x0 - 1, z0 - 1), 0);
    }

    #[test]
    fn a_room_is_all_one_mosaic() {
        for l in mazes() {
            let w = &l.world;
            for z in 1..w.h - 1 {
                for x in 1..w.w - 1 {
                    // Mosaic running on into mosaic (a room, or a run of corridor) keeps
                    // to one design across each cell.
                    if w.floor(x, z) == Floor::Tiles && mosaic_at(x, z) == (CELL + 1) as usize {
                        let first = w.flag(x, z, SECOND_MOTIF);
                        for dz in 0..3 {
                            for dx in 0..3 {
                                assert_eq!(w.flag(x + dx, z + dz, SECOND_MOTIF), first);
                            }
                        }
                    }
                }
            }
        }
    }
}
