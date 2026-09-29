//! Home: the inside of the farmhouse. Furniture goes wherever you like, rugs on the floor,
//! pictures on the back wall, new wallpaper and floors, fish in their tanks. All of it adds up
//! to your charisma, which shopkeepers and neighbours notice. The stove is where the cooking
//! happens.

use std::collections::HashMap;

use glam::{Vec2, Vec3};
use serde::{Deserialize, Serialize};

use super::Io;
use super::items::{ALL_ITEMS, CATS, Cat, Item, Kind, Placeable, RECIPES, Stack};
use super::menus::{Grid, Menu, Tab, bag_grid, panel_layout};
use super::play::{Banner, Play, Trans, tile_center};
use super::player::RADIUS;
use super::town::Place;
use super::world::{Area, Floor, Obj, Wall, World};
use crate::assets::Assets;
use crate::audio::Sfx;
use crate::input::{Action, Button};
use crate::palette::*;
use crate::ui::{Canvas, Style};

/// Every kind of furniture.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, Serialize, Deserialize)]
pub enum Furn {
    Bed,
    CanopyBed,
    Stove,
    Range,
    Counter,
    Icebox,
    RoundTable,
    DiningTable,
    Chair,
    Armchair,
    Sofa,
    Bookshelf,
    Wardrobe,
    Dresser,
    FloorLamp,
    SnailLamp,
    Candelabra,
    Fireplace,
    Fern,
    Cactus,
    Vase,
    Clock,
    Piano,
    Globe,
    Telescope,
    Plush,
    FishBowl,
    FishTank,
}

pub const FURNS: [Furn; 28] = [
    Furn::Bed,
    Furn::CanopyBed,
    Furn::Stove,
    Furn::Range,
    Furn::Counter,
    Furn::Icebox,
    Furn::RoundTable,
    Furn::DiningTable,
    Furn::Chair,
    Furn::Armchair,
    Furn::Sofa,
    Furn::Bookshelf,
    Furn::Wardrobe,
    Furn::Dresser,
    Furn::FloorLamp,
    Furn::SnailLamp,
    Furn::Candelabra,
    Furn::Fireplace,
    Furn::Fern,
    Furn::Cactus,
    Furn::Vase,
    Furn::Clock,
    Furn::Piano,
    Furn::Globe,
    Furn::Telescope,
    Furn::Plush,
    Furn::FishBowl,
    Furn::FishTank,
];

/// What using a piece of furniture does.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Use {
    Nothing,
    Sleep,
    Cook,
    Sit,
    Tank,
    Play,
    Clock,
    Spin,
    Stars,
    Squeak,
    Read,
    Warm,
    Tidy,
}

pub struct FurnDef {
    /// Tiles across and deep before it is turned.
    pub size: (i32, i32),
    /// How much it adds to your charisma.
    pub charm: u32,
    pub use_: Use,
    /// How many fish it holds.
    pub cap: usize,
    /// Light it gives: (height, radius, power, warmth).
    pub light: Option<(f32, f32, f32, f32)>,
}

impl Furn {
    pub fn def(self) -> FurnDef {
        use Furn::*;
        let d = |size, charm, use_| FurnDef {
            size,
            charm,
            use_,
            cap: 0,
            light: None,
        };
        let lit = |size, charm, use_, light| FurnDef {
            light: Some(light),
            ..d(size, charm, use_)
        };
        match self {
            Bed => d((1, 2), 3, Use::Sleep),
            CanopyBed => d((2, 2), 14, Use::Sleep),
            Stove => lit((1, 1), 3, Use::Cook, (0.4, 2.2, 0.22, 7.0)),
            Range => lit((2, 1), 12, Use::Cook, (0.4, 2.6, 0.25, 7.0)),
            Counter => d((1, 1), 2, Use::Nothing),
            Icebox => d((1, 1), 3, Use::Tidy),
            RoundTable => d((1, 1), 3, Use::Nothing),
            DiningTable => d((2, 1), 6, Use::Nothing),
            Chair => d((1, 1), 1, Use::Sit),
            Armchair => d((1, 1), 5, Use::Sit),
            Sofa => d((2, 1), 9, Use::Sit),
            Bookshelf => d((1, 1), 6, Use::Read),
            Wardrobe => d((1, 1), 5, Use::Tidy),
            Dresser => d((1, 1), 3, Use::Tidy),
            FloorLamp => lit((1, 1), 3, Use::Nothing, (1.3, 4.0, 0.45, 5.0)),
            SnailLamp => lit((1, 1), 9, Use::Nothing, (1.0, 4.4, 0.5, 3.0)),
            Candelabra => lit((1, 1), 5, Use::Nothing, (1.0, 3.5, 0.4, 6.0)),
            Fireplace => lit((2, 1), 12, Use::Warm, (0.5, 4.5, 0.5, 6.5)),
            Fern => d((1, 1), 2, Use::Nothing),
            Cactus => d((1, 1), 2, Use::Nothing),
            Vase => d((1, 1), 3, Use::Nothing),
            Clock => d((1, 1), 8, Use::Clock),
            Piano => d((2, 1), 16, Use::Play),
            Globe => d((1, 1), 4, Use::Spin),
            Telescope => d((1, 1), 6, Use::Stars),
            Plush => d((1, 1), 2, Use::Squeak),
            FishBowl => FurnDef {
                cap: 2,
                ..lit((1, 1), 2, Use::Tank, (0.6, 2.0, 0.2, 2.0))
            },
            FishTank => FurnDef {
                cap: 6,
                ..lit((2, 1), 6, Use::Tank, (0.8, 3.2, 0.3, 2.0))
            },
        }
    }

    /// The item this piece is picked up as.
    pub fn item(self) -> Item {
        ALL_ITEMS
            .iter()
            .copied()
            .find(|i| i.def().kind == Kind::Place(Placeable::Furniture(self)))
            .unwrap_or(Item::WoodenChair)
    }

    pub fn name(self) -> &'static str {
        self.item().def().name
    }

    /// Charisma for each fish swimming in it.
    pub fn per_fish(self) -> u32 {
        match self {
            Furn::FishTank => 2,
            Furn::FishBowl => 1,
            _ => 0,
        }
    }
}

/// Rugs: (charisma, colours: base, pattern, edge).
pub const RUGS: [(u32, [u8; 3]); 4] = [
    (3, [SALMON, CREAM, CRIMSON]),
    (4, [SKY, WHITE, BLUE]),
    (5, [INDIGO, GOLD, SLATE]),
    (4, [AQUA, MINT, TEAL]),
];

/// Pictures for the back wall: charisma for each.
pub const ART: [u32; 4] = [4, 4, 6, 6];

