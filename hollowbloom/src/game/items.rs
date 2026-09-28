//! Items, crops, recipes and inventories.

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use super::gear::{self, Affix, Class, Gear, Group, Rarity, Stat};
use super::home::Furn;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Placeable {
    Chest,
    Torch,
    Lamp,
    Fence,
    WoodPath,
    StonePath,
    Sprinkler(u8),
    StoneWall,
    WoodWall,
    Workbench,
    FlowerPot,
    Bench,
    EnchantTable,
    /// Furniture for inside the house.
    Furniture(Furn),
    /// A rug laid on the house floor (see `home::RUGS`).
    Rug(u8),
    /// A picture hung on the back wall of the house (see `home::ART`).
    WallArt(u8),
}

/// A base item of gear: what it is, the level it first turns up at, its strength in percent
/// and the bonus every copy of it is born with.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Base {
    pub class: Class,
    pub lvl: u16,
    pub mult: u8,
    pub innate: Option<Stat>,
}

/// A food's lingering effect.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Buff {
    pub stat: Stat,
    pub val: i16,
    pub secs: u16,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Kind {
    Material,
    Gem,
    Relic,
    /// Worth this much copper; goes straight into your purse.
    Coin(u32),
    Seed(Crop),
    Produce {
        hp: i32,
        energy: i32,
    },
    Food {
        hp: i32,
        energy: i32,
        mana: i32,
        buff: Option<Buff>,
    },
    Gear(Base),
    Scroll(Group),
    Place(Placeable),
    Feather,
    HeartCrystal,
    SunStone,
    WishStar,
    /// Something someone in town asked you to find or carry. Only turns up while their
    /// request is open.
    Keepsake,
    /// Drunk in one gulp: health, mana and energy back at once.
    Potion {
        hp: i32,
        mana: i32,
        energy: i32,
    },
    /// A catch from the water (see `fish::FISH`).
    Fish,
    /// Papers the house walls in a style (`Wall::Paper`).
    Wallpaper(u8),
    /// Lays a new floor in the house (see `home::FLOORS`).
    Flooring(u8),
    /// Thrown, fizzes, and goes off: monsters, pots and cracked floors beware.
    Bomb,
}

pub struct ItemDef {
    pub key: &'static str,
    pub name: &'static str,
    pub icon: &'static str,
    pub stack: u16,
    pub price: u32,
    pub kind: Kind,
    pub desc: &'static str,
}

macro_rules! items {
    ($( $id:ident = $key:literal, $name:literal, $icon:literal, $stack:expr, $price:expr, $kind:expr, $desc:literal; )*) => {
        #[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, PartialOrd, Ord)]
        #[repr(u16)]
        pub enum Item { $($id),* }
        pub static ITEMS: &[ItemDef] = &[
            $( ItemDef { key: $key, name: $name, icon: $icon, stack: $stack, price: $price, kind: $kind, desc: $desc } ),*
        ];
        pub const ALL_ITEMS: &[Item] = &[ $(Item::$id),* ];
    };
}

const fn g(class: Class, lvl: u16, mult: u8, innate: Option<Stat>) -> Kind {
    Kind::Gear(Base {
        class,
        lvl,
        mult,
        innate,
    })
}

const fn food(hp: i32, energy: i32) -> Kind {
    Kind::Food {
        hp,
        energy,
        mana: 0,
        buff: None,
    }
}

const fn feast(hp: i32, energy: i32, stat: Stat, val: i16, secs: u16) -> Kind {
    Kind::Food {
        hp,
        energy,
        mana: 0,
        buff: Some(Buff { stat, val, secs }),
    }
}

const fn produce(hp: i32, energy: i32) -> Kind {
    Kind::Produce { hp, energy }
}

const fn potion(hp: i32, mana: i32, energy: i32) -> Kind {
    Kind::Potion { hp, mana, energy }
}

use Class::*;
use Kind::{
    Feather as FeatherK, Gem as GemK, HeartCrystal as HeartK, Material, Relic, SunStone as SunK,
};
use Placeable as P;
use Stat as S;

