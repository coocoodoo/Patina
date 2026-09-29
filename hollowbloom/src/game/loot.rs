//! Loot: coins, what monsters, chests and pots leave behind, random gear and scrolls, and
//! Burrowby's daily specials.

use super::dungeon::{Foe, biome_seeds};
use super::gear::{self, Class, Gear, Group};
use super::items::{ALL_ITEMS, Base, Item, MAX_PACK, MIN_PACK, Pack, Stack};
use crate::util::{Rng, thousands};

/// 10 copper make a silver coin and 10 silver a gold coin. Money is counted in copper.
pub const SILVER: u64 = 10;
pub const GOLD: u64 = 100;

/// The gold, silver and copper coins that make up an amount.
pub fn money_parts(c: u64) -> (u64, u64, u64) {
    (c / GOLD, (c / SILVER) % 10, c % SILVER)
}

/// "12g 3s 4c", leaving out empty parts.
pub fn money_text(c: u64) -> String {
    let (g, s, cc) = money_parts(c);
    let mut parts = Vec::new();
    if g > 0 {
        parts.push(format!("{}g", thousands(g)));
    }
    if s > 0 {
        parts.push(format!("{s}s"));
    }
    if cc > 0 || parts.is_empty() {
        parts.push(format!("{cc}c"));
    }
    parts.join(" ")
}

/// Coin piles worth an amount, biggest coins first. Big hauls come out as several piles, so
/// they fountain out nicely.
pub fn coin_stacks(c: u64) -> Vec<Stack> {
    let (g, s, cc) = money_parts(c);
    let mut v = Vec::new();
    for (item, n) in [
        (Item::GoldCoin, g),
        (Item::SilverCoin, s),
        (Item::CopperCoin, cc),
    ] {
        let pile = (n / 6).max(4);
        let mut left = n;
        while left > 0 {
            let k = left.min(pile);
            v.push(Stack::new(item, k.min(999) as u16));
            left -= k;
        }
    }
    v
}

/// How lucky the player is right now.
#[derive(Clone, Copy, Debug, Default)]
pub struct Fortune {
    /// 0 = normal; each 0.1 is a noticeably better roll.
    pub luck: f32,
    /// Coin multiplier, 1 = normal.
    pub greed: f32,
}

impl Fortune {
    #[cfg(test)]
    pub fn plain() -> Fortune {
        Fortune {
            luck: 0.0,
            greed: 1.0,
        }
    }
}

fn gear_bases() -> impl Iterator<Item = (Item, Base)> {
    ALL_ITEMS.iter().filter_map(|i| i.base().map(|b| (*i, b)))
}

fn class_weight(c: Class) -> f32 {
    match c {
        Class::Sword => 1.0,
        Class::Wand => 0.75,
        Class::Staff => 0.6,
        Class::Shield => 0.8,
        Class::Head | Class::Chest => 1.0,
        Class::Legs | Class::Feet => 0.9,
        Class::Hoe | Class::Can | Class::Sickle => 0.3,
        Class::Axe | Class::Pickaxe => 0.35,
        // Rods come from the water folk and treasure chests (see `random_rod`).
        Class::Rod => 0.0,
    }
}

/// A base item suited to a depth: mostly things that first turn up near it.
pub fn pick_base(depth: u32, group: Option<Group>, rng: &mut Rng) -> Item {
    let d = depth.max(1) as f32;
    let mut items = Vec::new();
    let mut weights = Vec::new();
    for (item, b) in gear_bases() {
        if group.is_some_and(|g| b.class.group() != g) || b.class == Class::Rod {
            continue;
        }
        let lvl = b.lvl as f32;
        if lvl > d + 4.0 {
            continue;
        }
        let gap = (d - lvl).max(0.0) / 9.0;
        items.push(item);
        weights.push(class_weight(b.class) / (1.0 + gap * gap));
    }
    items[rng.weighted(&weights)]
}

