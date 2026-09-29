//! The tile world shared by the farm and every Hollow floor: floors, walls, objects,
//! collision and the cached chunk meshes.

use glam::{Mat4, Vec2, Vec3};
use serde::{Deserialize, Serialize};

use super::home::Furn;
use super::items::{Crop, Item, Stack};
use super::town::Place;
use crate::assets::models::TALL_TUFTS;
use crate::assets::{Assets, BIOMES, tiles};
use crate::render::{BoxUv, Mesh, TexId, UvRect};
use crate::util::hash2;

pub const CHUNK: i32 = 16;
pub const WALL_H: f32 = 1.0;
pub const WATER_Y: f32 = -0.22;
pub const WATERED: u8 = 1;
/// Watered by a can with Growth: the crop may grow an extra day tonight.
pub const FERTILE: u8 = 2;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize, Default)]
pub enum Floor {
    #[default]
    Void,
    Grass,
    Path,
    Soil,
    Tilled,
    Sand,
    Water,
    Planks,
    Cobble,
    Cave,
    Lava,
    /// Town cobbles.
    Street,
    /// Brick pavers around the fountain.
    Plaza,
    /// Checkered shop tiles.
    Tiles,
    /// Soft carpet.
    Carpet,
    /// A sewer's stone walkway along its channels.
    Walkway,
    /// A plank bridge over a sewer channel.
    Bridge,
    /// A bridge of old copper plate, gone green with age.
    CopperBridge,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize, Default)]
pub enum Wall {
    #[default]
    None,
    Rock,
    /// Ore vein, by ore index (see `assets::ORE_COLORS`).
    Ore(u8),
    Bedrock,
    Cliff,
    Brick,
    Timber,
    /// A trimmed hedge.
    Hedge,
    /// A tall papered wall inside a building, by style.
    Paper(u8),
    /// Old sewer brickwork.
    Sewer,
}