items! {
    // Materials.
    Wood = "wood", "Wood", "wood", 99, 2, Material, "Sturdy timber. Chop trees with an axe.";
    Stone = "stone", "Stone", "stone", 99, 2, Material, "Common stone, for paths, walls and lamps.";
    Fiber = "fiber", "Fiber", "fiber", 99, 1, Material, "Tough grass. Cut weeds to gather it.";
    CopperOre = "copper_ore", "Copper Ore", "copper_ore", 99, 6, Material, "A warm, soft metal from the upper Hollow.";
    IronOre = "iron_ore", "Iron Ore", "iron_ore", 99, 12, Material, "Cold and strong. Found a little deeper.";
    GoldOre = "gold_ore", "Gold Ore", "gold_ore", 99, 25, Material, "It glitters even in the dark.";
    Crystal = "crystal", "Glimmer Shard", "crystal", 99, 40, Material, "Hums softly. Grown in the Crystal Grotto.";
    EmberOre = "ember_ore", "Ember Ore", "ember_ore", 99, 60, Material, "Still warm to the touch.";
    FrostGem = "frost_gem", "Frost Opal", "frost_gem", 99, 80, Material, "Never melts. From the frozen depths.";
    Amber = "amber", "Amber", "amber", 99, 50, Material, "Old sunlight, trapped in resin.";
    SlimeGel = "slime_gel", "Slime Gel", "slime_gel", 99, 4, Material, "Wobbly. Surprisingly useful.";
    BatWing = "bat_wing", "Bat Wing", "bat_wing", 99, 8, Material, "Leathery and light.";
    Bone = "bone", "Old Bone", "bone", 99, 6, Material, "Someone's, once.";
    Spore = "spore", "Glow Spore", "spore", 99, 10, Material, "A pinch of living light.";
    CrabShell = "crab_shell", "Crab Shell", "crab_shell", 99, 14, Material, "Pearly, and still a little pinchy.";
    BeetleShell = "beetle_shell", "Beetle Shell", "beetle_shell", 99, 16, Material, "Shiny, tough and very well polished.";
    ShroomCap = "shroom_cap", "Shroomling Cap", "shroom_cap", 99, 12, Material, "It left this behind, very politely.";
    WispDust = "wisp_dust", "Wisp Dust", "wisp_dust", 99, 18, Material, "Glitters in your pocket for days.";
    ImpHorn = "imp_horn", "Imp Horn", "imp_horn", 99, 22, Material, "Warm, pointy and a little smug.";
    Ectoplasm = "ectoplasm", "Ectoplasm", "ectoplasm", 99, 24, Material, "Cold, wobbly and faintly giggling.";
    GolemHeart = "golem_heart", "Golem Heart", "golem_heart", 99, 40, Material, "A pebble that still beats.";
    Bomb = "bomb", "Bomb", "bomb", 20, 30, Kind::Bomb, "Throw it and stand back! It blasts monsters, pots and crates, and opens up cracked floors in the Hollow.";
    GraveDust = "grave_dust", "Grave Dust", "grave_dust", 99, 16, Material, "Shaken out of a zombie's pockets. Mostly zombie.";
    GoblinTooth = "goblin_tooth", "Goblin Tooth", "goblin_tooth", 99, 20, Material, "A goblin lost this. It will want it back.";
    Chitin = "chitin", "Bug Chitin", "chitin", 99, 12, Material, "A tough, glossy plate off a Hollow bug.";

    // Gems: scroll ink, and worth a pretty coin.
    Ruby = "ruby", "Ruby", "ruby", 99, 60, GemK, "Red as a winterberry. Scribes weapon scrolls.";
    Sapphire = "sapphire", "Sapphire", "sapphire", 99, 60, GemK, "Deep blue and calm. Scribes armor scrolls.";
    Emerald = "emerald", "Emerald", "emerald", 99, 60, GemK, "Green as new leaves. Scribes tool scrolls.";
    Topaz = "topaz", "Topaz", "topaz", 99, 45, GemK, "A drop of honey that turned to stone.";
    Amethyst = "amethyst", "Amethyst", "amethyst", 99, 50, GemK, "Grows in the quiet corners of the Hollow.";
    Moonstone = "moonstone", "Moonstone", "moonstone", 99, 120, GemK, "Holds a little moonlight inside.";
    StarDiamond = "star_diamond", "Star Diamond", "star_diamond", 99, 400, GemK, "Very rare. Very, very sparkly.";

    // Relics: lost treasures of the Hollow. Burrowby pays well for them.
    LostButton = "lost_button", "Lost Button", "lost_button", 99, 30, Relic, "Somebody's coat is missing this.";
    GlassMarble = "glass_marble", "Glass Marble", "glass_marble", 99, 45, Relic, "The swirly kind. The best kind.";
    RubberDuck = "rubber_duck", "Rubber Duck", "rubber_duck", 99, 60, Relic, "Squeak.";
    ChippedTeacup = "chipped_teacup", "Chipped Teacup", "chipped_teacup", 99, 70, Relic, "Tea time, down in the dark.";
    ToyBoat = "toy_boat", "Toy Boat", "toy_boat", 99, 90, Relic, "It has sailed the underground seas.";
    AncientCoin = "ancient_coin", "Ancient Coin", "ancient_coin", 99, 120, Relic, "Nobody spends these any more. Burrowby buys them.";
    GoldenAcorn = "golden_acorn", "Golden Acorn", "golden_acorn", 99, 150, Relic, "A squirrel's life savings.";
    MusicBox = "music_box", "Music Box", "music_box", 99, 200, Relic, "It still plays half a lullaby.";
    StarFossil = "star_fossil", "Star Fossil", "star_fossil", 99, 260, Relic, "A starfish that fell from the sky, long ago.";
    TinyCrown = "tiny_crown", "Tiny Crown", "tiny_crown", 99, 320, Relic, "Fit for a very small king.";
    MoonPearl = "moon_pearl", "Moon Pearl", "moon_pearl", 99, 450, Relic, "Glows softly when you hold it close.";
    DragonScale = "dragon_scale", "Dragon Scale", "dragon_scale", 99, 600, Relic, "From a very small dragon, probably.";

    // Coins go straight into your purse: 10 copper make a silver, 10 silver a gold.
    CopperCoin = "copper_coin", "Copper Coin", "coin_copper", 999, 1, Kind::Coin(1), "Worth 1 copper.";
    SilverCoin = "silver_coin", "Silver Coin", "coin_silver", 999, 10, Kind::Coin(10), "Worth 10 copper.";
    GoldCoin = "gold_coin", "Gold Coin", "coin_gold", 999, 100, Kind::Coin(100), "Worth 10 silver.";

    // Seeds.
    TurnipSeeds = "turnip_seeds", "Turnip Seeds", "turnip_seeds", 99, 10, Kind::Seed(Crop::Turnip), "Plant on tilled soil. Grows in 4 days.";
    CarrotSeeds = "carrot_seeds", "Cave Carrot Seeds", "carrot_seeds", 99, 15, Kind::Seed(Crop::CaveCarrot), "A hardy root from the Hollow. 3 days.";
    GlowcapSpores = "glowcap_spores", "Glowcap Spores", "glowcap_spores", 99, 25, Kind::Seed(Crop::Glowcap), "Glows at night once grown. 5 days.";
    BerrySeeds = "berry_seeds", "Crystal Berry Seeds", "berry_seeds", 99, 35, Kind::Seed(Crop::CrystalBerry), "6 days, then fruits every 3 days.";
    MelonSeeds = "melon_seeds", "Moss Melon Seeds", "melon_seeds", 99, 40, Kind::Seed(Crop::MossMelon), "Slow and sweet. 8 days.";
    PumpkinSeeds = "pumpkin_seeds", "Spore Pumpkin Seeds", "pumpkin_seeds", 99, 50, Kind::Seed(Crop::SporePumpkin), "A fungal pumpkin. 9 days.";
    PepperSeeds = "pepper_seeds", "Ember Pepper Seeds", "pepper_seeds", 99, 45, Kind::Seed(Crop::EmberPepper), "5 days, then fruits every 2 days.";
    LilyBulb = "lily_bulb", "Frost Lily Bulb", "lily_bulb", 99, 60, Kind::Seed(Crop::FrostLily), "A cold, lovely flower. 7 days.";
    MoonbloomSeeds = "moonbloom_seeds", "Moonbloom Seeds", "moonbloom_seeds", 99, 120, Kind::Seed(Crop::Moonbloom), "Legendary. Glows. 12 days.";
    SeedPotato = "seed_potato", "Seed Potato", "potato_seeds", 99, 20, Kind::Seed(Crop::Potato), "Humble and happy. 5 days.";
    RadishSeeds = "radish_seeds", "Radish Seeds", "radish_seeds", 99, 12, Kind::Seed(Crop::Radish), "Pink and peppy. 3 days.";
    CabbageSeeds = "cabbage_seeds", "Cabbage Seeds", "cabbage_seeds", 99, 35, Kind::Seed(Crop::Cabbage), "Takes its time. 7 days.";
    TomatoSeeds = "tomato_seeds", "Tomato Seeds", "tomato_seeds", 99, 30, Kind::Seed(Crop::Tomato), "6 days, then fruits every 3 days.";
    StrawberrySeeds = "strawberry_seeds", "Strawberry Seeds", "strawberry_seeds", 99, 45, Kind::Seed(Crop::Strawberry), "7 days, then fruits every 3 days.";
    WheatSeeds = "wheat_seeds", "Wheat Seeds", "wheat_seeds", 99, 8, Kind::Seed(Crop::Wheat), "For bread and baking. 4 days.";
    CornSeeds = "corn_seeds", "Corn Seeds", "corn_seeds", 99, 40, Kind::Seed(Crop::Corn), "9 days, then every 4 days.";
    BlueberrySeeds = "blueberry_seeds", "Blueberry Seeds", "blueberry_seeds", 99, 50, Kind::Seed(Crop::Blueberry), "8 days, then lots of berries every 2.";
    EggplantSeeds = "eggplant_seeds", "Eggplant Seeds", "eggplant_seeds", 99, 30, Kind::Seed(Crop::Eggplant), "6 days, then every 4 days.";
    GarlicBulb = "garlic_bulb", "Garlic Bulb", "garlic_seeds", 99, 20, Kind::Seed(Crop::Garlic), "Keeps the ghosts away. 4 days.";
    SunflowerSeeds = "sunflower_seeds", "Sunflower Seeds", "sunflower_seeds", 99, 40, Kind::Seed(Crop::Sunflower), "Grows taller than you. 8 days.";
    RoseSeeds = "rose_seeds", "Rose Seeds", "rose_seeds", 99, 35, Kind::Seed(Crop::Rose), "A lovely gift. 6 days.";
    PeaSeeds = "pea_seeds", "Sweet Pea Seeds", "pea_seeds", 99, 18, Kind::Seed(Crop::SweetPea), "5 days, then pods every 2 days.";
    MossberrySeeds = "mossberry_seeds", "Mossberry Seeds", "mossberry_seeds", 99, 30, Kind::Seed(Crop::Mossberry), "From the Burrows. 5 days, then every 2.";
    BunnyrootSeeds = "bunnyroot_seeds", "Bunnyroot Seeds", "bunnyroot_seeds", 99, 25, Kind::Seed(Crop::Bunnyroot), "Its leaves look like ears. 4 days.";
    PrismPearSeeds = "prism_pear_seeds", "Prism Pear Seeds", "prism_pear_seeds", 99, 70, Kind::Seed(Crop::PrismPear), "From the Grotto. 8 days, then every 4.";
    GeodeGourdSeeds = "geode_gourd_seeds", "Geode Gourd Seeds", "geode_gourd_seeds", 99, 80, Kind::Seed(Crop::GeodeGourd), "Sparkles inside. 9 days.";
    PuffballSpores = "puffball_spores", "Puffball Spores", "puffball_spores", 99, 30, Kind::Seed(Crop::Puffball), "Poof! 4 days.";
    JellySpores = "jelly_spores", "Jelly Shroom Spores", "jelly_spores", 99, 60, Kind::Seed(Crop::JellyShroom), "Wobbles and glows. 6 days.";
    TruffleSpores = "truffle_spores", "Truffle Spores", "truffle_spores", 99, 150, Kind::Seed(Crop::Truffle), "A treasure of the Fungal Hollow. 10 days.";
    FlameTulipBulb = "flame_tulip_bulb", "Flame Tulip Bulb", "flame_tulip_seeds", 99, 70, Kind::Seed(Crop::FlameTulip), "Its petals flicker. 7 days.";
    LavaLemonSeeds = "lava_lemon_seeds", "Lava Lemon Seeds", "lava_lemon_seeds", 99, 75, Kind::Seed(Crop::LavaLemon), "6 days, then every 3 days.";
    MagmaMelonSeeds = "magma_melon_seeds", "Magma Melon Seeds", "magma_melon_seeds", 99, 110, Kind::Seed(Crop::MagmaMelon), "Hot to hold. 10 days.";
    SnowPeaSeeds = "snow_pea_seeds", "Snow Pea Seeds", "snow_pea_seeds", 99, 55, Kind::Seed(Crop::SnowPea), "Crisp and cool. 5 days, then every 2.";
    IcePlumSeeds = "ice_plum_seeds", "Ice Plum Seeds", "ice_plum_seeds", 99, 90, Kind::Seed(Crop::IcePlum), "8 days, then every 3 days.";
    FrostMintSeeds = "frost_mint_seeds", "Frost Mint Seeds", "frost_mint_seeds", 99, 50, Kind::Seed(Crop::FrostMint), "Makes your breath sparkle. 4 days.";
    StarfruitSeeds = "starfruit_seeds", "Starfruit Seeds", "starfruit_seeds", 99, 200, Kind::Seed(Crop::Starfruit), "Shaped like a wish. Glows. 12 days.";
    GhostPepperSeeds = "ghost_pepper_seeds", "Ghost Pepper Seeds", "ghost_pepper_seeds", 99, 90, Kind::Seed(Crop::GhostPepper), "Boo! 7 days, then every 3.";
    AncientGrainSeeds = "ancient_grain_seeds", "Ancient Grain Seeds", "ancient_grain_seeds", 99, 100, Kind::Seed(Crop::AncientGrain), "From a forgotten farm. 6 days.";

    // Produce.
    Turnip = "turnip", "Turnip", "turnip", 99, 35, produce(8, 20), "Crisp and humble.";
    CaveCarrot = "cave_carrot", "Cave Carrot", "cave_carrot", 99, 30, produce(14, 14), "Earthy and sweet.";
    Glowcap = "glowcap", "Glowcap", "glowcap", 99, 60, produce(10, 24), "A luminous mushroom.";
    CrystalBerry = "crystal_berry", "Crystal Berry", "crystal_berry", 99, 45, produce(16, 10), "Crunchy, cold, and faintly fizzy.";
    MossMelon = "moss_melon", "Moss Melon", "moss_melon", 99, 170, produce(30, 40), "Heavy with juice.";
    SporePumpkin = "spore_pumpkin", "Spore Pumpkin", "spore_pumpkin", 99, 220, produce(36, 50), "Soft pink flesh. Smells like rain.";
    EmberPepper = "ember_pepper", "Ember Pepper", "ember_pepper", 99, 80, produce(22, 18), "Warms you from the inside.";
    FrostLily = "frost_lily", "Frost Lily", "frost_lily", 99, 140, produce(5, 30), "Petals like snow.";
    Moonbloom = "moonbloom", "Moonbloom", "moonbloom", 99, 420, produce(40, 60), "Blooms only for the patient.";
    Potato = "potato", "Potato", "potato", 99, 45, produce(12, 22), "Lumpy, lovely, endlessly useful.";
    Radish = "radish", "Radish", "radish", 99, 32, produce(8, 16), "A peppery little crunch.";
    Cabbage = "cabbage", "Cabbage", "cabbage", 99, 110, produce(20, 30), "Leaves upon leaves upon leaves.";
    Tomato = "tomato", "Tomato", "tomato", 99, 50, produce(14, 16), "Sun-warm and juicy.";
    Strawberry = "strawberry", "Strawberry", "strawberry", 99, 70, produce(16, 20), "The sweetest thing in the valley.";
    Wheat = "wheat", "Wheat", "wheat", 99, 20, Material, "Golden stalks. Bake it into something nice.";
    Corn = "corn", "Corn", "corn", 99, 55, produce(14, 24), "Every kernel a tiny sun.";
    Blueberry = "blueberry", "Blueberry", "blueberry", 99, 30, produce(6, 10), "Pop them like sweets.";
    Eggplant = "eggplant", "Eggplant", "eggplant", 99, 60, produce(18, 18), "Glossy, purple and proud.";
    Garlic = "garlic", "Garlic", "garlic", 99, 50, produce(6, 12), "Strong enough to scare a skeleton.";
    Sunflower = "sunflower", "Sunflower", "sunflower", 99, 110, produce(4, 30), "It follows you around the room.";
    Rose = "rose", "Rose", "rose", 99, 90, produce(2, 20), "Smells like a summer evening.";
    SweetPea = "sweet_pea", "Sweet Pea", "sweet_pea", 99, 28, produce(6, 12), "Five little friends in a pod.";
    Mossberry = "mossberry", "Mossberry", "mossberry", 99, 34, produce(8, 12), "Tart little berries from the Burrows.";
    Bunnyroot = "bunnyroot", "Bunnyroot", "bunnyroot", 99, 45, produce(12, 16), "It seems to be listening.";
    PrismPear = "prism_pear", "Prism Pear", "prism_pear", 99, 95, produce(22, 20), "A pear you can almost see through.";
    GeodeGourd = "geode_gourd", "Geode Gourd", "geode_gourd", 99, 240, produce(28, 34), "Crack it open: sparkles inside.";
    Puffball = "puffball", "Puffball", "puffball", 99, 48, produce(8, 20), "Pop it for a puff of glitter.";
    JellyShroom = "jelly_shroom", "Jelly Shroom", "jelly_shroom", 99, 110, produce(18, 26), "Bouncy, wobbly and sweet.";
    Truffle = "truffle", "Truffle", "truffle", 99, 480, produce(20, 40), "Chefs across the valley whisper about these.";
    FlameTulip = "flame_tulip", "Flame Tulip", "flame_tulip", 99, 170, produce(4, 36), "Its petals flicker like candles.";
    LavaLemon = "lava_lemon", "Lava Lemon", "lava_lemon", 99, 85, produce(10, 30), "Sour enough to melt stone.";
    MagmaMelon = "magma_melon", "Magma Melon", "magma_melon", 99, 360, produce(40, 50), "Hot to hold, sweet to eat.";
    SnowPea = "snow_pea", "Snow Pea", "snow_pea", 99, 60, produce(10, 18), "Crisp, cool and a little frosty.";
    IcePlum = "ice_plum", "Ice Plum", "ice_plum", 99, 120, produce(18, 22), "Frosted on the outside, jam on the inside.";
    FrostMint = "frost_mint", "Frost Mint", "frost_mint", 99, 90, produce(4, 34), "Makes your breath sparkle.";
    Starfruit = "starfruit", "Starfruit", "starfruit", 99, 700, produce(50, 70), "Shaped like a wish.";
    GhostPepper = "ghost_pepper", "Ghost Pepper", "ghost_pepper", 99, 130, produce(22, 26), "Boo! (It's very hot.)";
    AncientGrain = "ancient_grain", "Ancient Grain", "ancient_grain", 99, 150, produce(12, 30), "It remembers an older sun.";

    // Cooking.
    VeggieStew = "veggie_stew", "Veggie Stew", "veggie_stew", 20, 120, feast(50, 50, S::Defense, 3, 120), "Hearty. Tastes like home.";
    GlowSoup = "glow_soup", "Glowcap Soup", "glow_soup", 20, 170, feast(60, 60, S::Spirit, 3, 180), "It glows. Just a little.";
    EmberCurry = "ember_curry", "Ember Curry", "ember_curry", 20, 260, feast(90, 70, S::Burn, 15, 180), "Spicy enough to light a torch.";
    BerryTart = "berry_tart", "Berry Tart", "berry_tart", 20, 200, feast(70, 40, S::Crit, 6, 180), "Flaky crust, jewel-bright berries.";
    PumpkinPie = "pumpkin_pie", "Pumpkin Pie", "pumpkin_pie", 20, 520, feast(120, 100, S::Vitality, 30, 300), "A feast in a slice.";
    HealingTonic = "healing_tonic", "Healing Tonic", "healing_tonic", 20, 90, food(80, 0), "Restores 80 HP.";
    StaminaTonic = "stamina_tonic", "Stamina Tonic", "stamina_tonic", 20, 90, food(0, 80), "Restores 80 energy.";
    ManaTonic = "mana_tonic", "Mana Tonic", "mana_tonic", 20, 90, Kind::Food { hp: 0, energy: 0, mana: 60, buff: None }, "Restores 60 mana. Tastes like blueberries.";
    BakedPotato = "baked_potato", "Baked Potato", "baked_potato", 20, 110, feast(45, 40, S::Defense, 4, 180), "Fluffy inside, crispy outside.";
    FreshBread = "fresh_bread", "Fresh Bread", "fresh_bread", 20, 70, feast(30, 50, S::Stamina, 20, 240), "Still warm from the oven.";
    GardenSalad = "garden_salad", "Garden Salad", "garden_salad", 20, 180, feast(45, 45, S::Swift, 10, 180), "Crunchy, bright and full of vitamins.";
    TomatoSoup = "tomato_soup", "Tomato Soup", "tomato_soup", 20, 150, feast(55, 40, S::Regen, 3, 180), "Best with bread, on a rainy day.";
    Shortcake = "shortcake", "Strawberry Shortcake", "shortcake", 20, 320, feast(80, 60, S::Luck, 12, 240), "Cream, berries and a little luck.";
    BlueberryMuffin = "blueberry_muffin", "Blueberry Muffin", "blueberry_muffin", 20, 130, feast(40, 60, S::Haste, 8, 180), "Bursting with berries.";
    CornChowder = "corn_chowder", "Corn Chowder", "corn_chowder", 20, 190, feast(60, 55, S::Defense, 6, 240), "Thick, creamy and golden.";
    GarlicBread = "garlic_bread", "Garlic Bread", "garlic_bread", 20, 160, feast(50, 30, S::Damage, 6, 180), "Ghosts hate it. You love it.";
    MintTea = "mint_tea", "Frost Mint Tea", "mint_tea", 20, 200, Kind::Food { hp: 20, energy: 60, mana: 40, buff: Some(Buff { stat: S::Spirit, val: 4, secs: 240 }) }, "Cool, calm and very clear-headed.";
    Popcorn = "popcorn", "Popcorn", "popcorn", 20, 90, feast(15, 40, S::Dodge, 5, 120), "Pop! Pop! Pop!";
    CarrotCake = "carrot_cake", "Carrot Cake", "carrot_cake", 20, 210, feast(70, 50, S::Swift, 12, 240), "Hop to it!";
    MushroomSkewer = "mushroom_skewer", "Mushroom Skewer", "mushroom_skewer", 20, 170, feast(45, 45, S::Wisdom, 20, 240), "Charred, chewy and a bit magic.";
    TruffleRisotto = "truffle_risotto", "Truffle Risotto", "truffle_risotto", 20, 900, feast(150, 120, S::Luck, 20, 300), "The fanciest dish in the valley.";
    StarfruitTart = "starfruit_tart", "Starfruit Tart", "starfruit_tart", 20, 1100, feast(140, 120, S::Crit, 12, 300), "Tastes like a wish come true.";
    LavaLemonade = "lava_lemonade", "Lava Lemonade", "lava_lemonade", 20, 220, feast(20, 80, S::Burn, 18, 240), "It fizzes angrily.";
    PlumPudding = "plum_pudding", "Ice Plum Pudding", "plum_pudding", 20, 300, feast(70, 70, S::Chill, 18, 240), "A wobbly, frosty treat.";
    SpookyChili = "spooky_chili", "Spooky Chili", "spooky_chili", 20, 340, feast(90, 60, S::Shock, 18, 240), "Hot enough to make your hair stand up.";
    SunflowerCookies = "sunflower_cookies", "Sunflower Cookies", "sunflower_cookies", 20, 190, feast(30, 70, S::Greed, 25, 240), "Crunchy, golden and good for business.";
    PearCrumble = "pear_crumble", "Prism Pear Crumble", "pear_crumble", 20, 260, feast(60, 60, S::Block, 8, 240), "Glitters when the light hits it.";

    // Cooking with fish.
    FishAndChips = "fish_and_chips", "Fish & Chips", "fish_and_chips", 20, 180, feast(60, 50, S::Damage, 5, 240), "Crispy, golden and wrapped in paper.";
    GrilledTrout = "grilled_trout", "Grilled Trout", "grilled_trout", 20, 190, feast(55, 45, S::Regen, 3, 240), "Smoky, flaky and served with garlic.";
    MinnowFritters = "minnow_fritters", "Minnow Fritters", "minnow_fritters", 20, 120, feast(40, 50, S::Swift, 8, 180), "Crunchy little bites. Eat them hot!";
    Sushi = "sushi", "Seaweed Sushi", "sushi", 20, 230, Kind::Food { hp: 40, energy: 50, mana: 30, buff: Some(Buff { stat: S::Wisdom, val: 20, secs: 240 }) }, "Neatly rolled and very clever.";
    FishStew = "fish_stew", "Fisherman's Stew", "fish_stew", 20, 260, feast(70, 60, S::Vitality, 20, 300), "A big warm bowl after a long day on the water.";
    CarpCurry = "carp_curry", "Carp Curry", "carp_curry", 20, 250, feast(80, 60, S::Burn, 12, 240), "Hot enough to boil the pond.";
    SalmonSteak = "salmon_steak", "Salmon Steak", "salmon_steak", 20, 520, feast(110, 80, S::Crit, 10, 300), "Pink, peppery and perfectly seared.";
    CrayfishBoil = "crayfish_boil", "Crayfish Boil", "crayfish_boil", 20, 220, feast(60, 60, S::Defense, 6, 240), "Messy. Worth it.";
    CaveSkewer = "cave_skewer", "Cavefish Skewer", "cave_skewer", 20, 210, feast(50, 40, S::Spirit, 4, 240), "Glowcaps and cavefish on a stick.";
    Ceviche = "ceviche", "Crystal Ceviche", "ceviche", 20, 300, feast(60, 70, S::Crit, 8, 240), "Zingy lemon and glittering fish.";
    SmeltPie = "smelt_pie", "Snow Smelt Pie", "smelt_pie", 20, 380, feast(80, 60, S::Chill, 16, 240), "Little fish peeking out of the crust.";
    EelKebab = "eel_kebab", "Lava Eel Kebab", "eel_kebab", 20, 560, feast(90, 70, S::Burn, 20, 300), "Still sizzling from the lava.";
    FishTacos = "fish_tacos", "Fish Tacos", "fish_tacos", 20, 330, feast(55, 55, S::Haste, 8, 240), "Rainbow trout, corn and tomato. Fiesta!";
    EmperorPlatter = "emperor_platter", "Emperor's Platter", "emperor_platter", 20, 2400, feast(220, 160, S::Luck, 25, 420), "Golden carp and truffles, fit for a king.";

    // Weapons: swords.
    TwigSword = "twig_sword", "Twig Sword", "twig_sword", 1, 8, g(Sword, 1, 85, None), "A stick with big dreams.";
    Sword0 = "sword0", "Rusty Sword", "sword0", 1, 10, g(Sword, 1, 95, None), "Old, but it remembers how to fight.";
    CarrotBlade = "carrot_blade", "Carrot Blade", "carrot_blade", 1, 30, g(Sword, 3, 95, Some(S::Crit)), "Crunchy, and surprisingly sharp.";
    Baguette = "baguette", "Hero's Baguette", "baguette", 1, 40, g(Sword, 5, 92, Some(S::Lifesteal)), "Crusty on the outside. Deadly on the outside too.";
    Sword1 = "sword1", "Copper Sword", "sword1", 1, 80, g(Sword, 8, 100, None), "A bright, honest blade.";
    MightyLeek = "mighty_leek", "Mighty Leek", "mighty_leek", 1, 110, g(Sword, 12, 104, Some(S::Regen)), "Swing it with pride. Full of vitamins.";
    Sword2 = "sword2", "Iron Sword", "sword2", 1, 200, g(Sword, 16, 105, None), "Heavy and dependable.";
    CapwoodSabre = "capwood_sabre", "Capwood Sabre", "capwood_sabre", 1, 280, g(Sword, 22, 106, Some(S::Lifesteal)), "Carved from a very old mushroom stem.";
    Sword3 = "sword3", "Golden Sword", "sword3", 1, 450, g(Sword, 26, 108, Some(S::Greed)), "Shines like a sunrise.";
    Sword4 = "sword4", "Crystal Blade", "sword4", 1, 900, g(Sword, 36, 112, Some(S::Crit)), "Sings when it strikes.";
    FrostFang = "frost_fang", "Frost Fang", "frost_fang", 1, 1200, g(Sword, 44, 113, Some(S::Chill)), "Bitingly cold, even in the Ember Depths.";
    Sword5 = "sword5", "Ember Blade", "sword5", 1, 1600, g(Sword, 48, 118, Some(S::Burn)), "Forged in the deep fire.";
    BoneSabre = "bone_sabre", "Bone Sabre", "bone_sabre", 1, 1900, g(Sword, 54, 118, Some(S::Lifesteal)), "It rattles cheerfully when swung.";
    StarlightSword = "starlight_sword", "Starlight Sword", "starlight_sword", 1, 2600, g(Sword, 60, 125, Some(S::Shock)), "A falling star, caught and sharpened.";

    // Weapons: wands shoot magic bolts.
    TwigWand = "twig_wand", "Twig Wand", "twig_wand", 1, 12, g(Wand, 1, 90, None), "Point it and hope.";
    BubbleWand = "bubble_wand", "Bubble Wand", "bubble_wand", 1, 45, g(Wand, 4, 95, Some(S::Focus)), "Blows bubbles that bonk.";
    GlowcapWand = "glowcap_wand", "Glowcap Wand", "glowcap_wand", 1, 110, g(Wand, 9, 100, Some(S::Spirit)), "A mushroom on a stick. It hums.";
    CandyWand = "candy_wand", "Sugarstick Wand", "candy_wand", 1, 160, g(Wand, 14, 100, Some(S::Haste)), "Sweet magic. Sticky fingers.";
    CrystalWand = "crystal_wand", "Crystal Wand", "crystal_wand", 1, 320, g(Wand, 20, 105, Some(S::Crit)), "Focuses light into a sharp little star.";
    EmberWand = "ember_wand", "Ember Wand", "ember_wand", 1, 700, g(Wand, 32, 110, Some(S::Burn)), "Warm to the touch. Hot to the touched.";
    FrostWand = "frost_wand", "Frost Wand", "frost_wand", 1, 1000, g(Wand, 42, 110, Some(S::Chill)), "Leaves a trail of snowflakes.";
    StarWand = "star_wand", "Star Wand", "star_wand", 1, 1500, g(Wand, 52, 115, Some(S::Shock)), "Twinkle, twinkle.";
    MoonpetalWand = "moonpetal_wand", "Moonpetal Wand", "moonpetal_wand", 1, 2400, g(Wand, 60, 120, Some(S::Focus)), "Made from a Moonbloom's sleepy petal.";

    // Weapons: staffs set off magic blasts.
    OakStaff = "oak_staff", "Oak Staff", "oak_staff", 1, 20, g(Staff, 2, 90, None), "Sturdy, knobbly and a little magic.";
    SunflowerStaff = "sunflower_staff", "Sunflower Staff", "sunflower_staff", 1, 90, g(Staff, 7, 95, Some(S::Regen)), "It always turns to face the nearest foe.";
    MossyStaff = "mossy_staff", "Mossy Staff", "mossy_staff", 1, 160, g(Staff, 12, 100, Some(S::Spirit)), "Something is growing on it. Something nice.";
    MushroomStaff = "mushroom_staff", "Mushroom Staff", "mushroom_staff", 1, 380, g(Staff, 24, 105, Some(S::Luck)), "Puffs a little spore cloud when happy.";
    CrystalStaff = "crystal_staff", "Crystal Staff", "crystal_staff", 1, 600, g(Staff, 30, 108, Some(S::Crit)), "A whole geode on a stick.";
    EmberStaff = "ember_staff", "Ember Staff", "ember_staff", 1, 1000, g(Staff, 40, 112, Some(S::Burn)), "Crackles like a hearth.";
    FrostStaff = "frost_staff", "Frost Staff", "frost_staff", 1, 1300, g(Staff, 46, 112, Some(S::Chill)), "Snow falls wherever you point it.";
    MoonbloomStaff = "moonbloom_staff", "Moonbloom Staff", "moonbloom_staff", 1, 2500, g(Staff, 58, 120, Some(S::Focus)), "Opens its petals only for battle.";

    // Armour: shields.
    PotLid = "pot_lid", "Pot Lid", "pot_lid", 1, 8, g(Shield, 1, 85, Some(S::Block)), "Clang!";
    WoodBuckler = "wood_buckler", "Wooden Buckler", "wood_buckler", 1, 30, g(Shield, 3, 95, None), "A round shield of honest planks.";
    LeafShield = "leaf_shield", "Leaf Shield", "leaf_shield", 1, 45, g(Shield, 6, 95, Some(S::Dodge)), "A very big, very brave leaf.";
    CopperShield = "copper_shield", "Copper Shield", "copper_shield", 1, 90, g(Shield, 10, 100, Some(S::Block)), "Polished until you can see your smile.";
    TurtleShell = "turtle_shell", "Turtle Shell", "turtle_shell", 1, 140, g(Shield, 12, 105, Some(S::Defense)), "The turtle moved out. You moved in.";
    IronShield = "iron_shield", "Iron Shield", "iron_shield", 1, 220, g(Shield, 18, 105, Some(S::Block)), "Solid as a stone wall.";
    MushroomShield = "mushroom_shield", "Mushroom Shield", "mushroom_shield", 1, 300, g(Shield, 22, 102, Some(S::Regen)), "Bouncy. Things bounce off it.";
    GoldShield = "gold_shield", "Golden Shield", "gold_shield", 1, 480, g(Shield, 28, 108, Some(S::Greed)), "Blinding in the sunlight.";
    CrystalAegis = "crystal_aegis", "Crystal Aegis", "crystal_aegis", 1, 950, g(Shield, 38, 112, Some(S::Thorns)), "Its points are sharp on purpose.";
    FrostWard = "frost_ward", "Frost Ward", "frost_ward", 1, 1250, g(Shield, 46, 115, Some(S::Vitality)), "Cold to hold, warm to be behind.";
    EmberBulwark = "ember_bulwark", "Ember Bulwark", "ember_bulwark", 1, 1700, g(Shield, 52, 118, Some(S::Thorns)), "Too hot to hug.";

    // Armour: headgear.
    StrawHat = "straw_hat", "Straw Hat", "straw_hat", 1, 10, g(Head, 1, 90, Some(S::Stamina)), "Keeps the sun off and the smile on.";
    FlowerCrown = "flower_crown", "Flower Crown", "flower_crown", 1, 20, g(Head, 2, 85, Some(S::Regen)), "Picked fresh this morning.";
    LeafCap = "leaf_cap", "Leaf Cap", "leaf_cap", 1, 25, g(Head, 3, 90, Some(S::Forage)), "Smells like spring.";
    CatHood = "cat_hood", "Cat-Ear Hood", "cat_hood", 1, 60, g(Head, 5, 95, Some(S::Luck)), "Nya.";
    FrogHood = "frog_hood", "Frog Hood", "frog_hood", 1, 80, g(Head, 8, 95, Some(S::Dodge)), "Ribbit.";
    CopperHelm = "copper_helm", "Copper Helm", "copper_helm", 1, 90, g(Head, 9, 100, None), "Shiny, dented, loved.";
    MushroomCap = "mushroom_cap", "Mushroom Cap", "mushroom_cap", 1, 110, g(Head, 10, 100, Some(S::Wisdom)), "Very fashionable in the Fungal Hollow.";
    MinerHat = "miner_hat", "Miner's Hat", "miner_hat", 1, 160, g(Head, 14, 100, Some(S::Power)), "Comes with a tiny lamp.";
    IronHelm = "iron_helm", "Iron Helm", "iron_helm", 1, 220, g(Head, 17, 105, Some(S::Defense)), "Clonks nicely when you nod.";
    WizardHat = "wizard_hat", "Wizard Hat", "wizard_hat", 1, 300, g(Head, 20, 100, Some(S::Wisdom)), "Pointy, starry and very serious.";
    BunnyHood = "bunny_hood", "Bunny Hood", "bunny_hood", 1, 380, g(Head, 24, 100, Some(S::Swift)), "Hop hop.";
    CrystalCirclet = "crystal_circlet", "Crystal Circlet", "crystal_circlet", 1, 800, g(Head, 34, 108, Some(S::Spirit)), "Rings like a bell in the wind.";
    FrostHelm = "frost_helm", "Frost Helm", "frost_helm", 1, 1150, g(Head, 44, 112, Some(S::Vitality)), "With little icicle horns.";
    EmberCrown = "ember_crown", "Ember Crown", "ember_crown", 1, 1500, g(Head, 48, 115, Some(S::Crit)), "Keeps your thoughts toasty.";

    // Armour: chest.
    CozySweater = "cozy_sweater", "Cozy Sweater", "cozy_sweater", 1, 15, g(Chest, 1, 85, Some(S::Regen)), "Knitted with love, and a few dropped stitches.";
    FarmerTunic = "farmer_tunic", "Farmer's Tunic", "farmer_tunic", 1, 20, g(Chest, 2, 90, Some(S::Stamina)), "Pockets for seeds. Pockets for snacks.";
    LeafTunic = "leaf_tunic", "Leaf Tunic", "leaf_tunic", 1, 40, g(Chest, 6, 95, Some(S::Dodge)), "Rustles when you run.";
    LeatherVest = "leather_vest", "Leather Vest", "leather_vest", 1, 70, g(Chest, 8, 100, None), "Worn soft by many adventures.";
    CopperMail = "copper_mail", "Copper Mail", "copper_mail", 1, 100, g(Chest, 10, 100, Some(S::Defense)), "Jingles cheerfully.";
    FrogRaincoat = "frog_raincoat", "Frog Raincoat", "frog_raincoat", 1, 150, g(Chest, 12, 100, Some(S::Luck)), "Splish splash.";
    MageRobe = "mage_robe", "Mage Robe", "mage_robe", 1, 220, g(Chest, 16, 95, Some(S::Wisdom)), "Stars stitched on by hand.";
    IronPlate = "iron_plate", "Iron Plate", "iron_plate", 1, 260, g(Chest, 18, 105, Some(S::Defense)), "Heavy, but so reassuring.";
    WoollyPoncho = "woolly_poncho", "Woolly Poncho", "woolly_poncho", 1, 340, g(Chest, 22, 102, Some(S::Vitality)), "Warm as a hug.";
    CrystalMail = "crystal_mail", "Crystal Mail", "crystal_mail", 1, 950, g(Chest, 36, 112, Some(S::Thorns)), "Glitters with every step.";
    FrostCoat = "frost_coat", "Frost Coat", "frost_coat", 1, 1200, g(Chest, 44, 112, Some(S::Vitality)), "Fluffy collar included.";
    EmberPlate = "ember_plate", "Ember Plate", "ember_plate", 1, 1650, g(Chest, 48, 118, Some(S::Damage)), "Glows faintly orange in the dark.";
    StarRobe = "star_robe", "Starry Robe", "star_robe", 1, 2200, g(Chest, 56, 115, Some(S::Spirit)), "The night sky, tailored.";

    // Armour: legs.
    PatchedTrousers = "patched_trousers", "Patched Trousers", "patched_trousers", 1, 12, g(Legs, 1, 85, Some(S::Stamina)), "More patch than trouser.";
    GrassSkirt = "grass_skirt", "Grass Skirt", "grass_skirt", 1, 25, g(Legs, 3, 90, Some(S::Swift)), "Swishes when you walk.";
    LeatherLeggings = "leather_leggings", "Leather Leggings", "leather_leggings", 1, 60, g(Legs, 7, 100, None), "Good for kneeling in the garden.";
    CopperGreaves = "copper_greaves", "Copper Greaves", "copper_greaves", 1, 90, g(Legs, 10, 100, Some(S::Defense)), "Clink, clink, clink.";
    PumpkinBloomers = "pumpkin_bloomers", "Pumpkin Bloomers", "pumpkin_bloomers", 1, 130, g(Legs, 12, 95, Some(S::Luck)), "Orange, puffy, perfect.";
    IronGreaves = "iron_greaves", "Iron Greaves", "iron_greaves", 1, 230, g(Legs, 18, 105, Some(S::Defense)), "Your knees have never felt safer.";
    StarryLeggings = "starry_leggings", "Starry Leggings", "starry_leggings", 1, 330, g(Legs, 24, 100, Some(S::Dodge)), "Twinkle as you walk.";
    CrystalGreaves = "crystal_greaves", "Crystal Greaves", "crystal_greaves", 1, 900, g(Legs, 36, 110, Some(S::Vitality)), "Tinkle when you kneel.";
    FrostLeggings = "frost_leggings", "Frost Leggings", "frost_leggings", 1, 1150, g(Legs, 44, 112, Some(S::Swift)), "Lined with snow-rabbit fluff.";
    EmberGreaves = "ember_greaves", "Ember Greaves", "ember_greaves", 1, 1600, g(Legs, 50, 118, Some(S::Haste)), "Warm knees, warm heart.";

    // Armour: boots.
    RainBoots = "rain_boots", "Rain Boots", "rain_boots", 1, 12, g(Feet, 1, 90, Some(S::Forage)), "Made for puddles.";
    FrogSlippers = "frog_slippers", "Frog Slippers", "frog_slippers", 1, 30, g(Feet, 3, 90, Some(S::Swift)), "They croak when you tiptoe.";
    LeatherBoots = "leather_boots", "Leather Boots", "leather_boots", 1, 55, g(Feet, 6, 100, None), "Broken in just right.";
    BunnySlippers = "bunny_slippers", "Bunny Slippers", "bunny_slippers", 1, 80, g(Feet, 8, 95, Some(S::Dodge)), "Soft as a whisper.";
    CopperSabatons = "copper_sabatons", "Copper Sabatons", "copper_sabatons", 1, 90, g(Feet, 10, 100, Some(S::Defense)), "Clank, clank.";
    IronBoots = "iron_boots", "Iron Boots", "iron_boots", 1, 210, g(Feet, 18, 105, Some(S::Defense)), "Stomp stomp.";
    FeatherBoots = "feather_boots", "Feather Boots", "feather_boots", 1, 420, g(Feet, 26, 102, Some(S::Swift)), "Lighter than air. Nearly.";
    CrystalBoots = "crystal_boots", "Crystal Boots", "crystal_boots", 1, 880, g(Feet, 36, 110, Some(S::Dodge)), "Like walking on bells.";
    FrostWalkers = "frost_walkers", "Frost Walkers", "frost_walkers", 1, 1100, g(Feet, 44, 112, Some(S::Swift)), "Never slip on ice again.";
    EmberTreads = "ember_treads", "Ember Treads", "ember_treads", 1, 1550, g(Feet, 50, 118, Some(S::Haste)), "They leave tiny warm footprints.";

    // Tools: hoes.
    Hoe = "hoe", "Hoe", "hoe", 1, 10, g(Class::Hoe, 1, 100, None), "Tills grass and soil for planting.";
    SproutHoe = "sprout_hoe", "Sprout Hoe", "sprout_hoe", 1, 60, g(Class::Hoe, 4, 100, Some(S::Growth)), "A tiny sprout grows from the handle.";
    CopperHoe = "copper_hoe", "Copper Hoe", "copper_hoe", 1, 120, g(Class::Hoe, 10, 100, Some(S::Frugal)), "Tills without a fuss.";
    IronHoe = "iron_hoe", "Iron Hoe", "iron_hoe", 1, 260, g(Class::Hoe, 20, 100, Some(S::Haste)), "Tills a little row at a time.";
    GoldHoe = "gold_hoe", "Golden Hoe", "gold_hoe", 1, 500, g(Class::Hoe, 30, 100, Some(S::Forage)), "Turns up the most interesting things.";
    CrystalHoe = "crystal_hoe", "Crystal Hoe", "crystal_hoe", 1, 950, g(Class::Hoe, 40, 100, Some(S::Growth)), "The soil sparkles after it.";
    EmberHoe = "ember_hoe", "Ember Hoe", "ember_hoe", 1, 1600, g(Class::Hoe, 50, 100, Some(S::Frugal)), "Warms the earth as it tills.";

    // Tools: watering cans.
    Can0 = "can0", "Watering Can", "can0", 1, 10, g(Can, 1, 100, None), "Refill it at the pond.";
    DuckCan = "duck_can", "Duck Can", "duck_can", 1, 70, g(Can, 5, 100, Some(S::Growth)), "Quack. (It pours from the beak.)";
    Can1 = "can1", "Copper Can", "can1", 1, 120, g(Can, 10, 100, Some(S::Capacity)), "Holds a good long drink.";
    TeapotCan = "teapot_can", "Teapot Can", "teapot_can", 1, 200, g(Can, 16, 105, Some(S::Frugal)), "Tea for the turnips.";
    IronCan = "iron_can", "Iron Can", "iron_can", 1, 260, g(Can, 20, 105, Some(S::Capacity)), "Heavy when full, and it is always full.";
    FrogCan = "frog_can", "Frog Can", "frog_can", 1, 380, g(Can, 26, 105, Some(S::Reach)), "It has waited for rain all its life.";
    Can2 = "can2", "Crystal Can", "can2", 1, 600, g(Can, 36, 110, Some(S::Growth)), "The water it pours glitters.";
    CloudCan = "cloud_can", "Raincloud Can", "cloud_can", 1, 1500, g(Can, 50, 115, Some(S::Reach)), "A little storm cloud in a tin.";

    // Tools: sickles harvest and cut all around.
    Sickle = "sickle", "Sickle", "sickle", 1, 15, g(Class::Sickle, 1, 100, None), "Cuts grass and harvests ripe crops in a patch.";
    CopperSickle = "copper_sickle", "Copper Sickle", "copper_sickle", 1, 110, g(Class::Sickle, 10, 100, Some(S::Bounty)), "Swish!";
    IronSickle = "iron_sickle", "Iron Sickle", "iron_sickle", 1, 250, g(Class::Sickle, 20, 105, Some(S::Haste)), "Harvest day, every day.";
    GoldSickle = "gold_sickle", "Golden Sickle", "gold_sickle", 1, 520, g(Class::Sickle, 30, 108, Some(S::Forage)), "Finds seeds in every tuft.";
    MoonSickle = "moon_sickle", "Moon Sickle", "moon_sickle", 1, 1400, g(Class::Sickle, 45, 115, Some(S::Bounty)), "Curved like a sleepy moon.";

    // Tools: axes.
    Axe0 = "axe0", "Rusty Axe", "axe0", 1, 10, g(Axe, 1, 100, None), "Chops trees, stumps and logs.";
    Axe1 = "axe1", "Copper Axe", "axe1", 1, 80, g(Axe, 8, 100, None), "Chops a little faster.";
    BeaverAxe = "beaver_axe", "Beaver Axe", "beaver_axe", 1, 160, g(Axe, 14, 105, Some(S::Bounty)), "Has teeth marks. Friendly ones.";
    Axe2 = "axe2", "Iron Axe", "axe2", 1, 200, g(Axe, 16, 105, None), "Chops much faster.";
    Axe3 = "axe3", "Golden Axe", "axe3", 1, 450, g(Axe, 26, 108, Some(S::Greed)), "Timber!";
    Axe4 = "axe4", "Crystal Axe", "axe4", 1, 900, g(Axe, 36, 112, Some(S::Haste)), "Trees fall politely.";
    Axe5 = "axe5", "Ember Axe", "axe5", 1, 1600, g(Axe, 48, 118, Some(S::Power)), "One swing, usually.";

    // Tools: pickaxes.
    Pick0 = "pick0", "Rusty Pickaxe", "pick0", 1, 10, g(Pickaxe, 1, 100, None), "Breaks rocks, walls and placed things.";
    Pick1 = "pick1", "Copper Pickaxe", "pick1", 1, 80, g(Pickaxe, 8, 100, None), "Mines faster than rust.";
    Pick2 = "pick2", "Iron Pickaxe", "pick2", 1, 200, g(Pickaxe, 16, 105, None), "Makes short work of stone.";
    MolePick = "mole_pick", "Mole Pick", "mole_pick", 1, 300, g(Pickaxe, 20, 105, Some(S::Forage)), "Burrowby's old digger. It knows where the gems are.";
    Pick3 = "pick3", "Golden Pickaxe", "pick3", 1, 450, g(Pickaxe, 26, 108, Some(S::Greed)), "Fast and shiny.";
    Pick4 = "pick4", "Crystal Pickaxe", "pick4", 1, 900, g(Pickaxe, 36, 112, Some(S::Haste)), "Cuts rock like butter.";
    Pick5 = "pick5", "Ember Pickaxe", "pick5", 1, 1600, g(Pickaxe, 48, 118, Some(S::Power)), "Nothing is too hard.";

    // Tools: fishing rods. Garrick sells the plain ones; the Hollow's water folk carry the rest.
    BambooRod = "bamboo_rod", "Bamboo Rod", "bamboo_rod", 1, 20, g(Rod, 1, 100, None), "Springy bamboo, some string and a hopeful hook.";
    WillowRod = "willow_rod", "Willow Rod", "willow_rod", 1, 90, g(Rod, 6, 102, Some(S::Lure)), "Bends like a dancer. Fish love a dancer.";
    OakRod = "oak_rod", "Sturdy Oak Rod", "oak_rod", 1, 220, g(Rod, 14, 105, Some(S::Line)), "It has never once snapped, and it's very proud of that.";
    CoralRod = "coral_rod", "Coral Rod", "coral_rod", 1, 300, g(Rod, 8, 106, Some(S::Angler)), "Grown, not carved, in a pool under the Grotto.";
    KelpRod = "kelp_rod", "Kelpie Rod", "kelp_rod", 1, 360, g(Rod, 12, 106, Some(S::Lure)), "Smells of the sea. Fish follow it like a song.";
    PearlRod = "pearl_rod", "Pearl Rod", "pearl_rod", 1, 520, g(Rod, 18, 108, Some(S::Treasure)), "A pearl for a reel. Sunken things come to it.";
    CrystalRod = "crystal_rod", "Crystal Rod", "crystal_rod", 1, 800, g(Rod, 24, 110, Some(S::Angler)), "Its line glitters, and rare fish can't resist.";
    JellyRod = "jelly_rod", "Jellyfish Rod", "jelly_rod", 1, 950, g(Rod, 28, 110, Some(S::Line)), "Wobbly, stretchy and impossible to snap.";
    EmberRod = "ember_rod", "Ember Rod", "ember_rod", 1, 1400, g(Rod, 34, 114, Some(S::Treasure)), "Heat-proof. Fish in lava, if you dare.";
    FrostRod = "frost_rod", "Frost Rod", "frost_rod", 1, 1700, g(Rod, 44, 116, Some(S::Lure)), "Cold enough to keep your catch fresh.";
    StarRod = "star_rod", "Starlight Rod", "star_rod", 1, 2600, g(Rod, 52, 120, Some(S::Angler)), "Casts a line of starlight. Heat-proof, too.";
    LeviathanRod = "leviathan_rod", "Leviathan Rod", "leviathan_rod", 1, 4000, g(Rod, 60, 125, Some(S::Line)), "Made from one scale of something very, very big. Heat-proof.";

    // Scrolls, one enchantment each.
    WeaponScroll = "weapon_scroll", "Weapon Scroll", "weapon_scroll", 1, 40, Kind::Scroll(Group::Weapon), "Bind it to a sword, wand or staff at an enchanting table.";
    ArmorScroll = "armor_scroll", "Armor Scroll", "armor_scroll", 1, 40, Kind::Scroll(Group::Armor), "Bind it to a shield, hat, armor or boots at an enchanting table.";
    ToolScroll = "tool_scroll", "Tool Scroll", "tool_scroll", 1, 40, Kind::Scroll(Group::Tool), "Bind it to a hoe, can, sickle, axe or pickaxe at an enchanting table.";

    // Placeables.
    Chest = "chest", "Chest", "chest", 20, 30, Kind::Place(P::Chest), "Stores 30 stacks. Place anywhere.";
    Torch = "torch", "Torch", "torch", 99, 5, Kind::Place(P::Torch), "A warm light for dark places.";
    Lamp = "lamp", "Glowcap Lamp", "lamp", 20, 60, Kind::Place(P::Lamp), "A soft, cozy glow for your farm.";
    Fence = "fence", "Fence", "fence", 99, 3, Kind::Place(P::Fence), "Keeps paths tidy. Joins its neighbours.";
    WoodPath = "wood_path", "Wood Floor", "wood_path", 99, 2, Kind::Place(P::WoodPath), "Lay planks on the ground.";
    StonePath = "stone_path", "Stone Path", "stone_path", 99, 2, Kind::Place(P::StonePath), "A cobbled path.";
    Sprinkler = "sprinkler", "Sprinkler", "sprinkler", 20, 60, Kind::Place(P::Sprinkler(0)), "Waters the 4 tiles next to it every morning.";
    QualitySprinkler = "quality_sprinkler", "Quality Sprinkler", "quality_sprinkler", 20, 150, Kind::Place(P::Sprinkler(1)), "Waters the 8 tiles around it every morning.";
    CrystalSprinkler = "crystal_sprinkler", "Crystal Sprinkler", "crystal_sprinkler", 20, 400, Kind::Place(P::Sprinkler(2)), "Waters a 5x5 square every morning.";
    StoneWall = "stone_wall", "Stone Wall", "stone_wall", 99, 4, Kind::Place(P::StoneWall), "A solid block. Build anything.";
    WoodWall = "wood_wall", "Wood Wall", "wood_wall", 99, 4, Kind::Place(P::WoodWall), "A solid block of planks.";
    Workbench = "workbench", "Workbench", "workbench", 5, 40, Kind::Place(P::Workbench), "Opens the crafting menu. Pretty, too.";
    FlowerPot = "flower_pot", "Flower Pot", "flower_pot", 20, 20, Kind::Place(P::FlowerPot), "A little colour for the porch.";
    Bench = "bench", "Bench", "bench", 20, 30, Kind::Place(P::Bench), "Sit a while.";
    EnchantTable = "enchant_table", "Enchanting Table", "enchant_table", 5, 300, Kind::Place(P::EnchantTable), "Binds scroll enchantments to your gear.";

    // Furniture and decor for the house. Everything you place raises your charisma.
    CozyBed = "cozy_bed", "Cozy Bed", "cozy_bed", 5, 300, Kind::Place(P::Furniture(Furn::Bed)), "A patchwork quilt and a very soft pillow. Sleep here.";
    CanopyBed = "canopy_bed", "Canopy Bed", "canopy_bed", 5, 1800, Kind::Place(P::Furniture(Furn::CanopyBed)), "Curtains, posts and dreams fit for royalty.";
    WoodStove = "wood_stove", "Wood Stove", "wood_stove", 5, 250, Kind::Place(P::Furniture(Furn::Stove)), "A little iron stove. Cook your dishes here.";
    CopperRange = "copper_range", "Copper Range", "copper_range", 5, 2200, Kind::Place(P::Furniture(Furn::Range)), "A gleaming kitchen range. Sometimes cooks a second helping.";
    KitchenCounter = "kitchen_counter", "Kitchen Counter", "kitchen_counter", 10, 110, Kind::Place(P::Furniture(Furn::Counter)), "Somewhere to chop, knead and lean.";
    Icebox = "icebox", "Icebox", "icebox", 5, 240, Kind::Place(P::Furniture(Furn::Icebox)), "Keeps the milk cold and the cheese honest.";
    RoundTable = "round_table", "Round Table", "round_table", 10, 100, Kind::Place(P::Furniture(Furn::RoundTable)), "Perfect for tea for two.";
    DiningTable = "dining_table", "Dining Table", "dining_table", 5, 280, Kind::Place(P::Furniture(Furn::DiningTable)), "Seats the whole family, and a cat.";
    WoodenChair = "wooden_chair", "Wooden Chair", "wooden_chair", 20, 45, Kind::Place(P::Furniture(Furn::Chair)), "Four legs, one back, no complaints.";
    Armchair = "armchair", "Armchair", "armchair", 10, 300, Kind::Place(P::Furniture(Furn::Armchair)), "It hugs you back.";
    Sofa = "sofa", "Comfy Sofa", "sofa", 5, 750, Kind::Place(P::Furniture(Furn::Sofa)), "Big enough for a nap and a half.";
    Bookshelf = "bookshelf", "Bookshelf", "bookshelf", 10, 380, Kind::Place(P::Furniture(Furn::Bookshelf)), "Full of stories. Some of them are about you.";
    Wardrobe = "wardrobe", "Wardrobe", "wardrobe", 5, 330, Kind::Place(P::Furniture(Furn::Wardrobe)), "No, there's no secret kingdom in the back. Probably.";
    Dresser = "dresser", "Dresser", "dresser", 10, 200, Kind::Place(P::Furniture(Furn::Dresser)), "Drawers for socks and secrets.";
    FloorLamp = "floor_lamp", "Floor Lamp", "floor_lamp", 10, 150, Kind::Place(P::Furniture(Furn::FloorLamp)), "A warm pool of light for reading corners.";
    Candelabra = "candelabra", "Candelabra", "candelabra", 10, 320, Kind::Place(P::Furniture(Furn::Candelabra)), "Three little flames, very elegant.";
    Fireplace = "fireplace", "Stone Fireplace", "fireplace", 5, 1600, Kind::Place(P::Furniture(Furn::Fireplace)), "Crackles all night. Warm your hands for a while.";
    PottedFern = "potted_fern", "Potted Fern", "potted_fern", 20, 60, Kind::Place(P::Furniture(Furn::Fern)), "Green, leafy and easy to please.";
    PottedCactus = "potted_cactus", "Potted Cactus", "potted_cactus", 20, 70, Kind::Place(P::Furniture(Furn::Cactus)), "Low maintenance. High attitude.";
    FlowerVase = "flower_vase", "Flower Vase", "flower_vase", 20, 100, Kind::Place(P::Furniture(Furn::Vase)), "Fresh flowers on a little stand.";
    GrandfatherClock = "grandfather_clock", "Grandfather Clock", "grandfather_clock", 5, 860, Kind::Place(P::Furniture(Furn::Clock)), "Tick, tock. It chimes on the hour.";
    Piano = "piano", "Upright Piano", "piano", 5, 2900, Kind::Place(P::Furniture(Furn::Piano)), "Plays a little tune when you sit down at it.";
    Globe = "globe", "Globe", "globe", 10, 250, Kind::Place(P::Furniture(Furn::Globe)), "Give it a spin. The whole world, at your fingertips.";
    Telescope = "telescope", "Telescope", "telescope", 5, 600, Kind::Place(P::Furniture(Furn::Telescope)), "Look at the stars on a clear night. You might get lucky.";
    PlushBunny = "plush_bunny", "Plush Bunny", "plush_bunny", 20, 60, Kind::Place(P::Furniture(Furn::Plush)), "Squeak! Very huggable.";
    FishBowl = "fish_bowl", "Fish Bowl", "fish_bowl", 10, 150, Kind::Place(P::Furniture(Furn::FishBowl)), "A round little bowl for two fish.";
    FishTank = "fish_tank", "Fish Tank", "fish_tank", 5, 800, Kind::Place(P::Furniture(Furn::FishTank)), "A glass tank for six fish. They'll love it.";
    RoundRug = "round_rug", "Round Rug", "round_rug", 10, 130, Kind::Place(P::Rug(0)), "A soft, round rug. Lay it anywhere on the floor.";
    StripedRug = "striped_rug", "Striped Rug", "striped_rug", 10, 160, Kind::Place(P::Rug(1)), "Blue and white, like a summer awning.";
    StarRug = "star_rug", "Starry Rug", "star_rug", 10, 230, Kind::Place(P::Rug(2)), "A night sky for your feet.";
    FishRug = "fish_rug", "Fish Rug", "fish_rug", 10, 200, Kind::Place(P::Rug(3)), "Shaped like a very flat, very happy fish.";
    MeadowPainting = "meadow_painting", "Meadow Painting", "meadow_painting", 10, 200, Kind::Place(P::WallArt(0)), "Hang it on the back wall of your house.";
    SunsetPainting = "sunset_painting", "Sunset Painting", "sunset_painting", 10, 260, Kind::Place(P::WallArt(1)), "Golden hour, all day long.";
    StarryPainting = "starry_painting", "Starry Night", "starry_painting", 10, 400, Kind::Place(P::WallArt(2)), "Swirly stars. Painted by a very sleepy artist.";
    TrophyFish = "trophy_fish", "Trophy Fish", "trophy_fish", 10, 500, Kind::Place(P::WallArt(3)), "The one that didn't get away. Hang it with pride.";
    CozyWallpaper = "cozy_wallpaper", "Cozy Wallpaper", "cozy_wallpaper", 10, 50, Kind::Wallpaper(8), "Warm stripes. Use it in your house to re-paper the walls.";
    MintWallpaper = "mint_wallpaper", "Mint Wallpaper", "mint_wallpaper", 10, 90, Kind::Wallpaper(2), "Fresh mint stripes. Use it in your house.";
    StarryWallpaper = "starry_wallpaper", "Starry Wallpaper", "starry_wallpaper", 10, 120, Kind::Wallpaper(3), "Purple as midnight, sprinkled with stars.";
    HoneyWallpaper = "honey_wallpaper", "Honey Panels", "honey_wallpaper", 10, 110, Kind::Wallpaper(4), "Golden wood panels. Very snug.";
    BlossomWallpaper = "blossom_wallpaper", "Blossom Wallpaper", "blossom_wallpaper", 10, 110, Kind::Wallpaper(5), "Little pink flowers everywhere.";
    OceanWallpaper = "ocean_wallpaper", "Ocean Wallpaper", "ocean_wallpaper", 10, 130, Kind::Wallpaper(6), "Deep teal, like the bottom of the pond.";
    RoseWallpaper = "rose_wallpaper", "Rose Wallpaper", "rose_wallpaper", 10, 110, Kind::Wallpaper(7), "Peachy-pink with tiny roses.";
    OakFloor = "oak_floor", "Oak Floor", "oak_floor", 10, 50, Kind::Flooring(0), "Honest wooden boards. Use it in your house.";
    CheckerFloor = "checker_floor", "Checkered Tiles", "checker_floor", 10, 110, Kind::Flooring(1), "Like a cafe, or a very big board game.";
    RedCarpet = "red_carpet", "Red Carpet", "red_carpet", 10, 130, Kind::Flooring(2), "Roll it out. You're a star.";
    VioletCarpet = "violet_carpet", "Violet Carpet", "violet_carpet", 10, 130, Kind::Flooring(3), "Soft and purple, like a wizard's study.";
    TealCarpet = "teal_carpet", "Teal Carpet", "teal_carpet", 10, 130, Kind::Flooring(4), "Deep teal with golden flecks.";
    StoneFloor = "stone_floor", "Stone Floor", "stone_floor", 10, 90, Kind::Flooring(5), "Cool cobbles, like a castle kitchen.";

    // Special.
    Feather = "feather", "Homeward Feather", "feather", 20, 150, FeatherK, "Use in the Hollow to float back home.";
    HeartCrystal = "heart_crystal", "Heart Crystal", "heart_crystal", 20, 500, HeartK, "Use to raise max HP by 10.";
    SunStone = "sun_stone", "Sun Stone", "stamina_gem", 20, 500, SunK, "Use to raise max energy by 15.";
    WishStar = "wish_star", "Wish Star", "wish_star", 20, 500, Kind::WishStar, "Use to raise max mana by 10.";

    // Keepsakes: quest things that only exist while someone is waiting for them.
    Letter = "letter", "Sealed Letter", "letter", 9, 0, Kind::Keepsake, "A letter to deliver. Your quest log says to whom.";
    Parcel = "parcel", "Parcel", "parcel", 9, 0, Kind::Keepsake, "A tidy parcel tied with string.";
    PieBasket = "pie_basket", "Pie Basket", "pie_basket", 9, 0, Kind::Keepsake, "Still warm. Smells of cinnamon.";
    Locket = "locket", "Albert's Locket", "locket", 1, 0, Kind::Keepsake, "A little gold locket with a faded picture inside.";
    Teddy = "teddy", "Patches the Teddy", "teddy", 1, 0, Kind::Keepsake, "A much-loved bear with one button eye.";
    MapScrap = "map_scrap", "Map Scrap", "map_scrap", 9, 0, Kind::Keepsake, "Part of an old delver's map.";
    SlimeHeart = "slime_heart", "Slime Heart", "slime_heart", 99, 0, Kind::Keepsake, "Wobbly, warm and oddly cute.";
    Moonmoss = "moonmoss", "Moonmoss", "moonmoss", 99, 0, Kind::Keepsake, "Moss that glows the colour of moonlight.";
    BatFang = "bat_fang", "Bat Fang", "bat_fang", 99, 0, Kind::Keepsake, "Tiny, pointy, and still a bit scary.";
    SingingCrystal = "singing_crystal", "Singing Crystal", "singing_crystal", 99, 0, Kind::Keepsake, "It hums a soft note when you hold it.";
    RainbowSpore = "rainbow_spore", "Rainbow Spore", "rainbow_spore", 99, 0, Kind::Keepsake, "A spore that shimmers every colour at once.";
    EmberHeart = "ember_heart", "Ember Heart", "ember_heart", 99, 0, Kind::Keepsake, "An imp's little heart of fire. Warm to the touch.";
    FrostBlossom = "frost_blossom", "Frost Blossom", "frost_blossom", 99, 0, Kind::Keepsake, "A flower made of ice that never melts.";
    GhostLantern = "ghost_lantern", "Ghost Lantern", "ghost_lantern", 99, 0, Kind::Keepsake, "It glows with a gentle, friendly light.";
    StarShard = "star_shard", "Fallen Star Shard", "star_shard", 99, 0, Kind::Keepsake, "A splinter of a star that fell into the Hollow.";
    SongPage = "song_page", "Lost Song Page", "song_page", 9, 0, Kind::Keepsake, "A page of music, a little singed.";
    KnightBadge = "knight_badge", "Old Knight's Badge", "knight_badge", 1, 0, Kind::Keepsake, "The crest of the Frost Guard.";
    GoldenWhisk = "golden_whisk", "Golden Whisk", "golden_whisk", 1, 0, Kind::Keepsake, "A family heirloom. Makes the fluffiest cream.";
    Mailbag = "mailbag", "Lost Mailbag", "mailbag", 1, 0, Kind::Keepsake, "Full of letters. One of them is for you!";
    SeedPod = "seed_pod", "Ancient Seed Pod", "seed_pod", 9, 0, Kind::Keepsake, "Something very old is sleeping inside.";
    Cog = "cog", "Clockwork Cog", "cog", 99, 0, Kind::Keepsake, "A brass cog from the old clockworks below.";
    WishLeaf = "wish_leaf", "Wishing Leaf", "wish_leaf", 9, 0, Kind::Keepsake, "A leaf that drifted down from somewhere impossible.";
    RoyalJelly = "royal_jelly", "Royal Jelly", "royal_jelly", 1, 0, Kind::Keepsake, "The King Slime's crown jewel. Sticky.";
    MatriarchPearl = "matriarch_pearl", "Matriarch's Pearl", "matriarch_pearl", 1, 0, Kind::Keepsake, "Pulled from the Crystal Matriarch's shell.";
    CapwoodAcorn = "capwood_acorn", "Capwood Acorn", "capwood_acorn", 1, 0, Kind::Keepsake, "Old Capwood's last acorn.";
    EmberGem = "ember_gem", "Ember Lord's Gem", "ember_gem", 1, 0, Kind::Keepsake, "It burns without heat.";
    FrostCore = "frost_core", "Frost Core", "frost_core", 1, 0, Kind::Keepsake, "The heart of the Frost Colossus.";
    WardenKey = "warden_key", "Warden's Key", "warden_key", 1, 0, Kind::Keepsake, "Opens something, somewhere.";
    PocketWatch = "pocket_watch", "Pocket Watch", "pocket_watch", 1, 0, Kind::Keepsake, "Still ticking, after all these years.";
    Spectacles = "spectacles", "Reading Spectacles", "spectacles", 1, 0, Kind::Keepsake, "Round, gold-rimmed and very smudged.";
    BellClapper = "bell_clapper", "Bell Clapper", "bell_clapper", 1, 0, Kind::Keepsake, "The heart of the town's clock bell.";
    SpringStone = "spring_stone", "Spring Stone", "spring_stone", 9, 0, Kind::Keepsake, "Water wells up from it forever.";
    GlowOil = "glow_oil", "Glow Oil", "glow_oil", 99, 0, Kind::Keepsake, "Wisp light in a bottle. Lamps love it.";
    GlowBeetle = "glow_beetle", "Glow Beetle", "glow_beetle", 99, 0, Kind::Keepsake, "A sleepy beetle whose back glows green.";
    PhoenixQuill = "phoenix_quill", "Phoenix Quill", "phoenix_quill", 1, 0, Kind::Keepsake, "It writes in firelight.";

    // Potions and what goes into them.
    Vial = "vial", "Glass Vial", "vial", 99, 6, Material, "An empty little bottle, waiting for a potion.";
    Heartleaf = "heartleaf", "Heartleaf", "heartleaf", 99, 14, Material, "A red-veined herb that mends what ails you. Grows in the wild bushes.";
    SmallHealthPotion = "small_health_potion", "Small Health Potion", "hp_potion_s", 30, 30, potion(45, 0, 0), "A swig of red. Restores 45 HP at once.";
    HealthPotion = "health_potion", "Medium Health Potion", "hp_potion_m", 30, 90, potion(110, 0, 0), "Restores 110 HP at once.";
    LargeHealthPotion = "large_health_potion", "Large Health Potion", "hp_potion_l", 30, 260, potion(260, 0, 0), "A big bubbling jug. Restores 260 HP at once.";
    SmallManaPotion = "small_mana_potion", "Small Mana Potion", "mp_potion_s", 30, 35, potion(0, 35, 0), "Tastes of blueberries and starlight. Restores 35 mana.";
    ManaPotion = "mana_potion", "Medium Mana Potion", "mp_potion_m", 30, 100, potion(0, 85, 0), "Restores 85 mana at once.";
    LargeManaPotion = "large_mana_potion", "Large Mana Potion", "mp_potion_l", 30, 280, potion(0, 200, 0), "Restores 200 mana. Fizzes on your tongue.";
    SmallEnergyPotion = "small_energy_potion", "Small Energy Potion", "ep_potion_s", 30, 30, potion(0, 0, 45), "Zesty! Restores 45 energy.";
    EnergyPotion = "energy_potion", "Medium Energy Potion", "ep_potion_m", 30, 90, potion(0, 0, 110), "Restores 110 energy at once.";
    LargeEnergyPotion = "large_energy_potion", "Large Energy Potion", "ep_potion_l", 30, 250, potion(15, 0, 250), "Liquid sunshine. Restores 250 energy and a little health.";

    // Fishing: bait, what comes up with the fish, and the fish themselves (see `fish::FISH`).
    Bait = "bait", "Bait", "bait", 99, 3, Material, "Wriggly! Fish bite much sooner while you carry some.";
    Seaweed = "seaweed", "Seaweed", "seaweed", 99, 8, Material, "Salty and slippery. Good in sushi.";
    SoggyBoot = "soggy_boot", "Soggy Boot", "soggy_boot", 99, 1, Material, "Somebody is walking around with one boot.";
    TinCan = "tin_can", "Tin Can", "tin_can", 99, 2, Material, "Rattly, rusty rubbish.";
    SunnyMinnow = "sunny_minnow", "Sunny Minnow", "sunny_minnow", 99, 18, Kind::Fish, "A tiny flicker of gold that loves warm, shallow water.";
    PondPerch = "pond_perch", "Pond Perch", "pond_perch", 99, 35, Kind::Fish, "Striped, spiky and very fond of worms.";
    Bluegill = "bluegill", "Bluegill", "bluegill", 99, 32, Kind::Fish, "Blue in the face, gold in the belly.";
    MudCarp = "mud_carp", "Mud Carp", "mud_carp", 99, 60, Kind::Fish, "Wise, whiskery and in no hurry at all.";
    LilyKoi = "lily_koi", "Lily Koi", "lily_koi", 99, 190, Kind::Fish, "It naps under lily pads and dreams in colour.";
    BubbleGoby = "bubble_goby", "Bubble Goby", "bubble_goby", 99, 55, Kind::Fish, "Blows bubbles when it rains. Only when it rains.";
    MoonlitCatfish = "moonlit_catfish", "Moonlit Catfish", "moonlit_catfish", 99, 160, Kind::Fish, "Comes out after dark to look at the moon.";
    GoldenCarp = "golden_carp", "Golden Carp", "golden_carp", 99, 950, Kind::Fish, "The pond's oldest resident. Legend says it grants wishes.";
    BrookTrout = "brook_trout", "Brook Trout", "brook_trout", 99, 55, Kind::Fish, "Speckled like a pebbly stream.";
    RainbowTrout = "rainbow_trout", "Rainbow Trout", "rainbow_trout", 99, 110, Kind::Fish, "Every colour of a rain shower.";
    RiverChub = "river_chub", "River Chub", "river_chub", 99, 30, Kind::Fish, "Round, cheerful and not very bright.";
    PebbleLoach = "pebble_loach", "Pebble Loach", "pebble_loach", 99, 45, Kind::Fish, "It pretends to be a pebble. It is not a pebble.";
    SilverDace = "silver_dace", "Silver Dace", "silver_dace", 99, 40, Kind::Fish, "Quick as a wink and twice as shiny.";
    StripedBass = "striped_bass", "Striped Bass", "striped_bass", 99, 140, Kind::Fish, "Strong, stripy and stubborn.";
    StormSalmon = "storm_salmon", "Storm Salmon", "storm_salmon", 99, 330, Kind::Fish, "Leaps upstream when the rain comes down.";
    MoonfinEel = "moonfin_eel", "Moonfin Eel", "moonfin_eel", 99, 380, Kind::Fish, "Its fins glow like a crescent moon. Night only.";
    BramblePike = "bramble_pike", "Bramble Pike", "bramble_pike", 99, 210, Kind::Fish, "A toothy lurker in the reeds.";
    BlindCavefish = "blind_cavefish", "Blind Cavefish", "blind_cavefish", 99, 70, Kind::Fish, "Has never seen the sun, and doesn't miss it.";
    MossyPike = "mossy_pike", "Mossy Pike", "mossy_pike", 99, 150, Kind::Fish, "Moss grows on its back. It doesn't mind.";
    GlowwormEel = "glowworm_eel", "Glowworm Eel", "glowworm_eel", 99, 270, Kind::Fish, "Lights up the underground pools.";
    Crayfish = "crayfish", "Crayfish", "crayfish", 99, 60, Kind::Fish, "Pinchy! Lovely boiled with corn.";
    CrystalTetra = "crystal_tetra", "Crystal Tetra", "crystal_tetra", 99, 90, Kind::Fish, "You can see right through it. Rude to stare, though.";
    PrismGuppy = "prism_guppy", "Prism Guppy", "prism_guppy", 99, 170, Kind::Fish, "Splits the light into rainbows with its tail.";
    GeodePuffer = "geode_puffer", "Geode Puffer", "geode_puffer", 99, 230, Kind::Fish, "Puffs up into a spiky gem when startled.";
    DiamondRay = "diamond_ray", "Diamond Ray", "diamond_ray", 99, 500, Kind::Fish, "Glides like a kite made of ice.";
    SporeSnapper = "spore_snapper", "Spore Snapper", "spore_snapper", 99, 110, Kind::Fish, "Snaps at glowing spores. And fingers.";
    JellyAngelfish = "jelly_angelfish", "Jelly Angelfish", "jelly_angelfish", 99, 240, Kind::Fish, "Wobbly fins, graceful as a dancer.";
    TruffleCatfish = "truffle_catfish", "Truffle Catfish", "truffle_catfish", 99, 540, Kind::Fish, "Its whiskers can smell a truffle a mile away.";
    MoonJelly = "moon_jelly", "Moon Jelly", "moon_jelly", 99, 200, Kind::Fish, "Not a fish at all. Very pretty anyway.";
    LavaEel = "lava_eel", "Lava Eel", "lava_eel", 99, 320, Kind::Fish, "Swims through molten rock like it's bathwater.";
    MagmaGoby = "magma_goby", "Magma Goby", "magma_goby", 99, 270, Kind::Fish, "Tiny, fiery and absolutely furious.";
    CinderCarp = "cinder_carp", "Cinder Carp", "cinder_carp", 99, 440, Kind::Fish, "Covered in glowing embers. Handle with mittens.";
    PhoenixKoi = "phoenix_koi", "Phoenix Koi", "phoenix_koi", 99, 1900, Kind::Fish, "Reborn in the lava every hundred years.";
    SnowSmelt = "snow_smelt", "Snow Smelt", "snow_smelt", 99, 120, Kind::Fish, "Smells faintly of cucumber and snow.";
    IcePike = "ice_pike", "Ice Pike", "ice_pike", 99, 290, Kind::Fish, "Sharp as an icicle and twice as cold.";
    FrostChar = "frost_char", "Frost Char", "frost_char", 99, 270, Kind::Fish, "Pink as a winter sunset.";
    AuroraTrout = "aurora_trout", "Aurora Trout", "aurora_trout", 99, 720, Kind::Fish, "Shimmers with the northern lights.";
    GhostFish = "ghost_fish", "Ghost Fish", "ghost_fish", 99, 230, Kind::Fish, "Boo. (It's shy.)";
    BoneFish = "bone_fish", "Bone Fish", "bone_fish", 99, 310, Kind::Fish, "All bones and no manners.";
    Coelacanth = "coelacanth", "Ancient Coelacanth", "coelacanth", 99, 920, Kind::Fish, "It was old when the ruins were new.";
    StarSturgeon = "star_sturgeon", "Star Sturgeon", "star_sturgeon", 99, 2300, Kind::Fish, "Its scales map the stars of a forgotten sky.";
}

