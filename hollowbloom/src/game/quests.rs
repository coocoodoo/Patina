//! Quests: requests from the people of Bramblewick, what they ask for, how progress is
//! counted, what they give back, and the notices on the town and guild boards.

use glam::Vec3;
use serde::{Deserialize, Serialize};

use super::Io;
use super::dungeon::{Foe, biome_for};
use super::folk::{VILLAGERS, Villager};
use super::fx::Drop;
use super::gear::{Group, Rarity};
use super::items::{Crop, Item, Kind, Stack, seasonal_seeds};
use super::loot;
use super::play::{Play, Season};
use super::town::{self, FOUNTAIN};
use crate::audio::Sfx;
use crate::palette::*;
use crate::util::Rng;

/// Where quest things drop from.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum Source {
    /// One kind of creature.
    Foe(Foe),
    /// Anything living in a biome.
    Biome(u8),
    /// Anything at or below a floor.
    Deep(u32),
    /// A guardian (it returns to its floor while the quest is open).
    Boss(Foe),
    /// Rising out of the floor where the guardian of this floor falls (it returns to its
    /// floor while the quest is open).
    Eruption(u32),
}

#[derive(Clone, Copy, PartialEq, Debug, Serialize, Deserialize)]
pub enum Goal {
    /// Hand over some things.
    Bring(Item, u16),
    /// Find a keepsake lying somewhere on floors lo..=hi.
    Find(Item, u32, u32),
    /// Collect things that only drop while the quest is open.
    Gather(Item, u16, Source),
    /// Defeat creatures (any kind if `None`) on or below a floor.
    Slay(Option<Foe>, u16, u32),
    /// Reach a floor.
    Reach(u32),
    /// Defeat the guardian of a floor.
    Guardian(u32),
    /// Harvest crops (any kind if `None`).
    Harvest(Option<Crop>, u16),
    /// Earn this much copper from the shipping bin.
    Ship(u64),
    /// Bind scrolls at an enchanting table.
    Enchant(u16),
    /// Carry something to someone.
    Deliver(Item, Villager),
    /// Say hello to everyone in the set (bits by villager).
    Meet(u32),
    /// Harvest this many different crops.
    Almanac(u16),
    /// Find this many different gems and relics.
    Curios(u16),
    /// Cook this many different dishes.
    Cookbook(u16),
    /// Defeat this many different kinds of creature.
    Codex(u16),
    /// Cast this many spells.
    Cast(u16),
    /// Brew this many potions.
    Brew(u16),
    /// Train any one spell up to this level.
    Mastery(u8),
    /// Catch this many fish.
    Fish(u16),
    /// Catch this many different kinds of fish.
    Fishdex(u16),
    /// Make your home this charming.
    Charm(u32),
    /// Have this many pieces of furniture in your house.
    Furnish(u16),
    /// Keep this many fish in your tanks.
    Tank(u16),
    /// Cook this many dishes at a stove.
    Cook(u16),
}

#[derive(Clone, Copy, PartialEq, Debug, Serialize, Deserialize)]
pub enum Reward {
    Coins(u64),
    Item(Item, u16),
    /// A piece of gear at your level, at least this rare.
    Gear(Item, Rarity),
    /// A scroll at your level, at least this rare.
    Scroll(Group, Rarity),
    Friend(Villager, i32),
    /// Finishes a town project.
    Restore(u32),
    /// Lantern Guild marks.
    Marks(u32),
    /// Hazel teaches you a spell (or, if you know it, a lesson's worth of practice).
    Spell(super::spells::Spell),
}

pub struct QuestDef {
    pub key: &'static str,
    pub giver: Villager,
    pub title: &'static str,
    /// What they say when asking.
    pub ask: &'static str,
    /// What they say when it's done.
    pub thanks: &'static str,
    pub goal: Goal,
    pub reward: &'static [Reward],
    /// A quest that must be finished first ("" for none).
    pub after: &'static str,
    /// Deepest floor reached, hearts with the giver and day it opens on.
    pub depth: u32,
    pub hearts: u8,
    pub day: u32,
    /// How charming your home must be before they'll ask.
    pub charm: u32,
    /// Only asked in this season.
    pub season: Option<Season>,
}

const Q: QuestDef = QuestDef {
    key: "",
    giver: Villager::Thistle,
    title: "",
    ask: "",
    thanks: "",
    goal: Goal::Reach(1),
    reward: &[],
    after: "",
    depth: 0,
    hearts: 0,
    day: 0,
    charm: 0,
    season: None,
};

/// Bits for a set of villagers.
pub const fn meet(vs: &[Villager]) -> u32 {
    let mut m = 0;
    let mut i = 0;
    while i < vs.len() {
        m |= 1 << (vs[i] as u32);
        i += 1;
    }
    m
}

use Goal::*;
use Reward::{Coins, Friend, Marks, Restore, Scroll};
use Villager as V;

const fn gear(item: Item, r: Rarity) -> Reward {
    Reward::Gear(item, r)
}

const fn item(i: Item, n: u16) -> Reward {
    Reward::Item(i, n)
}

const UNCOMMON: Rarity = Rarity::Uncommon;
const RARE: Rarity = Rarity::Rare;
const EPIC: Rarity = Rarity::Epic;
const LEGEND: Rarity = Rarity::Legendary;

