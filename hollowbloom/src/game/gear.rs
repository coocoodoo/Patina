//! Gear: weapons, armour and farming tools. Every piece carries a level, a rolled base
//! value, random stats (affixes) and up to three enchantments from scrolls, and its rarity is
//! worked out from how good all of those rolls are. Scrolls carry one enchantment each; their
//! rarity comes from how strong that enchantment rolled. Armour can also be forged on the
//! anvil, other pieces melted into it: it gathers forge experience, and every forge level
//! makes its stats a little stronger.

use crate::palette::*;
use crate::util::Rng;

// ------------------------------------------------------------------------------------------
// Stats
// ------------------------------------------------------------------------------------------

/// Every stat gear can roll. Stats from everything worn (and the item in hand) add up.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, PartialOrd, Ord)]
pub enum Stat {
    Damage,
    Crit,
    CritDmg,
    Haste,
    Lifesteal,
    Burn,
    Chill,
    Shock,
    Defense,
    Vitality,
    Stamina,
    Wisdom,
    Regen,
    Swift,
    Dodge,
    Block,
    Thorns,
    Luck,
    Greed,
    Power,
    Frugal,
    Reach,
    Capacity,
    Bounty,
    Growth,
    Forage,
    Focus,
    Spirit,
    /// Fish bite sooner.
    Lure,
    /// A hooked fish slips away more slowly.
    Line,
    /// More sunken treasure on the line.
    Treasure,
    /// Rarer fish take the bait.
    Angler,
}

pub const STATS: usize = 32;

pub const ALL_STATS: [Stat; STATS] = [
    Stat::Damage,
    Stat::Crit,
    Stat::CritDmg,
    Stat::Haste,
    Stat::Lifesteal,
    Stat::Burn,
    Stat::Chill,
    Stat::Shock,
    Stat::Defense,
    Stat::Vitality,
    Stat::Stamina,
    Stat::Wisdom,
    Stat::Regen,
    Stat::Swift,
    Stat::Dodge,
    Stat::Block,
    Stat::Thorns,
    Stat::Luck,
    Stat::Greed,
    Stat::Power,
    Stat::Frugal,
    Stat::Reach,
    Stat::Capacity,
    Stat::Bounty,
    Stat::Growth,
    Stat::Forage,
    Stat::Focus,
    Stat::Spirit,
    Stat::Lure,
    Stat::Line,
    Stat::Treasure,
    Stat::Angler,
];

/// How a stat's number is written.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Fmt {
    /// "+12 Damage"
    Plus,
    /// "+8% Crit Chance"
    Pct,
    /// "-10% Mana Cost"
    Less,
}

pub struct StatDef {
    pub key: &'static str,
    pub name: &'static str,
    pub fmt: Fmt,
    /// Weakest roll at item level `l`: `lo.0 + lo.1 * l`.
    pub lo: (f32, f32),
    /// Strongest roll at item level `l`: `hi.0 + hi.1 * l`.
    pub hi: (f32, f32),
    /// No single roll goes above this.
    pub cap: f32,
    /// Text colour on dark tooltips.
    pub color: u8,
    /// Scroll title: "Scroll of Embers".
    pub title: &'static str,
    pub about: &'static str,
}

macro_rules! stat_defs {
    ($( $key:literal, $name:literal, $fmt:ident, ($l0:expr, $l1:expr), ($h0:expr, $h1:expr), $cap:expr, $col:expr, $title:literal, $about:literal; )*) => {
        static STAT_DEFS: [StatDef; STATS] = [
            $( StatDef { key: $key, name: $name, fmt: Fmt::$fmt, lo: ($l0, $l1), hi: ($h0, $h1), cap: $cap, color: $col, title: $title, about: $about } ),*
        ];
    };
}

