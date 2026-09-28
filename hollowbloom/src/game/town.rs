//! Bramblewick, the town at the end of the bus line: its streets and buildings, the rooms
//! inside the shops, and the projects that bring the old town back to life.

use serde::{Deserialize, Serialize};

use super::items::Item;
use super::world::{Area, Floor, Obj, Wall, World};
use crate::palette::*;
use crate::util::{Rng, hash2};

pub const TOWN_W: i32 = 72;
pub const TOWN_H: i32 = 52;

/// Town projects the mayor's restoration quests finish, as bits.
pub const FOUNTAIN: u32 = 1;
pub const LAMPS: u32 = 2;
pub const GARDENS: u32 = 4;
pub const BRIDGE: u32 = 8;
pub const BUNTING: u32 = 16;
pub const WISH_TREE: u32 = 32;
pub const CLOCK: u32 = 64;
pub const MARKET: u32 = 128;

/// The buildings you can walk into.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, Serialize, Deserialize)]
pub enum Place {
    Hall,
    Guild,
    Scrolls,
    Armory,
    Smithy,
    Tools,
    Jeweler,
    Nook,
    Seeds,
    Bakery,
    Tavern,
    /// The Starfall Spellery: spells and potions.
    Spellery,
}

pub const PLACES: [Place; 12] = [
    Place::Hall,
    Place::Guild,
    Place::Scrolls,
    Place::Armory,
    Place::Smithy,
    Place::Tools,
    Place::Jeweler,
    Place::Nook,
    Place::Seeds,
    Place::Bakery,
    Place::Tavern,
    Place::Spellery,
];

pub struct PlaceDef {
    pub name: &'static str,
    /// What the sign by the door says it is.
    pub trade: &'static str,
    /// Wallpaper style (see `Assets::wallpaper`).
    pub paper: u8,
    pub floor: Floor,
    /// Opening hours, in minutes after midnight.
    pub open: (f32, f32),
    /// What the shelves hold (see `draw_shelf`).
    pub goods: u8,
    /// The room, back wall first. See `room` for the legend.
    pub layout: &'static [&'static str],
}

impl Place {
    pub fn def(self) -> &'static PlaceDef {
        &PLACE_DEFS[self as usize]
    }

    pub fn is_open(self, min: f32) -> bool {
        let (a, b) = self.def().open;
        min >= a && min < b
    }

    /// The building this place is in.
    pub fn building(self) -> usize {
        BUILDINGS
            .iter()
            .position(|b| b.place == Some(self))
            .unwrap_or(0)
    }
}