pub static QUESTS: &[QuestDef] = &[
    // ---------------------------------------------------------------- Mayor Thistle
    // Seasonal: only asked in its season.
    QuestDef {
        key: "thistle_harvest",
        giver: V::Thistle,
        title: "The Harvest Festival",
        ask: "Autumn means the Harvest Festival! Well, it used to. Three big pumpkins for the plaza \
              and we'll bring it back. Carved, uncarved - I'm not fussy!",
        thanks: "Jack-o'-lanterns all over the plaza, and everyone's out! Soup from the festival pot, \
                 and candy corn to plant for next year.",
        goal: Bring(Item::Pumpkin, 3),
        reward: &[
            Coins(3000),
            item(Item::PumpkinSoup, 3),
            item(Item::CandyCornKernels, 10),
        ],
        day: 2,
        season: Some(Season::Autumn),
        ..Q
    },
    QuestDef {
        key: "thistle_hello",
        giver: V::Thistle,
        title: "Welcome to Bramblewick",
        ask: "Why don't you go and say hello to our shopkeepers? Hilde at the armoury, \
              Garrick at the smithy, Posy with her seeds, Mabel at the bakery and Nix \
              with the tools. Friendly faces make a town feel like home.",
        thanks: "Everyone's talking about the new farmer! Here - a little welcome gift \
                 from the town. We're so glad you came.",
        goal: Meet(meet(&[V::Hilde, V::Garrick, V::Posy, V::Mabel, V::Nix])),
        reward: &[
            Coins(500),
            item(Item::FreshBread, 3),
            Friend(V::Thistle, 60),
        ],
        ..Q
    },
    QuestDef {
        key: "thistle_fountain",
        giver: V::Thistle,
        title: "The Dry Fountain",
        ask: "Our fountain was fed by a Spring Stone that wells up water forever. Years \
              ago a greedy slime carried it down into the Hollow! Legend says it's \
              somewhere between floors 5 and 9. Would you look for it?",
        thanks: "The Spring Stone! Listen - can you hear it? The fountain's singing again! \
                 Oh, Grandma Fern is going to cry. I might cry. Here, you've earned this.",
        goal: Find(Item::SpringStone, 5, 9),
        reward: &[Coins(1500), Restore(FOUNTAIN), item(Item::HeartCrystal, 1)],
        after: "thistle_hello",
        ..Q
    },
    QuestDef {
        key: "thistle_lamps",
        giver: V::Thistle,
        title: "Light the Lamps",
        ask: "The street lamps burned on glow oil, and nobody's been able to make it since \
              old Tallow retired. The wisps in the Crystal Grotto are made of the stuff - \
              catch me twelve jars' worth and Bramblewick will glow again.",
        thanks: "Look at them shine! The whole town's out on the streets tonight. You've \
                 given us our evenings back, farmer.",
        goal: Gather(Item::GlowOil, 12, Source::Foe(Foe::Wisp)),
        reward: &[
            Coins(3000),
            Restore(town::LAMPS),
            item(Item::Lamp, 4),
            Friend(V::Mira, 40),
        ],
        after: "thistle_fountain",
        depth: 11,
        ..Q
    },
    QuestDef {
        key: "thistle_bunting",
        giver: V::Thistle,
        title: "Bunting for the Festival",
        ask: "A town needs a little celebration! Wren says she can sew festival bunting if \
              someone brings her enough fibre. Sixty bundles should do it.",
        thanks: "Flags from lamp to lamp! It's like the old harvest days. Pip hasn't stopped \
                 dancing since breakfast.",
        goal: Bring(Item::Fiber, 60),
        reward: &[
            Coins(2000),
            Restore(town::BUNTING),
            item(Item::Popcorn, 6),
            Friend(V::Wren, 40),
        ],
        after: "thistle_lamps",
        ..Q
    },
    QuestDef {
        key: "thistle_bridge",
        giver: V::Thistle,
        title: "Mend the Old Bridges",
        ask: "The bridges to the park washed away in a storm, and the Wishing Tree has \
              been lonely ever since. With a hundred and fifty lengths of wood, the whole \
              town could rebuild them together.",
        thanks: "The bridges are back! Olive's already over there pruning, and Mira says the \
                 park is the best place for stars. Thank you, truly.",
        goal: Bring(Item::Wood, 150),
        reward: &[
            Coins(5000),
            Restore(town::BRIDGE),
            item(Item::SunStone, 1),
            Friend(V::Olive, 60),
        ],
        after: "thistle_bunting",
        depth: 15,
        ..Q
    },
    QuestDef {
        key: "thistle_clock",
        giver: V::Thistle,
        title: "The Silent Clock",
        ask: "The Town Hall clock hasn't chimed in twenty years. Nix found the problem: the \
              bell's clapper is missing! Someone swears they saw it deep in the Fungal \
              Hollow, floors 25 to 29. Mushrooms, of all things.",
        thanks: "BONG! BONG! Ha! I'd forgotten how it sounds. Everyone stopped in the street \
                 to listen. Bramblewick has its heartbeat back.",
        goal: Find(Item::BellClapper, 25, 29),
        reward: &[
            Coins(8000),
            Restore(town::CLOCK),
            Scroll(Group::Tool, EPIC),
            Friend(V::Nix, 60),
        ],
        after: "thistle_bridge",
        depth: 25,
        ..Q
    },
    QuestDef {
        key: "thistle_market",
        giver: V::Thistle,
        title: "Market Day",
        ask: "Traders used to come from all over for our market. They'll come again if we \
              can show them Bramblewick is thriving. Thirty pieces of gold ore for the \
              market stalls' fittings would do it.",
        thanks: "Stalls in the plaza again! Fruit, flowers, trinkets - listen to all that \
                 chatter. This is what a town should sound like.",
        goal: Bring(Item::GoldOre, 30),
        reward: &[Coins(10000), Restore(town::MARKET), item(Item::WishStar, 1)],
        after: "thistle_clock",
        depth: 30,
        ..Q
    },
    QuestDef {
        key: "thistle_tree",
        giver: V::Thistle,
        title: "The Wishing Tree",
        ask: "One last thing, and it's a big one. The Wishing Tree in the park stopped \
              blooming when the town lost its way. Its wishing leaves blew down into the \
              deep Hollow - floor 40 and below. Bring back seven, and let's see it bloom.",
        thanks: "Oh... oh, look at it. Pink blossoms, glowing in the dark. You did this. You \
                 brought the whole town back to life, farmer. Bramblewick will never forget.",
        goal: Gather(Item::WishLeaf, 7, Source::Deep(40)),
        reward: &[
            Coins(25000),
            Restore(town::WISH_TREE),
            item(Item::HeartCrystal, 3),
            item(Item::WishStar, 2),
        ],
        after: "thistle_market",
        depth: 40,
        hearts: 6,
        ..Q
    },
    // ---------------------------------------------------------------- Captain Rowan
    QuestDef {
        key: "rowan_slimes",
        giver: V::Rowan,
        title: "Slime Trouble",
        ask: "Slimes are creeping up out of the Hollow and into gardens. Nothing dangerous, \
              but Olive's furious. Knock out twelve of them for the Guild.",
        thanks: "Twelve slimes, no fuss. The Guild takes care of its own - here's your pay, \
                 and a shield. You'll want one.",
        goal: Slay(Some(Foe::Slime), 12, 1),
        reward: &[Coins(600), gear(Item::CopperShield, UNCOMMON), Marks(20)],
        ..Q
    },
    QuestDef {
        key: "rowan_map",
        giver: V::Rowan,
        title: "The Old Delver's Map",
        ask: "An old Guild map was torn up and scattered on the upper floors. The creatures \
              down there carry the scraps around like treasure. Bring me three and we'll \
              piece it together.",
        thanks: "It's whole again! Look at these notes... shortcuts, hidden rooms. Take these \
                 feathers - they'll fly you home when things go wrong.",
        goal: Gather(Item::MapScrap, 3, Source::Deep(3)),
        reward: &[Coins(1200), item(Item::Feather, 3), Marks(25)],
        after: "rowan_slimes",
        ..Q
    },
    QuestDef {
        key: "rowan_king",
        giver: V::Rowan,
        title: "Long Live the King?",
        ask: "Floor ten has a guardian: the King Slime. Big, bouncy and very full of itself. \
              Beat it, and the waystone there will carry you home.",
        thanks: "You beat the King! Word's already spreading - the Guild has a new hero. \
                 This helm has your name on it. Well, it will once I engrave it.",
        goal: Guardian(10),
        reward: &[Coins(2500), gear(Item::IronHelm, RARE), Marks(40)],
        after: "rowan_map",
        ..Q
    },
    QuestDef {
        key: "rowan_crabs",
        giver: V::Rowan,
        title: "Shell Shock",
        ask: "Crystal crabs are nesting in the Grotto and snapping at delvers. Clear out \
              twenty-five of them, floor 11 or deeper.",
        thanks: "Twenty-five crabs! My knees hurt just thinking about it. Here - a turtle \
                 shell shield. Sturdy, and frankly adorable.",
        goal: Slay(Some(Foe::Crab), 25, 11),
        reward: &[Coins(3500), gear(Item::TurtleShell, RARE), Marks(40)],
        after: "rowan_king",
        depth: 11,
        ..Q
    },
    QuestDef {
        key: "rowan_matriarch",
        giver: V::Rowan,
        title: "The Crystal Matriarch",
        ask: "The Crystal Matriarch rules floor twenty. Her shell turns steel. You'll need \
              every trick you've got, farmer. Good luck.",
        thanks: "The Matriarch, down! You're the real thing. This aegis was cut from crystal \
                 like hers - it's only fitting it goes to you.",
        goal: Guardian(20),
        reward: &[Coins(6000), gear(Item::CrystalAegis, EPIC), Marks(60)],
        after: "rowan_crabs",
        ..Q
    },
    QuestDef {
        key: "rowan_imps",
        giver: V::Rowan,
        title: "Imp Infestation",
        ask: "Fire imps in the Ember Depths are raiding each other and setting everything \
              alight. Thirty of them, floor 31 or deeper, and the Guild will be in your debt.",
        thanks: "Thirty imps and not a singed eyebrow? Impressive. Have a scroll from the \
                 Guild vault - a good one.",
        goal: Slay(Some(Foe::Imp), 30, 31),
        reward: &[Coins(9000), Scroll(Group::Weapon, EPIC), Marks(80)],
        after: "rowan_matriarch",
        depth: 31,
        ..Q
    },
    QuestDef {
        key: "rowan_colossus",
        giver: V::Rowan,
        title: "Thaw the Colossus",
        ask: "The Frost Colossus guards floor fifty. Sir Clank's old squad couldn't move it. \
              I think you can.",
        thanks: "The Colossus has fallen! Sir Clank actually wept. The Guild's greatest ward \
                 is yours - it's legendary, like you.",
        goal: Guardian(50),
        reward: &[Coins(20000), gear(Item::FrostWard, LEGEND), Marks(120)],
        after: "rowan_imps",
        depth: 45,
        ..Q
    },
    QuestDef {
        key: "rowan_warden",
        giver: V::Rowan,
        title: "The Bone Warden",
        ask: "Past the Frost Caverns lie the Sunken Ruins, and on floor sixty the Bone \
              Warden. No delver alive has beaten it. Yet.",
        thanks: "...You did it. The Bone Warden. I'll be telling this story until I'm older \
                 than Grandma Fern. The Starlight Sword - the Guild's treasure - is yours.",
        goal: Guardian(60),
        reward: &[
            Coins(35000),
            item(Item::HeartCrystal, 2),
            gear(Item::StarlightSword, LEGEND),
            Marks(200),
        ],
        after: "rowan_colossus",
        depth: 55,
        ..Q
    },
    // ---------------------------------------------------------------- Elder Quill
    QuestDef {
        key: "quill_specs",
        giver: V::Quill,
        title: "Lost Spectacles",
        ask: "My reading spectacles! I took them on a little stroll down the Hollow - floors \
              3 to 6, I believe - and came back without them. My eyes aren't what they were.",
        thanks: "Ah, the world is sharp again! And you're much less blurry than I thought. \
                 Take this scroll, with my thanks.",
        goal: Find(Item::Spectacles, 3, 6),
        reward: &[Coins(800), Scroll(Group::Weapon, RARE)],
        ..Q
    },
    QuestDef {
        key: "quill_enchant",
        giver: V::Quill,
        title: "A First Enchantment",
        ask: "Have you tried the enchanting table? Take a scroll to one - the one by your \
              house will do - and bind it to a piece of gear. Words become power!",
        thanks: "I felt it from here! A lovely binding. Keep practising - here are two more \
                 scrolls to play with.",
        goal: Enchant(1),
        reward: &[
            Coins(1000),
            Scroll(Group::Armor, RARE),
            Scroll(Group::Tool, RARE),
        ],
        after: "quill_specs",
        ..Q
    },
    QuestDef {
        key: "quill_wisps",
        giver: V::Quill,
        title: "Ink of Wisps",
        ask: "The finest scroll ink is made from wisp dust. The wisps float about the \
              Crystal Grotto. Fifteen pinches, if you please.",
        thanks: "Wonderful! It sparkles on the page. Here - a crystal wand, and some tonics \
                 to keep your mana topped up.",
        goal: Bring(Item::WispDust, 15),
        reward: &[
            Coins(2500),
            item(Item::ManaTonic, 5),
            gear(Item::CrystalWand, RARE),
        ],
        after: "quill_enchant",
        depth: 11,
        ..Q
    },
    QuestDef {
        key: "quill_crystal",
        giver: V::Quill,
        title: "The Singing Crystal",
        ask: "The creatures of the Crystal Grotto sometimes carry singing crystals. Five of \
              them together would ring in harmony... and I would very much like to hear it.",
        thanks: "Listen... do you hear the chord? Magnificent. A wish star, for you. Use it \
                 well - it will deepen your well of mana.",
        goal: Gather(Item::SingingCrystal, 5, Source::Biome(1)),
        reward: &[Coins(4000), item(Item::WishStar, 1)],
        after: "quill_wisps",
        ..Q
    },
    QuestDef {
        key: "quill_scholar",
        giver: V::Quill,
        title: "Scribe's Apprentice",
        ask: "You have a gift. Bind eight scrolls, and I'll call you my apprentice. The \
              scrolls must match the gear, remember: weapon, armour, tool.",
        thanks: "Eight bindings! Apprentice no longer - you're a scribe in your own right. \
                 Two of my finest scrolls.",
        goal: Enchant(8),
        reward: &[
            Coins(6000),
            Scroll(Group::Weapon, EPIC),
            Scroll(Group::Armor, EPIC),
        ],
        after: "quill_crystal",
        hearts: 4,
        ..Q
    },
    QuestDef {
        key: "quill_phoenix",
        giver: V::Quill,
        title: "The Phoenix Quill",
        ask: "Legends speak of a phoenix quill in the Ember Depths, floors 35 to 39. A scroll \
              written with it would be... well. Let's just say I'd write you a wand.",
        thanks: "It writes in firelight! As promised - a wand from the Moonquill vault. The \
                 Moonpetal. It's been waiting for someone like you.",
        goal: Find(Item::PhoenixQuill, 35, 39),
        reward: &[Coins(15000), gear(Item::MoonpetalWand, LEGEND)],
        after: "quill_scholar",
        depth: 35,
        ..Q
    },
    // ---------------------------------------------------------------- Hilde
    // Seasonal: only asked in its season.
    QuestDef {
        key: "hilde_kale",
        giver: V::Hilde,
        title: "Winter Warmth",
        ask: "The forge keeps my hands warm, but my belly's another story. Three kale, and I'll \
              make stew the way they do up north.",
        thanks: "Thick enough to stand a spoon in! That's how you know it's done. Three bowls for you.",
        goal: Bring(Item::Kale, 3),
        reward: &[Coins(1400), item(Item::KaleStew, 3)],
        day: 2,
        season: Some(Season::Winter),
        ..Q
    },
    QuestDef {
        key: "hilde_ore",
        giver: V::Hilde,
        title: "Copper for the Forge",
        ask: "I'm out of copper! Twenty chunks of copper ore and I'll hammer you out a helm \
              that'll make the slimes weep.",
        thanks: "HA! Lovely stuff! One copper helm, fresh off the anvil. Wear it proud!",
        goal: Bring(Item::CopperOre, 20),
        reward: &[Coins(700), gear(Item::CopperHelm, UNCOMMON)],
        ..Q
    },
    QuestDef {
        key: "hilde_shells",
        giver: V::Hilde,
        title: "Crab Shell Plating",
        ask: "Crab shells make the best padding - light and tough! Bring me twelve from the \
              Crystal Grotto.",
        thanks: "Perfect shells! Feel this mail - light as a feather, tough as nails.",
        goal: Bring(Item::CrabShell, 12),
        reward: &[Coins(1800), gear(Item::CopperMail, RARE)],
        after: "hilde_ore",
        depth: 11,
        ..Q
    },
    QuestDef {
        key: "hilde_letter",
        giver: V::Hilde,
        title: "A Letter for Garrick",
        ask: "Could you take this letter to Garrick at the smithy? It's, erm, about a delivery \
              of coal. Nothing else. Don't read it!",
        thanks: "...Hilde wrote to me? ...Hm. She says my beard is 'distinguished'. ...Thank \
                 you, farmer. Here. For your trouble.",
        goal: Deliver(Item::Letter, V::Garrick),
        reward: &[
            Coins(500),
            Friend(V::Garrick, 60),
            Friend(V::Hilde, 60),
            item(Item::BerryTart, 2),
        ],
        after: "hilde_shells",
        hearts: 2,
        ..Q
    },
    QuestDef {
        key: "hilde_beetles",
        giver: V::Hilde,
        title: "Beetle Shell Greaves",
        ask: "The beetles in the Fungal Hollow have gorgeous shells. Twenty of those and I'll \
              make the finest greaves you ever saw.",
        thanks: "Iridescent! Your legs will be the envy of the valley.",
        goal: Bring(Item::BeetleShell, 20),
        reward: &[Coins(4000), gear(Item::IronGreaves, RARE)],
        after: "hilde_letter",
        depth: 21,
        ..Q
    },
    QuestDef {
        key: "hilde_gold",
        giver: V::Hilde,
        title: "Golden Plate",
        ask: "I want to make a golden shield. Just once in my life! Twenty-five gold ore.",
        thanks: "Look at it gleam! I'm keeping the sketches. The shield is yours - you earned it.",
        goal: Bring(Item::GoldOre, 25),
        reward: &[Coins(6000), gear(Item::GoldShield, EPIC)],
        after: "hilde_beetles",
        depth: 25,
        ..Q
    },
    QuestDef {
        key: "hilde_masterwork",
        giver: V::Hilde,
        title: "Hilde's Masterwork",
        ask: "Every armourer has one masterwork in them. Mine needs frost opals from the \
              Frost Caverns - ten of them. It'll be a coat worthy of a legend.",
        thanks: "My masterwork. My life's best work. And I made it for you. Don't you DARE \
                 get a scratch on it. ...Fine, scratches are stories. Go make some.",
        goal: Bring(Item::FrostGem, 10),
        reward: &[Coins(12000), gear(Item::FrostCoat, LEGEND)],
        after: "hilde_gold",
        depth: 41,
        hearts: 5,
        ..Q
    },
    // ---------------------------------------------------------------- Garrick
    QuestDef {
        key: "garrick_wood",
        giver: V::Garrick,
        title: "Handles and Hilts",
        ask: "...Need wood. Forty. For handles.",
        thanks: "...Good wood. Made you a blade. It's a carrot. Don't ask.",
        goal: Bring(Item::Wood, 40),
        reward: &[Coins(500), gear(Item::CarrotBlade, UNCOMMON)],
        ..Q
    },
    QuestDef {
        key: "garrick_iron",
        giver: V::Garrick,
        title: "Iron Will",
        ask: "Twenty iron ore. You'll get a leek. A mighty one.",
        thanks: "...The Mighty Leek. Finest vegetable-shaped weapon in the valley. Swing true.",
        goal: Bring(Item::IronOre, 20),
        reward: &[Coins(2000), gear(Item::MightyLeek, RARE)],
        after: "garrick_wood",
        depth: 8,
        ..Q
    },
    QuestDef {
        key: "garrick_fangs",
        giver: V::Garrick,
        title: "Bat Fang Serrations",
        ask: "Bat fangs make good serrations. Bats only drop them when I need them. Weird, \
              that. Ten fangs.",
        thanks: "...Sharp. Good. Here's a scroll. Put it on something pointy.",
        goal: Gather(Item::BatFang, 10, Source::Foe(Foe::Bat)),
        reward: &[Coins(2500), Scroll(Group::Weapon, RARE)],
        after: "garrick_iron",
        ..Q
    },
    QuestDef {
        key: "garrick_ember",
        giver: V::Garrick,
        title: "Fire in the Forge",
        ask: "Ember ore burns hotter than coal. Fifteen chunks, from the Ember Depths. Then \
              I'll make you something with fire in it.",
        thanks: "...Feel the heat off that. An ember wand. Burns anything it touches. Careful.",
        goal: Bring(Item::EmberOre, 15),
        reward: &[Coins(7000), gear(Item::EmberWand, EPIC)],
        after: "garrick_fangs",
        depth: 31,
        ..Q
    },
    QuestDef {
        key: "garrick_heart",
        giver: V::Garrick,
        title: "Golem Heart Steel",
        ask: "Golem hearts. Five. Makes steel that never dulls. Frost Caverns.",
        thanks: "...Frost Fang. Cold as a winter night. You've earned it.",
        goal: Bring(Item::GolemHeart, 5),
        reward: &[Coins(12000), gear(Item::FrostFang, EPIC)],
        after: "garrick_ember",
        depth: 41,
        ..Q
    },
    QuestDef {
        key: "garrick_legend",
        giver: V::Garrick,
        title: "The Blade of Legends",
        ask: "...One more. The Ember Lord on floor forty has a gem in its crown. With it, I \
              can make the sword I've dreamed about for thirty years. It'll be waiting for \
              you on its floor while you're looking. Please.",
        thanks: "...It's done. The Starlight Sword. My best work. ...Your hands, my steel. \
                 Good partnership. ...Don't make it weird.",
        goal: Gather(Item::EmberGem, 1, Source::Boss(Foe::Imp)),
        reward: &[Coins(20000), gear(Item::StarlightSword, LEGEND)],
        after: "garrick_heart",
        depth: 40,
        hearts: 5,
        ..Q
    },
    // ---------------------------------------------------------------- Nix
    // Seasonal: only asked in its season.
    QuestDef {
        key: "nix_holly",
        giver: V::Nix,
        title: "Deck the Workshop",
        ask: "I'm decorating the whole workshop for the long nights! Holly, holly and more holly. \
              Six berries' worth?",
        thanks: "Merry! Bright! Prickly! Just how I like it. Have some cake - and mistletoe, if \
                 you're feeling brave.",
        goal: Bring(Item::HollyBerry, 6),
        reward: &[
            Coins(1800),
            item(Item::HollyCake, 2),
            item(Item::MistletoeSprig, 5),
        ],
        day: 2,
        season: Some(Season::Winter),
        ..Q
    },
    QuestDef {
        key: "nix_test",
        giver: V::Nix,
        title: "Field Test",
        ask: "I need data! Harvest twenty crops on your farm and tell me how your tools feel. \
              Squeaky? Wobbly? On fire? All useful!",
        thanks: "Twenty harvests and no explosions! Excellent data. Here, try my Sprout Hoe - \
                 it tills a longer row!",
        goal: Harvest(None, 20),
        reward: &[Coins(600), gear(Item::SproutHoe, UNCOMMON)],
        ..Q
    },
    QuestDef {
        key: "nix_cogs",
        giver: V::Nix,
        title: "Clockwork Cogs",
        ask: "The crabs in the Crystal Grotto hoard old brass cogs! Only when I'm looking for \
              them, oddly. Six cogs, please, for my sprinkler prototype!",
        thanks: "Cogs! Beautiful cogs! Two quality sprinklers, fresh off the bench. They water \
                 all eight tiles around them!",
        goal: Gather(Item::Cog, 6, Source::Foe(Foe::Crab)),
        reward: &[Coins(2000), item(Item::QualitySprinkler, 2)],
        after: "nix_test",
        depth: 11,
        ..Q
    },
    QuestDef {
        key: "nix_crystal",
        giver: V::Nix,
        title: "Glimmer Power",
        ask: "Glimmer shards hold a charge! Ten of them and I can make a watering can that \
              pours like a teapot. Don't ask how. I don't know either.",
        thanks: "It works! A Teapot Can - holds loads and pours just right. Tea not included.",
        goal: Bring(Item::Crystal, 10),
        reward: &[Coins(3500), gear(Item::TeapotCan, RARE)],
        after: "nix_cogs",
        ..Q
    },
    QuestDef {
        key: "nix_scrolls",
        giver: V::Nix,
        title: "Tool Tune-Up",
        ask: "Enchanted tools are the future! Bind three scrolls to anything at all and \
              report back. For science!",
        thanks: "Three bindings! The data is gorgeous. Here's a tool scroll I 'borrowed' from \
                 Quill. He won't notice. Probably.",
        goal: Enchant(3),
        reward: &[Coins(3000), Scroll(Group::Tool, EPIC)],
        after: "nix_crystal",
        ..Q
    },
    QuestDef {
        key: "nix_music",
        giver: V::Nix,
        title: "The Music Box Motor",
        ask: "Music boxes have the tiniest, most perfect little motors. If you find one in \
              the Hollow, I'll trade you my best invention yet.",
        thanks: "Tiny gears! Tiny perfect gears! Crystal sprinklers - they water a whole five \
                 by five patch. You'll never carry a can again!",
        goal: Bring(Item::MusicBox, 1),
        reward: &[Coins(5000), item(Item::CrystalSprinkler, 2)],
        after: "nix_scrolls",
        depth: 20,
        ..Q
    },
    QuestDef {
        key: "nix_masterpiece",
        giver: V::Nix,
        title: "Nix's Masterpiece",
        ask: "Okay. Okay okay okay. A star diamond. If I had one, I could build a watering can \
              that makes its own RAIN. Please please please?",
        thanks: "IT RAINS! IT ACTUALLY RAINS! The Raincloud Can is yours, partner. Best. \
                 Invention. EVER.",
        goal: Bring(Item::StarDiamond, 1),
        reward: &[Coins(15000), gear(Item::CloudCan, LEGEND)],
        after: "nix_music",
        depth: 35,
        hearts: 5,
        ..Q
    },
    // ---------------------------------------------------------------- Opal
    QuestDef {
        key: "opal_gems",
        giver: V::Opal,
        title: "A Little Sparkle",
        ask: "Darling, I'm dying to see what the Hollow offers these days. Bring me a topaz - \
              warm as sunset, they are.",
        thanks: "Exquisite. Here, a moonstone in return - moonlight that fell asleep.",
        goal: Bring(Item::Topaz, 1),
        reward: &[Coins(800), item(Item::Moonstone, 1)],
        depth: 5,
        ..Q
    },
    QuestDef {
        key: "opal_curios",
        giver: V::Opal,
        title: "The Curio Cabinet",
        ask: "I keep a cabinet of every curio the Hollow gives up. Find five different gems or \
              relics - just show me you've held them - and I'll reward you handsomely.",
        thanks: "Five treasures! You have a collector's eye, darling. A circlet to match it.",
        goal: Curios(5),
        reward: &[Coins(2000), gear(Item::CrystalCirclet, RARE)],
        after: "opal_gems",
        ..Q
    },
    QuestDef {
        key: "opal_pearl",
        giver: V::Opal,
        title: "The Matriarch's Pearl",
        ask: "The Crystal Matriarch on floor twenty grows a pearl in her shell. The finest in \
              the world. She'll be back on her floor while you look for it. Bring it to me?",
        thanks: "Oh... it's perfect. I'll set it in the cabinet's centre. Take this moon pearl, \
                 and my eternal gratitude.",
        goal: Gather(Item::MatriarchPearl, 1, Source::Boss(Foe::Crab)),
        reward: &[Coins(6000), item(Item::MoonPearl, 1)],
        after: "opal_curios",
        depth: 20,
        ..Q
    },
    QuestDef {
        key: "opal_curios2",
        giver: V::Opal,
        title: "A Cabinet of Wonders",
        ask: "Twelve different curios, darling. Gems, relics, the odd crown. Show me the Hollow's \
              treasures!",
        thanks: "Twelve! The cabinet sings. A star diamond, for the star of my collection.",
        goal: Curios(12),
        reward: &[Coins(8000), item(Item::StarDiamond, 1)],
        after: "opal_pearl",
        ..Q
    },
    QuestDef {
        key: "opal_moon",
        giver: V::Opal,
        title: "Moonlight in a Box",
        ask: "I'm making mail. Crystal mail, set with moonstones. Three moonstones and it's yours.",
        thanks: "It shimmers! You'll catch the light everywhere you go, darling.",
        goal: Bring(Item::Moonstone, 3),
        reward: &[Coins(10000), gear(Item::CrystalMail, EPIC)],
        after: "opal_curios2",
        depth: 30,
        ..Q
    },
    QuestDef {
        key: "opal_complete",
        giver: V::Opal,
        title: "Every Sparkle",
        ask: "Every gem and every relic in the Hollow. All nineteen. The complete cabinet. It's \
              never been done. Will you be the one?",
        thanks: "Complete. The first complete cabinet in history, and it's ours. The Ember Crown - \
                 a legend for a legend. And two heart crystals, because I adore you.",
        goal: Curios(19),
        reward: &[
            Coins(30000),
            item(Item::HeartCrystal, 2),
            gear(Item::EmberCrown, LEGEND),
        ],
        after: "opal_moon",
        hearts: 6,
        ..Q
    },
    // ---------------------------------------------------------------- Wren
    // Seasonal: only asked in its season.
    QuestDef {
        key: "wren_poppies",
        giver: V::Wren,
        title: "Painted Poppies",
        ask: "I'm painting poppies on a dresser, and I paint best from life. Eight in a jar on the \
              bench?",
        thanks: "Look at those petals on the drawers! Keep these seeds - cosmos and hibiscus, for \
                 my next piece.",
        goal: Bring(Item::Poppy, 8),
        reward: &[
            Coins(1500),
            item(Item::CosmosSeeds, 8),
            item(Item::HibiscusSeeds, 5),
        ],
        day: 2,
        season: Some(Season::Summer),
        ..Q
    },
    QuestDef {
        key: "wren_wood",
        giver: V::Wren,
        title: "Timber!",
        ask: "I'm building benches for the plaza, but I'm out of wood. Sixty pieces would get \
              everyone sitting pretty.",
        thanks: "Lovely grain! Two benches for your farm, too. Sit, relax, enjoy.",
        goal: Bring(Item::Wood, 60),
        reward: &[Coins(600), item(Item::Bench, 2)],
        ..Q
    },
    QuestDef {
        key: "wren_cozy",
        giver: V::Wren,
        title: "Cozy Corner",
        ask: "Cushions! I need stuffing. Forty bundles of fibre, from weeds and grass.",
        thanks: "So soft! Here - lamps and flower pots, to make your farm feel like a hug.",
        goal: Bring(Item::Fiber, 40),
        reward: &[Coins(1200), item(Item::Lamp, 2), item(Item::FlowerPot, 3)],
        after: "wren_wood",
        ..Q
    },
    QuestDef {
        key: "wren_teacup",
        giver: V::Wren,
        title: "The Chipped Teacup",
        ask: "There's a pattern of teacup that went missing from the valley long ago. Sometimes \
              they turn up in the Hollow, chipped but whole. I'd treasure one.",
        thanks: "It's the one! Look at the little roses. I made you a poncho - it's the coziest \
                 thing I've ever knitted.",
        goal: Bring(Item::ChippedTeacup, 1),
        reward: &[Coins(3000), gear(Item::WoollyPoncho, RARE)],
        after: "wren_cozy",
        depth: 8,
        ..Q
    },
    QuestDef {
        key: "wren_parcel",
        giver: V::Wren,
        title: "Special Delivery",
        ask: "I made Grandma Fern a quilt, but my ankle's sore. Would you take the parcel to her?",
        thanks: "A quilt? From Wren? Oh, it has little flowers on it! What a dear girl. And \
                 what a dear you are, for bringing it.",
        goal: Deliver(Item::Parcel, V::Fern),
        reward: &[
            Coins(800),
            Friend(V::Fern, 80),
            Friend(V::Wren, 40),
            item(Item::CarrotCake, 2),
        ],
        after: "wren_teacup",
        hearts: 3,
        ..Q
    },
    QuestDef {
        key: "wren_amber",
        giver: V::Wren,
        title: "Amber Inlay",
        ask: "I want to inlay amber into my chests - they'd glow like honey. Ten pieces of amber.",
        thanks: "Honey-glow chests! Three of them for you, and a crystal sprinkler I got in trade.",
        goal: Bring(Item::Amber, 10),
        reward: &[
            Coins(7000),
            item(Item::Chest, 3),
            item(Item::CrystalSprinkler, 1),
        ],
        after: "wren_parcel",
        depth: 20,
        ..Q
    },
    // ---------------------------------------------------------------- Posy
    // Seasonal: only asked in its season.
    QuestDef {
        key: "posy_tulips",
        giver: V::Posy,
        title: "First Blooms",
        ask: "Spring! Spring spring spring! I want the very first tulips of the year in my window. \
              Five pastel tulips? Pleeease?",
        thanks: "My window's a rainbow! Here - daffodil and bluebell bulbs. Fill your fields with \
                 spring!",
        goal: Bring(Item::PastelTulip, 5),
        reward: &[
            Coins(1200),
            item(Item::DaffodilBulb, 8),
            item(Item::BluebellBulb, 8),
        ],
        day: 2,
        season: Some(Season::Spring),
        ..Q
    },
    QuestDef {
        key: "posy_turnips",
        giver: V::Posy,
        title: "Turnip Tuesday",
        ask: "Turnips are the friendliest crop! Grow me ten and I'll give you something much \
              more exciting to plant!",
        thanks: "Ten plump turnips! Here - strawberry seeds! They keep fruiting after the first \
                 harvest!",
        goal: Bring(Item::Turnip, 10),
        reward: &[Coins(500), item(Item::StrawberrySeeds, 10)],
        ..Q
    },
    QuestDef {
        key: "posy_almanac1",
        giver: V::Posy,
        title: "The Seed Almanac",
        ask: "I'm writing an almanac of every crop in the valley! Harvest eight different kinds \
              and tell me all about them!",
        thanks: "Eight crops! My almanac is growing! Moonbloom seeds - they glow at night!",
        goal: Almanac(8),
        reward: &[Coins(1500), item(Item::MoonbloomSeeds, 5)],
        after: "posy_turnips",
        ..Q
    },
    QuestDef {
        key: "posy_roses",
        giver: V::Posy,
        title: "A Rose for Everyone",
        ask: "I want to give everyone in town a rose! That's... a lot of roses. Eight to start?",
        thanks: "The town smells SO nice now! Here's a flower crown, just like mine!",
        goal: Bring(Item::Rose, 8),
        reward: &[Coins(2500), gear(Item::FlowerCrown, RARE)],
        after: "posy_almanac1",
        ..Q
    },
    QuestDef {
        key: "posy_almanac2",
        giver: V::Posy,
        title: "Almanac of the Deep",
        ask: "Eighteen different crops! Including the strange ones from deep in the Hollow!",
        thanks: "Eighteen! You're a real botanist! Starfruit seeds - the rarest I've got!",
        goal: Almanac(18),
        reward: &[Coins(6000), item(Item::StarfruitSeeds, 5)],
        after: "posy_roses",
        depth: 15,
        ..Q
    },
    QuestDef {
        key: "posy_pod",
        giver: V::Posy,
        title: "The Ancient Seed",
        ask: "Juniper says there's an ancient seed pod sleeping on floors 30 to 34. A seed from \
              before the valley even had a name! Could you find it?",
        thanks: "It's warm! It's ALIVE! I'll plant it in the greenhouse. Take these - truffle \
                 spores and ancient grain, my rarest seeds!",
        goal: Find(Item::SeedPod, 30, 34),
        reward: &[
            Coins(9000),
            item(Item::TruffleSpores, 8),
            item(Item::AncientGrainSeeds, 10),
        ],
        after: "posy_almanac2",
        depth: 30,
        ..Q
    },
    QuestDef {
        key: "posy_almanac3",
        giver: V::Posy,
        title: "The Great Almanac",
        ask: "Thirty-eight different crops! Every one from the valley and the Hollow! Then my \
              almanac will be nearly complete!",
        thanks: "THIRTY-EIGHT! Nobody's ever filled so many pages! You're the best farmer in the \
                 whole world! Take my crystal hoe - I've been saving it for someone special!",
        goal: Almanac(38),
        reward: &[
            Coins(30000),
            item(Item::SunStone, 2),
            gear(Item::CrystalHoe, LEGEND),
        ],
        after: "posy_pod",
        hearts: 6,
        ..Q
    },
    QuestDef {
        key: "posy_almanac4",
        giver: V::Posy,
        title: "A Year in Bloom",
        ask: "The seasonal crops are pages too! Spring, summer, autumn AND winter! All \
              seventy-eight crops, and my almanac will truly, truly be complete!",
        thanks: "Every page, every season, every crop! It's COMPLETE! This crown grew with the \
                 almanac - it's only ever been meant for you.",
        goal: Almanac(78),
        reward: &[
            Coins(60000),
            item(Item::StarfruitSeeds, 10),
            gear(Item::FlowerCrown, LEGEND),
        ],
        after: "posy_almanac3",
        hearts: 8,
        ..Q
    },
    // ---------------------------------------------------------------- Mabel
    // Seasonal: only asked in its season.
    QuestDef {
        key: "mabel_cherries",
        giver: V::Mabel,
        title: "Cherry Season",
        ask: "The cherry trees used to turn this whole street pink. I miss cherry tarts something \
              awful. Six sweet cherries, dear?",
        thanks: "Pitted, baked and golden! Three tarts, still warm. Mind the plate.",
        goal: Bring(Item::SweetCherry, 6),
        reward: &[Coins(1500), item(Item::CherryTart, 3)],
        day: 2,
        season: Some(Season::Spring),
        ..Q
    },
    QuestDef {
        key: "mabel_cranberries",
        giver: V::Mabel,
        title: "Pie Season",
        ask: "Autumn is pie season, and pie season needs cranberries! Five, and I'll show you \
              what a proper pie looks like.",
        thanks: "A crust you could write home about! Two pumpkin pies for you - the cranberry ones \
                 are going to the tavern.",
        goal: Bring(Item::Cranberry, 5),
        reward: &[Coins(1600), item(Item::PumpkinPie, 2)],
        day: 2,
        season: Some(Season::Autumn),
        ..Q
    },
    QuestDef {
        key: "mabel_wheat",
        giver: V::Mabel,
        title: "Flour Power",
        ask: "I'm short on wheat, dearie. Fifteen bundles and I'll bake you a batch of bread.",
        thanks: "Golden wheat! Here, fresh from the oven - careful, it's hot!",
        goal: Bring(Item::Wheat, 15),
        reward: &[Coins(500), item(Item::FreshBread, 5)],
        ..Q
    },
    QuestDef {
        key: "mabel_berries",
        giver: V::Mabel,
        title: "Berry Season",
        ask: "Strawberries! Ten of them and I'll make shortcake like my mother used to.",
        thanks: "Oh, they're perfect! Three shortcakes, dear. Don't eat them all at once. Or do.",
        goal: Bring(Item::Strawberry, 10),
        reward: &[Coins(1500), item(Item::Shortcake, 3)],
        after: "mabel_wheat",
        ..Q
    },
    QuestDef {
        key: "mabel_whisk",
        giver: V::Mabel,
        title: "The Golden Whisk",
        ask: "My grandmother's golden whisk! It fell down the Hollow when I was a girl - floors 7 \
              to 12, my father always said. Nothing whips cream like it.",
        thanks: "My whisk! Oh, I can hear Gran laughing. Pies for you, and a sun stone - it'll \
                 give you energy for days!",
        goal: Find(Item::GoldenWhisk, 7, 12),
        reward: &[
            Coins(3000),
            item(Item::PumpkinPie, 3),
            item(Item::SunStone, 1),
        ],
        after: "mabel_berries",
        ..Q
    },
    QuestDef {
        key: "mabel_basket",
        giver: V::Mabel,
        title: "Pie for Grandma",
        ask: "Would you carry this basket to Grandma Fern? Her favourite pie. She'll pretend she \
              doesn't want it.",
        thanks: "Oh, I couldn't possibly... oh, it's cinnamon. Well. Just a slice. Maybe two. \
                 Thank you, dear.",
        goal: Deliver(Item::PieBasket, V::Fern),
        reward: &[
            Coins(600),
            Friend(V::Fern, 80),
            Friend(V::Mabel, 40),
            item(Item::BerryTart, 3),
        ],
        after: "mabel_whisk",
        hearts: 2,
        ..Q
    },
    QuestDef {
        key: "mabel_pumpkins",
        giver: V::Mabel,
        title: "Pumpkin Festival",
        ask: "I want to bake the biggest pumpkin pie the valley has ever seen! Six spore \
              pumpkins, dear.",
        thanks: "It's ENORMOUS! The whole town's having a slice. And here - bloomers! Pumpkin \
                 bloomers! Hilde owed me a favour.",
        goal: Bring(Item::SporePumpkin, 6),
        reward: &[
            Coins(6000),
            item(Item::PumpkinPie, 5),
            gear(Item::PumpkinBloomers, EPIC),
        ],
        after: "mabel_basket",
        depth: 21,
        ..Q
    },
    QuestDef {
        key: "mabel_starfruit",
        giver: V::Mabel,
        title: "The Starfruit Tart",
        ask: "Every baker dreams of a starfruit tart. Three starfruit, and I can finally make one.",
        thanks: "It's... it's beautiful. I'm crying into the pastry. Five tarts for you, dear, \
                 and a heart crystal. You've given an old baker her dream.",
        goal: Bring(Item::Starfruit, 3),
        reward: &[
            Coins(15000),
            item(Item::StarfruitTart, 5),
            item(Item::HeartCrystal, 1),
        ],
        after: "mabel_pumpkins",
        hearts: 5,
        ..Q
    },
    // ---------------------------------------------------------------- Barley
    // Seasonal: only asked in its season.
    QuestDef {
        key: "barley_melons",
        giver: V::Barley,
        title: "Summer Refreshment",
        ask: "Phew, it's a hot one. Nothing beats cold watermelon after a day in the fields. Two of \
              the big ones?",
        thanks: "Crunchy, sweet and cold! I blended the rest. Slushes, for the hottest days.",
        goal: Bring(Item::Watermelon, 2),
        reward: &[Coins(1800), item(Item::WatermelonSlush, 4)],
        day: 2,
        season: Some(Season::Summer),
        ..Q
    },
    QuestDef {
        key: "barley_stew",
        giver: V::Barley,
        title: "Stew Supplies",
        ask: "The stew pot's running low, friend! Ten potatoes and there's a bowl in it for you.",
        thanks: "Beauties! Three bowls of Barley's famous stew. Fills you right up!",
        goal: Bring(Item::Potato, 10),
        reward: &[Coins(600), item(Item::VeggieStew, 3)],
        ..Q
    },
    QuestDef {
        key: "barley_cookbook",
        giver: V::Barley,
        title: "Barley's Cookbook",
        ask: "I'm collecting recipes! Cook five different dishes at a workbench and tell me how \
              they came out.",
        thanks: "Five dishes! You've got a cook's hands. Try my ember curry - it'll put fire in \
                 your belly.",
        goal: Cookbook(5),
        reward: &[Coins(2000), item(Item::EmberCurry, 3)],
        after: "barley_stew",
        ..Q
    },
    QuestDef {
        key: "barley_gossip",
        giver: V::Barley,
        title: "Barley's Big Gossip",
        ask: "Something's going on in town and I'm the last to know! Go chat with Toby, Bramble, \
              Fern and Pip and see what's what.",
        thanks: "Hilde wrote Garrick a LETTER? And Pip wants to be a knight now? Oh, this is \
                 good. Popcorn's on the house!",
        goal: Meet(meet(&[V::Toby, V::Bramble, V::Fern, V::Pip])),
        reward: &[Coins(1200), item(Item::Popcorn, 5), Friend(V::Barley, 60)],
        after: "barley_cookbook",
        ..Q
    },
    QuestDef {
        key: "barley_lemons",
        giver: V::Barley,
        title: "Lava Lemonade",
        ask: "Customers keep asking for something fizzy. Lava lemons! Eight of them, from those \
              seeds out of the Ember Depths.",
        thanks: "Fizzy AND spicy! It's a hit! Five bottles for you, on the house.",
        goal: Bring(Item::LavaLemon, 8),
        reward: &[Coins(5000), item(Item::LavaLemonade, 5)],
        after: "barley_gossip",
        depth: 33,
        ..Q
    },
    QuestDef {
        key: "barley_cookbook2",
        giver: V::Barley,
        title: "The Great Cookbook",
        ask: "Fifteen different dishes! We'll put together the finest cookbook in the valley.",
        thanks: "Fifteen! We'll call it 'Barley and the Farmer's Kitchen'. Truffle risotto for \
                 you, and a sun stone I've been saving.",
        goal: Cookbook(15),
        reward: &[
            Coins(12000),
            item(Item::TruffleRisotto, 3),
            item(Item::SunStone, 1),
        ],
        after: "barley_lemons",
        ..Q
    },
    QuestDef {
        key: "barley_truffle",
        giver: V::Barley,
        title: "Truffle Hunt",
        ask: "Truffles. The black gold of the kitchen. Three of them, and I'll give you the \
              greatest thing an innkeeper owns.",
        thanks: "Real truffles! As promised - a heart crystal. My old adventuring days' finest \
                 find. And a staff, because you'll want to protect that heart!",
        goal: Bring(Item::Truffle, 3),
        reward: &[
            Coins(15000),
            item(Item::HeartCrystal, 1),
            gear(Item::MushroomStaff, EPIC),
        ],
        after: "barley_cookbook2",
        hearts: 5,
        ..Q
    },
    // ---------------------------------------------------------------- Grandma Fern
    // Seasonal: only asked in its season.
    QuestDef {
        key: "fern_rhubarb",
        giver: V::Fern,
        title: "Rhubarb and Custard",
        ask: "Albert grew the finest rhubarb in the valley. Every spring I made crumble for the \
              whole street. Four stalks, dear, and I'll make it again.",
        thanks: "It tastes just like it used to. Two helpings for you - and lavender seeds from \
                 Albert's old tin.",
        goal: Bring(Item::Rhubarb, 4),
        reward: &[
            Coins(1600),
            item(Item::RhubarbCrumble, 2),
            item(Item::LavenderSeeds, 10),
        ],
        day: 2,
        season: Some(Season::Spring),
        ..Q
    },
    QuestDef {
        key: "fern_locket",
        giver: V::Fern,
        title: "Albert's Locket",
        ask: "My Albert gave me a locket, and I lost it down the Hollow picking mushrooms. Floors \
              2 to 5, I think. I'm too old to look now. Would you, dear?",
        thanks: "Oh... there he is. My Albert. Thank you, dear, thank you. Take this - he found it \
                 in the Hollow himself. He'd want you to have it.",
        goal: Find(Item::Locket, 2, 5),
        reward: &[Coins(800), item(Item::HeartCrystal, 1), Friend(V::Fern, 60)],
        ..Q
    },
    QuestDef {
        key: "fern_peas",
        giver: V::Fern,
        title: "Sweet Peas for the Porch",
        ask: "My porch used to be covered in sweet peas. Ten, dear, and it'll smell like summer \
              again.",
        thanks: "Summer! Here - I knitted this for Albert, but it'll suit you better.",
        goal: Bring(Item::SweetPea, 10),
        reward: &[Coins(1500), gear(Item::WoollyPoncho, RARE)],
        after: "fern_locket",
        ..Q
    },
    QuestDef {
        key: "fern_watch",
        giver: V::Fern,
        title: "Albert's Pocket Watch",
        ask: "Albert's pocket watch went with him on his last delve. He wrote that he left it on \
              floor fifteen or so, to mark the way home. Floors 14 to 18.",
        thanks: "It's still ticking... after all these years. Just like him. Feathers, for coming \
                 home safe, and a sun stone.",
        goal: Find(Item::PocketWatch, 14, 18),
        reward: &[Coins(4000), item(Item::Feather, 5), item(Item::SunStone, 1)],
        after: "fern_peas",
        depth: 14,
        ..Q
    },
    QuestDef {
        key: "fern_tea",
        giver: V::Fern,
        title: "A Tea Party",
        ask: "I'd like to throw a tea party for the whole street. Twelve blueberries for the \
              muffins, dear?",
        thanks: "The muffins came out perfect! Everyone came! Five for you, still warm.",
        goal: Bring(Item::Blueberry, 12),
        reward: &[Coins(3000), item(Item::BlueberryMuffin, 5)],
        after: "fern_watch",
        ..Q
    },
    QuestDef {
        key: "fern_story",
        giver: V::Fern,
        title: "Albert's Last Delve",
        ask: "Albert once reached floor fifty. Nobody believed him. Would you go, dear? And tell \
              me what he saw?",
        thanks: "Frost caverns, and ice that glows... just as he said. He wasn't fibbing after all. \
                 These were his boots. Feathers on the heels. Walk well, dear.",
        goal: Reach(50),
        reward: &[
            Coins(15000),
            gear(Item::FeatherBoots, LEGEND),
            item(Item::HeartCrystal, 1),
        ],
        after: "fern_tea",
        hearts: 5,
        ..Q
    },
    // ---------------------------------------------------------------- Pip
    // Seasonal: only asked in its season.
    QuestDef {
        key: "pip_pineapple",
        giver: V::Pip,
        title: "Pineapple Party!",
        ask: "I'm throwing a PINEAPPLE PARTY. It's like a normal party but with a pineapple. I just \
              need the pineapple. One! Please!",
        thanks: "It's SPIKY! And it smells like sunshine! You're invited, obviously. Here's cake. I \
                 made it. Mom helped.",
        goal: Bring(Item::Pineapple, 1),
        reward: &[
            Coins(900),
            item(Item::PineappleCake, 2),
            item(Item::PineappleTop, 2),
        ],
        day: 2,
        season: Some(Season::Summer),
        ..Q
    },
    // Only in autumn, and only ever once: the egg that hatches the farm's jumping spider.
    QuestDef {
        key: "pip_candy",
        giver: V::Pip,
        title: "A Sweet Secret",
        ask: "Psst! Closer! Every autumn, when floor ten's guardian falls, CANDY ROCKS burst \
              out of the ground! Nobody believes me. Bring me five and I'll trade you my \
              biggest secret. It's round. That's all I'm saying.",
        thanks: "They're REAL! Okay, a deal's a deal. I found this egg in the leaf pile. It hums \
                 at night! Set it down on your farm... and wait. Don't tell Mom.",
        goal: Gather(Item::CandyRock, 5, Source::Eruption(10)),
        reward: &[item(Item::MysteryEgg, 1)],
        season: Some(Season::Autumn),
        ..Q
    },
    QuestDef {
        key: "pip_teddy",
        giver: V::Pip,
        title: "Patches is Missing!",
        ask: "My teddy, Patches, went on an adventure without me! He's down the Hollow, I KNOW \
              it. Floors 1 to 3! Please find him!",
        thanks: "PATCHES! You're safe! You're a real hero! Here, my best cookies. I only ate one.",
        goal: Find(Item::Teddy, 1, 3),
        reward: &[
            Coins(300),
            item(Item::SunflowerCookies, 5),
            Friend(V::Pip, 80),
        ],
        ..Q
    },
    QuestDef {
        key: "pip_slime",
        giver: V::Pip,
        title: "Slime Hearts!",
        ask: "Did you know slimes have little hearts? Only sometimes! Bring me eight, I want to \
              see them jiggle!",
        thanks: "They JIGGLE! This is the best day ever! Popcorn! Mom says I have to share.",
        goal: Gather(Item::SlimeHeart, 8, Source::Foe(Foe::Slime)),
        reward: &[Coins(1500), item(Item::Popcorn, 5)],
        after: "pip_teddy",
        day: 3,
        ..Q
    },
    QuestDef {
        key: "pip_bugs",
        giver: V::Pip,
        title: "Bug Collector",
        ask: "Glow beetles! The beetles in the Fungal Hollow carry little glowing friends! Five \
              for my collection, pleeease!",
        thanks: "They GLOW! Sir Crunch has friends now! Here, my bunny hood. I'm getting a new one.",
        goal: Gather(Item::GlowBeetle, 5, Source::Foe(Foe::Beetle)),
        reward: &[Coins(2000), gear(Item::BunnyHood, RARE)],
        after: "pip_slime",
        depth: 21,
        ..Q
    },
    QuestDef {
        key: "pip_duck",
        giver: V::Pip,
        title: "Rubber Ducky",
        ask: "I heard there are RUBBER DUCKS in the Hollow. Is that true? I need one. It's very \
              important. Bath time important.",
        thanks: "SQUEAK! It's perfect! You can have my bubble wand. It makes bubbles! Magic ones!",
        goal: Bring(Item::RubberDuck, 1),
        reward: &[Coins(2500), gear(Item::BubbleWand, RARE)],
        after: "pip_bugs",
        ..Q
    },
    QuestDef {
        key: "pip_hero",
        giver: V::Pip,
        title: "Pip's Hero",
        ask: "Old Capwood lives on floor thirty! He's a GIANT MUSHROOM! If you beat him you'll be \
              the greatest adventurer EVER!",
        thanks: "YOU DID IT! I told EVERYONE! Mom helped me get you this. It's a real sword!",
        goal: Guardian(30),
        reward: &[Coins(8000), gear(Item::CapwoodSabre, EPIC)],
        after: "pip_duck",
        depth: 28,
        ..Q
    },
    QuestDef {
        key: "pip_training",
        giver: V::Pip,
        title: "Adventurer Training",
        ask: "Teach me to be a hero! Um. By being one. Defeat a hundred monsters and I'll write \
              it all down!",
        thanks: "A HUNDRED! My notebook's full! Here - my very first sword. I enchanted it with \
                 friendship. That makes it legendary.",
        goal: Slay(None, 100, 1),
        reward: &[
            Coins(5000),
            item(Item::Feather, 5),
            gear(Item::TwigSword, LEGEND),
        ],
        after: "pip_hero",
        hearts: 4,
        ..Q
    },
    // ---------------------------------------------------------------- Juniper
    // Seasonal: only asked in its season.
    QuestDef {
        key: "juniper_snowdrops",
        giver: V::Juniper,
        title: "Snowdrops in the Snow",
        ask: "Snowdrops push up through the frost when nothing else dares. I'd like five for my \
              study. Brave little things.",
        thanks: "They nod at me while I read. Here: snow rose seeds. They glow a little on the \
                 coldest nights.",
        goal: Bring(Item::Snowdrop, 5),
        reward: &[Coins(1600), item(Item::SnowRoseSeeds, 5)],
        day: 2,
        season: Some(Season::Winter),
        ..Q
    },
    QuestDef {
        key: "juniper_moss",
        giver: V::Juniper,
        title: "Moonmoss Samples",
        ask: "The creatures of the Mossy Burrows carry moonmoss in their fur. It only glows when \
              I'm studying it - fascinating! Five samples, please.",
        thanks: "Glowing beautifully! Here, glowcap spores. They grow into little lanterns.",
        goal: Gather(Item::Moonmoss, 5, Source::Biome(0)),
        reward: &[Coins(700), item(Item::GlowcapSpores, 10)],
        ..Q
    },
    QuestDef {
        key: "juniper_spores",
        giver: V::Juniper,
        title: "Rainbow Spores",
        ask: "The shroomlings of the Fungal Hollow shed rainbow spores. Six of them would change \
              my whole theory.",
        thanks: "My theory holds! They're lonely! ...That's the whole theory. Jelly shroom spores, \
                 for your farm.",
        goal: Gather(Item::RainbowSpore, 6, Source::Foe(Foe::Shroom)),
        reward: &[Coins(2500), item(Item::JellySpores, 8)],
        after: "juniper_moss",
        depth: 21,
        ..Q
    },
    QuestDef {
        key: "juniper_caps",
        giver: V::Juniper,
        title: "Shroomling Survey",
        ask: "I'm cataloguing shroomling caps. Fifteen should give me a good sample size.",
        thanks: "Perfect specimens! A mushroom cap for you - it suits you, honestly.",
        goal: Bring(Item::ShroomCap, 15),
        reward: &[Coins(2500), gear(Item::MushroomCap, RARE)],
        after: "juniper_spores",
        ..Q
    },
    QuestDef {
        key: "juniper_acorn",
        giver: V::Juniper,
        title: "Capwood's Acorn",
        ask: "Old Capwood, the guardian of floor thirty, grows one acorn a century. It'll be \
              waiting on its floor while you look. Imagine what it could grow into!",
        thanks: "It's humming! Truffle spores for you, and a staff grown from Capwood's own wood.",
        goal: Gather(Item::CapwoodAcorn, 1, Source::Boss(Foe::Shroom)),
        reward: &[
            Coins(8000),
            item(Item::TruffleSpores, 5),
            gear(Item::MushroomStaff, EPIC),
        ],
        after: "juniper_caps",
        depth: 30,
        ..Q
    },
    QuestDef {
        key: "juniper_frost",
        giver: V::Juniper,
        title: "Frost Blossoms",
        ask: "Flowers of ice, in the Frost Caverns! The creatures there carry them. Six, and I'll \
              share my rarest bulbs.",
        thanks: "They'll never melt! Frost lily bulbs and frost mint seeds - grow a winter garden!",
        goal: Gather(Item::FrostBlossom, 6, Source::Biome(4)),
        reward: &[
            Coins(10000),
            item(Item::LilyBulb, 10),
            item(Item::FrostMintSeeds, 10),
        ],
        after: "juniper_acorn",
        depth: 41,
        ..Q
    },
    QuestDef {
        key: "juniper_codex",
        giver: V::Juniper,
        title: "A Field Guide",
        ask: "Let's write a field guide to the Hollow's creatures! Defeat every kind - all ten - \
              so we know what we're dealing with.",
        thanks: "All ten! The first complete field guide! A heart crystal, and my leaf tunic - \
                 woven from moonmoss. It's the best thing I own.",
        goal: Codex(10),
        reward: &[
            Coins(15000),
            item(Item::HeartCrystal, 1),
            gear(Item::LeafTunic, LEGEND),
        ],
        after: "juniper_frost",
        hearts: 5,
        ..Q
    },
    // ---------------------------------------------------------------- Bramble
    QuestDef {
        key: "bramble_pages",
        giver: V::Bramble,
        title: "The Lost Song",
        ask: "My greatest song, blown away page by page down the Hollow! The creatures there \
              must have them. Four pages, from floor five on down!",
        thanks: "My song! Ahem. *Laaaa*. Perfect! A music box, for you - to remember this moment.",
        goal: Gather(Item::SongPage, 4, Source::Deep(5)),
        reward: &[Coins(1000), item(Item::MusicBox, 1), Friend(V::Bramble, 40)],
        depth: 5,
        ..Q
    },
    QuestDef {
        key: "bramble_tale",
        giver: V::Bramble,
        title: "A Tale Worth Telling",
        ask: "For your ballad's second verse, I need adventure! Reach floor twenty and come tell \
              me everything!",
        thanks: "Crystal caverns and a giant crab? Verse two writes itself! A hood, for the \
                 bard's favourite hero.",
        goal: Reach(20),
        reward: &[Coins(3000), gear(Item::CatHood, RARE)],
        after: "bramble_pages",
        ..Q
    },
    QuestDef {
        key: "bramble_lemonade",
        giver: V::Bramble,
        title: "Liquid Courage",
        ask: "Before a big show I like a lava lemonade. Or two. Barley makes them... could you \
              bring me two?",
        thanks: "Spicy! My voice has never been better! A scroll, for your armour. From a fan.",
        goal: Bring(Item::LavaLemonade, 2),
        reward: &[Coins(4000), Scroll(Group::Armor, EPIC)],
        after: "bramble_tale",
        depth: 33,
        ..Q
    },
    QuestDef {
        key: "bramble_ghosts",
        giver: V::Bramble,
        title: "Songs of the Sunken",
        ask: "The ghosts of the Sunken Ruins carry lanterns. They say each one holds a song. Bring \
              me five, and I'll learn them all.",
        thanks: "Five songs! Old and sad and lovely. A star wand, for bringing music back from \
                 the dark.",
        goal: Gather(Item::GhostLantern, 5, Source::Foe(Foe::Ghost)),
        reward: &[Coins(12000), gear(Item::StarWand, EPIC)],
        after: "bramble_lemonade",
        depth: 51,
        ..Q
    },
    QuestDef {
        key: "bramble_ballad",
        giver: V::Bramble,
        title: "The Ballad of the Farmer",
        ask: "The final verse. The Bone Warden, on floor sixty. Win, and I'll sing your ballad \
              in every tavern in the land.",
        thanks: "The ballad is complete! Fourteen verses, three key changes and one very long \
                 note. You're a legend, friend. Here - wishes, and a heart.",
        goal: Guardian(60),
        reward: &[
            Coins(30000),
            item(Item::WishStar, 2),
            item(Item::HeartCrystal, 1),
        ],
        after: "bramble_ghosts",
        hearts: 6,
        ..Q
    },
    // ---------------------------------------------------------------- Toby
    // Seasonal: only asked in its season.
    QuestDef {
        key: "toby_frostberries",
        giver: V::Toby,
        title: "Frosty Rounds",
        ask: "Winter rounds are cold ones. A pocketful of frostberries keeps me going, somehow. \
              Eight of them?",
        thanks: "Sweet in the cold, just like the seed packet says! Here - sorbet. Yes, in winter. \
                 Trust me.",
        goal: Bring(Item::Frostberry, 8),
        reward: &[Coins(1700), item(Item::FrostberrySorbet, 3)],
        day: 2,
        season: Some(Season::Winter),
        ..Q
    },
    QuestDef {
        key: "toby_mailbag",
        giver: V::Toby,
        title: "The Lost Mailbag",
        ask: "I dropped my mailbag down the Hollow! Don't ask how. Floors 3 to 6, I think. The \
              whole town's post is in there!",
        thanks: "My bag! And look - a letter for you! Well, there was. Here are some feathers \
                 instead, and slippers. Frog slippers!",
        goal: Find(Item::Mailbag, 3, 6),
        reward: &[
            Coins(700),
            item(Item::Feather, 2),
            gear(Item::FrogSlippers, UNCOMMON),
        ],
        ..Q
    },
    QuestDef {
        key: "toby_letter",
        giver: V::Toby,
        title: "Letter for the Mayor",
        ask: "Could you run this letter up to the Mayor at the Town Hall? My feet are killing me.",
        thanks: "A letter from the Capital? About our town? Oh my. Oh MY. Thank you, dear - and \
                 tell Toby I said thank you too.",
        goal: Deliver(Item::Letter, V::Thistle),
        reward: &[
            Coins(400),
            Friend(V::Thistle, 60),
            Friend(V::Toby, 40),
            item(Item::FreshBread, 3),
        ],
        after: "toby_mailbag",
        ..Q
    },
    QuestDef {
        key: "toby_tomatoes",
        giver: V::Toby,
        title: "Toby's Sandwiches",
        ask: "I'm on my feet all day and I live on tomato sandwiches. Ten tomatoes?",
        thanks: "A week of sandwiches! Here - a frog raincoat, just like mine. Rain can't stop us!",
        goal: Bring(Item::Tomato, 10),
        reward: &[Coins(2000), gear(Item::FrogRaincoat, RARE)],
        after: "toby_letter",
        ..Q
    },
    QuestDef {
        key: "toby_route",
        giver: V::Toby,
        title: "The Long Route",
        ask: "Could you say hello to the folks at the far ends of my route for me? Mira, Juniper, \
              Sir Clank, Olive and Bramble. They get lonely.",
        thanks: "Everybody's so cheerful today! That was you! Frost mint tea, my favourite.",
        goal: Meet(meet(&[V::Mira, V::Juniper, V::Clank, V::Olive, V::Bramble])),
        reward: &[Coins(1500), item(Item::MintTea, 2)],
        after: "toby_tomatoes",
        ..Q
    },
    QuestDef {
        key: "toby_final",
        giver: V::Toby,
        title: "Toby's Big Day",
        ask: "I carry the shipping bins too, you know! Ship twenty thousand coppers' worth of \
              goods and I'll get a promotion!",
        thanks: "I'm Head Postman now! Head Postman! The frog can is yours - it croaks when it's \
                 empty. Best can ever.",
        goal: Ship(20000),
        reward: &[Coins(8000), gear(Item::FrogCan, EPIC)],
        after: "toby_route",
        hearts: 4,
        ..Q
    },
    // ---------------------------------------------------------------- Sir Clank
    QuestDef {
        key: "clank_badge",
        giver: V::Clank,
        title: "The Old Badge",
        ask: "I lost my Frost Guard badge on a patrol, floors 8 to 12. A knight without his badge \
              is just a man in a very heavy coat.",
        thanks: "My badge! I feel ten years younger. Nine. Well, eight. An iron helm, for a \
                 brave squire.",
        goal: Find(Item::KnightBadge, 8, 12),
        reward: &[Coins(1500), gear(Item::IronHelm, RARE)],
        depth: 8,
        ..Q
    },
    QuestDef {
        key: "clank_bones",
        giver: V::Clank,
        title: "Bone Collector",
        ask: "The Guard used old bones for training dummies. Twenty-five bones, squire!",
        thanks: "Excellent! My dummies stand tall again. A bone sabre for you - practise hard.",
        goal: Bring(Item::Bone, 25),
        reward: &[Coins(2000), gear(Item::BoneSabre, RARE)],
        after: "clank_badge",
        ..Q
    },
    QuestDef {
        key: "clank_frost",
        giver: V::Clank,
        title: "The Frost Guard",
        ask: "Reach the Frost Caverns - floor forty-one - and walk where the Guard once walked.",
        thanks: "You've seen the ice. Now you're one of us. The Frost Guard helm is yours.",
        goal: Reach(41),
        reward: &[Coins(6000), gear(Item::FrostHelm, EPIC)],
        after: "clank_bones",
        ..Q
    },
    QuestDef {
        key: "clank_core",
        giver: V::Clank,
        title: "The Colossus Core",
        ask: "The Frost Colossus has a core of pure ice. Bring it to me, and my old squad can rest. \
              The beast will wait on floor fifty while you look.",
        thanks: "It's over. Truly over. Thank you, squire. My old boots - Frost Walkers. Never \
                 slip again.",
        goal: Gather(Item::FrostCore, 1, Source::Boss(Foe::Golem)),
        reward: &[Coins(15000), gear(Item::FrostWalkers, LEGEND)],
        after: "clank_frost",
        depth: 50,
        ..Q
    },
    QuestDef {
        key: "clank_squire",
        giver: V::Clank,
        title: "A Knight's Duty",
        ask: "One last patrol. Forty skeletons in the Sunken Ruins, floor fifty-one and below. \
              Then you'll be a true knight.",
        thanks: "Kneel, squire. ...Rise, Sir Farmer of the Frost Guard! My sword is yours. And \
                 two heart crystals, from the Guard's vault.",
        goal: Slay(Some(Foe::Skeleton), 40, 51),
        reward: &[
            Coins(20000),
            item(Item::HeartCrystal, 2),
            gear(Item::FrostFang, LEGEND),
        ],
        after: "clank_core",
        hearts: 5,
        ..Q
    },
    // ---------------------------------------------------------------- Mira
    QuestDef {
        key: "mira_star",
        giver: V::Mira,
        title: "A Falling Star",
        ask: "Stars fall into the Hollow, you know. The creatures below floor twelve pick up the \
              shards. Three would mean the world to me.",
        thanks: "They're warm... like holding a tiny sky. A wish star, for you.",
        goal: Gather(Item::StarShard, 3, Source::Deep(12)),
        reward: &[Coins(1500), item(Item::WishStar, 1)],
        depth: 12,
        ..Q
    },
    QuestDef {
        key: "mira_crystal",
        giver: V::Mira,
        title: "Starlight Lens",
        ask: "My telescope needs a new lens. Eight glimmer shards should do it.",
        thanks: "I can see the rings of the far planets! Starry leggings, for walking under the sky.",
        goal: Bring(Item::Crystal, 8),
        reward: &[Coins(2500), gear(Item::StarryLeggings, RARE)],
        after: "mira_star",
        ..Q
    },
    QuestDef {
        key: "mira_moonbloom",
        giver: V::Mira,
        title: "Moonbloom Garden",
        ask: "Moonblooms glow at night. Five, for my rooftop garden? It'll be like sleeping among \
              stars.",
        thanks: "My roof glows! A starry robe, for you. Made from a sky just like this one.",
        goal: Bring(Item::Moonbloom, 5),
        reward: &[Coins(6000), gear(Item::StarRobe, EPIC)],
        after: "mira_crystal",
        ..Q
    },
    QuestDef {
        key: "mira_diamond",
        giver: V::Mira,
        title: "A Star Diamond",
        ask: "A star diamond is a star that decided to stay. I'd love to hold one, just once.",
        thanks: "It's singing... can you hear it? Two wish stars. Wish for something wonderful.",
        goal: Bring(Item::StarDiamond, 1),
        reward: &[Coins(10000), item(Item::WishStar, 2)],
        after: "mira_moonbloom",
        depth: 30,
        ..Q
    },
    QuestDef {
        key: "mira_constellation",
        giver: V::Mira,
        title: "The Farmer's Constellation",
        ask: "I want to make a new constellation. From fallen stars. Twelve shards, from floor \
              forty and below. And I'll name it after you.",
        thanks: "Look up tonight. There - the Farmer, with a little sprout on top. It's yours \
                 forever. And so is this wand.",
        goal: Gather(Item::StarShard, 12, Source::Deep(40)),
        reward: &[
            Coins(25000),
            gear(Item::MoonpetalWand, LEGEND),
            item(Item::WishStar, 2),
        ],
        after: "mira_diamond",
        depth: 40,
        hearts: 6,
        ..Q
    },
    // ---------------------------------------------------------------- Olive
    // Seasonal: only asked in its season.
    QuestDef {
        key: "olive_grapes",
        giver: V::Olive,
        title: "The Grape Stomp",
        ask: "Every autumn my family stomped grapes barefoot in a big wooden tub. Messy, sticky, \
              wonderful! Nine bunches and we'll start again.",
        thanks: "Purple feet and all! Only juice this year, but next year... Here, try it. And plant \
                 these - chestnuts for roasting.",
        goal: Bring(Item::Grapes, 9),
        reward: &[
            Coins(2400),
            item(Item::GrapeJuice, 4),
            item(Item::ChestnutSapling, 4),
        ],
        day: 2,
        season: Some(Season::Autumn),
        ..Q
    },
    QuestDef {
        key: "olive_sunflowers",
        giver: V::Olive,
        title: "Flowers for the Plaza",
        ask: "The plaza's flower beds have been bare for years. Six sunflowers would bring them \
              back to life. What do you say, fellow farmer?",
        thanks: "Look at them nodding in the sun! The whole plaza's brighter. Rose seeds, for \
                 your own beds.",
        goal: Bring(Item::Sunflower, 6),
        reward: &[
            Coins(1200),
            Restore(town::GARDENS),
            item(Item::RoseSeeds, 10),
        ],
        day: 2,
        ..Q
    },
    QuestDef {
        key: "olive_cabbage",
        giver: V::Olive,
        title: "Cabbage Patch",
        ask: "My own cabbages got eaten by slimes. Ten, to tide me over?",
        thanks: "Crunchy! My old straw hat, for you. It's seen a lot of sun.",
        goal: Bring(Item::Cabbage, 10),
        reward: &[Coins(2500), gear(Item::StrawHat, RARE)],
        after: "olive_sunflowers",
        ..Q
    },
    QuestDef {
        key: "olive_melons",
        giver: V::Olive,
        title: "Moss Melons",
        ask: "Moss melons for the town picnic! Five big ones.",
        thanks: "Picnic saved! Four quality sprinklers - save your back, farmer.",
        goal: Bring(Item::MossMelon, 5),
        reward: &[Coins(4000), item(Item::QualitySprinkler, 4)],
        after: "olive_cabbage",
        ..Q
    },
    QuestDef {
        key: "olive_harvest",
        giver: V::Olive,
        title: "A Bumper Harvest",
        ask: "Let's see what you've really got. Harvest three hundred crops!",
        thanks: "Three hundred! You're a machine! A golden sickle and crystal sprinklers. Go \
                 grow more!",
        goal: Harvest(None, 300),
        reward: &[
            Coins(10000),
            gear(Item::GoldSickle, EPIC),
            item(Item::CrystalSprinkler, 4),
        ],
        after: "olive_melons",
        hearts: 4,
        ..Q
    },
    QuestDef {
        key: "olive_ship",
        giver: V::Olive,
        title: "Farmer of the Year",
        ask: "The valley gives a prize for the farmer of the year. Ship a hundred thousand \
              coppers' worth of goods and I'll nominate you myself.",
        thanks: "FARMER OF THE YEAR! The whole town voted! The Ember Hoe - the valley's grand \
                 prize. And sun stones, for all that hard work.",
        goal: Ship(100000),
        reward: &[
            Coins(25000),
            item(Item::SunStone, 2),
            gear(Item::EmberHoe, LEGEND),
        ],
        after: "olive_harvest",
        hearts: 6,
        ..Q
    },
    // ---------------------------------------------------------------- Hazel
    QuestDef {
        key: "hazel_spark",
        giver: V::Hazel,
        title: "A Spark to Start",
        ask: "Magic is like a muscle - it wants stretching! Cast ten spells, anywhere at all. \
              The Hollow's best, but the farm's fine too.",
        thanks: "I could feel it from here, all fizzy! You're a natural. Take these - and a \
                 second spell, so your other hand isn't bored.",
        goal: Cast(10),
        reward: &[
            Coins(600),
            item(Item::SmallManaPotion, 6),
            Reward::Spell(super::spells::Spell::Mend),
        ],
        ..Q
    },
    QuestDef {
        key: "hazel_herbs",
        giver: V::Hazel,
        title: "Heartleaf Harvest",
        ask: "I'm out of heartleaf! It grows in the wild bushes that spring up on farms - \
              cut them back and you'll find it. Weeds hide some too. Eight sprigs, please.",
        thanks: "Oh, they're lovely and fresh! Here - vials, a few potions, and my own recipe \
                 book's worth of advice: two smalls brew into a medium!",
        goal: Bring(Item::Heartleaf, 8),
        reward: &[
            Coins(900),
            item(Item::Vial, 12),
            item(Item::HealthPotion, 3),
            Friend(V::Hazel, 60),
        ],
        after: "hazel_spark",
        ..Q
    },
    QuestDef {
        key: "hazel_brew",
        giver: V::Hazel,
        title: "Bubble and Brew",
        ask: "Every spellwright should know their way round a potion. Brew six - any kind, \
              any size. The crafting book has a Potions page.",
        thanks: "Six potions, and not one exploded! My proudest moment. Have some of my big \
                 bottles, and Bloom - the spell every farmer wishes they'd learned sooner.",
        goal: Brew(6),
        reward: &[
            Coins(1500),
            item(Item::LargeHealthPotion, 2),
            item(Item::LargeManaPotion, 2),
            Reward::Spell(super::spells::Spell::Bloom),
        ],
        after: "hazel_herbs",
        ..Q
    },
    QuestDef {
        key: "hazel_practice",
        giver: V::Hazel,
        title: "Practice Makes Perfect",
        ask: "A spell grows with you: stronger every level, and cheaper to cast. Train any \
              one of yours up to level four and I'll show you something special.",
        thanks: "Level four! Listen to it hum. Here's a wish star to deepen your mana, and a \
                 spell for getting out of trouble in a hurry.",
        goal: Mastery(4),
        reward: &[
            Coins(3000),
            item(Item::WishStar, 1),
            Reward::Spell(super::spells::Spell::Blink),
        ],
        after: "hazel_brew",
        ..Q
    },
    QuestDef {
        key: "hazel_wisps",
        giver: V::Hazel,
        title: "Bottled Starlight",
        ask: "Wisp dust makes the finest mana potions in the valley. The wisps drift about \
              the Crystal Grotto. Twelve pinches?",
        thanks: "Glittering! I'll brew something wonderful. And for you: Chain Spark. \
                 Lightning that doesn't know when to stop.",
        goal: Bring(Item::WispDust, 12),
        reward: &[
            Coins(3500),
            item(Item::ManaPotion, 5),
            Reward::Spell(super::spells::Spell::ChainSpark),
        ],
        after: "hazel_practice",
        depth: 11,
        ..Q
    },
    QuestDef {
        key: "hazel_ghosts",
        giver: V::Hazel,
        title: "Something in the Walls",
        ask: "Ghosts drift through walls down in the Fungal Hollow and below, and they've \
              been curdling my potions from afar. Could you calm ten of them?",
        thanks: "My cauldron's stopped sulking! Take Ward - a bubble of light for the scary \
                 bits. It's saved my life more than once.",
        goal: Slay(Some(Foe::Ghost), 10, 1),
        reward: &[
            Coins(6000),
            item(Item::LargeHealthPotion, 3),
            Reward::Spell(super::spells::Spell::Ward),
        ],
        after: "hazel_wisps",
        depth: 20,
        ..Q
    },
    QuestDef {
        key: "hazel_brewmaster",
        giver: V::Hazel,
        title: "Brewmaster",
        ask: "You've the hands for it now. Brew forty potions and I'll hang your name on \
              the Spellery wall, right next to mine.",
        thanks: "Forty! The whole town smells of blueberries. Here's my finest wand - it \
                 has chosen you, I think.",
        goal: Brew(40),
        reward: &[
            Coins(9000),
            gear(Item::MoonpetalWand, EPIC),
            item(Item::LargeManaPotion, 5),
        ],
        after: "hazel_ghosts",
        hearts: 4,
        ..Q
    },
    QuestDef {
        key: "hazel_stars",
        giver: V::Hazel,
        title: "Where the Stars Fall",
        ask: "They say that past floor thirty the Hollow is so deep it touches the sky from \
              the other side. Go and see, and I'll teach you to call the stars down.",
        thanks: "You've seen it! Then you're ready. Starfall - my greatest spell. Use it well.",
        goal: Reach(30),
        reward: &[
            Coins(8000),
            Reward::Spell(super::spells::Spell::Starfall),
            item(Item::WishStar, 1),
        ],
        after: "hazel_ghosts",
        ..Q
    },
    QuestDef {
        key: "hazel_frost",
        giver: V::Hazel,
        title: "A Cold Snap",
        ask: "Frost opals from the Frost Caverns keep my potions fresh. Five, if you can \
              brave the cold. You'll want Frost Nova down there - I'll teach it now.",
        thanks: "Brr, they're perfect. Keep practising Frost Nova - at level five it freezes \
                 things solid!",
        goal: Bring(Item::FrostGem, 5),
        reward: &[
            Coins(7000),
            Reward::Spell(super::spells::Spell::FrostNova),
            item(Item::LargeEnergyPotion, 3),
        ],
        after: "hazel_wisps",
        depth: 40,
        ..Q
    },
    QuestDef {
        key: "hazel_archmage",
        giver: V::Hazel,
        title: "Archmage of Bramblewick",
        ask: "There's one thing left to learn, and nobody can teach it: mastery. Take one \
              spell all the way to level ten.",
        thanks: "Level ten. I've only ever seen that once before - in a mirror. Archmage, \
                 this is yours.",
        goal: Mastery(10),
        reward: &[
            Coins(25000),
            gear(Item::StarWand, LEGEND),
            item(Item::WishStar, 2),
            Friend(V::Hazel, 200),
        ],
        after: "hazel_practice",
        hearts: 6,
        ..Q
    },
    // ---------------------------------------------------------------- Fishing with Garrick
    QuestDef {
        key: "garrick_fishing",
        giver: V::Garrick,
        title: "Quiet Waters",
        ask: "...You fish? Should. Quiet work. Grab a bamboo rod off my rack - cheap - and \
              catch five fish. Any kind. The pond on your farm's full of them. Hold to wind \
              up, let go to cast. When it bites, strike.",
        thanks: "Five. Good. You've got the patience. Here - a willow rod. Bends better. And \
                 bait. Fish can't resist a wriggle.",
        goal: Fish(5),
        reward: &[
            Coins(400),
            gear(Item::WillowRod, UNCOMMON),
            item(Item::Bait, 10),
            Friend(V::Garrick, 40),
        ],
        ..Q
    },
    QuestDef {
        key: "garrick_koi",
        giver: V::Garrick,
        title: "The Pond's Jewel",
        ask: "There's a koi in your pond. White and orange, naps under the lilies. Only \
              comes up by day. Bring me one. I want to paint it. ...Don't tell anyone I paint.",
        thanks: "Look at those colours. ...Right. Painting. Here, take this before I say \
                 something soppy.",
        goal: Bring(Item::LilyKoi, 1),
        reward: &[Coins(1200), item(Item::Bait, 15), item(Item::TrophyFish, 1)],
        after: "garrick_fishing",
        ..Q
    },
    QuestDef {
        key: "garrick_river",
        giver: V::Garrick,
        title: "Upstream",
        ask: "The stream by the park has trout, bass, chub, loach. Different water, different \
              fish. Catch eight different kinds and you'll know what I mean.",
        thanks: "Eight kinds. You're learning the water. This oak rod's never snapped. \
                 Take it.",
        goal: Fishdex(8),
        reward: &[
            Coins(1500),
            gear(Item::OakRod, RARE),
            Friend(V::Garrick, 40),
        ],
        after: "garrick_koi",
        ..Q
    },
    QuestDef {
        key: "garrick_deep",
        giver: V::Garrick,
        title: "Fish in the Dark",
        ask: "Delvers talk about ponds down in the Hollow. Eels that glow. Frogs that guard \
              them. Bring me a glowworm eel and I'll believe it.",
        thanks: "It glows. It actually glows. ...The Hollow's full of surprises. So are you. \
                 Here: a crystal rod. Found it in a frog's hoard. Long story.",
        goal: Bring(Item::GlowwormEel, 1),
        reward: &[
            Coins(2500),
            gear(Item::CrystalRod, RARE),
            item(Item::Bait, 20),
        ],
        after: "garrick_river",
        depth: 5,
        ..Q
    },
    QuestDef {
        key: "garrick_golden",
        giver: V::Garrick,
        title: "The Golden Carp",
        ask: "Your pond has an old carp. Golden. Older than the town. I hooked it once, forty \
              years ago, and let it go. Catch it. Show me it's still there.",
        thanks: "...Still there. Still golden. Thank you. This rod was meant for a fish like \
                 that. It's yours now.",
        goal: Bring(Item::GoldenCarp, 1),
        reward: &[
            Coins(8000),
            gear(Item::LeviathanRod, LEGEND),
            item(Item::WishStar, 1),
        ],
        after: "garrick_river",
        hearts: 4,
        ..Q
    },
    QuestDef {
        key: "garrick_lava",
        giver: V::Garrick,
        title: "Fire Fishing",
        ask: "They say eels swim in the lava down in the Ember Depths. You'd need a rod \
              that won't burn. The frogs and crabs down there hoard them. Bring me two lava \
              eels.",
        thanks: "Hot. Very hot. ...Ow. Worth it. Here - you've earned this.",
        goal: Bring(Item::LavaEel, 2),
        reward: &[
            Coins(6000),
            Scroll(Group::Tool, EPIC),
            item(Item::EelKebab, 3),
            item(Item::Fireplace, 1),
        ],
        after: "garrick_deep",
        depth: 31,
        ..Q
    },
    QuestDef {
        key: "garrick_master",
        giver: V::Garrick,
        title: "Master Angler",
        ask: "Thirty kinds of fish. Pond, stream, every pool in the Hollow, even the lava. \
              Do that and there's nothing left I can teach you.",
        thanks: "Thirty. You're a master angler now. Better than me. ...Don't let it go to \
                 your head. This is the finest rod I ever saw. Take it.",
        goal: Fishdex(30),
        reward: &[
            Coins(20000),
            gear(Item::StarRod, LEGEND),
            item(Item::HeartCrystal, 1),
            Friend(V::Garrick, 100),
        ],
        after: "garrick_golden",
        hearts: 6,
        ..Q
    },
    // ---------------------------------------------------------------- Cooking at home
    QuestDef {
        key: "barley_fishfry",
        giver: V::Barley,
        title: "Fish Fry Friday",
        ask: "Friday's fish fry night at the Snail, and my supplier's gone fishing - \
              without me! Could you catch three pond perch?",
        thanks: "Perch! Proper perch! The regulars will cry. Have some fish and chips on \
                 the house.",
        goal: Bring(Item::PondPerch, 3),
        reward: &[
            Coins(600),
            item(Item::FishAndChips, 3),
            Friend(V::Barley, 40),
        ],
        ..Q
    },
    QuestDef {
        key: "barley_stove",
        giver: V::Barley,
        title: "Chef at Home",
        ask: "A farmer who cooks is a farmer who thrives! Cook ten dishes on your own stove \
              at home and I'll send you something special for that kitchen of yours.",
        thanks: "Ten dishes! You're a proper chef now. I've had a copper range sitting in my \
                 cellar - it's yours. Sometimes it cooks a second helping, if it likes you.",
        goal: Cook(10),
        reward: &[
            Coins(1000),
            item(Item::CopperRange, 1),
            Friend(V::Barley, 60),
        ],
        after: "barley_fishfry",
        ..Q
    },
    QuestDef {
        key: "mabel_oven",
        giver: V::Mabel,
        title: "A Baker's Kitchen",
        ask: "Every good kitchen starts with a warm stove, dearie. Cook five dishes on yours \
              and I'll give you a little something for the counter.",
        thanks: "Five dishes! You've got flour on your nose - that's how I know it's true. \
                 Here, for your kitchen.",
        goal: Cook(5),
        reward: &[
            Coins(500),
            item(Item::KitchenCounter, 2),
            item(Item::FlowerVase, 1),
            Friend(V::Mabel, 40),
        ],
        ..Q
    },
    QuestDef {
        key: "fern_stew",
        giver: V::Fern,
        title: "Grandma's Stew",
        ask: "My late husband made the best fisherman's stew. Bluegill, tomato, cabbage. \
              Would you cook two bowls for an old lady? I'd give anything to taste it again.",
        thanks: "Oh... it tastes just like his. Thank you, dear. Take his old clock - it \
                 always kept better time in a happy house.",
        goal: Bring(Item::FishStew, 2),
        reward: &[
            Coins(800),
            item(Item::GrandfatherClock, 1),
            Friend(V::Fern, 80),
        ],
        ..Q
    },
    // ---------------------------------------------------------------- Home, sweet home
    QuestDef {
        key: "wren_home",
        giver: V::Wren,
        title: "Make It Cozy",
        ask: "Have you looked inside your farmhouse lately? Let's make it cozy! Hold a piece \
              of furniture and use it inside to set it down - T turns it. Get your home to \
              20 charisma and come and tell me.",
        thanks: "Twenty! I can feel the coziness from here. Here's an armchair for reading \
                 in, and a rug to put under it.",
        goal: Charm(20),
        reward: &[
            Coins(600),
            item(Item::Armchair, 1),
            item(Item::StripedRug, 1),
            Friend(V::Wren, 40),
        ],
        ..Q
    },
    QuestDef {
        key: "wren_furnish",
        giver: V::Wren,
        title: "Room to Breathe",
        ask: "A house needs things in it to feel lived in. Fourteen pieces of furniture - \
              tables, chairs, lamps, plants, anything!",
        thanks: "Fourteen pieces and it still has room to breathe? You've got an eye for this. \
                 A bookshelf and a lamp, for your reading corner.",
        goal: Furnish(14),
        reward: &[
            Coins(1000),
            item(Item::Bookshelf, 1),
            item(Item::FloorLamp, 1),
        ],
        after: "wren_home",
        ..Q
    },
    QuestDef {
        key: "wren_charming",
        giver: V::Wren,
        title: "A Charming Home",
        ask: "Everyone's talking about your house! Let's make them talk more. Reach 30 \
              charisma - new wallpaper and floors help, and so do fish in a tank.",
        thanks: "Thirty! The whole town wants an invitation now. Here's a sofa big enough \
                 for all of them, and a starry rug.",
        goal: Charm(30),
        reward: &[
            Coins(3000),
            item(Item::Sofa, 1),
            item(Item::StarRug, 1),
            Friend(V::Wren, 60),
        ],
        after: "wren_furnish",
        charm: 15,
        ..Q
    },
    QuestDef {
        key: "wren_dazzling",
        giver: V::Wren,
        title: "Dazzling!",
        ask: "I've never said this to anyone: I think your home could be the finest in the \
              valley. Sixty charisma. Dazzle me.",
        thanks: "Dazzling. Truly. I've been saving these for a home like yours - my very best \
                 bed, and a piano. Play me something sometime?",
        goal: Charm(60),
        reward: &[
            Coins(10000),
            item(Item::CanopyBed, 1),
            item(Item::Piano, 1),
            Friend(V::Wren, 100),
        ],
        after: "wren_charming",
        charm: 40,
        ..Q
    },
    QuestDef {
        key: "thistle_home",
        giver: V::Thistle,
        title: "Home of the Year",
        ask: "Every year Bramblewick awards the Golden Doormat to the loveliest home in the \
              valley. Word is yours might win! Get it to 65 charisma before the judges visit.",
        thanks: "The Golden Doormat goes to... you! Hollowbloom Farm, home of the year! The \
                 whole council agreed - well, Clank abstained, he prefers castles.",
        goal: Charm(65),
        reward: &[
            Coins(20000),
            item(Item::HeartCrystal, 2),
            item(Item::WishStar, 1),
            Friend(V::Thistle, 100),
        ],
        after: "thistle_hello",
        charm: 45,
        ..Q
    },
    QuestDef {
        key: "pip_fishbowl",
        giver: V::Pip,
        title: "Fishy Friends",
        ask: "Do you have fish? Real ones? In a tank? Can you get three? I want to name \
              them. I've already picked names. They're all called Captain.",
        thanks: "Captain, Captain and Captain! They're perfect! Here's my duck and my boat - \
                 your fish need toys too!",
        goal: Tank(3),
        reward: &[
            Coins(500),
            item(Item::RubberDuck, 1),
            item(Item::ToyBoat, 1),
            Friend(V::Pip, 60),
        ],
        ..Q
    },
    QuestDef {
        key: "pip_crayfish",
        giver: V::Pip,
        title: "Pinchy!",
        ask: "My friend says crayfish can pinch through a boot. I need to see. For science. \
              Can you catch three?",
        thanks: "They DO pinch! Ow! Science is amazing. Here, this is my best marble.",
        goal: Bring(Item::Crayfish, 3),
        reward: &[
            Coins(400),
            item(Item::GlassMarble, 1),
            item(Item::CrayfishBoil, 2),
        ],
        after: "pip_fishbowl",
        ..Q
    },
    QuestDef {
        key: "mira_night",
        giver: V::Mira,
        title: "Night Fishing",
        ask: "There's a catfish that only rises at night to look at the moon, like me. Would \
              you catch one from your pond? I'd love to meet a fellow stargazer.",
        thanks: "Hello, friend. Look at its purple whiskers! Here - take my old telescope. \
                 On a clear night you might catch a shooting star.",
        goal: Bring(Item::MoonlitCatfish, 1),
        reward: &[Coins(1200), item(Item::Telescope, 1), Friend(V::Mira, 60)],
        ..Q
    },
    QuestDef {
        key: "mira_aurora",
        giver: V::Mira,
        title: "Northern Lights",
        ask: "Deep in the Frost Caverns, they say, a trout carries the northern lights on its \
              scales - but only after dark. I'd give my best painting to see one.",
        thanks: "The whole sky, on one little fish... Thank you. The painting is yours, as \
                 promised.",
        goal: Bring(Item::AuroraTrout, 1),
        reward: &[
            Coins(5000),
            item(Item::StarryPainting, 1),
            item(Item::Moonstone, 2),
        ],
        after: "mira_night",
        depth: 41,
        ..Q
    },
    QuestDef {
        key: "toby_storm",
        giver: V::Toby,
        title: "Storm Chaser",
        ask: "Rainy days are the worst for a postman. Except! Storm salmon leap up the stream \
              when it pours. Catch me one on a wet day and I'll love the rain forever.",
        thanks: "A storm salmon! I'll never grumble about rain again. Probably. Here, a \
                 raincoat - I got two.",
        goal: Bring(Item::StormSalmon, 1),
        reward: &[
            Coins(1500),
            gear(Item::FrogRaincoat, RARE),
            Friend(V::Toby, 60),
        ],
        ..Q
    },
    QuestDef {
        key: "clank_pike",
        giver: V::Clank,
        title: "The Frozen Lake",
        ask: "In my Frost Guard days we fished the frozen lakes for ice pike. Sharp as \
              swords! Bring me one, and I'll give you the rod I carried then.",
        thanks: "An ice pike! Ha! The Frost Guard would be proud. The rod is yours, friend.",
        goal: Bring(Item::IcePike, 1),
        reward: &[
            Coins(4000),
            gear(Item::FrostRod, EPIC),
            Friend(V::Clank, 60),
        ],
        depth: 41,
        ..Q
    },
    QuestDef {
        key: "opal_ray",
        giver: V::Opal,
        title: "A Living Diamond",
        ask: "Darling, I hear there's a ray in the Crystal Grotto that glitters like a cut \
              diamond. Only the most charming delvers ever land one. Bring it to me?",
        thanks: "It's exquisite. Look at the facets! For you - a real star diamond, and \
                 candlelight to see your home sparkle by.",
        goal: Bring(Item::DiamondRay, 1),
        reward: &[
            Coins(3000),
            item(Item::StarDiamond, 1),
            item(Item::Candelabra, 1),
            Friend(V::Opal, 60),
        ],
        depth: 11,
        charm: 25,
        ..Q
    },
    QuestDef {
        key: "hazel_scales",
        giver: V::Hazel,
        title: "Ghostly Scales",
        ask: "Ghost fish swim in the Sunken Ruins' pools. Their scales make the finest ink \
              for spellbooks. Two, please - and mind the puffers.",
        thanks: "Perfect. See how they shimmer? A wish star and some potions for your trouble.",
        goal: Bring(Item::GhostFish, 2),
        reward: &[
            Coins(3500),
            item(Item::WishStar, 1),
            item(Item::LargeManaPotion, 3),
        ],
        depth: 51,
        ..Q
    },
    QuestDef {
        key: "juniper_seaweed",
        giver: V::Juniper,
        title: "Pond Weeds",
        ask: "Seaweed is a marvellous fertiliser. Anglers pull it up all the time! Ten \
              strands, if you can spare them.",
        thanks: "Lovely and slimy. Here, some plants for your house - they'll love you back.",
        goal: Bring(Item::Seaweed, 10),
        reward: &[
            Coins(500),
            item(Item::PottedFern, 2),
            item(Item::PottedCactus, 1),
        ],
        ..Q
    },
    QuestDef {
        key: "rowan_frogs",
        giver: V::Rowan,
        title: "Frog Trouble",
        ask: "Bog frogs by the underground ponds are spitting at delvers. Harmless, mostly. \
              Humiliating, definitely. Clear out ten.",
        thanks: "Ten frogs, no more spit. The Guild thanks you. They hoard bait, too - keep it.",
        goal: Slay(Some(Foe::Frog), 10, 1),
        reward: &[Coins(1200), Marks(30), item(Item::Bait, 20)],
        after: "rowan_slimes",
        ..Q
    },
    QuestDef {
        key: "rowan_puffers",
        giver: V::Rowan,
        title: "Spiky Business",
        ask: "Puffers in the Grotto blow up into spiky balls and fire needles everywhere. \
              Eight of them, floor 11 or deeper. Keep your distance when they swell.",
        thanks: "Eight puffers deflated. Here's your pay - and a pearl rod we pulled out of \
                 one of their nests.",
        goal: Slay(Some(Foe::Puffer), 8, 11),
        reward: &[Coins(2500), Marks(40), gear(Item::PearlRod, RARE)],
        after: "rowan_frogs",
        depth: 11,
        ..Q
    },
    QuestDef {
        key: "olive_party",
        giver: V::Olive,
        title: "Garden Party",
        ask: "I'm throwing a garden party, and your home is so lovely everyone wants it \
              decorated with your roses. Five roses?",
        thanks: "Beautiful! The party was a triumph. Take these vases - fill them with \
                 something pretty.",
        goal: Bring(Item::Rose, 5),
        reward: &[Coins(1500), item(Item::FlowerVase, 2), Friend(V::Olive, 60)],
        charm: 20,
        ..Q
    },
    QuestDef {
        key: "bramble_housewarming",
        giver: V::Bramble,
        title: "Housewarming",
        ask: "A home as charming as yours deserves a song and a feast! Bring three plates of \
              fish tacos and I'll write the ballad of Hollowbloom Farm.",
        thanks: "♪ Oh, the farm above the Hollow, where the tacos always flow... ♪ Needs \
                 work. Here, a globe and a music box for your parlour.",
        goal: Bring(Item::FishTacos, 3),
        reward: &[
            Coins(2000),
            item(Item::Globe, 1),
            item(Item::MusicBox, 1),
            Friend(V::Bramble, 60),
        ],
        charm: 25,
        ..Q
    },
    // The deeper Hollow's folk: lantern snails and book-worm bibliomancers.
    QuestDef {
        key: "rowan_snails",
        giver: V::Rowan,
        title: "Lanterns in the Dark",
        ask: "Delvers past floor ten keep seeing lights where no lamp should be. Snails, it \
              turns out, under shells of glowing glass. Slow, but they hide when you hit \
              them - be patient. Ten of them for the Guild.",
        thanks: "Ten lantern snails! And you kept a few panes of glass, I hope? Here's your \
                 pay, and a lamp of our own to light your way.",
        goal: Slay(Some(Foe::Snail), 10, 11),
        reward: &[Coins(3500), Marks(40), item(Item::StainedGlass, 3)],
        after: "rowan_slimes",
        depth: 11,
        ..Q
    },
    QuestDef {
        key: "quill_ink",
        giver: V::Quill,
        title: "Ink That Knows the Way",
        ask: "Caterpillars in spectacles, spitting ink that writes the way out of the Hollow! \
              I must study it. Six drops of their glow ink, if you'd be so kind.",
        thanks: "Remarkable - it's still trying to point somewhere! I've copied the old delvers' \
                 trick. Read one of these maps down there and the ink will show you the way.",
        goal: Bring(Item::GlowInk, 6),
        reward: &[Coins(3500), item(Item::InkMap, 4)],
        depth: 11,
        ..Q
    },
    QuestDef {
        key: "opal_glass",
        giver: V::Opal,
        title: "Glass That Glows",
        ask: "They say the snails below floor ten wear shells of stained glass that glow \
              without a flame. I have to see it. Eight panes, and I'll make you something \
              lovely.",
        thanks: "It's still warm with light! I set a whole shell of it on a stand for you - \
                 put it somewhere you'll see it every night.",
        goal: Bring(Item::StainedGlass, 8),
        reward: &[Coins(4500), item(Item::SnailLamp, 1)],
        depth: 11,
        ..Q
    },
    QuestDef {
        key: "mira_worms",
        giver: V::Mira,
        title: "Reading Between the Lines",
        ask: "Those bookish caterpillars below floor ten read the same book over and over, \
              and spit ink at anyone who interrupts. Rude! Interrupt eight of them for me.",
        thanks: "And they dropped their bookmarks! Well, scrolls. You keep them - I've read \
                 enough for one night.",
        goal: Slay(Some(Foe::Bookworm), 8, 11),
        reward: &[Coins(3000), item(Item::GlowInk, 4), item(Item::InkMap, 2)],
        depth: 11,
        ..Q
    },
    // The Hollow's newer residents: the walking dead, goblins fat and skinny, and bugs.
    QuestDef {
        key: "rowan_zombies",
        giver: V::Rowan,
        title: "Restless Dead",
        ask: "Zombies. Shambling about the Hollow with their arms out like they're looking \
              for a hug. Don't give them one. Put fifteen of them back to sleep.",
        thanks: "Fifteen fewer shufflers. Good work, delver - you've earned these.",
        goal: Slay(Some(Foe::Zombie), 15, 1),
        reward: &[Coins(1500), Marks(35), item(Item::HealthPotion, 3)],
        after: "rowan_slimes",
        ..Q
    },
    QuestDef {
        key: "clank_goblins",
        giver: V::Clank,
        title: "Goblin Brutes",
        ask: "In my day we'd face a goblin brute with nothing but a pot lid and a stern \
              look. Those big fellows swing a mean club. Ten of them, from floor 3 down - \
              and mind the shockwave when the club comes down!",
        thanks: "Ten brutes! Splendid! Have my old shield - it's seen off more goblins than \
                 I've had hot dinners.",
        goal: Slay(Some(Foe::Brute), 10, 3),
        reward: &[
            Coins(2400),
            gear(Item::IronShield, RARE),
            Friend(V::Clank, 40),
        ],
        depth: 3,
        ..Q
    },
    QuestDef {
        key: "pip_sneaks",
        giver: V::Pip,
        title: "Sneaky Goblins",
        ask: "The skinny goblins throw DAGGERS! And they run away! That's cheating! Beat \
              twelve of them for me. Pleeease?",
        thanks: "Twelve! You're the best adventurer EVER. Here, a feather - it floats you \
                 home if they get too sneaky.",
        goal: Slay(Some(Foe::Sneak), 12, 1),
        reward: &[Coins(1200), item(Item::Feather, 2)],
        after: "pip_slime",
        ..Q
    },
    QuestDef {
        key: "hilde_teeth",
        giver: V::Hilde,
        title: "Goblin Teeth",
        ask: "Goblin teeth make the finest rivets - hard as iron and they never rust. Bring \
              me twelve and I'll make it worth your while.",
        thanks: "Lovely set of teeth. Here, a helm riveted with the last lot.",
        goal: Bring(Item::GoblinTooth, 12),
        reward: &[Coins(1800), gear(Item::IronHelm, RARE)],
        depth: 3,
        ..Q
    },
    QuestDef {
        key: "olive_bugs",
        giver: V::Olive,
        title: "Bug Hunt",
        ask: "Bugs from the Hollow keep crawling up and nibbling my roses! Squash twenty \
              of them down there before they get any ideas.",
        thanks: "Twenty bugs! My roses thank you. Plant these - they're the hardy kind.",
        goal: Slay(Some(Foe::Bug), 20, 1),
        reward: &[Coins(1400), item(Item::RoseSeeds, 6), Friend(V::Olive, 40)],
        ..Q
    },
    QuestDef {
        key: "hazel_dust",
        giver: V::Hazel,
        title: "Grave Matters",
        ask: "Grave dust! A pinch keeps a spell from wandering off. Zombies carry it in \
              their pockets, don't ask me why. Ten pinches, please.",
        thanks: "Perfectly dusty. My spells will behave themselves now. Take these for your \
                 trouble.",
        goal: Bring(Item::GraveDust, 10),
        reward: &[Coins(1600), item(Item::ManaPotion, 3), Friend(V::Hazel, 40)],
        ..Q
    },
    QuestDef {
        key: "nix_chitin",
        giver: V::Nix,
        title: "Tough Plates",
        ask: "Bug chitin! Light as paper, tough as tin. Fifteen plates and I can finish my \
              clockwork beetle. It will definitely not bite anyone.",
        thanks: "It bit me. Worth it! Here's your pay.",
        goal: Bring(Item::Chitin, 15),
        reward: &[Coins(1500), Friend(V::Nix, 40)],
        ..Q
    },
];

