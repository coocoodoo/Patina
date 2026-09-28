//! What Bramblewick's shops sell and buy: every shop keeps its own shelves, has a few
//! special pieces each morning, and pays extra for the things it specialises in.

use super::gear::Class;
use super::items::{ALL_ITEMS, Item, Kind, Placeable, Stack};
use super::loot::{self, Fortune};
use super::menus::{ShopTab, shop_goods, shop_seeds};
use super::play::Play;
use super::town::Place;
use crate::util::Rng;

/// The tabs a shop shows.
pub fn tabs(at: Option<Place>) -> &'static [ShopTab] {
    match at {
        None => &[
            ShopTab::Seeds,
            ShopTab::Goods,
            ShopTab::Specials,
            ShopTab::Sell,
        ],
        Some(Place::Seeds) => &[ShopTab::Seeds, ShopTab::Sell],
        Some(Place::Armory | Place::Smithy | Place::Tools | Place::Scrolls | Place::Guild) => {
            &[ShopTab::Goods, ShopTab::Specials, ShopTab::Sell]
        }
        Some(_) => &[ShopTab::Goods, ShopTab::Sell],
    }
}

pub fn tab_name(at: Option<Place>, t: ShopTab) -> &'static str {
    match (at, t) {
        (_, ShopTab::Seeds) => "Seeds",
        (_, ShopTab::Sell) => "Sell",
        (Some(_), ShopTab::Specials) => "Today",
        (None, ShopTab::Specials) => "Specials",
        (Some(Place::Bakery), ShopTab::Goods) => "Treats",
        (Some(Place::Tavern), ShopTab::Goods) => "Menu",
        (Some(Place::Jeweler), ShopTab::Goods) => "Gems",
        (Some(Place::Nook), ShopTab::Goods) => "Decor",
        (Some(Place::Scrolls), ShopTab::Goods) => "Magic",
        (Some(Place::Guild), ShopTab::Goods) => "Supplies",
        (_, ShopTab::Goods) => "Goods",
    }
}

/// Who's behind the counter, and a line for each tab.
pub fn patter(at: Option<Place>, t: ShopTab) -> &'static str {
    match (at, t) {
        (None, ShopTab::Seeds) => "Burrowby: \"New seeds as you go deeper!\"",
        (None, ShopTab::Goods) => "Burrowby: \"Everything a delver needs.\"",
        (None, _) => "Burrowby: \"Fresh finds, every morning!\"",
        (Some(_), ShopTab::Sell) => "\"I'll pay extra for anything in my line of trade!\"",
        (Some(Place::Armory), ShopTab::Specials) => "Hilde: \"Freshly hammered this morning!\"",
        (Some(Place::Armory), _) => "Hilde: \"Protection with personality!\"",
        (Some(Place::Smithy), ShopTab::Specials) => "Garrick: \"...Today's work. Good steel.\"",
        (Some(Place::Smithy), _) => "Garrick: \"...Sharp. Fair.\"",
        (Some(Place::Tools), ShopTab::Specials) => "Nix: \"Prototypes! Barely exploded!\"",
        (Some(Place::Tools), _) => "Nix: \"Tools for every job and some for none!\"",
        (Some(Place::Scrolls), ShopTab::Specials) => "Quill: \"Freshly inked, still drying.\"",
        (Some(Place::Scrolls), _) => "Quill: \"Words have power. Choose well.\"",
        (Some(Place::Guild), ShopTab::Specials) => "Rowan: \"Guild spoils. Earned the hard way.\"",
        (Some(Place::Guild), _) => "Rowan: \"Never delve without these.\"",
        (Some(Place::Seeds), _) => "Posy: \"Every seed is a tiny promise!\"",
        (Some(Place::Bakery), _) => "Mabel: \"Eat something, dearie!\"",
        (Some(Place::Tavern), _) => "Barley: \"Best stew this side of the Hollow!\"",
        (Some(Place::Jeweler), _) => "Opal: \"Everything that sparkles, darling.\"",
        (Some(Place::Nook), _) => "Wren: \"Make your farm feel like a hug!\"",
        (Some(Place::Hall), _) => "",
    }
}