pub static PLACE_DEFS: [PlaceDef; 12] = [
    PlaceDef {
        name: "Town Hall",
        trade: "Mayor's office",
        paper: 10,
        floor: Floor::Planks,
        open: (480.0, 1080.0),
        goods: 6,
        layout: &[
            "##W##P#L#P##W##",
            "#B.p...K...p.B#",
            "#.....ddd.....#",
            "#.............#",
            "#t...._____..t#",
            "#c...._____..c#",
            "#.............#",
            "#p...........p#",
            "#.............#",
            "       D       ",
        ],
    },
    PlaceDef {
        name: "The Lantern Guild",
        trade: "Adventurers' guild",
        paper: 9,
        floor: Floor::Planks,
        open: (420.0, 1320.0),
        goods: 1,
        layout: &[
            "###W###L###W###",
            "#r..R..K..h..r#",
            "#....CCCCC....#",
            "#.............#",
            "#m..t.c...t.c.#",
            "#...c.....c...#",
            "#.............#",
            "#b....___....x#",
            "#.............#",
            "       D       ",
        ],
    },
    PlaceDef {
        name: "Moonquill Scriptorium",
        trade: "Scrolls and magic",
        paper: 3,
        floor: Floor::Carpet,
        open: (540.0, 1260.0),
        goods: 3,
        layout: &[
            "##W##L###L##W##",
            "#B.B...K...B.B#",
            "#....CCCCC....#",
            "#.............#",
            "#u.....E.....p#",
            "#.............#",
            "#s....___....s#",
            "#.............#",
            "#.............#",
            "       D       ",
        ],
    },
    PlaceDef {
        name: "Petalplate Armory",
        trade: "Armour and shields",
        paper: 0,
        floor: Floor::Planks,
        open: (480.0, 1200.0),
        goods: 0,
        layout: &[
            "##W###L#L###W##",
            "#s.s...K...s.s#",
            "#....CCCCC....#",
            "#.............#",
            "#m.m.......m.m#",
            "#.............#",
            "#m....___....b#",
            "#............x#",
            "#.............#",
            "       D       ",
        ],
    },
    PlaceDef {
        name: "Sprig & Steel",
        trade: "Swords, wands and staffs",
        paper: 1,
        floor: Floor::Cobble,
        open: (480.0, 1200.0),
        goods: 1,
        layout: &[
            "##W##L###L##W##",
            "#r.r.h.K.a.r.r#",
            "#....CCCCC....#",
            "#.............#",
            "#r...........r#",
            "#.............#",
            "#b....___....x#",
            "#x....___....b#",
            "#.............#",
            "       D       ",
        ],
    },
    PlaceDef {
        name: "Tinkerbolt Tools",
        trade: "Tools and sprinklers",
        paper: 4,
        floor: Floor::Planks,
        open: (480.0, 1200.0),
        goods: 4,
        layout: &[
            "##W##L###L##W##",
            "#s.n...K...n.s#",
            "#....CCCCC....#",
            "#.............#",
            "#s...........s#",
            "#.............#",
            "#x.b..___..b.x#",
            "#x....___....x#",
            "#.............#",
            "       D       ",
        ],
    },
    PlaceDef {
        name: "Glimmer & Gold",
        trade: "Gems and curios",
        paper: 6,
        floor: Floor::Carpet,
        open: (540.0, 1200.0),
        goods: 5,
        layout: &[
            "##W##P#L#P##W##",
            "#p.g...K...g.p#",
            "#....CCCCC....#",
            "#.............#",
            "#g...g...g...g#",
            "#.............#",
            "#.....___.....#",
            "#p...........p#",
            "#.............#",
            "       D       ",
        ],
    },
    PlaceDef {
        name: "Cozy Nook",
        trade: "Furniture and decor",
        paper: 7,
        floor: Floor::Planks,
        open: (480.0, 1200.0),
        goods: 7,
        // Digits are furniture on display (see `display_piece`).
        layout: &[
            "##W##P#L#P##W##",
            "#s.s...K...s.s#",
            "#....CCCCC....#",
            "#4...........5#",
            "#l..f.....f..l#",
            "#1..7......3..#",
            "#q....___....q#",
            "#j...........j#",
            "#8..........6.#",
            "       D       ",
        ],
    },
    PlaceDef {
        name: "Sproutling Seeds",
        trade: "Seeds for every season",
        paper: 2,
        floor: Floor::Planks,
        open: (420.0, 1140.0),
        goods: 2,
        layout: &[
            "##W##L###L##W##",
            "#s.s.p.K.p.s.s#",
            "#....CCCCC....#",
            "#.............#",
            "#i.i.......i.i#",
            "#.............#",
            "#p....___....p#",
            "#i...........i#",
            "#.............#",
            "       D       ",
        ],
    },
    PlaceDef {
        name: "Honeycrumb Bakery",
        trade: "Bread, cakes and treats",
        paper: 5,
        floor: Floor::Tiles,
        open: (390.0, 1140.0),
        goods: 8,
        layout: &[
            "##W##L###L##W##",
            "#o.s...K...s.o#",
            "#....CCCCC....#",
            "#.............#",
            "#t.c.......c.t#",
            "#c...........c#",
            "#.....___.....#",
            "#p...........p#",
            "#.............#",
            "       D       ",
        ],
    },
    PlaceDef {
        name: "The Sleepy Snail",
        trade: "Tavern and kitchen",
        paper: 8,
        floor: Floor::Planks,
        open: (660.0, 1500.0),
        goods: 9,
        layout: &[
            "##W#L##W##L#W##",
            "#b.T...K...T.b#",
            "#....CCCCC....#",
            "#.............#",
            "#t.c..t.c..t.c#",
            "#c....c....c..#",
            "#.............#",
            "#h...t.c.....x#",
            "#....c........#",
            "       D       ",
        ],
    },
    PlaceDef {
        name: "Starfall Spellery",
        trade: "Spells and potions",
        paper: 6,
        floor: Floor::Carpet,
        open: (600.0, 1320.0),
        goods: 10,
        layout: &[
            "##W##L###L##W##",
            "#B.u...K...u.B#",
            "#....CCCCC....#",
            "#.............#",
            "#s...........s#",
            "#.....___.....#",
            "#p....___....p#",
            "#s....___....s#",
            "#.............#",
            "       D       ",
        ],
    },
];

/// Walls, roof and trim for a building.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Facade {
    Plaster,
    Brick,
    Stone,
    Boards,
    Timber,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Extra {
    None,
    ClockTower,
    Turret,
    Banners,
    SnailSign,
    Greenhouse,
    Telescope,
    Mailbox,
}

#[derive(Clone, Copy, Debug)]
pub struct Look {
    pub facade: Facade,
    /// Light, main and dark wall colours.
    pub wall: [u8; 3],
    pub roof: [u8; 3],
    /// Door and window frames.
    pub trim: u8,
    pub door: u8,
    pub awning: Option<(u8, u8)>,
    /// The icon on the hanging sign.
    pub sign: Option<&'static str>,
    pub tall: bool,
    /// The gable end faces the street.
    pub gable: bool,
    pub chimney: bool,
    pub extra: Extra,
}

