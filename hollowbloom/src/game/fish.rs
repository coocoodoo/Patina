//! Fishing: what lives in which water and when it bites, casting a line, and reeling it in.
//!
//! Hold the use button with a rod in hand to wind up a cast, let go to throw. When the
//! bobber dips, press use to hook it, then keep the fish inside the green bar by holding
//! (to rise) and letting go (to sink) until the catch meter fills.

use glam::{Mat4, Vec3};

use super::Io;
use super::dungeon::biome_for;
use super::gear::{Class, Stat};
use super::items::{Item, Stack};
use super::loot;
use super::play::Play;
use super::world::{Area, Floor, WATER_Y};
use crate::assets::models::{HERO, SHOULDER, SHOULDER_X};
use crate::audio::Sfx;
use crate::input::Action;
use crate::palette::*;

/// The waters you can fish in.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum Water {
    Pond,
    River,
    Cave,
    Grotto,
    Fungal,
    Lava,
    Frost,
    Ruins,
}

pub const WATERS: [Water; 8] = [
    Water::Pond,
    Water::River,
    Water::Cave,
    Water::Grotto,
    Water::Fungal,
    Water::Lava,
    Water::Frost,
    Water::Ruins,
];

impl Water {
    pub fn name(self) -> &'static str {
        [
            "the farm pond",
            "Bramblewick's stream",
            "the Mossy Burrows' pools",
            "the Crystal Grotto's pools",
            "the Fungal Hollow's pools",
            "the Ember Depths' lava",
            "the Frost Caverns' pools",
            "the Sunken Ruins' pools",
        ][self as usize]
    }

    /// A short name, for lists.
    pub fn short(self) -> &'static str {
        [
            "Farm pond",
            "Town stream",
            "Mossy Burrows",
            "Crystal Grotto",
            "Fungal Hollow",
            "Ember Depths",
            "Frost Caverns",
            "Sunken Ruins",
        ][self as usize]
    }
}

/// Body shapes, for the art.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Shape {
    /// A little darting minnow.
    Slim,
    /// Deep and rounded, with a spiky back fin.
    Round,
    /// An eel or a pike.
    Long,
    /// A ray: a wide diamond with a whip of a tail.
    Flat,
    /// Round, with spines.
    Puffer,
    /// Long flowing fins.
    Fancy,
    /// A big head with whiskers.
    Whiskers,
    /// The classic fish.
    Trout,
    /// Not a fish at all: a bell and trailing threads.
    Jelly,
    /// Claws and a tail.
    Cray,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Pattern {
    Plain,
    /// Bands from back to belly.
    Stripes,
    /// One bright stripe along the side.
    Band,
    /// Thin lines along the side.
    Lines,
    Spots,
    /// A pale belly.
    Belly,
    Speckle,
    /// Spots that glow in the dark.
    Glow,
    /// A skeleton showing through.
    Bones,
}

/// How a hooked fish swims about.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Moves {
    Calm,
    Dart,
    Sink,
    Float,
    Wild,
}

/// When a fish is about.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum When {
    Any,
    Day,
    Night,
    Rain,
}

impl When {
    pub fn label(self) -> &'static str {
        match self {
            When::Any => "Bites any time",
            When::Day => "Bites by day",
            When::Night => "Bites at night",
            When::Rain => "Bites in the rain",
        }
    }
}

pub struct FishDef {
    pub item: Item,
    pub water: &'static [Water],
    pub shape: Shape,
    /// Light, mid and dark body colours.
    pub colors: [u8; 3],
    pub fin: u8,
    pub pattern: Pattern,
    /// The pattern's colour.
    pub accent: u8,
    pub when: When,
    /// 0 common, 1 uncommon, 2 rare, 3 legendary.
    pub rarity: u8,
    /// How hard it fights, 1..10.
    pub level: u8,
    pub moves: Moves,
    /// Length range in centimetres.
    pub size: (u16, u16),
}

#[allow(clippy::too_many_arguments)]
const fn f(
    item: Item,
    water: &'static [Water],
    shape: Shape,
    colors: [u8; 3],
    fin: u8,
    pattern: Pattern,
    accent: u8,
    when: When,
    rarity: u8,
    level: u8,
    moves: Moves,
    size: (u16, u16),
) -> FishDef {
    FishDef {
        item,
        water,
        shape,
        colors,
        fin,
        pattern,
        accent,
        when,
        rarity,
        level,
        moves,
        size,
    }
}

use Moves::*;
use Pattern as Pt;
use Shape::*;
use Water as W;
use When::{Any, Day, Night, Rain};

