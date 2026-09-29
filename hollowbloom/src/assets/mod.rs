//! Every texture, sprite, font and model the game uses, generated at start-up.

pub mod deep_art;
pub mod delve_art;
pub mod fish_art;
pub mod font;
pub mod gear_art;
pub mod home_art;
pub mod item_art;
pub mod logo;
pub mod magic_art;
pub mod models;
pub mod monster_art;
pub mod pet_art;
pub mod quest_art;
pub mod season_art;
pub mod sewer_art;
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
/// Creatures of the old sewers wear a look of their own after the six biomes' (in bones,
/// mostly), and there's one more of each family's looks for it.
pub const SEWER_LOOK: usize = BIOMES;
pub const LOOKS: usize = BIOMES + 1;
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
    pub slime_core: Vec<crate::render::Mesh>,
    /// Each look's slime colours (light, mid, dark), for bubbles and glints.
    pub slime_cols: [[u8; 3]; LOOKS],
    pub shroom: Vec<crate::render::Mesh>,
    pub bat_body: Vec<crate::render::Mesh>,
    pub bat_wing: Vec<crate::render::Mesh>,
    pub crab: Vec<crate::render::Mesh>,
    pub wisp: Vec<crate::render::Mesh>,
    pub beetle: Vec<crate::render::Mesh>,
    pub golem: Vec<crate::render::Mesh>,
    pub frog: Vec<crate::render::Mesh>,
    pub frog_leg: Vec<crate::render::Mesh>,
    pub puffer: Vec<crate::render::Mesh>,
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

/// Old bone: pale, in rows with a dark gap after every `rib` of them (which a lathe wraps
/// round into a ribcage).
fn bone_tex(w: u32, h: u32, rib: i32) -> Texture {
    let mut t = Texture::new(w, h, SAND);
    for y in 0..h as i32 {
        for x in 0..w as i32 {
            let c = match y % (rib + 1) {
                k if k == rib => SHADOW,
                0 => WHITE,
                _ if (x * 5 + y * 3) % 11 == 0 => KHAKI,
                _ => SAND,
            };
            t.set(x, y, c);
        }
    }
    t
}

/// A bat's wing with nothing left of it but the bones: the arm along the top and the
/// fingers spreading from it.
fn bone_wing() -> Texture {
    let mut t = Texture::clear(16, 8);
    for x in 0..16 {
        t.set(x, 0, WHITE);
        t.set(x, 1, SAND);
        if x % 5 <= 1 {
            for y in 2..8 {
                if !(y >= 5 && x % 5 > 7 - y) {
                    t.set(x, y, if x % 5 == 0 { WHITE } else { KHAKI });
                }
            }
        }
    }
    t
}