pub struct Building {
    pub name: &'static str,
    pub place: Option<Place>,
    /// Front-left tile (the anchor); the building covers `w` tiles to the east and `d` rows
    /// to the north.
    pub x: i32,
    pub z: i32,
    pub w: i32,
    pub d: i32,
    /// Door column, counted from the left.
    pub door: i32,
    pub look: Look,
}

impl Building {
    /// The tile in front of the door.
    pub fn step(&self) -> (i32, i32) {
        (self.x + self.door, self.z + 1)
    }
}

const fn look(facade: Facade, wall: [u8; 3], roof: [u8; 3], trim: u8) -> Look {
    Look {
        facade,
        wall,
        roof,
        trim,
        door: RUST,
        awning: None,
        sign: None,
        tall: false,
        gable: false,
        chimney: false,
        extra: Extra::None,
    }
}

const PLASTER: [u8; 3] = [WHITE, CREAM, SAND];
const ROSE_PLASTER: [u8; 3] = [PEACH, SALMON, ROSEWOOD];
const MINT_PLASTER: [u8; 3] = [WHITE, MINT, AQUA];
const SKY_PLASTER: [u8; 3] = [WHITE, SKY, BLUE];
const LILAC_PLASTER: [u8; 3] = [WHITE, BLUSH, LAVENDER];
const RED_BRICK: [u8; 3] = [CLAY, RUST, MAROON];
const STONE: [u8; 3] = [SAND, KHAKI, SHADOW];
const WOOD: [u8; 3] = [GOLD, CLAY, RUST];

const RED_ROOF: [u8; 3] = [SALMON, CRIMSON, PLUM];
const BLUE_ROOF: [u8; 3] = [SKY, BLUE, INDIGO];
const GREEN_ROOF: [u8; 3] = [LIME, GREEN, TEAL];
const GOLD_ROOF: [u8; 3] = [GOLD, CLAY, RUST];
const PURPLE_ROOF: [u8; 3] = [LAVENDER, PURPLE, GRAPE];
const SLATE_ROOF: [u8; 3] = [SLATE, INDIGO, INK];
const TEAL_ROOF: [u8; 3] = [AQUA, TEAL, DEEP_TEAL];
const PINK_ROOF: [u8; 3] = [BLUSH, PINK, CRIMSON];