/// Rolls a piece of gear of a given kind at a level.
pub fn roll_gear(item: Item, level: u16, luck: f32, rng: &mut Rng) -> Stack {
    let b = item.base().expect("gear item");
    let level = level.max(b.lvl).max(1);
    Stack::with_gear(item, Gear::roll(b.class, level, luck, rng))
}

/// A random piece of gear for a depth.
pub fn random_gear(depth: u32, f: Fortune, rng: &mut Rng) -> Stack {
    let item = pick_base(depth, None, rng);
    let level = (depth as i32 + rng.range(-2, 3)).max(1) as u16;
    roll_gear(item, level, f.luck + depth as f32 * 0.004, rng)
}

/// Gear rolled about as well as it can be: the best of three very lucky rolls a few levels
/// up, and never shoddy. What gleaming chests and guardians' hoards are made of.
pub fn fine_gear(depth: u32, f: Fortune, rng: &mut Rng) -> Stack {
    let mut best: Option<(f32, Stack)> = None;
    // Weapons and armour: the pieces with room for the most (and best) enchantments.
    let group = if rng.chance(0.5) {
        Group::Weapon
    } else {
        Group::Armor
    };
    for _ in 0..3 {
        let item = pick_base(depth + 3, Some(group), rng);
        let level = (depth + 2 + rng.below(3) as u32) as u16;
        let s = roll_gear(item, level, f.luck + 1.6 + depth as f32 * 0.004, rng);
        let score = s.gear.map_or(0.0, |g| g.score());
        if best.as_ref().is_none_or(|(b, _)| score > *b) {
            best = Some((score, s));
        }
    }
    let (_, mut s) = best.expect("three rolls");
    if let (Some(g), Some(b)) = (s.gear.as_mut(), s.item.base()) {
        g.polish(b.class, 3, 0.6, rng);
    }
    s
}

/// A fishing rod from the Hollow: mostly the colourful kinds you can't buy, with better
/// rolls than anything on Garrick's shelf.
pub fn random_rod(depth: u32, f: Fortune, rng: &mut Rng) -> Stack {
    let d = depth.max(1) as f32;
    let mut items = Vec::new();
    let mut weights = Vec::new();
    for (item, b) in gear_bases() {
        if b.class != Class::Rod || b.lvl as f32 > d + 6.0 {
            continue;
        }
        let shop = matches!(item, Item::BambooRod | Item::WillowRod | Item::OakRod);
        let gap = (d - b.lvl as f32).max(0.0) / 10.0;
        items.push(item);
        weights.push(if shop { 0.3 } else { 1.0 } / (1.0 + gap * gap));
    }
    let item = items[rng.weighted(&weights)];
    let level = (depth as i32 + rng.range(-1, 3)).max(1) as u16;
    roll_gear(item, level, f.luck + 0.35 + depth as f32 * 0.004, rng)
}

/// A chest fished up from the bottom: coins, and often something shiny.
pub fn sunken_treasure(depth: u32, lava: bool, f: Fortune, rng: &mut Rng) -> Vec<Stack> {
    let d = depth.max(1);
    let mut out = Vec::new();
    let c = (12.0 + d as f32 * 3.5) * rng.range_f(0.7, 1.5) * f.greed;
    out.extend(coin_stacks(c.round() as u64));
    if rng.chance(0.5) {
        out.push(Stack::new(random_gem(d, rng), 1));
    }
    if rng.chance(0.25 + f.luck * 0.1) {
        out.push(Stack::new(random_relic(d, rng), 1));
    }
    if rng.chance(0.14 + f.luck * 0.1) {
        out.push(random_rod(d, f, rng));
    }
    if rng.chance(0.12) {
        out.push(random_scroll(d, f, rng));
    }
    if lava {
        out.push(Stack::new(Item::EmberOre, 2 + rng.below(3) as u16));
    } else if rng.chance(0.4) {
        out.push(Stack::new(Item::Bait, 3 + rng.below(4) as u16));
    }
    out
}

/// A random scroll for a depth.
pub fn random_scroll(depth: u32, f: Fortune, rng: &mut Rng) -> Stack {
    let group = [Group::Weapon, Group::Armor, Group::Tool][rng.weighted(&[0.4, 0.4, 0.2])];
    scroll_of(group, depth, f, rng)
}