pub static FISH: [FishDef; 41] = [
    // The farm pond.
    f(
        Item::SunnyMinnow,
        &[W::Pond, W::River],
        Slim,
        [CREAM, GOLD, CLAY],
        GOLD,
        Pt::Belly,
        WHITE,
        Any,
        0,
        1,
        Calm,
        (4, 9),
    ),
    f(
        Item::PondPerch,
        &[W::Pond],
        Round,
        [LIME, GREEN, TEAL],
        ORANGE,
        Pt::Stripes,
        DEEP_TEAL,
        Any,
        0,
        2,
        Dart,
        (12, 26),
    ),
    f(
        Item::Bluegill,
        &[W::Pond],
        Round,
        [SKY, BLUE, INDIGO],
        BLUE,
        Pt::Belly,
        GOLD,
        Day,
        0,
        2,
        Calm,
        (10, 22),
    ),
    f(
        Item::MudCarp,
        &[W::Pond],
        Whiskers,
        [SAND, KHAKI, ROSEWOOD],
        KHAKI,
        Pt::Speckle,
        SHADOW,
        Any,
        1,
        3,
        Sink,
        (25, 60),
    ),
    f(
        Item::LilyKoi,
        &[W::Pond],
        Fancy,
        [WHITE, CREAM, SAND],
        ORANGE,
        Pt::Spots,
        ORANGE,
        Day,
        2,
        5,
        Float,
        (30, 55),
    ),
    f(
        Item::BubbleGoby,
        &[W::Pond],
        Slim,
        [BLUSH, PINK, CRIMSON],
        PINK,
        Pt::Spots,
        WHITE,
        Rain,
        1,
        2,
        Float,
        (5, 12),
    ),
    f(
        Item::MoonlitCatfish,
        &[W::Pond],
        Whiskers,
        [LAVENDER, PURPLE, GRAPE],
        GRAPE,
        Pt::Glow,
        CREAM,
        Night,
        1,
        5,
        Sink,
        (35, 80),
    ),
    f(
        Item::GoldenCarp,
        &[W::Pond],
        Whiskers,
        [CREAM, GOLD, CLAY],
        GOLD,
        Pt::Glow,
        WHITE,
        Any,
        3,
        8,
        Wild,
        (60, 110),
    ),
    // Bramblewick's stream and pond.
    f(
        Item::BrookTrout,
        &[W::River],
        Trout,
        [PEACH, SALMON, RUST],
        SALMON,
        Pt::Spots,
        CREAM,
        Any,
        0,
        3,
        Dart,
        (18, 35),
    ),
    f(
        Item::RainbowTrout,
        &[W::River],
        Trout,
        [WHITE, SKY, SLATE],
        SKY,
        Pt::Band,
        PINK,
        Day,
        1,
        4,
        Dart,
        (25, 50),
    ),
    f(
        Item::RiverChub,
        &[W::River],
        Round,
        [SAND, KHAKI, SHADOW],
        CLAY,
        Pt::Plain,
        SAND,
        Any,
        0,
        2,
        Calm,
        (10, 24),
    ),
    f(
        Item::PebbleLoach,
        &[W::River],
        Long,
        [SAND, KHAKI, ROSEWOOD],
        SAND,
        Pt::Speckle,
        SHADOW,
        Any,
        0,
        3,
        Sink,
        (8, 16),
    ),
    f(
        Item::SilverDace,
        &[W::River],
        Slim,
        [WHITE, SKY, SLATE],
        SKY,
        Pt::Belly,
        WHITE,
        Day,
        0,
        2,
        Dart,
        (10, 20),
    ),
    f(
        Item::StripedBass,
        &[W::River],
        Trout,
        [WHITE, SAND, KHAKI],
        KHAKI,
        Pt::Lines,
        INDIGO,
        Any,
        1,
        5,
        Wild,
        (30, 70),
    ),
    f(
        Item::StormSalmon,
        &[W::River],
        Trout,
        [SALMON, CRIMSON, PLUM],
        CRIMSON,
        Pt::Speckle,
        CREAM,
        Rain,
        2,
        6,
        Float,
        (45, 90),
    ),
    f(
        Item::MoonfinEel,
        &[W::River],
        Long,
        [SKY, INDIGO, SLATE],
        CREAM,
        Pt::Glow,
        WHITE,
        Night,
        2,
        7,
        Wild,
        (50, 110),
    ),
    f(
        Item::BramblePike,
        &[W::River],
        Long,
        [LIME, GREEN, DEEP_TEAL],
        GREEN,
        Pt::Spots,
        LIME,
        Any,
        1,
        6,
        Dart,
        (40, 90),
    ),
    // The Mossy Burrows.
    f(
        Item::BlindCavefish,
        &[W::Cave, W::Ruins],
        Slim,
        [BLUSH, PEACH, SALMON],
        BLUSH,
        Pt::Plain,
        WHITE,
        Any,
        0,
        3,
        Calm,
        (8, 18),
    ),
    f(
        Item::MossyPike,
        &[W::Cave],
        Long,
        [LIME, TEAL, DEEP_TEAL],
        TEAL,
        Pt::Speckle,
        LIME,
        Any,
        1,
        5,
        Dart,
        (40, 80),
    ),
    f(
        Item::GlowwormEel,
        &[W::Cave, W::Fungal],
        Long,
        [MINT, TEAL, DEEP_TEAL],
        MINT,
        Pt::Glow,
        LIME,
        Any,
        2,
        6,
        Wild,
        (40, 90),
    ),
    f(
        Item::Crayfish,
        &[W::Cave, W::Pond],
        Cray,
        [SALMON, RED, MAROON],
        RED,
        Pt::Plain,
        SALMON,
        Any,
        0,
        2,
        Sink,
        (6, 14),
    ),
    // The Crystal Grotto.
    f(
        Item::CrystalTetra,
        &[W::Grotto],
        Slim,
        [WHITE, MINT, AQUA],
        MINT,
        Pt::Glow,
        WHITE,
        Any,
        0,
        3,
        Dart,
        (5, 10),
    ),
    f(
        Item::PrismGuppy,
        &[W::Grotto],
        Fancy,
        [SKY, LAVENDER, PURPLE],
        PINK,
        Pt::Stripes,
        BLUSH,
        Any,
        1,
        4,
        Float,
        (4, 9),
    ),
    f(
        Item::GeodePuffer,
        &[W::Grotto, W::Frost],
        Puffer,
        [SAND, KHAKI, ROSEWOOD],
        AQUA,
        Pt::Speckle,
        AQUA,
        Any,
        1,
        5,
        Calm,
        (15, 30),
    ),
    f(
        Item::DiamondRay,
        &[W::Grotto],
        Flat,
        [WHITE, SKY, BLUE],
        SKY,
        Pt::Speckle,
        WHITE,
        Any,
        2,
        7,
        Float,
        (40, 90),
    ),
    // The Fungal Hollow.
    f(
        Item::SporeSnapper,
        &[W::Fungal],
        Round,
        [LAVENDER, PURPLE, GRAPE],
        PINK,
        Pt::Spots,
        LIME,
        Any,
        0,
        4,
        Dart,
        (15, 30),
    ),
    f(
        Item::JellyAngelfish,
        &[W::Fungal],
        Fancy,
        [BLUSH, PINK, PURPLE],
        LAVENDER,
        Pt::Stripes,
        WHITE,
        Any,
        1,
        5,
        Float,
        (12, 25),
    ),
    f(
        Item::TruffleCatfish,
        &[W::Fungal],
        Whiskers,
        [CLAY, RUST, MAROON],
        SAND,
        Pt::Speckle,
        SAND,
        Any,
        2,
        7,
        Sink,
        (40, 90),
    ),
    f(
        Item::MoonJelly,
        &[W::Fungal, W::Ruins],
        Jelly,
        [WHITE, LAVENDER, BLUSH],
        LAVENDER,
        Pt::Glow,
        WHITE,
        Any,
        1,
        4,
        Float,
        (10, 30),
    ),
    // Lava in the Ember Depths.
    f(
        Item::LavaEel,
        &[W::Lava],
        Long,
        [GOLD, ORANGE, RED],
        RED,
        Pt::Glow,
        CREAM,
        Any,
        0,
        5,
        Wild,
        (40, 90),
    ),
    f(
        Item::MagmaGoby,
        &[W::Lava],
        Slim,
        [ORANGE, RED, MAROON],
        GOLD,
        Pt::Speckle,
        GOLD,
        Any,
        0,
        4,
        Dart,
        (6, 14),
    ),
    f(
        Item::CinderCarp,
        &[W::Lava],
        Whiskers,
        [ROSEWOOD, SHADOW, MAROON],
        RUST,
        Pt::Glow,
        ORANGE,
        Any,
        1,
        6,
        Sink,
        (30, 70),
    ),
    f(
        Item::PhoenixKoi,
        &[W::Lava],
        Fancy,
        [CREAM, GOLD, RED],
        ORANGE,
        Pt::Glow,
        WHITE,
        Any,
        3,
        9,
        Wild,
        (40, 80),
    ),
    // The Frost Caverns.
    f(
        Item::SnowSmelt,
        &[W::Frost],
        Slim,
        [WHITE, WHITE, SKY],
        SKY,
        Pt::Plain,
        WHITE,
        Any,
        0,
        3,
        Dart,
        (8, 18),
    ),
    f(
        Item::IcePike,
        &[W::Frost],
        Long,
        [WHITE, SKY, BLUE],
        SKY,
        Pt::Stripes,
        BLUE,
        Any,
        1,
        6,
        Dart,
        (40, 90),
    ),
    f(
        Item::FrostChar,
        &[W::Frost],
        Trout,
        [PEACH, SALMON, INDIGO],
        SALMON,
        Pt::Spots,
        WHITE,
        Any,
        1,
        5,
        Calm,
        (25, 55),
    ),
    f(
        Item::AuroraTrout,
        &[W::Frost],
        Trout,
        [MINT, LAVENDER, INDIGO],
        MINT,
        Pt::Band,
        PINK,
        Night,
        2,
        8,
        Wild,
        (30, 65),
    ),
    // The Sunken Ruins.
    f(
        Item::GhostFish,
        &[W::Ruins],
        Slim,
        [WHITE, BLUSH, LAVENDER],
        WHITE,
        Pt::Glow,
        WHITE,
        Any,
        0,
        4,
        Float,
        (10, 25),
    ),
    f(
        Item::BoneFish,
        &[W::Ruins],
        Long,
        [WHITE, SAND, KHAKI],
        SAND,
        Pt::Bones,
        KHAKI,
        Any,
        1,
        5,
        Dart,
        (30, 70),
    ),
    f(
        Item::Coelacanth,
        &[W::Ruins],
        Trout,
        [SLATE, INDIGO, SHADOW],
        SLATE,
        Pt::Spots,
        WHITE,
        Any,
        2,
        8,
        Sink,
        (80, 180),
    ),
    f(
        Item::StarSturgeon,
        &[W::Ruins],
        Long,
        [INDIGO, SLATE, SHADOW],
        GOLD,
        Pt::Spots,
        GOLD,
        Night,
        3,
        9,
        Wild,
        (100, 220),
    ),
];