impl Serialize for Item {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(self.def().key)
    }
}

impl<'de> Deserialize<'de> for Item {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Item, D::Error> {
        let key = String::deserialize(d)?;
        Item::from_key(&key).ok_or_else(|| serde::de::Error::custom(format!("unknown item {key}")))
    }
}

impl Item {
    pub fn def(self) -> &'static ItemDef {
        &ITEMS[self as usize]
    }

    pub fn from_key(key: &str) -> Option<Item> {
        ALL_ITEMS.iter().copied().find(|i| i.def().key == key)
    }

    /// The gear base, for weapons, armour and tools.
    pub fn base(self) -> Option<Base> {
        match self.def().kind {
            Kind::Gear(b) => Some(b),
            _ => None,
        }
    }

    pub fn class(self) -> Option<Class> {
        self.base().map(|b| b.class)
    }

    pub fn scroll_group(self) -> Option<Group> {
        match self.def().kind {
            Kind::Scroll(g) => Some(g),
            _ => None,
        }
    }

    /// True for things that carry their own rolled data (gear and scrolls).
    pub fn is_unique(self) -> bool {
        matches!(self.def().kind, Kind::Gear(_) | Kind::Scroll(_))
    }

    pub fn coin_value(self) -> Option<u32> {
        match self.def().kind {
            Kind::Coin(v) => Some(v),
            _ => None,
        }
    }

    /// The scroll item for a group.
    pub fn scroll(group: Group) -> Item {
        match group {
            Group::Weapon => Item::WeaponScroll,
            Group::Armor => Item::ArmorScroll,
            Group::Tool => Item::ToolScroll,
        }
    }

    /// Seeds that grow a crop.
    pub fn seed_of(crop: Crop) -> Option<Item> {
        ALL_ITEMS
            .iter()
            .copied()
            .find(|i| matches!(i.def().kind, Kind::Seed(c) if c == crop))
    }
}