pub fn scroll_of(group: Group, depth: u32, f: Fortune, rng: &mut Rng) -> Stack {
    let level = (depth as i32 + rng.range(0, 3)).max(1) as u16;
    Stack::with_gear(
        Item::scroll(group),
        gear::roll_scroll(group, level, f.luck, rng),
    )
}

/// A gem, rarer ones deeper down.
pub fn random_gem(depth: u32, rng: &mut Rng) -> Item {
    let d = depth as f32;
    let table = [
        (Item::Topaz, 4.0),
        (Item::Ruby, 3.0),
        (Item::Sapphire, 3.0),
        (Item::Emerald, 3.0),
        (Item::Amethyst, if d >= 15.0 { 3.0 } else { 1.0 }),
        (Item::Moonstone, if d >= 30.0 { 1.5 } else { 0.2 }),
        (Item::StarDiamond, if d >= 45.0 { 0.5 } else { 0.06 }),
    ];
    let w: Vec<f32> = table.iter().map(|t| t.1).collect();
    table[rng.weighted(&w)].0
}

/// A lost treasure. The fancier ones hide deeper.
pub fn random_relic(depth: u32, rng: &mut Rng) -> Item {
    let relics = [
        Item::LostButton,
        Item::GlassMarble,
        Item::RubberDuck,
        Item::ChippedTeacup,
        Item::ToyBoat,
        Item::AncientCoin,
        Item::GoldenAcorn,
        Item::MusicBox,
        Item::StarFossil,
        Item::TinyCrown,
        Item::MoonPearl,
        Item::DragonScale,
    ];
    let reach = 1.0 + depth as f32 / 12.0;
    let w: Vec<f32> = relics
        .iter()
        .enumerate()
        .map(|(i, _)| 1.0 / (1.0 + (i as f32 / reach).powi(2)))
        .collect();
    relics[rng.weighted(&w)]
}

/// Seeds from a biome.
pub fn biome_seed(biome: usize, rng: &mut Rng) -> Item {
    let seeds = biome_seeds(biome);
    let w: Vec<f32> = seeds.iter().map(|s| s.1).collect();
    seeds[rng.weighted(&w)].0
}

fn roll(out: &mut Vec<Stack>, rng: &mut Rng, item: Item, p: f32, lo: u16, hi: u16) {
    if rng.chance(p) {
        out.push(Stack::new(
            item,
            lo + rng.below((hi - lo + 1) as usize) as u16,
        ));
    }
}