stat_defs! {
    "damage", "Damage", Plus, (1.0, 0.25), (3.0, 0.7), 9999.0, SALMON, "Might", "Added to every hit you land.";
    "crit", "Crit Chance", Pct, (2.0, 0.02), (5.0, 0.1), 20.0, GOLD, "Stars", "Chance for a hit to be critical.";
    "crit_dmg", "Crit Damage", Pct, (8.0, 0.2), (25.0, 0.6), 120.0, ORANGE, "Pouncing", "Extra damage on critical hits.";
    "haste", "Speed", Pct, (3.0, 0.03), (7.0, 0.1), 25.0, SKY, "Hummingbirds", "Swing, cast and work faster.";
    "lifesteal", "Heartsip", Pct, (1.0, 0.02), (3.0, 0.05), 10.0, PINK, "Heartsip", "Heals a share of the damage you deal.";
    "burn", "Burn Chance", Pct, (4.0, 0.05), (10.0, 0.2), 35.0, ORANGE, "Embers", "Chance to set foes aflame.";
    "chill", "Chill Chance", Pct, (4.0, 0.05), (10.0, 0.2), 35.0, SKY, "Snowflakes", "Chance to slow foes down.";
    "shock", "Shock Chance", Pct, (4.0, 0.05), (10.0, 0.2), 35.0, CREAM, "Sparks", "Chance to zap foes nearby.";
    "defense", "Defense", Plus, (1.0, 0.15), (2.0, 0.4), 9999.0, SKY, "the Turtle", "Softens every blow you take.";
    "vitality", "Max HP", Plus, (3.0, 0.6), (8.0, 1.8), 9999.0, PINK, "Hearts", "More health.";
    "stamina", "Max Energy", Plus, (4.0, 0.4), (10.0, 1.2), 9999.0, GOLD, "Sunshine", "More energy for a long day.";
    "wisdom", "Max Mana", Plus, (4.0, 0.4), (10.0, 1.2), 9999.0, LAVENDER, "Owls", "More mana for wands and staffs.";
    "regen", "HP Regen", Plus, (1.0, 0.03), (2.0, 0.12), 99.0, LIME, "Mending", "Health restored every few seconds.";
    "swift", "Move Speed", Pct, (3.0, 0.03), (7.0, 0.1), 25.0, MINT, "Bunnies", "Walk faster.";
    "dodge", "Dodge", Pct, (2.0, 0.02), (5.0, 0.08), 20.0, AQUA, "Butterflies", "Chance to slip past a hit.";
    "block", "Block", Pct, (3.0, 0.04), (8.0, 0.15), 30.0, SKY, "the Hedgehog", "Chance to block a hit completely.";
    "thorns", "Thorns", Plus, (1.0, 0.2), (4.0, 0.6), 9999.0, LIME, "Brambles", "Hurts whatever hurts you.";
    "luck", "Luck", Pct, (3.0, 0.05), (8.0, 0.2), 40.0, LIME, "Clover", "Better and rarer loot.";
    "greed", "Coin Find", Pct, (5.0, 0.1), (15.0, 0.3), 80.0, GOLD, "Magpies", "More coins from everything.";
    "power", "Tool Power", Plus, (1.0, 0.03), (2.0, 0.1), 99.0, PEACH, "Moles", "Break rocks and trees faster.";
    "frugal", "Tool Energy", Less, (5.0, 0.1), (12.0, 0.25), 50.0, CREAM, "Cozy Naps", "Tools cost less energy.";
    "reach", "Tool Reach", Plus, (1.0, 0.0), (1.0, 0.04), 3.0, MINT, "Wide Fields", "Hoes, cans and sickles cover more tiles.";
    "capacity", "Water", Plus, (5.0, 0.3), (15.0, 1.0), 9999.0, SKY, "Rainclouds", "The can holds more water.";
    "bounty", "Bounty", Pct, (5.0, 0.1), (12.0, 0.3), 60.0, ORANGE, "Plenty", "Chance of extra crops, wood and ore.";
    "growth", "Growth", Pct, (3.0, 0.05), (8.0, 0.15), 30.0, LIME, "Sprouting", "Watered crops may grow an extra day.";
    "forage", "Forage", Pct, (4.0, 0.08), (10.0, 0.2), 40.0, GREEN, "Squirrels", "Find seeds and gems while you work.";
    "focus", "Mana Cost", Less, (4.0, 0.05), (10.0, 0.15), 40.0, LAVENDER, "Moonbeams", "Spells cost less mana.";
    "spirit", "Mana Regen", Plus, (1.0, 0.02), (2.0, 0.06), 99.0, AQUA, "Dewdrops", "Mana comes back faster.";
    "lure", "Bite Speed", Pct, (5.0, 0.1), (12.0, 0.3), 60.0, MINT, "the Heron", "Fish bite sooner.";
    "line", "Line Strength", Pct, (4.0, 0.08), (10.0, 0.25), 50.0, SKY, "Steady Hands", "A hooked fish slips away more slowly.";
    "treasure", "Treasure Find", Pct, (3.0, 0.05), (8.0, 0.15), 35.0, GOLD, "Sunken Gold", "More sunken chests on the line.";
    "angler", "Rare Fish", Pct, (3.0, 0.05), (8.0, 0.15), 40.0, AQUA, "the Otter", "Rarer fish take the bait.";
}