// ------------------------------------------------------------------------------------------
// Crops
// ------------------------------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum Crop {
    Turnip,
    CaveCarrot,
    Glowcap,
    CrystalBerry,
    MossMelon,
    SporePumpkin,
    EmberPepper,
    FrostLily,
    Moonbloom,
    Potato,
    Radish,
    Cabbage,
    Tomato,
    Strawberry,
    Wheat,
    Corn,
    Blueberry,
    Eggplant,
    Garlic,
    Sunflower,
    Rose,
    SweetPea,
    Mossberry,
    Bunnyroot,
    PrismPear,
    GeodeGourd,
    Puffball,
    JellyShroom,
    Truffle,
    FlameTulip,
    LavaLemon,
    MagmaMelon,
    SnowPea,
    IcePlum,
    FrostMint,
    Starfruit,
    GhostPepper,
    AncientGrain,
}

pub const ALL_CROPS: [Crop; 38] = [
    Crop::Turnip,
    Crop::CaveCarrot,
    Crop::Glowcap,
    Crop::CrystalBerry,
    Crop::MossMelon,
    Crop::SporePumpkin,
    Crop::EmberPepper,
    Crop::FrostLily,
    Crop::Moonbloom,
    Crop::Potato,
    Crop::Radish,
    Crop::Cabbage,
    Crop::Tomato,
    Crop::Strawberry,
    Crop::Wheat,
    Crop::Corn,
    Crop::Blueberry,
    Crop::Eggplant,
    Crop::Garlic,
    Crop::Sunflower,
    Crop::Rose,
    Crop::SweetPea,
    Crop::Mossberry,
    Crop::Bunnyroot,
    Crop::PrismPear,
    Crop::GeodeGourd,
    Crop::Puffball,
    Crop::JellyShroom,
    Crop::Truffle,
    Crop::FlameTulip,
    Crop::LavaLemon,
    Crop::MagmaMelon,
    Crop::SnowPea,
    Crop::IcePlum,
    Crop::FrostMint,
    Crop::Starfruit,
    Crop::GhostPepper,
    Crop::AncientGrain,
];