/// Floors you can lay in the house: the floor, and its carpet colour where it has one.
pub const FLOORS: [(Floor, u8); 6] = [
    (Floor::Planks, 0),
    (Floor::Tiles, 0),
    (Floor::Carpet, 0),
    (Floor::Carpet, 1),
    (Floor::Carpet, 2),
    (Floor::Cobble, 0),
];

pub const HOUSE_W: i32 = 15;
pub const HOUSE_H: i32 = 11;
/// The doormat you step in on (and out through).
pub const HOUSE_DOOR: (i32, i32) = (7, 10);
/// Windows in the back wall, by column.
pub const WINDOWS: [i32; 2] = [3, 11];
/// The wallpaper and floor a new house has.
pub const HOME_PAPER: u8 = 8;
pub const HOME_FLOOR: u8 = 0;

/// A rug on the floor, covering two by two tiles from its corner.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Rug {
    pub x: i32,
    pub z: i32,
    pub kind: u8,
}

impl Rug {
    pub fn covers(&self, x: i32, z: i32) -> bool {
        x >= self.x && x <= self.x + 1 && z >= self.z && z <= self.z + 1
    }
}

/// The inside of the farmhouse.
pub struct House {
    pub world: World,
    pub rugs: Vec<Rug>,
    /// Pictures on the back wall: (column, kind).
    pub art: Vec<(i32, u8)>,
    /// Wallpaper style (`Wall::Paper`) and floor (index into `FLOORS`).
    pub paper: u8,
    pub floor: u8,
    /// Extra quarter turns for the next piece placed (T turns it).
    pub turn: u8,
}

/// What goes into a save.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct HouseSave {
    pub objs: Vec<(i32, i32, Obj)>,
    #[serde(default)]
    pub rugs: Vec<Rug>,
    #[serde(default)]
    pub art: Vec<(i32, u8)>,
    #[serde(default)]
    pub paper: u8,
    #[serde(default)]
    pub floor: u8,
}

/// Is a tile inside the house, where furniture can stand?
pub fn indoors(x: i32, z: i32) -> bool {
    (1..HOUSE_W - 1).contains(&x) && (1..HOUSE_H - 1).contains(&z)
}

/// Tiles kept clear so you can always get out of the door.
fn doorway(x: i32, z: i32) -> bool {
    x == HOUSE_DOOR.0 && z >= HOUSE_H - 2
}

/// The tiles a piece covers when anchored at (x, z) and turned `rot` quarter turns.
pub fn footprint(f: Furn, rot: u8, x: i32, z: i32) -> Vec<(i32, i32)> {
    let (w, d) = turned(f, rot);
    (0..d)
        .flat_map(|dz| (0..w).map(move |dx| (x + dx, z + dz)))
        .collect()
}

/// Width and depth after turning.
pub fn turned(f: Furn, rot: u8) -> (i32, i32) {
    let (w, d) = f.def().size;
    if rot % 2 == 1 { (d, w) } else { (w, d) }
}

/// Where a piece's model stands.
pub fn center(f: Furn, rot: u8, x: i32, z: i32) -> Vec3 {
    let (w, d) = turned(f, rot);
    Vec3::new(x as f32 + w as f32 * 0.5, 0.0, z as f32 + d as f32 * 0.5)
}

/// The turn that faces a piece's front towards a spot.
pub fn facing(from: Vec3, to: Vec2) -> u8 {
    let d = to - Vec2::new(from.x, from.z);
    if d.y.abs() >= d.x.abs() {
        if d.y >= 0.0 { 0 } else { 2 }
    } else if d.x >= 0.0 {
        1
    } else {
        3
    }
}

/// Puts a piece of furniture down (anchor plus parts). The caller checks it fits.
pub fn put(w: &mut World, f: Furn, rot: u8, x: i32, z: i32) {
    for (tx, tz) in footprint(f, rot, x, z) {
        let o = if (tx, tz) == (x, z) {
            Obj::Furniture {
                f,
                rot,
                fish: Vec::new(),
            }
        } else {
            Obj::Part {
                ax: x as i16,
                az: z as i16,
            }
        };
        w.set_obj(tx, tz, Some(o));
    }
}

/// The empty shell of the house: walls, windows and floor.
fn shell(w: &mut World, paper: u8, floor: u8) {
    let (fl, style) = FLOORS[floor as usize % FLOORS.len()];
    w.style = style;
    for z in 0..HOUSE_H {
        for x in 0..HOUSE_W {
            let wall = z == 0 || ((x == 0 || x == HOUSE_W - 1) && z < HOUSE_H - 1);
            let inside = z < HOUSE_H - 1 || (x, z) == HOUSE_DOOR;
            if !inside {
                w.set_floor(x, z, Floor::Void);
                w.set_wall(x, z, Wall::None);
                continue;
            }
            w.set_floor(x, z, fl);
            w.set_wall(x, z, if wall { Wall::Paper(paper) } else { Wall::None });
        }
    }
}

impl House {
    /// A new farmhouse: a bed, a little kitchen, a table for tea and a rug. Plenty of room
    /// (and charm) left to add.
    pub fn new() -> House {
        let mut world = World::new(HOUSE_W, HOUSE_H, Area::Home, 0);
        shell(&mut world, HOME_PAPER, HOME_FLOOR);
        put(&mut world, Furn::Bed, 0, 2, 1);
        put(&mut world, Furn::Stove, 0, 10, 1);
        put(&mut world, Furn::Counter, 0, 11, 1);
        put(&mut world, Furn::RoundTable, 0, 7, 5);
        put(&mut world, Furn::Chair, 1, 6, 5);
        put(&mut world, Furn::Chair, 3, 8, 5);
        world.set_obj(
            13,
            1,
            Some(Obj::Chest {
                items: vec![None; 30],
            }),
        );
        House {
            world,
            rugs: vec![Rug {
                x: 6,
                z: 4,
                kind: 0,
            }],
            art: Vec::new(),
            paper: HOME_PAPER,
            floor: HOME_FLOOR,
            turn: 0,
        }
    }

    pub fn save(&self) -> HouseSave {
        let mut objs = Vec::new();
        for z in 0..self.world.h {
            for x in 0..self.world.w {
                if let Some(o) = self.world.obj(x, z) {
                    objs.push((x, z, o.clone()));
                }
            }
        }
        HouseSave {
            objs,
            rugs: self.rugs.clone(),
            art: self.art.clone(),
            paper: self.paper,
            floor: self.floor,
        }
    }

    pub fn load(s: &HouseSave) -> House {
        let mut h = House::new();
        if s.objs.is_empty() {
            return h;
        }
        h.paper = s.paper;
        h.floor = s.floor % FLOORS.len() as u8;
        shell(&mut h.world, h.paper, h.floor);
        for o in h.world.objs.iter_mut() {
            *o = None;
        }
        for (x, z, o) in &s.objs {
            if indoors(*x, *z) {
                h.world.set_obj(*x, *z, Some(o.clone()));
            }
        }
        h.rugs = s.rugs.clone();
        h.art = s.art.clone();
        for o in h.world.objs.iter_mut().flatten() {
            if let Obj::Chest { items } = o {
                for st in items.iter_mut().flatten() {
                    st.normalize();
                }
            }
        }
        h.world.touch_all();
        h
    }

