//! The Hollow: an endless dungeon. Every floor is rebuilt from `(world seed, depth)`, the
//! biome changes every ten floors, and every tenth floor holds a guardian and a waystone
//! that leads back to the surface.

use super::items::Item;
use super::world::{Area, Floor, Obj, Wall, World};
use crate::assets::BIOMES;
use crate::util::Rng;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Foe {
    Slime,
    Bat,
    Shroom,
    Crab,
    Wisp,
    Beetle,
    Imp,
    Skeleton,
    Golem,
    Ghost,
}

pub struct Spawn {
    pub foe: Foe,
    pub x: f32,
    pub z: f32,
    pub boss: bool,
}

pub struct Level {
    pub world: World,
    pub start: (i32, i32),
    pub stairs: (i32, i32),
    pub waystone: Option<(i32, i32)>,
    pub spawns: Vec<Spawn>,
}

pub fn biome_for(depth: u32) -> usize {
    (((depth.max(1) - 1) / 10) as usize) % BIOMES
}

pub fn is_waystone_floor(depth: u32) -> bool {
    depth > 0 && depth % 10 == 0
}

/// Enemies that live in a biome, with weights.
pub fn biome_foes(biome: usize) -> &'static [(Foe, f32)] {
    match biome {
        0 => &[(Foe::Slime, 4.0), (Foe::Bat, 2.0), (Foe::Shroom, 2.5)],
        1 => &[
            (Foe::Slime, 3.0),
            (Foe::Crab, 2.5),
            (Foe::Wisp, 1.5),
            (Foe::Bat, 1.0),
        ],
        2 => &[
            (Foe::Shroom, 3.0),
            (Foe::Slime, 2.0),
            (Foe::Beetle, 2.5),
            (Foe::Bat, 1.0),
        ],
        3 => &[
            (Foe::Slime, 2.5),
            (Foe::Imp, 2.5),
            (Foe::Bat, 1.5),
            (Foe::Beetle, 1.5),
        ],
        4 => &[
            (Foe::Slime, 2.5),
            (Foe::Crab, 2.0),
            (Foe::Wisp, 2.0),
            (Foe::Golem, 1.0),
        ],
        _ => &[
            (Foe::Skeleton, 3.0),
            (Foe::Ghost, 2.0),
            (Foe::Golem, 1.5),
            (Foe::Wisp, 1.0),
        ],
    }
}

/// The guardian of a biome's tenth floor.
pub fn boss_for(biome: usize) -> Foe {
    [
        Foe::Slime,
        Foe::Crab,
        Foe::Shroom,
        Foe::Imp,
        Foe::Golem,
        Foe::Skeleton,
    ][biome % BIOMES]
}

/// Seeds that turn up in a biome, weighted.
pub fn biome_seeds(biome: usize) -> &'static [(Item, f32)] {
    match biome {
        0 => &[
            (Item::CarrotSeeds, 5.0),
            (Item::MossberrySeeds, 3.0),
            (Item::BunnyrootSeeds, 3.0),
            (Item::GlowcapSpores, 2.5),
            (Item::TurnipSeeds, 1.5),
            (Item::SeedPotato, 1.5),
            (Item::MelonSeeds, 0.8),
        ],
        1 => &[
            (Item::BerrySeeds, 5.0),
            (Item::PrismPearSeeds, 3.0),
            (Item::GeodeGourdSeeds, 1.5),
            (Item::MelonSeeds, 1.5),
            (Item::CarrotSeeds, 1.0),
            (Item::LilyBulb, 0.6),
        ],
        2 => &[
            (Item::PumpkinSeeds, 4.0),
            (Item::PuffballSpores, 4.0),
            (Item::GlowcapSpores, 3.0),
            (Item::JellySpores, 2.5),
            (Item::MelonSeeds, 1.0),
            (Item::TruffleSpores, 0.6),
        ],
        3 => &[
            (Item::PepperSeeds, 5.0),
            (Item::FlameTulipBulb, 3.0),
            (Item::LavaLemonSeeds, 3.0),
            (Item::MagmaMelonSeeds, 1.0),
            (Item::PumpkinSeeds, 1.0),
        ],
        4 => &[
            (Item::LilyBulb, 4.0),
            (Item::SnowPeaSeeds, 4.0),
            (Item::IcePlumSeeds, 3.0),
            (Item::FrostMintSeeds, 3.0),
            (Item::BerrySeeds, 1.5),
            (Item::MoonbloomSeeds, 0.4),
        ],
        _ => &[
            (Item::GhostPepperSeeds, 3.0),
            (Item::AncientGrainSeeds, 3.0),
            (Item::MoonbloomSeeds, 1.0),
            (Item::LilyBulb, 1.0),
            (Item::StarfruitSeeds, 0.6),
            (Item::TruffleSpores, 0.4),
        ],
    }
}