pub struct CropDef {
    pub days: u8,
    /// Days to fruit again after a harvest (0 = the plant is used up).
    pub regrow: u8,
    pub produce: Item,
    pub young: &'static str,
    /// Glows at night (a light source on the farm).
    pub glow: bool,
    pub yield_max: u8,
}

impl Crop {
    pub fn def(self) -> CropDef {
        use Crop::*;
        let d = |days, regrow, produce, young, glow, yield_max| CropDef {
            days,
            regrow,
            produce,
            young,
            glow,
            yield_max,
        };
        match self {
            Turnip => d(4, 0, Item::Turnip, "young", false, 1),
            CaveCarrot => d(3, 0, Item::CaveCarrot, "young", false, 2),
            Glowcap => d(5, 0, Item::Glowcap, "young_fungal", true, 2),
            CrystalBerry => d(6, 3, Item::CrystalBerry, "young_frost", false, 3),
            MossMelon => d(8, 0, Item::MossMelon, "young_vine", false, 1),
            SporePumpkin => d(9, 0, Item::SporePumpkin, "young_fungal", false, 1),
            EmberPepper => d(5, 2, Item::EmberPepper, "young_ember", false, 2),
            FrostLily => d(7, 0, Item::FrostLily, "young_frost", false, 1),
            Moonbloom => d(12, 0, Item::Moonbloom, "young_bud", true, 1),
            Potato => d(5, 0, Item::Potato, "young", false, 3),
            Radish => d(3, 0, Item::Radish, "young", false, 1),
            Cabbage => d(7, 0, Item::Cabbage, "young_leafy", false, 1),
            Tomato => d(6, 3, Item::Tomato, "young_vine", false, 2),
            Strawberry => d(7, 3, Item::Strawberry, "young_leafy", false, 2),
            Wheat => d(4, 0, Item::Wheat, "young_grain", false, 2),
            Corn => d(9, 4, Item::Corn, "young_grain", false, 2),
            Blueberry => d(8, 2, Item::Blueberry, "young_leafy", false, 4),
            Eggplant => d(6, 4, Item::Eggplant, "young_vine", false, 1),
            Garlic => d(4, 0, Item::Garlic, "young_grain", false, 1),
            Sunflower => d(8, 0, Item::Sunflower, "young_bud", false, 1),
            Rose => d(6, 0, Item::Rose, "young_bud", false, 1),
            SweetPea => d(5, 2, Item::SweetPea, "young_vine", false, 2),
            Mossberry => d(5, 2, Item::Mossberry, "young_leafy", false, 3),
            Bunnyroot => d(4, 0, Item::Bunnyroot, "young", false, 1),
            PrismPear => d(8, 4, Item::PrismPear, "young_crystal", false, 2),
            GeodeGourd => d(9, 0, Item::GeodeGourd, "young_crystal", false, 1),
            Puffball => d(4, 0, Item::Puffball, "young_fungal", false, 2),
            JellyShroom => d(6, 0, Item::JellyShroom, "young_fungal", true, 2),
            Truffle => d(10, 0, Item::Truffle, "young_fungal", false, 1),
            FlameTulip => d(7, 0, Item::FlameTulip, "young_ember", true, 1),
            LavaLemon => d(6, 3, Item::LavaLemon, "young_ember", false, 2),
            MagmaMelon => d(10, 0, Item::MagmaMelon, "young_ember", true, 1),
            SnowPea => d(5, 2, Item::SnowPea, "young_frost", false, 2),
            IcePlum => d(8, 3, Item::IcePlum, "young_frost", false, 2),
            FrostMint => d(4, 0, Item::FrostMint, "young_frost", false, 2),
            Starfruit => d(12, 0, Item::Starfruit, "young_ruin", true, 1),
            GhostPepper => d(7, 3, Item::GhostPepper, "young_ruin", false, 2),
            AncientGrain => d(6, 0, Item::AncientGrain, "young_grain", false, 2),
        }
    }