/// A fish's details, if the item is a fish.
pub fn fish_def(item: Item) -> Option<&'static FishDef> {
    FISH.iter().find(|d| d.item == item)
}

pub fn rarity_name(r: u8) -> &'static str {
    ["Common", "Uncommon", "Rare", "Legendary"][r.min(3) as usize]
}

pub fn rarity_color(r: u8) -> u8 {
    [WHITE, LIME, SKY, GOLD][r.min(3) as usize]
}

/// Rods that shrug off lava.
pub fn heatproof(item: Item) -> bool {
    matches!(item, Item::EmberRod | Item::StarRod | Item::LeviathanRod)
}

/// How far the rod reaches beyond the hand.
pub const ROD_LEN: f32 = 1.05;

/// Where the tip of a rod is, held by the hero at `pos` facing `yaw` with the arm raised by
/// `lift` (as in `Swing::Fish`).
pub fn rod_tip(pos: Vec3, yaw: f32, lift: f32) -> Vec3 {
    let s = HERO.scale;
    let root = Mat4::from_translation(pos) * Mat4::from_rotation_y(yaw);
    let right = root
        * Mat4::from_translation(Vec3::new(SHOULDER_X * s, SHOULDER * s, 0.0))
        * Mat4::from_rotation_x(-1.5 - lift * 1.5);
    let hand = right * Mat4::from_translation(Vec3::new(0.0, -0.22, 0.02));
    hand.transform_point3(Vec3::new(0.0, -ROD_LEN, 0.0))
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Phase {
    /// Winding up: hold to build power.
    Charge,
    /// The bobber sailing out.
    Fly,
    /// Bobbing on the water.
    Wait,
    /// A bite! Hook it quickly.
    Bite,
    /// Reeling it in.
    Reel,
    /// Holding up the catch.
    Caught,
    /// Winding the line back in.
    Reeled,
}

/// What's on the end of the line.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Hooked {
    Fish(Item, u16),
    Junk(Item),
    Treasure,
}

