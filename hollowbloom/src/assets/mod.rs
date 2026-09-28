//! Every texture, sprite, font and model the game uses, generated at start-up.

pub mod font;
pub mod gear_art;
pub mod item_art;
pub mod models;
pub mod quest_art;
pub mod sprites;
pub mod tiles;
pub mod town_art;

use std::collections::HashMap;

use crate::palette::*;
use crate::render::{Mesh, TexBank, TexId, Texture};
use font::Font;
use gear_art::GearArt;
use models::{Critters, Humanoid, Props};

pub const BIOMES: usize = 6;
pub const ORES: usize = 6;

pub struct BiomeTex {
    pub floor: [TexId; 3],
    pub side: TexId,
    pub top: TexId,
    pub ore_side: [TexId; ORES],
    pub ore_top: [TexId; ORES],
}

pub struct BiomeStyle {
    pub name: &'static str,
    pub floor: [u8; 4],
    pub accent: u8,
    pub side: [u8; 4],
    pub top: [u8; 4],
    pub bricks: bool,
    /// Ambient light level and warmth in the dark.
    pub ambient: f32,
    pub warmth: f32,
    pub clear: u8,
}

pub const BIOME_STYLES: [BiomeStyle; BIOMES] = [
    BiomeStyle {
        name: "Mossy Burrows",
        floor: [KHAKI, ROSEWOOD, SHADOW, INK],
        accent: TEAL,
        side: [KHAKI, ROSEWOOD, SHADOW, INK],
        top: [ROSEWOOD, SHADOW, SHADOW, INK],
        bricks: false,
        ambient: 0.42,
        warmth: 3.6,
        clear: INK,
    },
    BiomeStyle {
        name: "Crystal Grotto",
        floor: [SKY, BLUE, INDIGO, SLATE],
        accent: AQUA,
        side: [SKY, BLUE, INDIGO, SLATE],
        top: [BLUE, INDIGO, SLATE, INK],
        bricks: false,
        ambient: 0.4,
        warmth: 2.4,
        clear: INK,
    },
    BiomeStyle {
        name: "Fungal Hollow",
        floor: [LAVENDER, PURPLE, GRAPE, INK],
        accent: PINK,
        side: [LAVENDER, PURPLE, GRAPE, INK],
        top: [PURPLE, GRAPE, GRAPE, INK],
        bricks: false,
        ambient: 0.4,
        warmth: 3.4,
        clear: INK,
    },
    BiomeStyle {
        name: "Ember Depths",
        floor: [CLAY, RUST, MAROON, INK],
        accent: ORANGE,
        side: [CLAY, RUST, MAROON, INK],
        top: [RUST, MAROON, MAROON, INK],
        bricks: false,
        ambient: 0.42,
        warmth: 6.0,
        clear: INK,
    },
    BiomeStyle {
        name: "Frost Caverns",
        floor: [WHITE, SKY, BLUE, INDIGO],
        accent: WHITE,
        side: [WHITE, SKY, BLUE, INDIGO],
        top: [SKY, BLUE, INDIGO, SLATE],
        bricks: false,
        ambient: 0.5,
        warmth: 2.0,
        clear: SLATE,
    },
    BiomeStyle {
        name: "Sunken Ruins",
        floor: [SAND, KHAKI, ROSEWOOD, SHADOW],
        accent: TEAL,
        side: [SAND, KHAKI, ROSEWOOD, SHADOW],
        top: [KHAKI, ROSEWOOD, SHADOW, INK],
        bricks: true,
        ambient: 0.44,
        warmth: 4.6,
        clear: INK,
    },
];

/// Ore colours (highlight, mid, shadow): copper, iron, gold, crystal, ember, frost.
pub const ORE_COLORS: [[u8; 3]; ORES] = [
    [GOLD, ORANGE, RUST],
    [WHITE, SKY, INDIGO],
    [CREAM, GOLD, CLAY],
    [MINT, AQUA, TEAL],
    [GOLD, RED, MAROON],
    [WHITE, LAVENDER, PURPLE],
];

/// Enemy meshes recoloured per biome.
pub struct FoeSkins {
    pub slime: Vec<crate::render::Mesh>,
    pub shroom: Vec<crate::render::Mesh>,
    pub bat_body: Vec<crate::render::Mesh>,
    pub bat_wing: Vec<crate::render::Mesh>,
    pub crab: Vec<crate::render::Mesh>,
    pub wisp: Vec<crate::render::Mesh>,
    pub beetle: Vec<crate::render::Mesh>,
    pub golem: Vec<crate::render::Mesh>,
}