    /// New wallpaper or a new floor.
    pub fn redecorate(&mut self, paper: u8, floor: u8) {
        self.paper = paper;
        self.floor = floor % FLOORS.len() as u8;
        shell(&mut self.world, self.paper, self.floor);
        self.world.touch_all();
    }

    /// Every placed piece: (anchor x, z, kind, turn, fish inside).
    pub fn pieces(&self) -> Vec<(i32, i32, Furn, u8, &Vec<Item>)> {
        let mut v = Vec::new();
        for z in 0..self.world.h {
            for x in 0..self.world.w {
                if let Some(Obj::Furniture { f, rot, fish }) = self.world.obj(x, z) {
                    v.push((x, z, *f, *rot, fish));
                }
            }
        }
        v
    }

    /// How charming the house is. Every piece counts, the first two of a kind in full and
    /// the rest at half, so a varied room beats fifty chairs.
    pub fn charisma(&self) -> u32 {
        let mut half = 0u32;
        let mut seen: HashMap<Furn, u32> = HashMap::new();
        let count = |key: u32, n: &mut HashMap<u32, u32>, value: u32| -> u32 {
            let k = n.entry(key).or_insert(0);
            *k += 1;
            if *k <= 2 { value * 2 } else { value }
        };
        let mut other: HashMap<u32, u32> = HashMap::new();
        for o in self.world.objs.iter().flatten() {
            match o {
                Obj::Furniture { f, fish, .. } => {
                    let k = seen.entry(*f).or_insert(0);
                    *k += 1;
                    let v = f.def().charm;
                    half += if *k <= 2 { v * 2 } else { v };
                    // Every fish charms, and every different kind a little more.
                    let mut kinds = fish.clone();
                    kinds.sort();
                    kinds.dedup();
                    half += f.per_fish() * 2 * fish.len() as u32 + kinds.len() as u32 * 2;
                }
                Obj::Lamp => half += count(100, &mut other, 3),
                Obj::FlowerPot { .. } => half += count(101, &mut other, 2),
                Obj::Bench => half += count(102, &mut other, 1),
                Obj::EnchantTable => half += count(103, &mut other, 4),
                _ => {}
            }
        }
        for r in &self.rugs {
            half += count(200 + r.kind as u32, &mut other, RUGS[r.kind as usize % 4].0);
        }
        for (_, k) in &self.art {
            half += count(300 + *k as u32, &mut other, ART[*k as usize % 4]);
        }
        if self.paper != HOME_PAPER {
            half += 6;
        }
        if self.floor != HOME_FLOOR {
            half += 6;
        }
        half / 2
    }

    /// Fish swimming in every tank.
    pub fn fish_kept(&self) -> usize {
        self.pieces().iter().map(|p| p.4.len()).sum()
    }

    /// Pieces of furniture standing in the house.
    pub fn furnished(&self) -> usize {
        self.pieces().len()
    }
}

impl Default for House {
    fn default() -> Self {
        Self::new()
    }
}

/// A title for how charming a house is.
pub fn charm_title(c: u32) -> &'static str {
    match c {
        0..=9 => "Plain",
        10..=24 => "Cozy",
        25..=44 => "Charming",
        45..=69 => "Delightful",
        _ => "Dazzling",
    }
}

/// A dish on the stove.
pub struct Cooking {
    /// Index into `RECIPES`.
    pub recipe: usize,
    pub t: f32,
    /// The stove it's cooking on.
    pub at: Vec3,
    /// Where the crafting book was: category, row and scroll.
    pub book: (usize, usize, usize),
    /// On a copper range: a chance of a second helping.
    pub fancy: bool,
    pub sizzle: f32,
}

/// Seconds a dish takes on the stove.
pub const COOK_TIME: f32 = 1.4;

/// The crafting book's page for kitchen recipes.
pub fn kitchen_page() -> usize {
    1 + CATS.iter().position(|c| *c == Cat::Kitchen).unwrap_or(0)
}

impl Play {
    /// How charming your home is.
    pub fn charisma(&self) -> u32 {
        self.house.charisma()
    }

    /// What a shopkeeper asks of a charming customer: up to a fifth off.
    pub fn buy_price(&self, base: u64) -> u64 {
        let off = self.charisma().min(60) as u64 / 3;
        (base * (100 - off)).div_ceil(100).max(1)
    }

    /// Extra a shopkeeper pays a charming customer, in percent (up to +10%).
    pub fn sell_bonus(&self) -> u64 {
        100 + self.charisma().min(60) as u64 / 6
    }

    /// Friendship grows faster for someone with a lovely home (up to +60%).
    pub fn charm_friend(&self, n: i32) -> i32 {
        if n <= 0 {
            n
        } else {
            n + n * self.charisma().min(60) as i32 / 100
        }
    }

    /// Goes indoors (after the fade).
    pub fn enter_house(&mut self) {
        let (dx, dz) = HOUSE_DOOR;
        self.drop_rod();
        self.player.pos = Vec2::new(dx as f32 + 0.5, dz as f32 - 0.45);
        self.player.facing = Vec2::new(0.0, -1.0);
        self.player.act = None;
        self.area = Area::Home;
        self.drops.clear();
        self.cam_pos = self.room_center();
        let c = self.charisma();
        self.banner = Some(Banner {
            title: "Home".into(),
            sub: format!("Charisma {c} - {}", charm_title(c)),
            t: 0.8,
        });
    }

    /// Back out onto the farm, on the doorstep.
    pub fn leave_house(&mut self) {
        let (dx, dz) = super::farm::MARKS.door;
        self.player.pos = Vec2::new(dx as f32 + 0.5, dz as f32 + 0.6);
        self.player.facing = Vec2::new(0.0, 1.0);
        self.player.act = None;
        self.cooking = None;
        self.area = Area::Farm;
        self.drops.clear();
        self.cam_pos = self.player.world_pos();
    }

    /// Where you wake up: beside your bed.
    pub fn bedside(&self) -> Vec2 {
        let w = &self.house.world;
        for (x, z, f, rot, _) in self.house.pieces() {
            if f.def().use_ == Use::Sleep {
                let c = center(f, rot, x, z);
                let (tx, tz) = w.nearest_open(c.x as i32, c.z as i32 + 1);
                return Vec2::new(tx as f32 + 0.5, tz as f32 + 0.5);
            }
        }
        let (dx, dz) = HOUSE_DOOR;
        Vec2::new(dx as f32 + 0.5, dz as f32 - 0.45)
    }