pub struct Fishing {
    pub phase: Phase,
    pub t: f32,
    /// Cast power, 0..1.
    pub power: f32,
    /// Where the bobber is, and where it flew from and to.
    pub bob: Vec3,
    pub from: Vec3,
    pub to: Vec3,
    pub water: Option<Water>,
    pub bite_in: f32,
    pub nibble: f32,
    /// How far the bobber is pulled under (0..1).
    pub dip: f32,
    pub hooked: Option<Hooked>,
    /// The reeling game: the bar you steer (bottom edge, speed, height), the fish, and how
    /// close you are to landing it.
    pub zone: f32,
    pub zone_v: f32,
    pub zone_h: f32,
    pub fish_y: f32,
    pub fish_to: f32,
    pub fish_t: f32,
    pub progress: f32,
    pub perfect: bool,
    pub bait: bool,
    /// Something new for the journal (for the caught banner).
    pub new: bool,
    pub record: bool,
}

impl Fishing {
    fn new() -> Fishing {
        Fishing {
            phase: Phase::Charge,
            t: 0.0,
            power: 0.0,
            bob: Vec3::ZERO,
            from: Vec3::ZERO,
            to: Vec3::ZERO,
            water: None,
            bite_in: 0.0,
            nibble: 0.0,
            dip: 0.0,
            hooked: None,
            zone: 0.0,
            zone_v: 0.0,
            zone_h: 0.25,
            fish_y: 0.3,
            fish_to: 0.3,
            fish_t: 0.0,
            progress: 0.3,
            perfect: true,
            bait: false,
            new: false,
            record: false,
        }
    }

    /// The hero stands still while casting, reeling and showing off a catch.
    pub fn holds_still(&self) -> bool {
        matches!(
            self.phase,
            Phase::Charge | Phase::Fly | Phase::Bite | Phase::Reel | Phase::Caught
        )
    }

    /// How high the rod is held (see `Swing::Fish`).
    pub fn lift(&self, time: f32) -> f32 {
        match self.phase {
            Phase::Charge => 0.55 + self.power * 0.3,
            Phase::Fly => 1.0 - (self.t / FLY).min(1.0) * 0.72,
            Phase::Wait => 0.3 + (time * 1.3).sin() * 0.015,
            Phase::Bite => 0.5,
            Phase::Reel => 0.42 + (time * 23.0).sin() * 0.03 + self.zone_v.max(0.0) * 0.06,
            Phase::Caught => 0.7,
            Phase::Reeled => 0.3 + (self.t / REEL_IN).min(1.0) * 0.5,
        }
    }