impl Wall {
    /// How tall the wall stands.
    pub fn height(self) -> f32 {
        match self {
            Wall::Paper(_) => 2.0,
            Wall::Hedge => 0.8,
            _ => WALL_H,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Obj {
    Tree {
        var: u8,
        hp: i16,
    },
    Pine {
        var: u8,
        hp: i16,
    },
    Stump {
        hp: i16,
    },
    Log {
        hp: i16,
    },
    Rock {
        var: u8,
        hp: i16,
    },
    Boulder {
        hp: i16,
    },
    Weed {
        var: u8,
    },
    Flower {
        var: u8,
    },
    Crystal {
        var: u8,
        hp: i16,
    },
    Stalagmite {
        var: u8,
    },
    Mushroom {
        var: u8,
    },
    Bones,
    Pot {
        hp: i16,
    },
    Crate {
        hp: i16,
    },
    /// A cracked patch of floor in the Hollow: something's hollow underneath, and a bomb
    /// will open it up.
    Crack,
    /// A blasted-open hole with a rope down to a secret room.
    Hole,
    /// The rope back up out of a secret room, in a shaft of light.
    Rope,
    LootChest {
        opened: bool,
        /// A rare gleaming chest, sparkling gold, with well-rolled treasure inside.
        #[serde(default)]
        gleam: bool,
    },
    StairsDown,
    Waystone,
    Campfire,
    Torch,
    House,
    /// A tile covered by a multi-tile structure anchored elsewhere.
    Part {
        ax: i16,
        az: i16,
    },
    Bin,
    Hollow,
    Stall,
    Sign {
        text: u8,
    },
    Chest {
        items: Vec<Option<Stack>>,
    },
    Lamp,
    Fence,
    Sprinkler {
        tier: u8,
    },
    Workbench,
    FlowerPot {
        var: u8,
    },
    Bench,
    Crop {
        crop: Crop,
        days: u8,
        harvested: bool,
    },
    EnchantTable,
    /// The bus shelter (2x1).
    BusStop,
    /// A town building, by index into `town::BUILDINGS`.
    Building {
        id: u8,
    },
    Fountain {
        flowing: bool,
    },
    StreetLamp {
        lit: bool,
        /// Festival flags strung to the next lamp along.
        bunting: bool,
    },
    /// The Wishing Tree in the park.
    WishTree {
        blooming: bool,
    },
    /// The request board.
    Board,
    Planter {
        var: u8,
    },
    Bush {
        var: u8,
    },
    Well,
    /// A market stand, by colour.
    Stand {
        var: u8,
    },
    Barrel,
    /// A shop counter.
    Counter,
    /// Shelves of goods against a wall, by what they hold.
    Shelf {
        var: u8,
    },
    /// A rack of weapons.
    Rack {
        var: u8,
    },
    /// A wooden dummy showing off an outfit.
    Mannequin {
        var: u8,
    },
    Table {
        var: u8,
    },
    Stool,
    Hearth,
    /// Shop furniture with a special look, by kind (oven, anvil, cauldron...).
    Fixture {
        var: u8,
    },
    /// A wild bush that sprang up on the farm overnight: leafy, blueberry or bramble.
    Shrub {
        var: u8,
        hp: i16,
    },
    /// A piece of furniture in the house, turned `rot` quarter turns; the other tiles it
    /// covers are `Part`s. Fish tanks keep their fish here.
    Furniture {
        f: Furn,
        rot: u8,
        #[serde(default)]
        fish: Vec<Item>,
    },
    /// Rock candy that burst up out of the Hollow's floor where a guardian fell (see
    /// `candy`). A pickaxe knocks it loose.
    CandyRock {
        hp: i16,
    },
    /// Pip's mysterious egg, set down on the farm on day `laid` (see `pets`).
    Egg {
        laid: u32,
    },
    /// A drain pipe coming out of the sewer wall behind, trickling onto the walkway.
    Drain,
    /// A round grate in a sewer walkway.
    Grate,
}

impl Obj {
    pub fn solid(&self) -> bool {
        !matches!(
            self,
            Obj::Weed { .. }
                | Obj::Flower { .. }
                | Obj::Mushroom { .. }
                | Obj::Bones
                | Obj::Crop { .. }
                | Obj::Torch
                | Obj::StairsDown
                | Obj::Crack
                | Obj::Drain
                | Obj::Grate
        )
    }

    /// Light emitted: (height, radius, power, warmth).
    pub fn light(&self) -> Option<(f32, f32, f32, f32)> {
        match self {
            Obj::Torch => Some((0.8, 5.5, 0.72, 7.0)),
            Obj::Campfire => Some((0.4, 6.5, 0.9, 7.5)),
            Obj::Lamp => Some((1.0, 5.0, 0.85, 3.2)),
            Obj::Crystal { .. } => Some((0.5, 4.0, 0.6, 1.5)),
            Obj::CandyRock { .. } => Some((0.4, 3.0, 0.5, 3.0)),
            Obj::Egg { .. } => Some((0.3, 2.0, 0.35, 1.2)),
            Obj::Mushroom { .. } => Some((0.3, 3.0, 0.45, 2.5)),
            Obj::Waystone => Some((1.2, 5.0, 0.8, 2.0)),
            Obj::EnchantTable => Some((0.9, 3.4, 0.5, 1.4)),
            Obj::StreetLamp { lit: true, .. } => Some((1.7, 5.0, 0.55, 5.5)),
            Obj::WishTree { blooming: true } => Some((1.6, 6.0, 0.7, 2.0)),
            Obj::Hearth => Some((0.5, 4.2, 0.42, 6.5)),
            Obj::Furniture { f, .. } => f.def().light,
            // Daylight falling down the shaft from the floor above.
            Obj::Rope => Some((2.2, 4.5, 0.75, 2.2)),
            Obj::LootChest {
                opened: false,
                gleam: true,
            } => Some((0.5, 2.6, 0.45, 6.5)),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum Area {
    Farm,
    Hollow {
        depth: u32,
    },
    Town,
    /// Inside one of the town's buildings.
    Inside(Place),
    /// Inside the farmhouse.
    Home,
}

pub struct World {
    pub w: i32,
    pub h: i32,
    pub floor: Vec<Floor>,
    pub wall: Vec<Wall>,
    pub flags: Vec<u8>,
    pub objs: Vec<Option<Obj>>,
    pub area: Area,
    pub biome: usize,
    /// A room's carpet colour (the house's floor).
    pub style: u8,
    /// Accumulated damage on walls being mined, by tile index.
    pub wall_dmg: std::collections::HashMap<usize, i16>,
    /// Winter: snow lies on the lawns and hedges (see `set_snow`).
    snow: bool,
    /// An old sewer floor of the Hollow: its water is murky green (see `sewer`).
    pub sewer: bool,
    chunks: Vec<Mesh>,
    /// Tufts of grass on the open lawn, in `TUFTS`-tile blocks, drawn swaying in the breeze.
    grass: Vec<Mesh>,
    dirty: Vec<bool>,
    grass_dirty: Vec<bool>,
    cw: i32,
    ch: i32,
    /// Grass blocks across.
    gw: i32,
}

/// Grass is meshed in smaller blocks than the ground, so less of it is drawn off screen.
const TUFTS: i32 = 8;

impl World {
    pub fn new(w: i32, h: i32, area: Area, biome: usize) -> World {
        let n = (w * h) as usize;
        let cw = (w + CHUNK - 1) / CHUNK;
        let ch = (h + CHUNK - 1) / CHUNK;
        let (gw, gh) = ((w + TUFTS - 1) / TUFTS, (h + TUFTS - 1) / TUFTS);
        World {
            w,
            h,
            floor: vec![Floor::Void; n],
            wall: vec![Wall::None; n],
            flags: vec![0; n],
            objs: vec![None; n],
            area,
            biome: biome.min(BIOMES - 1),
            style: 0,
            wall_dmg: Default::default(),
            snow: false,
            sewer: false,
            chunks: vec![Mesh::new(); (cw * ch) as usize],
            grass: vec![Mesh::new(); (gw * gh) as usize],
            dirty: vec![true; (cw * ch) as usize],
            grass_dirty: vec![true; (gw * gh) as usize],
            cw,
            ch,
            gw,
        }
    }

    /// The grass block holding a tile.
    fn tuft_block(&self, x: i32, z: i32) -> usize {
        (z / TUFTS * self.gw + x / TUFTS) as usize
    }

    #[inline]
    pub fn inside(&self, x: i32, z: i32) -> bool {
        x >= 0 && z >= 0 && x < self.w && z < self.h
    }

    #[inline]
    pub fn idx(&self, x: i32, z: i32) -> usize {
        (z * self.w + x) as usize
    }

    pub fn floor(&self, x: i32, z: i32) -> Floor {
        if self.inside(x, z) {
            self.floor[self.idx(x, z)]
        } else {
            Floor::Void
        }
    }

    pub fn wall(&self, x: i32, z: i32) -> Wall {
        if self.inside(x, z) {
            self.wall[self.idx(x, z)]
        } else {
            Wall::Bedrock
        }
    }

    pub fn obj(&self, x: i32, z: i32) -> Option<&Obj> {
        if self.inside(x, z) {
            self.objs[self.idx(x, z)].as_ref()
        } else {
            None
        }
    }

    pub fn obj_mut(&mut self, x: i32, z: i32) -> Option<&mut Obj> {
        if self.inside(x, z) {
            let i = self.idx(x, z);
            self.objs[i].as_mut()
        } else {
            None
        }
    }

    pub fn set_floor(&mut self, x: i32, z: i32, f: Floor) {
        if self.inside(x, z) {
            let i = self.idx(x, z);
            self.floor[i] = f;
            self.touch(x, z);
        }
    }

    pub fn set_wall(&mut self, x: i32, z: i32, w: Wall) {
        if self.inside(x, z) {
            let i = self.idx(x, z);
            self.wall[i] = w;
            self.wall_dmg.remove(&i);
            self.touch(x, z);
        }
    }

    pub fn set_obj(&mut self, x: i32, z: i32, o: Option<Obj>) {
        if self.inside(x, z) {
            let i = self.idx(x, z);
            let stairs =
                matches!(self.objs[i], Some(Obj::StairsDown)) || matches!(o, Some(Obj::StairsDown));
            self.objs[i] = o;
            if stairs {
                self.touch(x, z);
            }
            // Grass only grows where nothing stands.
            let b = self.tuft_block(x, z);
            self.grass_dirty[b] = true;
        }
    }

    pub fn flag(&self, x: i32, z: i32, f: u8) -> bool {
        self.inside(x, z) && self.flags[self.idx(x, z)] & f != 0
    }

    pub fn set_flag(&mut self, x: i32, z: i32, f: u8, on: bool) {
        if self.inside(x, z) {
            let i = self.idx(x, z);
            let before = self.flags[i];
            if on {
                self.flags[i] |= f;
            } else {
                self.flags[i] &= !f;
            }
            if before != self.flags[i] {
                self.touch(x, z);
            }
        }
    }

    /// Marks the chunk holding a tile (and neighbours across edges and corners, whose
    /// rounded corners may change) for re-meshing.
    pub fn touch(&mut self, x: i32, z: i32) {
        for (dx, dz) in [
            (0, 0),
            (-1, 0),
            (1, 0),
            (0, -1),
            (0, 1),
            (-1, -1),
            (1, -1),
            (-1, 1),
            (1, 1),
        ] {
            let (tx, tz) = (x + dx, z + dz);
            if self.inside(tx, tz) {
                let c = (tz / CHUNK * self.cw + tx / CHUNK) as usize;
                self.dirty[c] = true;
                let b = self.tuft_block(tx, tz);
                self.grass_dirty[b] = true;
            }
        }
    }

    pub fn touch_all(&mut self) {
        self.dirty.fill(true);
        self.grass_dirty.fill(true);
    }

    /// Lays or clears winter's snow, re-meshing the ground if that changes it.
    pub fn set_snow(&mut self, snow: bool) {
        if self.snow != snow {
            self.snow = snow;
            self.dirty.fill(true);
        }
    }

    #[cfg(test)]
    pub fn snowy(&self) -> bool {
        self.snow
    }

    /// True if movement is blocked at this tile.
    pub fn blocked(&self, x: i32, z: i32) -> bool {
        if !self.inside(x, z) {
            return true;
        }
        let i = self.idx(x, z);
        if self.wall[i] != Wall::None {
            return true;
        }
        if matches!(self.floor[i], Floor::Water | Floor::Lava | Floor::Void) {
            return true;
        }
        self.objs[i].as_ref().is_some_and(|o| o.solid())
    }

    /// True if the tile stops light.
    pub fn opaque(&self, x: i32, z: i32) -> bool {
        !self.inside(x, z) || self.wall[self.idx(x, z)] != Wall::None
    }

    /// Pushes a circle out of blocked tiles. Returns the corrected position.
    pub fn collide(&self, pos: Vec2, r: f32) -> Vec2 {
        let mut p = pos;
        let x0 = (p.x - r).floor() as i32 - 1;
        let x1 = (p.x + r).floor() as i32 + 1;
        let z0 = (p.y - r).floor() as i32 - 1;
        let z1 = (p.y + r).floor() as i32 + 1;
        for _ in 0..2 {
            for z in z0..=z1 {
                for x in x0..=x1 {
                    if !self.blocked(x, z) {
                        continue;
                    }
                    let (bx0, bx1) = (x as f32, x as f32 + 1.0);
                    let (bz0, bz1) = (z as f32, z as f32 + 1.0);
                    let cx = p.x.clamp(bx0, bx1);
                    let cz = p.y.clamp(bz0, bz1);
                    let d = Vec2::new(p.x - cx, p.y - cz);
                    let dist2 = d.length_squared();
                    if dist2 < r * r {
                        if dist2 > 1e-8 {
                            let dist = dist2.sqrt();
                            p += d / dist * (r - dist);
                        } else {
                            // Centre inside the box: push out along the shallowest axis.
                            let pen = [p.x - bx0, bx1 - p.x, p.y - bz0, bz1 - p.y];
                            let (k, _) = pen
                                .iter()
                                .enumerate()
                                .min_by(|a, b| a.1.total_cmp(b.1))
                                .unwrap();
                            match k {
                                0 => p.x = bx0 - r,
                                1 => p.x = bx1 + r,
                                2 => p.y = bz0 - r,
                                _ => p.y = bz1 + r,
                            }
                        }
                    }
                }
            }
        }
        p
    }

    /// Walks a circle through the world with sliding.
    pub fn move_circle(&self, pos: Vec2, delta: Vec2, r: f32) -> Vec2 {
        let steps = ((delta.length() / 0.2).ceil() as i32).max(1);
        let mut p = pos;
        let d = delta / steps as f32;
        for _ in 0..steps {
            p = self.collide(p + d, r);
        }
        p
    }

    /// Straight line check between two points (for enemy sight and projectiles).
    /// The shortest walk from one tile to another, four ways round walls and whatever
    /// else is in the way: the tiles after `from`, ending at `to`. Empty when there's no
    /// way through (or `from` is `to`).
    pub fn path(&self, from: (i32, i32), to: (i32, i32)) -> Vec<(i32, i32)> {
        if from == to || !self.inside(from.0, from.1) || !self.inside(to.0, to.1) {
            return Vec::new();
        }
        let idx = |x: i32, z: i32| (z * self.w + x) as usize;
        let mut dist = vec![u32::MAX; (self.w * self.h) as usize];
        let mut q = std::collections::VecDeque::new();
        dist[idx(to.0, to.1)] = 0;
        q.push_back(to);
        const WAYS: [(i32, i32); 4] = [(1, 0), (-1, 0), (0, 1), (0, -1)];
        // Out from the goal until the walk reaches `from`.
        while let Some((x, z)) = q.pop_front() {
            if (x, z) == from {
                break;
            }
            let d = dist[idx(x, z)];
            for (dx, dz) in WAYS {
                let (nx, nz) = (x + dx, z + dz);
                if self.inside(nx, nz)
                    && dist[idx(nx, nz)] == u32::MAX
                    && (!self.blocked(nx, nz) || (nx, nz) == from)
                {
                    dist[idx(nx, nz)] = d + 1;
                    q.push_back((nx, nz));
                }
            }
        }
        if dist[idx(from.0, from.1)] == u32::MAX {
            return Vec::new();
        }
        // Then back downhill to it.
        let mut out = Vec::new();
        let mut at = from;
        while at != to {
            let d = dist[idx(at.0, at.1)];
            let Some(next) = WAYS
                .iter()
                .map(|(dx, dz)| (at.0 + dx, at.1 + dz))
                .find(|&(x, z)| self.inside(x, z) && dist[idx(x, z)] < d)
            else {
                break;
            };
            out.push(next);
            at = next;
        }
        out
    }

    pub fn clear_line(&self, a: Vec2, b: Vec2) -> bool {
        let d = b - a;
        let steps = ((d.length() / 0.25).ceil() as i32).max(1);
        for s in 1..steps {
            let p = a + d * (s as f32 / steps as f32);
            let (x, z) = (p.x.floor() as i32, p.y.floor() as i32);
            if self.opaque(x, z) {
                return false;
            }
        }
        true
    }

    /// The anchor of a multi-tile structure (or the tile itself).
    pub fn anchor(&self, x: i32, z: i32) -> (i32, i32) {
        match self.obj(x, z) {
            Some(Obj::Part { ax, az }) => (*ax as i32, *az as i32),
            _ => (x, z),
        }
    }

    // --------------------------------------------------------------------------------------
    // Meshing
    // --------------------------------------------------------------------------------------

    /// Re-meshes dirty chunks that intersect the rectangle.
    pub fn update_meshes(&mut self, a: &Assets, rect: (i32, i32, i32, i32)) {
        let (x0, z0, x1, z1) = rect;
        let cx0 = (x0.max(0) / CHUNK).min(self.cw - 1);
        let cx1 = (x1.max(0) / CHUNK).min(self.cw - 1);
        let cz0 = (z0.max(0) / CHUNK).min(self.ch - 1);
        let cz1 = (z1.max(0) / CHUNK).min(self.ch - 1);
        for cz in cz0..=cz1 {
            for cx in cx0..=cx1 {
                let c = (cz * self.cw + cx) as usize;
                if self.dirty[c] {
                    self.chunks[c] = self.build_chunk(a, cx, cz);
                    self.dirty[c] = false;
                }
            }
        }
        for b in self.tuft_range(rect) {
            if self.grass_dirty[b] {
                let (bx, bz) = (b as i32 % self.gw, b as i32 / self.gw);
                self.grass[b] = self.build_grass(a, bx, bz);
                self.grass_dirty[b] = false;
            }
        }
    }

    /// Chunk meshes that intersect the rectangle.
    pub fn visible_chunks(&self, rect: (i32, i32, i32, i32)) -> Vec<&Mesh> {
        self.chunk_range(rect)
            .into_iter()
            .map(|c| &self.chunks[c])
            .collect()
    }

    /// Grass meshes that intersect the rectangle.
    pub fn visible_grass(&self, rect: (i32, i32, i32, i32)) -> Vec<&Mesh> {
        self.tuft_range(rect)
            .into_iter()
            .map(|b| &self.grass[b])
            .filter(|m| !m.tris.is_empty())
            .collect()
    }

    /// Grass blocks that intersect the rectangle.
    fn tuft_range(&self, rect: (i32, i32, i32, i32)) -> Vec<usize> {
        let (x0, z0, x1, z1) = rect;
        let mut out = Vec::new();
        if x1 < 0 || z1 < 0 || x0 >= self.w || z0 >= self.h {
            return out;
        }
        let gh = (self.h + TUFTS - 1) / TUFTS;
        let bx0 = (x0.max(0) / TUFTS).min(self.gw - 1);
        let bx1 = (x1.max(0) / TUFTS).min(self.gw - 1);
        let bz0 = (z0.max(0) / TUFTS).min(gh - 1);
        let bz1 = (z1.max(0) / TUFTS).min(gh - 1);
        for bz in bz0..=bz1 {
            for bx in bx0..=bx1 {
                out.push((bz * self.gw + bx) as usize);
            }
        }
        out
    }

    fn chunk_range(&self, rect: (i32, i32, i32, i32)) -> Vec<usize> {
        let (x0, z0, x1, z1) = rect;
        let mut out = Vec::new();
        if x1 < 0 || z1 < 0 || x0 >= self.w || z0 >= self.h {
            return out;
        }
        let cx0 = (x0.max(0) / CHUNK).min(self.cw - 1);
        let cx1 = (x1.max(0) / CHUNK).min(self.cw - 1);
        let cz0 = (z0.max(0) / CHUNK).min(self.ch - 1);
        let cz1 = (z1.max(0) / CHUNK).min(self.ch - 1);
        for cz in cz0..=cz1 {
            for cx in cx0..=cx1 {
                out.push((cz * self.cw + cx) as usize);
            }
        }
        out
    }

    /// Scatters tufts over the open lawn of a block, a few to a tile.
    fn build_grass(&self, a: &Assets, bx: i32, bz: i32) -> Mesh {
        let mut m = Mesh::new();
        if !matches!(self.area, Area::Farm | Area::Town) || a.props.tufts.is_empty() {
            return m;
        }
        for z in bz * TUFTS..((bz + 1) * TUFTS).min(self.h) {
            for x in bx * TUFTS..((bx + 1) * TUFTS).min(self.w) {
                let i = self.idx(x, z);
                if self.floor[i] != Floor::Grass
                    || self.wall[i] != Wall::None
                    || self.objs[i].is_some()
                {
                    continue;
                }
                let h = hash2(x, z, 31);
                let n = [1, 2, 2, 3, 1, 2, 3, 2][(h % 8) as usize];
                // Meadows: patches where the grass grows long.
                let meadow = hash2(x.div_euclid(5), z.div_euclid(4), 33) % 4 == 0;
                let short = a.props.tufts.len() - TALL_TUFTS;
                for k in 0..n {
                    let hk = hash2(x * 7 + k, z * 5 - k, 32);
                    let ox = 0.15 + (hk % 70) as f32 / 100.0;
                    let oz = 0.15 + ((hk / 70) % 70) as f32 / 100.0;
                    let rot = ((hk / 4900) % 628) as f32 / 100.0;
                    let s = 0.75 + ((hk / 7) % 50) as f32 / 100.0;
                    let pick = hk as usize / 13;
                    let tall = if meadow {
                        pick % 3 != 0
                    } else {
                        pick % 11 == 0
                    };
                    let tuft = if tall {
                        &a.props.tufts[short + pick % TALL_TUFTS]
                    } else {
                        &a.props.tufts[pick % short]
                    };
                    m.append(
                        tuft,
                        Mat4::from_translation(Vec3::new(x as f32 + ox, 0.0, z as f32 + oz))
                            * Mat4::from_rotation_y(rot)
                            * Mat4::from_scale(Vec3::splat(s)),
                    );
                }
            }
        }
        m
    }

    /// Open lawn (no wall on it): roads round their corners off into it.
    fn lawn(&self, x: i32, z: i32) -> bool {
        self.floor(x, z) == Floor::Grass && self.wall(x, z) == Wall::None
    }

    /// Which road texture a floor uses, as an index into `ROADS` order.
    fn road(&self, x: i32, z: i32) -> Option<usize> {
        if self.wall(x, z) != Wall::None {
            return None;
        }
        match self.floor(x, z) {
            Floor::Path => Some((hash2(x, z, 7) % 2) as usize),
            Floor::Sand => Some(2),
            Floor::Cobble => Some(3),
            Floor::Planks => Some(4),
            Floor::Street => Some(5),
            Floor::Plaza => Some(6),
            _ => None,
        }
    }

    /// Corners of a road tile that stick out into the lawn on both sides.
    fn outer_corners(&self, x: i32, z: i32) -> u8 {
        let (n, s) = (self.lawn(x, z - 1), self.lawn(x, z + 1));
        let (w, e) = (self.lawn(x - 1, z), self.lawn(x + 1, z));
        let mut m = 0;
        if n && w {
            m |= tiles::NW;
        }
        if n && e {
            m |= tiles::NE;
        }
        if s && w {
            m |= tiles::SW;
        }
        if s && e {
            m |= tiles::SE;
        }
        m
    }

    /// Inner corners of a lawn tile where one kind of road bends around it.
    fn inner_corners(&self, x: i32, z: i32) -> Option<(usize, u8)> {
        let kind = |tx: i32, tz: i32| self.road(tx, tz).map(|r| if r < 2 { 0 } else { r });
        let mut found: Option<(usize, u8)> = None;
        for (bit, dx, dz) in [
            (tiles::NW, -1, -1),
            (tiles::NE, 1, -1),
            (tiles::SW, -1, 1),
            (tiles::SE, 1, 1),
        ] {
            let (a, b, c) = (kind(x + dx, z), kind(x, z + dz), kind(x + dx, z + dz));
            let Some(k) = a else { continue };
            if b != Some(k) || c != Some(k) {
                continue;
            }
            match &mut found {
                None => found = Some((k, bit)),
                Some((fk, m)) if *fk == k => *m |= bit,
                _ => {}
            }
        }
        found
    }

    fn floor_tex(&self, a: &Assets, x: i32, z: i32, f: Floor) -> TexId {
        let h = hash2(x, z, 7);
        let lawn = (h % 4) as usize;
        match f {
            Floor::Grass => {
                let (fillets, flowers, grass) = if self.snow {
                    (&a.snow.fillets, &a.snow.flowers, &a.snow.grass)
                } else {
                    (&a.fillets, &a.grass_flowers, &a.grass)
                };
                if let Some((road, mask)) = self.inner_corners(x, z) {
                    return fillets[road * 4 + lawn][mask as usize];
                }
                if h % 11 == 0 {
                    flowers[(h as usize / 11) % 2]
                } else {
                    grass[lawn]
                }
            }
            Floor::Path
            | Floor::Sand
            | Floor::Cobble
            | Floor::Planks
            | Floor::Street
            | Floor::Plaza => {
                let road = self.road(x, z).unwrap_or(0);
                let mask = self.outer_corners(x, z);
                if mask != 0 {
                    let rounded = if self.snow {
                        &a.snow.rounded
                    } else {
                        &a.rounded
                    };
                    return rounded[road * 4 + lawn][mask as usize];
                }
                match f {
                    Floor::Path => a.path[(h % 2) as usize],
                    Floor::Sand => a.sand,
                    Floor::Cobble => a.stone_floor,
                    Floor::Street => a.street[(h % 3) as usize],
                    Floor::Plaza => a.plaza,
                    _ => a.wood_floor,
                }
            }
            Floor::Tiles => a.checker,
            Floor::Carpet => match self.area {
                Area::Inside(Place::Scrolls | Place::Spellery) => a.carpets[1],
                Area::Inside(Place::Jeweler) => a.carpets[2],
                Area::Home => a.carpets[self.style as usize % 3],
                _ => a.carpets[0],
            },
            Floor::Soil => a.soil,
            Floor::Tilled => {
                if self.flag(x, z, WATERED) {
                    a.tilled_wet
                } else {
                    a.tilled
                }
            }
            Floor::Water if self.sewer => a.sewer.water[0],
            Floor::Water => a.water[0],
            Floor::Walkway => a.sewer.walk[self.curb(x, z) as usize],
            Floor::Bridge => a.sewer.wood[self.deck_turn(x, z)],
            Floor::CopperBridge => a.sewer.copper[self.deck_turn(x, z)],
            Floor::Cave => {
                let b = &a.biomes[self.biome];
                b.floor[if h % 5 == 0 { 1 } else { (h % 2) as usize * 2 }]
            }
            Floor::Lava => a.lava[0],
            Floor::Void => a.bedrock,
        }
    }

    fn wall_tex(&self, a: &Assets, w: Wall) -> (TexId, TexId) {
        let b = &a.biomes[self.biome];
        match w {
            Wall::Rock => (b.side, b.top),
            Wall::Ore(o) => (
                b.ore_side[o as usize % b.ore_side.len()],
                b.ore_top[o as usize % b.ore_top.len()],
            ),
            Wall::Bedrock => (b.side, a.bedrock),
            Wall::Cliff if self.snow => (a.cliff_side, a.snow.grass[1]),
            Wall::Cliff => (a.cliff_side, a.grass[1]),
            Wall::Hedge if self.snow => (a.hedge_side, a.snow.hedge_top),
            Wall::Brick => (a.stone_wall_side, a.stone_wall_top),
            Wall::Timber => (a.wood_wall_side, a.wood_wall_top),
            Wall::Hedge => (a.hedge_side, a.hedge_top),
            Wall::Paper(k) => (a.wallpaper[k as usize % a.wallpaper.len()], a.beam),
            Wall::Sewer => (a.sewer.wall_side, a.sewer.wall_top),
            Wall::None => (b.side, b.top),
        }
    }

    /// Which sides of a walkway meet the water (see `sewer_art::NORTH` and so on).
    fn curb(&self, x: i32, z: i32) -> u8 {
        use crate::assets::sewer_art::{EAST, NORTH, SOUTH, WEST};
        [(0, -1, NORTH), (1, 0, EAST), (0, 1, SOUTH), (-1, 0, WEST)]
            .iter()
            .filter(|(dx, dz, _)| self.floor(x + dx, z + dz) == Floor::Water)
            .fold(0, |m, (_, _, bit)| m | bit)
    }

    /// Which way a bridge's boards run: across the way over it (0 along z, when the water
    /// runs north and south of it; 1 along x).
    fn deck_turn(&self, x: i32, z: i32) -> usize {
        let wet = |tx: i32, tz: i32| self.floor(tx, tz) == Floor::Water;
        if wet(x, z - 1) || wet(x, z + 1) { 0 } else { 1 }
    }

    fn floor_y(f: Floor) -> f32 {
        match f {
            Floor::Water => WATER_Y,
            Floor::Lava => -0.12,
            _ => 0.0,
        }
    }

    fn build_chunk(&self, a: &Assets, cx: i32, cz: i32) -> Mesh {
        let mut m = Mesh::new();
        let full = UvRect::new(0.0, 0.0, 16.0, 16.0);
        let solid = |x: i32, z: i32| self.wall(x, z) != Wall::None;
        // A face shows unless a wall at least as tall stands next to it.
        let hides = |x: i32, z: i32, hgt: f32| {
            let w = self.wall(x, z);
            w != Wall::None && w.height() >= hgt
        };
        for z in cz * CHUNK..((cz + 1) * CHUNK).min(self.h) {
            for x in cx * CHUNK..((cx + 1) * CHUNK).min(self.w) {
                let i = self.idx(x, z);
                let (fx, fz) = (x as f32, z as f32);
                let wall = self.wall[i];
                if wall != Wall::None {
                    let (side, top) = self.wall_tex(a, wall);
                    let hgt = wall.height();
                    let top_ao = match wall {
                        Wall::Cliff | Wall::Brick | Wall::Timber | Wall::Hedge => [1.0; 4],
                        Wall::Paper(_) => [0.5; 4],
                        _ => [0.62; 4],
                    };
                    m.quad_ao(
                        [
                            Vec3::new(fx, hgt, fz + 1.0),
                            Vec3::new(fx + 1.0, hgt, fz + 1.0),
                            Vec3::new(fx + 1.0, hgt, fz),
                            Vec3::new(fx, hgt, fz),
                        ],
                        full,
                        top,
                        top_ao,
                    );
                    // South face (towards the camera).
                    if !hides(x, z + 1, hgt) {
                        let base = Self::floor_y(self.floor(x, z + 1)).min(0.0);
                        m.quad(
                            [
                                Vec3::new(fx, base, fz + 1.0),
                                Vec3::new(fx + 1.0, base, fz + 1.0),
                                Vec3::new(fx + 1.0, hgt, fz + 1.0),
                                Vec3::new(fx, hgt, fz + 1.0),
                            ],
                            UvRect::new(0.0, 0.0, 16.0, 16.0 * (hgt - base)),
                            side,
                        );
                    }
                    let tall = UvRect::new(0.0, 0.0, 16.0, 16.0 * hgt);
                    if !hides(x + 1, z, hgt) {
                        m.quad(
                            [
                                Vec3::new(fx + 1.0, 0.0, fz + 1.0),
                                Vec3::new(fx + 1.0, 0.0, fz),
                                Vec3::new(fx + 1.0, hgt, fz),
                                Vec3::new(fx + 1.0, hgt, fz + 1.0),
                            ],
                            tall,
                            side,
                        );
                    }
                    if !hides(x - 1, z, hgt) {
                        m.quad(
                            [
                                Vec3::new(fx, 0.0, fz),
                                Vec3::new(fx, 0.0, fz + 1.0),
                                Vec3::new(fx, hgt, fz + 1.0),
                                Vec3::new(fx, hgt, fz),
                            ],
                            tall,
                            side,
                        );
                    }
                    continue;
                }
                let f = self.floor[i];
                if f == Floor::Void {
                    continue;
                }
                if matches!(self.objs[i], Some(Obj::StairsDown)) {
                    continue;
                }
                let y = Self::floor_y(f);
                // Ambient occlusion at each corner from surrounding walls.
                let ao = |ox: i32, oz: i32| {
                    let mut n = 0;
                    for (dx, dz) in [(-1, -1), (0, -1), (-1, 0), (0, 0)] {
                        if solid(x + ox + dx, z + oz + dz) {
                            n += 1;
                        }
                    }
                    [1.0, 0.82, 0.7, 0.62, 0.6][n]
                };
                let corners = [ao(0, 1), ao(1, 1), ao(1, 0), ao(0, 0)];
                let tex = self.floor_tex(a, x, z, f);
                m.quad_ao(
                    [
                        Vec3::new(fx, y, fz + 1.0),
                        Vec3::new(fx + 1.0, y, fz + 1.0),
                        Vec3::new(fx + 1.0, y, fz),
                        Vec3::new(fx, y, fz),
                    ],
                    full,
                    tex,
                    corners,
                );
                if matches!(f, Floor::Bridge | Floor::CopperBridge) {
                    self.bridge(a, &mut m, x, z, f);
                }
                // Banks where recessed floors (water, lava) meet higher ground.
                if y < 0.0 {
                    // Earthy banks up top; the cave's own rock underground.
                    let bank = if self.sewer {
                        a.sewer.bank
                    } else if f == Floor::Water && !matches!(self.area, Area::Hollow { .. }) {
                        a.soil
                    } else {
                        a.biomes[self.biome].side
                    };
                    let higher = |tx: i32, tz: i32| {
                        let nf = self.floor(tx, tz);
                        !solid(tx, tz)
                            && Self::floor_y(nf) > y
                            && !matches!(nf, Floor::Void | Floor::Bridge | Floor::CopperBridge)
                    };
                    if higher(x, z - 1) {
                        m.quad(
                            [
                                Vec3::new(fx, y, fz),
                                Vec3::new(fx + 1.0, y, fz),
                                Vec3::new(fx + 1.0, 0.0, fz),
                                Vec3::new(fx, 0.0, fz),
                            ],
                            UvRect::new(0.0, 12.0, 16.0, 16.0),
                            bank,
                        );
                    }
                    if higher(x - 1, z) {
                        m.quad(
                            [
                                Vec3::new(fx, y, fz + 1.0),
                                Vec3::new(fx, y, fz),
                                Vec3::new(fx, 0.0, fz),
                                Vec3::new(fx, 0.0, fz + 1.0),
                            ],
                            UvRect::new(0.0, 12.0, 16.0, 16.0),
                            bank,
                        );
                    }
                    if higher(x + 1, z) {
                        m.quad(
                            [
                                Vec3::new(fx + 1.0, y, fz),
                                Vec3::new(fx + 1.0, y, fz + 1.0),
                                Vec3::new(fx + 1.0, 0.0, fz + 1.0),
                                Vec3::new(fx + 1.0, 0.0, fz),
                            ],
                            UvRect::new(0.0, 12.0, 16.0, 16.0),
                            bank,
                        );
                    }
                }
            }
        }
        m
    }

    /// A bridge's underside: the channel running on beneath the deck, a beam along each
    /// edge over the water, and a railing on posts standing up out of it.
    fn bridge(&self, a: &Assets, m: &mut Mesh, x: i32, z: i32, f: Floor) {
        let (fx, fz) = (x as f32, z as f32);
        let beam = if f == Floor::Bridge {
            a.sewer.wood_beam
        } else {
            a.sewer.copper_beam
        };
        let water = self.floor_tex(a, x, z, Floor::Water);
        m.quad(
            [
                Vec3::new(fx, WATER_Y, fz + 1.0),
                Vec3::new(fx + 1.0, WATER_Y, fz + 1.0),
                Vec3::new(fx + 1.0, WATER_Y, fz),
                Vec3::new(fx, WATER_Y, fz),
            ],
            UvRect::new(0.0, 0.0, 16.0, 16.0),
            water,
        );
        let uv = BoxUv::all(UvRect::new(0.0, 0.0, 16.0, 4.0));
        let post_uv = BoxUv::all(UvRect::new(0.0, 0.0, 4.0, 16.0));
        let (t, rail_lo, rail_hi, top) = (0.07, 0.3, 0.37, 0.42);
        let mut post = |px: f32, pz: f32| {
            m.cube(
                Vec3::new(px - t * 0.5, WATER_Y - 0.05, pz - t * 0.5),
                Vec3::new(px + t * 0.5, top, pz + t * 0.5),
                &post_uv,
                beam,
                0,
            );
        };
        let wet = |tx: i32, tz: i32| self.floor(tx, tz) == Floor::Water;
        let (x0, x1, z0, z1) = (fx, fx + 1.0, fz, fz + 1.0);
        let i = t * 0.5;
        // Posts at the corners over the water.
        if wet(x, z - 1) {
            post(x0 + i, z0 + i);
            post(x1 - i, z0 + i);
        }
        if wet(x, z + 1) {
            post(x0 + i, z1 - i);
            post(x1 - i, z1 - i);
        }
        if wet(x - 1, z) {
            post(x0 + i, z0 + i);
            post(x0 + i, z1 - i);
        }
        if wet(x + 1, z) {
            post(x1 - i, z0 + i);
            post(x1 - i, z1 - i);
        }
        // The deck's edge beam and the rail above it, along each side over the water.
        let mut side = |lo: Vec3, hi: Vec3| {
            m.cube(lo, hi, &uv, beam, 0);
        };
        if wet(x, z - 1) {
            side(Vec3::new(x0, -0.1, z0), Vec3::new(x1, 0.02, z0 + t));
            side(Vec3::new(x0, rail_lo, z0), Vec3::new(x1, rail_hi, z0 + t));
        }
        if wet(x, z + 1) {
            side(Vec3::new(x0, -0.1, z1 - t), Vec3::new(x1, 0.02, z1));
            side(Vec3::new(x0, rail_lo, z1 - t), Vec3::new(x1, rail_hi, z1));
        }
        if wet(x - 1, z) {
            side(Vec3::new(x0, -0.1, z0), Vec3::new(x0 + t, 0.02, z1));
            side(Vec3::new(x0, rail_lo, z0), Vec3::new(x0 + t, rail_hi, z1));
        }
        if wet(x + 1, z) {
            side(Vec3::new(x1 - t, -0.1, z0), Vec3::new(x1, 0.02, z1));
            side(Vec3::new(x1 - t, rail_lo, z0), Vec3::new(x1, rail_hi, z1));
        }
    }

    /// Finds the nearest open floor tile to a position (spawning, landing).
    pub fn nearest_open(&self, x: i32, z: i32) -> (i32, i32) {
        for r in 0i32..20 {
            for dz in -r..=r {
                for dx in -r..=r {
                    if dx.abs() != r && dz.abs() != r {
                        continue;
                    }
                    let (tx, tz) = (x + dx, z + dz);
                    if self.inside(tx, tz) && !self.blocked(tx, tz) {
                        return (tx, tz);
                    }
                }
            }
        }
        (x, z)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn winter_lays_snow_on_the_lawn_and_hedges() {
        let a = Assets::new();
        let mut w = World::new(4, 4, Area::Farm, 0);
        for z in 0..4 {
            for x in 0..4 {
                w.set_floor(x, z, Floor::Grass);
            }
        }
        let lawn = |w: &World, a: &Assets| w.floor_tex(a, 1, 1, Floor::Grass);
        let green = lawn(&w, &a);
        assert!(a.grass.contains(&green) || a.grass_flowers.contains(&green));
        w.dirty.fill(false);
        w.set_snow(true);
        assert!(w.dirty.iter().all(|d| *d), "the ground is re-meshed");
        let white = lawn(&w, &a);
        assert!(a.snow.grass.contains(&white) || a.snow.flowers.contains(&white));
        assert_eq!(w.wall_tex(&a, Wall::Hedge).1, a.snow.hedge_top);
        // Laying it again changes nothing.
        w.dirty.fill(false);
        w.set_snow(true);
        assert!(w.dirty.iter().all(|d| !*d));
    }
}
