//! Sewer floors. Now and then a floor of the Hollow is a stretch of old sewer instead of
//! caves: murky channels running between stone walkways, one either side of the water,
//! meeting at junctions and ending in rounded chambers, with bridges across of weathered
//! planks or of copper gone green with age.
//!
//! The channels follow a grid. Every node on it is a junction (or, if only one channel
//! reaches it, a dead end the walkways wrap round), and the channels between them form a
//! random spanning tree plus a few loops, so the whole sewer hangs together. Every stretch
//! of channel has a bridge, so both walkways are always in reach.

use super::dungeon::{
    CRACK_CHANCE, DEEP_FOLK, Foe, GLEAM_CHANCE, Level, Spawn, is_waystone_floor, ore_weights,
    too_tough,
};
use super::world::{Area, Floor, Obj, Wall, World};
use crate::util::Rng;

/// How often a floor is sewers (never the first few, nor a guardian's floor).
pub const SEWER_CHANCE: f32 = 0.22;
pub const FIRST_SEWER: u32 = 3;

/// Tiles from the centre line of a channel to the walls either side: two of water in the
/// middle, and two lanes of walkway along each side.
const HALF: i32 = 3;
/// Roughly how far apart the junctions are.
const SPACING: i32 = 13;

/// Who lives in the old sewers, whatever the biome above: sludge slimes, and the Hollow's
/// folk down to their bones (see `Enemy::in_the_sewers`), with bone snails and bone
/// bibliomancers deeper down.
pub fn sewer_foes(depth: u32) -> Vec<(Foe, f32)> {
    let mut v = vec![
        (Foe::Slime, 3.0),
        (Foe::Bat, 1.4),
        (Foe::Zombie, 1.4),
        (Foe::Sneak, 1.1),
        (Foe::Brute, 0.8),
        (Foe::Bug, 1.5),
        (Foe::Skeleton, 1.2),
        (Foe::Ghost, 0.8),
    ];
    if depth >= DEEP_FOLK {
        v.extend([(Foe::Snail, 1.0), (Foe::Bookworm, 0.9)]);
    }
    v
}

/// The sewers' water folk: bone frogs, and the odd bone puffer.
pub const SEWER_POND: [Foe; 3] = [Foe::Frog, Foe::Frog, Foe::Puffer];

/// Is this floor sewers?
pub fn is_sewer(seed: u64, depth: u32) -> bool {
    depth >= FIRST_SEWER
        && !is_waystone_floor(depth)
        && Rng::new(seed ^ (depth as u64).wrapping_mul(0xC2B2_AE3D) ^ 0x5E3E).chance(SEWER_CHANCE)
}

/// A stretch of channel between two junctions (indices into the node list).
#[derive(Clone, Copy)]
struct Edge {
    a: usize,
    b: usize,
}

fn find(parent: &mut [usize], i: usize) -> usize {
    let mut i = i;
    while parent[i] != i {
        parent[i] = parent[parent[i]];
        i = parent[i];
    }
    i
}

fn fill(w: &mut World, x0: i32, z0: i32, x1: i32, z1: i32, f: Floor) {
    for z in z0..=z1 {
        for x in x0..=x1 {
            if w.inside(x, z) && w.wall(x, z) != Wall::Bedrock {
                w.set_wall(x, z, Wall::None);
                w.set_floor(x, z, f);
            }
        }
    }
}

/// Grid lines `count` of them from `lo` to `hi`, a little uneven.
fn lines(r: &mut Rng, lo: i32, hi: i32) -> Vec<i32> {
    let n = (((hi - lo) as f32 / SPACING as f32).round() as i32 + 1).max(2);
    let step = (hi - lo) / (n - 1);
    (0..n)
        .map(|i| {
            let at = lo + i * step;
            if i == 0 || i == n - 1 {
                at
            } else {
                at + r.range(-1, 2)
            }
        })
        .collect()
}