    /// Where the bobber is drawn right now.
    pub fn bobber(&self, time: f32) -> Vec3 {
        match self.phase {
            Phase::Fly => {
                let k = (self.t / FLY).min(1.0);
                let mut p = self.from.lerp(self.to, k);
                p.y += (k * std::f32::consts::PI).sin() * (0.8 + self.power * 0.8);
                p
            }
            Phase::Reeled => {
                let k = (self.t / REEL_IN).min(1.0);
                self.bob.lerp(self.from, k)
            }
            Phase::Wait | Phase::Bite | Phase::Reel => {
                let wobble = (time * 2.6).sin() * 0.015;
                let jig = if self.phase == Phase::Reel {
                    Vec3::new((time * 17.0).sin() * 0.05, 0.0, (time * 13.0).cos() * 0.05)
                } else {
                    Vec3::ZERO
                };
                self.bob + jig + Vec3::Y * (wobble - self.dip * 0.08)
            }
            _ => self.from,
        }
    }
}

/// Seconds the bobber flies, the window to hook a bite, and winding back in.
const FLY: f32 = 0.45;
const BITE: f32 = 0.95;
const REEL_IN: f32 = 0.3;
const CAUGHT: f32 = 1.5;

impl Play {
    /// Which water a tile holds, if any.
    pub fn water_at(&self, x: i32, z: i32) -> Option<Water> {
        let f = self.world().floor(x, z);
        match (self.area, f) {
            (Area::Farm, Floor::Water) => Some(Water::Pond),
            (Area::Town, Floor::Water) => Some(Water::River),
            (Area::Hollow { depth }, Floor::Water) => Some(match biome_for(depth) {
                0 => Water::Cave,
                1 => Water::Grotto,
                2 => Water::Fungal,
                4 => Water::Frost,
                _ => Water::Ruins,
            }),
            (Area::Hollow { .. }, Floor::Lava) => Some(Water::Lava),
            _ => None,
        }
    }

    /// Reel power of the rod in hand.
    fn reel_power(&self) -> i32 {
        self.player
            .held_stack()
            .filter(|s| s.item.class() == Some(Class::Rod))
            .and_then(|s| s.main_value())
            .unwrap_or(8)
    }

    /// Everything to do with the rod: called every frame while one is in hand or a line is
    /// out. Returns true while fishing takes over the use button.
    pub fn update_fishing(&mut self, io: &mut Io) -> bool {
        let dt = io.dt;
        let input = io.input;
        let rod = self.player.held_class() == Some(Class::Rod);
        if !rod {
            if self.fishing.is_some() {
                self.fishing = None;
            }
            return false;
        }
        let down = input.down(Action::Use);
        let pressed = input.pressed(Action::Use);
        let moving = input.move_axis().length_squared() > 0.0;
        let Some(fsh) = &mut self.fishing else {
            if pressed && self.player.act.is_none() && self.player.dodge <= 0.0 {
                if self.area == Area::Home || matches!(self.area, Area::Inside(_)) {
                    if self.nag <= 0.0 {
                        self.nag = 2.0;
                        self.toast("No fish in here! Try the pond.", None, 0);
                        io.audio.play(Sfx::Denied);
                    }
                    return true;
                }
                self.fishing = Some(Fishing::new());
            }
            return pressed;
        };
        fsh.t += dt;
        match fsh.phase {
            Phase::Charge => {
                // Power swings up and down while you hold.
                let k = (fsh.t * 1.1).fract();
                fsh.power = if k < 0.5 { k * 2.0 } else { 2.0 - k * 2.0 };
                if !down {
                    self.cast(io);
                }
            }
            Phase::Fly => {
                if fsh.t >= FLY {
                    self.land(io);
                }
            }
            Phase::Wait => {
                fsh.dip = (fsh.dip - dt * 3.0).max(0.0);
                fsh.bite_in -= dt;
                fsh.nibble -= dt;
                if fsh.nibble <= 0.0 && fsh.bite_in > 0.8 {
                    fsh.nibble = 0.9 + self.rng.f32() * 2.2;
                    fsh.dip = 0.5;
                    io.audio.play_at(Sfx::Plop, 0.25, 1.4);
                }
                if pressed || moving {
                    self.reel_in();
                } else if fsh.bite_in <= 0.0 {
                    self.bite(io);
                }
            }
            Phase::Bite => {
                fsh.dip = 1.0;
                if pressed {
                    self.hook(io);
                } else if fsh.t > BITE || moving {
                    self.toast("It got away...", None, 0);
                    self.reel_in();
                }
            }
            Phase::Reel => {
                self.reel_game(dt, down, io);
            }
            Phase::Caught => {
                if fsh.t > CAUGHT || (pressed && fsh.t > 0.4) {
                    self.fishing = None;
                }
            }
            Phase::Reeled => {
                if fsh.t > REEL_IN {
                    self.fishing = None;
                }
            }
        }
        true
    }