/// The bits a creature carries (gel, wings, bones, teeth...), without the coins and treasure.
fn foe_bits(foe: Foe, biome: usize, rng: &mut Rng) -> Vec<Stack> {
    let mut out = Vec::new();
    let o = &mut out;
    match foe {
        Foe::Slime => roll(o, rng, Item::SlimeGel, 0.75, 1, 2),
        Foe::Bat => roll(o, rng, Item::BatWing, 0.55, 1, 1),
        Foe::Shroom => {
            roll(o, rng, Item::Spore, 0.4, 1, 2);
            roll(o, rng, Item::ShroomCap, 0.35, 1, 1);
            roll(o, rng, Item::GlowcapSpores, 0.1, 1, 2);
        }
        Foe::Crab => {
            roll(o, rng, Item::Crystal, 0.35, 1, 2);
            roll(o, rng, Item::CrabShell, 0.45, 1, 1);
        }
        Foe::Wisp => {
            roll(o, rng, Item::WispDust, 0.5, 1, 2);
            roll(o, rng, Item::Spore, 0.25, 1, 1);
        }
        Foe::Beetle => {
            roll(o, rng, Item::BeetleShell, 0.45, 1, 1);
            roll(o, rng, Item::Amber, 0.12, 1, 1);
        }
        Foe::Imp => {
            roll(o, rng, Item::ImpHorn, 0.45, 1, 1);
            roll(o, rng, Item::EmberOre, 0.25, 1, 2);
        }
        Foe::Skeleton => {
            roll(o, rng, Item::Bone, 0.7, 1, 2);
            roll(o, rng, Item::GoldOre, 0.2, 1, 2);
        }
        Foe::Golem => {
            roll(o, rng, Item::Stone, 0.9, 3, 6);
            roll(o, rng, Item::IronOre, 0.4, 1, 3);
            roll(o, rng, Item::GolemHeart, 0.25, 1, 1);
            roll(o, rng, Item::FrostGem, 0.1, 1, 1);
        }
        Foe::Ghost => {
            roll(o, rng, Item::Ectoplasm, 0.5, 1, 1);
            roll(o, rng, Item::WispDust, 0.2, 1, 1);
        }
        Foe::Frog => {
            roll(o, rng, Item::Bait, 0.6, 2, 4);
            roll(o, rng, Item::Seaweed, 0.25, 1, 2);
            roll(o, rng, Item::Crayfish, 0.15, 1, 1);
        }
        Foe::Jelly => {
            roll(o, rng, Item::SlimeGel, 0.45, 1, 2);
            roll(o, rng, Item::Seaweed, 0.3, 1, 2);
            roll(o, rng, Item::MoonJelly, 0.08, 1, 1);
        }
        Foe::Puffer => {
            roll(o, rng, Item::Bait, 0.35, 1, 3);
            roll(o, rng, Item::GeodePuffer, 0.1, 1, 1);
            roll(o, rng, Item::Crystal, 0.2, 1, 1);
        }
        Foe::Zombie => {
            roll(o, rng, Item::GraveDust, 0.55, 1, 2);
            roll(o, rng, Item::Bone, 0.25, 1, 1);
            // Whatever they had in their pockets when they went under.
            roll(o, rng, Item::LostButton, 0.03, 1, 1);
        }
        Foe::Brute => {
            roll(o, rng, Item::GoblinTooth, 0.5, 1, 2);
            roll(o, rng, Item::IronOre, 0.2, 1, 2);
        }
        Foe::Sneak => {
            roll(o, rng, Item::GoblinTooth, 0.35, 1, 1);
            // They pinch shiny things.
            roll(o, rng, Item::GlassMarble, 0.03, 1, 1);
        }
        Foe::Bug => {
            roll(o, rng, Item::Chitin, 0.6, 1, 2);
            let (extra, p) = [
                (Item::Fiber, 0.4),
                (Item::Crystal, 0.2),
                (Item::Spore, 0.35),
                (Item::EmberOre, 0.15),
                (Item::FrostGem, 0.05),
                (Item::GoldOre, 0.2),
            ][biome % 6];
            roll(o, rng, extra, p, 1, 1);
        }
        Foe::Snail => {
            roll(o, rng, Item::StainedGlass, 0.55, 1, 2);
            roll(o, rng, Item::SlimeGel, 0.3, 1, 1);
        }
        Foe::Bookworm => {
            roll(o, rng, Item::GlowInk, 0.55, 1, 2);
            roll(o, rng, Item::WispDust, 0.2, 1, 1);
        }
        Foe::Drake => {
            let (scale, gem) = if biome % 6 == 3 {
                (Item::CinderScale, Item::EmberOre)
            } else {
                (Item::FrostScale, Item::FrostGem)
            };
            roll(o, rng, scale, 0.6, 1, 2);
            roll(o, rng, gem, 0.15, 1, 1);
        }
        Foe::Leafling => {
            roll(o, rng, Item::Heartleaf, 0.5, 1, 2);
            roll(o, rng, Item::Fiber, 0.4, 1, 3);
            roll(o, rng, Item::MossberrySeeds, 0.12, 1, 2);
        }
        Foe::Werewolf => {
            roll(o, rng, Item::WolfFang, 0.75, 1, 2);
            roll(o, rng, Item::Bone, 0.3, 1, 1);
        }
        Foe::Minotaur => {
            roll(o, rng, Item::MinotaurHorn, 0.7, 1, 2);
            roll(o, rng, Item::IronOre, 0.3, 1, 3);
            roll(o, rng, Item::GoldOre, 0.12, 1, 2);
        }
        Foe::Griffin => {
            roll(o, rng, Item::GriffinFeather, 0.75, 1, 3);
            roll(o, rng, Item::GoldOre, 0.15, 1, 1);
        }
    }
    out
}