    /// Growth stage 0..=3 for a number of grown days (3 = ready).
    pub fn stage(self, days: u8) -> u8 {
        let total = self.def().days.max(1);
        if days >= total {
            3
        } else {
            ((days as u32 * 3) / total as u32).min(2) as u8
        }
    }
}

// ------------------------------------------------------------------------------------------
// Stacks and inventories
// ------------------------------------------------------------------------------------------

/// Some number of an item. Gear and scrolls always come one at a time, with their rolls.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Stack {
    pub item: Item,
    pub n: u16,
    pub gear: Option<Gear>,
}

impl Stack {
    /// A plain stack. Gear made this way is an unremarkable copy at its base level.
    pub fn new(item: Item, n: u16) -> Self {
        let gear = match item.def().kind {
            Kind::Gear(b) => {
                let mut g = Gear::plain(b.lvl);
                g.update_rarity();
                Some(g)
            }
            Kind::Scroll(group) => Some(gear::scroll_with(group.enchant_pool()[0], 1, 0.5)),
            _ => None,
        };
        Stack { item, n, gear }
    }

    pub fn with_gear(item: Item, gear: Gear) -> Self {
        Stack {
            item,
            n: 1,
            gear: Some(gear),
        }
    }

    pub fn rarity(&self) -> Option<Rarity> {
        self.gear.map(|g| g.rarity)
    }