    /// Lets the line fly.
    fn cast(&mut self, io: &mut Io) {
        let p = &mut self.player;
        let cost = p.tool_cost(1.2);
        p.energy = (p.energy - cost).max(-10.0);
        let tip = rod_tip(p.world_pos(), p.yaw, 1.0);
        let dir = self.player.facing.normalize_or_zero();
        let Some(fsh) = &mut self.fishing else { return };
        let reach = 1.4 + fsh.power * 3.3;
        let mut to = self.player.pos + dir * reach;
        // Land in the farthest water along the line, so a short throw still finds the pond.
        let mut water = None;
        for k in (2..=((reach / 0.25) as i32)).rev() {
            let q = self.player.pos + dir * (k as f32 * 0.25);
            let (tx, tz) = (q.x.floor() as i32, q.y.floor() as i32);
            if let Some(w) = self.water_at(tx, tz) {
                water = Some(w);
                to = q;
                break;
            }
        }
        let y = match water {
            Some(Water::Lava) => -0.1,
            Some(_) => WATER_Y + 0.04,
            None => 0.05,
        };
        let Some(fsh) = &mut self.fishing else { return };
        fsh.phase = Phase::Fly;
        fsh.t = 0.0;
        fsh.from = tip;
        fsh.to = Vec3::new(to.x, y, to.y);
        fsh.bob = fsh.to;
        fsh.water = water;
        io.audio.play_at(Sfx::Cast, 0.8, 0.9 + fsh.power * 0.3);
    }

    /// The bobber comes down: in water, or with a sad thud on land.
    fn land(&mut self, io: &mut Io) {
        let heat = self.player.held().is_some_and(heatproof);
        let lure = self.player.sheet.frac(Stat::Lure, 60);
        let rain = self.rain && matches!(self.area, Area::Farm | Area::Town);
        let bait = self.player.inv.count(Item::Bait) > 0;
        let wait = 2.4 + self.rng.f32() * 5.5;
        let Some(fsh) = &mut self.fishing else { return };
        fsh.bob = fsh.to;
        match fsh.water {
            None => {
                fsh.phase = Phase::Reeled;
                fsh.t = 0.0;
                io.audio.play(Sfx::Place);
                self.toast("Nothing bites on dry land!", None, 0);
            }
            Some(Water::Lava) if !heat => {
                fsh.phase = Phase::Reeled;
                fsh.t = 0.0;
                io.audio.play(Sfx::Denied);
                self.fx
                    .burst(fsh.to + Vec3::Y * 0.1, 8, &[GOLD, ORANGE, RED], 1.5, 1.2);
                self.toast(
                    "Your line would burn! Lava needs a heat-proof rod.",
                    None,
                    0,
                );
            }
            Some(w) => {
                fsh.phase = Phase::Wait;
                fsh.t = 0.0;
                fsh.bait = bait;
                fsh.bite_in = wait
                    * (1.0 - lure * 0.8)
                    * if bait { 0.5 } else { 1.0 }
                    * if rain { 0.8 } else { 1.0 };
                fsh.nibble = 0.8 + self.rng.f32();
                let splash = if w == Water::Lava {
                    [GOLD, ORANGE, RED]
                } else {
                    [WHITE, SKY, AQUA]
                };
                let at = fsh.to;
                if bait {
                    self.player.inv.take(Item::Bait, 1);
                }
                io.audio.play_at(Sfx::Plop, 0.7, 1.0);
                self.fx.burst(at + Vec3::Y * 0.05, 10, &splash, 1.8, 2.4);
            }
        }
    }

    /// Something takes the bait.
    fn bite(&mut self, io: &mut Io) {
        let Some(water) = self.fishing.as_ref().and_then(|f| f.water) else {
            return;
        };
        let hooked = self.roll_catch(water);
        let Some(fsh) = &mut self.fishing else { return };
        fsh.hooked = Some(hooked);
        fsh.phase = Phase::Bite;
        fsh.t = 0.0;
        let at = fsh.bob;
        io.audio.play(Sfx::Bite);
        self.fx
            .popup(self.player.world_pos() + Vec3::Y * 1.6, "!", GOLD);
        self.fx
            .burst(at + Vec3::Y * 0.05, 8, &[WHITE, SKY], 1.6, 2.0);
    }

    /// Hooked it: junk comes straight up, fish need reeling.
    fn hook(&mut self, io: &mut Io) {
        let reel = self.reel_power();
        let Some(fsh) = &mut self.fishing else { return };
        match fsh.hooked {
            Some(Hooked::Junk(_)) | Some(Hooked::Treasure) | None => {
                io.audio.play(Sfx::Reel);
                self.land_catch(io);
            }
            Some(Hooked::Fish(item, _)) => {
                let lv = fish_def(item).map_or(3, |d| d.level);
                fsh.phase = Phase::Reel;
                fsh.t = 0.0;
                fsh.zone_h =
                    (0.18 + reel as f32 / 250.0).clamp(0.18, 0.45) + 0.04 - lv as f32 * 0.006;
                fsh.zone = 0.0;
                fsh.zone_v = 0.0;
                fsh.fish_y = 0.25;
                fsh.fish_to = 0.4;
                fsh.fish_t = 0.4;
                fsh.progress = 0.3;
                fsh.perfect = true;
                io.audio.play(Sfx::Reel);
            }
        }
    }