pub static BUILDINGS: [Building; 20] = [
    Building {
        name: "The Lantern Guild",
        place: Some(Place::Guild),
        x: 4,
        z: 9,
        w: 8,
        d: 6,
        door: 3,
        look: Look {
            sign: Some("twig_sword"),
            tall: true,
            chimney: true,
            extra: Extra::Banners,
            door: MAROON,
            ..look(Facade::Stone, STONE, SLATE_ROOF, RUST)
        },
    },
    Building {
        name: "Juniper's cottage",
        place: None,
        x: 14,
        z: 9,
        w: 5,
        d: 4,
        door: 2,
        look: Look {
            gable: true,
            chimney: true,
            ..look(Facade::Plaster, MINT_PLASTER, TEAL_ROOF, RUST)
        },
    },
    Building {
        name: "Town Hall",
        place: Some(Place::Hall),
        x: 29,
        z: 9,
        w: 11,
        d: 6,
        door: 5,
        look: Look {
            tall: true,
            extra: Extra::ClockTower,
            door: MAROON,
            ..look(Facade::Brick, RED_BRICK, BLUE_ROOF, CREAM)
        },
    },
    Building {
        name: "Bramble's cottage",
        place: None,
        x: 42,
        z: 9,
        w: 5,
        d: 4,
        door: 1,
        look: Look {
            chimney: true,
            ..look(Facade::Timber, PLASTER, PURPLE_ROOF, RUST)
        },
    },
    Building {
        name: "Moonquill Scriptorium",
        place: Some(Place::Scrolls),
        x: 51,
        z: 9,
        w: 7,
        d: 5,
        door: 3,
        look: Look {
            sign: Some("weapon_scroll"),
            awning: Some((PURPLE, LAVENDER)),
            gable: true,
            extra: Extra::Turret,
            door: GRAPE,
            ..look(Facade::Plaster, LILAC_PLASTER, PURPLE_ROOF, GOLD)
        },
    },
    Building {
        name: "Petalplate Armory",
        place: Some(Place::Armory),
        x: 4,
        z: 18,
        w: 7,
        d: 4,
        door: 3,
        look: Look {
            sign: Some("copper_helm"),
            awning: Some((PINK, WHITE)),
            ..look(Facade::Stone, STONE, PINK_ROOF, RUST)
        },
    },
    Building {
        name: "Sprig & Steel",
        place: Some(Place::Smithy),
        x: 12,
        z: 18,
        w: 7,
        d: 4,
        door: 3,
        look: Look {
            sign: Some("mighty_leek"),
            awning: Some((ORANGE, CREAM)),
            chimney: true,
            door: MAROON,
            ..look(Facade::Brick, RED_BRICK, SLATE_ROOF, GOLD)
        },
    },
    Building {
        name: "Tinkerbolt Tools",
        place: Some(Place::Tools),
        x: 24,
        z: 18,
        w: 6,
        d: 4,
        door: 2,
        look: Look {
            sign: Some("copper_hoe"),
            awning: Some((GOLD, CREAM)),
            gable: true,
            ..look(Facade::Boards, WOOD, GREEN_ROOF, RUST)
        },
    },
    Building {
        name: "Honeycrumb Bakery",
        place: Some(Place::Bakery),
        x: 32,
        z: 18,
        w: 6,
        d: 4,
        door: 2,
        look: Look {
            sign: Some("fresh_bread"),
            awning: Some((SALMON, WHITE)),
            chimney: true,
            ..look(Facade::Plaster, ROSE_PLASTER, GOLD_ROOF, RUST)
        },
    },
    Building {
        name: "Glimmer & Gold",
        place: Some(Place::Jeweler),
        x: 40,
        z: 18,
        w: 6,
        d: 4,
        door: 3,
        look: Look {
            sign: Some("ruby"),
            awning: Some((TEAL, AQUA)),
            gable: true,
            door: DEEP_TEAL,
            ..look(Facade::Plaster, SKY_PLASTER, TEAL_ROOF, GOLD)
        },
    },
    Building {
        name: "Cozy Nook",
        place: Some(Place::Nook),
        x: 51,
        z: 18,
        w: 7,
        d: 4,
        door: 3,
        look: Look {
            sign: Some("lamp"),
            awning: Some((LAVENDER, WHITE)),
            ..look(Facade::Timber, PLASTER, RED_ROOF, RUST)
        },
    },
    Building {
        name: "Sproutling Seeds",
        place: Some(Place::Seeds),
        x: 5,
        z: 31,
        w: 7,
        d: 5,
        door: 3,
        look: Look {
            sign: Some("turnip_seeds"),
            awning: Some((GREEN, CREAM)),
            extra: Extra::Greenhouse,
            ..look(Facade::Boards, [CREAM, LIME, GREEN], GOLD_ROOF, RUST)
        },
    },
    Building {
        name: "Olive's cottage",
        place: None,
        x: 14,
        z: 31,
        w: 5,
        d: 4,
        door: 2,
        look: Look {
            chimney: true,
            ..look(Facade::Plaster, PLASTER, GREEN_ROOF, RUST)
        },
    },
    Building {
        name: "The Sleepy Snail",
        place: Some(Place::Tavern),
        x: 51,
        z: 31,
        w: 8,
        d: 5,
        door: 3,
        look: Look {
            tall: true,
            chimney: true,
            extra: Extra::SnailSign,
            door: MAROON,
            ..look(Facade::Timber, [PEACH, SAND, KHAKI], RED_ROOF, RUST)
        },
    },
    Building {
        name: "Grandma Fern's cottage",
        place: None,
        x: 4,
        z: 41,
        w: 5,
        d: 4,
        door: 2,
        look: Look {
            chimney: true,
            gable: true,
            ..look(Facade::Plaster, ROSE_PLASTER, PINK_ROOF, CREAM)
        },
    },
    Building {
        name: "Pip's house",
        place: None,
        x: 11,
        z: 41,
        w: 6,
        d: 4,
        door: 2,
        look: Look {
            chimney: true,
            ..look(Facade::Boards, [SKY, BLUE, INDIGO], GOLD_ROOF, CREAM)
        },
    },
    Building {
        name: "Toby's house",
        place: None,
        x: 25,
        z: 41,
        w: 5,
        d: 4,
        door: 2,
        look: Look {
            extra: Extra::Mailbox,
            ..look(Facade::Brick, RED_BRICK, GREEN_ROOF, CREAM)
        },
    },
    Building {
        name: "Sir Clank's house",
        place: None,
        x: 32,
        z: 41,
        w: 6,
        d: 4,
        door: 2,
        look: Look {
            chimney: true,
            gable: true,
            ..look(Facade::Stone, STONE, BLUE_ROOF, RUST)
        },
    },
    Building {
        name: "Mira's house",
        place: None,
        x: 40,
        z: 41,
        w: 5,
        d: 4,
        door: 2,
        look: Look {
            extra: Extra::Telescope,
            ..look(Facade::Plaster, SKY_PLASTER, PURPLE_ROOF, GOLD)
        },
    },
    Building {
        name: "Starfall Spellery",
        place: Some(Place::Spellery),
        x: 51,
        z: 41,
        w: 7,
        d: 5,
        door: 3,
        look: Look {
            sign: Some("spellbook"),
            awning: Some((INDIGO, GOLD)),
            tall: true,
            gable: true,
            extra: Extra::Turret,
            door: GRAPE,
            ..look(Facade::Stone, [LAVENDER, PURPLE, GRAPE], TEAL_ROOF, GOLD)
        },
    },
];