pub fn quest_def(key: &str) -> Option<&'static QuestDef> {
    QUESTS.iter().find(|q| q.key == key)
}

/// Guild ranks: marks needed, the title, and what the promotion brings.
pub static RANKS: [(u32, &str, &[Reward]); 6] = [
    (0, "Copper Lantern", &[]),
    (60, "Bronze Lantern", &[Coins(1000), item(Item::Feather, 3)]),
    (
        200,
        "Silver Lantern",
        &[Coins(3000), Scroll(Group::Weapon, EPIC)],
    ),
    (
        500,
        "Gold Lantern",
        &[Coins(8000), item(Item::HeartCrystal, 1)],
    ),
    (
        1000,
        "Mithril Lantern",
        &[Coins(15000), gear(Item::CrystalAegis, LEGEND)],
    ),
    (
        2000,
        "Starlight Lantern",
        &[
            Coins(40000),
            item(Item::HeartCrystal, 2),
            item(Item::WishStar, 2),
        ],
    ),
];

// ------------------------------------------------------------------------------------------
// Requests on the boards
// ------------------------------------------------------------------------------------------

/// A notice on the town's request board or the guild's bounty board.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Request {
    /// Day and slot, so a notice is only ever taken once.
    pub id: u32,
    pub giver: Villager,
    pub title: String,
    pub text: String,
    pub goal: Goal,
    pub reward: Vec<Reward>,
    pub guild: bool,
}

