//! The Hollow: an endless dungeon. Every floor is rebuilt from `(world seed, depth)`, the
//! biome changes every ten floors (in an order of each save's own), and every tenth floor
//! holds a guardian and a waystone that leads back to the surface.

use super::items::Item;
use super::world::{Area, Floor, Obj, Wall, World};
use crate::assets::BIOMES;
use crate::util::Rng;

#[derive(Clone, Copy, PartialEq, Eq, Debug, serde::Serialize, serde::Deserialize)]
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
    /// Water folk: they live by the underground ponds.
    Frog,
    Jelly,
    Puffer,
    /// The walking dead: slow, arms out, and a lunge when they get close.
    Zombie,
    /// A fat goblin with a club: tough, and slams the ground.
    Brute,
    /// A skinny goblin: quick, stabs and darts away, and throws daggers.
    Sneak,
    /// The biome's bug: moss spiders, glass mantises, spore moths, fire ants, frost
    /// spiders and scarabs.
    Bug,
    /// A lantern snail: soft and slow under a glowing stained-glass shell that lights up the
    /// dark round it. Struck, it hides in its shell for a moment. Floor 11 and down.
    Snail,
    /// A book-worm bibliomancer: a caterpillar in spectacles that keeps its distance and
    /// spits glowing ink, which dries into runes pointing the way on. Floor 11 and down.
    Bookworm,
    /// A drakeling: a little dragon, a frost drake in the Frost Caverns and a cinder drake
    /// in the Ember Depths. It keeps its distance, rears up and breathes frost or fire.
    /// Floor 11 and down.
    Drake,
    /// A leafling: a cross little sprite of leaves flitting about the Mossy Burrows. It
    /// throws leaves, and mends whoever's most hurt round it.
    Leafling,
    /// A werewolf: out only under a full moon, from floor 5 down. It howls to bring the
    /// whole floor running, then pounces.
    Werewolf,
    /// A minotaur, roaming the marble labyrinth with its axe. It lowers its horns and
    /// charges down the corridors, and running into a wall leaves it dazed.
    Minotaur,
    /// A griffin, nesting in the labyrinth's sunlit courtyard. It wheels round overhead and
    /// dives at you talons first.
    Griffin,
    /// A cactling: a stout little cactus standing among the canyon's cacti, looking just
    /// like them until you come close. Then it pops up out of the sand, waddles after you on
    /// its roots, and every so often bristles and sprays a ring of needles all round.
    Cactus,
    /// A sand cobra, slithering over the canyon's sand. Close by it rears up with its hood
    /// spread and strikes; from further off it rears and spits venom.
    Cobra,
}

/// The first floor the lantern snails and book-worms live on.
pub const DEEP_FOLK: u32 = 11;

/// Everyone living on a floor: the biome's folk, and from floor 11 down its deep folk as
/// well (see `deep_folk`).
pub fn floor_foes(biome: usize, depth: u32) -> Vec<(Foe, f32)> {
    let mut v = biome_foes(biome).to_vec();
    if depth >= DEEP_FOLK {
        v.extend_from_slice(deep_folk(biome));
    }
    v
}

/// Who lives in a biome from floor 11 down, with weights: lantern snails and book-worm
/// bibliomancers everywhere, and drakelings in the Ember Depths and the Frost Caverns.
pub fn deep_folk(biome: usize) -> &'static [(Foe, f32)] {
    match biome {
        3 | 4 => &[(Foe::Snail, 1.3), (Foe::Bookworm, 1.1), (Foe::Drake, 1.2)],
        _ => &[(Foe::Snail, 1.3), (Foe::Bookworm, 1.1)],
    }
}

/// The first floor werewolves prowl under a full moon.
pub const WEREWOLF_FLOOR: u32 = 5;