    /// The reeling game, one frame of it.
    fn reel_game(&mut self, dt: f32, down: bool, io: &mut Io) {
        let line = self.player.sheet.frac(Stat::Line, 50);
        let haste = self.player.sheet.frac(Stat::Haste, 60);
        let r1 = self.rng.f32();
        let r2 = self.rng.f32();
        let hurt = self.player.hurt > 0.3;
        let Some(fsh) = &mut self.fishing else { return };
        let Some(Hooked::Fish(item, _)) = fsh.hooked else {
            return;
        };
        let def = fish_def(item);
        let level = def.map_or(3, |d| d.level) as f32;
        let moves = def.map_or(Moves::Calm, |d| d.moves);
        // You steer the bar: hold to rise, let go to sink.
        fsh.zone_v += if down { 2.6 } else { -2.1 } * dt;
        fsh.zone_v = fsh.zone_v.clamp(-1.5, 1.5);
        fsh.zone += fsh.zone_v * dt;
        let top = 1.0 - fsh.zone_h;
        if fsh.zone < 0.0 {
            fsh.zone = 0.0;
            fsh.zone_v = -fsh.zone_v * 0.3;
        }
        if fsh.zone > top {
            fsh.zone = top;
            fsh.zone_v = 0.0;
        }
        // The fish picks somewhere new to swim now and then.
        fsh.fish_t -= dt;
        if fsh.fish_t <= 0.0 {
            let busy = (1.25 - level * 0.07).max(0.45);
            let (to, wait) = match moves {
                Moves::Calm => ((fsh.fish_y + (r1 - 0.5) * 0.5), 0.8 + r2 * 0.9),
                Moves::Dart => (r1, 0.5 + r2 * 0.7),
                Moves::Sink => (r1 * r1 * 0.8, 0.6 + r2 * 0.8),
                Moves::Float => (1.0 - r1 * r1 * 0.8, 0.6 + r2 * 0.8),
                Moves::Wild => (r1, 0.25 + r2 * 0.4),
            };
            fsh.fish_to = to.clamp(0.0, 1.0);
            fsh.fish_t = wait * busy;
        }
        let speed = (0.22 + level * 0.09)
            * match moves {
                Moves::Calm => 0.7,
                Moves::Dart => 1.5,
                Moves::Wild => 1.35,
                _ => 1.0,
            };
        let d = fsh.fish_to - fsh.fish_y;
        fsh.fish_y += d.clamp(-speed * dt, speed * dt);
        let inside = fsh.fish_y >= fsh.zone - 0.02 && fsh.fish_y <= fsh.zone + fsh.zone_h + 0.02;
        if inside {
            fsh.progress += 0.3 * (1.0 + haste) * dt;
        } else {
            fsh.progress -= (0.2 + level * 0.012) * (1.0 - line) * dt;
            fsh.perfect = false;
        }
        if hurt {
            fsh.progress = 0.0;
        }
        if fsh.progress >= 1.0 {
            self.land_catch(io);
        } else if fsh.progress <= 0.0 {
            io.audio.play(Sfx::Denied);
            self.toast("The line went slack... it got away!", None, 0);
            self.reel_in();
        }
    }

    fn reel_in(&mut self) {
        if let Some(fsh) = &mut self.fishing {
            fsh.phase = Phase::Reeled;
            fsh.t = 0.0;
            fsh.hooked = None;
        }
    }

    /// What bites: once in a while sunken treasure or junk, mostly fish of this water.
    fn roll_catch(&mut self, water: Water) -> Hooked {
        let treasure = self.player.sheet.frac(Stat::Treasure, 35);
        let angler = self.player.sheet.frac(Stat::Angler, 40);
        let luck = self.player.luck();
        if self.rng.chance(0.03 + treasure * 0.8 + luck * 0.02) {
            return Hooked::Treasure;
        }
        if water != Water::Lava && self.rng.chance(0.07) {
            let junk = [Item::Seaweed, Item::Seaweed, Item::SoggyBoot, Item::TinCan];
            return Hooked::Junk(junk[self.rng.below(junk.len())]);
        }
        let night = self.sky_night() > 0.5;
        let rain = self.rain;
        let base = [10.0, 4.0, 1.3, 0.3];
        let mut items = Vec::new();
        let mut weights = Vec::new();
        let bait = self.fishing.as_ref().is_some_and(|f| f.bait);
        for d in FISH.iter() {
            if !d.water.contains(&water) {
                continue;
            }
            let ok = match d.when {
                When::Any => true,
                When::Day => !night,
                When::Night => night,
                When::Rain => rain,
            };
            if !ok {
                continue;
            }
            let r = d.rarity as usize;
            let mut w = base[r];
            if r >= 1 {
                w *= 1.0 + angler * 2.5 + if bait { 0.2 } else { 0.0 };
            }
            if r >= 2 {
                w *= 1.0 + luck * 0.6;
            }
            items.push(d);
            weights.push(w);
        }
        if items.is_empty() {
            return Hooked::Junk(Item::Seaweed);
        }
        let d = items[self.rng.weighted(&weights)];
        let (lo, hi) = d.size;
        let t = self.rng.f32().powf(1.6 / (1.0 + luck));
        let cm = lo as f32 + (hi - lo) as f32 * t;
        Hooked::Fish(d.item, cm.round().max(1.0) as u16)
    }