/// Which quest an entry in the log is.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum QuestId {
    Story(String),
    Request(Request),
}

/// A quest you've taken on.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Quest {
    pub id: QuestId,
    /// Counter for creatures, harvests, sales and bindings.
    #[serde(default)]
    pub n: u64,
    /// Who you've said hello to (for Meet).
    #[serde(default)]
    pub met: u32,
    #[serde(default)]
    pub day: u32,
}

impl Quest {
    pub fn story(&self) -> Option<&'static QuestDef> {
        match &self.id {
            QuestId::Story(k) => quest_def(k),
            _ => None,
        }
    }

    pub fn request(&self) -> Option<&Request> {
        match &self.id {
            QuestId::Request(r) => Some(r),
            _ => None,
        }
    }

    pub fn giver(&self) -> Villager {
        self.story()
            .map(|d| d.giver)
            .or(self.request().map(|r| r.giver))
            .unwrap_or(Villager::Thistle)
    }

    pub fn title(&self) -> String {
        match &self.id {
            QuestId::Story(k) => quest_def(k).map_or(k.clone(), |d| d.title.to_string()),
            QuestId::Request(r) => r.title.clone(),
        }
    }

    pub fn goal(&self) -> Goal {
        self.story()
            .map(|d| d.goal)
            .or(self.request().map(|r| r.goal))
            .unwrap_or(Goal::Reach(1))
    }

    pub fn rewards(&self) -> Vec<Reward> {
        match &self.id {
            QuestId::Story(k) => quest_def(k).map_or(Vec::new(), |d| d.reward.to_vec()),
            QuestId::Request(r) => r.reward.clone(),
        }
    }

    pub fn key(&self) -> String {
        match &self.id {
            QuestId::Story(k) => k.clone(),
            QuestId::Request(r) => format!("request_{}", r.id),
        }
    }
}

