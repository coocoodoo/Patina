//! Glowcap caves. Now and then a floor of the Hollow (more often in the Fungal Hollow) is a
//! cave lit by glowing mushrooms instead of torches. You come in at the end of a wing of old
//! brick rooms along a torch-lit corridor. The corridor opens into a cave that winds away,
//! lined with glowcaps in blue, cyan, green and purple and shelf fungi up the walls, with
//! side passages ending in quiet alcoves; and at the far end of it a great chamber opens
//! out, with glowing pools round a giant mushroom and the stairs down. Sealed nooks glow in
//! the rock beside the way (a pickaxe opens them up), and past the rock there's nothing but
//! the dark.

use glam::Vec3;

use super::Io;
use super::dungeon::{
    CRACK_CHANCE, Foe, GLEAM_CHANCE, Level, Spawn, floor_foes, is_waystone_floor, ore_weights,
    pond_foes, too_tough,
};
use super::fx::Drop;
use super::gear::Stat;
use super::items::Item;
use super::play::{Play, tile_center};
use super::world::{Area, Floor, Obj, Wall, World};
use crate::assets::glowcave_art::GLOW;
use crate::audio::Sfx;
use crate::palette::WHITE;
use crate::util::{Rng, hash2};

/// How often a floor is glowcap caves: now and then anywhere, and often in the Fungal
/// Hollow. Never the first few floors, a guardian's floor, or the sewers.
pub const GLOWCAVE_CHANCE: f32 = 0.1;
pub const FUNGAL_CHANCE: f32 = 0.35;
pub const FIRST_GLOWCAVE: u32 = 4;

/// Glowcap sizes: a cluster of little ones, a few bigger ones, and one tall one (with a
/// couple of little ones at its foot) that you can't walk through.
pub const SMALL: usize = 0;
pub const MEDIUM: usize = 1;
pub const TALL: usize = 2;

/// Which wall a shelf fungus grows out of, beside the tile it's on.
pub const NORTH: usize = 0;
pub const EAST: usize = 1;
pub const WEST: usize = 2;

/// A glowcap of a colour (0 blue, 1 cyan, 2 green, 3 purple) and a size.
pub fn glowcap(colour: usize, size: usize) -> Obj {
    Obj::Glowcap {
        var: (colour % 4 + 4 * (size % 3)) as u8,
    }
}

pub fn cap_colour(var: u8) -> usize {
    var as usize % 4
}

pub fn cap_size(var: u8) -> usize {
    (var as usize / 4) % 3
}

/// Shelf fungi of a colour growing out of the wall on one side of a tile.
pub fn shelf(colour: usize, side: usize) -> Obj {
    Obj::ShelfFungus {
        var: (colour % 4 + 4 * (side % 3)) as u8,
    }
}

pub fn shelf_side(var: u8) -> usize {
    (var as usize / 4) % 3
}

/// Is this floor glowcap caves?
pub fn is_glowcave(seed: u64, depth: u32, biome: usize) -> bool {
    let chance = if biome == 2 {
        FUNGAL_CHANCE
    } else {
        GLOWCAVE_CHANCE
    };
    depth >= FIRST_GLOWCAVE
        && !is_waystone_floor(depth)
        && !super::sewer::is_sewer(seed, depth)
        && Rng::new(seed ^ (depth as u64).wrapping_mul(0x85EB_CA6B) ^ 0x6C0A).chance(chance)
}

impl Play {
    /// A sickle through a glowcap or shelf fungus: it comes away in a puff of glowing
    /// spores, leaving glowcaps (always from a tall one), glow spores, and now and then
    /// glowcap spores to grow your own. Returns false if there was nothing to gather.
    pub fn gather_glow(&mut self, x: i32, z: i32, io: &mut Io) -> bool {
        let (colour, size) = match self.world().obj(x, z) {
            Some(Obj::Glowcap { var }) => (cap_colour(*var), cap_size(*var)),
            Some(Obj::ShelfFungus { var }) => (cap_colour(*var), SMALL),
            _ => return false,
        };
        let shelf = matches!(self.world().obj(x, z), Some(Obj::ShelfFungus { .. }));
        self.world_mut().set_obj(x, z, None);
        let at = tile_center(x, z) + if shelf { Vec3::Y * 0.6 } else { Vec3::ZERO };
        let [light, mid, _] = GLOW[colour];
        self.fx
            .burst(at + Vec3::Y * 0.25, 10, &[light, mid, WHITE], 1.8, 1.6);
        self.fx.motes(at + Vec3::Y * 0.3, 6, &[light, mid], 0.4);
        io.audio.play_at(Sfx::Swing, 0.5, 1.5);
        let forage = self.player.sheet.frac(Stat::Forage, 60);
        let caps = match size {
            TALL => 1 + self.rng.below(2) as u16,
            MEDIUM => u16::from(self.rng.chance(0.5 + forage * 0.5)),
            _ if shelf => 0,
            _ => u16::from(self.rng.chance(0.2 + forage * 0.4)),
        };
        if caps > 0 {
            self.drops
                .push(Drop::item(Item::Glowcap, caps, at, &mut self.rng));
        }
        if self.rng.chance(if shelf { 0.8 } else { 0.55 }) {
            let n = 1 + self.rng.below(size + 1) as u16;
            self.drops
                .push(Drop::item(Item::Spore, n, at, &mut self.rng));
        }
        if self.rng.chance(0.04 + forage * 0.3) {
            self.drops
                .push(Drop::item(Item::GlowcapSpores, 1, at, &mut self.rng));
        }
        true
    }
}