/// The backpacks, commonest first, and how often each turns up in the Hollow.
const PACKS: [(Item, f32); 6] = [
    (Item::Knapsack, 3.0),
    (Item::Rucksack, 3.0),
    (Item::WickerPack, 2.0),
    (Item::DuffelPack, 2.0),
    (Item::FramePack, 1.4),
    (Item::ShellPack, 0.6),
];

/// A backpack found in the Hollow: any look, any colours, and a roll of how roomy it is
/// (roomier deeper down, and with luck).
pub fn random_pack(depth: u32, f: Fortune, rng: &mut Rng) -> Stack {
    let weights: Vec<f32> = PACKS.iter().map(|p| p.1).collect();
    let item = PACKS[rng.weighted(&weights)].0;
    roll_pack(item, depth, f, rng)
}

/// The backpack each creature carries, in its own shape.
pub fn monster_pack(foe: Foe) -> Item {
    match foe {
        Foe::Slime => Item::SlimePack,
        Foe::Bat => Item::BatPack,
        Foe::Shroom => Item::ShroomPack,
        Foe::Crab => Item::CrabPack,
        Foe::Wisp => Item::WispPack,
        Foe::Beetle => Item::BeetlePack,
        Foe::Imp => Item::ImpPack,
        Foe::Skeleton => Item::SkullPack,
        Foe::Golem => Item::GolemPack,
        Foe::Ghost => Item::GhostPack,
        Foe::Frog => Item::FrogPack,
        Foe::Jelly => Item::JellyPack,
        Foe::Puffer => Item::PufferPack,
        Foe::Zombie => Item::ZombiePack,
        Foe::Brute => Item::BrutePack,
        Foe::Sneak => Item::SneakPack,
        Foe::Bug => Item::BugPack,
        Foe::Snail => Item::SnailPack,
        Foe::Bookworm => Item::BookwormPack,
        Foe::Drake => Item::DrakePack,
        Foe::Leafling => Item::LeaflingPack,
        Foe::Werewolf => Item::WolfPack,
        Foe::Minotaur => Item::MinotaurPack,
        Foe::Griffin => Item::GriffinPack,
    }
}

/// A backpack of a given look, rolled for how roomy it is and its colours.
pub fn roll_pack(item: Item, depth: u32, f: Fortune, rng: &mut Rng) -> Stack {
    let top = (8 + depth / 3).min(MAX_PACK as u32) as f32;
    let roll = rng.f32().powf((1.8 - f.luck).clamp(0.6, 1.8));
    let slots = MIN_PACK as f32 + roll * (top - MIN_PACK as f32);
    let hue = rng.below(crate::assets::pack_art::HUES) as u8;
    Stack::with_pack(
        item,
        Pack {
            slots: (slots.round() as u8).clamp(MIN_PACK, MAX_PACK),
            hue,
        },
    )
}