impl Stat {
    pub fn def(self) -> &'static StatDef {
        &STAT_DEFS[self as usize]
    }

    pub fn from_key(key: &str) -> Option<Stat> {
        ALL_STATS.iter().copied().find(|s| s.def().key == key)
    }

    /// The weakest and strongest affix roll at an item level.
    pub fn range(self, level: u16) -> (f32, f32) {
        let d = self.def();
        let l = level as f32;
        let lo = (d.lo.0 + d.lo.1 * l).min(d.cap);
        let hi = (d.hi.0 + d.hi.1 * l).min(d.cap).max(lo);
        (lo, hi)
    }

    /// Enchantments roll a little stronger than affixes.
    pub fn enchant_range(self, level: u16) -> (f32, f32) {
        let (lo, hi) = self.range(level);
        let cap = self.def().cap * 1.25;
        ((lo * 1.2).min(cap), (hi * 1.35).min(cap).max(lo * 1.2))
    }

    /// A value `t` (0..1) of the way through a range.
    fn value_in(range: (f32, f32), t: f32) -> i16 {
        let (lo, hi) = range;
        (lo + (hi - lo) * t.clamp(0.0, 1.0))
            .round()
            .clamp(1.0, i16::MAX as f32) as i16
    }

    /// How good a value is inside a range, 0..1.
    fn norm_in(range: (f32, f32), v: i16) -> f32 {
        let (lo, hi) = range;
        let (lo, hi) = (lo.round(), hi.round());
        if hi - lo < 0.5 {
            return if v as f32 >= hi { 1.0 } else { 0.5 };
        }
        ((v as f32 - lo) / (hi - lo)).clamp(0.0, 1.0)
    }

    /// The line shown in tooltips: "+12 Damage", "+8% Crit Chance".
    pub fn line(self, v: i32) -> String {
        let d = self.def();
        match d.fmt {
            Fmt::Plus => format!("+{v} {}", d.name),
            Fmt::Pct => format!("+{v}% {}", d.name),
            Fmt::Less => format!("-{v}% {}", d.name),
        }
    }
}

// ------------------------------------------------------------------------------------------
// Rarity
// ------------------------------------------------------------------------------------------

#[derive(
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Debug,
    Default,
    Hash,
    serde::Serialize,
    serde::Deserialize,
)]
pub enum Rarity {
    #[default]
    Common,
    Uncommon,
    Rare,
    Epic,
    Legendary,
}

impl Rarity {
    pub fn name(self) -> &'static str {
        ["Common", "Uncommon", "Rare", "Epic", "Legendary"][self as usize]
    }

    /// Bright colour for dark backgrounds, loot beams and slot frames.
    pub fn color(self) -> u8 {
        [WHITE, LIME, SKY, LAVENDER, GOLD][self as usize]
    }

    /// Deeper colour for text on parchment.
    pub fn ink(self) -> u8 {
        [INK, TEAL, BLUE, PURPLE, RUST][self as usize]
    }

    /// Darker partner colour (frame shading, beam base).
    pub fn shade(self) -> u8 {
        [KHAKI, GREEN, BLUE, PURPLE, ORANGE][self as usize]
    }

    /// Price multiplier when selling.
    pub fn value(self) -> f32 {
        [1.0, 1.6, 2.6, 4.2, 7.0][self as usize]
    }

    /// Gear rarity from its roll score (see [`Gear::score`]).
    pub fn from_score(score: f32) -> Rarity {
        if score >= 3.2 {
            Rarity::Legendary
        } else if score >= 2.5 {
            Rarity::Epic
        } else if score >= 1.7 {
            Rarity::Rare
        } else if score >= 0.9 {
            Rarity::Uncommon
        } else {
            Rarity::Common
        }
    }

    /// Scroll rarity from how well its enchantment rolled (0..1).
    pub fn from_roll(t: f32) -> Rarity {
        if t >= 0.95 {
            Rarity::Legendary
        } else if t >= 0.83 {
            Rarity::Epic
        } else if t >= 0.62 {
            Rarity::Rare
        } else if t >= 0.36 {
            Rarity::Uncommon
        } else {
            Rarity::Common
        }
    }
}

// ------------------------------------------------------------------------------------------
// Classes
// ------------------------------------------------------------------------------------------

/// Which kind of scroll fits a piece of gear.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, serde::Serialize, serde::Deserialize)]
pub enum Group {
    Weapon,
    Armor,
    Tool,
}

impl Group {
    pub fn name(self) -> &'static str {
        ["Weapon", "Armor", "Tool"][self as usize]
    }

    /// Enchantments a scroll of this group can carry.
    pub fn enchant_pool(self) -> &'static [Stat] {
        use Stat::*;
        match self {
            Group::Weapon => &[
                Damage, Crit, CritDmg, Haste, Lifesteal, Burn, Chill, Shock, Focus, Luck, Greed,
            ],
            Group::Armor => &[
                Defense, Vitality, Stamina, Wisdom, Regen, Swift, Dodge, Block, Thorns, Spirit,
                Luck, Greed,
            ],
            Group::Tool => &[
                Power, Frugal, Haste, Reach, Capacity, Bounty, Growth, Forage, Luck, Greed, Lure,
                Line, Treasure, Angler,
            ],
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum Class {
    Sword,
    Wand,
    Staff,
    Shield,
    Head,
    Chest,
    Legs,
    Feet,
    Hoe,
    Can,
    Sickle,
    Axe,
    Pickaxe,
    /// A fishing rod.
    Rod,
}

pub const CLASSES: [Class; 14] = [
    Class::Sword,
    Class::Wand,
    Class::Staff,
    Class::Shield,
    Class::Head,
    Class::Chest,
    Class::Legs,
    Class::Feet,
    Class::Hoe,
    Class::Can,
    Class::Sickle,
    Class::Axe,
    Class::Pickaxe,
    Class::Rod,
];

/// Where armour is worn. Weapons and tools are used from the hotbar instead.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Slot {
    Head,
    Chest,
    Legs,
    Feet,
    Shield,
}