    /// Is there a stove (or a campfire) close enough to cook on? Returns where it is, and
    /// whether it's a fancy one.
    pub fn stove_near(&self) -> Option<(Vec3, bool)> {
        let p = self.player.pos;
        let w = self.world();
        let (px, pz) = self.player.tile();
        for z in pz - 2..=pz + 2 {
            for x in px - 2..=px + 2 {
                let (ax, az) = w.anchor(x, z);
                let found = match w.obj(ax, az) {
                    Some(Obj::Furniture { f, rot, .. }) if f.def().use_ == Use::Cook => {
                        Some((center(*f, *rot, ax, az), *f == Furn::Range))
                    }
                    Some(Obj::Campfire) => Some((tile_center(ax, az), false)),
                    _ => None,
                };
                if let Some((c, fancy)) = found {
                    // Close to any tile it stands on.
                    let near =
                        (x as f32 + 0.5 - p.x).abs() < 1.6 && (z as f32 + 0.5 - p.y).abs() < 1.6;
                    if near {
                        return Some((c, fancy));
                    }
                }
            }
        }
        None
    }

    /// Starts a dish cooking. The ingredients go in when it's done.
    pub fn start_cooking(
        &mut self,
        recipe: usize,
        book: (usize, usize, usize),
        io: &mut Io,
    ) -> bool {
        let Some((at, fancy)) = self.stove_near() else {
            self.toast("Cook at your stove at home (or a campfire).", None, 0);
            io.audio.play(Sfx::Denied);
            return false;
        };
        let r = &RECIPES[recipe];
        if !r.can_craft(&self.player.inv) {
            io.audio.play(Sfx::Denied);
            return false;
        }
        let d = Vec2::new(at.x, at.z) - self.player.pos;
        if d.length_squared() > 0.01 {
            self.player.facing = d.normalize();
        }
        self.cooking = Some(Cooking {
            recipe,
            t: 0.0,
            at,
            book,
            fancy,
            sizzle: 0.0,
        });
        io.audio.play_at(Sfx::Sizzle, 0.8, 1.0);
        true
    }

    /// The pan sizzles, steam rises, and out comes the dish.
    pub fn update_cooking(&mut self, io: &mut Io) {
        let dt = io.dt;
        let Some(c) = &mut self.cooking else { return };
        c.t += dt;
        c.sizzle -= dt;
        let at = c.at;
        if c.sizzle <= 0.0 {
            c.sizzle = 0.45;
            io.audio
                .play_at(Sfx::Sizzle, 0.45, 0.9 + self.rng.f32() * 0.25);
        }
        if (self.time * 12.0).fract() < dt * 12.0 {
            self.fx
                .motes(at + Vec3::Y * 0.85, 2, &[WHITE, SAND, CREAM], 0.25);
        }
        let Some(c) = &self.cooking else { return };
        if c.t < COOK_TIME {
            return;
        }
        let (recipe, book, fancy) = (c.recipe, c.book, c.fancy);
        self.cooking = None;
        let r = &RECIPES[recipe];
        if !r.can_craft(&self.player.inv) {
            io.audio.play(Sfx::Denied);
            return;
        }
        for (item, k) in r.needs {
            self.player.inv.take(*item, *k as u32);
        }
        let mut n = r.n;
        let second = fancy && self.rng.chance(0.25);
        if second {
            n += r.n;
        }
        let made = Stack::new(r.out, n);
        let left = self.player.inv.add_stack(made);
        if left > 0 {
            self.give_ground(Stack { n: left, ..made });
        }
        self.on_craft(r.out);
        self.on_cook(n);
        io.audio.play(Sfx::Craft);
        self.fx
            .burst(at + Vec3::Y * 0.8, 10, &[WHITE, CREAM, GOLD], 1.4, 1.0);
        self.fx.popup(at + Vec3::Y * 1.2, "Yum!", GOLD);
        if second {
            self.toast_colored(
                format!("Cooked {} - a second helping!", r.out.def().name),
                Some(r.out),
                n as u32,
                GOLD,
            );
        } else {
            self.toast(
                format!("Cooked {}", r.out.def().name),
                Some(r.out),
                n as u32,
            );
        }
        let (cat, row, scroll) = book;
        self.menu = Menu::Inventory {
            tab: Tab::Craft,
            cursor: 0,
            recipe: row,
            scroll,
            cat,
        };
    }

    /// Drops something at your feet.
    fn give_ground(&mut self, s: Stack) {
        let at = self.player.world_pos();
        self.drops.push(super::fx::Drop::new(s, at, &mut self.rng));
    }

    // --------------------------------------------------------------------------------------
    // Decorating
    // --------------------------------------------------------------------------------------

    /// How a piece aimed at tile (x, z) would go down: its turn (facing you, plus any turns
    /// from T) and its anchor. Big pieces grow away from you, never into you.
    pub fn placement(&self, f: Furn, x: i32, z: i32) -> (u8, i32, i32) {
        let c = Vec3::new(x as f32 + 0.5, 0.0, z as f32 + 0.5);
        let rot = (facing(c, self.player.pos) + self.house.turn) % 4;
        let (w, d) = turned(f, rot);
        let p = self.player.pos;
        let ax = if p.x > x as f32 + 1.0 { x - (w - 1) } else { x };
        let az = if p.y > z as f32 + 1.0 { z - (d - 1) } else { z };
        (rot, ax, az)
    }

    /// Where a rug aimed at tile (x, z) goes: it unrolls away from you.
    pub fn rug_anchor(&self, x: i32, z: i32) -> (i32, i32) {
        let p = self.player.pos;
        let rx = if p.x > x as f32 + 1.0 { x - 1 } else { x };
        let rz = if p.y > z as f32 + 1.0 { z - 1 } else { z };
        (rx, rz)
    }

    /// Can a piece go down here?
    pub fn furniture_fits(&self, f: Furn, rot: u8, x: i32, z: i32) -> bool {
        let w = &self.house.world;
        footprint(f, rot, x, z).into_iter().all(|(tx, tz)| {
            let c = Vec2::new(tx as f32 + 0.5, tz as f32 + 0.5);
            let d = (self.player.pos - c).abs();
            indoors(tx, tz)
                && !doorway(tx, tz)
                && w.wall(tx, tz) == Wall::None
                && w.obj(tx, tz).is_none()
                && !(d.x < 0.5 + RADIUS && d.y < 0.5 + RADIUS)
        })
    }

    /// Can a rug go down with its corner here?
    pub fn rug_fits(&self, x: i32, z: i32) -> bool {
        [(0, 0), (1, 0), (0, 1), (1, 1)]
            .iter()
            .all(|(dx, dz)| indoors(x + dx, z + dz) && !doorway(x + dx, z + dz))
            && !self
                .house
                .rugs
                .iter()
                .any(|r| (r.x - x).abs() <= 1 && (r.z - z).abs() <= 1)
    }