/// Important spots in town.
pub struct TownMarks {
    /// Where the bus stops (the shelter is just north of the road here).
    pub stop: (i32, i32),
    pub arrive: (i32, i32),
    pub board: (i32, i32),
    pub fountain: (i32, i32),
    pub wish_tree: (i32, i32),
}

pub const TOWN: TownMarks = TownMarks {
    stop: (18, 42),
    arrive: (20, 43),
    board: (30, 22),
    fountain: (35, 26),
    wish_tree: (67, 24),
};

/// The streets, as rectangles (x0, z0, x1, z1), inclusive.
const STREETS: [(i32, i32, i32, i32); 6] = [
    (3, 10, 62, 11),
    (3, 19, 62, 20),
    (3, 32, 62, 33),
    (0, 43, 71, 44),
    (21, 10, 22, 44),
    (48, 10, 49, 44),
];

/// Builds the town. `done` holds the restoration projects finished so far.
pub fn generate(done: u32) -> World {
    let mut w = World::new(TOWN_W, TOWN_H, Area::Town, 0);
    let mut r = Rng::new(0x0B2A_4B1E);
    for z in 0..TOWN_H {
        for x in 0..TOWN_W {
            w.set_floor(x, z, Floor::Grass);
        }
    }
    // The stream and its pond on the east side.
    for z in 3..TOWN_H - 3 {
        let wob = (hash2(0, z / 3, 11) % 2) as i32;
        for x in 61..=62 {
            w.set_floor(x + if z > 36 { wob } else { 0 }, z, Floor::Water);
        }
    }
    for z in 35..43 {
        for x in 66..69 {
            let dx = (x as f32 + 0.5 - 67.6) / 1.5;
            let dz = (z as f32 + 0.5 - 38.8) / 2.6;
            if dx * dx + dz * dz < 1.0 {
                w.set_floor(x, z, Floor::Water);
            } else if dx * dx + dz * dz < 1.5 {
                w.set_floor(x, z, Floor::Sand);
            }
        }
    }
    // Streets, the plaza and paths.
    for &(x0, z0, x1, z1) in &STREETS {
        for z in z0..=z1 {
            for x in x0..=x1 {
                if w.floor(x, z) == Floor::Water {
                    // Bridges: the bus road always has one, the others once rebuilt.
                    if z >= 43 || done & BRIDGE != 0 {
                        w.set_floor(x, z, Floor::Planks);
                    }
                } else {
                    w.set_floor(x, z, Floor::Street);
                }
            }
        }
    }
    for z in 21..=31 {
        for x in 24..=46 {
            w.set_floor(x, z, Floor::Plaza);
        }
    }
    // The park path east of the stream.
    for z in 6..=46 {
        w.set_floor(65, z, Floor::Path);
    }
    for x in 63..=68 {
        for z in [10, 11, 19, 20, 32, 33] {
            if w.floor(x, z) == Floor::Grass {
                w.set_floor(x, z, Floor::Path);
            }
        }
    }
    // Garden paths up to every door.
    for b in &BUILDINGS {
        let (sx, sz) = b.step();
        let mut z = sz;
        while w.floor(sx, z) == Floor::Grass && z < sz + 3 {
            w.set_floor(sx, z, Floor::Path);
            z += 1;
        }
    }

    // Buildings.
    for (id, b) in BUILDINGS.iter().enumerate() {
        for dz in 0..b.d {
            for dx in 0..b.w {
                let (x, z) = (b.x + dx, b.z - dz);
                let o = if dx == 0 && dz == 0 {
                    Obj::Building { id: id as u8 }
                } else {
                    Obj::Part {
                        ax: b.x as i16,
                        az: b.z as i16,
                    }
                };
                w.set_obj(x, z, Some(o));
            }
        }
    }

    // The fountain (3x3) in the middle of the plaza.
    let (fx, fz) = TOWN.fountain;
    for dz in -1..=1 {
        for dx in -1..=1 {
            let o = if dx == 0 && dz == 0 {
                Obj::Fountain {
                    flowing: done & FOUNTAIN != 0,
                }
            } else {
                Obj::Part {
                    ax: fx as i16,
                    az: fz as i16,
                }
            };
            w.set_obj(fx + dx, fz + dz, Some(o));
        }
    }
    w.set_obj(TOWN.board.0, TOWN.board.1, Some(Obj::Board));
    // The bus shelter (2x1) and its sign.
    let (bx, bz) = TOWN.stop;
    w.set_obj(bx, bz, Some(Obj::BusStop));
    w.set_obj(
        bx + 1,
        bz,
        Some(Obj::Part {
            ax: bx as i16,
            az: bz as i16,
        }),
    );

    // Plaza furniture.
    for (x, z) in [(28, 28), (42, 28), (28, 24), (42, 24)] {
        w.set_obj(x, z, Some(Obj::Bench));
    }
    let planters = [
        (32, 23),
        (38, 23),
        (32, 29),
        (38, 29),
        (24, 21),
        (46, 21),
        (24, 31),
        (46, 31),
    ];
    for (i, &(x, z)) in planters.iter().enumerate() {
        let var = if done & GARDENS != 0 {
            1 + (i % 3) as u8
        } else {
            0
        };
        w.set_obj(x, z, Some(Obj::Planter { var }));
    }
    for (x, z) in [(25, 22), (45, 22), (25, 30), (45, 30)] {
        w.set_obj(x, z, Some(Obj::Tree { var: 2, hp: 99 }));
    }
    if done & MARKET != 0 {
        for (i, &(x, z)) in [(26, 26), (44, 26), (27, 30), (43, 30)].iter().enumerate() {
            w.set_obj(x, z, Some(Obj::Stand { var: i as u8 }));
        }
    }
    // Street lamps along the streets.
    let lamp_spots = [
        (5, 12),
        (13, 12),
        (20, 12),
        (27, 12),
        (41, 12),
        (47, 12),
        (59, 12),
        (3, 21),
        (11, 21),
        (19, 21),
        (23, 21),
        (47, 21),
        (52, 21),
        (59, 21),
        (3, 34),
        (11, 34),
        (19, 34),
        (26, 34),
        (35, 34),
        (44, 34),
        (52, 34),
        (59, 34),
        (8, 42),
        (23, 42),
        (30, 42),
        (38, 42),
        (47, 42),
        (57, 42),
    ];
    for (x, z) in lamp_spots {
        if w.obj(x, z).is_none() {
            w.set_obj(
                x,
                z,
                Some(Obj::StreetLamp {
                    lit: done & LAMPS != 0,
                    bunting: done & BUNTING != 0,
                }),
            );
        }
    }
    // The green west of the plaza: a well, benches, flower beds.
    w.set_obj(12, 24, Some(Obj::Well));
    for (x, z) in [(9, 24), (15, 24)] {
        w.set_obj(x, z, Some(Obj::Bench));
    }
    for (x, z, v) in [(5, 23, 1), (18, 23, 2), (5, 25, 3), (18, 25, 1)] {
        let var = if done & GARDENS != 0 { v } else { 0 };
        w.set_obj(x, z, Some(Obj::Planter { var }));
    }
    // A few barrels and crates by the shops.
    for (x, z) in [(11, 17), (19, 17), (30, 17), (46, 17), (58, 17), (12, 30)] {
        if w.obj(x, z).is_none() {
            let o = if (x + z) % 2 == 0 {
                Obj::Barrel
            } else {
                Obj::Crate { hp: 99 }
            };
            w.set_obj(x, z, Some(o));
        }
    }
    // The Wishing Tree in the park, with a bench to sit and watch it.
    let (tx, tz) = TOWN.wish_tree;
    w.set_obj(
        tx,
        tz,
        Some(Obj::WishTree {
            blooming: done & WISH_TREE != 0,
        }),
    );
    w.set_obj(tx, tz + 2, Some(Obj::Bench));
    w.set_obj(67, 13, Some(Obj::Bench));

    // Hedges and a forest all round.
    for x in 0..TOWN_W {
        for z in 0..TOWN_H {
            let border = !(3..TOWN_W - 3).contains(&x) || !(3..TOWN_H - 3).contains(&z);
            if !border || w.floor(x, z) != Floor::Grass {
                continue;
            }
            let o = if (x + z) % 3 == 0 {
                Obj::Pine {
                    var: (x % 2) as u8,
                    hp: 99,
                }
            } else {
                Obj::Tree {
                    var: [0, 3, 0, 1][(hash2(x, z, 4) % 4) as usize],
                    hp: 99,
                }
            };
            w.set_obj(x, z, Some(o));
        }
    }
    // Hedges along the back gardens.
    for x in 3..61 {
        if w.floor(x, 3) == Floor::Grass && w.obj(x, 3).is_none() {
            w.set_wall(x, 3, Wall::Hedge);
        }
    }
    // Bushes, flowers and trees on the lawns that are left.
    let open = |w: &World, x: i32, z: i32| {
        w.floor(x, z) == Floor::Grass
            && w.obj(x, z).is_none()
            && w.wall(x, z) == Wall::None
            && ![(0, 1), (0, -1), (1, 0), (-1, 0)]
                .iter()
                .any(|(dx, dz)| matches!(w.floor(x + dx, z + dz), Floor::Street | Floor::Path))
    };
    let mut placed = 0;
    let mut tries = 0;
    while placed < 70 && tries < 4000 {
        tries += 1;
        let x = r.range(3, TOWN_W - 3);
        let z = r.range(4, TOWN_H - 3);
        if !open(&w, x, z) {
            continue;
        }
        // Keep the fronts of buildings clear.
        if BUILDINGS
            .iter()
            .any(|b| x >= b.x - 1 && x <= b.x + b.w && z > b.z && z <= b.z + 2)
        {
            continue;
        }
        let roll = r.f32();
        let o = if roll < 0.25 {
            Obj::Tree {
                var: r.below(4) as u8,
                hp: 99,
            }
        } else if roll < 0.55 {
            Obj::Bush {
                var: r.below(3) as u8,
            }
        } else {
            Obj::Flower {
                var: r.below(4) as u8,
            }
        };
        w.set_obj(x, z, Some(o));
        placed += 1;
    }
    w
}