pub const SLOTS: [Slot; 5] = [
    Slot::Head,
    Slot::Chest,
    Slot::Legs,
    Slot::Feet,
    Slot::Shield,
];

impl Slot {
    /// The faint picture shown in an empty equipment slot.
    pub fn ghost_icon(self) -> &'static str {
        [
            "ghost_head",
            "ghost_chest",
            "ghost_legs",
            "ghost_feet",
            "ghost_shield",
        ][self as usize]
    }
}

impl Class {
    pub fn group(self) -> Group {
        match self {
            Class::Sword | Class::Wand | Class::Staff => Group::Weapon,
            Class::Shield | Class::Head | Class::Chest | Class::Legs | Class::Feet => Group::Armor,
            _ => Group::Tool,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Class::Sword => "Sword",
            Class::Wand => "Wand",
            Class::Staff => "Staff",
            Class::Shield => "Shield",
            Class::Head => "Headgear",
            Class::Chest => "Chest Armor",
            Class::Legs => "Leg Armor",
            Class::Feet => "Boots",
            Class::Hoe => "Hoe",
            Class::Can => "Watering Can",
            Class::Sickle => "Sickle",
            Class::Axe => "Axe",
            Class::Pickaxe => "Pickaxe",
            Class::Rod => "Fishing Rod",
        }
    }

    pub fn plural(self) -> &'static str {
        match self {
            Class::Sword => "swords",
            Class::Wand => "wands",
            Class::Staff => "staffs",
            Class::Shield => "shields",
            Class::Head => "headgear",
            Class::Chest => "chest armor",
            Class::Legs => "leg armor",
            Class::Feet => "boots",
            Class::Hoe => "hoes",
            Class::Can => "watering cans",
            Class::Sickle => "sickles",
            Class::Axe => "axes",
            Class::Pickaxe => "pickaxes",
            Class::Rod => "fishing rods",
        }
    }

    pub fn slot(self) -> Option<Slot> {
        match self {
            Class::Head => Some(Slot::Head),
            Class::Chest => Some(Slot::Chest),
            Class::Legs => Some(Slot::Legs),
            Class::Feet => Some(Slot::Feet),
            Class::Shield => Some(Slot::Shield),
            _ => None,
        }
    }

    pub fn is_armor(self) -> bool {
        self.slot().is_some()
    }

    /// Stats this class can roll as affixes.
    pub fn affix_pool(self) -> &'static [Stat] {
        use Stat::*;
        match self {
            // Weapons and tools only count while held, so they never raise max HP, energy
            // or mana (those would drain away every time you switched items).
            Class::Sword => &[
                Damage, Crit, CritDmg, Haste, Lifesteal, Burn, Chill, Shock, Luck, Greed,
            ],
            Class::Wand => &[
                Damage, Crit, CritDmg, Haste, Lifesteal, Burn, Chill, Shock, Focus, Spirit, Luck,
            ],
            Class::Staff => &[
                Damage, Crit, CritDmg, Burn, Chill, Shock, Focus, Spirit, Regen, Lifesteal,
            ],
            Class::Shield => &[Defense, Block, Thorns, Vitality, Regen, Dodge, Stamina],
            Class::Head => &[
                Defense, Vitality, Wisdom, Luck, Regen, Crit, Spirit, Stamina,
            ],
            Class::Chest => &[Defense, Vitality, Stamina, Regen, Thorns, Wisdom, Damage],
            Class::Legs => &[Defense, Vitality, Stamina, Swift, Dodge, Greed, Haste],
            Class::Feet => &[Defense, Swift, Dodge, Stamina, Luck, Greed, Forage],
            Class::Hoe => &[Frugal, Haste, Reach, Forage, Growth, Luck],
            Class::Can => &[Capacity, Reach, Growth, Frugal, Haste, Bounty],
            Class::Sickle => &[Bounty, Reach, Haste, Forage, Frugal, Damage, Luck],
            Class::Axe => &[Power, Haste, Frugal, Bounty, Forage, Damage],
            Class::Pickaxe => &[Power, Haste, Frugal, Bounty, Forage, Luck, Greed],
            Class::Rod => &[Lure, Line, Treasure, Angler, Haste, Luck, Greed],
        }
    }

    /// Does an enchantment of this stat do anything on this class?
    pub fn suits(self, stat: Stat) -> bool {
        self.affix_pool().contains(&stat)
            || matches!(stat, Stat::Luck | Stat::Greed)
            || (self.group() == Group::Armor && self.group().enchant_pool().contains(&stat))
            || (self.group() == Group::Weapon
                && matches!(stat, Stat::Lifesteal | Stat::Haste | Stat::Damage))
    }

    /// How the base number reads in a tooltip.
    pub fn main_label(self) -> &'static str {
        match self {
            Class::Sword => "Damage",
            Class::Wand => "Magic Damage",
            Class::Staff => "Blast Damage",
            Class::Shield | Class::Head | Class::Chest | Class::Legs | Class::Feet => "Defense",
            Class::Axe => "Chopping Power",
            Class::Pickaxe => "Mining Power",
            Class::Hoe => "Tilling Reach",
            Class::Can => "Water",
            Class::Sickle => "Reap Damage",
            Class::Rod => "Reel Power",
        }
    }
}