/// Everything a defeated creature drops.
pub fn foe_loot(
    foe: Foe,
    boss: bool,
    biome: usize,
    depth: u32,
    f: Fortune,
    rng: &mut Rng,
) -> Vec<Stack> {
    let mut out = foe_bits(foe, biome, rng);
    // The water folk hoard fishing rods, better ones than any shop sells.
    let watery = matches!(foe, Foe::Crab | Foe::Frog | Foe::Jelly | Foe::Puffer);
    if watery && rng.chance(if boss { 1.0 } else { 0.1 + f.luck * 0.1 }) {
        out.push(random_rod(depth + if boss { 3 } else { 0 }, f, rng));
    }
    let d = depth.max(1);
    // Coins: copper up top, silver deeper, gold from guardians.
    let coin_p = if boss { 1.0 } else { 0.7 };
    if rng.chance(coin_p) {
        let base = 2.0 + d as f32 * 0.8;
        let mut c = base * rng.range_f(0.5, 1.5) * f.greed;
        // Goblins hoard coins.
        if matches!(foe, Foe::Brute | Foe::Sneak) {
            c *= 2.0;
        }
        if boss {
            c *= 30.0;
        }
        out.extend(coin_stacks(c.round().max(1.0) as u64));
    }
    if rng.chance(if boss { 1.0 } else { 0.09 }) {
        let seed = biome_seed(biome, rng);
        out.push(Stack::new(seed, if boss { 5 } else { 1 }));
    }
    let gear_p = 0.05 + f.luck * 0.08;
    // Book-worms carry scrolls about with them.
    let scroll_p = 0.03 + f.luck * 0.05 + if foe == Foe::Bookworm { 0.12 } else { 0.0 };
    if boss {
        // A guardian's hoard: a heap of gear (one piece always finely made), scrolls,
        // gems, relics, potions and a pile of whatever its kind carries.
        let lucky = Fortune {
            luck: f.luck + 0.6,
            ..f
        };
        for _ in 0..3 + rng.below(3) {
            out.push(random_gear(d + 2, lucky, rng));
        }
        out.push(fine_gear(d, f, rng));
        for _ in 0..2 {
            out.push(random_scroll(d + 2, lucky, rng));
        }
        out.push(Stack::new(Item::HeartCrystal, 1));
        out.push(Stack::new(Item::Feather, 2));
        out.push(Stack::new(random_gem(d, rng), 2 + rng.below(3) as u16));
        out.push(Stack::new(random_relic(d + 10, rng), 1));
        if rng.chance(0.5) {
            out.push(Stack::new(random_relic(d + 10, rng), 1));
        }
        let potion = if d >= 30 {
            Item::LargeHealthPotion
        } else {
            Item::HealthPotion
        };
        out.push(Stack::new(potion, 2 + rng.below(2) as u16));
        // Now and then something to carry it all home in.
        if rng.chance(0.35) {
            out.push(random_pack(d + 6, lucky, rng));
        }
        for _ in 0..4 {
            // Its kind's own bits, by the handful.
            out.extend(foe_bits(foe, biome, rng));
        }
    } else {
        if rng.chance(gear_p) {
            out.push(random_gear(d, f, rng));
        }
        if rng.chance(scroll_p) {
            out.push(random_scroll(d, f, rng));
        }
        if rng.chance(0.03 + f.luck * 0.03) {
            out.push(Stack::new(random_gem(d, rng), 1));
        }
        if rng.chance(0.01 + f.luck * 0.02) {
            out.push(Stack::new(random_relic(d, rng), 1));
        }
        // Goblins make off with other delvers' backpacks.
        let pack_p = if matches!(foe, Foe::Brute | Foe::Sneak) {
            0.02
        } else {
            0.004
        };
        if rng.chance(pack_p + f.luck * 0.005) {
            out.push(random_pack(d, f, rng));
        }
    }
    // Now and then a creature leaves a backpack in its own shape (a guardian, often).
    let own = if boss { 0.5 } else { 0.012 + f.luck * 0.01 };
    if rng.chance(own) {
        let lucky = Fortune {
            luck: f.luck + if boss { 0.8 } else { 0.0 },
            ..f
        };
        out.push(roll_pack(monster_pack(foe), d, lucky, rng));
    }
    out
}