    /// The column a picture would hang in, aiming at a tile.
    pub fn art_spot(&self, x: i32, z: i32) -> Option<i32> {
        (z <= 1 && (1..HOUSE_W - 1).contains(&x) && !WINDOWS.contains(&x)).then_some(x)
    }

    /// Can anything placeable in the house go on this tile?
    pub fn house_target_ok(&self, pl: Placeable, x: i32, z: i32) -> bool {
        match pl {
            Placeable::Furniture(f) => {
                let (rot, ax, az) = self.placement(f, x, z);
                self.furniture_fits(f, rot, ax, az)
            }
            Placeable::Rug(_) => {
                let (rx, rz) = self.rug_anchor(x, z);
                self.rug_fits(rx, rz)
            }
            Placeable::WallArt(_) => self
                .art_spot(x, z)
                .is_some_and(|c| !self.house.art.iter().any(|(ax, _)| *ax == c)),
            Placeable::Chest
            | Placeable::Lamp
            | Placeable::FlowerPot
            | Placeable::Bench
            | Placeable::Torch
            | Placeable::EnchantTable
            | Placeable::Workbench => {
                indoors(x, z)
                    && !doorway(x, z)
                    && self.house.world.obj(x, z).is_none()
                    && self.house.world.wall(x, z) == Wall::None
                    && {
                        let c = Vec2::new(x as f32 + 0.5, z as f32 + 0.5);
                        let d = (self.player.pos - c).abs();
                        !(d.x < 0.5 + RADIUS && d.y < 0.5 + RADIUS)
                    }
            }
            _ => false,
        }
    }

    /// Is there something to do at this tile with what's in hand? (For the target cursor.)
    pub fn house_act_ok(&self, x: i32, z: i32) -> bool {
        match self.player.held().map(|i| i.def().kind) {
            Some(Kind::Place(pl)) => self.house_target_ok(pl, x, z),
            Some(Kind::Wallpaper(_) | Kind::Flooring(_)) => true,
            _ => {
                let (ax, az) = self.house.world.anchor(x, z);
                matches!(
                    self.house.world.obj(ax, az),
                    Some(
                        Obj::Furniture { .. }
                            | Obj::Chest { .. }
                            | Obj::Lamp
                            | Obj::FlowerPot { .. }
                            | Obj::Bench
                            | Obj::Torch
                            | Obj::Workbench
                            | Obj::EnchantTable
                    )
                ) || self.house.art.iter().any(|(c, _)| *c == x && z <= 1)
                    || self.house.rugs.iter().any(|r| r.covers(x, z))
            }
        }
    }

    /// Using the item in hand indoors: place it, or (if `pick`) pick up whatever you aim at.
    pub fn house_use(&mut self, (x, z): (i32, i32), pick: bool, io: &mut Io) {
        let sel = self.player.sel;
        let held = self.player.held();
        match held.map(|i| i.def().kind) {
            Some(Kind::Place(pl)) if self.house_target_ok(pl, x, z) => {
                self.player.inv.take_one(sel);
                self.place_in_house(pl, x, z, io);
                return;
            }
            Some(Kind::Wallpaper(style)) => {
                self.player.inv.take_one(sel);
                self.repaper(style, io);
                return;
            }
            Some(Kind::Flooring(k)) => {
                self.player.inv.take_one(sel);
                self.refloor(k, io);
                return;
            }
            _ => {}
        }
        if pick && self.pick_up_decor(x, z, io) {
            return;
        }
        match held.map(|i| i.def().kind) {
            Some(Kind::Place(Placeable::Egg)) => {
                if self.nag <= 0.0 {
                    self.nag = 2.5;
                    self.toast("The egg wants fresh air. Set it down outside.", None, 0);
                }
                io.audio.play(Sfx::Denied);
            }
            Some(Kind::Place(_)) => {
                io.audio.play(Sfx::Denied);
            }
            Some(Kind::Gear(b)) if !b.class.is_armor() => {
                if self.nag <= 0.0 {
                    self.nag = 2.5;
                    let t = if b.class == super::gear::Class::Rod {
                        "No fish in here! Try the pond."
                    } else {
                        "Not in the house!"
                    };
                    self.toast(t, None, 0);
                    io.audio.play(Sfx::Denied);
                }
            }
            Some(_) => {
                if let Some(item) = held {
                    self.use_item_indoors(item, (x, z), io);
                }
            }
            None => {}
        }
    }

    fn place_in_house(&mut self, pl: Placeable, x: i32, z: i32, io: &mut Io) {
        match pl {
            Placeable::Furniture(f) => {
                let (rot, ax, az) = self.placement(f, x, z);
                put(&mut self.house.world, f, rot, ax, az);
                let c = center(f, rot, ax, az);
                self.fx
                    .burst(c + Vec3::Y * 0.2, 10, &[SAND, CREAM, KHAKI], 1.4, 1.2);
                self.fx.motes(c + Vec3::Y * 0.6, 6, &[PINK, WHITE], 0.3);
            }
            Placeable::Rug(k) => {
                let (x, z) = self.rug_anchor(x, z);
                self.house.rugs.push(Rug { x, z, kind: k });
                let at = tile_center(x, z);
                self.fx
                    .burst(at + Vec3::new(0.5, 0.05, 0.5), 8, &[CREAM, SAND], 1.2, 1.0);
            }
            Placeable::WallArt(k) => {
                if let Some(c) = self.art_spot(x, z) {
                    self.house.art.push((c, k));
                    self.fx.motes(
                        Vec3::new(c as f32 + 0.5, 1.2, 1.1),
                        8,
                        &[GOLD, CREAM, WHITE],
                        0.3,
                    );
                }
            }
            Placeable::Chest => self.house.world.set_obj(
                x,
                z,
                Some(Obj::Chest {
                    items: vec![None; 30],
                }),
            ),
            Placeable::Lamp => self.house.world.set_obj(x, z, Some(Obj::Lamp)),
            Placeable::FlowerPot => self
                .house
                .world
                .set_obj(x, z, Some(Obj::FlowerPot { var: 0 })),
            Placeable::Bench => self.house.world.set_obj(x, z, Some(Obj::Bench)),
            Placeable::Torch => self.house.world.set_obj(x, z, Some(Obj::Torch)),
            Placeable::Workbench => self.house.world.set_obj(x, z, Some(Obj::Workbench)),
            Placeable::EnchantTable => self.house.world.set_obj(x, z, Some(Obj::EnchantTable)),
            _ => {}
        }
        io.audio.play(Sfx::Place);
        self.charm_toast();
    }