// ------------------------------------------------------------------------------------------
// Rooms
// ------------------------------------------------------------------------------------------

/// Loose decoration inside a room, drawn but never in the way.
#[derive(Clone, Copy, Debug)]
pub enum Decor {
    /// A rug covering tiles (x0, z0)..=(x1, z1).
    Rug { x0: i32, z0: i32, x1: i32, z1: i32 },
    /// A window in the back wall above column x.
    Window { x: i32 },
    /// A framed picture on the back wall.
    Painting { x: i32 },
    /// A wall lamp (it lights the room).
    Sconce { x: i32 },
}

pub struct Room {
    pub place: Place,
    pub world: World,
    /// Stepping here leads back outside.
    pub exit: (i32, i32),
    /// Where the keeper stands.
    pub keeper: (i32, i32),
    pub decor: Vec<Decor>,
}

/// Builds the inside of a place from its layout.
///
/// Legend: `#` wall, `W` window, `P` painting, `L` wall lamp, `.` floor, `C` counter,
/// `K` the keeper, `s` shelves, `r` weapon rack, `m` mannequin, `t` table, `c` stool,
/// `b` barrel, `x` crate, `h` hearth, `p` potted plant, `E` enchanting table, `B` bookcase,
/// `o` oven, `a` anvil, `u` cauldron, `g` gem case, `n` tinker's bench, `i` seed bins,
/// `d` desk, `R` bounty board, `T` taps, `l` lamp, `f` flower pot, `q` chest, `j` bench,
/// `_` rug, `D` the door out, and spaces are nothing at all.
/// Furniture Wren shows off in her shop, by layout digit, and the fish in any tank.
fn display_piece(ch: char) -> Option<(super::home::Furn, &'static [Item])> {
    use super::home::Furn;
    Some(match ch {
        '1' => (Furn::Sofa, &[]),
        '3' => (
            Furn::FishTank,
            &[
                Item::LilyKoi,
                Item::Bluegill,
                Item::SunnyMinnow,
                Item::PrismGuppy,
            ],
        ),
        '4' => (Furn::Clock, &[]),
        '5' => (Furn::Bookshelf, &[]),
        '6' => (Furn::Piano, &[]),
        '7' => (Furn::Globe, &[]),
        '8' => (Furn::Fern, &[]),
        _ => return None,
    })
}