    /// The name shown for this stack ("Scroll of Embers" for a scroll).
    pub fn name(&self) -> String {
        match (self.item.def().kind, self.gear) {
            (Kind::Scroll(_), Some(g)) => match g.scroll_enchant() {
                Some(a) => format!("Scroll of {}", a.stat.def().title),
                None => self.item.def().name.to_string(),
            },
            _ => self.item.def().name.to_string(),
        }
    }

    /// The main number of a piece of gear (damage, defense, power, water).
    pub fn main_value(&self) -> Option<i32> {
        let b = self.item.base()?;
        let g = self.gear?;
        Some(gear::base_value(b.class, g.level, g.quality, b.mult))
    }

    /// Adds this piece's stats (innate bonus, affixes, enchantments) to a sheet.
    pub fn add_stats(&self, sheet: &mut gear::Sheet) {
        let Some(g) = &self.gear else { return };
        if let Some(b) = self.item.base() {
            if let Some(s) = b.innate {
                sheet.add(s, g.innate(s) as i32);
            }
            g.add_to(sheet);
        }
    }

    /// What one of these sells for.
    pub fn unit_price(&self) -> u64 {
        let d = self.item.def();
        match (d.kind, self.gear) {
            (Kind::Gear(b), Some(g)) => {
                let lv = g.level.max(b.lvl) as f64;
                ((d.price as f64 * 0.5 + 6.0 + lv * lv * 0.5 + lv * 4.0) * g.rarity.value() as f64)
                    .round() as u64
            }
            (Kind::Scroll(_), Some(g)) => {
                let lv = g.level as f64;
                ((30.0 + lv * 8.0) * g.rarity.value() as f64).round() as u64
            }
            _ => d.price as u64,
        }
    }

    pub fn value(&self) -> u64 {
        self.unit_price() * self.n as u64
    }

    /// Can `other` be added onto this stack?
    pub fn stacks_with(&self, other: &Stack) -> bool {
        self.item == other.item && !self.item.is_unique()
    }

    /// Fills in rolls that are missing (older saves) and refreshes rarity.
    pub fn normalize(&mut self) {
        match self.item.def().kind {
            Kind::Gear(b) => {
                let g = self.gear.get_or_insert_with(|| Gear::plain(b.lvl));
                g.level = g.level.max(1);
                g.update_rarity();
                self.n = 1;
            }
            Kind::Scroll(_) => {
                if self.gear.and_then(|g| g.scroll_enchant()).is_none() {
                    self.gear = Stack::new(self.item, 1).gear;
                }
                if let Some(g) = &mut self.gear {
                    g.update_scroll_rarity();
                }
                self.n = 1;
            }
            _ => self.gear = None,
        }
    }
}

#[derive(Serialize, Deserialize)]
struct AffixRepr(String, i16);

#[derive(Serialize, Deserialize)]
struct GearRepr {
    lv: u16,
    q: u8,
    #[serde(default)]
    a: Vec<AffixRepr>,
    #[serde(default)]
    e: Vec<Option<AffixRepr>>,
}

#[derive(Serialize, Deserialize)]
struct StackRepr {
    item: String,
    n: u16,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    gear: Option<GearRepr>,
}

fn affix_repr(a: &Affix) -> AffixRepr {
    AffixRepr(a.stat.def().key.to_string(), a.val)
}

fn affix_from(r: &AffixRepr) -> Option<Affix> {
    Some(Affix {
        stat: Stat::from_key(&r.0)?,
        val: r.1,
    })
}

impl Serialize for Stack {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        StackRepr {
            item: self.item.def().key.to_string(),
            n: self.n,
            gear: self.gear.map(|g| GearRepr {
                lv: g.level,
                q: g.quality,
                a: g.affixes().map(affix_repr).collect(),
                e: g.enchants
                    .iter()
                    .map(|e| e.as_ref().map(affix_repr))
                    .collect(),
            }),
        }
        .serialize(s)
    }
}

impl<'de> Deserialize<'de> for Stack {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let r = StackRepr::deserialize(d)?;
        let item = Item::from_key(&r.item)
            .ok_or_else(|| serde::de::Error::custom(format!("unknown item {}", r.item)))?;
        let gear = r.gear.map(|gr| {
            let mut g = Gear::plain(gr.lv);
            g.quality = gr.q.min(100);
            for (slot, a) in g.affixes.iter_mut().zip(gr.a.iter().filter_map(affix_from)) {
                *slot = Some(a);
            }
            for (slot, e) in g.enchants.iter_mut().zip(gr.e.iter()) {
                *slot = e.as_ref().and_then(affix_from);
            }
            g
        });
        let mut s = Stack { item, n: r.n, gear };
        s.normalize();
        Ok(s)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Inventory {
    pub slots: Vec<Option<Stack>>,
}

impl Inventory {
    pub fn new(n: usize) -> Self {
        Inventory {
            slots: vec![None; n],
        }
    }

    /// Adds as much as fits; returns what is left over.
    pub fn add(&mut self, item: Item, n: u16) -> u16 {
        if item.is_unique() {
            let mut left = n;
            while left > 0 {
                if self.add_stack(Stack::new(item, 1)) > 0 {
                    break;
                }
                left -= 1;
            }
            return left;
        }
        self.add_stack(Stack::new(item, n))
    }

    /// Adds a stack (with its rolls); returns how many did not fit.
    pub fn add_stack(&mut self, stack: Stack) -> u16 {
        let max = stack.item.def().stack;
        let mut n = stack.n;
        if !stack.item.is_unique() {
            for s in self.slots.iter_mut().flatten() {
                if s.stacks_with(&stack) && s.n < max {
                    let k = (max - s.n).min(n);
                    s.n += k;
                    n -= k;
                    if n == 0 {
                        return 0;
                    }
                }
            }
        }
        for s in self.slots.iter_mut() {
            if s.is_none() {
                let k = if stack.item.is_unique() {
                    1
                } else {
                    max.min(n)
                };
                *s = Some(Stack { n: k, ..stack });
                n -= k;
                if n == 0 {
                    return 0;
                }
            }
        }
        n
    }

    pub fn can_fit(&self, item: Item, n: u16) -> bool {
        self.can_fit_stack(&Stack::new(item, n))
    }

    pub fn can_fit_stack(&self, stack: &Stack) -> bool {
        let max = stack.item.def().stack;
        let mut room = 0u32;
        for s in &self.slots {
            match s {
                None => {
                    room += if stack.item.is_unique() {
                        1
                    } else {
                        max as u32
                    }
                }
                Some(s) if s.stacks_with(stack) => room += (max - s.n.min(max)) as u32,
                _ => {}
            }
        }
        room >= stack.n as u32
    }

    /// Every different thing in the bag, sorted.
    pub fn distinct(&self) -> Vec<Item> {
        let mut v: Vec<Item> = self.slots.iter().flatten().map(|s| s.item).collect();
        v.sort();
        v.dedup();
        v
    }

    pub fn count(&self, item: Item) -> u32 {
        self.slots
            .iter()
            .flatten()
            .filter(|s| s.item == item)
            .map(|s| s.n as u32)
            .sum()
    }

    /// Removes `n` of an item if there are enough.
    pub fn take(&mut self, item: Item, n: u32) -> bool {
        if self.count(item) < n {
            return false;
        }
        let mut left = n;
        for slot in self.slots.iter_mut().rev() {
            if let Some(s) = slot {
                if s.item == item {
                    let k = (s.n as u32).min(left);
                    s.n -= k as u16;
                    left -= k;
                    if s.n == 0 {
                        *slot = None;
                    }
                    if left == 0 {
                        break;
                    }
                }
            }
        }
        true
    }

    /// Removes one from a slot.
    pub fn take_one(&mut self, slot: usize) -> Option<Item> {
        let s = self.slots.get_mut(slot)?;
        let st = s.as_mut()?;
        let item = st.item;
        st.n -= 1;
        if st.n == 0 {
            *s = None;
        }
        Some(item)
    }

    /// Stacks of a class of gear.
    pub fn gear_of(&self, class: Class) -> impl Iterator<Item = &Stack> {
        self.slots
            .iter()
            .flatten()
            .filter(move |s| s.item.class() == Some(class))
    }

    /// Fills in missing rolls after loading.
    pub fn normalize(&mut self) {
        for s in self.slots.iter_mut().flatten() {
            s.normalize();
        }
    }
}

// ------------------------------------------------------------------------------------------
// Recipes
// ------------------------------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Cat {
    Tools,
    Gear,
    Home,
    Kitchen,
    Magic,
    Potions,
}

pub const CATS: [Cat; 6] = [
    Cat::Tools,
    Cat::Gear,
    Cat::Home,
    Cat::Kitchen,
    Cat::Magic,
    Cat::Potions,
];

impl Cat {
    pub fn name(self) -> &'static str {
        ["Tools", "Gear", "Home", "Kitchen", "Magic", "Potions"][self as usize]
    }
}

pub struct Recipe {
    pub out: Item,
    pub n: u16,
    pub needs: &'static [(Item, u16)],
    pub cat: Cat,
}

use Item as I;

macro_rules! recipes {
    ($( $cat:ident: $out:ident x $n:literal <= [$( $need:ident $k:literal ),*]; )*) => {
        pub static RECIPES: &[Recipe] = &[
            $( Recipe { out: I::$out, n: $n, needs: &[$( (I::$need, $k) ),*], cat: Cat::$cat } ),*
        ];
    };
}