/// The base number of a piece at an item level: damage for weapons, defense for armour,
/// power, reach or water for tools. `mult` is the base item's strength in percent.
pub fn base_value(class: Class, level: u16, quality: u8, mult: u8) -> i32 {
    let l = level.max(1) as f32;
    let q = 0.85 + 0.3 * quality.min(100) as f32 / 100.0;
    let m = mult as f32 / 100.0;
    let v = match class {
        Class::Sword => 5.0 + 1.7 * l,
        Class::Wand => 4.0 + 1.35 * l,
        Class::Staff => 6.0 + 1.9 * l,
        Class::Shield => 1.5 + 0.45 * l,
        Class::Head => 1.0 + 0.35 * l,
        Class::Chest => 2.0 + 0.6 * l,
        Class::Legs => 1.5 + 0.45 * l,
        Class::Feet => 1.0 + 0.3 * l,
        Class::Axe | Class::Pickaxe => 2.0 + 0.22 * l,
        Class::Sickle => 3.0 + 0.8 * l,
        Class::Can => 20.0 + 1.6 * l,
        Class::Rod => 8.0 + 1.0 * l,
        // Tilling reach in tiles; the quality roll does not change it.
        Class::Hoe => return (1 + level as i32 / 22).min(3),
    };
    (v * m * q).round().max(1.0) as i32
}

// ------------------------------------------------------------------------------------------
// Gear instances
// ------------------------------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Affix {
    pub stat: Stat,
    pub val: i16,
}

pub const AFFIXES: usize = 4;
pub const SOCKETS: usize = 3;

/// What makes one sword different from another sword of the same kind. Scrolls use the same
/// record: their enchantment is the first affix.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Gear {
    pub level: u16,
    /// The base value roll, 0..=100.
    pub quality: u8,
    pub rarity: Rarity,
    pub affixes: [Option<Affix>; AFFIXES],
    pub enchants: [Option<Affix>; SOCKETS],
    /// Forge experience from the anvil (armour only); its forge level follows from this.
    pub xp: u32,
}

impl Gear {
    /// An unremarkable piece: no affixes, average quality.
    pub fn plain(level: u16) -> Gear {
        Gear {
            level: level.max(1),
            quality: 50,
            rarity: Rarity::Common,
            affixes: [None; AFFIXES],
            enchants: [None; SOCKETS],
            xp: 0,
        }
    }

    pub fn affixes(&self) -> impl Iterator<Item = &Affix> {
        self.affixes.iter().flatten()
    }

    pub fn enchants(&self) -> impl Iterator<Item = &Affix> {
        self.enchants.iter().flatten()
    }

    /// Every stat line: affixes then enchantments.
    pub fn all(&self) -> impl Iterator<Item = &Affix> {
        self.affixes().chain(self.enchants())
    }

    /// How good the rolls are overall; rarity follows from this.
    pub fn score(&self) -> f32 {
        let mut s = self.quality.min(100) as f32 / 100.0 * 0.4;
        for a in self.affixes() {
            s += 0.25 + 0.75 * Stat::norm_in(a.stat.range(self.level), a.val);
        }
        for e in self.enchants() {
            s += 0.3 + 0.9 * Stat::norm_in(e.stat.enchant_range(self.level), e.val);
        }
        s
    }

    pub fn update_rarity(&mut self) {
        self.rarity = Rarity::from_score(self.score());
    }

    /// Rolls a fresh piece of a class. `luck` (0 = none, 1 = a lot) pushes towards more and
    /// better affixes.
    pub fn roll(class: Class, level: u16, luck: f32, rng: &mut Rng) -> Gear {
        let mut g = Gear::plain(level);
        let luck = luck.max(0.0);
        g.quality = (100.0 * rng.f32().powf(1.0 / (1.0 + luck * 0.6))).round() as u8;
        let k = 1.0 + luck;
        let weights = [26.0 / k, 40.0, 22.0 * k, 9.0 * k * k, 3.0 * k * k * k];
        let n = rng.weighted(&weights).min(AFFIXES);
        let mut pool: Vec<Stat> = class.affix_pool().to_vec();
        rng.shuffle(&mut pool);
        for (i, stat) in pool.into_iter().take(n).enumerate() {
            let t = rng.f32().powf(1.0 / (1.0 + luck * 0.5));
            g.affixes[i] = Some(Affix {
                stat,
                val: Stat::value_in(stat.range(g.level), t),
            });
        }
        g.update_rarity();
        g
    }