/// What you've collected and seen, for the collection quests.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Journal {
    #[serde(default)]
    pub crops: Vec<Crop>,
    #[serde(default)]
    pub curios: Vec<Item>,
    #[serde(default)]
    pub dishes: Vec<Item>,
    #[serde(default)]
    pub foes: Vec<Foe>,
    #[serde(default)]
    pub guardians: Vec<u32>,
    /// Kinds of fish caught, and the biggest of each (in cm).
    #[serde(default)]
    pub fish: Vec<Item>,
    #[serde(default)]
    pub records: Vec<(Item, u16)>,
}

fn note<T: PartialEq>(v: &mut Vec<T>, x: T) -> bool {
    if v.contains(&x) {
        false
    } else {
        v.push(x);
        true
    }
}

/// What the goal asks for, in a few words.
pub fn goal_text(g: Goal) -> String {
    match g {
        Goal::Bring(i, n) => format!("Bring {n} {}", i.def().name),
        Goal::Find(i, lo, hi) => format!("Find {} (floors {lo}-{hi})", i.def().name),
        Goal::Gather(i, n, s) => {
            let from = match s {
                Source::Foe(f) => format!("from {}s", foe_name(f)),
                Source::Biome(b) => {
                    format!(
                        "in the {}",
                        crate::assets::BIOME_STYLES[b as usize % 6].name
                    )
                }
                Source::Deep(d) => format!("from floor {d} down"),
                Source::Boss(f) => format!("from {}", super::foes::boss_name(f)),
                Source::Eruption(d) => format!("where floor {d}'s guardian falls"),
            };
            format!("Collect {n} {} {from}", i.def().name)
        }
        Goal::Slay(f, n, d) => {
            let who = f.map_or("monsters".to_string(), |f| format!("{}s", foe_name(f)));
            if d > 1 {
                format!("Defeat {n} {who} (floor {d}+)")
            } else {
                format!("Defeat {n} {who}")
            }
        }
        Goal::Reach(d) => format!("Reach floor {d}"),
        Goal::Guardian(d) => format!("Defeat the guardian of floor {d}"),
        Goal::Harvest(c, n) => match c {
            Some(c) => format!("Harvest {n} {}", c.def().produce.def().name),
            None => format!("Harvest {n} crops"),
        },
        Goal::Ship(v) => format!("Ship {} of goods", loot::money_text(v)),
        Goal::Enchant(n) => format!("Bind {n} scroll{}", if n == 1 { "" } else { "s" }),
        Goal::Deliver(i, to) => format!("Take the {} to {}", i.def().name, to.name()),
        Goal::Meet(m) => format!("Say hello to {} people", m.count_ones()),
        Goal::Almanac(n) => format!("Harvest {n} different crops"),
        Goal::Curios(n) => format!("Find {n} different gems and relics"),
        Goal::Cookbook(n) => format!("Cook {n} different dishes"),
        Goal::Codex(n) => format!("Defeat {n} kinds of creature"),
        Goal::Cast(n) => format!("Cast {n} spells (Q and R)"),
        Goal::Brew(n) => format!("Brew {n} potions (crafting, Potions tab)"),
        Goal::Mastery(l) => format!("Train a spell to level {l}"),
        Goal::Fish(n) => format!("Catch {n} fish"),
        Goal::Fishdex(n) => format!("Catch {n} different kinds of fish"),
        Goal::Charm(n) => format!("Make your home {n} charisma"),
        Goal::Furnish(n) => format!("Furnish your house with {n} pieces"),
        Goal::Tank(n) => format!("Keep {n} fish in a tank at home"),
        Goal::Cook(n) => format!("Cook {n} dishes at your stove"),
    }
}