/// Ore indices available at a depth with weights (see `assets::ORE_COLORS`).
fn ore_weights(depth: u32, biome: usize) -> [f32; 6] {
    let d = depth as f32;
    let mut w = [
        (1.0 - (d - 15.0).max(0.0) / 30.0).max(0.15) * 4.0,
        if d >= 5.0 { 3.0 } else { 0.5 },
        if d >= 14.0 { 2.0 } else { 0.0 },
        if biome == 1 {
            4.0
        } else if d >= 25.0 {
            0.8
        } else {
            0.0
        },
        if biome == 3 {
            4.0
        } else if d >= 38.0 {
            0.8
        } else {
            0.0
        },
        if biome == 4 {
            3.0
        } else if d >= 48.0 {
            0.6
        } else {
            0.0
        },
    ];
    if biome == 5 {
        w[2] += 2.0;
    }
    w
}

pub fn ore_item(ore: u8) -> Item {
    [
        Item::CopperOre,
        Item::IronOre,
        Item::GoldOre,
        Item::Crystal,
        Item::EmberOre,
        Item::FrostGem,
    ][ore as usize % 6]
}

#[derive(Clone, Copy)]
struct Room {
    x: i32,
    z: i32,
    w: i32,
    h: i32,
}

impl Room {
    fn center(&self) -> (i32, i32) {
        (self.x + self.w / 2, self.z + self.h / 2)
    }
    fn overlaps(&self, o: &Room, pad: i32) -> bool {
        self.x - pad < o.x + o.w
            && o.x - pad < self.x + self.w
            && self.z - pad < o.z + o.h
            && o.z - pad < self.z + self.h
    }
}