/// Copies a mesh, swapping each listed texture for a recoloured copy.
fn recolor(
    bank: &mut TexBank,
    mesh: &crate::render::Mesh,
    texs: &[TexId],
    map: &dyn Fn(u8) -> u8,
) -> crate::render::Mesh {
    let mut m = mesh.clone();
    for &t in texs {
        let nt = bank.get(t).map_colors(map);
        let id = bank.add(nt);
        m = m.retexture(t, id);
    }
    m
}

fn ramp(from: [u8; 3], to: [u8; 3]) -> impl Fn(u8) -> u8 {
    move |c| {
        if c == from[0] {
            to[0]
        } else if c == from[1] {
            to[1]
        } else if c == from[2] {
            to[2]
        } else {
            c
        }
    }
}

fn foe_skins(bank: &mut TexBank, c: &Critters) -> FoeSkins {
    let slime_cols = [
        [LIME, GREEN, TEAL],
        [SKY, BLUE, INDIGO],
        [BLUSH, PINK, CRIMSON],
        [GOLD, ORANGE, RUST],
        [WHITE, SKY, BLUE],
        [LAVENDER, PURPLE, GRAPE],
    ];
    let caps = [RED, BLUE, PINK, ORANGE, SKY, GOLD];
    let furs = [
        (GRAPE, PURPLE),
        (SLATE, INDIGO),
        (PLUM, CRIMSON),
        (MAROON, RED),
        (INDIGO, SKY),
        (SHADOW, ROSEWOOD),
    ];
    let shells = [
        [GOLD, CLAY, RUST],
        [MINT, AQUA, TEAL],
        [BLUSH, PINK, CRIMSON],
        [CREAM, ORANGE, RED],
        [WHITE, SKY, BLUE],
        [SAND, KHAKI, ROSEWOOD],
    ];
    let wisps = [
        [CREAM, GOLD, ORANGE],
        [WHITE, MINT, AQUA],
        [BLUSH, PINK, LAVENDER],
        [CREAM, GOLD, RED],
        [WHITE, WHITE, SKY],
        [BLUSH, LAVENDER, PURPLE],
    ];
    let beetles = [
        [LIME, GREEN, TEAL],
        [SKY, BLUE, INDIGO],
        [LAVENDER, PURPLE, GRAPE],
        [GOLD, ORANGE, RUST],
        [WHITE, SKY, BLUE],
        [CREAM, GOLD, CLAY],
    ];
    let golems = [
        [KHAKI, ROSEWOOD, SHADOW],
        [SKY, INDIGO, SLATE],
        [LAVENDER, PURPLE, GRAPE],
        [CLAY, RUST, MAROON],
        [WHITE, SKY, BLUE],
        [SAND, KHAKI, ROSEWOOD],
    ];
    let mut s = FoeSkins {
        slime: vec![],
        shroom: vec![],
        bat_body: vec![],
        bat_wing: vec![],
        crab: vec![],
        wisp: vec![],
        beetle: vec![],
        golem: vec![],
    };
    for b in 0..BIOMES {
        s.slime.push(recolor(
            bank,
            &c.slime,
            &[c.slime_tex],
            &ramp([LIME, GREEN, TEAL], slime_cols[b]),
        ));
        let cap = caps[b];
        s.shroom
            .push(recolor(bank, &c.shroom, &[c.cap_tex], &move |x| {
                if x == RED { cap } else { x }
            }));
        let (f0, f1) = furs[b];
        s.bat_body
            .push(recolor(bank, &c.bat_body, &[c.fur_tex], &move |x| {
                if x == GRAPE { f0 } else { x }
            }));
        s.bat_wing.push(recolor(
            bank,
            &c.bat_wing,
            &[c.wing_tex],
            &move |x| match x {
                GRAPE => f0,
                PURPLE => f1,
                _ => x,
            },
        ));
        s.crab.push(recolor(
            bank,
            &c.crab,
            &[c.shell_tex],
            &ramp([MINT, AQUA, TEAL], shells[b]),
        ));
        s.wisp.push(recolor(
            bank,
            &c.wisp,
            &[c.wisp_tex],
            &ramp([WHITE, MINT, AQUA], wisps[b]),
        ));
        s.beetle.push(recolor(
            bank,
            &c.beetle,
            &[c.beetle_tex],
            &ramp([LAVENDER, PURPLE, GRAPE], beetles[b]),
        ));
        s.golem.push(recolor(
            bank,
            &c.golem,
            &[c.golem_tex],
            &ramp([KHAKI, ROSEWOOD, SHADOW], golems[b]),
        ));
    }
    s
}