/// The contents of a treasure chest in the Hollow. A gleaming chest holds finely rolled gear,
/// a lucky scroll, gems, a relic and a heap more coin besides.
pub fn chest_loot(depth: u32, biome: usize, gleam: bool, f: Fortune, rng: &mut Rng) -> Vec<Stack> {
    let d = depth.max(1);
    let mut out = Vec::new();
    if gleam {
        let lucky = Fortune {
            luck: f.luck + 1.2,
            ..f
        };
        for _ in 0..2 {
            out.push(fine_gear(d, f, rng));
        }
        out.push(random_scroll(d + 3, lucky, rng));
        out.push(Stack::new(random_gem(d + 15, rng), 2));
        out.push(Stack::new(random_relic(d + 15, rng), 1));
        let c = (40.0 + d as f32 * 12.0) * rng.range_f(0.8, 1.3) * f.greed;
        out.extend(coin_stacks(c.round() as u64));
    }
    out.push(Stack::new(biome_seed(biome, rng), 2 + rng.below(3) as u16));
    let ores = [
        Item::CopperOre,
        Item::IronOre,
        Item::GoldOre,
        Item::Crystal,
        Item::EmberOre,
        Item::FrostGem,
    ];
    let top = ((d / 9) as usize).min(5);
    out.push(Stack::new(
        ores[rng.below(top + 1)],
        3 + rng.below(4) as u16,
    ));
    let foods = [
        Item::HealingTonic,
        Item::VeggieStew,
        Item::StaminaTonic,
        Item::GlowSoup,
        Item::ManaTonic,
        Item::BakedPotato,
    ];
    out.push(Stack::new(foods[rng.below(foods.len())], 1));
    out.push(random_gear(
        d + 1,
        Fortune {
            luck: f.luck + 0.2,
            ..f
        },
        rng,
    ));
    // Now and then even a plain chest holds something finely made.
    if !gleam && rng.chance(0.05 + f.luck * 0.05) {
        out.push(fine_gear(d, f, rng));
    }
    if rng.chance(0.45 + f.luck * 0.2) {
        out.push(random_scroll(d, f, rng));
    }
    if rng.chance(0.3) {
        out.push(Stack::new(random_gem(d, rng), 1 + rng.below(2) as u16));
    }
    if rng.chance(0.15 + f.luck * 0.1) {
        out.push(Stack::new(random_relic(d, rng), 1));
    }
    if rng.chance(0.25) {
        out.push(Stack::new(Item::Feather, 1));
    }
    // A delver's old backpack (a roomy one, in a gleaming chest).
    if rng.chance(if gleam { 0.45 } else { 0.07 + f.luck * 0.05 }) {
        let luck = if gleam { f.luck + 0.9 } else { f.luck };
        out.push(random_pack(d + 2, Fortune { luck, ..f }, rng));
    }
    // Delvers who drowned their sorrows left their rods behind.
    if rng.chance(0.22 + f.luck * 0.1) {
        out.push(random_rod(
            d + 1,
            Fortune {
                luck: f.luck + 0.2,
                ..f
            },
            rng,
        ));
    }
    // A potion or two, bigger the deeper you go.
    if rng.chance(0.5) {
        let tier = if d >= 30 && rng.chance(0.4) {
            2
        } else if d >= 10 && rng.chance(0.6) {
            1
        } else {
            0
        };
        let kinds = [
            [
                Item::SmallHealthPotion,
                Item::HealthPotion,
                Item::LargeHealthPotion,
            ],
            [
                Item::SmallManaPotion,
                Item::ManaPotion,
                Item::LargeManaPotion,
            ],
            [
                Item::SmallEnergyPotion,
                Item::EnergyPotion,
                Item::LargeEnergyPotion,
            ],
        ];
        out.push(Stack::new(
            kinds[rng.below(3)][tier],
            1 + rng.below(2) as u16,
        ));
    }
    if rng.chance(0.05 + d as f32 * 0.002) {
        let special = [Item::HeartCrystal, Item::SunStone, Item::WishStar];
        out.push(Stack::new(special[rng.below(3)], 1));
    }
    let c = (15.0 + d as f32 * 5.0) * rng.range_f(0.7, 1.4) * f.greed;
    out.extend(coin_stacks(c.round() as u64));
    out
}