pub fn room(place: Place) -> Room {
    let def = place.def();
    let rows = def.layout;
    let h = rows.len() as i32;
    let w = rows.iter().map(|r| r.len()).max().unwrap_or(1) as i32;
    let mut world = World::new(w, h, Area::Inside(place), 0);
    let mut exit = (w / 2, h - 1);
    let mut keeper = (w / 2, 1);
    let mut decor = Vec::new();
    let mut rug: Option<(i32, i32, i32, i32)> = None;
    let mut mannequins = 0u8;
    for (z, row) in rows.iter().enumerate() {
        let z = z as i32;
        for (x, ch) in row.chars().enumerate() {
            let x = x as i32;
            if ch == ' ' {
                continue;
            }
            if matches!(ch, '#' | 'W' | 'P' | 'L') {
                world.set_floor(x, z, def.floor);
                world.set_wall(x, z, Wall::Paper(def.paper));
                match ch {
                    'W' => decor.push(Decor::Window { x }),
                    'P' => decor.push(Decor::Painting { x }),
                    'L' => decor.push(Decor::Sconce { x }),
                    _ => {}
                }
                continue;
            }
            world.set_floor(x, z, def.floor);
            let var = def.goods;
            let o = match ch {
                'C' => Some(Obj::Counter),
                'K' => {
                    keeper = (x, z);
                    None
                }
                's' => Some(Obj::Shelf { var }),
                'r' => Some(Obj::Rack { var: (x as u8) % 3 }),
                'm' => {
                    mannequins += 1;
                    Some(Obj::Mannequin {
                        var: mannequins - 1,
                    })
                }
                't' => Some(Obj::Table { var }),
                'c' => Some(Obj::Stool),
                'b' => Some(Obj::Barrel),
                'x' => Some(Obj::Crate { hp: 99 }),
                'h' => Some(Obj::Hearth),
                'p' => Some(Obj::Fixture { var: 9 }),
                'E' => Some(Obj::EnchantTable),
                'B' => Some(Obj::Fixture { var: 6 }),
                'o' => Some(Obj::Fixture { var: 0 }),
                'a' => Some(Obj::Fixture { var: 1 }),
                'u' => Some(Obj::Fixture { var: 2 }),
                'g' => Some(Obj::Fixture { var: 3 }),
                'n' => Some(Obj::Fixture { var: 4 }),
                'i' => Some(Obj::Fixture { var: 5 }),
                'd' => Some(Obj::Fixture { var: 7 }),
                'T' => Some(Obj::Fixture { var: 8 }),
                'R' => Some(Obj::Board),
                'l' => Some(Obj::Lamp),
                'f' => Some(Obj::FlowerPot { var: 0 }),
                'q' => Some(Obj::Chest { items: Vec::new() }),
                'j' => Some(Obj::Bench),
                '_' => {
                    rug = Some(match rug {
                        None => (x, z, x, z),
                        Some((x0, z0, x1, z1)) => (x0.min(x), z0.min(z), x1.max(x), z1.max(z)),
                    });
                    None
                }
                'D' => {
                    exit = (x, z);
                    None
                }
                '0'..='9' => {
                    if let Some((f, fish)) = display_piece(ch) {
                        super::home::put(&mut world, f, 0, x, z);
                        if let Some(Obj::Furniture { fish: f_in, .. }) = world.obj_mut(x, z) {
                            f_in.extend_from_slice(fish);
                        }
                    }
                    None
                }
                _ => None,
            };
            if o.is_some() {
                world.set_obj(x, z, o);
            }
        }
    }
    if let Some((x0, z0, x1, z1)) = rug {
        decor.push(Decor::Rug { x0, z0, x1, z1 });
    }
    Room {
        place,
        world,
        exit,
        keeper,
        decor,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn buildings_fit_and_do_not_overlap() {
        let mut used = vec![false; (TOWN_W * TOWN_H) as usize];
        for b in &BUILDINGS {
            assert!(
                b.x >= 3 && b.x + b.w <= TOWN_W - 3,
                "{} is off the map",
                b.name
            );
            assert!(
                b.z - b.d + 1 >= 3 && b.z < TOWN_H - 3,
                "{} is off the map",
                b.name
            );
            assert!(b.door >= 0 && b.door < b.w);
            for dz in 0..b.d {
                for dx in 0..b.w {
                    let i = ((b.z - dz) * TOWN_W + b.x + dx) as usize;
                    assert!(!used[i], "{} overlaps another building", b.name);
                    used[i] = true;
                }
            }
        }
        for p in PLACES {
            assert_eq!(BUILDINGS[p.building()].place, Some(p));
        }
    }

    #[test]
    fn every_door_opens_onto_the_street() {
        let w = generate(0);
        for b in &BUILDINGS {
            let (x, z) = b.step();
            assert!(!w.blocked(x, z), "the door of {} is blocked", b.name);
        }
        let (x, z) = TOWN.arrive;
        assert!(!w.blocked(x, z));
    }

    #[test]
    fn rooms_have_a_keeper_and_a_way_out() {
        for p in PLACES {
            let r = room(p);
            let (ex, ez) = r.exit;
            assert!(!r.world.blocked(ex, ez), "{:?} exit blocked", p);
            assert!(
                !r.world.blocked(r.keeper.0, r.keeper.1),
                "{:?} keeper stuck",
                p
            );
            // The counter or desk sits in front of the keeper.
            assert!(
                r.world.blocked(r.keeper.0, r.keeper.1 + 1),
                "{:?} keeper exposed",
                p
            );
            // Every row has the same width.
            let w = p.def().layout[0].len();
            assert!(
                p.def().layout.iter().all(|r| r.len() == w),
                "{:?} ragged",
                p
            );
        }
    }
}