pub struct Assets {
    pub foes: FoeSkins,
    pub bank: TexBank,
    pub icons: HashMap<&'static str, TexId>,
    pub font: Font,
    pub grass: [TexId; 4],
    pub grass_flowers: [TexId; 2],
    pub path: [TexId; 2],
    pub soil: TexId,
    pub tilled: TexId,
    pub tilled_wet: TexId,
    pub sand: TexId,
    pub water: [TexId; 4],
    pub wood_floor: TexId,
    pub stone_floor: TexId,
    /// Roads with their outer corners rounded into grass: `rounded[road * 4 + grass][mask]`,
    /// with roads in `ROADS` order and `mask` from the corner bits in `tiles`.
    pub rounded: Vec<[TexId; 16]>,
    pub street: [TexId; 3],
    pub plaza: TexId,
    pub checker: TexId,
    /// Red, violet and teal carpets.
    pub carpets: [TexId; 3],
    pub hedge_side: TexId,
    pub hedge_top: TexId,
    /// Inside walls, by style.
    pub wallpaper: Vec<TexId>,
    /// The tops of inside walls.
    pub beam: TexId,
    pub town: town_art::TownArt,
    /// Grass with road filling its inner corners, indexed the same way.
    pub fillets: Vec<[TexId; 16]>,
    pub cliff_side: TexId,
    pub biomes: Vec<BiomeTex>,
    pub stone_wall_side: TexId,
    pub stone_wall_top: TexId,
    pub wood_wall_side: TexId,
    pub wood_wall_top: TexId,
    pub bedrock: TexId,
    pub lava: [TexId; 2],
    pub disk: TexId,
    pub cursor: TexId,
    pub cursor_bad: TexId,
    pub flame: [TexId; 4],
    pub hero: Humanoid,
    pub sprout: crate::render::Mesh,
    pub critters: Critters,
    pub props: Props,
    pub gear: GearArt,
}

