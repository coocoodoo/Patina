//! Loot: coins, what monsters, chests and pots leave behind, random gear and scrolls, and
//! Burrowby's daily specials.

use super::dungeon::{Foe, biome_seeds};
use super::gear::{self, Class, Gear, Group};
use super::items::{ALL_ITEMS, Base, Item, Stack};
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
    }
}

/// A base item suited to a depth: mostly things that first turn up near it.
pub fn pick_base(depth: u32, group: Option<Group>, rng: &mut Rng) -> Item {
    let d = depth.max(1) as f32;
    let mut items = Vec::new();
    let mut weights = Vec::new();
    for (item, b) in gear_bases() {
        if group.is_some_and(|g| b.class.group() != g) {
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

/// Everything a defeated creature drops.
pub fn foe_loot(
    foe: Foe,
    boss: bool,
    biome: usize,
    depth: u32,
    f: Fortune,
    rng: &mut Rng,
) -> Vec<Stack> {
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
    }
    let d = depth.max(1);
    // Coins: copper up top, silver deeper, gold from guardians.
    let coin_p = if boss { 1.0 } else { 0.7 };
    if rng.chance(coin_p) {
        let base = 2.0 + d as f32 * 0.8;
        let mut c = base * rng.range_f(0.5, 1.5) * f.greed;
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
    let scroll_p = 0.03 + f.luck * 0.05;
    if boss {
        let lucky = Fortune {
            luck: f.luck + 0.6,
            ..f
        };
        for _ in 0..2 + rng.below(2) {
            out.push(random_gear(d + 2, lucky, rng));
        }
        out.push(random_scroll(d + 2, lucky, rng));
        out.push(Stack::new(Item::HeartCrystal, 1));
        out.push(Stack::new(Item::Feather, 1));
        out.push(Stack::new(random_gem(d, rng), 2));
        if rng.chance(0.5) {
            out.push(Stack::new(random_relic(d + 10, rng), 1));
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
    }
    out
}

/// The contents of a treasure chest in the Hollow.
pub fn chest_loot(depth: u32, biome: usize, f: Fortune, rng: &mut Rng) -> Vec<Stack> {
    let d = depth.max(1);
    let mut out = Vec::new();
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
        let food = [Item::HealingTonic, Item::ManaTonic, Item::StaminaTonic];
        out.push(Stack::new(food[rng.below(3)], 1));
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