/// Out-of-the-way spots on a floor, far from where you come in and from each other: where
/// the full moon's werewolves prowl.
pub fn prowls(level: &Level, n: usize, rng: &mut Rng) -> Vec<(f32, f32)> {
    let w = &level.world;
    let (sx, sz) = level.start;
    let mut open = Vec::new();
    for z in 1..w.h - 1 {
        for x in 1..w.w - 1 {
            let far = (x - sx).pow(2) + (z - sz).pow(2) > 14 * 14;
            if far && !w.blocked(x, z) && w.obj(x, z).is_none() {
                open.push((x, z));
            }
        }
    }
    let mut out: Vec<(f32, f32)> = Vec::new();
    while out.len() < n && !open.is_empty() {
        let (x, z) = open.swap_remove(rng.below(open.len()));
        let (fx, fz) = (x as f32 + 0.5, z as f32 + 0.5);
        if out
            .iter()
            .all(|&(ox, oz)| (ox - fx).powi(2) + (oz - fz).powi(2) > 36.0)
        {
            out.push((fx, fz));
        }
    }
    out
}

/// How many werewolves prowl a floor under a full moon: more the deeper you go.
pub fn werewolves(depth: u32) -> usize {
    if depth < WEREWOLF_FLOOR || is_waystone_floor(depth) {
        0
    } else {
        (1 + depth as usize / 12).min(4)
    }
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
    /// Where a tenth floor's guardian waits: the most open ground of its arena.
    pub lair: (f32, f32),
    /// A cracked patch of floor with a secret room beneath, on some floors.
    pub crack: Option<(i32, i32)>,
}

/// How often an ordinary floor hides a secret room under a cracked tile.
pub const CRACK_CHANCE: f32 = 0.5;

/// Which biome a floor is in, as this save's Hollow goes. Every ten floors are one biome, in
/// an order of each save's own: every run of six bands goes through all six biomes,
/// shuffled, and no biome comes twice in a row. (A quest can hold a band to a biome it needs
/// for a while: see `Play::biome_at`.)
pub fn biome_for(seed: u64, depth: u32) -> usize {
    let band = (depth.max(1) - 1) / 10;
    biome_order(seed, band / BIOMES as u32)[(band % BIOMES as u32) as usize]
}

/// The biomes of one run of six bands of floors, in order.
fn biome_order(seed: u64, run: u32) -> [usize; BIOMES] {
    let mut r = Rng::new(seed ^ 0xB10E_5EED ^ (run as u64 + 1).wrapping_mul(0x9E37_79B9_7F4A_7C15));
    let mut order: [usize; BIOMES] = std::array::from_fn(|i| i);
    r.shuffle(&mut order);
    // Never the same biome twice in a row, across the seam with the run before.
    if run > 0 && order[0] == biome_order(seed, run - 1)[BIOMES - 1] {
        let k = 1 + r.below(BIOMES - 1);
        order.swap(0, k);
    }
    order
}

/// Does this creature live in this biome (on its floors, deeper down, or round its ponds)?
pub fn lives_in(foe: Foe, biome: usize) -> bool {
    biome_foes(biome)
        .iter()
        .chain(deep_folk(biome))
        .any(|&(f, _)| f == foe)
        || pond_foes(biome).contains(&foe)
}

pub fn is_waystone_floor(depth: u32) -> bool {
    depth > 0 && depth % 10 == 0
}