/// How much more than usual a shop pays for something (in percent).
pub fn buy_rate(at: Option<Place>, s: &Stack) -> u64 {
    let k = s.item.def().kind;
    let class = s.item.class();
    let yes = match at {
        Some(Place::Armory) => class.is_some_and(|c| c.is_armor()),
        Some(Place::Smithy) => matches!(class, Some(Class::Sword | Class::Wand | Class::Staff)),
        Some(Place::Tools) => class.is_some_and(|c| c.group() == super::gear::Group::Tool),
        Some(Place::Scrolls) => matches!(k, Kind::Scroll(_)),
        Some(Place::Jeweler) => matches!(k, Kind::Gem | Kind::Relic),
        Some(Place::Seeds) => matches!(k, Kind::Seed(_) | Kind::Produce { .. }),
        Some(Place::Bakery | Place::Tavern) => matches!(k, Kind::Food { .. }),
        Some(Place::Nook) => matches!(k, Kind::Place(_)),
        Some(Place::Guild) => matches!(k, Kind::Material),
        _ => false,
    };
    match (yes, at) {
        (true, Some(Place::Jeweler)) => 150,
        (true, Some(Place::Scrolls)) => 140,
        (true, _) => 130,
        (false, None) => 100,
        (false, Some(_)) => 90,
    }
}

/// Fixed shelves: (what, price in copper), opening up the deeper you've been.
pub fn goods(p: &Play, at: Place) -> Vec<(Item, u32)> {
    let d = p.deepest;
    let mut v: Vec<(Item, u32)> = Vec::new();
    let mut add = |need: u32, item: Item, price: u32| {
        if d >= need {
            v.push((item, price));
        }
    };
    match at {
        Place::Armory => {
            // Basic pieces, plain (rolled when bought).
            for (need, item) in [
                (0, Item::StrawHat),
                (0, Item::CozySweater),
                (0, Item::GrassSkirt),
                (0, Item::LeatherBoots),
                (0, Item::WoodBuckler),
                (3, Item::CopperHelm),
                (3, Item::CopperMail),
                (3, Item::CopperGreaves),
                (3, Item::CopperSabatons),
                (5, Item::CopperShield),
                (12, Item::IronHelm),
                (12, Item::IronPlate),
                (12, Item::IronGreaves),
                (12, Item::IronBoots),
                (14, Item::IronShield),
                (22, Item::GoldShield),
            ] {
                add(need, item, item.def().price * 3 + 20);
            }
        }
        Place::Smithy => {
            for (need, item) in [
                (0, Item::TwigSword),
                (0, Item::TwigWand),
                (0, Item::OakStaff),
                (2, Item::CarrotBlade),
                (2, Item::BubbleWand),
                (4, Item::Baguette),
                (6, Item::SunflowerStaff),
                (8, Item::MightyLeek),
                (10, Item::GlowcapWand),
                (12, Item::MossyStaff),
                (16, Item::CandyWand),
                (20, Item::CrystalWand),
                (24, Item::CapwoodSabre),
                (28, Item::CrystalStaff),
            ] {
                add(need, item, item.def().price * 3 + 20);
            }
        }
        Place::Tools => {
            for (need, item) in [
                (0, Item::SproutHoe),
                (0, Item::DuckCan),
                (0, Item::Sickle),
                (4, Item::CopperHoe),
                (4, Item::CopperSickle),
                (6, Item::BeaverAxe),
                (8, Item::MolePick),
                (12, Item::IronHoe),
                (12, Item::IronCan),
                (12, Item::IronSickle),
                (18, Item::TeapotCan),
                (24, Item::GoldHoe),
                (24, Item::GoldSickle),
            ] {
                add(need, item, item.def().price * 3 + 20);
            }
            add(3, Item::Sprinkler, 400);
            add(15, Item::QualitySprinkler, 1100);
            add(30, Item::CrystalSprinkler, 3200);
        }
        Place::Scrolls => {
            add(0, Item::ManaTonic, 110);
            add(0, Item::EnchantTable, 850);
            add(0, Item::WeaponScroll, 160);
            add(0, Item::ArmorScroll, 160);
            add(0, Item::ToolScroll, 160);
            add(20, Item::WishStar, 6000);
        }
        Place::Jeweler => {
            add(0, Item::Topaz, 200);
            add(0, Item::Amethyst, 220);
            add(5, Item::Ruby, 260);
            add(5, Item::Sapphire, 260);
            add(5, Item::Emerald, 260);
            add(15, Item::Moonstone, 600);
            add(35, Item::StarDiamond, 2400);
            add(25, Item::HeartCrystal, 9000);
        }
        Place::Nook => {
            for (need, item, price) in [
                (0, Item::Torch, 10),
                (0, Item::Fence, 7),
                (0, Item::WoodPath, 4),
                (0, Item::StonePath, 4),
                (0, Item::FlowerPot, 50),
                (0, Item::Bench, 100),
                (0, Item::Chest, 140),
                (0, Item::Lamp, 180),
                (0, Item::Workbench, 120),
                (0, Item::WoodWall, 10),
                (0, Item::StoneWall, 10),
                (0, Item::EnchantTable, 900),
            ] {
                add(need, item, price);
            }
        }
        Place::Seeds => {}
        Place::Bakery => {
            for (item, price) in [
                (Item::FreshBread, 90),
                (Item::BlueberryMuffin, 170),
                (Item::SunflowerCookies, 230),
                (Item::GarlicBread, 200),
                (Item::CarrotCake, 260),
                (Item::BerryTart, 250),
                (Item::Shortcake, 400),
                (Item::PearCrumble, 330),
                (Item::PumpkinPie, 640),
            ] {
                add(0, item, price);
            }
            add(40, Item::StarfruitTart, 1500);
        }
        Place::Tavern => {
            for (need, item, price) in [
                (0, Item::VeggieStew, 150),
                (0, Item::TomatoSoup, 190),
                (0, Item::CornChowder, 240),
                (0, Item::GardenSalad, 220),
                (0, Item::Popcorn, 110),
                (0, Item::BakedPotato, 140),
                (0, Item::HealingTonic, 110),
                (0, Item::StaminaTonic, 110),
                (8, Item::GlowSoup, 210),
                (10, Item::MushroomSkewer, 210),
                (31, Item::EmberCurry, 330),
                (33, Item::LavaLemonade, 280),
                (41, Item::MintTea, 260),
                (51, Item::SpookyChili, 430),
                (30, Item::TruffleRisotto, 1200),
            ] {
                add(need, item, price);
            }
        }
        Place::Guild => {
            add(0, Item::Feather, 260);
            add(0, Item::HealingTonic, 100);
            add(0, Item::StaminaTonic, 100);
            add(0, Item::ManaTonic, 100);
            add(0, Item::Torch, 10);
            add(10, Item::SunStone, 9000);
        }
        Place::Hall => {}
    }
    v
}

