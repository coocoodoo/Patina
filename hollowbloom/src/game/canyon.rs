//! The Sunscorch Canyon. Now and then a floor of the Hollow, from floor 6 down (and often in
//! the Sunken Ruins), is a deep canyon of red sandstone, its cliffs rising in layered
//! terraces over drifts of sand. You come in by an ancient stone gateway, and a torch-lit
//! sandy hall winds away into the canyon: to the Bone Warrens, a tangle of low ridges strewn
//! with bones and great skulls; to the Sunscorch Pit, a sunken basin where sand pours off
//! tall pillars and quicksand drags at your feet; to the Wyvern's Maw, where a great
//! wyvern's bones lie over its hoard; and to the Whispering Tombs, carved into the rock,
//! where the stairs lead down.

use super::dungeon::{
    CRACK_CHANCE, DEEP_FOLK, Foe, GLEAM_CHANCE, Level, Spawn, deep_folk, is_waystone_floor,
    too_tough,
};
use super::glowcave::{curve, noise, open, place_solid, reach, walk};
use super::world::{Area, Floor, Obj, Wall, World};
use crate::util::{Rng, hash2};

/// How often a floor is the canyon: now and then anywhere, and often in the Sunken Ruins.
/// Never the first few floors, a guardian's floor, or where another kind of floor is.
pub const CANYON_CHANCE: f32 = 0.07;
pub const RUINS_CHANCE: f32 = 0.3;
pub const FIRST_CANYON: u32 = 6;
/// How much quicksand slows whoever's wading through it.
pub const QUICKSAND_DRAG: f32 = 0.5;
/// The canyon's folk are dressed for the desert, as the Sunken Ruins' are: mummies,
/// pharaoh's guards, scarabs and the like.
pub const DESERT: usize = 5;

/// How a tile of sandstone looks (see `Wall::Sandstone`): layered strata, a paler banding,
/// or the tombs' carved blocks.
pub const STRATA: u8 = 0;
pub const BANDED: u8 = 1;
pub const TOMB: u8 = 2;
pub const LOOKS: usize = 3;

/// A tile of sandstone of a look, rising `rise` terraces (half a tile each) above the rim.
pub fn sandstone(look: u8, rise: u8) -> Wall {
    Wall::Sandstone(look * 4 + rise.min(3))
}

/// How tall a tile of sandstone stands.
pub fn terrace_height(v: u8) -> f32 {
    1.0 + 0.5 * (v % 4) as f32
}

pub fn sandstone_look(v: u8) -> usize {
    (v / 4) as usize % LOOKS
}

/// Skulls: a great horned one, a little pile of them among bones (which you can walk over),
/// and a fanged monster's.
pub const HORNED: u8 = 0;
pub const SKULL_PILE: u8 = 1;
pub const FANGED: u8 = 2;

/// Cacti: tall with arms, a round ribbed barrel, and a clump of paddles.
pub const SAGUARO: u8 = 0;
pub const BARREL: u8 = 1;
pub const PADDLE: u8 = 2;

/// The wyvern's bones lie over a block of tiles this big, anchored at its north-west.
pub const WYVERN_W: i32 = 3;
pub const WYVERN_H: i32 = 2;

/// Is this floor the canyon?
pub fn is_canyon(seed: u64, depth: u32, biome: usize) -> bool {
    let chance = if biome == DESERT {
        RUINS_CHANCE
    } else {
        CANYON_CHANCE
    };
    depth >= FIRST_CANYON
        && !is_waystone_floor(depth)
        && !super::sewer::is_sewer(seed, depth)
        && !super::glowcave::is_glowcave(seed, depth, biome)
        && !super::labyrinth::is_labyrinth(seed, depth, biome)
        && Rng::new(seed ^ (depth as u64).wrapping_mul(0x27D4_EB2F) ^ 0xCA7).chance(chance)
}

/// Who lives in the canyon, with weights: the desert's walking dead, bones, goblins, bugs
/// and beetles, and from floor 11 down the deep folk too.
pub fn canyon_folk(depth: u32) -> Vec<(Foe, f32)> {
    let mut v = vec![
        (Foe::Zombie, 1.4),
        (Foe::Skeleton, 1.5),
        (Foe::Bug, 1.8),
        (Foe::Beetle, 1.1),
        (Foe::Sneak, 1.2),
        (Foe::Brute, 0.8),
        (Foe::Ghost, 0.7),
    ];
    if depth >= DEEP_FOLK {
        v.extend_from_slice(deep_folk(DESERT));
    }
    v
}

/// Canyon rock you could dig a passage through (not the tombs' masonry).
fn rock(w: &World, x: i32, z: i32) -> bool {
    matches!(w.wall(x, z), Wall::Sandstone(v) if sandstone_look(v) != TOMB as usize)
}

