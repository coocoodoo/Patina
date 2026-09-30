//! The Starless Rift. Now and then a floor of the Hollow, from floor 12 down (and often in
//! the Crystal Grotto), is no cave at all but islands of black rock hanging over a void full
//! of stars, strung together by old stone bridges and great chains. You come in through a
//! portal on the Landing. Amethyst grows on the islands and blue light pours off their edges
//! into the dark; on the biggest of them stands the ruined Citadel, its walls carved with
//! glowing runes and watched by eyes, and the stairs lead down from its sanctum.

use super::dungeon::{
    CRACK_CHANCE, DEEP_FOLK, Foe, GLEAM_CHANCE, Level, Spawn, deep_folk, is_waystone_floor,
};
use super::glowcave::{noise, open, place_solid, reach, walk};
use super::world::{Area, Floor, Obj, Wall, World};
use crate::util::{Rng, hash2};

/// How often a floor is the rift: now and then anywhere deep enough, and often in the
/// Crystal Grotto. Never a guardian's floor, nor where another kind of floor is.
pub const RIFT_CHANCE: f32 = 0.06;
pub const GROTTO_CHANCE: f32 = 0.25;
pub const FIRST_RIFT: u32 = 12;
pub const GROTTO: usize = 1;

/// How a tile of the citadel's obsidian looks (see `Wall::Obsidian`): dark blocks, blocks
/// carved with runes, cracked, veined with amethyst, and a tower standing taller.
pub const BLOCKS: u8 = 0;
pub const RUNED: u8 = 1;
pub const CRACKED: u8 = 2;
pub const VEINED: u8 = 3;
pub const TOWER: u8 = 4;
pub const LOOKS: usize = 5;
/// How tall the citadel's walls stand, and its towers.
pub const WALL_H: f32 = 1.5;
pub const TOWER_H: f32 = 2.5;

/// How far an island's rock face hangs down into the void, and how thick a bridge's slab.
pub const CLIFF: f32 = 2.0;
pub const SLAB: f32 = 0.3;

pub fn obsidian_height(look: u8) -> f32 {
    if look == TOWER { TOWER_H } else { WALL_H }
}

/// Amethyst: a little cluster, a bigger one, and a great crystal you can't walk through.
pub const SMALL: u8 = 0;
pub const MID: u8 = 1;
pub const GREAT: u8 = 2;

/// Which way a voidfall pours off an island's edge, or a chain runs from its post: south
/// (towards you), east or west.
pub const SOUTH: u8 = 0;
pub const EAST: u8 = 1;
pub const WEST: u8 = 2;

pub fn step(dir: u8) -> (i32, i32) {
    match dir {
        EAST => (1, 0),
        WEST => (-1, 0),
        _ => (0, 1),
    }
}

/// Is this floor the rift?
pub fn is_rift(seed: u64, depth: u32, biome: usize) -> bool {
    let chance = if biome == GROTTO {
        GROTTO_CHANCE
    } else {
        RIFT_CHANCE
    };
    depth >= FIRST_RIFT
        && !is_waystone_floor(depth)
        && !super::sewer::is_sewer(seed, depth)
        && !super::glowcave::is_glowcave(seed, depth, biome)
        && !super::labyrinth::is_labyrinth(seed, depth, biome)
        && !super::canyon::is_canyon(seed, depth, biome)
        && Rng::new(seed ^ (depth as u64).wrapping_mul(0x1656_67B1) ^ 0x51F7).chance(chance)
}