/// Smooth value noise in -1..1, varying over `cell` tiles.
fn noise(x: f32, z: f32, cell: f32, seed: u32) -> f32 {
    let (gx, gz) = (x / cell, z / cell);
    let (ix, iz) = (gx.floor() as i32, gz.floor() as i32);
    let (tx, tz) = (gx - ix as f32, gz - iz as f32);
    let v = |a: i32, b: i32| (hash2(a, b, seed) % 1001) as f32 / 500.0 - 1.0;
    let s = |t: f32| t * t * (3.0 - 2.0 * t);
    let (sx, sz) = (s(tx), s(tz));
    let top = v(ix, iz) * (1.0 - sx) + v(ix + 1, iz) * sx;
    let bottom = v(ix, iz + 1) * (1.0 - sx) + v(ix + 1, iz + 1) * sx;
    top * (1.0 - sz) + bottom * sz
}

/// A point along a Catmull-Rom curve through `b` and `c`.
fn curve(a: (f32, f32), b: (f32, f32), c: (f32, f32), d: (f32, f32), t: f32) -> (f32, f32) {
    let (t2, t3) = (t * t, t * t * t);
    let f = |a: f32, b: f32, c: f32, d: f32| {
        0.5 * (2.0 * b
            + (c - a) * t
            + (2.0 * a - 5.0 * b + 4.0 * c - d) * t2
            + (3.0 * b - a - 3.0 * c + d) * t3)
    };
    (f(a.0, b.0, c.0, d.0), f(a.1, b.1, c.1, d.1))
}

/// Open ground you can see (not rock, not the dark).
pub(super) fn open(w: &World, x: i32, z: i32) -> bool {
    w.wall(x, z) == Wall::None && w.floor(x, z) != Floor::Void
}

/// Digs out the rock in a disc (never the ruins' brickwork, nor the edge of the world).
fn dig(w: &mut World, cx: f32, cz: f32, rad: f32) {
    let r = rad.ceil() as i32 + 1;
    let (tx, tz) = (cx.floor() as i32, cz.floor() as i32);
    for z in tz - r..=tz + r {
        for x in tx - r..=tx + r {
            let d = (x as f32 + 0.5 - cx).hypot(z as f32 + 0.5 - cz);
            if d <= rad
                && x >= 3
                && z >= 3
                && x < w.w - 3
                && z < w.h - 3
                && w.wall(x, z) == Wall::Rock
            {
                w.set_wall(x, z, Wall::None);
            }
        }
    }
}

/// How many tiles you can walk to from `from`.
fn reach(w: &World, from: (i32, i32)) -> usize {
    let mut seen = vec![false; (w.w * w.h) as usize];
    let mut q = std::collections::VecDeque::new();
    seen[w.idx(from.0, from.1)] = true;
    q.push_back(from);
    let mut n = 0;
    while let Some((x, z)) = q.pop_front() {
        n += 1;
        for (dx, dz) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
            let (nx, nz) = (x + dx, z + dz);
            if w.inside(nx, nz) && !w.blocked(nx, nz) && !seen[w.idx(nx, nz)] {
                seen[w.idx(nx, nz)] = true;
                q.push_back((nx, nz));
            }
        }
    }
    n
}

/// Walking distance from a tile to every tile (u32::MAX where you can't get to).
pub(super) fn walk(w: &World, from: (i32, i32)) -> Vec<u32> {
    let mut dist = vec![u32::MAX; (w.w * w.h) as usize];
    let mut q = std::collections::VecDeque::new();
    dist[w.idx(from.0, from.1)] = 0;
    q.push_back(from);
    while let Some((x, z)) = q.pop_front() {
        let d = dist[w.idx(x, z)];
        for (dx, dz) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
            let (nx, nz) = (x + dx, z + dz);
            if w.inside(nx, nz) && !w.blocked(nx, nz) && dist[w.idx(nx, nz)] == u32::MAX {
                dist[w.idx(nx, nz)] = d + 1;
                q.push_back((nx, nz));
            }
        }
    }
    dist
}

/// Sets something solid down only if everywhere you could walk to before, you still can.
pub(super) fn place_solid(w: &mut World, x: i32, z: i32, o: Obj, from: (i32, i32)) -> bool {
    if w.blocked(x, z) || w.obj(x, z).is_some() {
        return false;
    }
    let before = reach(w, from);
    w.set_obj(x, z, Some(o));
    if reach(w, from) + 1 < before {
        w.set_obj(x, z, None);
        return false;
    }
    true
}

/// Which way a wall stands beside a tile, for shelf fungi: north first (its face is the one
/// you look at), then east or west.
fn wall_beside(w: &World, x: i32, z: i32) -> Option<usize> {
    let rock = |x: i32, z: i32| matches!(w.wall(x, z), Wall::Rock | Wall::Ore(_));
    if rock(x, z - 1) {
        Some(NORTH)
    } else if rock(x + 1, z) {
        Some(EAST)
    } else if rock(x - 1, z) {
        Some(WEST)
    } else {
        None
    }
}