/// Enemies that live in a biome, with weights. Zombies, goblins, bugs, skeletons and ghosts
/// turn up everywhere, dressed for wherever they live.
pub fn biome_foes(biome: usize) -> &'static [(Foe, f32)] {
    match biome {
        0 => &[
            (Foe::Slime, 3.0),
            (Foe::Bat, 1.5),
            (Foe::Shroom, 2.0),
            (Foe::Leafling, 1.3),
            (Foe::Frog, 0.6),
            (Foe::Zombie, 1.2),
            (Foe::Sneak, 1.2),
            (Foe::Brute, 0.7),
            (Foe::Bug, 1.6),
            (Foe::Skeleton, 0.7),
            (Foe::Ghost, 0.6),
        ],
        1 => &[
            (Foe::Slime, 2.2),
            (Foe::Crab, 2.0),
            (Foe::Wisp, 1.2),
            (Foe::Bat, 0.8),
            (Foe::Puffer, 0.6),
            (Foe::Zombie, 1.2),
            (Foe::Sneak, 1.2),
            (Foe::Brute, 1.0),
            (Foe::Bug, 1.6),
            (Foe::Skeleton, 0.9),
            (Foe::Ghost, 0.8),
        ],
        2 => &[
            (Foe::Shroom, 2.4),
            (Foe::Slime, 1.5),
            (Foe::Beetle, 1.8),
            (Foe::Bat, 0.8),
            (Foe::Jelly, 0.6),
            (Foe::Zombie, 1.2),
            (Foe::Sneak, 1.2),
            (Foe::Brute, 1.0),
            (Foe::Bug, 1.8),
            (Foe::Skeleton, 0.8),
            (Foe::Ghost, 1.0),
        ],
        3 => &[
            (Foe::Slime, 1.8),
            (Foe::Imp, 2.0),
            (Foe::Bat, 1.0),
            (Foe::Beetle, 1.0),
            (Foe::Zombie, 1.2),
            (Foe::Sneak, 1.2),
            (Foe::Brute, 1.2),
            (Foe::Bug, 1.8),
            (Foe::Skeleton, 1.2),
            (Foe::Ghost, 0.8),
        ],
        4 => &[
            (Foe::Slime, 1.8),
            (Foe::Crab, 1.5),
            (Foe::Wisp, 1.5),
            (Foe::Golem, 0.8),
            (Foe::Jelly, 0.6),
            (Foe::Zombie, 1.2),
            (Foe::Sneak, 1.0),
            (Foe::Brute, 1.2),
            (Foe::Bug, 1.6),
            (Foe::Skeleton, 1.0),
            (Foe::Ghost, 1.2),
        ],
        _ => &[
            (Foe::Skeleton, 2.4),
            (Foe::Ghost, 1.8),
            (Foe::Golem, 1.2),
            (Foe::Wisp, 0.8),
            (Foe::Puffer, 0.6),
            (Foe::Zombie, 1.6),
            (Foe::Sneak, 1.0),
            (Foe::Brute, 1.0),
            (Foe::Bug, 1.6),
        ],
    }
}

/// The toughest creatures stay out of the first few floors.
pub fn too_tough(foe: Foe, depth: u32) -> bool {
    depth < 3 && matches!(foe, Foe::Brute | Foe::Skeleton)
}

/// Who lives around the underground ponds of a biome.
pub fn pond_foes(biome: usize) -> &'static [Foe] {
    match biome {
        0 => &[Foe::Frog, Foe::Frog, Foe::Crab],
        1 => &[Foe::Crab, Foe::Puffer, Foe::Jelly],
        2 => &[Foe::Frog, Foe::Jelly],
        4 => &[Foe::Jelly, Foe::Puffer, Foe::Crab],
        _ => &[Foe::Puffer, Foe::Jelly],
    }
}

/// The guardian of a tenth floor (in `biome`): the classic six on the first trip down
/// through the biomes, then giants of the Hollow's commoner folk the next time round, and so
/// on.
pub fn boss_for(depth: u32, biome: usize) -> Foe {
    let cycle = (depth.max(1) - 1) / (10 * BIOMES as u32);
    if cycle % 2 == 0 {
        [
            Foe::Slime,
            Foe::Crab,
            Foe::Shroom,
            Foe::Imp,
            Foe::Golem,
            Foe::Skeleton,
        ][biome]
    } else {
        [
            Foe::Brute,
            Foe::Bug,
            Foe::Zombie,
            Foe::Sneak,
            Foe::Ghost,
            Foe::Zombie,
        ][biome]
    }
}