    /// A little note when the house gets more charming.
    fn charm_toast(&mut self) {
        let c = self.charisma();
        self.toast_colored(format!("Charisma {c} - {}", charm_title(c)), None, 0, PINK);
    }

    /// Picks up a piece of furniture, a rug or a picture. Returns true if something came up.
    fn pick_up_decor(&mut self, x: i32, z: i32, io: &mut Io) -> bool {
        let w = &self.house.world;
        let (ax, az) = w.anchor(x, z);
        let obj = w.obj(ax, az).cloned();
        let placed_whole = matches!(obj, Some(Obj::Furniture { .. }) | Some(Obj::Chest { .. }));
        let item = match obj {
            Some(Obj::Furniture { f, rot, fish }) => {
                if f.def().use_ == Use::Sleep
                    && self
                        .house
                        .pieces()
                        .iter()
                        .filter(|p| p.2.def().use_ == Use::Sleep)
                        .count()
                        <= 1
                {
                    self.toast("You'll want somewhere to sleep!", None, 0);
                    io.audio.play(Sfx::Denied);
                    return true;
                }
                for (tx, tz) in footprint(f, rot, ax, az) {
                    self.house.world.set_obj(tx, tz, None);
                }
                for fsh in fish {
                    self.give(Stack::new(fsh, 1));
                }
                Some(f.item())
            }
            Some(Obj::Chest { items }) => {
                if items.iter().any(|s| s.is_some()) {
                    self.toast("Empty the chest before you move it.", None, 0);
                    io.audio.play(Sfx::Denied);
                    return true;
                }
                self.house.world.set_obj(ax, az, None);
                Some(Item::Chest)
            }
            Some(Obj::Lamp) => Some(Item::Lamp),
            Some(Obj::FlowerPot { .. }) => Some(Item::FlowerPot),
            Some(Obj::Bench) => Some(Item::Bench),
            Some(Obj::Torch) => Some(Item::Torch),
            Some(Obj::Workbench) => Some(Item::Workbench),
            Some(Obj::EnchantTable) => Some(Item::EnchantTable),
            _ => None,
        };
        let item = match item {
            Some(i) => {
                if !placed_whole {
                    self.house.world.set_obj(ax, az, None);
                }
                Some(i)
            }
            None => {
                // A picture on the wall, or a rug underfoot.
                if let Some(k) = self.house.art.iter().position(|(c, _)| *c == x && z <= 1) {
                    let (_, kind) = self.house.art.remove(k);
                    art_item(kind)
                } else if let Some(k) = self.house.rugs.iter().rposition(|r| r.covers(x, z)) {
                    let r = self.house.rugs.remove(k);
                    rug_item(r.kind)
                } else {
                    None
                }
            }
        };
        let Some(item) = item else { return false };
        self.give(Stack::new(item, 1));
        io.audio.play(Sfx::Place);
        self.fx.burst(
            tile_center(ax, az) + Vec3::Y * 0.3,
            6,
            &[SAND, KHAKI],
            1.4,
            1.2,
        );
        true
    }

    fn repaper(&mut self, style: u8, io: &mut Io) {
        let old = self.house.paper;
        self.house.redecorate(style, self.house.floor);
        if let Some(i) = wallpaper_item(old) {
            self.give(Stack::new(i, 1));
        }
        io.audio.play(Sfx::Craft);
        self.fx.motes(
            self.player.world_pos() + Vec3::Y * 1.0,
            14,
            &[CREAM, PINK, WHITE],
            0.8,
        );
        self.charm_toast();
    }

    fn refloor(&mut self, k: u8, io: &mut Io) {
        let old = self.house.floor;
        self.house.redecorate(self.house.paper, k);
        if let Some(i) = flooring_item(old) {
            self.give(Stack::new(i, 1));
        }
        io.audio.play(Sfx::Craft);
        self.fx.motes(
            self.player.world_pos() + Vec3::Y * 0.2,
            14,
            &[SAND, CREAM, WHITE],
            0.8,
        );
        self.charm_toast();
    }

    /// Eating and drinking still work indoors.
    fn use_item_indoors(&mut self, item: Item, tile: (i32, i32), io: &mut Io) {
        match item.def().kind {
            Kind::Place(_) => io.audio.play(Sfx::Denied),
            _ => self.use_item(item, tile, io),
        }
    }

    /// Using a piece of furniture (E on it).
    pub fn use_furniture(&mut self, ax: i32, az: i32, io: &mut Io) -> bool {
        let Some(Obj::Furniture { f, rot, .. }) = self.house.world.obj(ax, az).cloned() else {
            return false;
        };
        let c = center(f, rot, ax, az);
        match f.def().use_ {
            Use::Sleep => {
                self.menu = Menu::dialog_choice(
                    "Your cozy bed is waiting. Go to sleep and end the day?",
                    vec![
                        ("Sleep", super::menus::Choice::Sleep),
                        ("Not yet", super::menus::Choice::Close),
                    ],
                );
                io.audio.play(Sfx::UiSelect);
            }
            Use::Cook => {
                self.menu = Menu::Inventory {
                    tab: Tab::Craft,
                    cursor: 0,
                    recipe: 0,
                    scroll: 0,
                    cat: kitchen_page(),
                };
                io.audio.play(Sfx::UiSelect);
            }
            Use::Sit => {
                self.fx.motes(c + Vec3::Y * 0.6, 4, &[PINK, WHITE], 0.2);
                self.toast("Ahh, a nice sit down.", None, 0);
                self.player.energy =
                    (self.player.energy + 5.0).min(self.player.max_energy() as f32);
            }
            Use::Tank => {
                self.menu = Menu::Tank {
                    x: ax,
                    z: az,
                    cursor: 0,
                };
                io.audio.play(Sfx::UiSelect);
            }
            Use::Play => {
                io.audio.play(Sfx::Piano);
                for k in 0..4 {
                    self.fx.popup(
                        c + Vec3::new(k as f32 * 0.3 - 0.45, 1.2 + k as f32 * 0.1, 0.0),
                        "♪",
                        [PINK, SKY, GOLD, LIME][k],
                    );
                }
                self.toast("You play a little tune. Lovely!", None, 0);
            }
            Use::Clock => {
                io.audio.play_at(Sfx::Bell, 0.35, 1.5);
                let t = self.clock.label();
                self.toast(format!("It's {t}. Tick, tock."), None, 0);
            }
            Use::Spin => {
                self.house_spin = 2.5;
                io.audio.play_at(Sfx::Swing, 0.5, 0.7);
                self.toast("Round and round the world goes!", None, 0);
            }
            Use::Stars => self.stargaze(c, io),
            Use::Squeak => {
                io.audio.play_at(Sfx::Pickup, 0.8, 1.6);
                self.fx.popup(c + Vec3::Y * 0.7, "♥", PINK);
                self.fx.motes(c + Vec3::Y * 0.5, 5, &[PINK, BLUSH], 0.2);
            }
            Use::Read => {
                let lines = [
                    "\"The Golden Carp only bites for the patient.\" Useful!",
                    "A cookbook: \"Any fish tastes better with garlic.\"",
                    "\"Rain brings the salmon upstream.\" Noted.",
                    "\"Some fish only come out at night.\" Spooky.",
                    "A book of fairy tales. You read one about a very small dragon.",
                    "\"Lava fish need a heat-proof rod.\" Who knew?",
                ];
                let k = (self.time * 7.0) as usize % lines.len();
                self.toast(lines[k], None, 0);
            }
            Use::Warm => {
                if self.warmed != self.clock.day {
                    self.warmed = self.clock.day;
                    self.player.add_buff(
                        super::items::Buff {
                            stat: super::gear::Stat::Regen,
                            val: 3,
                            secs: 300,
                        },
                        Item::Fireplace,
                    );
                    self.player.refresh();
                    self.toast_colored("Toasty! HP Regen +3 for a while.", None, 0, ORANGE);
                    self.fx
                        .motes(c + Vec3::Y * 0.5, 10, &[GOLD, ORANGE, CREAM], 0.4);
                } else {
                    self.toast("Warm and toasty.", None, 0);
                }
            }
            Use::Tidy => {
                let t = match f {
                    Furn::Icebox => "Milk, cheese and one very old turnip.",
                    Furn::Wardrobe => "Your clothes, neatly folded.",
                    _ => "Socks, and a few secrets.",
                };
                self.toast(t, None, 0);
            }
            Use::Nothing => return false,
        }
        true
    }