pub fn foe_name(f: Foe) -> &'static str {
    match f {
        Foe::Slime => "slime",
        Foe::Bat => "bat",
        Foe::Shroom => "shroomling",
        Foe::Crab => "crystal crab",
        Foe::Wisp => "wisp",
        Foe::Beetle => "beetle",
        Foe::Imp => "imp",
        Foe::Skeleton => "skeleton",
        Foe::Golem => "golem",
        Foe::Ghost => "ghost",
        Foe::Frog => "bog frog",
        Foe::Jelly => "drift jelly",
        Foe::Puffer => "puffer",
        Foe::Zombie => "zombie",
        Foe::Brute => "fat goblin",
        Foe::Sneak => "skinny goblin",
        Foe::Bug => "bug",
        Foe::Snail => "lantern snail",
        Foe::Bookworm => "book-worm",
    }
}

/// A reward, in words.
pub fn reward_text(r: &Reward) -> String {
    match *r {
        Reward::Coins(c) => loot::money_text(c),
        Reward::Item(i, n) if n > 1 => format!("{} x{n}", i.def().name),
        Reward::Item(i, _) => i.def().name.to_string(),
        Reward::Gear(i, r) => format!("{} {}", r.name(), i.def().name),
        Reward::Scroll(g, r) => format!(
            "{} {} Scroll",
            r.name(),
            Item::scroll(g).def().name.split(' ').next().unwrap_or("")
        ),
        Reward::Friend(v, n) => format!("{} ♥{:+}", v.name(), n),
        Reward::Restore(bit) => format!("Bramblewick: {}", project_name(bit)),
        Reward::Marks(n) => format!("{n} guild marks"),
        Reward::Spell(sp) => format!("Spell: {}", sp.def().name),
    }
}