    /// Finishes a piece off finely: fine quality, at least `min` affixes (topped up from what
    /// its class can roll), and none of them rolled below `floor` (0..1 through its range).
    pub fn polish(&mut self, class: Class, min: usize, floor: f32, rng: &mut Rng) {
        self.quality = self.quality.max(85);
        let level = self.level;
        let lift = |stat: Stat, rng: &mut Rng| {
            Stat::value_in(stat.range(level), floor + rng.f32() * (1.0 - floor) * 0.6)
        };
        for a in self.affixes.iter_mut().flatten() {
            if Stat::norm_in(a.stat.range(level), a.val) < floor {
                a.val = lift(a.stat, rng);
            }
        }
        let mut pool: Vec<Stat> = class
            .affix_pool()
            .iter()
            .copied()
            .filter(|s| !self.affixes().any(|a| a.stat == *s))
            .collect();
        rng.shuffle(&mut pool);
        let mut have = self.affixes().count();
        for slot in self.affixes.iter_mut() {
            if have >= min.min(AFFIXES) {
                break;
            }
            if slot.is_none() {
                let Some(stat) = pool.pop() else { break };
                *slot = Some(Affix {
                    stat,
                    val: lift(stat, rng),
                });
                have += 1;
            }
        }
        self.update_rarity();
    }

    /// Puts an enchantment in a socket (replacing what was there).
    pub fn enchant(&mut self, socket: usize, a: Affix) {
        if socket < SOCKETS {
            self.enchants[socket] = Some(a);
            self.update_rarity();
        }
    }

    /// The first empty socket, if any.
    pub fn free_socket(&self) -> Option<usize> {
        self.enchants.iter().position(|e| e.is_none())
    }

    /// Adds this piece's affixes and enchantments to a stat sheet (made stronger by forging).
    pub fn add_to(&self, sheet: &mut Sheet) {
        for a in self.all() {
            sheet.add(a.stat, self.forged_stat(a.val as i32));
        }
    }

    /// The innate bonus of a base item (a Frog Hood's luck, an Ember Blade's burn).
    pub fn innate(&self, stat: Stat) -> i16 {
        let (lo, hi) = stat.range(self.level);
        (((lo + hi) * 0.5) * 0.6).round().max(1.0) as i16
    }
}

// ------------------------------------------------------------------------------------------
// Forging
// ------------------------------------------------------------------------------------------

/// How far armour can be forged: +10.
pub const MAX_FORGE: u8 = 10;
/// How much stronger each forge level makes a piece: its main number (defense), and every
/// other stat on it (rounded up, so the first level always shows).
pub const FORGE_MAIN: f32 = 0.08;
pub const FORGE_STAT: f32 = 0.04;

/// The forge experience a piece needs to go up from forge level `forge`. Every level asks
/// half again as much as the last, and armour from deeper down (a higher item level) asks
/// more from the start: the deeper you go, the harder it gets.
pub fn forge_need(level: u16, forge: u8) -> u32 {
    let base = 40.0 + level.max(1) as f32 * 3.0;
    (base * 1.5f32.powi(forge as i32)).round() as u32
}

/// All the forge experience it takes to reach forge level `forge` from nothing.
pub fn forge_total(level: u16, forge: u8) -> u32 {
    (0..forge.min(MAX_FORGE))
        .map(|f| forge_need(level, f))
        .sum()
}

/// What a strike of the anvil costs: more for armour from deeper down, and a good deal more
/// for every forge level it already has.
pub fn forge_cost(level: u16, forge: u8) -> u64 {
    let base = 20.0 + level.max(1) as f64 * 2.0;
    (base * (1.0 + forge as f64).powf(1.5)).round() as u64
}

/// The forge experience a piece of armour gives up when it's melted into another: more for a
/// higher level (things found deeper down) and a rarer roll, and half of all the forge
/// experience it had gathered itself; a quarter more again melted into its own kind (a helm
/// into a helm).
pub fn fodder_xp(fodder: &Gear, same_kind: bool) -> u32 {
    let rarity = [1.0, 1.5, 2.3, 3.5, 5.5][fodder.rarity as usize];
    let base = (10.0 + fodder.level.max(1) as f32 * 2.5) * rarity;
    let kind = if same_kind { 1.25 } else { 1.0 };
    (base * kind).round() as u32 + fodder.xp / 2
}

impl Gear {
    /// How far this piece has been forged, 0 (never) to `MAX_FORGE`.
    pub fn forge(&self) -> u8 {
        let mut left = self.xp;
        let mut f = 0;
        while f < MAX_FORGE {
            let need = forge_need(self.level, f);
            if left < need {
                break;
            }
            left -= need;
            f += 1;
        }
        f
    }

    /// Forge experience gathered towards the next level, and what that level asks (`None`
    /// once it's forged as far as it goes).
    pub fn forge_progress(&self) -> (u32, Option<u32>) {
        let f = self.forge();
        let into = self.xp - forge_total(self.level, f);
        if f >= MAX_FORGE {
            (0, None)
        } else {
            (into, Some(forge_need(self.level, f)))
        }
    }