    /// Looking through the telescope: at night, a shooting star brings luck once a day.
    fn stargaze(&mut self, c: Vec3, io: &mut Io) {
        if self.sky_night() < 0.5 {
            self.toast("Just clouds and birds. Try again at night.", None, 0);
            return;
        }
        if self.stargazed == self.clock.day {
            self.toast("The stars twinkle back at you.", None, 0);
            return;
        }
        self.stargazed = self.clock.day;
        self.player.add_buff(
            super::items::Buff {
                stat: super::gear::Stat::Luck,
                val: 6,
                secs: 600,
            },
            Item::Telescope,
        );
        self.player.refresh();
        io.audio.play(Sfx::Rare);
        self.fx
            .motes(c + Vec3::Y * 1.3, 12, &[WHITE, CREAM, LAVENDER], 0.5);
        self.toast_colored("A shooting star! Luck +6 tonight.", None, 0, LAVENDER);
    }

    /// Puts a fish from the bag into a tank. Returns true if it went in.
    pub fn tank_add(&mut self, x: i32, z: i32, slot: usize, io: &mut Io) -> bool {
        let Some(s) = self.player.inv.slots.get(slot).copied().flatten() else {
            return false;
        };
        if s.item.def().kind != Kind::Fish {
            self.toast("Only fish can live in there!", None, 0);
            io.audio.play(Sfx::Denied);
            return false;
        }
        let Some(Obj::Furniture { f, fish, .. }) = self.house.world.obj_mut(x, z) else {
            return false;
        };
        if fish.len() >= f.def().cap {
            io.audio.play(Sfx::Denied);
            return false;
        }
        fish.push(s.item);
        self.player.inv.take_one(slot);
        io.audio.play(Sfx::Water);
        self.fx.burst(
            tile_center(x, z) + Vec3::Y * 0.7,
            8,
            &[WHITE, SKY, AQUA],
            1.0,
            1.0,
        );
        true
    }

    /// Takes a fish back out of a tank.
    pub fn tank_take(&mut self, x: i32, z: i32, k: usize, io: &mut Io) -> bool {
        let Some(Obj::Furniture { fish, .. }) = self.house.world.obj_mut(x, z) else {
            return false;
        };
        if k >= fish.len() {
            return false;
        }
        let item = fish[k];
        if !self.player.inv.can_fit(item, 1) {
            io.audio.play(Sfx::Denied);
            return false;
        }
        if let Some(Obj::Furniture { fish, .. }) = self.house.world.obj_mut(x, z) {
            fish.remove(k);
        }
        self.player.inv.add(item, 1);
        io.audio.play(Sfx::Pickup);
        true
    }

    /// The farmhouse door, from outside: walking up to it goes in.
    pub fn house_door_ahead(&self) -> bool {
        self.area == Area::Farm && self.player.tile() == super::farm::MARKS.door
    }

    /// Starts going indoors.
    pub fn go_indoors(&mut self, io: &mut Io) {
        io.audio.play(Sfx::Door);
        self.start_fade(Trans::House { enter: true });
    }
}

/// The row of a tank's fish in its menu.
fn tank_grid(l: &super::menus::Layout, cap: usize) -> Grid {
    Grid {
        x: l.px + (l.pw - cap as i32 * super::menus::CELL) / 2,
        y: l.py + 36,
        cols: cap.max(1),
        rows: 1,
        gap: 0,
    }
}

impl Play {
    /// What a shop pays for something, with your charm on top.
    pub fn sell_rate(&self, at: Option<Place>, s: &Stack) -> u64 {
        super::shops::buy_rate(at, s) * self.sell_bonus() / 100
    }

    /// The fish tank menu: click fish in the bag to put them in, click them in the tank to
    /// take them out.
    pub fn update_tank(&mut self, io: &mut Io, x: i32, z: i32, mut cursor: usize) -> Menu {
        let input = io.input;
        let (w, h) = (io.view.0 as i32, io.view.1 as i32);
        let l = panel_layout(w, h);
        let Some(Obj::Furniture { f, .. }) = self.house.world.obj(x, z) else {
            return Menu::None;
        };
        let cap = f.def().cap;
        if input.pressed(Action::Cancel) || input.pressed(Action::Inventory) {
            self.close_menu(io);
            return Menu::None;
        }
        let total = cap + 40;
        let before = cursor;
        if input.pressed_repeat(Action::Right) {
            cursor = (cursor + 1) % total;
        }
        if input.pressed_repeat(Action::Left) {
            cursor = (cursor + total - 1) % total;
        }
        if input.pressed_repeat(Action::Down) {
            cursor = if cursor < cap {
                cap
            } else {
                cap + (cursor - cap + 10) % 40
            };
        }
        if input.pressed_repeat(Action::Up) {
            cursor = if cursor < cap {
                cursor
            } else if cursor - cap < 10 {
                0
            } else {
                cursor - 10
            };
        }
        if cursor != before {
            io.audio.play_at(Sfx::UiMove, 0.5, 1.0);
        }
        let tg = tank_grid(&l, cap);
        let bg = bag_grid(&l, 78);
        let lclick = input.button_pressed(Button::Left) || input.button_pressed(Button::Right);
        if let Some(i) = tg.hit(input.mouse) {
            if input.mouse_moved {
                cursor = i;
            }
            if lclick {
                self.tank_take(x, z, i, io);
            }
        } else if let Some(i) = bg.hit(input.mouse) {
            if input.mouse_moved {
                cursor = cap + i;
            }
            if lclick {
                self.tank_add(x, z, i, io);
            }
        }
        if input.pressed(Action::Confirm) {
            if cursor < cap {
                self.tank_take(x, z, cursor, io);
            } else {
                self.tank_add(x, z, cursor - cap, io);
            }
        }
        Menu::Tank { x, z, cursor }
    }