pub fn generate(seed: u64, depth: u32, via_waystone: bool) -> Level {
    let biome = biome_for(depth);
    let mut r = Rng::new(seed ^ (depth as u64).wrapping_mul(0x9E37_79B9));
    let grow = depth.min(30) as i32;
    let (w, h) = (44 + grow, 36 + grow * 2 / 3);
    let mut world = World::new(w, h, Area::Hollow { depth }, biome);
    for z in 0..h {
        for x in 0..w {
            world.set_floor(x, z, Floor::Cave);
            let edge = x < 2 || z < 2 || x >= w - 2 || z >= h - 2;
            world.set_wall(x, z, if edge { Wall::Bedrock } else { Wall::Rock });
        }
    }

    // Rooms.
    let mut rooms: Vec<Room> = Vec::new();
    let want = (9 + depth as usize / 3).min(20);
    for _ in 0..want * 40 {
        if rooms.len() >= want {
            break;
        }
        let rw = r.range(5, 12);
        let rh = r.range(4, 9);
        let room = Room {
            x: r.range(3, w - rw - 3),
            z: r.range(3, h - rh - 3),
            w: rw,
            h: rh,
        };
        if rooms.iter().any(|o| o.overlaps(&room, 2)) {
            continue;
        }
        rooms.push(room);
    }
    let organic = biome != 5;
    for room in &rooms {
        carve_room(&mut world, room, organic, &mut r);
    }

    // Connect rooms: a spanning tree plus a few loops, with two-wide corridors.
    let n = rooms.len();
    let mut in_tree = vec![false; n];
    in_tree[0] = true;
    let mut edges = Vec::new();
    for _ in 1..n {
        let mut best = (f32::MAX, 0, 0);
        for i in 0..n {
            if !in_tree[i] {
                continue;
            }
            for j in 0..n {
                if in_tree[j] {
                    continue;
                }
                let (ax, az) = rooms[i].center();
                let (bx, bz) = rooms[j].center();
                let d = ((ax - bx).pow(2) + (az - bz).pow(2)) as f32;
                if d < best.0 {
                    best = (d, i, j);
                }
            }
        }
        in_tree[best.2] = true;
        edges.push((best.1, best.2));
    }
    for _ in 0..(n / 4).max(1) {
        let a = r.below(n);
        let b = r.below(n);
        if a != b {
            edges.push((a, b));
        }
    }
    for (a, b) in edges {
        let (ax, az) = rooms[a].center();
        let (bx, bz) = rooms[b].center();
        carve_corridor(&mut world, ax, az, bx, bz, r.chance(0.5));
    }

    // Soften corners in the caves.
    if organic {
        let snapshot: Vec<Wall> = world.wall.clone();
        for z in 3..h - 3 {
            for x in 3..w - 3 {
                if snapshot[(z * w + x) as usize] != Wall::Rock {
                    continue;
                }
                let mut open = 0;
                for dz in -1..=1 {
                    for dx in -1..=1 {
                        if snapshot[((z + dz) * w + x + dx) as usize] == Wall::None {
                            open += 1;
                        }
                    }
                }
                if open >= 5 {
                    world.set_wall(x, z, Wall::None);
                }
            }
        }
    }

    // Start and stairs: the stairs go in the room farthest (by walking) from the start.
    let start_room = r.below(n);
    let start = rooms[start_room].center();
    let dist = flood(&world, start);
    let mut far = (0, start_room);
    for (i, room) in rooms.iter().enumerate() {
        let (cx, cz) = room.center();
        let d = dist[(cz * w + cx) as usize];
        if d != u32::MAX && d > far.0 {
            far = (d, i);
        }
    }
    let stairs_room = rooms[far.1];
    let stairs = stairs_room.center();
    world.set_obj(stairs.0, stairs.1, Some(Obj::StairsDown));
    let waystone = if is_waystone_floor(depth) {
        let p = (stairs.0 - 2, stairs.1);
        world.set_wall(p.0, p.1, Wall::None);
        world.set_obj(p.0, p.1, Some(Obj::Waystone));
        let c = (stairs.0 + 2, stairs.1 + 1);
        world.set_wall(c.0, c.1, Wall::None);
        world.set_obj(c.0, c.1, Some(Obj::Campfire));
        // A little enchanting table so travellers can bind their scrolls.
        let t = (stairs.0 - 3, stairs.1);
        world.set_wall(t.0, t.1, Wall::None);
        world.set_obj(t.0, t.1, Some(Obj::EnchantTable));
        Some(p)
    } else {
        None
    };

    // Ore veins in walls that face open floor.
    let weights = ore_weights(depth, biome);
    let ore_chance = 0.05 + (depth as f32 * 0.0012).min(0.05);
    for z in 2..h - 2 {
        for x in 2..w - 2 {
            if world.wall(x, z) != Wall::Rock || !touches_open(&world, x, z) {
                continue;
            }
            if r.chance(ore_chance) {
                let ore = r.weighted(&weights) as u8;
                world.set_wall(x, z, Wall::Ore(ore));
                for _ in 0..r.range(0, 3) {
                    let (nx, nz) = (x + r.range(-1, 2), z + r.range(-1, 2));
                    if world.wall(nx, nz) == Wall::Rock {
                        world.set_wall(nx, nz, Wall::Ore(ore));
                    }
                }
            }
        }
    }

    // Decorations, breakables and light.
    let keep_clear = |x: i32, z: i32| {
        (x - start.0).abs() <= 1 && (z - start.1).abs() <= 1
            || (x - stairs.0).abs() <= 1 && (z - stairs.1).abs() <= 1
    };
    let mut torches = 0;
    for z in 2..h - 2 {
        for x in 2..w - 2 {
            if world.wall(x, z) != Wall::None || world.obj(x, z).is_some() || keep_clear(x, z) {
                continue;
            }
            let near_wall = touches_wall(&world, x, z);
            let roll = r.f32();
            let deco = match biome {
                0 => pick(
                    roll,
                    &[
                        (0.012, Obj::Mushroom { var: 0 }),
                        (0.01, Obj::Bones),
                        (0.02, Obj::Stalagmite { var: 0 }),
                    ],
                ),
                1 => pick(
                    roll,
                    &[
                        (
                            0.03,
                            Obj::Crystal {
                                var: r.below(2) as u8,
                                hp: 6,
                            },
                        ),
                        (0.015, Obj::Stalagmite { var: 1 }),
                    ],
                ),
                2 => pick(
                    roll,
                    &[
                        (0.04, Obj::Mushroom { var: 1 }),
                        (0.012, Obj::Mushroom { var: 2 }),
                    ],
                ),
                3 => pick(
                    roll,
                    &[
                        (0.015, Obj::Stalagmite { var: 0 }),
                        (0.012, Obj::Bones),
                        (0.008, Obj::Crystal { var: 3, hp: 6 }),
                    ],
                ),
                4 => pick(
                    roll,
                    &[
                        (0.025, Obj::Crystal { var: 4, hp: 6 }),
                        (0.02, Obj::Stalagmite { var: 1 }),
                    ],
                ),
                _ => pick(
                    roll,
                    &[
                        (0.015, Obj::Bones),
                        (0.02, Obj::Pot { hp: 1 }),
                        (0.01, Obj::Crate { hp: 1 }),
                    ],
                ),
            };
            if let Some(o) = deco {
                if !(o.solid() && near_corridor(&world, x, z)) {
                    world.set_obj(x, z, Some(o));
                    continue;
                }
            }
            if near_wall && r.chance(0.035) {
                world.set_obj(
                    x,
                    z,
                    Some(if r.chance(0.6) {
                        Obj::Pot { hp: 1 }
                    } else {
                        Obj::Crate { hp: 1 }
                    }),
                );
                continue;
            }
            if near_wall && r.chance(0.022) && torches < 14 + depth as i32 / 3 {
                world.set_obj(x, z, Some(Obj::Torch));
                torches += 1;
            }
        }
    }
    // A torch by the arrival point so you never land in the dark.
    let (tx, tz) = (start.0 + 1, start.1 - 1);
    if world.wall(tx, tz) == Wall::None && world.obj(tx, tz).is_none() {
        world.set_obj(tx, tz, Some(Obj::Torch));
    }
    // Lava pools in the Ember Depths.
    if biome == 3 {
        for room in rooms.iter().skip(1) {
            if r.chance(0.5) {
                let (cx, cz) = room.center();
                let (lx, lz) = (cx + r.range(-2, 3), cz + r.range(-1, 2));
                for dz in -1..=1 {
                    for dx in -2..=2 {
                        let (x, z) = (lx + dx, lz + dz);
                        if world.wall(x, z) == Wall::None
                            && world.obj(x, z).is_none()
                            && !keep_clear(x, z)
                            && (dx.abs() + dz.abs() < 3)
                        {
                            world.set_floor(x, z, Floor::Lava);
                        }
                    }
                }
            }
        }
    }
    // Treasure chests in quiet corners.
    let chests = 1 + usize::from(r.chance(0.45)) + usize::from(is_waystone_floor(depth));
    let mut placed = 0;
    for _ in 0..400 {
        if placed >= chests {
            break;
        }
        let room = rooms[r.below(n)];
        let x = room.x + r.range(0, room.w);
        let z = room.z + r.range(0, room.h);
        if world.wall(x, z) == Wall::None
            && world.obj(x, z).is_none()
            && world.floor(x, z) == Floor::Cave
            && touches_wall(&world, x, z)
            && !keep_clear(x, z)
        {
            world.set_obj(x, z, Some(Obj::LootChest { opened: false }));
            placed += 1;
        }
    }

    // Enemies.
    let mut spawns = Vec::new();
    let foes = biome_foes(biome);
    let weights: Vec<f32> = foes.iter().map(|f| f.1).collect();
    let cap = (6 + depth as usize / 2).min(28);
    let mut order: Vec<usize> = (0..n).filter(|&i| i != start_room).collect();
    r.shuffle(&mut order);
    'rooms: for i in order {
        let room = rooms[i];
        let count = 1 + r.below(2 + depth as usize / 12);
        for _ in 0..count {
            if spawns.len() >= cap {
                break 'rooms;
            }
            for _ in 0..10 {
                let x = room.x + r.range(0, room.w);
                let z = room.z + r.range(0, room.h);
                if !world.blocked(x, z) && !keep_clear(x, z) {
                    spawns.push(Spawn {
                        foe: foes[r.weighted(&weights)].0,
                        x: x as f32 + 0.5,
                        z: z as f32 + 0.5,
                        boss: false,
                    });
                    break;
                }
            }
        }
    }
    if is_waystone_floor(depth) && !via_waystone {
        spawns.push(Spawn {
            foe: boss_for(biome),
            x: stairs.0 as f32 + 0.5,
            z: stairs.1 as f32 + 2.5,
            boss: true,
        });
    }

    let start = if via_waystone {
        waystone.map(|(x, z)| (x, z + 1)).unwrap_or(start)
    } else {
        start
    };
    let start = world.nearest_open(start.0, start.1);
    Level {
        world,
        start,
        stairs,
        waystone,
        spawns,
    }
}