pub fn project_name(bit: u32) -> &'static str {
    match bit {
        town::FOUNTAIN => "the fountain flows",
        town::LAMPS => "the lamps are lit",
        town::GARDENS => "the gardens bloom",
        town::BRIDGE => "the bridges are mended",
        town::BUNTING => "festival bunting",
        town::WISH_TREE => "the Wishing Tree blooms",
        town::CLOCK => "the clock chimes",
        town::MARKET => "market day returns",
        _ => "something wonderful",
    }
}

/// A celebration on screen after finishing a quest.
pub struct Cheer {
    pub title: String,
    pub who: Villager,
    pub lines: Vec<(Option<Stack>, String, u8)>,
    pub t: f32,
}

impl Play {
    // --------------------------------------------------------------------------------------
    // Progress
    // --------------------------------------------------------------------------------------

    /// How far along a quest is: (have, need).
    pub fn progress(&self, q: &Quest) -> (u64, u64) {
        let inv = |i: Item| self.player.inv.count(i) as u64;
        match q.goal() {
            Goal::Bring(i, n) => (inv(i).min(n as u64), n as u64),
            Goal::Find(i, _, _) => (inv(i).min(1), 1),
            Goal::Gather(i, n, _) => (inv(i).min(n as u64), n as u64),
            Goal::Slay(_, n, _)
            | Goal::Harvest(_, n)
            | Goal::Enchant(n)
            | Goal::Cast(n)
            | Goal::Brew(n)
            | Goal::Fish(n)
            | Goal::Cook(n) => (q.n.min(n as u64), n as u64),
            Goal::Fishdex(n) => (self.journal.fish.len().min(n as usize) as u64, n as u64),
            Goal::Charm(n) => (self.charisma().min(n) as u64, n as u64),
            Goal::Furnish(n) => (self.house.furnished().min(n as usize) as u64, n as u64),
            Goal::Tank(n) => (self.house.fish_kept().min(n as usize) as u64, n as u64),
            Goal::Mastery(l) => (self.spells.best_level().min(l) as u64, l as u64),
            Goal::Ship(v) => (q.n.min(v), v),
            Goal::Reach(d) => (self.deepest.min(d) as u64, d as u64),
            Goal::Guardian(d) => (self.journal.guardians.contains(&d) as u64, 1),
            Goal::Deliver(_, _) => (0, 1),
            Goal::Meet(m) => ((q.met & m).count_ones() as u64, m.count_ones() as u64),
            Goal::Almanac(n) => (self.journal.crops.len().min(n as usize) as u64, n as u64),
            Goal::Curios(n) => (self.journal.curios.len().min(n as usize) as u64, n as u64),
            Goal::Cookbook(n) => (self.journal.dishes.len().min(n as usize) as u64, n as u64),
            Goal::Codex(n) => (self.journal.foes.len().min(n as usize) as u64, n as u64),
        }
    }

    pub fn quest_ready(&self, q: &Quest) -> bool {
        let (have, need) = self.progress(q);
        have >= need
    }

    pub fn is_done(&self, key: &str) -> bool {
        self.done.iter().any(|d| d == key)
    }

    pub fn is_active(&self, key: &str) -> bool {
        self.quests.iter().any(|q| q.key() == key)
    }