/// Where a guardian keeps: the first tenth floor it guards, and the biome it's found in
/// there. `None` for creatures that never guard a floor.
pub fn guardian_home(foe: Foe) -> Option<(u32, usize)> {
    (0..2 * BIOMES as u32).find_map(|band| {
        let (floor, biome) = (band * 10 + 10, band as usize % BIOMES);
        (boss_for(floor, biome) == foe).then_some((floor, biome))
    })
}

/// How often a treasure chest gleams: a rare one, full of well-rolled things.
pub const GLEAM_CHANCE: f32 = 0.07;

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
pub(crate) fn ore_weights(depth: u32, biome: usize) -> [f32; 6] {
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

/// Builds a floor of the Hollow in `biome` (see `Play::biome_at`).
pub fn generate(seed: u64, depth: u32, biome: usize, via_waystone: bool) -> Level {
    if super::sewer::is_sewer(seed, depth) {
        return super::sewer::generate(seed, depth, biome);
    }
    if super::glowcave::is_glowcave(seed, depth, biome) {
        return super::glowcave::generate(seed, depth, biome);
    }
    if super::labyrinth::is_labyrinth(seed, depth, biome) {
        return super::labyrinth::generate(seed, depth, biome);
    }
    if super::canyon::is_canyon(seed, depth, biome) {
        return super::canyon::generate(seed, depth, biome);
    }
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
    if is_waystone_floor(depth) {
        // The guardian needs room to move: an arena around the stairs, opening out to the
        // south where it waits.
        let (aw, ah) = (13, 10);
        let arena = Room {
            x: (stairs.0 - aw / 2).clamp(3, w - aw - 3),
            z: (stairs.1 - 3).clamp(3, h - ah - 3),
            w: aw,
            h: ah,
        };
        carve_room(&mut world, &arena, organic, &mut r);
    }
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
    // Underground ponds, with the water folk living round them. A pond never cuts the way to
    // the stairs.
    let mut ponds = Vec::new();
    if biome != 3 {
        for (i, room) in rooms.iter().enumerate() {
            if i == start_room || room.w < 7 || room.h < 6 || !r.chance(0.4) {
                continue;
            }
            let (cx, cz) = room.center();
            let (px, pz) = (cx + r.range(-1, 2), cz + r.range(-1, 1));
            let mut dug = Vec::new();
            for dz in -1..=1 {
                for dx in -2..=2 {
                    let (x, z) = (px + dx, pz + dz);
                    let edge = dx.abs() == 2 && dz != 0;
                    if !edge
                        && world.wall(x, z) == Wall::None
                        && world.obj(x, z).is_none()
                        && !keep_clear(x, z)
                        && world.floor(x, z) == Floor::Cave
                    {
                        world.set_floor(x, z, Floor::Water);
                        dug.push((x, z));
                    }
                }
            }
            if dug.len() < 4 || !reachable(&world, start, stairs) {
                for (x, z) in dug {
                    world.set_floor(x, z, Floor::Cave);
                }
            } else {
                ponds.push((px, pz));
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
            world.set_obj(
                x,
                z,
                Some(Obj::LootChest {
                    opened: false,
                    gleam: r.chance(GLEAM_CHANCE),
                }),
            );
            placed += 1;
        }
    }

    // Now and then, a cracked patch of floor out in the open, over a secret room.
    let mut crack = None;
    if !is_waystone_floor(depth) && r.chance(CRACK_CHANCE) {
        for _ in 0..200 {
            let i = r.below(n);
            if i == start_room || i == far.1 {
                continue;
            }
            let room = rooms[i];
            let x = room.x + r.range(1, room.w - 1);
            let z = room.z + r.range(1, room.h - 1);
            if world.wall(x, z) == Wall::None
                && world.obj(x, z).is_none()
                && world.floor(x, z) == Floor::Cave
                && !touches_wall(&world, x, z)
                && !keep_clear(x, z)
            {
                world.set_obj(x, z, Some(Obj::Crack));
                crack = Some((x, z));
                break;
            }
        }
    }

    // Enemies.
    let mut spawns = Vec::new();
    let foes = floor_foes(biome, depth);
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
                    break;
                }
            }
        }
    }
    // Water folk by the ponds.
    let folk = pond_foes(biome);
    for &(px, pz) in &ponds {
        for _ in 0..1 + r.below(2) {
            if spawns.len() >= cap + 3 {
                break;
            }
            for _ in 0..12 {
                let x = px + r.range(-3, 4);
                let z = pz + r.range(-2, 3);
                if !world.blocked(x, z) && !keep_clear(x, z) {
                    spawns.push(Spawn {
                        foe: folk[r.below(folk.len())],
                        x: x as f32 + 0.5,
                        z: z as f32 + 0.5,
                        boss: false,
                    });
                    break;
                }
            }
        }
    }
    let lair = lair(&world, stairs);
    if is_waystone_floor(depth) && !via_waystone {
        spawns.push(Spawn {
            foe: boss_for(depth, biome),
            x: lair.0,
            z: lair.1,
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
        lair,
        crack,
    }
}

/// The secret room under a floor's cracked tile: one room, lit by the shaft of daylight the
/// rope hangs in, with treasure along the back wall and its keepers standing guard.
pub fn vault(seed: u64, depth: u32, biome: usize, crack: (i32, i32)) -> Level {
    let key = (crack.0 as u64) << 16 | crack.1 as u64;
    let mut r = Rng::new(seed ^ (depth as u64).wrapping_mul(0x51ED_2701) ^ key ^ 0x5EC2E7);
    // Plenty of rock round the room, so the view never runs off the edge of the world.
    let (w, h) = (25, 23);
    let mut world = World::new(w, h, Area::Hollow { depth }, biome);
    for z in 0..h {
        for x in 0..w {
            world.set_floor(x, z, Floor::Cave);
            let edge = x < 2 || z < 2 || x >= w - 2 || z >= h - 2;
            world.set_wall(x, z, if edge { Wall::Bedrock } else { Wall::Rock });
        }
    }
    let room = Room {
        x: 6,
        z: 6,
        w: 13,
        h: 9,
    };
    carve_room(&mut world, &room, biome != 5, &mut r);
    // Make sure the middle is open whatever the carving did.
    for z in room.z + 1..room.z + room.h - 1 {
        for x in room.x + 2..room.x + room.w - 2 {
            world.set_wall(x, z, Wall::None);
        }
    }
    // The rope down, near the front; you land beside it.
    let rope = (w / 2, room.z + room.h - 2);
    world.set_obj(rope.0, rope.1, Some(Obj::Rope));
    let start = (rope.0 + 1, rope.1);
    // Treasure along the back wall, now and then a gleaming chest among it.
    let chests = 2 + r.below(3);
    let mut placed = 0;
    let mut x = room.x + 2 + r.range(0, 2);
    while placed < chests && x < room.x + room.w - 2 {
        let z = room.z + 1;
        if world.wall(x, z) == Wall::None && world.obj(x, z).is_none() {
            world.set_obj(
                x,
                z,
                Some(Obj::LootChest {
                    opened: false,
                    gleam: r.chance(0.25),
                }),
            );
            placed += 1;
        }
        x += 2 + r.range(0, 2);
    }
    // Pots and crates in the corners, torches either side.
    for (x, z) in [
        (room.x + 2, room.z + room.h - 2),
        (room.x + room.w - 3, room.z + room.h - 2),
        (room.x + 2, room.z + 3),
        (room.x + room.w - 3, room.z + 3),
    ] {
        if world.obj(x, z).is_none() && world.wall(x, z) == Wall::None {
            let hp = 1;
            let o = if r.chance(0.5) {
                Obj::Pot { hp }
            } else {
                Obj::Crate { hp }
            };
            world.set_obj(x, z, Some(o));
        }
    }
    for x in [room.x + 1, room.x + room.w - 2] {
        let z = room.z + room.h / 2;
        if world.wall(x, z) == Wall::None {
            world.set_obj(x, z, Some(Obj::Torch));
        }
    }
    // Its keepers: a handful of the biome's creatures, back from the rope.
    let foes = floor_foes(biome, depth);
    let weights: Vec<f32> = foes.iter().map(|f| f.1).collect();
    let n = (3 + depth as usize / 12).min(7);
    let mut spawns = Vec::new();
    for _ in 0..n * 10 {
        if spawns.len() >= n {
            break;
        }
        let x = room.x + 2 + r.range(0, room.w - 4);
        let z = room.z + 2 + r.range(0, room.h - 5);
        let near = (x - start.0).abs() + (z - start.1).abs() < 4;
        if !world.blocked(x, z) && !near {
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
    }
    Level {
        world,
        start,
        stairs: rope,
        waystone: None,
        spawns,
        lair: (start.0 as f32 + 0.5, start.1 as f32 + 0.5),
        crack: None,
    }
}

/// The roomiest open ground a few steps from the stairs, where a giant has space to move.
fn lair(world: &World, stairs: (i32, i32)) -> (f32, f32) {
    let open = |x: i32, z: i32| world.wall(x, z) == Wall::None;
    let mut best = (i32::MIN, stairs.0, stairs.1 + 2);
    for z in stairs.1 - 6..=stairs.1 + 7 {
        for x in stairs.0 - 7..=stairs.0 + 7 {
            let d = ((x - stairs.0) as f32).hypot((z - stairs.1) as f32);
            if !(2.5..=5.5).contains(&d) || !open(x, z) || world.obj(x, z).is_some() {
                continue;
            }
            let mut room = 0;
            for dz in -2..=2 {
                for dx in -2..=2 {
                    room += i32::from(open(x + dx, z + dz));
                }
            }
            // Roomiest first; then south of the stairs (facing whoever comes to them).
            let score = room * 8 + i32::from(z > stairs.1) * 3 - (d - 3.0).abs() as i32;
            if score > best.0 {
                best = (score, x, z);
            }
        }
    }
    (best.1 as f32 + 0.5, best.2 as f32 + 0.5)
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

/// Can you walk from one tile to another (around water, lava and anything solid)?
pub fn reachable(w: &World, from: (i32, i32), to: (i32, i32)) -> bool {
    let mut seen = vec![false; (w.w * w.h) as usize];
    let mut q = std::collections::VecDeque::new();
    if !w.inside(from.0, from.1) {
        return false;
    }
    seen[w.idx(from.0, from.1)] = true;
    q.push_back(from);
    while let Some((x, z)) = q.pop_front() {
        if (x, z) == to {
            return true;
        }
        for (dx, dz) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
            let (nx, nz) = (x + dx, z + dz);
            // The stairs themselves count as open.
            let open = (nx, nz) == to || !w.blocked(nx, nz);
            if w.inside(nx, nz) && open {
                let i = w.idx(nx, nz);
                if !seen[i] {
                    seen[i] = true;
                    q.push_back((nx, nz));
                }
            }
        }
    }
    false
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
            let biome = biome_for(42, depth);
            let a = generate(42, depth, biome, false);
            let b = generate(42, depth, biome, false);
            assert_eq!(a.start, b.start);
            assert_eq!(a.stairs, b.stairs);
            let dist = flood(&a.world, a.start);
            let i = a.world.idx(a.stairs.0, a.stairs.1);
            assert_ne!(dist[i], u32::MAX, "stairs unreachable on floor {depth}");
            if depth % 10 != 0 && biome != 3 {
                assert!(
                    reachable(&a.world, a.start, a.stairs),
                    "a pond blocks the way on floor {depth}"
                );
            }
            assert!(!a.world.blocked(a.start.0, a.start.1));
            assert_eq!(a.waystone.is_some(), depth % 10 == 0);
        }
    }

    #[test]
    fn ponds_turn_up_with_water_folk_around_them() {
        let mut ponds = 0;
        let mut folk = 0;
        for depth in 1..40 {
            let biome = biome_for(9, depth);
            if biome == 3 {
                continue;
            }
            let l = generate(9, depth, biome, false);
            ponds += l.world.floor.iter().filter(|f| **f == Floor::Water).count();
            folk += l
                .spawns
                .iter()
                .filter(|s| matches!(s.foe, Foe::Frog | Foe::Jelly | Foe::Puffer))
                .count();
        }
        assert!(ponds > 40, "only {ponds} pond tiles");
        assert!(folk > 5, "only {folk} water folk");
    }

    #[test]
    fn every_ten_floors_are_one_biome_in_each_saves_own_order() {
        for seed in 0..50u64 {
            let bands: Vec<usize> = (0..24).map(|b| biome_for(seed, b * 10 + 1)).collect();
            for (b, &biome) in bands.iter().enumerate() {
                // The whole band is one biome...
                let lo = b as u32 * 10 + 1;
                assert!((lo..lo + 10).all(|d| biome_for(seed, d) == biome));
                // ...never the same as the one before it...
                if b > 0 {
                    assert_ne!(biome, bands[b - 1], "seed {seed} band {b}");
                }
            }
            // ...and each run of six has all six.
            for run in bands.chunks(BIOMES) {
                let mut seen = [false; BIOMES];
                run.iter().for_each(|&b| seen[b] = true);
                assert!(seen.iter().all(|&s| s), "seed {seed}: {run:?}");
            }
        }
        // Not the same order every time.
        let firsts: std::collections::HashSet<[usize; 6]> = (0..40u64)
            .map(|seed| std::array::from_fn(|b| biome_for(seed, b as u32 * 10 + 1)))
            .collect();
        assert!(firsts.len() > 20, "only {} orders", firsts.len());
    }

    #[test]
    fn imps_live_in_the_ember_depths_only() {
        assert!(lives_in(Foe::Imp, 3));
        assert!((0..BIOMES).filter(|&b| lives_in(Foe::Imp, b)).count() == 1);
        assert!(lives_in(Foe::Crab, 0), "round the Mossy Burrows' ponds");
        // The deep folk count too: snails everywhere, drakelings where it's hot or cold.
        assert!((0..BIOMES).all(|b| lives_in(Foe::Snail, b)));
        let drakes: Vec<usize> = (0..BIOMES).filter(|&b| lives_in(Foe::Drake, b)).collect();
        assert_eq!(drakes, [3, 4]);
        assert!(lives_in(Foe::Leafling, 0) && !lives_in(Foe::Leafling, 3));
        assert!(
            (0..BIOMES).all(|b| !lives_in(Foe::Werewolf, b)),
            "only the moon brings them"
        );
    }

    #[test]
    fn drakelings_live_deep_and_werewolves_come_with_depth() {
        for biome in 0..BIOMES {
            let shallow = floor_foes(biome, DEEP_FOLK - 1);
            assert!(!shallow.iter().any(|f| f.0 == Foe::Drake));
        }
        assert!(floor_foes(3, DEEP_FOLK).iter().any(|f| f.0 == Foe::Drake));
        assert!(floor_foes(4, 30).iter().any(|f| f.0 == Foe::Drake));
        assert!(!floor_foes(1, 30).iter().any(|f| f.0 == Foe::Drake));
        assert_eq!(werewolves(WEREWOLF_FLOOR - 1), 0);
        assert_eq!(werewolves(WEREWOLF_FLOOR), 1);
        assert_eq!(werewolves(20), 0, "not on a guardian's floor");
        assert_eq!(werewolves(25), 3);
        assert_eq!(werewolves(99), 4);
    }
}