    /// Melts forge experience into this piece (none past the top level); returns how many
    /// levels it went up.
    pub fn add_forge_xp(&mut self, xp: u32) -> u8 {
        let before = self.forge();
        let top = forge_total(self.level, MAX_FORGE);
        self.xp = self.xp.saturating_add(xp).min(top);
        self.forge() - before
    }

    /// Its main number (defense, damage...) with its forging.
    pub fn forged_main(&self, v: i32) -> i32 {
        forged(v, FORGE_MAIN, self.forge())
    }

    /// One of its other stats with its forging.
    pub fn forged_stat(&self, v: i32) -> i32 {
        forged(v, FORGE_STAT, self.forge())
    }
}

fn forged(v: i32, per: f32, forge: u8) -> i32 {
    if forge == 0 || v <= 0 {
        v
    } else {
        // (A hair under, so float dust never tips an exact bonus up a whole point.)
        v + (v as f32 * per * forge as f32 - 1e-3).ceil() as i32
    }
}

// ------------------------------------------------------------------------------------------
// Scrolls
// ------------------------------------------------------------------------------------------

/// Rolls a scroll's enchantment. The rarer outcomes are the stronger rolls.
pub fn roll_scroll(group: Group, level: u16, luck: f32, rng: &mut Rng) -> Gear {
    let pool = group.enchant_pool();
    let stat = pool[rng.below(pool.len())];
    let t = rng.f32().powf(1.6 / (1.0 + luck.max(0.0)));
    scroll_with(stat, level, t)
}

/// A scroll carrying `stat`, `t` (0..1) of the way through its range.
pub fn scroll_with(stat: Stat, level: u16, t: f32) -> Gear {
    let mut g = Gear::plain(level);
    let val = Stat::value_in(stat.enchant_range(g.level), t);
    g.affixes[0] = Some(Affix { stat, val });
    g.update_scroll_rarity();
    g
}

impl Gear {
    /// A scroll's enchantment.
    pub fn scroll_enchant(&self) -> Option<Affix> {
        self.affixes[0]
    }

    pub fn update_scroll_rarity(&mut self) {
        self.rarity = match self.affixes[0] {
            Some(a) => Rarity::from_roll(Stat::norm_in(a.stat.enchant_range(self.level), a.val)),
            None => Rarity::Common,
        };
    }
}

// ------------------------------------------------------------------------------------------
// Stat sheets
// ------------------------------------------------------------------------------------------

/// Stats added up from gear, food and anything else.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Sheet {
    pub v: [i32; STATS],
}

impl Sheet {
    pub fn get(&self, s: Stat) -> i32 {
        self.v[s as usize]
    }

    pub fn add(&mut self, s: Stat, v: i32) {
        self.v[s as usize] += v;
    }