    /// The next story quest a villager could ask you about, if any.
    pub fn quest_for(&self, v: Villager) -> Option<&'static QuestDef> {
        QUESTS.iter().find(|d| {
            d.giver == v
                && !self.is_done(d.key)
                && !self.is_active(d.key)
                && (d.after.is_empty() || self.is_done(d.after))
                && self.deepest >= d.depth
                && self.friends.hearts(v) >= d.hearts
                && self.clock.day >= d.day
                && self.charisma() >= d.charm
                && d.season.is_none_or(|s| s == self.clock.season())
        })
    }

    /// A quest ready to hand in to this villager (theirs, or a delivery for them).
    pub fn quest_to_finish(&self, v: Villager) -> Option<usize> {
        self.quests.iter().position(|q| {
            if q.request().is_some() {
                return false;
            }
            match q.goal() {
                Goal::Deliver(item, to) => to == v && self.player.inv.count(item) > 0,
                _ => q.giver() == v && self.quest_ready(q),
            }
        })
    }

    /// The marker over a villager's head: '!' for something new, '?' to hand in.
    pub fn marker(&self, v: Villager) -> Option<bool> {
        if self.quest_to_finish(v).is_some() {
            Some(true)
        } else if self.quest_for(v).is_some() || self.present_due(v).is_some() {
            Some(false)
        } else {
            None
        }
    }

    /// Takes a quest on.
    pub fn accept(&mut self, id: QuestId, io: &mut Io) -> bool {
        let q = Quest {
            id,
            n: 0,
            met: 0,
            day: self.clock.day,
        };
        if let Goal::Deliver(item, to) = q.goal() {
            if !self.player.inv.can_fit(item, 1) {
                self.toast("Make room in your bag first.", None, 0);
                io.audio.play(Sfx::Denied);
                return false;
            }
            self.player.inv.add(item, 1);
            self.toast(
                format!("Got the {} for {}", item.def().name, to.name()),
                Some(item),
                1,
            );
        }
        let title = q.title();
        self.quests.push(q);
        self.toast_colored(format!("New quest: {title}"), None, 0, GOLD);
        io.audio.play(Sfx::Accept);
        true
    }

    /// Hands a quest in: takes what was asked for and gives the rewards.
    pub fn finish_quest(&mut self, i: usize, io: &mut Io) {
        let q = self.quests.remove(i);
        match q.goal() {
            Goal::Bring(item, n) | Goal::Gather(item, n, _) => {
                self.player.inv.take(item, n as u32);
            }
            Goal::Find(item, _, _) | Goal::Deliver(item, _) => {
                self.player.inv.take(item, 1);
            }
            _ => {}
        }
        if let QuestId::Story(k) = &q.id {
            self.done.push(k.clone());
        }
        self.tidy_keepsakes(q.goal());
        let giver = q.giver();
        let mut lines = Vec::new();
        let rewards = q.rewards();
        // Finishing anything warms up the person who asked.
        let warm = if q.request().is_some() { 40 } else { 90 };
        if self.friends.add(giver, warm) {
            let h = self.friends.hearts(giver);
            let hearts = if h == 1 { "heart" } else { "hearts" };
            lines.push((None, format!("{}: {h} {hearts}!", giver.name()), PINK));
        }
        for r in rewards {
            if let Some(line) = self.grant(&r, io) {
                lines.push(line);
            }
        }
        self.stats.quests += 1;
        io.audio.play(Sfx::Fanfare);
        let at = self.player.world_pos() + Vec3::Y * 0.8;
        self.fx
            .burst(at, 30, &[GOLD, PINK, SKY, LIME, WHITE], 3.5, 3.0);
        self.fx.motes(at, 24, &[GOLD, CREAM, WHITE], 0.8);
        self.cheer = Some(Cheer {
            title: q.title(),
            who: giver,
            lines,
            t: 0.0,
        });
    }

    /// Gives up on a quest: whatever it gave you to carry goes back.
    pub fn drop_quest(&mut self, i: usize) {
        if i >= self.quests.len() {
            return;
        }
        let q = self.quests.remove(i);
        self.tidy_keepsakes(q.goal());
        self.toast(format!("Dropped \"{}\"", q.title()), None, 0);
    }

    /// Keepsakes nobody needs any more crumble away.
    fn tidy_keepsakes(&mut self, g: Goal) {
        let item = match g {
            Goal::Find(i, ..) | Goal::Gather(i, ..) | Goal::Deliver(i, _) => i,
            _ => return,
        };
        let still = self.quests.iter().any(|q| match q.goal() {
            Goal::Find(i, ..) | Goal::Gather(i, ..) | Goal::Deliver(i, _) => i == item,
            _ => false,
        });
        if !still {
            let n = self.player.inv.count(item);
            if n > 0 {
                self.player.inv.take(item, n);
            }
        }
    }

    /// Gives one reward. Returns a line for the celebration.
    pub fn grant(&mut self, r: &Reward, io: &mut Io) -> Option<(Option<Stack>, String, u8)> {
        let level = (self.deepest.max(1) as u16).max(self.player.level as u16);
        match *r {
            Reward::Coins(c) => {
                self.money += c;
                self.stats.earned += c;
                io.audio.play(Sfx::Coin);
                Some((
                    Some(Stack::new(Item::GoldCoin, 1)),
                    loot::money_text(c),
                    GOLD,
                ))
            }
            Reward::Item(item, n) => {
                let s = Stack::new(item, n);
                self.give(s);
                Some((Some(s), reward_text(r), CREAM))
            }
            Reward::Gear(item, rarity) => {
                let mut s = loot::roll_gear(item, level, 0.5, &mut self.rng);
                for _ in 0..200 {
                    if s.rarity() >= Some(rarity) {
                        break;
                    }
                    s = loot::roll_gear(
                        item,
                        level,
                        0.5 + rarity as u8 as f32 * 0.3,
                        &mut self.rng,
                    );
                }
                if let Some(g) = &mut s.gear {
                    g.quality = g.quality.max(80);
                    if g.rarity < rarity {
                        g.rarity = rarity;
                    }
                }
                self.give(s);
                let col = s.rarity().map_or(CREAM, |r| r.color());
                Some((Some(s), s.name(), col))
            }
            Reward::Scroll(group, rarity) => {
                let f = loot::Fortune {
                    luck: 0.5 + rarity as u8 as f32 * 0.4,
                    greed: 0.0,
                };
                let mut s = loot::scroll_of(group, level as u32, f, &mut self.rng);
                for _ in 0..200 {
                    if s.rarity() >= Some(rarity) {
                        break;
                    }
                    s = loot::scroll_of(group, level as u32, f, &mut self.rng);
                }
                self.give(s);
                let col = s.rarity().map_or(CREAM, |r| r.color());
                Some((Some(s), s.name(), col))
            }
            Reward::Friend(v, n) => {
                self.friends.add(v, n);
                Some((None, format!("{} ♥ +{}", v.name(), n), PINK))
            }
            Reward::Restore(bit) => {
                let new = self.restored & bit == 0;
                self.restored |= bit;
                if new {
                    self.town = town::generate(self.restored);
                }
                if bit == town::CLOCK {
                    io.audio.play(Sfx::Bell);
                }
                Some((None, format!("Bramblewick: {}!", project_name(bit)), MINT))
            }
            Reward::Marks(n) => {
                self.add_marks(n, io);
                Some((None, format!("{n} guild marks"), SKY))
            }
            Reward::Spell(sp) => {
                let def = sp.def();
                if self.spells.learn(sp) {
                    io.audio.play(Sfx::SpellUp);
                    Some((None, format!("New spell: {}", def.name), def.colors[1]))
                } else {
                    // Already known: a lesson's worth of practice instead.
                    let k = self.spells.get_mut(sp)?;
                    let need = super::spells::xp_to_next(k.level);
                    k.practise(need);
                    Some((
                        None,
                        format!("{} lesson: now Lv {}", def.name, k.level),
                        def.colors[1],
                    ))
                }
            }
        }
    }

    /// Into the bag, or onto the ground at your feet if it's full.
    pub fn give(&mut self, s: Stack) {
        self.on_pickup(s.item);
        let left = self.player.inv.add_stack(s);
        if left > 0 {
            let at = self.player.world_pos();
            self.drops
                .push(Drop::new(Stack { n: left, ..s }, at, &mut self.rng));
        }
    }

    /// Guild marks, and promotions when enough pile up.
    pub fn add_marks(&mut self, n: u32, io: &mut Io) {
        self.marks += n;
        while (self.rank as usize) + 1 < RANKS.len()
            && self.marks >= RANKS[self.rank as usize + 1].0
        {
            self.rank += 1;
            let (_, title, rewards) = RANKS[self.rank as usize];
            self.toast_colored(format!("Guild rank up: {title}!"), None, 0, GOLD);
            io.audio.play(Sfx::LevelUp);
            for r in rewards {
                if let Some((_, line, _)) = self.grant(r, io) {
                    self.toast(line, None, 0);
                }
            }
        }
    }

    // --------------------------------------------------------------------------------------
    // Hooks from the rest of the game
    // --------------------------------------------------------------------------------------

    /// A creature fell: count it, and maybe drop something a quest wants.
    pub fn on_kill(&mut self, foe: Foe, boss: bool, depth: u32, at: Vec3) {
        note(&mut self.journal.foes, foe);
        if boss {
            note(&mut self.journal.guardians, depth);
        }
        let biome = biome_for(depth) as u8;
        let mut drops = Vec::new();
        for q in &mut self.quests {
            match q.goal() {
                Goal::Slay(f, _, d) if f.is_none_or(|f| f == foe) && depth >= d => q.n += 1,
                Goal::Gather(item, n, src) => {
                    let (hit, chance) = match src {
                        Source::Foe(f) => (f == foe && !boss, 0.4),
                        Source::Biome(b) => (b == biome, 0.3),
                        Source::Deep(d) => (depth >= d, 0.2),
                        Source::Boss(f) => (boss && f == foe, 1.0),
                        // These come up out of the floor instead (see `candy`).
                        Source::Eruption(_) => (false, 0.0),
                    };
                    let have = self.player.inv.count(item)
                        + drops.iter().filter(|(i, _)| *i == item).count() as u32
                        + self
                            .drops
                            .iter()
                            .filter(|d| d.stack.item == item)
                            .map(|d| d.stack.n as u32)
                            .sum::<u32>();
                    if hit && have < n as u32 && self.rng.chance(chance) {
                        drops.push((item, 1u16));
                    }
                }
                _ => {}
            }
        }
        for (item, n) in drops {
            self.drops.push(Drop::item(item, n, at, &mut self.rng));
        }
    }

    pub fn on_harvest(&mut self, crop: Crop, n: u16) {
        note(&mut self.journal.crops, crop);
        for q in &mut self.quests {
            if let Goal::Harvest(c, _) = q.goal() {
                if c.is_none_or(|c| c == crop) {
                    q.n += n as u64;
                }
            }
        }
    }

    pub fn on_ship(&mut self, value: u64) {
        for q in &mut self.quests {
            if let Goal::Ship(_) = q.goal() {
                q.n += value;
            }
        }
    }

    pub fn on_enchant(&mut self) {
        for q in &mut self.quests {
            if let Goal::Enchant(_) = q.goal() {
                q.n += 1;
            }
        }
    }

    pub fn on_craft(&mut self, item: Item) {
        if matches!(item.def().kind, Kind::Food { .. }) {
            note(&mut self.journal.dishes, item);
        }
    }

    /// A spell cast (at some level).
    pub fn on_cast(&mut self, _level: u8) {
        for q in &mut self.quests {
            if let Goal::Cast(_) = q.goal() {
                q.n += 1;
            }
        }
    }

    /// A fish landed. Returns whether it's a new kind, and whether it's a new record.
    pub fn on_catch(&mut self, item: Item, cm: u16) -> (bool, bool) {
        self.stats.caught += 1;
        let new = note(&mut self.journal.fish, item);
        let record = match self.journal.records.iter_mut().find(|r| r.0 == item) {
            Some(r) if cm > r.1 => {
                r.1 = cm;
                true
            }
            Some(_) => false,
            None => {
                self.journal.records.push((item, cm));
                false
            }
        };
        for q in &mut self.quests {
            if let Goal::Fish(_) = q.goal() {
                q.n += 1;
            }
        }
        (new, record)
    }

    /// Dishes cooked on a stove.
    pub fn on_cook(&mut self, n: u16) {
        self.stats.cooked += n as u32;
        for q in &mut self.quests {
            if let Goal::Cook(_) = q.goal() {
                q.n += n as u64;
            }
        }
    }

    /// Potions brewed.
    pub fn on_brew(&mut self, n: u16) {
        self.stats.brewed += n as u32;
        for q in &mut self.quests {
            if let Goal::Brew(_) = q.goal() {
                q.n += n as u64;
            }
        }
    }

    pub fn on_pickup(&mut self, item: Item) {
        if matches!(item.def().kind, Kind::Gem | Kind::Relic) {
            note(&mut self.journal.curios, item);
        }
    }

    pub fn on_meet(&mut self, v: Villager) {
        for q in &mut self.quests {
            if let Goal::Meet(m) = q.goal() {
                q.met |= m & (1 << v as u32);
            }
        }
    }

    /// Keepsakes that are waiting on this floor, and guardians returning for their prizes.
    pub fn keepsakes_here(&self, depth: u32) -> Vec<Item> {
        self.quests
            .iter()
            .filter_map(|q| match q.goal() {
                Goal::Find(item, lo, hi)
                    if depth >= lo && depth <= hi && self.player.inv.count(item) == 0 =>
                {
                    Some(item)
                }
                _ => None,
            })
            .collect()
    }

    /// True if a quest wants this floor's guardian back.
    pub fn guardian_wanted(&self, depth: u32) -> bool {
        if !super::dungeon::is_waystone_floor(depth) {
            return false;
        }
        let boss = super::dungeon::boss_for(depth);
        self.quests.iter().any(|q| match q.goal() {
            Goal::Gather(item, n, Source::Boss(f)) => {
                f == boss && self.player.inv.count(item) < n as u32
            }
            Goal::Gather(item, n, Source::Eruption(d)) => {
                d == depth && self.player.inv.count(item) < n as u32
            }
            _ => false,
        })
    }

    /// What a quest wants to see come up out of this floor when its guardian falls, if
    /// anything.
    pub fn eruption_wanted(&self, depth: u32) -> Option<Item> {
        self.quests.iter().find_map(|q| match q.goal() {
            Goal::Gather(item, n, Source::Eruption(d))
                if d == depth && self.player.inv.count(item) < n as u32 =>
            {
                Some(item)
            }
            _ => None,
        })
    }

    // --------------------------------------------------------------------------------------
    // The boards
    // --------------------------------------------------------------------------------------

    /// Today's notices on a board.
    pub fn notices(&self, guild: bool) -> Vec<Request> {
        let day = self.clock.day;
        let n = if guild { 2 } else { 3 };
        (0..n)
            .map(|slot| make_request(self, day, slot, guild))
            .filter(|r| !self.taken.contains(&r.id))
            .collect()
    }
}

/// Builds one notice from the day and slot, scaled to how deep you've been.
pub fn make_request(p: &Play, day: u32, slot: u32, guild: bool) -> Request {
    let id = day * 16 + slot + if guild { 8 } else { 0 };
    let mut r = Rng::new(p.seed ^ (id as u64).wrapping_mul(0x9E37_79B9));
    let deepest = p.deepest.max(1);
    let biome = biome_for(deepest).min(5) as u8;
    let givers: Vec<Villager> = VILLAGERS.to_vec();
    let giver = if guild {
        Villager::Rowan
    } else {
        givers[r.below(givers.len())]
    };
    let scale = 1.0 + deepest as f32 / 12.0;
    let (title, text, goal, coins) = if guild || r.chance(0.3) {
        // Monsters.
        let floor = (r.range(0, biome as i32 + 1) as u32) * 10 + 1;
        let foes = super::dungeon::floor_foes(r.range(0, biome as i32 + 1) as usize, floor);
        let foe = foes[r.below(foes.len())].0;
        let n = r.range(8, 20) as u16;
        let coins = (n as f32 * 45.0 * (1.0 + floor as f32 / 8.0)) as u64;
        (
            format!("{} Cull", cap(foe_name(foe))),
            format!(
                "Too many {}s about. Thin them out, floor {floor} or deeper.",
                foe_name(foe)
            ),
            Goal::Slay(Some(foe), n, floor),
            coins,
        )
    } else if r.chance(0.3) {
        // A fish from waters you've reached.
        let reach: Vec<super::fish::Water> = {
            use super::fish::Water as W;
            let mut v = vec![W::Pond, W::River];
            // The Hollow's waters, one for each biome going down. (Not the sewers: they come
            // and go, so nobody counts on them.)
            for (b, w) in super::fish::WATERS[2..].iter().enumerate() {
                // Lava fish need a heat-proof rod: nobody asks for those on the board.
                if b as u8 <= biome && deepest > b as u32 * 10 && !matches!(w, W::Lava | W::Sewer) {
                    v.push(*w);
                }
            }
            v
        };
        let fish: Vec<&super::fish::FishDef> = super::fish::FISH
            .iter()
            .filter(|d| {
                d.rarity <= 1
                    && d.when != super::fish::When::Rain
                    && d.water.iter().any(|w| reach.contains(w))
            })
            .collect();
        let d = fish[r.below(fish.len())];
        let n = if d.rarity == 0 {
            r.range(1, 4) as u16
        } else {
            1
        };
        let coins = (d.item.def().price as f32 * n as f32 * 3.0) as u64 + 200;
        let from = d
            .water
            .iter()
            .find(|w| reach.contains(w))
            .map_or("the farm pond", |w| w.name());
        (
            format!("Wanted: {}", d.item.def().name),
            format!(
                "Fancy a spot of fishing? I'm after {n} {} from {from}.",
                d.item.def().name
            ),
            Goal::Bring(d.item, n),
            coins,
        )
    } else if r.chance(0.5) {
        // Crops from the farm, often something in season.
        let season = super::play::Clock { day, min: 0.0 }.season();
        let ripe = seasonal_seeds(season.bit());
        let fresh = r.chance(0.5);
        let seed = if fresh {
            ripe[r.below(ripe.len())]
        } else {
            // Only what grows on the notice's day.
            let mut crops = super::menus::shop_seeds(p);
            crops
                .retain(|(i, _)| matches!(i.def().kind, Kind::Seed(c) if c.grows_in(season.bit())));
            crops[r.below(crops.len())].0
        };
        let crop = match seed.def().kind {
            Kind::Seed(c) => c,
            _ => Crop::Turnip,
        };
        let produce = crop.def().produce;
        let n = r.range(4, 12) as u16;
        let coins = (produce.def().price as f32 * n as f32 * 2.4) as u64 + 100;
        let name = produce.def().name;
        if fresh {
            (
                format!("In Season: {name}"),
                format!("It's {} at last! I'd love {n} fresh {name}.", season.name()),
                Goal::Bring(produce, n),
                coins,
            )
        } else {
            (
                format!("Wanted: {name}"),
                format!("I need {n} {name} for something special. Can you help?"),
                Goal::Bring(produce, n),
                coins,
            )
        }
    } else {
        // Things from the Hollow.
        let mats: &[Item] = match biome {
            0 => &[
                Item::SlimeGel,
                Item::BatWing,
                Item::ShroomCap,
                Item::CopperOre,
                Item::Stone,
            ],
            1 => &[
                Item::CrabShell,
                Item::WispDust,
                Item::Crystal,
                Item::IronOre,
            ],
            2 => &[
                Item::BeetleShell,
                Item::Spore,
                Item::ShroomCap,
                Item::GoldOre,
            ],
            3 => &[Item::ImpHorn, Item::EmberOre, Item::Amber, Item::GoldOre],
            4 => &[
                Item::FrostGem,
                Item::GolemHeart,
                Item::WispDust,
                Item::IronOre,
            ],
            _ => &[
                Item::Bone,
                Item::Ectoplasm,
                Item::GolemHeart,
                Item::AncientCoin,
            ],
        };
        let item = mats[r.below(mats.len())];
        let n = r.range(3, 12) as u16;
        let coins = (item.def().price as f32 * n as f32 * 2.6) as u64 + 150;
        (
            format!("Hollow Goods: {}", item.def().name),
            format!(
                "Could anyone bring me {n} {} from the Hollow?",
                item.def().name
            ),
            Goal::Bring(item, n),
            coins,
        )
    };
    let coins = ((coins as f32 * scale * if guild { 1.3 } else { 1.0 }) as u64 / 10).max(10) * 10;
    let mut reward = vec![Coins(coins)];
    if guild {
        reward.push(Marks(15 + deepest.min(60) / 2));
    } else {
        reward.push(Friend(giver, 40));
    }
    if r.chance(0.35) {
        let group = [Group::Weapon, Group::Armor, Group::Tool][r.below(3)];
        reward.push(Scroll(group, Rarity::Uncommon));
    } else if r.chance(0.3) {
        let seeds = super::dungeon::biome_seeds(biome as usize);
        reward.push(item(seeds[r.below(seeds.len())].0, 5));
    }
    Request {
        id,
        giver,
        title,
        text,
        goal,
        reward,
        guild,
    }
}

fn cap(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
        None => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quests_are_well_formed() {
        let mut keys = std::collections::HashSet::new();
        for q in QUESTS {
            assert!(keys.insert(q.key), "duplicate quest {}", q.key);
            assert!(!q.title.is_empty() && !q.ask.is_empty() && !q.thanks.is_empty());
            assert!(!q.reward.is_empty(), "{} gives nothing", q.key);
            if !q.after.is_empty() {
                let before =
                    quest_def(q.after).unwrap_or_else(|| panic!("{} needs a missing quest", q.key));
                assert!(
                    QUESTS.iter().position(|x| x.key == before.key)
                        < QUESTS.iter().position(|x| x.key == q.key),
                    "{} comes before what it needs",
                    q.key
                );
            }
            match q.goal {
                Goal::Find(item, lo, hi) => {
                    assert!(lo <= hi, "{} has an empty floor range", q.key);
                    assert_eq!(
                        item.def().kind,
                        Kind::Keepsake,
                        "{} hides a normal item",
                        q.key
                    );
                }
                Goal::Gather(item, _, _) | Goal::Deliver(item, _) => {
                    assert_eq!(
                        item.def().kind,
                        Kind::Keepsake,
                        "{} asks for a normal item",
                        q.key
                    );
                }
                Goal::Bring(item, _) => {
                    assert_ne!(
                        item.def().kind,
                        Kind::Keepsake,
                        "{} wants a keepsake brought",
                        q.key
                    );
                }
                _ => {}
            }
        }
        assert!(QUESTS.len() >= 100, "only {} story quests", QUESTS.len());
        // Everyone in town has something to ask.
        for &v in VILLAGERS {
            assert!(
                QUESTS.iter().filter(|q| q.giver == v).count() >= 5,
                "{} is idle",
                v.name()
            );
        }
    }

    #[test]
    fn seasonal_quests_wait_for_their_season() {
        use crate::game::play::SEASON_DAYS;
        let mut p = Play::new(9);
        let seasonal: Vec<&QuestDef> = QUESTS.iter().filter(|q| q.season.is_some()).collect();
        assert!(
            seasonal.len() >= 12,
            "only {} seasonal quests",
            seasonal.len()
        );
        for q in seasonal {
            let at = q.season.unwrap() as u32;
            for s in 0..4 {
                // A few days in, and again a year later.
                for year in [0, 4] {
                    p.clock.day = (s + year) * SEASON_DAYS + 5;
                    let asks = p.quest_for(q.giver).is_some_and(|d| d.key == q.key);
                    assert_eq!(asks, s == at, "{} in season {s}", q.key);
                }
            }
            // What they ask for can be grown in that season.
            if let Goal::Bring(item, _) = q.goal {
                let crop = crate::game::items::ALL_CROPS
                    .iter()
                    .find(|c| c.def().produce == item);
                if let Some(c) = crop {
                    assert!(c.grows_in(1 << at), "{} asks out of season", q.key);
                }
            }
        }
    }

    #[test]
    fn board_asks_for_what_is_in_season() {
        use crate::game::play::SEASON_DAYS;
        let p = Play::new(9);
        for season in 0..4u32 {
            let bit = 1u8 << season;
            let mut fresh = 0;
            for d in 0..SEASON_DAYS {
                let day = season * SEASON_DAYS + d + 1;
                for slot in 0..3 {
                    let r = make_request(&p, day, slot, false);
                    if let Goal::Bring(item, _) = r.goal {
                        let c = crate::game::items::ALL_CROPS
                            .iter()
                            .find(|c| c.def().produce == item);
                        if let Some(c) = c {
                            assert!(c.grows_in(bit), "{} asked for out of season", r.title);
                            if c.def().seasons != 0 {
                                fresh += 1;
                            }
                        }
                    }
                }
            }
            assert!(fresh > 0, "no seasonal notices in season {season}");
        }
    }

    #[test]
    fn notices_are_steady_and_scale() {
        let mut p = Play::new(9);
        let a = make_request(&p, 3, 0, false);
        let b = make_request(&p, 3, 0, false);
        assert_eq!(a.title, b.title);
        let early: u64 = (0..20)
            .map(|d| match make_request(&p, d, 1, true).reward[0] {
                Reward::Coins(c) => c,
                _ => 0,
            })
            .sum();
        p.deepest = 50;
        let late: u64 = (0..20)
            .map(|d| match make_request(&p, d, 1, true).reward[0] {
                Reward::Coins(c) => c,
                _ => 0,
            })
            .sum();
        assert!(late > early * 2, "bounties don't grow: {early} vs {late}");
    }
}