    /// Up it comes! Into the bag, the journal and the record books.
    fn land_catch(&mut self, io: &mut Io) {
        let Some(fsh) = &self.fishing else { return };
        let hooked = fsh.hooked;
        let perfect = fsh.perfect && fsh.phase == Phase::Reel;
        let water = fsh.water;
        let at = self.player.world_pos() + Vec3::Y * 1.4;
        let mut new = false;
        let mut record = false;
        match hooked {
            Some(Hooked::Fish(item, cm)) => {
                let cm = if perfect {
                    let top = fish_def(item).map_or(cm, |d| d.size.1);
                    ((cm as f32 * 1.2) as u16).min(top + top / 10)
                } else {
                    cm
                };
                self.give(Stack::new(item, 1));
                (new, record) = self.on_catch(item, cm);
                let def = fish_def(item);
                let col = def.map_or(WHITE, |d| rarity_color(d.rarity));
                self.toast_colored(
                    format!("Caught: {} ({cm} cm)", item.def().name),
                    Some(item),
                    0,
                    col,
                );
                if perfect {
                    self.fx.popup_big(at + Vec3::Y * 0.3, "PERFECT!", GOLD);
                }
                if def.is_some_and(|d| d.rarity >= 2) {
                    io.audio.play(Sfx::Rare);
                } else {
                    io.audio.play(Sfx::Catch);
                }
                if let Some(f) = &mut self.fishing {
                    f.hooked = Some(Hooked::Fish(item, cm));
                }
            }
            Some(Hooked::Junk(item)) => {
                self.give(Stack::new(item, 1));
                self.toast(format!("Fished up: {}", item.def().name), Some(item), 0);
                io.audio.play(Sfx::Pickup);
            }
            Some(Hooked::Treasure) => {
                let depth = match self.area {
                    Area::Hollow { depth } => depth,
                    _ => (self.deepest / 2).max(1),
                };
                let f = self.fortune();
                let loot =
                    loot::sunken_treasure(depth, water == Some(Water::Lava), f, &mut self.rng);
                let from = self.fishing.as_ref().map_or(at, |f| f.bob);
                self.spill(loot, from, io);
                io.audio.play(Sfx::Chest);
                self.fx
                    .motes(from + Vec3::Y * 0.3, 18, &[GOLD, CREAM, WHITE], 0.5);
                self.toast_colored("A sunken treasure chest!", None, 0, GOLD);
            }
            None => {}
        }
        if let Some(f) = &mut self.fishing {
            f.phase = Phase::Caught;
            f.t = 0.0;
            f.new = new;
            f.record = record;
            if !matches!(hooked, Some(Hooked::Fish(..))) {
                f.phase = Phase::Reeled;
            }
        }
        self.fx.burst(at, 12, &[WHITE, SKY, AQUA], 2.0, 2.0);
    }
}

/// A rod's line colour: plain white, or its rarity's.
pub fn line_color(s: Option<&Stack>) -> u8 {
    match s.and_then(|s| s.rarity()) {
        Some(r) if r > super::gear::Rarity::Common => r.color(),
        _ => WHITE,
    }
}

/// Points along a fishing line from the rod tip to the bobber, sagging in the middle.
pub fn line_points(tip: Vec3, bob: Vec3, sag: f32, n: usize) -> Vec<Vec3> {
    let mid = (tip + bob) * 0.5 - Vec3::Y * sag;
    (0..=n)
        .map(|i| {
            let t = i as f32 / n as f32;
            let a = tip.lerp(mid, t);
            let b = mid.lerp(bob, t);
            a.lerp(b, t)
        })
        .collect()
}

/// Where the fish sits in the reel bar, for drawing: (fish, zone bottom, zone height).
pub fn reel_bar(f: &Fishing) -> (f32, f32, f32) {
    (f.fish_y, f.zone, f.zone_h)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::items::Kind;

    #[test]
    fn every_fish_is_a_fish_item_and_every_water_has_fish() {
        let mut seen = std::collections::HashSet::new();
        for d in FISH.iter() {
            assert_eq!(d.item.def().kind, Kind::Fish, "{:?}", d.item);
            assert!(seen.insert(d.item), "{:?} listed twice", d.item);
            assert!(d.size.0 < d.size.1);
            assert!((1..=10).contains(&d.level));
            assert!(!d.water.is_empty());
        }
        for it in crate::game::items::ALL_ITEMS {
            if it.def().kind == Kind::Fish {
                assert!(fish_def(*it).is_some(), "{it:?} has no fish entry");
            }
        }
        for w in WATERS {
            let n = FISH.iter().filter(|d| d.water.contains(&w)).count();
            assert!(n >= 4, "{w:?} has only {n} fish");
            assert!(
                FISH.iter()
                    .any(|d| d.water.contains(&w) && d.when == When::Any && d.rarity == 0),
                "{w:?} needs an everyday fish"
            );
        }
        assert!(FISH.len() >= 40);
    }
}