impl Assets {
    pub fn new() -> Assets {
        let mut bank = TexBank::default();
        let mut icons = sprites::build(&mut bank);
        item_art::build(&mut bank, &mut icons);
        let gear = gear_art::build(&mut bank, &mut icons);
        quest_art::build(&mut bank, &mut icons);
        let grass = [
            bank.add(tiles::grass(1, false)),
            bank.add(tiles::grass(2, false)),
            bank.add(tiles::grass(3, false)),
            bank.add(tiles::grass(4, false)),
        ];
        let grass_flowers = [
            bank.add(tiles::grass(5, true)),
            bank.add(tiles::grass(6, true)),
        ];
        let path = [bank.add(tiles::dirt_path(7)), bank.add(tiles::dirt_path(8))];
        let soil = bank.add(tiles::soil(9));
        let tilled = bank.add(tiles::tilled(false));
        let tilled_wet = bank.add(tiles::tilled(true));
        let sand = bank.add(tiles::sand(10));
        let water = [
            bank.add(tiles::water(0)),
            bank.add(tiles::water(1)),
            bank.add(tiles::water(2)),
            bank.add(tiles::water(3)),
        ];
        let wood_floor = bank.add(tiles::planks(CLAY, GOLD, RUST, 20));
        let stone_floor = bank.add(tiles::cobbles(KHAKI, SAND, SHADOW, 21));
        let cliff_side = bank.add(tiles::cliff_side());
        let street = [
            bank.add(tiles::street(22)),
            bank.add(tiles::street(23)),
            bank.add(tiles::street(24)),
        ];
        let plaza = bank.add(tiles::pavers());
        let checker = bank.add(tiles::checker(CREAM, BLUSH, SAND));
        let carpets = [
            bank.add(tiles::carpet(CRIMSON, PLUM, GOLD)),
            bank.add(tiles::carpet(GRAPE, INDIGO, LAVENDER)),
            bank.add(tiles::carpet(DEEP_TEAL, INDIGO, GOLD)),
        ];
        let hedge_side = bank.add(tiles::hedge(25));
        let hedge_top = bank.add(tiles::hedge(26));
        let wallpaper = (0..11).map(|k| bank.add(tiles::wallpaper(k))).collect();
        let beam = bank.add(tiles::planks(MAROON, RUST, INK, 27));
        // Every road gets soft corners where it meets the lawn.
        let roads = [
            path[0],
            path[1],
            sand,
            stone_floor,
            wood_floor,
            street[0],
            plaza,
        ];
        let mut rounded = Vec::new();
        let mut fillets = Vec::new();
        for &road in &roads {
            for &lawn in &grass {
                let (rt, gt) = (bank.get(road).clone(), bank.get(lawn).clone());
                let mut outer = [road; 16];
                let mut inner = [lawn; 16];
                for mask in 1..16u8 {
                    outer[mask as usize] = bank.add(tiles::round_corners(&rt, &gt, mask, 6.0));
                    inner[mask as usize] = bank.add(tiles::round_corners(&gt, &rt, mask, 3.0));
                }
                rounded.push(outer);
                fillets.push(inner);
            }
        }

        let mut biomes = Vec::new();
        for (i, st) in BIOME_STYLES.iter().enumerate() {
            let seed = 100 + i as u64 * 10;
            let (floor, side) = if st.bricks {
                (
                    [
                        bank.add(tiles::cobbles(st.floor[1], st.floor[0], st.floor[2], seed)),
                        bank.add(tiles::cobbles(
                            st.floor[1],
                            st.floor[0],
                            st.floor[3],
                            seed + 1,
                        )),
                        bank.add(tiles::cave_floor(st.floor, st.accent, seed + 2)),
                    ],
                    tiles::bricks(st.side[1], st.side[0], st.side[3], seed + 3),
                )
            } else {
                (
                    [
                        bank.add(tiles::cave_floor(st.floor, CLEAR, seed)),
                        bank.add(tiles::cave_floor(st.floor, st.accent, seed + 1)),
                        bank.add(tiles::cave_floor(st.floor, CLEAR, seed + 2)),
                    ],
                    tiles::rock_side(st.side, seed + 3),
                )
            };
            let top = tiles::rock_top(st.top, seed + 4);
            let mut ore_side = [0; ORES];
            let mut ore_top = [0; ORES];
            for o in 0..ORES {
                ore_side[o] = bank.add(tiles::with_ore(&side, ORE_COLORS[o], seed + 20 + o as u64));
                ore_top[o] = bank.add(tiles::with_ore(&top, ORE_COLORS[o], seed + 30 + o as u64));
            }
            biomes.push(BiomeTex {
                floor,
                side: bank.add(side),
                top: bank.add(top),
                ore_side,
                ore_top,
            });
        }
        let stone_wall_side = bank.add(tiles::bricks(KHAKI, SAND, SHADOW, 40));
        let stone_wall_top = bank.add(tiles::cobbles(KHAKI, SAND, SHADOW, 41));
        let wood_wall_side = bank.add(tiles::planks(CLAY, GOLD, RUST, 42));
        let wood_wall_top = bank.add(tiles::planks(RUST, CLAY, MAROON, 43));
        let bedrock = bank.add(tiles::rock_top([SHADOW, SHADOW, INK, INK], 44));
        let lava = [
            bank.add(tiles::crystal([CREAM, ORANGE, RED])),
            bank.add(tiles::crystal([GOLD, RED, ORANGE])),
        ];
        let disk = bank.add(tiles::disk());
        let cursor = bank.add(tiles::cursor(WHITE));
        let cursor_bad = bank.add(tiles::cursor(RED));
        let flame = [
            bank.add(tiles::flame(0)),
            bank.add(tiles::flame(1)),
            bank.add(tiles::flame(2)),
            bank.add(tiles::flame(3)),
        ];
        let hero = models::humanoid(&mut bank, &models::HERO);
        let sprout = models::sprout(&mut bank);
        let critters = models::critters(&mut bank);
        let props = models::props(&mut bank);
        let town = town_art::build(&mut bank, &icons, water[0]);
        let foes = foe_skins(&mut bank, &critters);
        Assets {
            foes,
            bank,
            icons,
            font: Font::regular(),
            grass,
            grass_flowers,
            path,
            soil,
            tilled,
            tilled_wet,
            sand,
            water,
            wood_floor,
            stone_floor,
            rounded,
            fillets,
            street,
            plaza,
            checker,
            carpets,
            hedge_side,
            hedge_top,
            wallpaper,
            beam,
            town,
            cliff_side,
            biomes,
            stone_wall_side,
            stone_wall_top,
            wood_wall_side,
            wood_wall_top,
            bedrock,
            lava,
            disk,
            cursor,
            cursor_bad,
            flame,
            hero,
            sprout,
            critters,
            props,
            gear,
        }
    }

    /// The 3D model of a weapon or tool held in the hand.
    pub fn held_mesh(&self, icon: &str) -> Option<&Mesh> {
        self.gear.held.get(icon)
    }

    pub fn hat_mesh(&self, icon: &str) -> Option<&Mesh> {
        self.gear.hats.get(icon)
    }

    pub fn boot_mesh(&self, icon: &str) -> Option<&Mesh> {
        self.gear.boots.get(icon)
    }

    pub fn shield_mesh(&self, icon: &str) -> Option<&Mesh> {
        self.gear.shields.get(icon)
    }

    /// Body and sleeve textures for chest armour.
    pub fn chest_skin(&self, icon: &str) -> Option<(TexId, TexId)> {
        self.gear.chest.get(icon).copied()
    }

    pub fn leg_skin(&self, icon: &str) -> Option<TexId> {
        self.gear.legs.get(icon).copied()
    }

    pub fn icon(&self, name: &str) -> TexId {
        *self
            .icons
            .get(name)
            .unwrap_or_else(|| panic!("missing sprite {name}"))
    }

    pub fn tex(&self, id: TexId) -> &Texture {
        self.bank.get(id)
    }
}

impl Default for Assets {
    fn default() -> Self {
        Self::new()
    }
}