/// Lays a flagstone floor in the ruins.
fn pave(w: &mut World, ruin: &mut [bool], x: i32, z: i32) {
    w.set_wall(x, z, Wall::None);
    w.set_floor(x, z, Floor::Walkway);
    ruin[w.idx(x, z)] = true;
}

/// Rock that could be dug into a sealed nook: nothing open round it but `mine`.
fn sealed(w: &World, x: i32, z: i32, mine: &[(i32, i32)]) -> bool {
    x >= 4
        && z >= 4
        && x < w.w - 4
        && z < w.h - 4
        && w.wall(x, z) == Wall::Rock
        && (-1..=1).all(|dz| {
            (-1..=1).all(|dx| {
                let (nx, nz) = (x + dx, z + dz);
                mine.contains(&(nx, nz)) || w.wall(nx, nz) == Wall::Rock
            })
        })
}

pub fn generate(seed: u64, depth: u32, biome: usize) -> Level {
    let mut r = Rng::new(seed ^ (depth as u64).wrapping_mul(0x9E37_79B9) ^ 0x6C0A_6C0A);
    let noise_seed = hash2(depth as i32, 0x6C0A, seed as u32);
    let grow = depth.min(30) as i32;
    let (w, h) = (58 + grow, 40 + grow * 2 / 3);
    let mut world = World::new(w, h, Area::Hollow { depth }, biome);
    world.glowcave = true;
    for z in 0..h {
        for x in 0..w {
            world.set_floor(x, z, Floor::Cave);
            let edge = x < 2 || z < 2 || x >= w - 2 || z >= h - 2;
            world.set_wall(x, z, if edge { Wall::Bedrock } else { Wall::Rock });
        }
    }
    // Laid out with the ruins to the west and the chamber to the east, or the other way
    // round.
    let flip = r.chance(0.5);
    let fx = |x: i32| if flip { w - 1 - x } else { x };
    let ff = |x: f32| if flip { w as f32 - x } else { x };

    // The ruins: a corridor two wide, and rooms either side of it with doorways onto it.
    let len = r.range(13, 18);
    let cz = (h / 2 + r.range(-4, 5)).clamp(10, h - 12);
    let (x0, x1) = (4, 4 + len - 1);
    let mut rooms: Vec<(i32, i32, i32, i32)> = Vec::new();
    for north in [true, false] {
        let mut x = x0;
        while x1 - x >= 3 {
            let rw = r.range(4, 8).min(x1 - x + 1);
            let rh = r.range(3, 6);
            let rz = if north { cz - 1 - rh } else { cz + 3 };
            if rz >= 4 && rz + rh <= h - 5 && (r.chance(0.8) || rooms.len() < 2) {
                rooms.push((x, rz, rw, rh));
            }
            x += rw + 1;
        }
    }
    let mut ruin = vec![false; (w * h) as usize];
    for x in x0..=x1 {
        pave(&mut world, &mut ruin, fx(x), cz);
        pave(&mut world, &mut ruin, fx(x), cz + 1);
    }
    for &(rx, rz, rw, rh) in &rooms {
        for z in rz..rz + rh {
            for x in rx..rx + rw {
                pave(&mut world, &mut ruin, fx(x), z);
            }
        }
        let door = if rw >= 5 { 2 } else { 1 };
        let dx = rx + r.range(0, rw - door + 1);
        let dz = if rz < cz { cz - 1 } else { cz + 2 };
        for k in 0..door {
            pave(&mut world, &mut ruin, fx(dx + k), dz);
        }
    }
    // Now and then a doorway through to the room next door.
    for &(ax, az, aw, ah) in &rooms {
        for &(bx, bz, _, bh) in &rooms {
            if bx == ax + aw + 1 && (az < cz) == (bz < cz) && r.chance(0.5) {
                let (lo, hi) = (az.max(bz), (az + ah).min(bz + bh));
                if hi > lo {
                    pave(&mut world, &mut ruin, fx(ax + aw), r.range(lo, hi));
                }
            }
        }
    }
    // Old brickwork all round them...
    for z in 1..h - 1 {
        for x in 1..w - 1 {
            if ruin[(z * w + x) as usize] || world.wall(x, z) == Wall::Bedrock {
                continue;
            }
            let by = (-1..=1)
                .any(|dz: i32| (-1..=1).any(|dx: i32| ruin[((z + dz) * w + x + dx) as usize]));
            if by {
                world.set_wall(x, z, Wall::Sewer);
            }
        }
    }
    // ...but for the end of the corridor, open onto the cave.
    for z in [cz, cz + 1] {
        pave(&mut world, &mut ruin, fx(x1 + 1), z);
    }

    // The chamber at the far end.
    let crx = 7.5 + grow as f32 / 7.0 + r.f32() * 1.5;
    let crz = 5.2 + grow as f32 / 10.0 + r.f32() * 1.2;
    let ccx = w as f32 - 4.5 - crx - r.f32() * 1.5;
    let ccz = (h as f32 * 0.5 + r.range(-3, 4) as f32).clamp(crz + 4.5, h as f32 - crz - 5.0);
    for z in 3..h - 3 {
        for x in 3..w - 3 {
            let cx = x as f32 + 0.5;
            let (dx, dz) = ((cx - ccx) / crx, (z as f32 + 0.5 - ccz) / crz);
            let n = noise(ff(cx), z as f32, 3.0, noise_seed);
            if dx * dx + dz * dz < 1.0 + 0.24 * n {
                dig(&mut world, ff(cx), z as f32 + 0.5, 0.5);
            }
        }
    }
    // A cave winding from the ruins to the chamber, swinging north and south on the way.
    let start_of_cave = (x1 as f32 + 2.5, cz as f32 + 1.0);
    let into_chamber = (ccx - crx * 0.75, ccz + r.range(-2, 3) as f32);
    let mut pts = vec![start_of_cave];
    let bends = 2 + r.below(2);
    let mut north = r.chance(0.5);
    for i in 1..=bends {
        let t = i as f32 / (bends + 1) as f32;
        let x = start_of_cave.0 + (into_chamber.0 - start_of_cave.0) * t + r.range(-2, 3) as f32;
        let swing = (h as f32 * 0.5 - 8.0) * r.range_f(0.45, 0.95);
        let z = h as f32 * 0.5 + if north { -swing } else { swing };
        pts.push((x, z.clamp(7.0, h as f32 - 8.0)));
        north = !north;
    }
    pts.push(into_chamber);
    let mut path: Vec<(f32, f32, f32)> = Vec::new();
    let (ph1, ph2) = (r.f32() * 6.3, r.f32() * 6.3);
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
            let p = curve(a, b, c, d, s as f32 / steps as f32);
            if let Some(&(qx, qz, _)) = path.last() {
                along += (p.0 - qx).hypot(p.1 - qz);
            }
            let rad = 1.45
                + 0.5 * ((along * 0.45 + ph1).sin() * 0.5 + 0.5)
                + 0.4 * ((along * 0.17 + ph2).sin() * 0.5 + 0.5);
            path.push((p.0, p.1, rad));
        }
    }
    for &(px, pz, rad) in &path {
        dig(&mut world, ff(px), pz, rad);
    }
    // Side passages, each ending in a quiet alcove.
    let mut alcoves: Vec<(i32, i32)> = Vec::new();
    for _ in 0..2 + r.below(2) {
        let k = r.range(path.len() as i32 / 6, path.len() as i32 * 5 / 6) as usize;
        let (px, pz, _) = path[k];
        let (qx, qz, _) = path[(k + 1).min(path.len() - 1)];
        let (tx, tz) = (qx - px, qz - pz);
        let tl = tx.hypot(tz).max(1e-3);
        let side = if r.chance(0.5) { 1.0 } else { -1.0 };
        let (nx, nz) = (-tz / tl * side, tx / tl * side);
        let reach = r.range_f(5.0, 9.0);
        let (ex, ez) = (px + nx * reach, pz + nz * reach);
        if ex < x1 as f32 + 4.0 || ex > w as f32 - 6.0 || ez < 6.0 || ez > h as f32 - 7.0 {
            continue;
        }
        let steps = (reach / 0.35) as usize;
        for s in 0..=steps {
            let t = s as f32 / steps as f32;
            let wob = (t * 3.1 + side).sin() * 0.8;
            dig(
                &mut world,
                ff(px + nx * reach * t + tx / tl * wob),
                pz + nz * reach * t + tz / tl * wob,
                1.2 + 0.25 * (t * 5.0).sin().abs(),
            );
        }
        let rad = r.range_f(1.9, 2.6);
        dig(&mut world, ff(ex), ez, rad);
        alcoves.push((ff(ex).floor() as i32, ez.floor() as i32));
    }
    // Round off the rock between the passages a little.
    let snapshot: Vec<Wall> = world.wall.clone();
    for z in 3..h - 3 {
        for x in 3..w - 3 {
            if snapshot[(z * w + x) as usize] != Wall::Rock {
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

    // In at the far end of the corridor, on its north side (the wall across the south side
    // would hide you from view).
    let start = (fx(x0 + 1), cz);

    // Nooks sealed off in the rock beside the way, one wall's thickness from it.
    let mut nooks: Vec<Vec<(i32, i32)>> = Vec::new();
    let want = 3 + r.below(4);
    for _ in 0..800 {
        if nooks.len() >= want {
            break;
        }
        let (x, z) = (r.range(4, w - 4), r.range(4, h - 4));
        if !sealed(&world, x, z, &[]) {
            continue;
        }
        let beside = [(0, 2), (0, -2), (2, 0), (-2, 0)].iter().any(|&(dx, dz)| {
            open(&world, x + dx, z + dz)
                && world.floor(x + dx, z + dz) == Floor::Cave
                && world.wall(x + dx / 2, z + dz / 2) == Wall::Rock
        });
        let spaced = nooks
            .iter()
            .flatten()
            .all(|&(nx, nz)| (nx - x).abs() + (nz - z).abs() >= 7);
        let ruins_near = (-3..=3)
            .any(|dz: i32| (-3..=3).any(|dx: i32| world.wall(x + dx, z + dz) == Wall::Sewer));
        if !beside || !spaced || ruins_near {
            continue;
        }
        let mut cells = vec![(x, z)];
        for _ in 0..r.range(1, 4) {
            let (bx, bz) = cells[r.below(cells.len())];
            let (dx, dz) = [(1, 0), (-1, 0), (0, 1), (0, -1)][r.below(4)];
            let c = (bx + dx, bz + dz);
            if !cells.contains(&c) && sealed(&world, c.0, c.1, &cells) {
                cells.push(c);
            }
        }
        for &(nx, nz) in &cells {
            world.set_wall(nx, nz, Wall::None);
        }
        nooks.push(cells);
    }

    // Past the rock round it all, nothing but the dark.
    let solid = world.wall.clone();
    let seen = |x: i32, z: i32| {
        world.inside(x, z)
            && solid[(z * w + x) as usize] == Wall::None
            && world.floor(x, z) != Floor::Void
    };
    let mut dark = Vec::new();
    for z in 0..h {
        for x in 0..w {
            if solid[(z * w + x) as usize] == Wall::None {
                continue;
            }
            if !(-1..=1).any(|dz| (-1..=1).any(|dx| seen(x + dx, z + dz))) {
                dark.push((x, z));
            }
        }
    }
    for (x, z) in dark {
        world.set_wall(x, z, Wall::None);
        world.set_floor(x, z, Floor::Void);
    }

    // The giant mushroom, a little back from the middle of the chamber, on four tiles.
    let giant = (ff(ccx).round() as i32, (ccz - crz * 0.2).round() as i32);
    let (gx, gz) = (giant.0 - 1, giant.1 - 1);
    let footprint = [(gx, gz), (gx + 1, gz), (gx, gz + 1), (gx + 1, gz + 1)];
    let colour = r.below(4);
    if footprint.iter().all(|&(x, z)| open(&world, x, z)) {
        world.set_obj(gx, gz, Some(Obj::GiantShroom { var: colour as u8 }));
        for &(x, z) in &footprint[1..] {
            world.set_obj(
                x,
                z,
                Some(Obj::Part {
                    ax: gx as i16,
                    az: gz as i16,
                }),
            );
        }
    }

    // Glowing pools round it, and one or two along the cave where it opens out. None of
    // them ever cuts anywhere off.
    let mut pools: Vec<(i32, i32)> = Vec::new();
    let mut pool = |world: &mut World, cx: f32, cz: f32, rad: f32| {
        let (tx, tz) = (cx.floor() as i32, cz.floor() as i32);
        let span = rad.ceil() as i32 + 1;
        // Each pool on its own, never run into another.
        let crowded = (tz - span - 2..=tz + span + 2)
            .any(|z| (tx - span - 2..=tx + span + 2).any(|x| world.floor(x, z) == Floor::Water));
        if crowded {
            return false;
        }
        let before = reach(world, start);
        let mut dug = Vec::new();
        for z in tz - span..=tz + span {
            for x in tx - span..=tx + span {
                let d = (x as f32 + 0.5 - cx).hypot(z as f32 + 0.5 - cz)
                    + noise(x as f32, z as f32, 2.0, noise_seed ^ 0x9001) * 0.2;
                let clear = (-1..=1).all(|dz: i32| {
                    (-1..=1).all(|dx: i32| {
                        let (nx, nz) = (x + dx, z + dz);
                        open(world, nx, nz)
                            && !matches!(world.obj(nx, nz), Some(Obj::GiantShroom { .. }))
                            && !matches!(world.obj(nx, nz), Some(Obj::Part { .. }))
                    })
                });
                if d <= rad
                    && clear
                    && world.floor(x, z) == Floor::Cave
                    && world.obj(x, z).is_none()
                    && !ruin[world.idx(x, z)]
                    && (x - start.0).abs() + (z - start.1).abs() > 3
                {
                    world.set_floor(x, z, Floor::Water);
                    dug.push((x, z));
                }
            }
        }
        if dug.len() < 3 || reach(world, start) + dug.len() < before {
            for (x, z) in dug {
                world.set_floor(x, z, Floor::Cave);
            }
            return false;
        }
        pools.push((tx, tz));
        true
    };
    let mut tries = 0;
    let mut made = 0;
    while made < 2 + r.below(2) && tries < 60 {
        tries += 1;
        let a = r.f32() * std::f32::consts::TAU;
        let d = r.range_f(0.35, 0.8);
        let (px, pz) = (ccx + a.cos() * crx * d, ccz + a.sin() * crz * d);
        let (px, pz) = (ff(px), pz);
        if (px - giant.0 as f32).hypot(pz - giant.1 as f32) < 3.6 {
            continue;
        }
        let rad = r.range_f(1.3, 2.2);
        if pool(&mut world, px, pz, rad) {
            made += 1;
        }
    }
    let wide: Vec<&(f32, f32, f32)> = path.iter().filter(|p| p.2 > 2.05).collect();
    for _ in 0..r.below(3) {
        if wide.is_empty() {
            break;
        }
        let &(px, pz, _) = wide[r.below(wide.len())];
        let (ox, oz) = (r.range_f(-0.8, 0.8), r.range_f(-0.8, 0.8));
        let rad = r.range_f(0.9, 1.4);
        pool(&mut world, ff(px + ox), pz + oz, rad);
    }

    // Down the stairs at the far side of the chamber: as far a walk from the ruins as it
    // gets.
    let dist = walk(&world, start);
    let in_chamber = |x: i32, z: i32| {
        let cx = ff(x as f32 + 0.5);
        let (dx, dz) = ((cx - ccx) / crx, (z as f32 + 0.5 - ccz) / crz);
        dx * dx + dz * dz < 0.9
    };
    let wet_by = |world: &World, x: i32, z: i32| {
        (-1..=1).any(|dz| (-1..=1).any(|dx| world.floor(x + dx, z + dz) == Floor::Water))
    };
    let mut far = (0, None);
    for z in 0..h {
        for x in 0..w {
            let d = dist[world.idx(x, z)];
            if d != u32::MAX
                && d > far.0
                && in_chamber(x, z)
                && world.floor(x, z) == Floor::Cave
                && world.obj(x, z).is_none()
                && !wet_by(&world, x, z)
                && (x - giant.0).abs().max((z - giant.1).abs()) > 3
            {
                far = (d, Some((x, z)));
            }
        }
    }
    let stairs = far.1.unwrap_or_else(|| {
        // However the chamber came out, somewhere as far off as you can walk.
        let mut best = (0, start);
        for z in 0..h {
            for x in 0..w {
                let d = dist[world.idx(x, z)];
                if d != u32::MAX && d > best.0 && world.obj(x, z).is_none() {
                    best = (d, (x, z));
                }
            }
        }
        best.1
    });
    world.set_obj(stairs.0, stairs.1, Some(Obj::StairsDown));
    let keep_clear = |x: i32, z: i32| {
        (x - start.0).abs() <= 1 && (z - start.1).abs() <= 1
            || (x - stairs.0).abs() <= 1 && (z - stairs.1).abs() <= 1
    };

    // Ore seams in the rock along the way.
    let ores = ore_weights(depth, biome);
    for z in 2..h - 2 {
        for x in 2..w - 2 {
            let faces = [(1, 0), (-1, 0), (0, 1), (0, -1)]
                .iter()
                .any(|&(dx, dz)| open(&world, x + dx, z + dz));
            if world.wall(x, z) == Wall::Rock && faces && r.chance(0.035) {
                world.set_wall(x, z, Wall::Ore(r.weighted(&ores) as u8));
            }
        }
    }

    // The ruins: torches along the corridor, and in the rooms old barrels, crates and pots,
    // bones, a torch each, and a chest in one of them.
    for (k, x) in (x0 + 1..=x1).step_by(4).enumerate() {
        let z = if k % 2 == 0 { cz } else { cz + 1 };
        let wall = if z == cz { z - 1 } else { z + 1 };
        let wx = fx(x);
        if world.wall(wx, wall) == Wall::Sewer && !keep_clear(wx, z) {
            world.set_obj(wx, z, Some(Obj::Torch));
        }
    }
    let chest_room = r.below(rooms.len().max(1));
    for (i, &(rx, rz, rw, rh)) in rooms.iter().enumerate() {
        let mut spots: Vec<(i32, i32)> = Vec::new();
        for z in rz..rz + rh {
            for x in rx..rx + rw {
                let wx = fx(x);
                let by_wall = [(1, 0), (-1, 0), (0, 1), (0, -1)]
                    .iter()
                    .any(|&(dx, dz)| world.wall(wx + dx, z + dz) == Wall::Sewer);
                if by_wall && world.obj(wx, z).is_none() && !keep_clear(wx, z) {
                    spots.push((wx, z));
                }
            }
        }
        r.shuffle(&mut spots);
        let mut spots = spots.into_iter();
        if let Some((x, z)) = spots.next() {
            world.set_obj(x, z, Some(Obj::Torch));
        }
        if i == chest_room {
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
            let o = match r.below(10) {
                0..=3 => Obj::Keg { hp: 1 },
                4..=6 => Obj::Crate { hp: 1 },
                _ => Obj::Pot { hp: 1 },
            };
            place_solid(&mut world, x, z, o, start);
        }
        if let Some((x, z)) = spots.next() {
            world.set_obj(x, z, Some(Obj::Bones));
        }
    }
    for _ in 0..3 {
        let (x, z) = (fx(r.range(x0, x1 + 1)), cz + r.range(0, 2));
        if world.obj(x, z).is_none() && !keep_clear(x, z) {
            world.set_obj(x, z, Some(Obj::Bones));
        }
    }

    // Treasure in an alcove, now and then, and in a sealed nook for whoever digs in.
    let mut nook_chest = None;
    if r.chance(0.35) && !nooks.is_empty() {
        let n = r.below(nooks.len());
        let (x, z) = nooks[n][0];
        world.set_obj(
            x,
            z,
            Some(Obj::LootChest {
                opened: false,
                gleam: r.chance(GLEAM_CHANCE * 3.0),
            }),
        );
        nook_chest = Some(n);
    }
    if r.chance(0.6) {
        if let Some(&(ax, az)) = alcoves.get(r.below(alcoves.len().max(1))) {
            for (dx, dz) in [(0, -2), (0, -1), (1, -1), (-1, -1), (0, 0)] {
                let chest = Obj::LootChest {
                    opened: false,
                    gleam: r.chance(GLEAM_CHANCE),
                };
                let (x, z) = (ax + dx, az + dz);
                if !keep_clear(x, z)
                    && world.floor(x, z) == Floor::Cave
                    && place_solid(&mut world, x, z, chest, start)
                {
                    break;
                }
            }
        }
    }

    // Glowcaps everywhere along the walls, in patches of one colour, and shelf fungi up the
    // walls; thicker in the alcoves and round the pools, and filling the nooks.
    let patch = |x: i32, z: i32, r: &mut Rng| {
        let c = hash2(x.div_euclid(5), z.div_euclid(4), noise_seed ^ 0xC0) as usize % 4;
        if r.chance(0.12) { r.below(4) } else { c }
    };
    let in_alcove = |x: i32, z: i32| {
        alcoves
            .iter()
            .any(|&(ax, az)| (ax - x).abs() + (az - z).abs() <= 3)
    };
    for z in 2..h - 2 {
        for x in 2..w - 2 {
            let i = world.idx(x, z);
            if ruin[i]
                || !open(&world, x, z)
                || world.floor(x, z) != Floor::Cave
                || world.obj(x, z).is_some()
                || keep_clear(x, z)
                || nooks.iter().any(|n| n.contains(&(x, z)))
            {
                continue;
            }
            let colour = patch(x, z, &mut r);
            let by_pool = [(1, 0), (-1, 0), (0, 1), (0, -1)]
                .iter()
                .any(|&(dx, dz)| world.floor(x + dx, z + dz) == Floor::Water);
            match wall_beside(&world, x, z) {
                Some(side) => {
                    let thick = if in_alcove(x, z) { 1.6 } else { 1.0 };
                    let roll = r.f32();
                    if roll < 0.1 * thick && (side == NORTH || r.chance(0.5)) {
                        world.set_obj(x, z, Some(shelf(colour, side)));
                    } else if roll < 0.34 * thick {
                        let size = match r.below(100) {
                            0..=51 => SMALL,
                            52..=85 => MEDIUM,
                            _ => TALL,
                        };
                        let cap = glowcap(colour, size);
                        if size == TALL {
                            if !place_solid(&mut world, x, z, cap, start) {
                                world.set_obj(x, z, Some(glowcap(colour, MEDIUM)));
                            }
                        } else {
                            world.set_obj(x, z, Some(cap));
                        }
                    }
                }
                None if by_pool && r.chance(0.28) => {
                    let size = if r.chance(0.6) { SMALL } else { MEDIUM };
                    world.set_obj(x, z, Some(glowcap(colour, size)));
                }
                None if r.chance(0.022) => {
                    world.set_obj(x, z, Some(glowcap(colour, SMALL)));
                }
                None if r.chance(0.008) => world.set_obj(x, z, Some(Obj::Bones)),
                None => {}
            }
        }
    }
    for (n, cells) in nooks.iter().enumerate() {
        let colour = r.below(4);
        for &(x, z) in cells {
            if world.obj(x, z).is_some() {
                continue;
            }
            let size = match r.below(10) {
                0..=3 => TALL,
                4..=8 => MEDIUM,
                _ => SMALL,
            };
            let size = if nook_chest == Some(n) { SMALL } else { size };
            world.set_obj(x, z, Some(glowcap(colour, size)));
        }
    }
    // A couple of little glowcaps creeping in at the end of the corridor.
    for z in [cz, cz + 1] {
        let x = fx(x1 - r.range(0, 2));
        if world.obj(x, z).is_none() && r.chance(0.7) {
            world.set_obj(x, z, Some(glowcap(r.below(4), SMALL)));
        }
    }

    // Now and then, a cracked patch of cave floor over a secret room.
    let mut crack = None;
    if r.chance(CRACK_CHANCE) {
        for _ in 0..400 {
            let (x, z) = (r.range(3, w - 3), r.range(3, h - 3));
            let clear = (-1..=1).all(|dz| {
                (-1..=1).all(|dx| {
                    open(&world, x + dx, z + dz) && world.floor(x + dx, z + dz) == Floor::Cave
                })
            });
            if clear
                && world.obj(x, z).is_none()
                && !keep_clear(x, z)
                && !ruin[world.idx(x, z)]
                && dist[world.idx(x, z)] != u32::MAX
            {
                world.set_obj(x, z, Some(Obj::Crack));
                crack = Some((x, z));
                break;
            }
        }
    }

    // Who lives here: the biome's folk, and plenty of shroomlings glowing like the caps;
    // and water folk round the pools.
    let mut foes = floor_foes(biome, depth);
    foes.push((Foe::Shroom, 3.0));
    let weights: Vec<f32> = foes.iter().map(|f| f.1).collect();
    let cap = (6 + depth as usize / 2).min(28);
    let dist = walk(&world, start);
    let mut spawns = Vec::new();
    for _ in 0..cap * 40 {
        if spawns.len() >= cap {
            break;
        }
        let (x, z) = (r.range(2, w - 2), r.range(2, h - 2));
        let d = dist[world.idx(x, z)];
        if d == u32::MAX || d < 10 || world.blocked(x, z) || keep_clear(x, z) {
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
    let folk = pond_foes(2);
    for &(px, pz) in &pools {
        for _ in 0..20 {
            let (x, z) = (px + r.range(-3, 4), pz + r.range(-3, 4));
            let wet = [(1, 0), (-1, 0), (0, 1), (0, -1)]
                .iter()
                .any(|&(dx, dz)| world.floor(x + dx, z + dz) == Floor::Water);
            if wet && !world.blocked(x, z) && dist[world.idx(x, z)] != u32::MAX {
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

    /// Some glowcap caves to look at, whatever the chance says.
    fn caves() -> Vec<Level> {
        [(1u64, 4u32), (7, 12), (42, 25), (9, 47), (123, 68), (5, 99)]
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
    fn the_stairs_are_a_long_walk_from_the_ruins() {
        for l in caves() {
            let w = &l.world;
            assert!(w.glowcave);
            let dist = walk(w, l.start);
            let d = dist[w.idx(l.stairs.0, l.stairs.1)];
            assert_ne!(d, u32::MAX, "the stairs are out of reach");
            assert!(d > 25, "only {d} steps");
            assert_eq!(
                w.floor(l.start.0, l.start.1),
                Floor::Walkway,
                "in the ruins"
            );
            assert_eq!(w.floor(l.stairs.0, l.stairs.1), Floor::Cave);
        }
    }

    #[test]
    fn ruins_caves_and_a_chamber_all_lit_up() {
        for l in caves() {
            let bricks = count(&l, |w, x, z| w.wall(x, z) == Wall::Sewer);
            let torches = count(&l, |w, x, z| matches!(w.obj(x, z), Some(Obj::Torch)));
            let caps = count(&l, |w, x, z| {
                matches!(w.obj(x, z), Some(Obj::Glowcap { .. }))
            });
            let shelves = count(&l, |w, x, z| {
                matches!(w.obj(x, z), Some(Obj::ShelfFungus { .. }))
            });
            let giants = count(&l, |w, x, z| {
                matches!(w.obj(x, z), Some(Obj::GiantShroom { .. }))
            });
            let pools = count(&l, |w, x, z| w.floor(x, z) == Floor::Water);
            let dark = count(&l, |w, x, z| w.floor(x, z) == Floor::Void);
            assert!(bricks > 30, "{bricks} bricks");
            assert!(torches >= 4, "{torches} torches");
            assert!(caps > 40, "{caps} glowcaps");
            assert!(shelves > 5, "{shelves} shelf fungi");
            assert_eq!(giants, 1);
            assert!(pools >= 6, "{pools} tiles of pool");
            assert!(dark > 200, "{dark} tiles of the dark");
            // Every colour turns up.
            for c in 0..4 {
                assert!(
                    count(&l, |w, x, z| matches!(
                        w.obj(x, z),
                        Some(Obj::Glowcap { var }) if cap_colour(*var) == c
                    )) > 0
                );
            }
        }
    }

    /// Where you could walk if nothing were in the way: everywhere but rock, water and the
    /// dark.
    fn open_ground(w: &World, from: (i32, i32)) -> Vec<bool> {
        let mut seen = vec![false; (w.w * w.h) as usize];
        let mut q = std::collections::VecDeque::from([from]);
        seen[w.idx(from.0, from.1)] = true;
        while let Some((x, z)) = q.pop_front() {
            for (dx, dz) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                let (nx, nz) = (x + dx, z + dz);
                if open(w, nx, nz) && w.floor(nx, nz) != Floor::Water && !seen[w.idx(nx, nz)] {
                    seen[w.idx(nx, nz)] = true;
                    q.push_back((nx, nz));
                }
            }
        }
        seen
    }

    #[test]
    fn nooks_are_sealed_but_glowing() {
        let mut any = 0;
        for l in caves() {
            let w = &l.world;
            let ground = open_ground(w, l.start);
            for z in 1..w.h - 1 {
                for x in 1..w.w - 1 {
                    // Open ground apart from the rest is a nook: shut in by rock all round,
                    // with something glowing (or a chest) in it.
                    if !open(w, x, z) || w.floor(x, z) != Floor::Cave || ground[w.idx(x, z)] {
                        continue;
                    }
                    for dz in -1..=1 {
                        for dx in -1..=1 {
                            let (nx, nz) = (x + dx, z + dz);
                            assert!(
                                !ground[w.idx(nx, nz)],
                                "a nook open to the cave at ({x}, {z})"
                            );
                        }
                    }
                    assert!(matches!(
                        w.obj(x, z),
                        Some(Obj::Glowcap { .. } | Obj::LootChest { .. })
                    ));
                    any += 1;
                }
            }
        }
        assert!(any > 12, "only {any} nook tiles");
    }

    #[test]
    fn nothing_shuts_the_way_anywhere() {
        for seed in 0..30u64 {
            for depth in [4u32, 17, 33, 52, 76] {
                let seed = seed * 7919 + 11;
                let l = generate(seed, depth, biome_for(seed, depth));
                let w = &l.world;
                let dist = walk(w, l.start);
                assert_ne!(dist[w.idx(l.stairs.0, l.stairs.1)], u32::MAX);
                // Everywhere in the ruins and along the cave you could walk if nothing
                // stood in the way, you can.
                let ground = open_ground(w, l.start);
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
    fn deterministic() {
        let a = generate(77, 23, 2);
        let b = generate(77, 23, 2);
        assert_eq!(a.start, b.start);
        assert_eq!(a.stairs, b.stairs);
        assert_eq!(a.world.floor, b.world.floor);
        assert_eq!(a.world.wall, b.world.wall);
    }

    #[test]
    fn more_often_in_the_fungal_hollow() {
        let count = |biome: usize| {
            (0..400u64)
                .filter(|&seed| is_glowcave(seed, 23, biome))
                .count()
        };
        let (fungal, frost) = (count(2), count(4));
        assert!(fungal > frost * 2, "{fungal} vs {frost}");
        assert!(frost > 10, "{frost}");
        assert!(
            !(0..400u64).any(|seed| is_glowcave(seed, 20, 2)),
            "never a guardian's"
        );
        assert!(
            !(0..400u64).any(|seed| is_glowcave(seed, 3, 2)),
            "not so soon"
        );
    }
}