/// Walking distances from a tile over everything walkable (bridges yes, water no).
fn walk(w: &World, from: (i32, i32)) -> Vec<u32> {
    let mut dist = vec![u32::MAX; (w.w * w.h) as usize];
    let mut q = std::collections::VecDeque::new();
    dist[w.idx(from.0, from.1)] = 0;
    q.push_back(from);
    while let Some((x, z)) = q.pop_front() {
        let d = dist[w.idx(x, z)];
        for (dx, dz) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
            let (nx, nz) = (x + dx, z + dz);
            if w.inside(nx, nz) && !w.blocked(nx, nz) {
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

fn next_to(w: &World, x: i32, z: i32, f: impl Fn(Floor, Wall) -> bool) -> bool {
    [(1, 0), (-1, 0), (0, 1), (0, -1)]
        .iter()
        .any(|(dx, dz)| f(w.floor(x + dx, z + dz), w.wall(x + dx, z + dz)))
}

/// A walkway tile along a wall, clear of the water and the bridges: where things can stand
/// without getting in anyone's way.
fn wall_lane(w: &World, x: i32, z: i32) -> bool {
    w.floor(x, z) == Floor::Walkway
        && w.wall(x, z) == Wall::None
        && w.obj(x, z).is_none()
        && next_to(w, x, z, |_, wall| wall != Wall::None)
        && !next_to(w, x, z, |f, _| {
            matches!(f, Floor::Water | Floor::Bridge | Floor::CopperBridge)
        })
}

/// Would something solid here shut a neighbour in, leaving it no other way out? (Tucked into
/// a corner, two barrels either side of it would.)
fn traps(w: &World, x: i32, z: i32) -> bool {
    const SIDES: [(i32, i32); 4] = [(1, 0), (-1, 0), (0, 1), (0, -1)];
    SIDES.iter().any(|&(dx, dz)| {
        let (nx, nz) = (x + dx, z + dz);
        !w.blocked(nx, nz)
            && SIDES.iter().all(|&(ex, ez)| {
                let (mx, mz) = (nx + ex, nz + ez);
                (mx, mz) == (x, z) || w.blocked(mx, mz)
            })
    })
}

pub fn generate(seed: u64, depth: u32, biome: usize) -> Level {
    let mut r = Rng::new(seed ^ (depth as u64).wrapping_mul(0x9E37_79B9) ^ 0x5E3E_5E3E);
    let grow = depth.min(30) as i32;
    let (w, h) = (46 + grow, 38 + grow * 2 / 3);
    let mut world = World::new(w, h, Area::Hollow { depth }, biome);
    world.sewer = true;
    for z in 0..h {
        for x in 0..w {
            world.set_floor(x, z, Floor::Cave);
            let edge = x < 2 || z < 2 || x >= w - 2 || z >= h - 2;
            world.set_wall(x, z, if edge { Wall::Bedrock } else { Wall::Sewer });
        }
    }

    // The junctions, and the channels between them: a spanning tree, and a few loops.
    let xs = lines(&mut r, 6, w - 7);
    let zs = lines(&mut r, 6, h - 7);
    let (cols, rows) = (xs.len(), zs.len());
    let node = |c: usize, rw: usize| rw * cols + c;
    let at = |i: usize| (xs[i % cols], zs[i / cols]);
    let mut all = Vec::new();
    for rw in 0..rows {
        for c in 0..cols {
            if c + 1 < cols {
                all.push(Edge {
                    a: node(c, rw),
                    b: node(c + 1, rw),
                });
            }
            if rw + 1 < rows {
                all.push(Edge {
                    a: node(c, rw),
                    b: node(c, rw + 1),
                });
            }
        }
    }
    r.shuffle(&mut all);
    let mut parent: Vec<usize> = (0..cols * rows).collect();
    let mut edges = Vec::new();
    for e in all {
        let (ra, rb) = (find(&mut parent, e.a), find(&mut parent, e.b));
        if ra != rb {
            parent[ra] = rb;
            edges.push(e);
        } else if r.chance(0.28) {
            edges.push(e);
        }
    }
    let mut degree = vec![0usize; cols * rows];
    for e in &edges {
        degree[e.a] += 1;
        degree[e.b] += 1;
    }

    // Walkways first: round every junction and along every channel...
    for (i, &d) in degree.iter().enumerate() {
        if d > 0 {
            let (cx, cz) = at(i);
            fill(
                &mut world,
                cx - HALF,
                cz - HALF,
                cx + HALF - 1,
                cz + HALF - 1,
                Floor::Walkway,
            );
        }
    }
    for e in &edges {
        let ((ax, az), (bx, bz)) = (at(e.a), at(e.b));
        fill(
            &mut world,
            ax.min(bx) - HALF,
            az.min(bz) - HALF,
            ax.max(bx) + HALF - 1,
            az.max(bz) + HALF - 1,
            Floor::Walkway,
        );
    }
    // ...then the water down the middle.
    for e in &edges {
        let ((ax, az), (bx, bz)) = (at(e.a), at(e.b));
        fill(
            &mut world,
            ax.min(bx) - 1,
            az.min(bz) - 1,
            ax.max(bx),
            az.max(bz),
            Floor::Water,
        );
    }
    // Dead ends are rounded chambers: the walkways wrap round the end of the channel, and
    // its far corners are walled in.
    for (i, &d) in degree.iter().enumerate() {
        if d != 1 {
            continue;
        }
        let (cx, cz) = at(i);
        let e = edges.iter().find(|e| e.a == i || e.b == i).unwrap();
        let other = at(if e.a == i { e.b } else { e.a });
        // The two corners on the far side from the channel.
        if other.0 != cx {
            let fx = if other.0 > cx {
                cx - HALF
            } else {
                cx + HALF - 1
            };
            for z in [cz - HALF, cz + HALF - 1] {
                world.set_wall(fx, z, Wall::Sewer);
            }
        } else {
            let fz = if other.1 > cz {
                cz - HALF
            } else {
                cz + HALF - 1
            };
            for x in [cx - HALF, cx + HALF - 1] {
                world.set_wall(x, fz, Wall::Sewer);
            }
        }
    }
    // A bridge (or two, on the long stretches) across every channel, of planks or of old
    // copper plate.
    for e in &edges {
        let ((ax, az), (bx, bz)) = (at(e.a), at(e.b));
        let (lo, hi) = if ax == bx {
            (az.min(bz), az.max(bz))
        } else {
            (ax.min(bx), ax.max(bx))
        };
        // Clear of the walkways round the junctions at either end.
        let (first, last) = (lo + HALF, hi - HALF - 2);
        if last < first {
            continue;
        }
        let mut spots = vec![r.range(first, last + 1)];
        if last - first >= 9 && r.chance(0.5) {
            let s = spots[0];
            let other = if s - first > last - s {
                r.range(first, (s - 3).max(first) + 1)
            } else {
                r.range((s + 3).min(last), last + 1)
            };
            if (other - s).abs() >= 3 {
                spots.push(other);
            }
        }
        for s in spots {
            let kind = if r.chance(0.4) {
                Floor::CopperBridge
            } else {
                Floor::Bridge
            };
            for k in 0..2 {
                for side in -1..=0 {
                    let (x, z) = if ax == bx {
                        (ax + side, s + k)
                    } else {
                        (s + k, az + side)
                    };
                    world.set_floor(x, z, kind);
                }
            }
        }
    }

    // Past its walls there's nothing but the dark: the sewer stands alone in it.
    let solid = world.wall.clone();
    for z in 0..h {
        for x in 0..w {
            if solid[(z * w + x) as usize] == Wall::None {
                continue;
            }
            let near = (-1..=1).any(|dz: i32| {
                (-1..=1).any(|dx: i32| {
                    let (nx, nz) = (x + dx, z + dz);
                    world.inside(nx, nz) && solid[(nz * w + nx) as usize] == Wall::None
                })
            });
            if !near {
                world.set_wall(x, z, Wall::None);
                world.set_floor(x, z, Floor::Void);
            }
        }
    }

    // In at one dead end (or junction), down the stairs at the far end of the walk.
    let ends: Vec<usize> = (0..degree.len()).filter(|&i| degree[i] == 1).collect();
    let used: Vec<usize> = (0..degree.len()).filter(|&i| degree[i] > 0).collect();
    let first = if ends.is_empty() {
        used[r.below(used.len())]
    } else {
        ends[r.below(ends.len())]
    };
    let (cx, cz) = at(first);
    let start = world.nearest_open(cx - HALF + 1, cz - HALF + 1);
    let dist = walk(&world, start);
    let mut far = (0, start);
    for z in 0..h {
        for x in 0..w {
            let d = dist[world.idx(x, z)];
            if d != u32::MAX
                && d > far.0
                && world.floor(x, z) == Floor::Walkway
                && !next_to(&world, x, z, |f, _| {
                    matches!(f, Floor::Bridge | Floor::CopperBridge)
                })
            {
                far = (d, (x, z));
            }
        }
    }
    let stairs = far.1;
    world.set_obj(stairs.0, stairs.1, Some(Obj::StairsDown));
    let keep_clear = |x: i32, z: i32| {
        (x - start.0).abs() <= 1 && (z - start.1).abs() <= 1
            || (x - stairs.0).abs() <= 1 && (z - stairs.1).abs() <= 1
    };

    // Ore seams in the old brickwork.
    let ores = ore_weights(depth, biome);
    for z in 2..h - 2 {
        for x in 2..w - 2 {
            if world.wall(x, z) == Wall::Sewer
                && next_to(&world, x, z, |_, wall| wall == Wall::None)
                && r.chance(0.03)
            {
                world.set_wall(x, z, Wall::Ore(r.weighted(&ores) as u8));
            }
        }
    }

    // Drains pouring into the walkways, grates in the floor, torches, and barrels, crates
    // and pots stacked along the walls.
    let mut torches = 0;
    for z in 2..h - 2 {
        for x in 2..w - 2 {
            if keep_clear(x, z) || !wall_lane(&world, x, z) {
                continue;
            }
            // Pipes come out of walls you can see: behind, or to the side.
            let wall_behind = world.wall(x, z - 1) == Wall::Sewer;
            let roll = r.f32();
            let obj = if wall_behind && roll < 0.06 {
                Obj::Drain
            } else if roll < 0.08 {
                Obj::Grate
            } else if roll < 0.11 && torches < 12 + depth as i32 / 3 {
                torches += 1;
                Obj::Torch
            } else if roll < 0.15 {
                Obj::Keg { hp: 1 }
            } else if roll < 0.18 {
                Obj::Crate { hp: 1 }
            } else if roll < 0.19 {
                Obj::Pot { hp: 1 }
            } else {
                continue;
            };
            if obj.solid() && traps(&world, x, z) {
                continue;
            }
            world.set_obj(x, z, Some(obj));
        }
    }
    // Old bones and broken planks strewn along the walkways, and more planks drifting in
    // the channels.
    for z in 2..h - 2 {
        for x in 2..w - 2 {
            if keep_clear(x, z) || world.obj(x, z).is_some() || world.wall(x, z) != Wall::None {
                continue;
            }
            let roll = r.f32();
            let obj = match world.floor(x, z) {
                Floor::Walkway if roll < 0.03 => Obj::Bones,
                Floor::Walkway if roll < 0.055 => Obj::Debris {
                    var: r.below(3) as u8,
                },
                Floor::Water if roll < 0.05 => Obj::Debris {
                    var: r.below(3) as u8,
                },
                _ => continue,
            };
            world.set_obj(x, z, Some(obj));
        }
    }
    // A torch by the arrival point so you never land in the dark.
    for (dx, dz) in [(1, 0), (0, 1), (-1, 0), (0, -1), (1, 1)] {
        let (tx, tz) = (start.0 + dx, start.1 + dz);
        if world.floor(tx, tz) == Floor::Walkway
            && world.wall(tx, tz) == Wall::None
            && world.obj(tx, tz).is_none()
        {
            world.set_obj(tx, tz, Some(Obj::Torch));
            break;
        }
    }

    // Treasure in the quiet dead ends.
    let chests = 1 + usize::from(r.chance(0.5));
    let mut placed = 0;
    let mut order: Vec<usize> = ends.iter().copied().filter(|&i| i != first).collect();
    r.shuffle(&mut order);
    for i in order.into_iter().chain(used.iter().copied()) {
        if placed >= chests {
            break;
        }
        let (cx, cz) = at(i);
        for _ in 0..20 {
            let (x, z) = (cx + r.range(-HALF, HALF), cz + r.range(-HALF, HALF));
            if !keep_clear(x, z) && wall_lane(&world, x, z) && !traps(&world, x, z) {
                world.set_obj(
                    x,
                    z,
                    Some(Obj::LootChest {
                        opened: false,
                        gleam: r.chance(GLEAM_CHANCE),
                    }),
                );
                placed += 1;
                break;
            }
        }
    }

    // Now and then, a cracked patch of walkway over a secret room.
    let mut crack = None;
    if r.chance(CRACK_CHANCE) {
        for _ in 0..300 {
            let (x, z) = (r.range(2, w - 2), r.range(2, h - 2));
            if world.floor(x, z) == Floor::Walkway
                && world.wall(x, z) == Wall::None
                && world.obj(x, z).is_none()
                && !keep_clear(x, z)
                && !next_to(&world, x, z, |f, wall| {
                    wall != Wall::None || f != Floor::Walkway
                })
            {
                world.set_obj(x, z, Some(Obj::Crack));
                crack = Some((x, z));
                break;
            }
        }
    }

    // Who lives down here: the sewers' creatures on the walkways, and water folk along the
    // channels.
    let foes = sewer_foes(depth);
    let weights: Vec<f32> = foes.iter().map(|f| f.1).collect();
    let cap = (6 + depth as usize / 2).min(28);
    let mut spawns = Vec::new();
    let near_start = |x: i32, z: i32| (x - start.0).abs() + (z - start.1).abs() < 8;
    for _ in 0..cap * 30 {
        if spawns.len() >= cap {
            break;
        }
        let (x, z) = (r.range(2, w - 2), r.range(2, h - 2));
        if world.floor(x, z) != Floor::Walkway || world.blocked(x, z) || near_start(x, z) {
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
    let folk = SEWER_POND;
    let mut wet = 0;
    for _ in 0..400 {
        if wet >= 3 + depth as usize / 15 {
            break;
        }
        let (x, z) = (r.range(2, w - 2), r.range(2, h - 2));
        if world.floor(x, z) == Floor::Walkway
            && !world.blocked(x, z)
            && !near_start(x, z)
            && next_to(&world, x, z, |f, _| f == Floor::Water)
        {
            spawns.push(Spawn {
                foe: folk[r.below(folk.len())],
                x: x as f32 + 0.5,
                z: z as f32 + 0.5,
                boss: false,
            });
            wet += 1;
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

    /// Some sewer floors to look at, whatever the chance says.
    fn sewers() -> Vec<Level> {
        [(1u64, 3u32), (7, 12), (42, 25), (9, 47), (123, 68), (5, 99)]
            .into_iter()
            .map(|(seed, depth)| generate(seed, depth, biome_for(seed, depth)))
            .collect()
    }

    #[test]
    fn every_walkway_and_bridge_is_in_reach() {
        for l in sewers() {
            let w = &l.world;
            let dist = walk(w, l.start);
            let mut walkways = 0;
            for z in 0..w.h {
                for x in 0..w.w {
                    let f = w.floor(x, z);
                    let open = w.wall(x, z) == Wall::None && !w.blocked(x, z);
                    if open && matches!(f, Floor::Walkway | Floor::Bridge | Floor::CopperBridge) {
                        walkways += 1;
                        assert_ne!(dist[w.idx(x, z)], u32::MAX, "({x}, {z}) is cut off");
                    }
                }
            }
            assert!(walkways > 200, "only {walkways} walkway tiles");
            assert!(super::super::dungeon::reachable(w, l.start, l.stairs));
            assert_ne!(l.start, l.stairs);
        }
    }

    #[test]
    fn clutter_never_shuts_anywhere_in() {
        for seed in 0..40u64 {
            for depth in [4u32, 17, 33, 52, 76] {
                let seed = seed * 7919 + 3;
                let l = generate(seed, depth, biome_for(seed, depth));
                let w = &l.world;
                let dist = walk(w, l.start);
                for z in 0..w.h {
                    for x in 0..w.w {
                        if matches!(
                            w.floor(x, z),
                            Floor::Walkway | Floor::Bridge | Floor::CopperBridge
                        ) && w.wall(x, z) == Wall::None
                            && !w.blocked(x, z)
                        {
                            assert_ne!(dist[w.idx(x, z)], u32::MAX, "seed {seed} depth {depth}");
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn channels_have_a_walkway_each_side_and_bridges_across() {
        for l in sewers() {
            let w = &l.world;
            let mut bridges = [0; 2];
            for z in 1..w.h - 1 {
                for x in 1..w.w - 1 {
                    match w.floor(x, z) {
                        // Water is never up against a wall: there's always a walkway (or
                        // more water, or a bridge).
                        Floor::Water => {
                            for (dx, dz) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                                let (nx, nz) = (x + dx, z + dz);
                                assert_eq!(w.wall(nx, nz), Wall::None, "({x}, {z})");
                                assert!(matches!(
                                    w.floor(nx, nz),
                                    Floor::Water
                                        | Floor::Walkway
                                        | Floor::Bridge
                                        | Floor::CopperBridge
                                ));
                            }
                        }
                        Floor::Bridge => bridges[0] += 1,
                        Floor::CopperBridge => bridges[1] += 1,
                        _ => {}
                    }
                    // Two lanes: walkway beside water has more walkway behind it.
                    if w.floor(x, z) == Floor::Walkway && w.wall(x, z) == Wall::None {
                        for (dx, dz) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                            if w.floor(x + dx, z + dz) == Floor::Water {
                                let back = (x - dx, z - dz);
                                assert_ne!(w.floor(back.0, back.1), Floor::Water);
                            }
                        }
                    }
                }
            }
            assert!(bridges[0] + bridges[1] >= 8, "{bridges:?}");
        }
        // Across all of them, both kinds of bridge turn up.
        let kinds = sewers().iter().fold([false; 2], |k, l| {
            let w = &l.world;
            let has = |f| (0..w.h).any(|z| (0..w.w).any(|x| w.floor(x, z) == f));
            [k[0] || has(Floor::Bridge), k[1] || has(Floor::CopperBridge)]
        });
        assert_eq!(kinds, [true, true]);
    }

    #[test]
    fn some_floors_are_sewers_but_never_the_first_or_a_guardians() {
        let seed = 77;
        let sewers: Vec<u32> = (1..200).filter(|&d| is_sewer(seed, d)).collect();
        assert!(sewers.len() > 20, "{sewers:?}");
        assert!(sewers.iter().all(|&d| d >= FIRST_SEWER && d % 10 != 0));
    }
}