/// Who lives in the rift, with weights: gazers, spineback slugs, violet imps and ogres,
/// ghosts and wisps, and the deep folk.
pub fn rift_folk(depth: u32) -> Vec<(Foe, f32)> {
    let mut v = vec![
        (Foe::Gazer, 1.6),
        (Foe::Slug, 1.2),
        (Foe::Imp, 1.4),
        (Foe::Ogre, 0.7),
        (Foe::Ghost, 0.5),
        (Foe::Wisp, 0.5),
    ];
    if depth >= DEEP_FOLK {
        v.extend_from_slice(deep_folk(GROTTO));
    }
    v
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Kind {
    /// Where you come in, by the portal.
    Landing,
    /// The ruined citadel, the stairs down in its sanctum.
    Citadel,
    /// Grown over with amethyst.
    Crystal,
    /// A little islet with treasure on it.
    Shrine,
    /// Bare rock.
    Rock,
}

#[derive(Clone, Copy, Debug)]
struct Isle {
    c: (f32, f32),
    r: (f32, f32),
    kind: Kind,
}

impl Isle {
    fn tile(&self) -> (i32, i32) {
        (self.c.0.floor() as i32, self.c.1.floor() as i32)
    }
}

/// Solid ground (rock or bridge), as against the void.
fn ground(w: &World, x: i32, z: i32) -> bool {
    w.inside(x, z) && w.floor(x, z) != Floor::Void
}

/// A tile of floor right under a wall's face (the wall to its north).
fn under_face(w: &World, x: i32, z: i32) -> bool {
    open(w, x, z) && w.obj(x, z).is_none() && w.wall(x, z - 1) != Wall::None
}

/// Lays a two-wide bridge from `a` to `b`, along and then down (or down and then along),
/// over whatever void lies between.
fn bridge(w: &mut World, a: (i32, i32), b: (i32, i32), along_first: bool) {
    let lay = |w: &mut World, x: i32, z: i32| {
        for (dx, dz) in [(0, 0), (1, 0), (0, 1), (1, 1)] {
            let (tx, tz) = (x + dx, z + dz);
            if w.inside(tx, tz) && w.floor(tx, tz) == Floor::Void {
                w.set_floor(tx, tz, Floor::Span);
            }
        }
    };
    let elbow = if along_first { (b.0, a.1) } else { (a.0, b.1) };
    for (from, to) in [(a, elbow), (elbow, b)] {
        let (mut x, mut z) = from;
        lay(w, x, z);
        while (x, z) != to {
            x += (to.0 - x).signum();
            z += (to.1 - z).signum();
            lay(w, x, z);
        }
    }
}

/// How many tiles of void a bridge from `a` to `b` would cross.
fn span_len(w: &World, a: (i32, i32), b: (i32, i32), along_first: bool) -> usize {
    let elbow = if along_first { (b.0, a.1) } else { (a.0, b.1) };
    let mut n = 0;
    for (from, to) in [(a, elbow), (elbow, b)] {
        let (mut x, mut z) = from;
        while (x, z) != to {
            x += (to.0 - x).signum();
            z += (to.1 - z).signum();
            if !ground(w, x, z) {
                n += 1;
            }
        }
    }
    n
}

pub fn generate(seed: u64, depth: u32, biome: usize) -> Level {
    let mut r = Rng::new(seed ^ (depth as u64).wrapping_mul(0x9E37_79B9) ^ 0x51F7_51F7);
    let ns = hash2(depth as i32, 0x51F, seed as u32);
    let grow = depth.min(60) as i32;
    let (w, h) = (64 + grow / 3, 46 + grow / 4);
    let mut world = World::new(w, h, Area::Hollow { depth }, biome);
    world.rift = true;
    for z in 0..h {
        for x in 0..w {
            world.set_floor(x, z, Floor::Void);
            world.set_wall(x, z, Wall::None);
        }
    }
    let (wf, hf) = (w as f32, h as f32);
    let flip = r.chance(0.5);
    let side = |x: f32| if flip { wf - x } else { x };

    // The islands: the Landing at one end, the Citadel at the other, and between and round
    // them islands of amethyst, bare rock, and little islets with treasure on them.
    let mut isles = vec![
        Isle {
            c: (side(9.5), hf * 0.5 + r.range_f(-5.0, 5.0)),
            r: (4.8, 4.0),
            kind: Kind::Landing,
        },
        Isle {
            c: (side(wf - 16.0), hf * 0.5 + r.range_f(-3.0, 3.0)),
            r: (10.0 + grow as f32 / 20.0, 7.5 + grow as f32 / 30.0),
            kind: Kind::Citadel,
        },
    ];
    let want = 6 + (grow / 12) as usize;
    for _ in 0..3000 {
        if isles.len() >= 2 + want {
            break;
        }
        let kind = [Kind::Crystal, Kind::Shrine, Kind::Rock][isles.len() % 3];
        let rx = if kind == Kind::Shrine {
            r.range_f(2.4, 3.0)
        } else {
            r.range_f(3.4, 6.2)
        };
        let rz = (rx * r.range_f(0.65, 0.95)).max(2.3);
        let c = (
            r.range_f(rx + 3.0, wf - rx - 3.0),
            r.range_f(rz + 3.0, hf - rz - 3.0),
        );
        let clear = isles.iter().all(|o| {
            ((c.0 - o.c.0) / (rx + o.r.0 + 3.5)).hypot((c.1 - o.c.1) / (rz + o.r.1 + 3.0)) > 1.0
        });
        if clear {
            isles.push(Isle {
                c,
                r: (rx, rz),
                kind,
            });
        }
    }
    for (k, isle) in isles.iter().enumerate() {
        let ((cx, cz), (rx, rz)) = (isle.c, isle.r);
        for z in 2..h - 2 {
            for x in 2..w - 2 {
                let (dx, dz) = ((x as f32 + 0.5 - cx) / rx, (z as f32 + 0.5 - cz) / rz);
                let ragged = 0.35 * noise(x as f32, z as f32, 2.5, ns ^ (k as u32 * 97));
                if dx * dx + dz * dz < 1.0 + ragged {
                    world.set_floor(x, z, Floor::Cave);
                }
            }
        }
    }
    // Smooth off lone spurs and fill in notches.
    for _ in 0..2 {
        let before = world.floor.clone();
        let rock = |x: i32, z: i32| {
            x >= 0 && z >= 0 && x < w && z < h && before[(z * w + x) as usize] == Floor::Cave
        };
        for z in 2..h - 2 {
            for x in 2..w - 2 {
                let n = [(1, 0), (-1, 0), (0, 1), (0, -1)]
                    .iter()
                    .filter(|&&(dx, dz)| rock(x + dx, z + dz))
                    .count();
                if rock(x, z) && n <= 1 {
                    world.set_floor(x, z, Floor::Void);
                } else if !rock(x, z) && n >= 3 {
                    world.set_floor(x, z, Floor::Cave);
                }
            }
        }
    }

    // The bridges: every island joined to its nearest neighbour already joined, and a loop
    // or two besides.
    let n = isles.len();
    let dist = |a: usize, b: usize| {
        let (p, q) = (isles[a].c, isles[b].c);
        (p.0 - q.0).hypot(p.1 - q.1)
    };
    let mut joined = vec![false; n];
    joined[0] = true;
    let mut links: Vec<(usize, usize)> = Vec::new();
    for _ in 1..n {
        let mut best: Option<(f32, usize, usize)> = None;
        for a in (0..n).filter(|&a| joined[a]) {
            for b in (0..n).filter(|&b| !joined[b]) {
                let d = dist(a, b);
                if best.is_none_or(|(bd, _, _)| d < bd) {
                    best = Some((d, a, b));
                }
            }
        }
        let Some((_, a, b)) = best else { break };
        joined[b] = true;
        links.push((a, b));
    }
    let mut spare: Vec<(f32, usize, usize)> = (0..n)
        .flat_map(|a| (a + 1..n).map(move |b| (a, b)))
        .filter(|&(a, b)| !links.contains(&(a, b)) && !links.contains(&(b, a)))
        .map(|(a, b)| (dist(a, b), a, b))
        .collect();
    spare.sort_by(|p, q| p.0.total_cmp(&q.0));
    for &(_, a, b) in spare.iter().take(1 + r.below(2)) {
        links.push((a, b));
    }
    for &(a, b) in &links {
        let (p, q) = (isles[a].tile(), isles[b].tile());
        // Whichever way round crosses the less void.
        let along = span_len(&world, p, q, true) <= span_len(&world, p, q, false);
        bridge(&mut world, p, q, along);
    }

    let landing = isles[0];
    let citadel = isles[1];
    let start = landing.tile();
    // The citadel's floor laid with flagstones inside its walls.
    let (ccx, ccz) = citadel.tile();
    let (crx, crz) = (citadel.r.0 as i32 - 2, citadel.r.1 as i32 - 2);
    let inside_walls = |x: i32, z: i32| {
        let (dx, dz) = (
            (x - ccx) as f32 / crx.max(2) as f32,
            (z - ccz) as f32 / crz.max(2) as f32,
        );
        dx * dx + dz * dz < 1.0
    };
    for z in ccz - crz..=ccz + crz {
        for x in ccx - crx..=ccx + crx {
            if inside_walls(x, z) && world.floor(x, z) == Floor::Cave {
                world.set_floor(x, z, Floor::Cobble);
            }
        }
    }
    // Pale stone showing through the dark here and there.
    for z in 2..h - 2 {
        for x in 2..w - 2 {
            if world.floor(x, z) == Floor::Cave && noise(x as f32, z as f32, 3.0, ns ^ 0x9A1) > 0.55
            {
                world.set_floor(x, z, Floor::Path);
            }
        }
    }

    // The sanctum: a hall walled on three sides at the heart of the citadel, open towards
    // you, the stairs down inside it.
    let (sx0, sz0) = (ccx - 2, ccz - 2);
    let (sx1, sz1) = (ccx + 2, ccz + 1);
    for z in sz0 - 1..=sz1 {
        for x in sx0 - 1..=sx1 + 1 {
            let edge = z == sz0 - 1 || x == sx0 - 1 || x == sx1 + 1;
            world.set_floor(x, z, Floor::Cobble);
            if edge {
                let corner = z == sz0 - 1 && (x == sx0 - 1 || x == sx1 + 1);
                let look = if corner {
                    TOWER
                } else if z == sz0 - 1 {
                    RUNED
                } else {
                    BLOCKS
                };
                world.set_wall(x, z, Wall::Obsidian(look));
            }
        }
    }
    for x in sx0 - 1..=sx1 + 1 {
        world.set_floor(x, sz1 + 1, Floor::Cobble);
    }
    let stairs = (ccx, ccz - 1);
    world.set_obj(stairs.0, stairs.1, Some(Obj::StairsDown));
    let keep_clear = |x: i32, z: i32| {
        (x - start.0).abs() <= 1 && (z - start.1).abs() <= 2
            || (x - stairs.0).abs() <= 1 && (0..=1).contains(&(z - stairs.1))
    };
    // The portal you came through, just behind you on the Landing.
    world.set_obj(start.0, start.1 - 2, Some(Obj::RiftGate));

    // The citadel's broken outer walls, gapped where they would shut the way.
    let place_wall = |world: &mut World, x: i32, z: i32, wall: Wall| {
        if !open(world, x, z) || world.obj(x, z).is_some() || keep_clear(x, z) {
            return false;
        }
        let before = reach(world, start);
        world.set_wall(x, z, wall);
        if reach(world, start) + 1 < before {
            world.set_wall(x, z, Wall::None);
            return false;
        }
        true
    };
    let ring: Vec<(i32, i32)> = (ccz - crz - 1..=ccz + crz + 1)
        .flat_map(|z| (ccx - crx - 1..=ccx + crx + 1).map(move |x| (x, z)))
        .filter(|&(x, z)| {
            !inside_walls(x, z)
                && [(1, 0), (-1, 0), (0, 1), (0, -1)]
                    .iter()
                    .any(|&(dx, dz)| inside_walls(x + dx, z + dz))
        })
        .collect();
    for &(x, z) in &ring {
        // Broken here and there, and never over a bridge.
        if world.floor(x, z) == Floor::Span
            || noise(x as f32, z as f32, 3.0, ns ^ 0x3E) < -0.25
            || !ground(&world, x, z)
        {
            continue;
        }
        let look = match hash2(x, z, ns) % 10 {
            0 | 1 => CRACKED,
            2 => VEINED,
            3 => RUNED,
            _ => BLOCKS,
        };
        place_wall(&mut world, x, z, Wall::Obsidian(look));
    }
    // Towers where the wall turns.
    for &(x, z) in &ring {
        let corner = matches!(world.wall(x, z), Wall::Obsidian(_))
            && ((x - ccx).abs() >= crx || (z - ccz).abs() >= crz)
            && hash2(x, z, ns ^ 0x70) % 5 == 0;
        if corner {
            world.set_wall(x, z, Wall::Obsidian(TOWER));
        }
    }

    // Faces of the walls: runes glowing and eyes watching.
    let mut eyes = 0;
    for z in 2..h - 2 {
        for x in 2..w - 2 {
            if !under_face(&world, x, z) || keep_clear(x, z) {
                continue;
            }
            let sanctum = z == sz0 && (sx0..=sx1).contains(&x);
            let roll = r.f32();
            if sanctum || roll < 0.3 {
                let var = (hash2(x, z, ns ^ 0x7E) % 4) as u8;
                world.set_obj(x, z, Some(Obj::Runes { var }));
            } else if roll < 0.55 && eyes < 12 {
                world.set_obj(x, z, Some(Obj::WallEye));
                eyes += 1;
            }
        }
    }
    let free = |world: &World, x: i32, z: i32| {
        open(world, x, z)
            && world.obj(x, z).is_none()
            && world.floor(x, z) != Floor::Span
            && !keep_clear(x, z)
    };

    // What's on each island.
    for (k, isle) in isles.iter().enumerate() {
        let (cx, cz) = isle.tile();
        let (rx, rz) = (isle.r.0 as i32, isle.r.1 as i32);
        let mut spots: Vec<(i32, i32)> = (cz - rz..=cz + rz)
            .flat_map(|z| (cx - rx..=cx + rx).map(move |x| (x, z)))
            .filter(|&(x, z)| free(&world, x, z))
            .collect();
        r.shuffle(&mut spots);
        let mut spots = spots.into_iter();
        match isle.kind {
            Kind::Crystal => {
                for _ in 0..5 + r.below(5) {
                    let Some((x, z)) = spots.next() else { break };
                    let var = [SMALL, MID, MID, GREAT][r.below(4)];
                    if var == GREAT {
                        place_solid(&mut world, x, z, Obj::Amethyst { var }, start);
                    } else {
                        world.set_obj(x, z, Some(Obj::Amethyst { var }));
                    }
                }
            }
            Kind::Shrine => {
                if let Some((x, z)) = spots.find(|&(x, z)| (x - cx).abs() + (z - cz).abs() <= 1) {
                    let chest = Obj::LootChest {
                        opened: false,
                        gleam: r.chance(GLEAM_CHANCE * 2.0),
                    };
                    place_solid(&mut world, x, z, chest, start);
                }
                for _ in 0..2 {
                    if let Some((x, z)) = spots.next() {
                        world.set_obj(x, z, Some(Obj::Amethyst { var: SMALL }));
                    }
                }
            }
            Kind::Citadel => {
                // A chest tucked in a corner, columns (some fallen), and crystal breaking
                // through the flagstones.
                for (x, z) in spots.by_ref().take(40) {
                    if world.floor(x, z) != Floor::Cobble {
                        continue;
                    }
                    let roll = r.f32();
                    if roll < 0.07 {
                        let var = if r.chance(0.5) {
                            super::labyrinth::STANDING
                        } else {
                            super::labyrinth::BROKEN
                        };
                        place_solid(&mut world, x, z, Obj::Column { var }, start);
                    } else if roll < 0.12 {
                        world.set_obj(x, z, Some(Obj::Amethyst { var: SMALL }));
                    } else if roll < 0.16 {
                        world.set_obj(x, z, Some(Obj::Bones));
                    }
                }
                let nook = (ccz - crz..=ccz + crz)
                    .flat_map(|z| (ccx - crx..=ccx + crx).map(move |x| (x, z)))
                    .filter(|&(x, z)| {
                        free(&world, x, z)
                            && world.floor(x, z) == Floor::Cobble
                            && (x - stairs.0).abs() + (z - stairs.1).abs() > 4
                            && world.wall(x, z - 1) != Wall::None
                    })
                    .min_by_key(|&(x, z)| hash2(x, z, ns ^ 0xC0));
                if let Some((x, z)) = nook {
                    world.set_obj(x, z, None);
                    let chest = Obj::LootChest {
                        opened: false,
                        gleam: r.chance(GLEAM_CHANCE * 1.5),
                    };
                    place_solid(&mut world, x, z, chest, start);
                }
            }
            Kind::Rock | Kind::Landing => {
                for _ in 0..r.below(3) {
                    if let Some((x, z)) = spots.next() {
                        world.set_obj(x, z, Some(Obj::Amethyst { var: SMALL }));
                    }
                }
                if k > 0 && r.chance(0.5) {
                    if let Some((x, z)) = spots.next() {
                        place_solid(&mut world, x, z, Obj::Pot { hp: 1 }, start);
                    }
                }
            }
        }
    }

    // Voidfalls: blue light pouring off the islands' edges into the dark.
    let mut falls: Vec<(i32, i32)> = Vec::new();
    for _ in 0..400 {
        if falls.len() >= 5 {
            break;
        }
        let (x, z) = (r.range(3, w - 3), r.range(3, h - 3));
        let dir = [SOUTH, SOUTH, EAST, WEST][r.below(4)];
        let (dx, dz) = step(dir);
        let edge = free(&world, x, z)
            && world.floor(x, z) != Floor::Cobble
            && !ground(&world, x + dx, z + dz)
            && !ground(&world, x + dx * 2, z + dz * 2);
        let spaced = falls
            .iter()
            .all(|&(fx, fz)| (fx - x).abs() + (fz - z).abs() > 6);
        if edge && spaced {
            world.set_obj(x, z, Some(Obj::Voidfall { dir }));
            falls.push((x, z));
        }
    }

    // Great chains slung between the islands.
    let mut chains = 0;
    for _ in 0..600 {
        if chains >= 3 {
            break;
        }
        let (x, z) = (r.range(3, w - 3), r.range(3, h - 3));
        let dir = if r.chance(0.5) { EAST } else { SOUTH };
        let (dx, dz) = step(dir);
        if !free(&world, x, z) || ground(&world, x + dx, z + dz) {
            continue;
        }
        let mut len = 1;
        while len < 14
            && !ground(&world, x + dx * len, z + dz * len)
            && world.inside(x + dx * len, z + dz * len)
        {
            len += 1;
        }
        let (ex, ez) = (x + dx * len, z + dz * len);
        if (3..14).contains(&len) && free(&world, ex, ez) && world.floor(ex, ez) != Floor::Cobble {
            let chain = Obj::Chain {
                dir,
                len: len as u8,
            };
            if !place_solid(&mut world, x, z, chain, start) {
                continue;
            }
            if !place_solid(&mut world, ex, ez, Obj::ChainPost, start) {
                world.set_obj(x, z, None);
                continue;
            }
            chains += 1;
        }
    }

    // Now and then, a cracked flagstone over a secret room.
    let dist = walk(&world, start);
    let mut crack = None;
    if r.chance(CRACK_CHANCE) {
        for _ in 0..400 {
            let (x, z) = (r.range(3, w - 3), r.range(3, h - 3));
            let clear = (-1..=1).all(|dz| {
                (-1..=1).all(|dx| {
                    free(&world, x + dx, z + dz)
                        && matches!(world.floor(x + dx, z + dz), Floor::Cave | Floor::Path)
                })
            });
            if clear && dist[world.idx(x, z)] != u32::MAX {
                world.set_obj(x, z, Some(Obj::Crack));
                crack = Some((x, z));
                break;
            }
        }
    }

    // Who lives here.
    let foes = rift_folk(depth);
    let weights: Vec<f32> = foes.iter().map(|f| f.1).collect();
    let cap = (6 + depth as usize / 4).min(24);
    let mut spawns = Vec::new();
    for _ in 0..cap * 40 {
        if spawns.len() >= cap {
            break;
        }
        let (x, z) = (r.range(2, w - 2), r.range(2, h - 2));
        let d = dist[world.idx(x, z)];
        if d == u32::MAX || d < 12 || world.blocked(x, z) || keep_clear(x, z) {
            continue;
        }
        spawns.push(Spawn {
            foe: foes[r.weighted(&weights)].0,
            x: x as f32 + 0.5,
            z: z as f32 + 0.5,
            boss: false,
        });
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

    /// Some rifts to look at, whatever the chance says.
    fn rifts() -> Vec<Level> {
        [
            (1u64, 12u32),
            (7, 18),
            (42, 25),
            (9, 47),
            (123, 68),
            (5, 99),
        ]
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
    fn in_through_the_portal_and_down_from_the_sanctum() {
        for l in rifts() {
            let w = &l.world;
            assert!(w.rift);
            assert!(!w.blocked(l.start.0, l.start.1));
            assert!(matches!(
                w.obj(l.start.0, l.start.1 - 2),
                Some(Obj::RiftGate)
            ));
            assert!(matches!(
                w.obj(l.stairs.0, l.stairs.1),
                Some(Obj::StairsDown)
            ));
            assert!(
                matches!(w.wall(l.stairs.0, l.stairs.1 - 2), Wall::Obsidian(_)),
                "under the sanctum's back wall"
            );
            let dist = walk(w, l.start);
            let far = dist[w.idx(l.stairs.0, l.stairs.1)];
            assert!(far != u32::MAX, "the stairs can be reached");
            assert!(far > 25, "a walk across the islands, not {far}");
        }
    }

    #[test]
    fn islands_over_the_void_strung_with_bridges() {
        for l in rifts() {
            let w = &l.world;
            let area = (w.w * w.h) as usize;
            let void = count(&l, |w, x, z| w.floor(x, z) == Floor::Void);
            assert!(void > area * 2 / 5, "{void} of {area} void");
            assert!(count(&l, |w, x, z| w.floor(x, z) == Floor::Span) > 20);
            assert!(
                count(&l, |w, x, z| w.floor(x, z) == Floor::Path) > 5,
                "pale stone"
            );
            // Nothing on the edge of the world.
            for x in 0..w.w {
                assert!(!ground(w, x, 0) && !ground(w, x, w.h - 1));
            }
        }
    }

    #[test]
    fn nothing_is_cut_off() {
        for l in rifts() {
            let w = &l.world;
            let open_tiles = count(&l, |w, x, z| !w.blocked(x, z));
            assert_eq!(
                reach(w, l.start),
                open_tiles,
                "every island can be walked to"
            );
        }
    }

    #[test]
    fn the_citadel_is_carved_and_watched() {
        for l in rifts() {
            let w = &l.world;
            let obsidian = count(&l, |w, x, z| matches!(w.wall(x, z), Wall::Obsidian(_)));
            assert!(obsidian > 20, "{obsidian} walls");
            let runes = count(&l, |w, x, z| matches!(w.obj(x, z), Some(Obj::Runes { .. })));
            assert!(runes >= 3, "{runes} runes");
            let eyes = count(&l, |w, x, z| matches!(w.obj(x, z), Some(Obj::WallEye)));
            assert!(eyes >= 1, "{eyes} eyes");
            for (o, x, z) in (0..w.h)
                .flat_map(|z| (0..w.w).map(move |x| (x, z)))
                .filter_map(|(x, z)| w.obj(x, z).map(|o| (o, x, z)))
            {
                if matches!(o, Obj::Runes { .. } | Obj::WallEye) {
                    assert!(w.wall(x, z - 1) != Wall::None, "on a wall's face");
                }
            }
            let crystals = count(&l, |w, x, z| {
                matches!(w.obj(x, z), Some(Obj::Amethyst { .. }))
            });
            assert!(crystals >= 5, "{crystals} amethysts");
            let chests = count(&l, |w, x, z| {
                matches!(w.obj(x, z), Some(Obj::LootChest { .. }))
            });
            assert!(chests >= 2, "{chests} chests");
        }
    }

    #[test]
    fn voidfalls_and_chains_hang_over_the_void() {
        let mut falls = 0;
        let mut chains = 0;
        for l in rifts() {
            let w = &l.world;
            for z in 0..w.h {
                for x in 0..w.w {
                    match w.obj(x, z) {
                        Some(Obj::Voidfall { dir }) => {
                            let (dx, dz) = step(*dir);
                            assert!(!ground(w, x + dx, z + dz), "pouring into the void");
                            falls += 1;
                        }
                        Some(Obj::Chain { dir, len }) => {
                            let (dx, dz) = step(*dir);
                            let len = *len as i32;
                            for k in 1..len {
                                assert!(!ground(w, x + dx * k, z + dz * k), "slung over the void");
                            }
                            assert!(matches!(
                                w.obj(x + dx * len, z + dz * len),
                                Some(Obj::ChainPost)
                            ));
                            chains += 1;
                        }
                        _ => {}
                    }
                }
            }
        }
        assert!(falls >= 12, "{falls} voidfalls");
        assert!(chains >= 5, "{chains} chains");
    }

    #[test]
    fn deterministic() {
        let a = generate(77, 23, 1);
        let b = generate(77, 23, 1);
        assert_eq!(a.start, b.start);
        assert_eq!(a.stairs, b.stairs);
        assert_eq!(a.world.floor, b.world.floor);
        assert_eq!(a.world.wall, b.world.wall);
    }

    #[test]
    fn often_in_the_crystal_grotto_never_too_soon() {
        let count = |biome: usize| (0..800u64).filter(|&seed| is_rift(seed, 23, biome)).count();
        let (grotto, frost) = (count(GROTTO), count(4));
        assert!(grotto > frost * 2, "{grotto} vs {frost}");
        assert!(frost > 10, "{frost}");
        assert!(
            !(0..400u64).any(|seed| is_rift(seed, 20, 1)),
            "never a guardian's"
        );
        assert!(!(0..400u64).any(|seed| is_rift(seed, 11, 1)), "not so soon");
        for seed in 0..300u64 {
            for depth in 12..60 {
                let biome = biome_for(seed, depth);
                if is_rift(seed, depth, biome) {
                    assert!(!super::super::canyon::is_canyon(seed, depth, biome));
                    assert!(!super::super::labyrinth::is_labyrinth(seed, depth, biome));
                }
            }
        }
    }
}