/// Every seed Posy stocks: the valley's, and more from the deep the further you've been.
pub fn seeds(p: &Play) -> Vec<(Item, u32)> {
    let mut v = shop_seeds(p);
    for (price, _) in v.iter_mut().map(|e| (&mut e.1, ())) {
        *price = (*price as f32 * 0.9) as u32;
    }
    let d = p.deepest;
    for (need, item, price) in [
        (0, Item::SunflowerSeeds, 70),
        (0, Item::StrawberrySeeds, 80),
        (8, Item::MoonbloomSeeds, 240),
        (18, Item::GeodeGourdSeeds, 180),
        (25, Item::TruffleSpores, 320),
        (38, Item::MagmaMelonSeeds, 240),
        (46, Item::IcePlumSeeds, 200),
        (50, Item::StarfruitSeeds, 450),
    ] {
        if d >= need && !v.iter().any(|(i, _)| *i == item) {
            v.push((item, price));
        }
    }
    v
}

/// The morning's special pieces: gear and scrolls of the shop's trade, rolled for the day.
pub fn specials(p: &Play, at: Place) -> Vec<(Stack, u64)> {
    let mut rng = Rng::new(
        p.seed ^ (p.clock.day as u64).wrapping_mul(0x2545_F491) ^ (at as u64 + 1) * 0xA5A5,
    );
    let depth = p.deepest.max(2);
    let f = Fortune {
        luck: 0.25,
        greed: 1.0,
    };
    let classes: &[Class] = match at {
        Place::Armory => &[
            Class::Head,
            Class::Chest,
            Class::Legs,
            Class::Feet,
            Class::Shield,
        ],
        Place::Smithy => &[Class::Sword, Class::Wand, Class::Staff],
        Place::Tools => &[
            Class::Hoe,
            Class::Can,
            Class::Sickle,
            Class::Axe,
            Class::Pickaxe,
        ],
        Place::Guild => &[
            Class::Sword,
            Class::Wand,
            Class::Staff,
            Class::Head,
            Class::Chest,
            Class::Shield,
        ],
        _ => &[],
    };
    let mut out = Vec::new();
    if at == Place::Scrolls {
        for g in [
            super::gear::Group::Weapon,
            super::gear::Group::Weapon,
            super::gear::Group::Armor,
            super::gear::Group::Armor,
            super::gear::Group::Tool,
            super::gear::Group::Tool,
        ] {
            let s = loot::scroll_of(g, depth, f, &mut rng);
            out.push((s, s.unit_price() * 3 + 40));
        }
        return out;
    }
    let count = if at == Place::Guild { 3 } else { 6 };
    for k in 0..count {
        let class = classes[k % classes.len()];
        let Some(item) = pick_of_class(depth, class, &mut rng) else {
            continue;
        };
        let luck = if at == Place::Guild { 1.2 } else { f.luck };
        let level = (depth as i32 + rng.range(-2, 2)).max(1) as u16;
        let s = loot::roll_gear(item, level, luck, &mut rng);
        let mark = if at == Place::Guild { 5 } else { 3 };
        out.push((s, s.unit_price() * mark + 60));
    }
    out
}