fn pick(roll: f32, table: &[(f32, Obj)]) -> Option<Obj> {
    let mut acc = 0.0;
    for (p, o) in table {
        acc += p;
        if roll < acc {
            return Some(o.clone());
        }
    }
    None
}

fn carve_room(w: &mut World, room: &Room, organic: bool, r: &mut Rng) {
    let (cx, cz) = (
        room.x as f32 + room.w as f32 * 0.5,
        room.z as f32 + room.h as f32 * 0.5,
    );
    for z in room.z..room.z + room.h {
        for x in room.x..room.x + room.w {
            if organic {
                let dx = (x as f32 + 0.5 - cx) / (room.w as f32 * 0.5 + 0.8);
                let dz = (z as f32 + 0.5 - cz) / (room.h as f32 * 0.5 + 0.8);
                if dx * dx + dz * dz > 1.0 + r.f32() * 0.25 {
                    continue;
                }
            }
            w.set_wall(x, z, Wall::None);
        }
    }
}

fn carve_corridor(w: &mut World, ax: i32, az: i32, bx: i32, bz: i32, horizontal_first: bool) {
    let mut open = |x: i32, z: i32| {
        for (dx, dz) in [(0, 0), (1, 0), (0, 1), (1, 1)] {
            let (tx, tz) = (x + dx, z + dz);
            if tx > 1 && tz > 1 && tx < w.w - 2 && tz < w.h - 2 {
                w.set_wall(tx, tz, Wall::None);
            }
        }
    };
    let (mut x, mut z) = (ax, az);
    let step = |a: i32, b: i32| (b - a).signum();
    if horizontal_first {
        while x != bx {
            open(x, z);
            x += step(x, bx);
        }
        while z != bz {
            open(x, z);
            z += step(z, bz);
        }
    } else {
        while z != bz {
            open(x, z);
            z += step(z, bz);
        }
        while x != bx {
            open(x, z);
            x += step(x, bx);
        }
    }
    open(x, z);
}