    pub fn draw_tank(
        &self,
        c: &mut Canvas,
        a: &Assets,
        x: i32,
        z: i32,
        cursor: usize,
        mouse: Vec2,
    ) {
        let (w, h) = (c.w(), c.h());
        let l = panel_layout(w, h);
        let Some(Obj::Furniture { f, fish, .. }) = self.house.world.obj(x, z) else {
            return;
        };
        let cap = f.def().cap;
        c.panel(l.px, l.py, l.pw, l.ph, Style::Paper);
        c.text(l.px + 10, l.py + 7, f.name(), RUST);
        let count = format!("{}/{} fish", fish.len(), cap);
        c.text(
            l.px + l.pw - 10 - c.text_width(&count),
            l.py + 7,
            &count,
            TEAL,
        );
        // A strip of water behind the fish.
        let tg = tank_grid(&l, cap);
        c.rect(tg.x - 4, tg.y - 4, tg.w() + 8, 26, DEEP_TEAL);
        c.rect(tg.x - 4, tg.y - 4, tg.w() + 8, 2, AQUA);
        let tank = super::items::Inventory {
            slots: (0..cap)
                .map(|i| fish.get(i).map(|it| Stack::new(*it, 1)))
                .collect(),
        };
        self.draw_grid(c, a, &tg, &tank, (cursor < cap).then_some(cursor));
        c.text_center(
            l.px + l.pw / 2,
            tg.y + 24,
            &format!("Charm +{} per fish, +1 per kind", f.per_fish()),
            SHADOW,
        );
        let bg = bag_grid(&l, 78);
        let tip = if self.pad {
            "Bag - A puts a fish in"
        } else {
            "Bag - click a fish to put it in"
        };
        c.text(bg.x, bg.y - 11, tip, RUST);
        self.draw_grid(
            c,
            a,
            &bg,
            &self.player.inv,
            (cursor >= cap).then(|| cursor - cap),
        );
        // What's under the mouse, or else under the cursor.
        let at = |i: usize| {
            if i < cap {
                tank.slots
                    .get(i)
                    .copied()
                    .flatten()
                    .map(|s| (s, tg.slot_pos(i)))
            } else {
                self.player.inv.slots[i - cap].map(|s| (s, bg.slot_pos(i - cap)))
            }
        };
        let hover = tg
            .hit(mouse)
            .or_else(|| bg.hit(mouse).map(|i| i + cap))
            .map_or_else(|| at(cursor), at);
        if let Some((s, (sx, sy))) = hover {
            self.tip_at(c, a, &l, sx, sy, &s);
        }
    }
}

/// What pressing E on a piece does, in a word or two.
pub fn use_hint(f: Furn) -> &'static str {
    match f.def().use_ {
        Use::Sleep => "Sleep",
        Use::Cook => "Cook",
        Use::Sit => "Sit",
        Use::Tank => "Fish tank",
        Use::Play => "Play a tune",
        Use::Clock => "Check the time",
        Use::Spin => "Spin the globe",
        Use::Stars => "Look at the stars",
        Use::Squeak => "Squeeze",
        Use::Read => "Read",
        Use::Warm => "Warm up",
        Use::Tidy => "Look inside",
        Use::Nothing => "Admire",
    }
}

/// The item for a rug, picture, wallpaper or floor.
pub fn rug_item(k: u8) -> Option<Item> {
    find_item(|kind| kind == Kind::Place(Placeable::Rug(k)))
}

pub fn art_item(k: u8) -> Option<Item> {
    find_item(|kind| kind == Kind::Place(Placeable::WallArt(k)))
}

pub fn wallpaper_item(style: u8) -> Option<Item> {
    find_item(|kind| kind == Kind::Wallpaper(style))
}

pub fn flooring_item(k: u8) -> Option<Item> {
    find_item(|kind| kind == Kind::Flooring(k))
}

fn find_item(f: impl Fn(Kind) -> bool) -> Option<Item> {
    ALL_ITEMS.iter().copied().find(|i| f(i.def().kind))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_piece_has_an_item_and_fits_the_house() {
        for f in FURNS {
            let item = f.item();
            assert_eq!(
                item.def().kind,
                Kind::Place(Placeable::Furniture(f)),
                "{f:?}"
            );
            let (w, d) = f.def().size;
            assert!(w >= 1 && d >= 1 && w <= 2 && d <= 2);
            for rot in 0..4 {
                assert_eq!(footprint(f, rot, 3, 3).len() as i32, w * d);
            }
        }
        for k in 0..RUGS.len() as u8 {
            assert!(rug_item(k).is_some());
        }
        for k in 0..ART.len() as u8 {
            assert!(art_item(k).is_some());
        }
        for k in 0..FLOORS.len() as u8 {
            assert!(flooring_item(k).is_some(), "floor {k}");
        }
        assert!(wallpaper_item(HOME_PAPER).is_some());
    }

    #[test]
    fn a_new_house_is_furnished_and_charming_enough() {
        let h = House::new();
        assert!(h.pieces().iter().any(|p| p.2 == Furn::Bed));
        assert!(h.pieces().iter().any(|p| p.2 == Furn::Stove));
        let c = h.charisma();
        assert!((5..20).contains(&c), "starting charisma {c}");
        // The way out is always clear.
        let (dx, dz) = HOUSE_DOOR;
        assert!(!h.world.blocked(dx, dz));
        assert!(!h.world.blocked(dx, dz - 1));
        // Save and load keeps it all.
        let back = House::load(&h.save());
        assert_eq!(back.charisma(), c);
        assert_eq!(back.pieces().len(), h.pieces().len());
    }

    #[test]
    fn variety_beats_a_pile_of_chairs() {
        let mut a = House::new();
        let mut b = House::new();
        let base = a.charisma();
        for k in 0..6 {
            put(&mut a.world, Furn::Chair, 0, 2 + k, 7);
        }
        for (k, f) in [Furn::Armchair, Furn::Bookshelf, Furn::Clock]
            .into_iter()
            .enumerate()
        {
            put(&mut b.world, f, 0, 2 + k as i32, 7);
        }
        assert!(b.charisma() > a.charisma());
        assert!(a.charisma() > base);
    }
}