    /// A percentage stat as a fraction, capped.
    pub fn frac(&self, s: Stat, cap: i32) -> f32 {
        self.get(s).clamp(0, cap) as f32 / 100.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stat_keys_round_trip() {
        for (i, s) in ALL_STATS.iter().enumerate() {
            assert_eq!(*s as usize, i);
            assert_eq!(Stat::from_key(s.def().key), Some(*s));
        }
    }

    #[test]
    fn ranges_grow_with_level_and_respect_caps() {
        for s in ALL_STATS {
            let (lo1, hi1) = s.range(1);
            let (lo60, hi60) = s.range(60);
            assert!(lo1 <= hi1 && lo60 <= hi60, "{s:?}");
            assert!(hi60 >= hi1, "{s:?} should not shrink");
            assert!(hi60 <= s.def().cap, "{s:?} over cap");
            let (elo, ehi) = s.enchant_range(60);
            assert!(elo <= ehi);
        }
    }

    #[test]
    fn rarity_follows_rolls() {
        let mut g = Gear::plain(20);
        g.quality = 0;
        g.update_rarity();
        assert_eq!(g.rarity, Rarity::Common);
        // Four perfect affixes make a legend.
        let pool = Class::Sword.affix_pool();
        for (slot, s) in g.affixes.iter_mut().zip(pool.iter()) {
            *slot = Some(Affix {
                stat: *s,
                val: Stat::value_in(s.range(20), 1.0),
            });
        }
        g.quality = 100;
        g.update_rarity();
        assert_eq!(g.rarity, Rarity::Legendary);
        // Enchanting can lift a plain piece.
        let mut p = Gear::plain(20);
        p.quality = 60;
        p.update_rarity();
        let before = p.rarity;
        for socket in 0..SOCKETS {
            let s = Stat::Damage;
            p.enchant(
                socket,
                Affix {
                    stat: s,
                    val: Stat::value_in(s.enchant_range(20), 1.0),
                },
            );
        }
        assert!(p.rarity > before);
        assert!(p.free_socket().is_none());
    }

    #[test]
    fn rolls_are_varied_and_luck_helps() {
        let mut rng = Rng::new(3);
        let mut seen = [0u32; 5];
        let mut lucky = [0u32; 5];
        for _ in 0..4000 {
            seen[Gear::roll(Class::Chest, 30, 0.0, &mut rng).rarity as usize] += 1;
            lucky[Gear::roll(Class::Chest, 30, 1.0, &mut rng).rarity as usize] += 1;
        }
        assert!(seen.iter().all(|&n| n > 0), "every rarity drops: {seen:?}");
        assert!(seen[0] > seen[4] * 10, "legendaries are rare: {seen:?}");
        assert!(lucky[3] + lucky[4] > seen[3] + seen[4], "luck helps");
    }

    #[test]
    fn scroll_rarity_comes_from_its_enchantment() {
        let weak = scroll_with(Stat::Burn, 30, 0.0);
        let strong = scroll_with(Stat::Burn, 30, 1.0);
        assert_eq!(weak.rarity, Rarity::Common);
        assert_eq!(strong.rarity, Rarity::Legendary);
        let (w, s) = (
            weak.scroll_enchant().unwrap().val,
            strong.scroll_enchant().unwrap().val,
        );
        assert!(s > w);
        let mut rng = Rng::new(9);
        let mut seen = [0u32; 5];
        for _ in 0..3000 {
            let g = roll_scroll(Group::Armor, 25, 0.0, &mut rng);
            assert!(
                Group::Armor
                    .enchant_pool()
                    .contains(&g.scroll_enchant().unwrap().stat)
            );
            seen[g.rarity as usize] += 1;
        }
        assert!(seen.iter().all(|&n| n > 0), "{seen:?}");
        assert!(seen[0] > seen[4]);
    }

    #[test]
    fn base_values_scale_with_level() {
        for c in CLASSES {
            let a = base_value(c, 1, 50, 100);
            let b = base_value(c, 60, 50, 100);
            assert!(b >= a, "{c:?}");
            assert!(a >= 1);
        }
        assert!(base_value(Class::Sword, 30, 100, 100) > base_value(Class::Sword, 30, 0, 100));
    }

    #[test]
    fn forging_grows_harder_and_makes_armour_a_little_stronger() {
        // Each level asks more than the last, and deeper armour asks more from the start.
        for lvl in [1u16, 20, 60] {
            for f in 1..MAX_FORGE {
                assert!(forge_need(lvl, f) > forge_need(lvl, f - 1));
            }
        }
        assert!(forge_need(50, 0) > forge_need(5, 0));
        assert!(forge_cost(30, 5) > forge_cost(30, 0));
        assert!(forge_cost(50, 2) > forge_cost(10, 2));
        // Experience climbs through the levels and stops at the top.
        let mut g = Gear::plain(10);
        assert_eq!(g.forge(), 0);
        assert_eq!(g.forge_progress(), (0, Some(forge_need(10, 0))));
        assert_eq!(g.add_forge_xp(forge_need(10, 0)), 1);
        assert_eq!(g.forge(), 1);
        assert_eq!(g.add_forge_xp(forge_need(10, 1) - 1), 0);
        assert_eq!(
            g.forge_progress(),
            (forge_need(10, 1) - 1, Some(forge_need(10, 1)))
        );
        assert_eq!(g.add_forge_xp(1_000_000), MAX_FORGE - 1);
        assert_eq!(g.forge(), MAX_FORGE);
        assert_eq!(g.forge_progress(), (0, None));
        assert_eq!(g.xp, forge_total(10, MAX_FORGE));
        // A bit stronger for it: 80% on the main number and 40% on the rest, at +10...
        assert_eq!(g.forged_main(20), 36);
        assert_eq!(g.forged_stat(10), 14);
        assert_eq!(Gear::plain(10).forged_main(20), 20);
        // ...and something to show for the very first level, however small the number.
        let mut one = Gear::plain(10);
        one.add_forge_xp(forge_need(10, 0));
        assert_eq!(one.forged_main(2), 3);
        assert_eq!(one.forged_stat(1), 2);
        // Every level shows on a middling main number.
        let mut up = Gear::plain(10);
        let mut last = up.forged_main(13);
        for f in 0..MAX_FORGE {
            up.add_forge_xp(forge_need(10, f));
            assert!(up.forged_main(13) > last, "+{}", f + 1);
            last = up.forged_main(13);
        }
        // What a piece gives up: more for deeper, rarer pieces, and some of its own forging.
        let mut deep = Gear::plain(40);
        let shallow = Gear::plain(4);
        assert!(fodder_xp(&deep, false) > fodder_xp(&shallow, false));
        assert!(fodder_xp(&deep, true) > fodder_xp(&deep, false));
        let plain = fodder_xp(&deep, false);
        deep.rarity = Rarity::Legendary;
        assert!(fodder_xp(&deep, false) > plain * 5);
        deep.xp = 1000;
        assert!(fodder_xp(&deep, false) >= plain * 5 + 500);
    }

    #[test]
    fn every_class_has_useful_enchantments() {
        for c in CLASSES {
            let fits = c
                .group()
                .enchant_pool()
                .iter()
                .filter(|s| c.suits(**s))
                .count();
            assert!(fits >= 3, "{c:?} has only {fits} useful enchantments");
        }
    }
}