recipes! {
    // Home.
    Home: Torch x 4 <= [Wood 2, SlimeGel 1];
    Home: Chest x 1 <= [Wood 20];
    Home: Fence x 4 <= [Wood 4];
    Home: WoodPath x 4 <= [Wood 2];
    Home: StonePath x 4 <= [Stone 2];
    Home: StoneWall x 2 <= [Stone 4];
    Home: WoodWall x 2 <= [Wood 4];
    Home: Lamp x 1 <= [Stone 6, Glowcap 1];
    Home: FlowerPot x 1 <= [Stone 4, Fiber 2];
    Home: Bench x 1 <= [Wood 10];
    Home: Workbench x 1 <= [Wood 15, Stone 5];
    Home: EnchantTable x 1 <= [Wood 20, Crystal 4, Amethyst 1];
    Home: Sprinkler x 1 <= [CopperOre 5, IronOre 2];
    Home: QualitySprinkler x 1 <= [IronOre 5, GoldOre 2, Crystal 1];
    Home: CrystalSprinkler x 1 <= [GoldOre 5, Crystal 4, FrostGem 1];
    Home: WoodenChair x 1 <= [Wood 8];
    Home: RoundTable x 1 <= [Wood 12];
    Home: KitchenCounter x 1 <= [Wood 10, Stone 4];
    Home: Bookshelf x 1 <= [Wood 25, Fiber 6];
    Home: PottedFern x 1 <= [Stone 3, Fiber 6];
    Home: FishBowl x 1 <= [Crystal 2, Stone 4];

    // Kitchen.
    Kitchen: HealingTonic x 1 <= [CaveCarrot 2, SlimeGel 2];
    Kitchen: StaminaTonic x 1 <= [Turnip 2, Spore 1];
    Kitchen: ManaTonic x 1 <= [Blueberry 3, WispDust 1];
    Kitchen: VeggieStew x 1 <= [Turnip 1, CaveCarrot 1];
    Kitchen: BakedPotato x 1 <= [Potato 2];
    Kitchen: FreshBread x 1 <= [Wheat 3];
    Kitchen: TomatoSoup x 1 <= [Tomato 2, Garlic 1];
    Kitchen: GardenSalad x 1 <= [Cabbage 1, Radish 1, Tomato 1];
    Kitchen: GarlicBread x 1 <= [FreshBread 1, Garlic 1];
    Kitchen: Popcorn x 2 <= [Corn 1];
    Kitchen: CornChowder x 1 <= [Corn 2, Potato 1];
    Kitchen: GlowSoup x 1 <= [Glowcap 2, Turnip 1];
    Kitchen: MushroomSkewer x 1 <= [Puffball 2, Glowcap 1];
    Kitchen: BlueberryMuffin x 1 <= [Blueberry 3, Wheat 1];
    Kitchen: BerryTart x 1 <= [CrystalBerry 3, SlimeGel 1];
    Kitchen: CarrotCake x 1 <= [CaveCarrot 2, Wheat 2, Bunnyroot 1];
    Kitchen: Shortcake x 1 <= [Strawberry 3, Wheat 2];
    Kitchen: SunflowerCookies x 2 <= [Sunflower 1, Wheat 2];
    Kitchen: PearCrumble x 1 <= [PrismPear 2, Wheat 1];
    Kitchen: EmberCurry x 1 <= [EmberPepper 1, CaveCarrot 1, Turnip 1];
    Kitchen: LavaLemonade x 1 <= [LavaLemon 2, Mossberry 2];
    Kitchen: MintTea x 1 <= [FrostMint 2, Rose 1];
    Kitchen: PlumPudding x 1 <= [IcePlum 2, SnowPea 1];
    Kitchen: SpookyChili x 1 <= [GhostPepper 1, Eggplant 1, Tomato 1];
    Kitchen: PumpkinPie x 1 <= [SporePumpkin 1, CrystalBerry 2];
    Kitchen: TruffleRisotto x 1 <= [Truffle 1, AncientGrain 2, Garlic 1];
    Kitchen: StarfruitTart x 1 <= [Starfruit 1, Wheat 2, JellyShroom 1];
    Kitchen: FishAndChips x 1 <= [PondPerch 1, Potato 2];
    Kitchen: GrilledTrout x 1 <= [BrookTrout 1, Garlic 1];
    Kitchen: MinnowFritters x 2 <= [SunnyMinnow 3, Wheat 1];
    Kitchen: Sushi x 2 <= [SilverDace 1, Seaweed 2, Wheat 1];
    Kitchen: FishStew x 1 <= [Bluegill 1, Tomato 1, Cabbage 1];
    Kitchen: CarpCurry x 1 <= [MudCarp 1, EmberPepper 1];
    Kitchen: SalmonSteak x 1 <= [StormSalmon 1, Garlic 1];
    Kitchen: CrayfishBoil x 1 <= [Crayfish 3, Corn 1];
    Kitchen: CaveSkewer x 1 <= [BlindCavefish 2, Glowcap 1];
    Kitchen: Ceviche x 1 <= [CrystalTetra 2, LavaLemon 1];
    Kitchen: SmeltPie x 1 <= [SnowSmelt 2, Wheat 2];
    Kitchen: EelKebab x 1 <= [LavaEel 1, EmberPepper 1];
    Kitchen: FishTacos x 1 <= [RainbowTrout 1, Corn 1, Tomato 1];
    Kitchen: EmperorPlatter x 1 <= [GoldenCarp 1, Truffle 1];

    // Tools.
    Tools: Sickle x 1 <= [Wood 3, Stone 4];
    Tools: CopperHoe x 1 <= [CopperOre 8, Wood 4];
    Tools: Can1 x 1 <= [CopperOre 10];
    Tools: CopperSickle x 1 <= [CopperOre 8, Wood 3];
    Tools: Axe1 x 1 <= [CopperOre 8, Wood 5];
    Tools: Pick1 x 1 <= [CopperOre 8, Wood 5];
    Tools: IronHoe x 1 <= [IronOre 10, CopperOre 4];
    Tools: IronCan x 1 <= [IronOre 12, CopperOre 4];
    Tools: IronSickle x 1 <= [IronOre 10, CopperOre 4];
    Tools: Axe2 x 1 <= [IronOre 10, CopperOre 4];
    Tools: Pick2 x 1 <= [IronOre 10, CopperOre 4];
    Tools: GoldHoe x 1 <= [GoldOre 12, IronOre 5];
    Tools: GoldSickle x 1 <= [GoldOre 12, IronOre 5];
    Tools: Axe3 x 1 <= [GoldOre 12, IronOre 5];
    Tools: Pick3 x 1 <= [GoldOre 12, IronOre 5];
    Tools: CrystalHoe x 1 <= [Crystal 12, GoldOre 5];
    Tools: Can2 x 1 <= [Crystal 6, GoldOre 4];
    Tools: Axe4 x 1 <= [Crystal 12, GoldOre 5];
    Tools: Pick4 x 1 <= [Crystal 12, GoldOre 5];
    Tools: EmberHoe x 1 <= [EmberOre 14, Crystal 6];
    Tools: Axe5 x 1 <= [EmberOre 14, Crystal 6];
    Tools: Pick5 x 1 <= [EmberOre 14, Crystal 6];
    Tools: Bait x 5 <= [Fiber 2, SlimeGel 1];

    // Weapons and armour.
    Gear: TwigWand x 1 <= [Wood 3, Spore 1];
    Gear: OakStaff x 1 <= [Wood 8, Spore 2];
    Gear: WoodBuckler x 1 <= [Wood 12];
    Gear: StrawHat x 1 <= [Fiber 12];
    Gear: FlowerCrown x 1 <= [Fiber 4, Rose 2];
    Gear: GrassSkirt x 1 <= [Fiber 15];
    Gear: LeafTunic x 1 <= [Fiber 20, SlimeGel 2];
    Gear: LeatherBoots x 1 <= [BatWing 4, Fiber 6];
    Gear: Sword1 x 1 <= [CopperOre 10, Wood 5];
    Gear: CopperShield x 1 <= [CopperOre 10, Wood 4];
    Gear: CopperHelm x 1 <= [CopperOre 8];
    Gear: CopperMail x 1 <= [CopperOre 14, Fiber 4];
    Gear: CopperGreaves x 1 <= [CopperOre 10, Fiber 3];
    Gear: CopperSabatons x 1 <= [CopperOre 8, Fiber 2];
    Gear: GlowcapWand x 1 <= [Wood 4, Glowcap 2, Spore 3];
    Gear: TurtleShell x 1 <= [CrabShell 6, Fiber 4];
    Gear: MushroomCap x 1 <= [ShroomCap 5, Spore 2];
    Gear: Sword2 x 1 <= [IronOre 12, CopperOre 5];
    Gear: IronShield x 1 <= [IronOre 12, Wood 4];
    Gear: IronHelm x 1 <= [IronOre 10];
    Gear: IronPlate x 1 <= [IronOre 16, Fiber 4];
    Gear: IronGreaves x 1 <= [IronOre 12, Fiber 3];
    Gear: IronBoots x 1 <= [IronOre 10, Fiber 2];
    Gear: WizardHat x 1 <= [Fiber 10, WispDust 4, Amethyst 1];
    Gear: MageRobe x 1 <= [Fiber 16, WispDust 5];
    Gear: CrystalWand x 1 <= [Crystal 8, Wood 4, WispDust 2];
    Gear: MushroomStaff x 1 <= [ShroomCap 6, Wood 6, Spore 4];
    Gear: Sword3 x 1 <= [GoldOre 14, IronOre 6];
    Gear: GoldShield x 1 <= [GoldOre 14, IronOre 4];
    Gear: CrystalStaff x 1 <= [Crystal 12, GoldOre 4];
    Gear: Sword4 x 1 <= [Crystal 14, GoldOre 6];
    Gear: CrystalAegis x 1 <= [Crystal 14, GoldOre 4];
    Gear: CrystalCirclet x 1 <= [Crystal 10, GoldOre 3];
    Gear: CrystalMail x 1 <= [Crystal 16, GoldOre 5];
    Gear: CrystalGreaves x 1 <= [Crystal 12, GoldOre 4];
    Gear: CrystalBoots x 1 <= [Crystal 10, GoldOre 3];
    Gear: EmberWand x 1 <= [EmberOre 8, ImpHorn 3, Wood 4];
    Gear: EmberStaff x 1 <= [EmberOre 12, ImpHorn 4, Wood 6];
    Gear: Sword5 x 1 <= [EmberOre 16, Crystal 8];
    Gear: EmberCrown x 1 <= [EmberOre 10, GoldOre 6, Ruby 1];
    Gear: EmberPlate x 1 <= [EmberOre 18, Crystal 6];
    Gear: EmberGreaves x 1 <= [EmberOre 14, Crystal 4];
    Gear: EmberTreads x 1 <= [EmberOre 12, Crystal 4];
    Gear: FrostWand x 1 <= [FrostGem 6, Crystal 6, Ectoplasm 2];
    Gear: FrostCoat x 1 <= [FrostGem 8, Fiber 20, BatWing 6];

    // Magic.
    Magic: WeaponScroll x 1 <= [Fiber 5, Ruby 1, Spore 2];
    Magic: ArmorScroll x 1 <= [Fiber 5, Sapphire 1, Spore 2];
    Magic: ToolScroll x 1 <= [Fiber 5, Emerald 1, Spore 2];
    Magic: Feather x 1 <= [BatWing 4, WispDust 2];
    Magic: WishStar x 1 <= [Moonstone 2, WispDust 6, Starfruit 1];
    Magic: SunStone x 1 <= [Topaz 3, Amber 3, Sunflower 2];
    Magic: HeartCrystal x 1 <= [Amber 3, FrostGem 2, Moonbloom 1];

    // Potions: small ones from scratch, then two of a size brew into one of the next.
    Potions: Vial x 3 <= [Stone 2, SlimeGel 1];
    Potions: SmallHealthPotion x 2 <= [Vial 2, Heartleaf 1];
    Potions: HealthPotion x 1 <= [SmallHealthPotion 2, SlimeGel 2];
    Potions: LargeHealthPotion x 1 <= [HealthPotion 2, Amber 1];
    Potions: SmallManaPotion x 2 <= [Vial 2, Blueberry 2];
    Potions: ManaPotion x 1 <= [SmallManaPotion 2, WispDust 1];
    Potions: LargeManaPotion x 1 <= [ManaPotion 2, Crystal 2];
    Potions: SmallEnergyPotion x 2 <= [Vial 2, CaveCarrot 1];
    Potions: EnergyPotion x 1 <= [SmallEnergyPotion 2, Spore 2];
    Potions: LargeEnergyPotion x 1 <= [EnergyPotion 2, Sunflower 1];
}

impl Recipe {
    pub fn can_craft(&self, inv: &Inventory) -> bool {
        self.needs.iter().all(|(i, n)| inv.count(*i) >= *n as u32) && inv.can_fit(self.out, self.n)
    }

    /// Known from the start: made from nothing but wood, stone and fibre.
    pub fn common(&self) -> bool {
        self.needs.iter().all(|(i, _)| COMMON.contains(i))
    }

    /// With one of each of its ingredients in hand (however many it really takes), you
    /// can work out how it's made. `bag` is sorted.
    pub fn hinted_by(&self, bag: &[Item]) -> bool {
        self.needs.iter().all(|(i, _)| bag.binary_search(i).is_ok())
    }
}

/// What everyone knows how to work: recipes made only of these are known from the start;
/// the rest are learned by finding their ingredients.
pub const COMMON: [Item; 3] = [Item::Wood, Item::Stone, Item::Fiber];

/// The recipes a new farmer knows: the common ones, and any the starting kit gives away.
pub fn starter_recipes(inv: &Inventory) -> std::collections::BTreeSet<Item> {
    let bag = inv.distinct();
    RECIPES
        .iter()
        .filter(|r| r.common() || r.hinted_by(&bag))
        .map(|r| r.out)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keys_are_unique_and_round_trip() {
        for (i, it) in ALL_ITEMS.iter().enumerate() {
            assert_eq!(*it as usize, i);
            assert_eq!(Item::from_key(it.def().key), Some(*it));
        }
    }

    #[test]
    fn every_crop_has_seeds_and_produce() {
        for c in ALL_CROPS {
            assert!(Item::seed_of(c).is_some(), "{c:?} has no seeds");
            let p = c.def().produce;
            assert!(p.def().price > 0);
        }
        assert!(ALL_CROPS.len() >= 36);
    }

    #[test]
    fn plenty_of_gear() {
        let mut per_class = std::collections::HashMap::new();
        for it in ALL_ITEMS {
            if let Some(c) = it.class() {
                *per_class.entry(c).or_insert(0) += 1;
                assert_eq!(it.def().stack, 1, "{:?} must not stack", it);
            }
        }
        for c in gear::CLASSES {
            assert!(per_class.get(&c).copied().unwrap_or(0) >= 5, "{c:?}");
        }
        assert!(per_class.values().sum::<i32>() >= 110);
    }

    #[test]
    fn inventory_stacks_and_takes() {
        let mut inv = Inventory::new(3);
        assert_eq!(inv.add(Item::Wood, 150), 0);
        assert_eq!(inv.count(Item::Wood), 150);
        assert_eq!(inv.add(Item::Stone, 99), 0);
        assert_eq!(inv.add(Item::Stone, 1), 1);
        assert!(inv.take(Item::Wood, 120));
        assert_eq!(inv.count(Item::Wood), 30);
        assert!(!inv.take(Item::Wood, 31));
    }

    #[test]
    fn gear_never_stacks() {
        let mut inv = Inventory::new(4);
        assert_eq!(inv.add(Item::Sword1, 2), 0);
        assert_eq!(inv.slots.iter().flatten().count(), 2);
        assert!(
            inv.slots
                .iter()
                .flatten()
                .all(|s| s.n == 1 && s.gear.is_some())
        );
        assert!(inv.can_fit(Item::Sword2, 1));
        assert_eq!(inv.add(Item::Sword2, 3), 1);
    }

    #[test]
    fn crop_stages() {
        assert_eq!(Crop::Turnip.stage(0), 0);
        assert_eq!(Crop::Turnip.stage(4), 3);
        assert!(Crop::Moonbloom.stage(11) < 3);
    }

    #[test]
    fn stack_serde() {
        let s = Stack::new(Item::Glowcap, 7);
        let j = serde_json::to_string(&s).unwrap();
        assert!(!j.contains("gear"), "plain stacks stay small: {j}");
        let back: Stack = serde_json::from_str(&j).unwrap();
        assert_eq!(back, s);
    }

    #[test]
    fn gear_serde_keeps_rolls() {
        let mut rng = crate::util::Rng::new(5);
        let mut g = Gear::roll(Class::Sword, 23, 0.5, &mut rng);
        g.enchant(
            1,
            Affix {
                stat: Stat::Burn,
                val: 9,
            },
        );
        let s = Stack::with_gear(Item::Sword2, g);
        let j = serde_json::to_string(&s).unwrap();
        let back: Stack = serde_json::from_str(&j).unwrap();
        assert_eq!(back, s);
        // Old saves: gear without rolls gets a plain copy.
        let old: Stack = serde_json::from_str(r#"{"item":"sword3","n":1}"#).unwrap();
        assert_eq!(old.gear.unwrap().level, 26);
        let scroll: Stack = serde_json::from_str(r#"{"item":"tool_scroll","n":1}"#).unwrap();
        assert!(scroll.gear.unwrap().scroll_enchant().is_some());
    }

    #[test]
    fn recipes_make_real_things() {
        for r in RECIPES {
            assert!(r.n > 0 && !r.needs.is_empty());
            assert!(r.needs.iter().all(|(i, _)| *i != r.out));
        }
        assert!(RECIPES.len() > 80);
    }
}