/// A pufferfish down to its bones: a see-through cage of ribs (drawn two-sided) round a
/// glowing green heart, a spine along its back, a fan of a bony tail, and a skull's face with
/// red eyes glowing in the sockets and a mouthful of teeth. Its spines are the puffer's own.
fn bone_puffer(bank: &mut TexBank, red: TexId) -> crate::render::Mesh {
    use glam::{Mat4, Vec3};
    use models::{lathe, skin_box};
    let w4 = Texture::new(4, 4, 0);
    // Bars of rib curving down round the sides from the spine to a keel, with nothing
    // between them (the texture runs round the body, and down it).
    let mut t = Texture::clear(16, 16);
    for y in 0..16 {
        for x in 0..16 {
            let c = match (y, x % 4) {
                (7.., _) => SAND,
                (_, 0) => WHITE,
                (_, 1) => SAND,
                _ => continue,
            };
            t.set(x, y, c);
        }
    }
    let ribs = bank.add(t);
    let solid = |bank: &mut TexBank, c: u8| bank.add(tiles::solid(c));
    let (bone, dark, heart) = (solid(bank, WHITE), solid(bank, INK), solid(bank, LIME));
    let mut m = crate::render::Mesh::new();
    lathe(
        &mut m,
        Vec3::ZERO,
        &[
            (0.0, -0.24),
            (0.17, -0.2),
            (0.26, -0.06),
            (0.25, 0.08),
            (0.17, 0.2),
            (0.0, 0.25),
        ],
        8,
        0.39,
        ribs,
        false,
    );
    // The heart inside, glowing.
    lathe(
        &mut m,
        Vec3::new(0.0, -0.07, 0.0),
        &[
            (0.0, 0.0),
            (0.07, 0.03),
            (0.08, 0.08),
            (0.05, 0.13),
            (0.0, 0.14),
        ],
        6,
        0.0,
        heart,
        false,
    );
    // The spine, knob by knob over its back.
    for k in 0..6 {
        let z = -0.2 + k as f32 * 0.075;
        let y = 0.24 - (z * 2.8).powi(2) * 0.03;
        skin_box(
            &mut m,
            Vec3::new(-0.024, y, z - 0.022),
            Vec3::new(0.024, y + 0.045, z + 0.022),
            bone,
            &w4,
        );
    }
    // Its face: sockets with a red glow deep in them, and teeth.
    for sx in [-1.0f32, 1.0] {
        skin_box(
            &mut m,
            Vec3::new(sx * 0.085 - 0.04, 0.03, 0.2),
            Vec3::new(sx * 0.085 + 0.04, 0.11, 0.262),
            dark,
            &w4,
        );
        skin_box(
            &mut m,
            Vec3::new(sx * 0.085 - 0.02, 0.05, 0.255),
            Vec3::new(sx * 0.085 + 0.02, 0.085, 0.268),
            red,
            &w4,
        );
    }
    skin_box(
        &mut m,
        Vec3::new(-0.07, -0.08, 0.215),
        Vec3::new(0.07, -0.03, 0.26),
        dark,
        &w4,
    );
    for x in [-0.045f32, -0.015, 0.015, 0.045] {
        skin_box(
            &mut m,
            Vec3::new(x - 0.011, -0.055, 0.252),
            Vec3::new(x + 0.011, -0.03, 0.266),
            bone,
            &w4,
        );
    }
    // A tail of bony rays, fanned out behind.
    for a in [-0.55f32, 0.0, 0.55] {
        let mut ray = crate::render::Mesh::new();
        skin_box(
            &mut ray,
            Vec3::new(-0.012, -0.012, -0.17),
            Vec3::new(0.012, 0.012, 0.0),
            bone,
            &w4,
        );
        m.append(
            &ray,
            Mat4::from_translation(Vec3::new(0.0, 0.0, -0.22)) * Mat4::from_rotation_x(a),
        );
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
        // The sewers' sludge.
        [CLAY, RUST, MAROON],
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
    // Bright against each biome's floor: poison-dart pinks in the blue grotto, and so on.
    let frogs = [
        [LIME, GREEN, TEAL],
        [BLUSH, PINK, CRIMSON],
        [GOLD, ORANGE, RUST],
        [CREAM, GOLD, CLAY],
        [LIME, GREEN, TEAL],
        [MINT, AQUA, TEAL],
    ];
    let puffers = [
        [CREAM, SAND, KHAKI],
        [CREAM, GOLD, ORANGE],
        [BLUSH, PINK, CRIMSON],
        [CREAM, GOLD, ORANGE],
        [WHITE, WHITE, SKY],
        [CREAM, GOLD, CLAY],
    ];
    let mut s = FoeSkins {
        frog: vec![],
        frog_leg: vec![],
        puffer: vec![],
        slime: vec![],
        slime_core: vec![],
        slime_cols,
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
        s.slime_core.push(recolor(
            bank,
            &c.slime_core,
            &[c.slime_core_tex],
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
        let fr = ramp([LIME, GREEN, TEAL], frogs[b]);
        s.frog.push(recolor(bank, &c.frog, &[c.frog_tex], &fr));
        s.frog_leg
            .push(recolor(bank, &c.frog_leg, &[c.frog_tex], &fr));
        s.puffer.push(recolor(
            bank,
            &c.puffer,
            &[c.puffer_tex],
            &ramp([CREAM, SAND, KHAKI], puffers[b]),
        ));
    }
    // Down in the sewers: bats, frogs and puffers are nothing but bones, and the rest (who
    // don't live there, but just in case) are bleached or gone brown.
    // Evil eyes on them all: dark sockets with a red glow in them.
    let red = bank.add(tiles::solid(RED));
    let socket = bank.add(tiles::solid(INK));
    let ribs = bank.add(bone_tex(4, 4, 2));
    s.bat_body.push(
        c.bat_body
            .retexture(c.fur_tex, ribs)
            .retexture(c.bat_eye_tex, red),
    );
    let wing = bank.add(bone_wing());
    s.bat_wing.push(c.bat_wing.retexture(c.wing_tex, wing));
    let ribs = bank.add(bone_tex(16, 16, 2));
    let mut frog = c
        .frog
        .retexture(c.frog_tex, ribs)
        .retexture(c.white_tex, socket);
    let w4 = Texture::new(4, 4, 0);
    for sx in [-1.0f32, 1.0] {
        let e = glam::Vec3::new(sx * 0.12, 0.3, 0.1);
        models::skin_box(
            &mut frog,
            e + glam::Vec3::new(-0.026, -0.014, 0.083),
            e + glam::Vec3::new(0.026, 0.018, 0.09),
            red,
            &w4,
        );
    }
    s.frog.push(frog);
    s.frog_leg.push(c.frog_leg.retexture(c.frog_tex, ribs));
    s.puffer.push(bone_puffer(bank, red));
    let sludge = ramp([LIME, GREEN, TEAL], slime_cols[SEWER_LOOK]);
    s.slime
        .push(recolor(bank, &c.slime, &[c.slime_tex], &sludge));
    s.slime_core
        .push(recolor(bank, &c.slime_core, &[c.slime_core_tex], &sludge));
    s.shroom.push(recolor(bank, &c.shroom, &[c.cap_tex], &|x| {
        if x == RED { KHAKI } else { x }
    }));
    let bone = [WHITE, SAND, KHAKI];
    s.crab.push(recolor(
        bank,
        &c.crab,
        &[c.shell_tex],
        &ramp([MINT, AQUA, TEAL], bone),
    ));
    s.wisp.push(recolor(
        bank,
        &c.wisp,
        &[c.wisp_tex],
        &ramp([WHITE, MINT, AQUA], [CREAM, LIME, GREEN]),
    ));
    s.beetle.push(recolor(
        bank,
        &c.beetle,
        &[c.beetle_tex],
        &ramp([LAVENDER, PURPLE, GRAPE], bone),
    ));
    s.golem.push(recolor(
        bank,
        &c.golem,
        &[c.golem_tex],
        &ramp([KHAKI, ROSEWOOD, SHADOW], bone),
    ));
    s
}

/// Winter's ground, laid out like the lawn it covers.
pub struct SnowTex {
    pub grass: [TexId; 4],
    pub flowers: [TexId; 2],
    pub rounded: Vec<[TexId; 16]>,
    pub fillets: Vec<[TexId; 16]>,
    pub hedge_top: TexId,
}

/// Every road with its outer corners rounded into each lawn, and each lawn with the road
/// filling its inner corners: `[road * lawns + lawn][mask]`.
fn corners(
    bank: &mut TexBank,
    roads: &[TexId],
    lawns: &[TexId],
) -> (Vec<[TexId; 16]>, Vec<[TexId; 16]>) {
    let mut rounded = Vec::new();
    let mut fillets = Vec::new();
    for &road in roads {
        for &lawn in lawns {
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
    (rounded, fillets)
}

pub struct Assets {
    pub foes: FoeSkins,
    pub bank: TexBank,
    pub icons: HashMap<&'static str, TexId>,
    pub font: Font,
    /// The title screen's logo, largest first.
    pub logo: Vec<logo::Picture>,
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
    /// The ground under winter's snow.
    pub snow: SnowTex,
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
    /// The people of Bramblewick, in `folk::VILLAGERS` order.
    pub folk: Vec<Humanoid>,
    pub sprout: crate::render::Mesh,
    pub critters: Critters,
    pub props: Props,
    pub gear: GearArt,
    /// Furniture, rugs and pictures for the farmhouse.
    pub home: home_art::HomeArt,
    /// Zombies, goblins, skeletons, ghosts and bugs, in a look for every biome.
    pub monsters: monster_art::Monsters,
    /// Bombs, cracked floors, blasted holes and ropes.
    pub delve: delve_art::DelveArt,
    /// The cat, the jumping spider, its egg, and candy rocks.
    pub pets: pet_art::PetArt,
    /// Lantern snails, book-worms and their ink runes.
    pub deep: deep_art::DeepArt,
    /// The old sewers: walkways, brickwork, murky water and bridges.
    pub sewer: sewer_art::SewerArt,
}

impl Assets {
    pub fn new() -> Assets {
        let mut bank = TexBank::default();
        let mut icons = sprites::build(&mut bank);
        item_art::build(&mut bank, &mut icons);
        let gear = gear_art::build(&mut bank, &mut icons);
        quest_art::build(&mut bank, &mut icons);
        season_art::build(&mut bank, &mut icons);
        deep_art::icons(&mut bank, &mut icons);
        magic_art::build(&mut bank, &mut icons);
        fish_art::build(&mut bank, &mut icons);
        let home = home_art::build(&mut bank, &mut icons);
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
        let checker = bank.add(tiles::checker(CREAM, PEACH, SAND));
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
        let (rounded, fillets) = corners(&mut bank, &roads, &grass);
        // Winter lays snow wherever the grass grows.
        let snow_lawn = [
            bank.add(tiles::snow(1, false)),
            bank.add(tiles::snow(2, false)),
            bank.add(tiles::snow(3, false)),
            bank.add(tiles::snow(4, false)),
        ];
        let (snow_rounded, snow_fillets) = corners(&mut bank, &roads, &snow_lawn);
        let snow = SnowTex {
            grass: snow_lawn,
            flowers: [
                bank.add(tiles::snow(5, true)),
                bank.add(tiles::snow(6, true)),
            ],
            rounded: snow_rounded,
            fillets: snow_fillets,
            hedge_top: bank.add(tiles::snowy_hedge(26)),
        };

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
        let folk = crate::game::folk::VILLAGER_DEFS
            .iter()
            .map(|v| models::humanoid(&mut bank, &v.look))
            .collect();
        let sprout = models::sprout(&mut bank);
        let critters = models::critters(&mut bank);
        let props = models::props(&mut bank);
        let town = town_art::build(&mut bank, &icons, water[0]);
        let foes = foe_skins(&mut bank, &critters);
        let monsters = monster_art::build(&mut bank);
        let delve = delve_art::build(&mut bank);
        let pets = pet_art::build(&mut bank);
        let deep = deep_art::build(&mut bank);
        let sewer = sewer_art::build(&mut bank);
        Assets {
            foes,
            bank,
            icons,
            font: Font::regular(),
            logo: logo::logos(),
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
            snow,
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
            folk,
            sprout,
            critters,
            props,
            gear,
            home,
            monsters,
            delve,
            pets,
            deep,
            sewer,
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