fn touches_open(w: &World, x: i32, z: i32) -> bool {
    [(1, 0), (-1, 0), (0, 1), (0, -1)]
        .iter()
        .any(|(dx, dz)| w.wall(x + dx, z + dz) == Wall::None)
}

fn touches_wall(w: &World, x: i32, z: i32) -> bool {
    [(1, 0), (-1, 0), (0, 1), (0, -1)]
        .iter()
        .any(|(dx, dz)| w.wall(x + dx, z + dz) != Wall::None)
}

/// True in narrow passages, where a solid decoration could block the way.
fn near_corridor(w: &World, x: i32, z: i32) -> bool {
    let horiz = w.wall(x - 1, z) != Wall::None || w.wall(x + 1, z) != Wall::None;
    let vert = w.wall(x, z - 1) != Wall::None || w.wall(x, z + 1) != Wall::None;
    let open8 = (-1..=1)
        .flat_map(|dz| (-1..=1).map(move |dx| (dx, dz)))
        .filter(|(dx, dz)| w.wall(x + dx, z + dz) == Wall::None)
        .count();
    (horiz && vert) || open8 < 7
}

/// Walking distance from a tile to every tile (u32::MAX where unreachable).
pub fn flood(w: &World, from: (i32, i32)) -> Vec<u32> {
    let mut dist = vec![u32::MAX; (w.w * w.h) as usize];
    let mut q = std::collections::VecDeque::new();
    if !w.inside(from.0, from.1) {
        return dist;
    }
    dist[w.idx(from.0, from.1)] = 0;
    q.push_back(from);
    while let Some((x, z)) = q.pop_front() {
        let d = dist[w.idx(x, z)];
        for (dx, dz) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
            let (nx, nz) = (x + dx, z + dz);
            if w.inside(nx, nz) && w.wall(nx, nz) == Wall::None {
                let i = w.idx(nx, nz);
                if dist[i] == u32::MAX {
                    dist[i] = d + 1;
                    q.push_back((nx, nz));
                }
            }
        }
    }
    dist
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn floors_are_deterministic_and_connected() {
        for depth in [1, 7, 10, 23, 55, 101] {
            let a = generate(42, depth, false);
            let b = generate(42, depth, false);
            assert_eq!(a.start, b.start);
            assert_eq!(a.stairs, b.stairs);
            let dist = flood(&a.world, a.start);
            let i = a.world.idx(a.stairs.0, a.stairs.1);
            assert_ne!(dist[i], u32::MAX, "stairs unreachable on floor {depth}");
            assert!(!a.world.blocked(a.start.0, a.start.1));
            assert_eq!(a.waystone.is_some(), depth % 10 == 0);
        }
    }

    #[test]
    fn biomes_cycle() {
        assert_eq!(biome_for(1), 0);
        assert_eq!(biome_for(10), 0);
        assert_eq!(biome_for(11), 1);
        assert_eq!(biome_for(61), 0);
    }
}