/// Digs the sandstone out in a disc, down to the sand (never the tombs' masonry, nor the
/// edge of the world).
fn dig(w: &mut World, cx: f32, cz: f32, rad: f32) {
    let r = rad.ceil() as i32 + 1;
    let (tx, tz) = (cx.floor() as i32, cz.floor() as i32);
    for z in tz - r..=tz + r {
        for x in tx - r..=tx + r {
            let d = (x as f32 + 0.5 - cx).hypot(z as f32 + 0.5 - cz);
            if d <= rad && x >= 3 && z >= 3 && x < w.w - 3 && z < w.h - 3 && rock(w, x, z) {
                w.set_wall(x, z, Wall::None);
            }
        }
    }
}

/// Digs out an oval with a ragged edge.
fn bowl(w: &mut World, (cx, cz): (f32, f32), (rx, rz): (f32, f32), ragged: f32, seed: u32) {
    for z in 3..w.h - 3 {
        for x in 3..w.w - 3 {
            let (dx, dz) = ((x as f32 + 0.5 - cx) / rx, (z as f32 + 0.5 - cz) / rz);
            if dx * dx + dz * dz < 1.0 + ragged * noise(x as f32, z as f32, 3.0, seed) {
                dig(w, x as f32 + 0.5, z as f32 + 0.5, 0.5);
            }
        }
    }
}

/// Carves a winding passage along a curve through `pts`, `rad` wide give or take `wobble`,
/// and hands back the points along it.
fn passage(
    w: &mut World,
    pts: &[(f32, f32)],
    rad: f32,
    wobble: f32,
    r: &mut Rng,
) -> Vec<(f32, f32, f32)> {
    let (ph1, ph2) = (r.f32() * 6.3, r.f32() * 6.3);
    let mut path: Vec<(f32, f32, f32)> = Vec::new();
    let mut along = 0.0f32;
    for i in 0..pts.len() - 1 {
        let (a, b, c, d) = (
            pts[i.saturating_sub(1)],
            pts[i],
            pts[i + 1],
            pts[(i + 2).min(pts.len() - 1)],
        );
        let steps = ((b.0 - c.0).hypot(b.1 - c.1) / 0.35).ceil() as usize;
        for s in 0..steps.max(1) {
            let p = curve(a, b, c, d, s as f32 / steps.max(1) as f32);
            if let Some(&(qx, qz, _)) = path.last() {
                along += (p.0 - qx).hypot(p.1 - qz);
            }
            let rr =
                rad + wobble * (0.6 * (along * 0.4 + ph1).sin() + 0.4 * (along * 0.15 + ph2).sin());
            path.push((p.0, p.1, rr));
        }
    }
    if let Some(&last) = pts.last() {
        path.push((last.0, last.1, rad));
    }
    for &(x, z, rr) in &path {
        dig(w, x, z, rr);
    }
    path
}

/// A tile of floor right under a rock face (the wall to its north): somewhere for a
/// torch.
fn under_face(w: &World, x: i32, z: i32) -> bool {
    open(w, x, z) && w.obj(x, z).is_none() && w.wall(x, z - 1) != Wall::None
}

/// Next to a rock wall, somewhere for a cactus, a skull or a heap of bones.
fn by_rock(w: &World, x: i32, z: i32) -> bool {
    [(1, 0), (-1, 0), (0, 1), (0, -1)]
        .iter()
        .any(|&(dx, dz)| w.wall(x + dx, z + dz) != Wall::None)
}