/// A base of one class suited to a depth.
fn pick_of_class(depth: u32, class: Class, rng: &mut Rng) -> Option<Item> {
    let d = depth.max(1) as f32;
    let mut items = Vec::new();
    let mut weights = Vec::new();
    for &i in ALL_ITEMS {
        let Some(b) = i.base() else { continue };
        if b.class != class || b.lvl as f32 > d + 4.0 {
            continue;
        }
        let gap = (d - b.lvl as f32).max(0.0) / 9.0;
        items.push(i);
        weights.push(1.0 / (1.0 + gap * gap));
    }
    if items.is_empty() {
        None
    } else {
        Some(items[rng.weighted(&weights)])
    }
}

/// A shop's rows for a tab: (what, price, sold out).
pub fn rows(p: &Play, at: Option<Place>, tab: ShopTab) -> Vec<(Stack, u64, bool)> {
    let plain = |v: Vec<(Item, u32)>| -> Vec<(Stack, u64, bool)> {
        v.into_iter()
            .map(|(i, price)| (Stack::new(i, 1), price as u64, false))
            .collect()
    };
    match (at, tab) {
        (_, ShopTab::Sell) => Vec::new(),
        (None, ShopTab::Seeds) => plain(shop_seeds(p)),
        (None, ShopTab::Goods) => plain(shop_goods(p)),
        (None, ShopTab::Specials) => loot::specials(p.seed, p.clock.day, p.deepest)
            .into_iter()
            .enumerate()
            .map(|(k, (s, price))| (s, price, p.bought.contains(&k)))
            .collect(),
        (Some(_), ShopTab::Seeds) => plain(seeds(p)),
        (Some(place), ShopTab::Goods) => plain(goods(p, place)),
        (Some(place), ShopTab::Specials) => specials(p, place)
            .into_iter()
            .enumerate()
            .map(|(k, (s, price))| (s, price, p.bought.contains(&special_id(Some(place), k))))
            .collect(),
    }
}

/// The key a special is remembered by once bought today.
pub fn special_id(at: Option<Place>, k: usize) -> usize {
    match at {
        None => k,
        Some(p) => (p as usize + 1) * 100 + k,
    }
}

/// True for placeable furniture (for the Nook's patter).
#[allow(dead_code)]
pub fn is_furniture(i: Item) -> bool {
    matches!(
        i.def().kind,
        Kind::Place(Placeable::Lamp | Placeable::Bench)
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::town::PLACES;

    #[test]
    fn every_shop_has_something_to_sell() {
        let mut p = Play::new(4);
        for deep in [0u32, 20, 60] {
            p.deepest = deep;
            for place in PLACES {
                if place == Place::Hall {
                    continue;
                }
                let any: usize = tabs(Some(place))
                    .iter()
                    .map(|t| rows(&p, Some(place), *t).len())
                    .sum();
                assert!(any > 0, "{:?} is empty at depth {deep}", place);
            }
        }
        // Specialists pay more for their trade.
        let helm = Stack::new(Item::CopperHelm, 1);
        assert!(buy_rate(Some(Place::Armory), &helm) > buy_rate(Some(Place::Bakery), &helm));
        let ruby = Stack::new(Item::Ruby, 1);
        assert_eq!(buy_rate(Some(Place::Jeweler), &ruby), 150);
    }
}