/// What might be inside a pot or crate.
pub fn pot_loot(depth: u32, biome: usize, f: Fortune, rng: &mut Rng) -> Vec<Stack> {
    let d = depth.max(1);
    let r = rng.f32();
    let mut out = Vec::new();
    if r < 0.42 {
        let c = (1.0 + d as f32 * 0.5) * rng.range_f(0.6, 1.6) * f.greed;
        out.extend(coin_stacks(c.round().max(1.0) as u64));
    } else if r < 0.55 {
        out.push(Stack::new(biome_seed(biome, rng), 1));
    } else if r < 0.64 {
        out.push(Stack::new(Item::Torch, 2));
    } else if r < 0.71 {
        let food = [
            Item::HealingTonic,
            Item::ManaTonic,
            Item::StaminaTonic,
            Item::SmallHealthPotion,
            Item::SmallManaPotion,
            Item::SmallEnergyPotion,
        ];
        out.push(Stack::new(food[rng.below(food.len())], 1));
    } else if r < 0.78 {
        out.push(Stack::new(Item::SlimeGel, 1));
    } else if r < 0.83 {
        out.push(Stack::new(random_gem(d, rng), 1));
    } else if r < 0.86 {
        out.push(random_scroll(d, f, rng));
    } else if r < 0.88 {
        out.push(random_gear(d, f, rng));
    } else if r < 0.9 {
        out.push(Stack::new(random_relic(d, rng), 1));
    }
    out
}

/// Burrowby's specials for a day: a few pieces of gear and scrolls, priced to sell.
pub fn specials(seed: u64, day: u32, deepest: u32) -> Vec<(Stack, u64)> {
    let mut rng = Rng::new(seed ^ (day as u64).wrapping_mul(0x51_7CC1) ^ 0x5E_ED5);
    let depth = deepest.max(2);
    let f = Fortune {
        luck: 0.15,
        greed: 1.0,
    };
    let mut out = Vec::new();
    for g in [Group::Weapon, Group::Armor, Group::Armor, Group::Tool] {
        let item = pick_base(depth, Some(g), &mut rng);
        let level = (depth as i32 + rng.range(-3, 1)).max(1) as u16;
        let s = roll_gear(item, level, f.luck, &mut rng);
        out.push((s, s.unit_price() * 3 + 40));
    }
    for _ in 0..2 {
        let s = random_scroll(depth, f, &mut rng);
        out.push((s, s.unit_price() * 3 + 30));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn money_reads_nicely() {
        assert_eq!(money_parts(1234), (12, 3, 4));
        assert_eq!(money_text(1234), "12g 3s 4c");
        assert_eq!(money_text(0), "0c");
        assert_eq!(money_text(50), "5s");
        assert_eq!(money_text(100_000), "1,000g");
        let total: u64 = coin_stacks(987)
            .iter()
            .map(|s| s.item.coin_value().unwrap() as u64 * s.n as u64)
            .sum();
        assert_eq!(total, 987);
    }

    #[test]
    fn bosses_shower_you_in_loot() {
        let mut rng = Rng::new(1);
        let loot = foe_loot(Foe::Slime, true, 0, 10, Fortune::plain(), &mut rng);
        assert!(loot.iter().filter(|s| s.item.class().is_some()).count() >= 2);
        assert!(loot.iter().any(|s| s.item.scroll_group().is_some()));
        assert!(loot.iter().any(|s| s.item == Item::GoldCoin));
    }

    #[test]
    fn gear_suits_the_depth() {
        let mut rng = Rng::new(2);
        for depth in [1u32, 10, 30, 60] {
            for _ in 0..50 {
                let s = random_gear(depth, Fortune::plain(), &mut rng);
                let b = s.item.base().unwrap();
                assert!(b.lvl as u32 <= depth + 4);
                assert!(s.gear.unwrap().level as u32 + 3 >= depth.min(b.lvl as u32));
            }
        }
    }

    #[test]
    fn specials_are_stable_for_a_day() {
        let a = specials(7, 3, 20);
        let b = specials(7, 3, 20);
        assert_eq!(a.len(), b.len());
        assert!(
            a.iter()
                .zip(b.iter())
                .all(|(x, y)| x.0 == y.0 && x.1 == y.1)
        );
        assert!(specials(7, 4, 20)[0].0 != a[0].0 || specials(7, 4, 20)[1].0 != a[1].0);
    }
}