pub fn generate(seed: u64, depth: u32, biome: usize) -> Level {
    let mut r = Rng::new(seed ^ (depth as u64).wrapping_mul(0x9E37_79B9) ^ 0xCA7_CA7);
    let ns = hash2(depth as i32, 0xCA7, seed as u32);
    let grow = depth.min(40) as i32;
    let (w, h) = (62 + grow * 2 / 3, 44 + grow / 2);
    let mut world = World::new(w, h, Area::Hollow { depth }, biome);
    world.canyon = true;
    for z in 0..h {
        for x in 0..w {
            world.set_floor(x, z, Floor::Sand);
            let look = if noise(x as f32, z as f32, 7.0, ns ^ 0xB) > 0.2 {
                BANDED
            } else {
                STRATA
            };
            world.set_wall(x, z, sandstone(look, 0));
        }
    }
    // Laid out with the gateway and the warrens to the west and the tombs and the maw to
    // the east, or the other way round.
    let flip = r.chance(0.5);
    let east = if flip { -1.0 } else { 1.0 };
    let side = |x: f32| if flip { w as f32 - x } else { x };
    let (wf, hf) = (w as f32, h as f32);

    // The Whispering Tombs: four chambers of carved blocks in the north-east, floored with
    // old flagstones, with a door on the canyon's side.
    let (tw, th) = ((wf * 0.26) as i32, (hf * 0.34) as i32);
    let tz0 = 4;
    let tx0 = if flip { 4 } else { w - 4 - tw };
    let (tx1, tz1) = (tx0 + tw - 1, tz0 + th - 1);
    let (mx, mz) = (tx0 + tw / 2, tz0 + th / 2);
    let mut tomb = vec![false; (w * h) as usize];
    for z in tz0..=tz1 {
        for x in tx0..=tx1 {
            let edge = x == tx0 || x == tx1 || z == tz0 || z == tz1 || x == mx || z == mz;
            if edge {
                world.set_wall(x, z, sandstone(TOMB, 0));
            } else {
                world.set_wall(x, z, Wall::None);
                world.set_floor(x, z, Floor::Walkway);
                tomb[(z * w + x) as usize] = true;
            }
        }
    }
    // The chambers: west or east of the middle wall, north or south of it.
    let chamber = |west: bool, north: bool| {
        let (x0, x1) = if west {
            (tx0 + 1, mx - 1)
        } else {
            (mx + 1, tx1 - 1)
        };
        let (z0, z1) = if north {
            (tz0 + 1, mz - 1)
        } else {
            (mz + 1, tz1 - 1)
        };
        (x0, z0, x1, z1)
    };
    // In by the chamber nearest the canyon (south, on the canyon's side), and out down the
    // stairs in the one diagonally across from it.
    let near_west = !flip;
    let entry = chamber(near_west, false);
    let far = chamber(!near_west, true);
    let door_x = r.range(entry.0 + 1, entry.2);
    for dx in 0..2 {
        world.set_wall(door_x + dx, tz1, Wall::None);
        world.set_floor(door_x + dx, tz1, Floor::Walkway);
        tomb[(tz1 * w + door_x + dx) as usize] = true;
    }
    // Doorways between the chambers, all round.
    let (wx0, wx1) = (tx0 + 1, mx - 1);
    let (ex0, ex1) = (mx + 1, tx1 - 1);
    let (nz0, nz1) = (tz0 + 1, mz - 1);
    let (sz0, sz1) = (mz + 1, tz1 - 1);
    let mut doorway = |world: &mut World, x: i32, z: i32| {
        world.set_wall(x, z, Wall::None);
        world.set_floor(x, z, Floor::Walkway);
        tomb[(z * w + x) as usize] = true;
    };
    for (x0, x1) in [(wx0, wx1), (ex0, ex1)] {
        let x = r.range(x0, x1 + 1);
        doorway(&mut world, x, mz);
    }
    for (z0, z1) in [(nz0, nz1), (sz0, sz1)] {
        if r.chance(0.75) || z0 == nz0 {
            let z = r.range(z0, z1 + 1);
            doorway(&mut world, mx, z);
        }
    }

    // The gateway you come in by, in the north-west, and a little sandy clearing round it.
    let (ex, ez) = (side(7.5), 6.5 + r.range(0, 3) as f32);
    for (dx, dz, rad) in [(0.0, 0.0, 3.0), (east * 1.5, 1.2, 2.6)] {
        dig(&mut world, ex + dx, ez + dz, rad);
    }
    // The Sunscorch Pit in the middle, with its sandfalls.
    let (px, pz) = (wf * 0.5 + r.range(-2, 3) as f32, hf * 0.56);
    let (prx, prz) = (8.0 + grow as f32 / 8.0, 5.5 + grow as f32 / 12.0);
    bowl(&mut world, (px, pz), (prx, prz), 0.2, ns ^ 0x5A);
    // The Wyvern's Maw in the south-east.
    let (mawx, mawz) = (side(wf * 0.8), hf * 0.74);
    let (mrx, mrz) = (6.0 + grow as f32 / 14.0, 4.4 + grow as f32 / 18.0);
    bowl(&mut world, (mawx, mawz), (mrx, mrz), 0.18, ns ^ 0x3A);
    // The Bone Warrens in the south-west: open ground to be filled with ridges.
    let (ww, wz0) = ((wf * 0.27) as i32, (hf * 0.5) as i32);
    let wh = h - 5 - wz0;
    let wx0 = if flip { w - 5 - ww } else { 5 };
    for z in wz0..wz0 + wh {
        for x in wx0..wx0 + ww {
            let ragged = noise(x as f32, z as f32, 2.5, ns ^ 0x77) * 0.8;
            let inset = (x - wx0)
                .min(wx0 + ww - 1 - x)
                .min(z - wz0)
                .min(wz0 + wh - 1 - z);
            if inset as f32 + ragged > 0.3 {
                dig(&mut world, x as f32 + 0.5, z as f32 + 0.5, 0.5);
            }
        }
    }
    let warrens = |x: i32, z: i32| x >= wx0 && x < wx0 + ww && z >= wz0 && z < wz0 + wh;

    // The passages: the Sandrift Halls from the gateway round to the pit, and on from the
    // pit to the warrens, the maw and the tombs' door, with a way down to the warrens from
    // the halls too.
    let pit_at = |a: f32, k: f32| (px + prx * k * a.cos(), pz + prz * k * a.sin());
    let halls_mid = (side(wf * 0.3), ez + r.range(-1, 3) as f32);
    let halls = passage(
        &mut world,
        &[
            (ex + east * 2.5, ez + 0.5),
            halls_mid,
            (side(wf * 0.4), pz - prz - 1.5),
            (px - east * prx * 0.45, pz - prz * 0.7),
        ],
        1.9,
        0.5,
        &mut r,
    );
    let west_a = if flip {
        0.15
    } else {
        std::f32::consts::PI - 0.15
    };
    let wmid = (wx0 as f32 + ww as f32 * 0.5, wz0 as f32 + wh as f32 * 0.5);
    passage(
        &mut world,
        &[
            pit_at(west_a, 0.8),
            (
                side(if flip {
                    wf - (wx0 as f32)
                } else {
                    wx0 as f32 + ww as f32 + 1.0
                }),
                wmid.1 - 1.0,
            ),
            wmid,
        ],
        1.6,
        0.35,
        &mut r,
    );
    passage(
        &mut world,
        &[
            halls_mid,
            (halls_mid.0, wz0 as f32 + 2.0),
            (wmid.0, wz0 as f32 + 3.0),
        ],
        1.5,
        0.35,
        &mut r,
    );
    let east_a = if flip {
        std::f32::consts::PI - 0.4
    } else {
        0.4
    };
    passage(
        &mut world,
        &[pit_at(east_a, 0.8), (mawx - east * mrx * 0.6, mawz)],
        1.7,
        0.4,
        &mut r,
    );
    let door_mid = door_x as f32 + 1.0;
    let north_a = if flip {
        std::f32::consts::PI + 0.9
    } else {
        -0.9
    };
    passage(
        &mut world,
        &[
            pit_at(north_a, 0.8),
            (door_mid, tz1 as f32 + 3.5),
            (door_mid, tz1 as f32 + 1.2),
        ],
        1.5,
        0.3,
        &mut r,
    );
    // Round off the rock between them a little.
    let snapshot = world.wall.clone();
    for z in 3..h - 3 {
        for x in 3..w - 3 {
            if !rock(&world, x, z) {
                continue;
            }
            let near = (-1..=1)
                .flat_map(|dz: i32| (-1..=1).map(move |dx: i32| (dx, dz)))
                .filter(|&(dx, dz)| snapshot[((z + dz) * w + x + dx) as usize] == Wall::None)
                .count();
            if near >= 5 {
                world.set_wall(x, z, Wall::None);
            }
        }
    }

    // The gateway, on the north side of its clearing, and in just south of it.
    let (gx, gz) = (ex.floor() as i32, ez.floor() as i32 - 2);
    for dx in -1..=1 {
        world.set_wall(gx + dx, gz, Wall::None);
        world.set_obj(gx + dx, gz, None);
    }
    world.set_obj(gx, gz, Some(Obj::Gateway));
    for dx in [-1, 1] {
        world.set_obj(
            gx + dx,
            gz,
            Some(Obj::Part {
                ax: gx as i16,
                az: gz as i16,
            }),
        );
    }
    let start = (gx, gz + 2);

    // The Bone Warrens' ridges: low walls of rock this way and that, never shutting any
    // ground off.
    let mut ridges = 0;
    let want = (ww * wh / 11) as usize;
    for _ in 0..600 {
        if ridges >= want {
            break;
        }
        let (x, z) = (
            r.range(wx0 + 1, wx0 + ww - 1),
            r.range(wz0 + 1, wz0 + wh - 1),
        );
        let (dx, dz) = if r.chance(0.5) { (1, 0) } else { (0, 1) };
        let len = r.range(2, 6);
        let tiles: Vec<(i32, i32)> = (0..len).map(|k| (x + dx * k, z + dz * k)).collect();
        let fits = tiles.iter().all(|&(tx, tz)| {
            warrens(tx, tz)
                && open(&world, tx, tz)
                && (tx - start.0).abs() + (tz - start.1).abs() > 4
                && [(1, 0), (-1, 0), (0, 1), (0, -1)]
                    .iter()
                    .filter(|&&(ox, oz)| !tiles.contains(&(tx + ox, tz + oz)))
                    .all(|&(ox, oz)| open(&world, tx + ox, tz + oz))
        });
        if !fits {
            continue;
        }
        let before = reach(&world, start);
        for &(tx, tz) in &tiles {
            world.set_wall(tx, tz, sandstone(STRATA, 0));
        }
        if reach(&world, start) + tiles.len() < before {
            for &(tx, tz) in &tiles {
                world.set_wall(tx, tz, Wall::None);
            }
        } else {
            ridges += tiles.len();
        }
    }

    // The canyon walls rise in terraces the further they are from the canyon floor (but
    // never so tall, just south of it, as to hide what's on it).
    let mut dist = vec![u8::MAX; (w * h) as usize];
    let mut q = std::collections::VecDeque::new();
    for z in 0..h {
        for x in 0..w {
            if world.wall(x, z) == Wall::None {
                dist[(z * w + x) as usize] = 0;
                q.push_back((x, z));
            }
        }
    }
    while let Some((x, z)) = q.pop_front() {
        let d = dist[(z * w + x) as usize];
        if d >= 4 {
            continue;
        }
        for dz in -1..=1 {
            for dx in -1..=1 {
                let (nx, nz) = (x + dx, z + dz);
                if world.inside(nx, nz) && dist[(nz * w + nx) as usize] == u8::MAX {
                    dist[(nz * w + nx) as usize] = d + 1;
                    q.push_back((nx, nz));
                }
            }
        }
    }
    for z in 0..h {
        for x in 0..w {
            let Wall::Sandstone(v) = world.wall(x, z) else {
                continue;
            };
            let look = sandstone_look(v) as u8;
            if look == TOMB {
                continue;
            }
            let d = dist[(z * w + x) as usize].min(4);
            let over = (1..=4).find(|&k| world.wall(x, z - k) == Wall::None);
            let cap = match over {
                Some(1 | 2) => 0,
                Some(3) => 2,
                _ => 3,
            };
            let rise = (d.saturating_sub(1)).min(cap).min(3);
            world.set_wall(x, z, sandstone(look, rise));
        }
    }

    // Sand everywhere but the tombs, and in the pit patches of quicksand.
    let mut quick = 0;
    for _ in 0..40 {
        if quick >= 2 + r.below(2) {
            break;
        }
        let a = r.f32() * std::f32::consts::TAU;
        let k = r.range_f(0.2, 0.6);
        let (cx, cz) = pit_at(a, k);
        let rad = r.range_f(1.2, 2.0);
        let mut n = 0;
        for z in (cz - rad) as i32 - 1..=(cz + rad) as i32 + 1 {
            for x in (cx - rad) as i32 - 1..=(cx + rad) as i32 + 1 {
                let d = (x as f32 + 0.5 - cx).hypot(z as f32 + 0.5 - cz)
                    + noise(x as f32, z as f32, 1.5, ns ^ 0x9) * 0.4;
                let clear = (-1..=1).all(|oz: i32| {
                    (-1..=1).all(|ox: i32| world.wall(x + ox, z + oz) == Wall::None)
                });
                if d < rad && clear && world.floor(x, z) == Floor::Sand {
                    world.set_floor(x, z, Floor::Quicksand);
                    n += 1;
                }
            }
        }
        if n > 0 {
            quick += 1;
        }
    }

    // Down the stairs in the far chamber of the tombs.
    let stairs = ((far.0 + far.2) / 2, (far.1 + far.3) / 2);
    world.set_obj(stairs.0, stairs.1, Some(Obj::StairsDown));
    let keep_clear = |x: i32, z: i32| {
        (x - start.0).abs() <= 1 && (z - start.1).abs() <= 1
            || (x - stairs.0).abs() <= 1 && (z - stairs.1).abs() <= 1
    };
    let free = |world: &World, x: i32, z: i32| {
        open(world, x, z)
            && world.obj(x, z).is_none()
            && world.floor(x, z) != Floor::Quicksand
            && !keep_clear(x, z)
    };

    // The tombs: stone coffins, broken columns in the corners, torches, canopic jars,
    // bones, and a chest.
    let chambers = [
        chamber(true, true),
        chamber(false, true),
        chamber(true, false),
        chamber(false, false),
    ];
    let chest_in = r.below(4);
    let gold_in = r.below(4);
    let skulls_in = (gold_in + 1 + r.below(3)) % 4;
    for (k, &(x0, z0, x1, z1)) in chambers.iter().enumerate() {
        for (cx, cz) in [(x0, z0), (x1, z0), (x0, z1), (x1, z1)] {
            if r.chance(0.55) && free(&world, cx, cz) {
                let var = if r.chance(0.5) {
                    super::labyrinth::STANDING
                } else {
                    super::labyrinth::BROKEN
                };
                place_solid(&mut world, cx, cz, Obj::Column { var }, start);
            }
        }
        for _ in 0..1 + r.below(2) {
            let (x, z) = (r.range(x0 + 1, x1), z0 + 1);
            if free(&world, x, z) {
                place_solid(&mut world, x, z, Obj::Sarcophagus, start);
            }
        }
        for x in x0..=x1 {
            if under_face(&world, x, z0) && r.chance(0.3) && !keep_clear(x, z0) {
                world.set_obj(x, z0, Some(Obj::Sconce));
            }
        }
        let mut spots: Vec<(i32, i32)> = (z0..=z1)
            .flat_map(|z| (x0..=x1).map(move |x| (x, z)))
            .filter(|&(x, z)| by_rock(&world, x, z))
            .collect();
        r.shuffle(&mut spots);
        let mut spots = spots.into_iter();
        if k == chest_in {
            for (x, z) in spots.by_ref() {
                let chest = Obj::LootChest {
                    opened: false,
                    gleam: r.chance(GLEAM_CHANCE * 1.5),
                };
                if free(&world, x, z) && place_solid(&mut world, x, z, chest, start) {
                    break;
                }
            }
        }
        for _ in 0..r.range(1, 4) {
            let Some((x, z)) = spots.next() else { break };
            if free(&world, x, z) {
                place_solid(&mut world, x, z, Obj::Pot { hp: 1 }, start);
            }
        }
        if let Some((x, z)) = spots.next() {
            if free(&world, x, z) {
                world.set_obj(x, z, Some(Obj::Bones));
            }
        }
        // Grave goods heaped in one chamber, and skulls in another.
        let offering = if k == gold_in {
            Some(Obj::GoldPile)
        } else if k == skulls_in {
            Some(Obj::Skull { var: SKULL_PILE })
        } else {
            None
        };
        if let Some(o) = offering {
            for _ in 0..12 {
                let (x, z) = (r.range(x0 + 1, x1), r.range(z0 + 1, z1));
                if free(&world, x, z) && (x - stairs.0).abs() + (z - stairs.1).abs() > 2 {
                    world.set_obj(x, z, Some(o));
                    break;
                }
            }
        }
    }

    // The pit's sandfalls, pouring off tall pillars along its north side.
    let mut falls = 0;
    for _ in 0..40 {
        if falls >= 3 {
            break;
        }
        let a = -std::f32::consts::FRAC_PI_2 + r.range_f(-0.9, 0.9);
        let (cx, cz) = pit_at(a, r.range_f(0.55, 0.8));
        let (x, z) = (cx.floor() as i32, cz.floor() as i32);
        let spaced = (-2..=2).all(|oz: i32| {
            (-2..=2).all(|ox: i32| !matches!(world.obj(x + ox, z + oz), Some(Obj::Sandfall { .. })))
        });
        if spaced
            && free(&world, x, z)
            && world.floor(x, z) == Floor::Sand
            && place_solid(&mut world, x, z, Obj::Sandfall { var: falls as u8 }, start)
        {
            falls += 1;
        }
    }

    // The Wyvern's Maw: the great bones in the middle, its gold heaped round them, a chest
    // or two at the back, and skulls.
    let (wyx, wyz) = (mawx.round() as i32 - 1, mawz.round() as i32 - 1);
    let footprint: Vec<(i32, i32)> = (0..WYVERN_H)
        .flat_map(|dz| (0..WYVERN_W).map(move |dx| (wyx + dx, wyz + dz)))
        .collect();
    if footprint.iter().all(|&(x, z)| free(&world, x, z)) {
        let before = reach(&world, start);
        world.set_obj(wyx, wyz, Some(Obj::Wyvern));
        for &(x, z) in &footprint[1..] {
            world.set_obj(
                x,
                z,
                Some(Obj::Part {
                    ax: wyx as i16,
                    az: wyz as i16,
                }),
            );
        }
        if reach(&world, start) + footprint.len() < before {
            for &(x, z) in &footprint {
                world.set_obj(x, z, None);
            }
        }
    }
    let in_maw = |x: i32, z: i32| {
        let (dx, dz) = ((x as f32 + 0.5 - mawx) / mrx, (z as f32 + 0.5 - mawz) / mrz);
        dx * dx + dz * dz < 0.85
    };
    let mut gold = 0;
    for _ in 0..80 {
        if gold >= 6 {
            break;
        }
        let (x, z) = (
            wyx + r.range(-2, WYVERN_W + 2),
            wyz + r.range(-1, WYVERN_H + 2),
        );
        if in_maw(x, z) && free(&world, x, z) {
            world.set_obj(x, z, Some(Obj::GoldPile));
            gold += 1;
        }
    }
    let mut chests = 1 + usize::from(r.chance(0.5));
    for _ in 0..200 {
        if chests == 0 {
            break;
        }
        let (x, z) = (
            mawx as i32 + r.range(-(mrx as i32), mrx as i32 + 1),
            mawz as i32 + r.range(-(mrz as i32), mrz as i32 + 1),
        );
        let chest = Obj::LootChest {
            opened: false,
            gleam: r.chance(GLEAM_CHANCE * 2.0),
        };
        if in_maw(x, z)
            && by_rock(&world, x, z)
            && free(&world, x, z)
            && place_solid(&mut world, x, z, chest, start)
        {
            chests -= 1;
        }
    }

    // The warrens, strewn with bones and skulls, a chest hidden deep in them.
    let mut chest_hidden = false;
    let far_walk = walk(&world, start);
    let mut warren_tiles: Vec<(u32, i32, i32)> = Vec::new();
    for z in wz0..wz0 + wh {
        for x in wx0..wx0 + ww {
            let d = far_walk[world.idx(x, z)];
            if d != u32::MAX && free(&world, x, z) {
                warren_tiles.push((d, x, z));
            }
        }
    }
    warren_tiles.sort_by(|a, b| b.0.cmp(&a.0));
    for &(_, x, z) in warren_tiles.iter().take(12) {
        if !chest_hidden && by_rock(&world, x, z) {
            let chest = Obj::LootChest {
                opened: false,
                gleam: r.chance(GLEAM_CHANCE),
            };
            if place_solid(&mut world, x, z, chest, start) {
                chest_hidden = true;
            }
        }
    }
    for z in wz0..wz0 + wh {
        for x in wx0..wx0 + ww {
            if !free(&world, x, z) {
                continue;
            }
            let roll = r.f32();
            if roll < 0.1 {
                world.set_obj(x, z, Some(Obj::Bones));
            } else if roll < 0.14 {
                world.set_obj(x, z, Some(Obj::Skull { var: SKULL_PILE }));
            } else if roll < 0.165 && by_rock(&world, x, z) {
                let var = if r.chance(0.5) { HORNED } else { FANGED };
                place_solid(&mut world, x, z, Obj::Skull { var }, start);
            }
        }
    }

    // Everywhere else: torches along the halls, cacti by the canyon walls, bones and skulls
    // lying about, and big skulls round the maw.
    for (k, &(hx, hz, _)) in halls.iter().enumerate() {
        if k % 14 != 7 {
            continue;
        }
        let (x0, z0) = (hx.floor() as i32, hz.floor() as i32);
        'torch: for dz in -3..=0 {
            for dx in [0, 1, -1] {
                let (x, z) = (x0 + dx, z0 + dz);
                if under_face(&world, x, z) && !keep_clear(x, z) && !tomb[world.idx(x, z)] {
                    world.set_obj(x, z, Some(Obj::Sconce));
                    break 'torch;
                }
            }
        }
    }
    for z in 2..h - 2 {
        for x in 2..w - 2 {
            let i = world.idx(x, z);
            if tomb[i] || warrens(x, z) || !free(&world, x, z) {
                continue;
            }
            let wall = by_rock(&world, x, z);
            let roll = r.f32();
            if wall && roll < 0.045 {
                let var = match r.below(10) {
                    0..=4 => SAGUARO,
                    5..=7 => BARREL,
                    _ => PADDLE,
                };
                place_solid(&mut world, x, z, Obj::Cactus { var }, start);
            } else if roll < 0.06 {
                world.set_obj(x, z, Some(Obj::Bones));
            } else if roll < 0.07 {
                world.set_obj(x, z, Some(Obj::Skull { var: SKULL_PILE }));
            } else if wall && roll < 0.085 && in_maw(x, z) {
                let var = if r.chance(0.5) { HORNED } else { FANGED };
                place_solid(&mut world, x, z, Obj::Skull { var }, start);
            }
        }
    }

    // Now and then, a patch of drifted sand over a secret room.
    let dist = walk(&world, start);
    let mut crack = None;
    if r.chance(CRACK_CHANCE) {
        for _ in 0..400 {
            let (x, z) = (r.range(3, w - 3), r.range(3, h - 3));
            let clear = (-1..=1).all(|dz| {
                (-1..=1).all(|dx| {
                    free(&world, x + dx, z + dz) && world.floor(x + dx, z + dz) == Floor::Sand
                })
            });
            if clear && dist[world.idx(x, z)] != u32::MAX {
                world.set_obj(x, z, Some(Obj::Crack));
                crack = Some((x, z));
                break;
            }
        }
    }

    // Who lives here: the desert's folk.
    let foes = canyon_folk(depth);
    let weights: Vec<f32> = foes.iter().map(|f| f.1).collect();
    let cap = (5 + depth as usize / 3).min(22);
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

    /// Some canyons to look at, whatever the chance says.
    fn canyons() -> Vec<Level> {
        [(1u64, 6u32), (7, 12), (42, 25), (9, 47), (123, 68), (5, 99)]
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
    fn in_by_the_gateway_and_down_in_the_tombs() {
        for l in canyons() {
            let w = &l.world;
            assert!(w.canyon);
            let dist = walk(w, l.start);
            let d = dist[w.idx(l.stairs.0, l.stairs.1)];
            assert_ne!(d, u32::MAX, "the stairs are out of reach");
            assert!(d > 30, "only {d} steps");
            assert_eq!(
                w.floor(l.stairs.0, l.stairs.1),
                Floor::Walkway,
                "in the tombs"
            );
            assert!(matches!(
                w.obj(l.start.0, l.start.1 - 2),
                Some(Obj::Gateway)
            ));
        }
    }

    #[test]
    fn sand_strata_quicksand_and_everything_in_its_place() {
        for l in canyons() {
            let sand = count(&l, |w, x, z| open(w, x, z) && w.floor(x, z) == Floor::Sand);
            let quick = count(&l, |w, x, z| w.floor(x, z) == Floor::Quicksand);
            let tombs = count(
                &l,
                |w, x, z| matches!(w.wall(x, z), Wall::Sandstone(v) if sandstone_look(v) == TOMB as usize),
            );
            let tall = count(&l, |w, x, z| w.wall(x, z).height() > 1.9);
            let has = |f: &dyn Fn(&Obj) -> bool| count(&l, |w, x, z| w.obj(x, z).is_some_and(f));
            assert!(sand > 400, "{sand} tiles of sand");
            assert!(quick >= 4, "{quick} tiles of quicksand");
            assert!(tombs > 30, "{tombs} tomb blocks");
            assert!(tall > 100, "{tall} tall cliffs");
            assert!(has(&|o| matches!(o, Obj::Sandfall { .. })) >= 1);
            assert_eq!(has(&|o| matches!(o, Obj::Wyvern)), 1);
            assert!(has(&|o| matches!(o, Obj::GoldPile)) >= 3);
            assert!(has(&|o| matches!(o, Obj::Sarcophagus)) >= 2);
            assert!(has(&|o| matches!(o, Obj::Cactus { .. })) >= 4);
            assert!(has(&|o| matches!(o, Obj::Skull { .. })) >= 4);
            assert!(has(&|o| matches!(o, Obj::Sconce)) >= 3);
            assert!(has(&|o| matches!(o, Obj::LootChest { .. })) >= 2);
            assert!(has(&|o| matches!(o, Obj::Bones)) >= 10);
        }
    }

    #[test]
    fn nothing_shuts_the_way_anywhere() {
        for seed in 0..30u64 {
            for depth in [6u32, 17, 33, 52, 76] {
                let seed = seed * 7919 + 11;
                let l = generate(seed, depth, biome_for(seed, depth));
                let w = &l.world;
                let dist = walk(w, l.start);
                assert_ne!(dist[w.idx(l.stairs.0, l.stairs.1)], u32::MAX);
                // Everywhere you could walk if nothing stood in the way, you can.
                let mut ground = vec![false; (w.w * w.h) as usize];
                let mut q = std::collections::VecDeque::from([l.start]);
                ground[w.idx(l.start.0, l.start.1)] = true;
                while let Some((x, z)) = q.pop_front() {
                    for (dx, dz) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                        let (nx, nz) = (x + dx, z + dz);
                        if open(w, nx, nz) && !ground[w.idx(nx, nz)] {
                            ground[w.idx(nx, nz)] = true;
                            q.push_back((nx, nz));
                        }
                    }
                }
                for z in 0..w.h {
                    for x in 0..w.w {
                        let i = w.idx(x, z);
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
    fn cliffs_never_hide_the_canyon_floor() {
        for l in canyons() {
            let w = &l.world;
            for z in 0..w.h {
                for x in 0..w.w {
                    if w.wall(x, z) == Wall::None {
                        continue;
                    }
                    // Looking down from the south, a wall hides this far behind it: past
                    // the tile right behind it, never any open ground.
                    let hides = w.wall(x, z).height() / 47f32.to_radians().tan();
                    for k in 2..=4 {
                        if hides > (k - 1) as f32 + 0.05 {
                            assert_ne!(
                                w.wall(x, z - k),
                                Wall::None,
                                "({x}, {z}) hides ({x}, {})",
                                z - k
                            );
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn deterministic() {
        let a = generate(77, 23, 5);
        let b = generate(77, 23, 5);
        assert_eq!(a.start, b.start);
        assert_eq!(a.stairs, b.stairs);
        assert_eq!(a.world.floor, b.world.floor);
        assert_eq!(a.world.wall, b.world.wall);
    }

    #[test]
    fn often_in_the_sunken_ruins_never_too_soon() {
        let count = |biome: usize| {
            (0..600u64)
                .filter(|&seed| is_canyon(seed, 23, biome))
                .count()
        };
        let (ruins, frost) = (count(DESERT), count(4));
        assert!(ruins > frost * 2, "{ruins} vs {frost}");
        assert!(frost > 10, "{frost}");
        assert!(
            !(0..400u64).any(|seed| is_canyon(seed, 20, 5)),
            "never a guardian's"
        );
        assert!(
            !(0..400u64).any(|seed| is_canyon(seed, 5, 5)),
            "not so soon"
        );
        for seed in 0..300u64 {
            for depth in 6..50 {
                let biome = biome_for(seed, depth);
                if is_canyon(seed, depth, biome) {
                    assert!(!super::super::labyrinth::is_labyrinth(seed, depth, biome));
                    assert!(!super::super::glowcave::is_glowcave(seed, depth, biome));
                    assert!(!super::super::sewer::is_sewer(seed, depth));
                }
            }
        }
    }
}
