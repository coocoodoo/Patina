//! The people of Bramblewick: who they are, what they wear and love, where they spend their
//! days, and how they get about town.

use std::collections::VecDeque;

use glam::{Vec2, Vec3};
use serde::{Deserialize, Serialize};

use super::items::{Item, Stack};
use super::play::Play;
use super::town::{BUILDINGS, Place, TOWN};
use super::world::{Area, World};
use crate::assets::models::{Hair, Look};
use crate::palette::*;
use crate::util::{Rng, damp, hash2, wrap_angle};

macro_rules! folk {
    ($( $id:ident ),* $(,)?) => {
        #[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, PartialOrd, Ord, Serialize, Deserialize)]
        pub enum Villager { $($id),* }
        pub const VILLAGERS: &[Villager] = &[ $(Villager::$id),* ];
    };
}

folk! {
    Thistle,
    Rowan,
    Quill,
    Hilde,
    Garrick,
    Nix,
    Opal,
    Wren,
    Posy,
    Mabel,
    Barley,
    Fern,
    Pip,
    Juniper,
    Bramble,
    Toby,
    Clank,
    Mira,
    Olive,
    Hazel,
}

pub const FOLK: usize = 20;

/// Where someone is at a given time.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Spot {
    Town,
    Inside(Place),
    Away,
}

pub struct VillagerDef {
    pub name: &'static str,
    /// A few words under their name.
    pub title: &'static str,
    pub look: Look,
    /// Gear they wear (for their hat, clothes and boots).
    pub wears: &'static [Item],
    pub holds: Option<Item>,
    /// Pitch of their chatter.
    pub voice: f32,
    /// The shop or hall they keep, if any.
    pub keeps: Option<Place>,
    /// Their home, by building index.
    pub home: usize,
    /// (from, to, where) in minutes after midnight; the first match wins, else they're home.
    pub hours: &'static [(f32, f32, Spot)],
    /// Favourite places around town.
    pub haunts: &'static [(i32, i32)],
    pub loves: &'static [Item],
    pub likes: &'static [Item],
    pub hates: &'static [Item],
    /// The first thing they ever say to you.
    pub hello: &'static str,
    /// Everyday chatter.
    pub chat: &'static [&'static str],
    /// Chatter once you're close friends (six hearts or more).
    pub close: &'static [&'static str],
}

const SKIN: [u8; 3] = [PEACH, PEACH, SALMON];
const ROSY: [u8; 3] = [PEACH, SALMON, ROSEWOOD];
const TAN: [u8; 3] = [SAND, KHAKI, ROSEWOOD];
const BROWN: [u8; 3] = [CLAY, RUST, MAROON];

const fn look(
    hair: [u8; 3],
    skin: [u8; 3],
    shirt: [u8; 3],
    pants: u8,
    style: Hair,
    scale: f32,
) -> Look {
    Look {
        hair,
        skin,
        eyes: INK,
        cheeks: SALMON,
        shirt,
        belt: RUST,
        pants,
        boots: MAROON,
        style,
        beard: false,
        scale,
    }
}

// Building indices, from `town::BUILDINGS`.
const GUILD: usize = 0;
const JUNIPER_HOME: usize = 1;
const HALL: usize = 2;
const BRAMBLE_HOME: usize = 3;
const SCROLLS: usize = 4;
const ARMORY: usize = 5;
const SMITHY: usize = 6;
const TOOLS: usize = 7;
const BAKERY: usize = 8;
const JEWELER: usize = 9;
const NOOK: usize = 10;
const SEEDS: usize = 11;
const OLIVE_HOME: usize = 12;
const TAVERN: usize = 13;
const FERN_HOME: usize = 14;
const PIP_HOME: usize = 15;
const TOBY_HOME: usize = 16;
const CLANK_HOME: usize = 17;
const MIRA_HOME: usize = 18;
const SPELLERY: usize = 19;

use Spot::{Inside, Town};

pub static VILLAGER_DEFS: [VillagerDef; FOLK] = [
    VillagerDef {
        name: "Mayor Thistle",
        title: "Mayor of Bramblewick",
        look: Look {
            belt: GOLD,
            boots: GRAPE,
            ..look(
                [WHITE, LAVENDER, PURPLE],
                SKIN,
                [LAVENDER, PURPLE, GRAPE],
                GRAPE,
                Hair::Bun,
                0.94,
            )
        },
        wears: &[],
        holds: None,
        voice: 0.9,
        keeps: Some(Place::Hall),
        home: HALL,
        hours: &[
            (480.0, 1080.0, Inside(Place::Hall)),
            (1080.0, 1250.0, Town),
            (1250.0, 1380.0, Inside(Place::Tavern)),
        ],
        haunts: &[(35, 23), (33, 28), (34, 12), (30, 22), (37, 28)],
        loves: &[Item::Rose, Item::PumpkinPie, Item::MoonPearl],
        likes: &[
            Item::Sunflower,
            Item::FreshBread,
            Item::MintTea,
            Item::TinyCrown,
            Item::Pumpkin,
            Item::Chrysanthemum,
            Item::Poinsettia,
        ],
        hates: &[Item::SlimeGel, Item::Bone],
        hello: "Oh! A new face! You must be the one who took on the old farm over the \
                Hollow. Welcome to Bramblewick! We've seen better days, I'm afraid - the \
                fountain's dry, the lamps are out - but with a little help, who knows?",
        chat: &[
            "Bramblewick was the jewel of the valley once. We'll get there again.",
            "Do say hello to everyone. A town is only as warm as its neighbours.",
            "The request board in the plaza always has something that needs doing.",
            "Paperwork, paperwork. Even mayors dream of adventure, you know.",
            "The bus runs all day. Our driver has never once been late. Or early.",
        ],
        close: &[
            "You've done more for this town in a season than I did in ten years.",
            "Between you and me, I keep a jar of your turnips on my desk for luck.",
        ],
    },
    VillagerDef {
        name: "Captain Rowan",
        title: "Lantern Guild captain",
        look: look(
            [SLATE, SHADOW, INK],
            BROWN,
            [SKY, BLUE, INDIGO],
            INDIGO,
            Hair::Fluffy,
            1.08,
        ),
        wears: &[Item::IronPlate, Item::IronBoots, Item::IronShield],
        holds: None,
        voice: 0.8,
        keeps: Some(Place::Guild),
        home: GUILD,
        hours: &[(420.0, 1320.0, Inside(Place::Guild))],
        haunts: &[(8, 12), (21, 12)],
        loves: &[Item::EmberCurry, Item::GolemHeart, Item::DragonScale],
        likes: &[
            Item::ImpHorn,
            Item::IronOre,
            Item::VeggieStew,
            Item::BatWing,
            Item::RoastedChestnuts,
            Item::StuffedPeppers,
        ],
        hates: &[Item::Rose, Item::Popcorn],
        hello: "So you're the farmer who goes down the Hollow. Brave, or a little mad - \
                the best delvers are both. The Lantern Guild pays for monsters cleared \
                and odd jobs done. Check the bounty board any time.",
        chat: &[
            "Keep your lantern lit and your boots laced.",
            "Every floor you clear makes the valley a little safer.",
            "Guardians sit on every tenth floor. Don't face one on an empty stomach.",
            "I've a bounty or two on the board. The pay's honest.",
            "Roll through an attack, not away from it. Trust me.",
        ],
        close: &[
            "When I was your age I got lost on floor nine for a week. Don't tell anyone.",
            "You've got the makings of a legend, farmer.",
        ],
    },
    VillagerDef {
        name: "Elder Quill",
        title: "Keeper of scrolls",
        look: Look {
            beard: true,
            ..look(
                [WHITE, WHITE, SAND],
                SKIN,
                [SKY, BLUE, INDIGO],
                INDIGO,
                Hair::Bald,
                0.92,
            )
        },
        wears: &[Item::WizardHat, Item::MageRobe],
        holds: Some(Item::StarWand),
        voice: 0.75,
        keeps: Some(Place::Scrolls),
        home: SCROLLS,
        hours: &[
            (420.0, 540.0, Town),
            (540.0, 1020.0, Inside(Place::Scrolls)),
            (1020.0, 1260.0, Town),
        ],
        haunts: &[(55, 12), (58, 21), (65, 15), (65, 24)],
        loves: &[Item::WishStar, Item::Moonbloom, Item::StarFossil],
        likes: &[
            Item::Amethyst,
            Item::WispDust,
            Item::MintTea,
            Item::Glowcap,
            Item::LavenderTea,
            Item::Bluebell,
            Item::SnowRoseTea,
        ],
        hates: &[Item::GhostPepper],
        hello: "Hm? Ah, a visitor. Forgive me, I was reading. The Moonquill Scriptorium \
                sells scrolls for the enchanting table - weapon, armour and tool, each \
                to its own kind. Words have power, young one. Choose them well.",
        chat: &[
            "A scroll binds best to the gear it was written for. Never mix them.",
            "The stars were very talkative last night.",
            "Where did I leave my glasses? Oh. On my head. Again.",
            "Luck is a stat, you know. A fickle, wonderful stat.",
            "The deeper the Hollow, the older the magic.",
        ],
        close: &[
            "I was an adventurer once. Then I took a scroll to the knee. A paper cut.",
            "You have a gift for enchanting. I can feel it in the air around you.",
        ],
    },
    VillagerDef {
        name: "Hilde",
        title: "Armourer",
        look: look(
            [GOLD, ORANGE, RUST],
            ROSY,
            [PINK, CRIMSON, PLUM],
            MAROON,
            Hair::Pigtails,
            1.04,
        ),
        wears: &[Item::CopperMail, Item::LeatherBoots],
        holds: None,
        voice: 1.05,
        keeps: Some(Place::Armory),
        home: ARMORY,
        hours: &[
            (480.0, 1020.0, Inside(Place::Armory)),
            (1020.0, 1200.0, Town),
            (1200.0, 1320.0, Inside(Place::Tavern)),
        ],
        haunts: &[(8, 21), (12, 26)],
        loves: &[Item::GoldOre, Item::Ruby, Item::BerryTart],
        likes: &[
            Item::IronOre,
            Item::CopperOre,
            Item::CrabShell,
            Item::BeetleShell,
            Item::KaleStew,
            Item::Crabapple,
            Item::HollyCake,
        ],
        hates: &[Item::JellyShroom],
        hello: "HA! Another adventurer with no armour worth the name! Come in, come in. \
                Petalplate makes the sturdiest - and cutest - protection in the valley. \
                Helmets, chest plates, greaves, boots and shields. You name it!",
        chat: &[
            "A good helmet is worth ten good excuses.",
            "Dents are just stories written in metal.",
            "Pink armour scares monsters. It's true. They don't know what to make of it.",
            "Bring me ore from the Hollow and I'll be your friend for life!",
            "Defense softens every blow. Stack it up!",
            "See my anvil? Melt your old armour into the pieces you love. Every strike, a \
             little stronger!",
            "Armour from deep down needs a hotter fire to forge. And costs more. Ha!",
        ],
        close: &[
            "You're my favourite customer. Don't tell Sir Clank.",
            "I hammered a little heart inside the last piece you bought. Just for luck.",
        ],
    },
    VillagerDef {
        name: "Garrick",
        title: "Blacksmith",
        look: Look {
            beard: true,
            ..look(
                [ORANGE, RUST, MAROON],
                TAN,
                [SAND, KHAKI, SHADOW],
                SHADOW,
                Hair::Bald,
                1.1,
            )
        },
        wears: &[Item::LeatherVest, Item::IronBoots],
        holds: None,
        voice: 0.7,
        keeps: Some(Place::Smithy),
        home: SMITHY,
        hours: &[
            (480.0, 1020.0, Inside(Place::Smithy)),
            (1020.0, 1200.0, Town),
            (1200.0, 1380.0, Inside(Place::Tavern)),
        ],
        haunts: &[(15, 21), (21, 25)],
        loves: &[
            Item::EmberOre,
            Item::SpookyChili,
            Item::GolemHeart,
            Item::GoldenCarp,
            Item::GrilledTrout,
        ],
        likes: &[
            Item::IronOre,
            Item::GoldOre,
            Item::EmberPepper,
            Item::Wood,
            Item::BrookTrout,
            Item::Bait,
            Item::Watermelon,
            Item::PumpkinSoup,
        ],
        hates: &[Item::FrostLily, Item::SoggyBoot],
        hello: "...Hm. Sprig & Steel. Swords, wands, staffs. And rods - fishing's the only \
                quiet thing left in this valley. Good steel, fair prices. You break it, you \
                buy another. That's the deal.",
        chat: &[
            "...Hm.",
            "Heat the steel. Hit the steel. Simple.",
            "Wands are for people who don't like getting close. Fair enough.",
            "Bring ember ore if you find any. Burns hotter than anything.",
            "My forge hasn't gone out in thirty years.",
            "Fish don't talk. That's why I like them.",
            "When the bobber dips, strike. Don't think. Strike.",
            "The frogs down in the Hollow hoard rods. Better ones than mine. Don't tell anyone.",
        ],
        close: &[
            "...You're alright, farmer. Don't make it weird.",
            "Made you something. Don't thank me. ...Fine, you can thank me a little.",
            "Caught a golden carp once. Let it go. Some things should stay in the pond.",
        ],
    },
    VillagerDef {
        name: "Nix",
        title: "Inventor and toolmaker",
        look: look(
            [MINT, AQUA, TEAL],
            SKIN,
            [GOLD, CLAY, RUST],
            RUST,
            Hair::Spiky,
            0.9,
        ),
        wears: &[Item::MinerHat],
        holds: None,
        voice: 1.3,
        keeps: Some(Place::Tools),
        home: TOOLS,
        hours: &[
            (480.0, 1020.0, Inside(Place::Tools)),
            (1020.0, 1300.0, Town),
        ],
        haunts: &[(27, 21), (35, 12), (28, 26)],
        loves: &[Item::Crystal, Item::MusicBox, Item::ToyBoat],
        likes: &[
            Item::CopperOre,
            Item::GlassMarble,
            Item::LostButton,
            Item::Popcorn,
            Item::CandyCorn,
            Item::Frostberry,
            Item::WatermelonSlush,
        ],
        hates: &[Item::Fiber],
        hello: "Oh! Hi! Hello! Are you the farmer? Do you need a hoe? A better hoe? A hoe \
                that tills three rows? I'm working on one that tills FIVE. It only \
                exploded twice. Tinkerbolt Tools, at your service!",
        chat: &[
            "Did you know sprinklers were invented by a very lazy genius? Me. I mean, my gran.",
            "Enchant your tools! Haste on a pickaxe is SO satisfying.",
            "I'm building a clock. Or a toaster. It's hard to tell yet.",
            "Every gear has a purpose. Even the squeaky ones.",
            "Tools level up with you, you know. Deeper floors, better tools!",
        ],
        close: &[
            "You're like... the best test subject. I mean friend! Friend.",
            "I named a cog after you. It's the shiniest one.",
        ],
    },
    VillagerDef {
        name: "Opal",
        title: "Jeweller",
        look: look(
            [WHITE, SKY, BLUE],
            SKIN,
            [MINT, AQUA, TEAL],
            DEEP_TEAL,
            Hair::Long,
            1.0,
        ),
        wears: &[Item::CrystalCirclet],
        holds: None,
        voice: 1.1,
        keeps: Some(Place::Jeweler),
        home: JEWELER,
        hours: &[
            (420.0, 540.0, Town),
            (540.0, 1020.0, Inside(Place::Jeweler)),
            (1020.0, 1200.0, Town),
        ],
        haunts: &[(43, 21), (39, 27)],
        loves: &[Item::StarDiamond, Item::Moonstone, Item::MoonPearl],
        likes: &[
            Item::Ruby,
            Item::Sapphire,
            Item::Emerald,
            Item::Topaz,
            Item::Amethyst,
            Item::SnowRose,
            Item::Grapes,
        ],
        hates: &[Item::Bone, Item::SlimeGel],
        hello: "Welcome to Glimmer & Gold, darling. I buy gems and curios at the finest \
                prices in the valley - far better than any shipping bin. Everything \
                that sparkles has a story. Bring me yours.",
        chat: &[
            "A moonstone is just moonlight that fell asleep.",
            "I pay extra for gems and relics. Much extra. Tell your friends.",
            "Darling, that outfit needs a circlet.",
            "Did you know rubies love the Ember Depths? Warm little things.",
            "Every curio in my cabinet was once lost by someone.",
        ],
        close: &[
            "You're the only one who listens when I talk about facets. I adore you for it.",
            "I've set a stone aside for you. Don't argue, it suits you.",
        ],
    },
    VillagerDef {
        name: "Wren",
        title: "Furniture maker",
        look: look(
            [CLAY, RUST, MAROON],
            BROWN,
            [SALMON, CRIMSON, PLUM],
            RUST,
            Hair::Bob,
            0.95,
        ),
        wears: &[Item::CozySweater],
        holds: None,
        voice: 1.0,
        keeps: Some(Place::Nook),
        home: NOOK,
        hours: &[(480.0, 1020.0, Inside(Place::Nook)), (1020.0, 1280.0, Town)],
        haunts: &[(54, 21), (56, 25)],
        loves: &[
            Item::ChippedTeacup,
            Item::Sunflower,
            Item::CarrotCake,
            Item::PlushBunny,
        ],
        likes: &[
            Item::Wood,
            Item::Rose,
            Item::BlueberryMuffin,
            Item::Fiber,
            Item::LilyKoi,
            Item::Cosmos,
            Item::Poppy,
            Item::Mistletoe,
        ],
        hates: &[Item::Ectoplasm, Item::TinCan],
        hello: "Oh, hello! Come in out of the draught. The Cozy Nook has everything to make \
                a house a home - beds, sofas, lamps, rugs, wallpaper, fish tanks. The more \
                charming your home, the more folk will warm to you. A home should hug you \
                when you walk in.",
        chat: &[
            "Put a lamp by your door. Coming home in the dark is so much nicer.",
            "A bench in the right spot can fix a whole day.",
            "I'm knitting a tiny sweater for the fountain. Is that strange?",
            "Every piece you put in your house makes it more charming. Folk notice!",
            "The best furniture is the kind you want to sit in forever.",
            "Press T while you're holding furniture to turn it before you set it down.",
            "Two of everything is lovely. Twenty of anything is a warehouse.",
            "I keep my fanciest pieces for the most charming homes. You'll see.",
        ],
        close: &[
            "I made a cushion with your farm on it. It's lumpy. I love it.",
            "You always make the Nook feel busier. In a good way.",
        ],
    },
    VillagerDef {
        name: "Posy",
        title: "Seed seller",
        look: look(
            [BLUSH, PINK, CRIMSON],
            SKIN,
            [LIME, GREEN, TEAL],
            TEAL,
            Hair::Long,
            0.9,
        ),
        wears: &[Item::FlowerCrown],
        holds: None,
        voice: 1.25,
        keeps: Some(Place::Seeds),
        home: SEEDS,
        hours: &[
            (420.0, 1020.0, Inside(Place::Seeds)),
            (1020.0, 1260.0, Town),
        ],
        haunts: &[(9, 34), (32, 23), (38, 29)],
        loves: &[Item::Rose, Item::FlameTulip, Item::Moonbloom],
        likes: &[
            Item::Sunflower,
            Item::SweetPea,
            Item::FrostLily,
            Item::Strawberry,
            Item::PastelTulip,
            Item::Daffodil,
            Item::Hibiscus,
            Item::Snowdrop,
        ],
        hates: &[Item::ShroomCap],
        hello: "Hiiii! Welcome to Sproutling Seeds! Every seed is a tiny promise! I stock \
                seeds from the valley and from the deep Hollow - the deeper you've been, \
                the stranger the seeds I'll trust you with!",
        chat: &[
            "Water every day and the soil will love you back!",
            "Did you know flowers grow happier when you talk to them? I tell them jokes.",
            "Some crops keep giving after the first harvest. Those are my favourites!",
            "Fertile soil can grow twice in a night. Magic cans help!",
            "I'm trying to grow every single crop in the valley. Help me?",
        ],
        close: &[
            "You're my best customer AND my best friend. Double best!",
            "I named a rose after you. It's very prickly and very sweet.",
        ],
    },
    VillagerDef {
        name: "Mabel",
        title: "Baker",
        look: Look {
            belt: PINK,
            ..look(
                [CREAM, GOLD, CLAY],
                ROSY,
                [WHITE, CREAM, SAND],
                SALMON,
                Hair::Bun,
                1.0,
            )
        },
        wears: &[],
        holds: Some(Item::Baguette),
        voice: 1.0,
        keeps: Some(Place::Bakery),
        home: BAKERY,
        hours: &[
            (390.0, 1020.0, Inside(Place::Bakery)),
            (1020.0, 1260.0, Town),
        ],
        haunts: &[(35, 21), (33, 27)],
        loves: &[Item::Strawberry, Item::GoldenAcorn, Item::AncientGrain],
        likes: &[
            Item::Wheat,
            Item::Blueberry,
            Item::CarrotCake,
            Item::Corn,
            Item::SweetCherry,
            Item::CherryTart,
            Item::Cranberry,
        ],
        hates: &[Item::GhostPepper],
        hello: "Well aren't you a sight! Come here, you look half-starved. Honeycrumb \
                Bakery has fresh bread, cakes and treats - and every one of them gives \
                you a little something extra for a while. Eat up, dearie!",
        chat: &[
            "A warm muffin fixes almost anything.",
            "I bake bread before the rooster wakes. The rooster is very jealous.",
            "Bring me wheat and berries and I'll bake you something special.",
            "Food gives you a boost for a little while. Eat before a big delve!",
            "You're too thin, dear. Have a cookie. Have two.",
        ],
        close: &[
            "I save the biggest cinnamon roll for you every morning.",
            "You're family now, dear. Family gets the good jam.",
        ],
    },
    VillagerDef {
        name: "Barley",
        title: "Innkeeper",
        look: Look {
            beard: true,
            ..look(
                [CLAY, RUST, MAROON],
                SKIN,
                [CREAM, SAND, KHAKI],
                MAROON,
                Hair::Fluffy,
                1.12,
            )
        },
        wears: &[],
        holds: None,
        voice: 0.85,
        keeps: Some(Place::Tavern),
        home: TAVERN,
        hours: &[(480.0, 540.0, Town), (540.0, 1500.0, Inside(Place::Tavern))],
        haunts: &[(54, 34), (45, 34)],
        loves: &[Item::AncientGrain, Item::TruffleRisotto, Item::LavaLemonade],
        likes: &[
            Item::Wheat,
            Item::Corn,
            Item::Potato,
            Item::Garlic,
            Item::Pineapple,
            Item::GrapeJuice,
            Item::SweetPotato,
        ],
        hates: &[Item::WispDust],
        hello: "Welcome to the Sleepy Snail, friend! Best stew this side of the Hollow, \
                and the only place in town open past dark. Sit, eat, listen to the gossip. \
                Everybody ends up here sooner or later.",
        chat: &[
            "Heard Bramble's writing a song about you. Nothing rhymes with 'turnip'.",
            "The stew's been simmering since Tuesday. It's better that way.",
            "Evenings get lively in here. Come by after the shops close!",
            "If you hear anything interesting in the Hollow, tell old Barley.",
            "A hot meal and a warm fire. What else does anyone need?",
        ],
        close: &[
            "Your stool by the fire? Nobody else sits there. I made sure.",
            "You're good for this town, friend. Good for old Barley too.",
        ],
    },
    VillagerDef {
        name: "Grandma Fern",
        title: "Bramblewick's oldest",
        look: look(
            [WHITE, WHITE, SAND],
            SKIN,
            [BLUSH, PINK, PLUM],
            PLUM,
            Hair::Bun,
            0.8,
        ),
        wears: &[Item::WoollyPoncho],
        holds: None,
        voice: 0.95,
        keeps: None,
        home: FERN_HOME,
        hours: &[
            (480.0, 900.0, Town),
            (900.0, 1010.0, Inside(Place::Bakery)),
            (1010.0, 1140.0, Town),
        ],
        haunts: &[(6, 42), (9, 25), (12, 26), (15, 25)],
        loves: &[Item::ChippedTeacup, Item::PlumPudding, Item::SweetPea],
        likes: &[
            Item::Rose,
            Item::MintTea,
            Item::Blueberry,
            Item::PumpkinPie,
            Item::RhubarbCrumble,
            Item::PeachCobbler,
            Item::Lavender,
        ],
        hates: &[Item::ImpHorn],
        hello: "Oh my, what a sweet face. You remind me of my Albert, when he was young. \
                He used to go down that Hollow too, you know. Come and visit an old \
                woman sometime, dear. I have stories. So many stories.",
        chat: &[
            "When I was a girl, the fountain sang. Really sang!",
            "My knees tell me it'll rain tomorrow. My knees are usually wrong.",
            "Have you eaten? You should eat. Mabel makes lovely scones.",
            "Albert always said the Hollow gives back what it takes.",
            "The lamps used to glow all night. Such a pretty sight.",
        ],
        close: &[
            "You're the grandchild I always wished for. Don't tell Pip.",
            "Albert would have liked you very much, dear.",
        ],
    },
    VillagerDef {
        name: "Pip",
        title: "Adventurer in training",
        look: look(
            [GOLD, CLAY, RUST],
            TAN,
            [SKY, BLUE, INDIGO],
            INDIGO,
            Hair::Fluffy,
            0.72,
        ),
        wears: &[Item::BunnyHood, Item::BunnySlippers],
        holds: Some(Item::TwigSword),
        voice: 1.55,
        keeps: None,
        home: PIP_HOME,
        hours: &[
            (480.0, 720.0, Town),
            (720.0, 780.0, Inside(Place::Bakery)),
            (780.0, 1140.0, Town),
        ],
        haunts: &[(13, 42), (35, 29), (29, 26), (41, 26), (22, 30)],
        loves: &[Item::RubberDuck, Item::Popcorn, Item::SunflowerCookies],
        likes: &[
            Item::GlassMarble,
            Item::Strawberry,
            Item::ToyBoat,
            Item::BatWing,
            Item::CandyCorn,
            Item::PineappleCake,
            Item::FrostberrySorbet,
        ],
        hates: &[Item::Garlic, Item::Cabbage],
        hello: "WHOA! Are you a REAL adventurer?! Have you fought a slime? Have you \
                fought a KING slime?! I'm Pip! I'm going to be the greatest adventurer \
                ever! I already have a sword! ...It's a stick.",
        chat: &[
            "HIYAH! Take that, imaginary slime!",
            "When I grow up I'm going to floor one THOUSAND.",
            "Mom says I can't go in the Hollow until I'm taller than the fence.",
            "Can you teach me the roll thing? The dodge roll? PLEASE?",
            "I found a bug today. I named him Sir Crunch.",
        ],
        close: &[
            "You're my hero! Like, my actual hero!",
            "I told everyone at school you're my best friend. Is that ok?",
        ],
    },
    VillagerDef {
        name: "Juniper",
        title: "Botanist",
        look: look(
            [LIME, GREEN, TEAL],
            BROWN,
            [LIME, GREEN, TEAL],
            TEAL,
            Hair::Bob,
            0.97,
        ),
        wears: &[Item::LeafTunic, Item::LeafCap],
        holds: None,
        voice: 1.1,
        keeps: None,
        home: JUNIPER_HOME,
        hours: &[
            (480.0, 1140.0, Town),
            (1140.0, 1320.0, Inside(Place::Tavern)),
        ],
        haunts: &[(16, 10), (26, 12), (65, 30), (5, 24), (18, 25)],
        loves: &[Item::Truffle, Item::JellyShroom, Item::CrystalBerry],
        likes: &[
            Item::Glowcap,
            Item::Spore,
            Item::ShroomCap,
            Item::Puffball,
            Item::Snowdrop,
            Item::Cloudberry,
            Item::Chestnut,
        ],
        hates: &[Item::Popcorn],
        hello: "Oh, hello - careful, don't step on the moss, it's a rare strain. I'm \
                Juniper. I study the plants that grow in the Hollow. Did you know they \
                glow because they're lonely? That's my theory, anyway.",
        chat: &[
            "Fungi are neither plants nor animals. They're better.",
            "Every biome down there has its own seeds. Fascinating, isn't it?",
            "I'd love samples from the deep floors, if you ever find any.",
            "I talk to my mushrooms. They never interrupt.",
            "The Hollow's ecosystem is a perfect little mystery.",
        ],
        close: &[
            "I named a new species of moss after you. It's very soft.",
            "You're the only one who doesn't glaze over when I talk about spores.",
        ],
    },
    VillagerDef {
        name: "Bramble",
        title: "Travelling bard",
        look: look(
            [LAVENDER, PURPLE, GRAPE],
            SKIN,
            [ORANGE, RED, MAROON],
            PLUM,
            Hair::Spiky,
            1.0,
        ),
        wears: &[Item::CatHood],
        holds: None,
        voice: 1.15,
        keeps: None,
        home: BRAMBLE_HOME,
        hours: &[
            (600.0, 1080.0, Town),
            (1080.0, 1440.0, Inside(Place::Tavern)),
        ],
        haunts: &[(43, 10), (35, 29), (27, 28), (44, 25)],
        loves: &[Item::MusicBox, Item::Moonbloom, Item::LavaLemonade],
        likes: &[
            Item::Rose,
            Item::AncientCoin,
            Item::GlassMarble,
            Item::Starfruit,
            Item::Honeydew,
            Item::Raspberry,
        ],
        hates: &[Item::Cabbage],
        hello: "Ahh, a new verse walks into my song! I am Bramble, bard of the byways, \
                teller of tales! I seek a story worth singing. Perhaps... yours? A \
                farmer who delves the endless deep! Oh, that's GOOD.",
        chat: &[
            "Every hero needs a ballad. Yours is coming along nicely.",
            "I play the Sleepy Snail every evening. Come listen!",
            "What rhymes with 'Hollow'? Wallow? Swallow? ...Apollo?",
            "Inspiration is everywhere. Mostly at the bottom of a pie.",
            "A song is just a story that learned to dance.",
        ],
        close: &[
            "Your ballad has fourteen verses now. Verse nine is about your hat.",
            "You're my muse, friend. My very own muse.",
        ],
    },
    VillagerDef {
        name: "Toby",
        title: "Postman",
        look: look(
            [GOLD, CLAY, RUST],
            ROSY,
            [LIME, GREEN, TEAL],
            TEAL,
            Hair::Fluffy,
            0.95,
        ),
        wears: &[Item::FrogHood, Item::FrogRaincoat, Item::RainBoots],
        holds: None,
        voice: 1.2,
        keeps: None,
        home: TOBY_HOME,
        hours: &[
            (420.0, 1020.0, Town),
            (1020.0, 1140.0, Inside(Place::Tavern)),
        ],
        haunts: &[(27, 42), (8, 21), (34, 12), (54, 34), (16, 34), (44, 21)],
        loves: &[Item::MintTea, Item::RubberDuck, Item::SnowPea],
        likes: &[
            Item::FreshBread,
            Item::Radish,
            Item::LostButton,
            Item::Tomato,
            Item::SnowMelon,
            Item::Peach,
        ],
        hates: &[Item::Ectoplasm],
        hello: "Ribbit! Ha, sorry, habit. I'm Toby, I deliver the post! Rain or shine - \
                mostly rain, that's why the frog coat. If you ever need something carried \
                across town, I'm your frog. Er. Man.",
        chat: &[
            "Neither rain nor sleet nor slime shall stop the mail!",
            "Frogs are the most underrated animal. Discuss.",
            "I've walked every street in Bramblewick a thousand times.",
            "Letters are like little hugs you can post.",
            "My bag gets heavier every year. Or I get older.",
        ],
        close: &[
            "You're my favourite address to deliver to.",
            "I'd carry your letters for free. I do, actually. Shh.",
        ],
    },
    VillagerDef {
        name: "Sir Clank",
        title: "Retired knight",
        look: Look {
            beard: true,
            ..look(
                [WHITE, SAND, KHAKI],
                SKIN,
                [SKY, BLUE, SLATE],
                SLATE,
                Hair::Bald,
                1.05,
            )
        },
        wears: &[Item::FrostHelm, Item::FrostCoat, Item::FrostWalkers],
        holds: Some(Item::FrostFang),
        voice: 0.72,
        keeps: None,
        home: CLANK_HOME,
        hours: &[
            (480.0, 1020.0, Town),
            (1020.0, 1140.0, Inside(Place::Guild)),
            (1140.0, 1320.0, Inside(Place::Tavern)),
        ],
        haunts: &[(34, 42), (35, 34), (21, 30), (30, 24)],
        loves: &[Item::DragonScale, Item::FrostGem, Item::AncientCoin],
        likes: &[
            Item::IronOre,
            Item::Bone,
            Item::FrostMint,
            Item::VeggieStew,
            Item::Parsnip,
            Item::BrusselsSprouts,
        ],
        hates: &[Item::Sunflower],
        hello: "Halt! Who goes... oh, a farmer. Forgive me, old habits. Sir Clank, knight \
                of the Frost Guard, retired. I fought on floor fifty when it was just \
                floor fifty. Ahem. Carry on, carry on.",
        chat: &[
            "In my day we fought the Frost Colossus with a spoon. Uphill.",
            "Never turn your back on a skeleton. They're rude about it.",
            "My armour creaks more than I do. Almost.",
            "Chill slows your foes down. Frost magic is a knight's best friend.",
            "A knight's first duty: a good breakfast.",
        ],
        close: &[
            "You fight like my old squire. Better, honestly. Don't tell him.",
            "I'd follow you to floor one hundred, farmer. Slowly. With snacks.",
        ],
    },
    VillagerDef {
        name: "Mira",
        title: "Stargazer",
        look: look(
            [INDIGO, SLATE, SHADOW],
            TAN,
            [INDIGO, SLATE, SHADOW],
            INDIGO,
            Hair::Long,
            0.95,
        ),
        wears: &[Item::StarRobe],
        holds: None,
        voice: 1.05,
        keeps: None,
        home: MIRA_HOME,
        hours: &[
            (660.0, 960.0, Inside(Place::Scrolls)),
            (1170.0, 1500.0, Town),
        ],
        haunts: &[(42, 42), (65, 15), (35, 29), (66, 36)],
        loves: &[Item::StarDiamond, Item::Starfruit, Item::StarFossil],
        likes: &[
            Item::Moonbloom,
            Item::Moonstone,
            Item::WishStar,
            Item::WispDust,
            Item::SnowMelon,
            Item::Bluebell,
        ],
        hates: &[Item::EmberPepper],
        hello: "Mm? Oh... hello. Sorry, I'm usually asleep at this hour. I watch the \
                stars from the park at night. Did you know some stars fall into the \
                Hollow? I'd give anything to hold one.",
        chat: &[
            "The sky over Bramblewick is the clearest in the valley.",
            "Every wish star was once a real star. Isn't that sad and lovely?",
            "I sleep all day and count stars all night. It's a good life.",
            "The lamps going out was bad for the town, but good for stargazing.",
            "There's a constellation shaped like a turnip. I'll show you sometime.",
        ],
        close: &[
            "I found a new star last night. I named it after you.",
            "Come watch the sky with me sometime. It's quieter with a friend.",
        ],
    },
    VillagerDef {
        name: "Olive",
        title: "Town gardener",
        look: look(
            [GOLD, CLAY, RUST],
            ROSY,
            [GOLD, CLAY, RUST],
            RUST,
            Hair::Bob,
            0.98,
        ),
        wears: &[Item::StrawHat, Item::FarmerTunic, Item::RainBoots],
        holds: Some(Item::DuckCan),
        voice: 0.95,
        keeps: None,
        home: OLIVE_HOME,
        hours: &[
            (360.0, 900.0, Town),
            (900.0, 1010.0, Inside(Place::Seeds)),
            (1010.0, 1200.0, Town),
        ],
        haunts: &[(16, 32), (5, 23), (18, 23), (32, 23), (38, 29), (65, 24)],
        loves: &[Item::Sunflower, Item::Cabbage, Item::MossMelon],
        likes: &[
            Item::Turnip,
            Item::Potato,
            Item::Tomato,
            Item::SporePumpkin,
            Item::Zucchini,
            Item::Butternut,
            Item::Beetroot,
            Item::ButterLettuce,
        ],
        hates: &[Item::Bone],
        hello: "Morning! Mind the flower beds - well, what's left of them. I'm Olive, I \
                look after Bramblewick's green bits. The beds have been bare for years. \
                A fellow farmer! Maybe you can help me bring them back.",
        chat: &[
            "Weeds are just flowers nobody's cheering for.",
            "The park across the stream was lovely before the bridge fell.",
            "Good soil is patient soil.",
            "I water the planters every morning, even the empty ones. Just in case.",
            "Sunflowers always face the light. We could learn from them.",
        ],
        close: &[
            "Your farm's the best-kept patch in the valley. Don't tell the mayor I said so.",
            "I planted a row of your favourites by my door. For you.",
        ],
    },
    VillagerDef {
        name: "Hazel",
        title: "Spellwright",
        look: Look {
            eyes: GRAPE,
            cheeks: BLUSH,
            belt: GOLD,
            boots: GRAPE,
            ..look(
                [MINT, AQUA, TEAL],
                ROSY,
                [LAVENDER, PURPLE, GRAPE],
                GRAPE,
                Hair::Long,
                0.95,
            )
        },
        wears: &[Item::WizardHat, Item::StarryLeggings],
        holds: Some(Item::MoonpetalWand),
        voice: 1.2,
        keeps: Some(Place::Spellery),
        home: SPELLERY,
        hours: &[
            (480.0, 540.0, Town),
            (540.0, 1020.0, Inside(Place::Spellery)),
            (1020.0, 1420.0, Town),
        ],
        haunts: &[(54, 44), (65, 26), (66, 22), (58, 44)],
        loves: &[Item::Moonbloom, Item::WispDust, Item::LargeManaPotion],
        likes: &[
            Item::Heartleaf,
            Item::Blueberry,
            Item::Amethyst,
            Item::MintTea,
            Item::Glowcap,
            Item::Mistletoe,
            Item::Lavender,
        ],
        hates: &[Item::Garlic, Item::Bone],
        hello: "Oh! Careful, that cauldron bites. Welcome to the Starfall Spellery! I'm Hazel. \
                I teach spells - little pieces of starlight you carry with you. Every delver \
                should have one, so here: Firebolt, on the house! Press Q to throw it. You can \
                carry two spells at a time, and my attuning circle is the only place to swap \
                them. Oh, and I brew potions, if you ever need a pick-me-up.",
        chat: &[
            "The more you cast a spell, the better it knows you. And the cheaper it gets!",
            "Two spells at a time. Any more and they start arguing in your head.",
            "Heartleaf grows in the wild bushes on farms. It's lovely in a health potion.",
            "Two small potions brew into a medium one. Two medium into a large. Easy!",
            "Mana comes back on its own, slowly. Potions are for when you can't wait.",
            "Bloom is my favourite. Farmers love it. Crops love it more.",
        ],
        close: &[
            "You cast like you were born to it. I'm a little jealous, honestly.",
            "I named a star after you. It's a very small star, but it's yours.",
        ],
    },
];

impl Villager {
    pub fn def(self) -> &'static VillagerDef {
        &VILLAGER_DEFS[self as usize]
    }

    pub fn name(self) -> &'static str {
        self.def().name
    }

    /// Where they are at this time of day.
    pub fn spot(self, min: f32) -> Spot {
        self.def()
            .hours
            .iter()
            .find(|(a, b, _)| min >= *a && min < *b)
            .map_or(Spot::Away, |(_, _, s)| *s)
    }

    /// The building they keep, or live in.
    pub fn home_door(self) -> (i32, i32) {
        BUILDINGS[self.def().home].step()
    }

    /// How they feel about a gift.
    pub fn taste(self, item: Item) -> Taste {
        let d = self.def();
        if d.loves.contains(&item) {
            Taste::Love
        } else if d.likes.contains(&item) {
            Taste::Like
        } else if d.hates.contains(&item) {
            Taste::Hate
        } else {
            Taste::Fine
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Taste {
    Love,
    Like,
    Fine,
    Hate,
}

impl Taste {
    pub fn points(self) -> i32 {
        match self {
            Taste::Love => 80,
            Taste::Like => 45,
            Taste::Fine => 20,
            Taste::Hate => -30,
        }
    }

    pub fn reply(self, who: Villager, item: &str) -> String {
        let name = who.name();
        match self {
            Taste::Love => format!(
                "{}! Oh, I LOVE this! How did you know? Thank you, thank you!",
                item
            ),
            Taste::Like => format!("A {}? That's really thoughtful. Thank you!", item),
            Taste::Fine => format!("Oh, a {}. That's kind of you. Thanks!", item),
            Taste::Hate => {
                let _ = name;
                format!(
                    "...A {}. Um. I'll... find a place for it. Somewhere. Far away.",
                    item
                )
            }
        }
    }
}

/// How you get on with everyone, saved with the game.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Friends {
    /// Friendship points by villager (100 to a heart, ten hearts at most).
    pub points: Vec<u16>,
    /// Talked to today.
    pub talked: Vec<bool>,
    /// Given a gift today.
    pub gifted: Vec<bool>,
    /// Ever met.
    pub met: Vec<bool>,
    /// Heart milestones already rewarded (bits for 3, 6 and 9 hearts).
    pub presents: Vec<u8>,
}

impl Default for Friends {
    fn default() -> Self {
        Friends {
            points: vec![0; FOLK],
            talked: vec![false; FOLK],
            gifted: vec![false; FOLK],
            met: vec![false; FOLK],
            presents: vec![0; FOLK],
        }
    }
}

impl Friends {
    /// Pads out lists from older saves.
    pub fn fix(&mut self) {
        for v in [&mut self.talked, &mut self.gifted, &mut self.met] {
            v.resize(FOLK, false);
        }
        self.points.resize(FOLK, 0);
        self.presents.resize(FOLK, 0);
    }

    pub fn hearts(&self, v: Villager) -> u8 {
        (self.points[v as usize] / 100).min(10) as u8
    }

    /// Adds (or takes away) friendship. Returns true when a new heart fills.
    pub fn add(&mut self, v: Villager, n: i32) -> bool {
        let before = self.hearts(v);
        let p = &mut self.points[v as usize];
        *p = (*p as i32 + n).clamp(0, 1000) as u16;
        self.hearts(v) > before
    }

    pub fn new_day(&mut self) {
        self.talked.fill(false);
        self.gifted.fill(false);
    }
}

// ------------------------------------------------------------------------------------------
// Townsfolk out and about
// ------------------------------------------------------------------------------------------

/// A villager somewhere you can see them.
pub struct Npc {
    pub who: Villager,
    pub pos: Vec2,
    pub yaw: f32,
    pub walk: f32,
    pub stride: f32,
    /// Tiles still to walk through.
    pub path: VecDeque<(i32, i32)>,
    /// Seconds to stand about before moving on.
    pub wait: f32,
    /// Turned to face you while you talk.
    pub talking: bool,
    /// Heading out: vanish on reaching the end of the path.
    pub leaving: bool,
    /// Stays put (shopkeepers behind their counters).
    pub fixed: bool,
}

pub const NPC_RADIUS: f32 = 0.3;
const SPEED: f32 = 1.7;

impl Npc {
    fn new(who: Villager, x: i32, z: i32) -> Npc {
        Npc {
            who,
            pos: Vec2::new(x as f32 + 0.5, z as f32 + 0.5),
            yaw: 0.0,
            walk: 0.0,
            stride: 0.0,
            path: VecDeque::new(),
            wait: 0.0,
            talking: false,
            leaving: false,
            fixed: false,
        }
    }

    pub fn world_pos(&self) -> Vec3 {
        Vec3::new(self.pos.x, 0.0, self.pos.y)
    }

    pub fn tile(&self) -> (i32, i32) {
        (self.pos.x.floor() as i32, self.pos.y.floor() as i32)
    }
}

/// Breadth-first path over open tiles, from `a` to `b` (both included, `a` dropped).
pub fn find_path(w: &World, a: (i32, i32), b: (i32, i32)) -> Option<VecDeque<(i32, i32)>> {
    if !w.inside(b.0, b.1) || w.blocked(b.0, b.1) {
        return None;
    }
    let n = (w.w * w.h) as usize;
    let mut from = vec![u32::MAX; n];
    let start = w.idx(a.0, a.1);
    from[start] = start as u32;
    let mut q = VecDeque::new();
    q.push_back(a);
    while let Some((x, z)) = q.pop_front() {
        if (x, z) == b {
            let mut path = VecDeque::new();
            let mut i = w.idx(x, z);
            while i != start {
                path.push_front((i as i32 % w.w, i as i32 / w.w));
                i = from[i] as usize;
            }
            return Some(path);
        }
        for (dx, dz) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
            let (nx, nz) = (x + dx, z + dz);
            if !w.inside(nx, nz) || w.blocked(nx, nz) {
                continue;
            }
            let j = w.idx(nx, nz);
            if from[j] == u32::MAX {
                from[j] = w.idx(x, z) as u32;
                q.push_back((nx, nz));
            }
        }
    }
    None
}

impl Play {
    /// Who's here now, where the hero is.
    fn folk_here(&self) -> Vec<Villager> {
        let min = self.clock.min;
        VILLAGERS
            .iter()
            .copied()
            .filter(|v| match (self.area, v.spot(min)) {
                (Area::Town, Spot::Town) => true,
                (Area::Inside(p), Spot::Inside(q)) => p == q,
                _ => false,
            })
            .collect()
    }

    /// Puts the right people in the right places when you arrive somewhere.
    pub fn arrive_folk(&mut self) {
        self.folk.clear();
        let mut rng = Rng::new(self.seed ^ (self.clock.day as u64 * 131) ^ self.clock.min as u64);
        for v in self.folk_here() {
            if let Some(npc) = self.place_npc(v, &mut rng, false) {
                self.folk.push(npc);
            }
        }
    }

    /// A villager standing where they'd be, or walking out of a door if `door`.
    fn place_npc(&self, v: Villager, rng: &mut Rng, door: bool) -> Option<Npc> {
        let w = self.world();
        match self.area {
            Area::Inside(place) => {
                let room = self.room.as_ref()?;
                if v.def().keeps == Some(place) {
                    let (x, z) = room.keeper;
                    let mut n = Npc::new(v, x, z);
                    n.fixed = true;
                    return Some(n);
                }
                if door {
                    let (x, z) = room.exit;
                    let mut n = Npc::new(v, x, z - 1);
                    n.wait = 0.5;
                    return Some(n);
                }
                // A free spot somewhere in the room.
                for _ in 0..40 {
                    let x = rng.range(1, w.w - 1);
                    let z = rng.range(3, w.h - 2);
                    if !w.blocked(x, z) && self.folk.iter().all(|f| f.tile() != (x, z)) {
                        let mut n = Npc::new(v, x, z);
                        n.wait = rng.range_f(1.0, 4.0);
                        n.yaw = rng.range_f(-0.8, 0.8);
                        return Some(n);
                    }
                }
                None
            }
            Area::Town => {
                if door {
                    let (x, z) = self.door_of(v);
                    let mut n = Npc::new(v, x, z);
                    n.yaw = 0.0;
                    return Some(n);
                }
                let spots: Vec<(i32, i32)> = v
                    .def()
                    .haunts
                    .iter()
                    .copied()
                    .filter(|&(x, z)| !w.blocked(x, z))
                    .collect();
                let (x, z) = if spots.is_empty() {
                    TOWN.arrive
                } else {
                    spots[rng.below(spots.len())]
                };
                let (x, z) = w.nearest_open(x, z);
                let mut n = Npc::new(v, x, z);
                n.wait = rng.range_f(0.5, 5.0);
                n.yaw = rng.range_f(-1.0, 1.0);
                Some(n)
            }
            _ => None,
        }
    }

    /// The door a villager uses to come and go from town at this hour.
    fn door_of(&self, v: Villager) -> (i32, i32) {
        match v.spot(self.clock.min) {
            Spot::Inside(p) => BUILDINGS[p.building()].step(),
            _ => v.home_door(),
        }
    }

    /// Moves the townsfolk about. `active` is false while a menu or fade holds the world.
    pub fn update_folk(&mut self, dt: f32, active: bool) {
        if !self.in_town() {
            self.folk.clear();
            return;
        }
        let min = self.clock.min;
        // Arrivals and departures as the day goes on.
        if active {
            let here = self.folk_here();
            let mut rng = Rng::new(hash2(self.clock.day as i32, min as i32, 77) as u64);
            for v in here.iter().copied() {
                if !self.folk.iter().any(|n| n.who == v) {
                    if let Some(n) = self.place_npc(v, &mut rng, true) {
                        self.folk.push(n);
                    }
                }
            }
            let area = self.area;
            let room_exit = self.room.as_ref().map(|r| r.exit);
            let mut goals = Vec::new();
            for (i, n) in self.folk.iter().enumerate() {
                if !here.contains(&n.who) && !n.leaving {
                    let door = match area {
                        Area::Inside(_) => room_exit.map(|(x, z)| (x, z - 1)),
                        _ => Some(self.door_of(n.who)),
                    };
                    goals.push((i, door));
                }
            }
            for (i, door) in goals {
                let start = self.folk[i].tile();
                let path = door.and_then(|d| find_path(self.world(), start, d));
                let n = &mut self.folk[i];
                n.leaving = true;
                n.fixed = false;
                n.talking = false;
                n.wait = 0.0;
                n.path = path.unwrap_or_default();
            }
        }
        let ppos = self.player.pos;
        let world = super::travel::area_world(
            self.area,
            &self.farm,
            &self.town,
            &self.house.world,
            &self.level,
            &self.room,
        );
        let mut gone = Vec::new();
        let mut rng = Rng::new(hash2(self.clock.day as i32, (self.time * 10.0) as i32, 5) as u64);
        let count = self.folk.len();
        for i in 0..count {
            let n = &mut self.folk[i];
            let mut moving = false;
            if n.talking {
                let d = ppos - n.pos;
                let want = d.x.atan2(d.y);
                n.yaw += wrap_angle(want - n.yaw) * damp(10.0, dt);
            } else if n.fixed {
                n.yaw += wrap_angle(0.0 - n.yaw) * damp(4.0, dt);
            } else if active {
                if let Some(&(tx, tz)) = n.path.front() {
                    let target = Vec2::new(tx as f32 + 0.5, tz as f32 + 0.5);
                    let d = target - n.pos;
                    let near_player =
                        (ppos - (n.pos + d.normalize_or_zero() * 0.4)).length() < 0.55;
                    if near_player {
                        // Politely wait for you to pass.
                    } else if d.length() < 0.08 {
                        n.path.pop_front();
                    } else {
                        let step = d.normalize() * (SPEED * dt).min(d.length());
                        n.pos += step;
                        n.yaw += wrap_angle(step.x.atan2(step.y) - n.yaw) * damp(12.0, dt);
                        moving = true;
                    }
                } else if n.leaving {
                    gone.push(i);
                } else {
                    n.wait -= dt;
                    if n.wait <= 0.0 {
                        // Off to somewhere else they like.
                        n.wait = rng.range_f(3.0, 9.0);
                        let dest = match self.area {
                            Area::Town => {
                                let h = n.who.def().haunts;
                                let (x, z) =
                                    h[rng.below(h.len().max(1)).min(h.len().saturating_sub(1))];
                                let (ox, oz) = (rng.range(-2, 3), rng.range(-1, 2));
                                (x + ox, z + oz)
                            }
                            _ => {
                                let (x, z) = n.tile();
                                (x + rng.range(-3, 4), z + rng.range(-2, 3))
                            }
                        };
                        if let Some(p) = find_path(world, n.tile(), dest) {
                            if p.len() < 60 {
                                n.path = p;
                            }
                        }
                    }
                }
            }
            if moving {
                n.walk += dt * 10.0;
                n.stride = (n.stride + dt * 6.0).min(1.0);
            } else {
                n.stride = (n.stride - dt * 6.0).max(0.0);
            }
        }
        for i in gone.into_iter().rev() {
            self.folk.remove(i);
        }
        // Nobody walks through the hero.
        for n in &self.folk {
            let d = self.player.pos - n.pos;
            let min_d = NPC_RADIUS + super::player::RADIUS;
            let l = d.length();
            if l < min_d && l > 1e-4 {
                let push = d / l * (min_d - l);
                self.player.pos = world.move_circle(self.player.pos, push, super::player::RADIUS);
            }
        }
    }

    /// The villager in front of you, close enough to talk to.
    pub fn npc_near(&self) -> Option<usize> {
        let p = self.player.pos;
        let f = self.player.facing;
        let mut best: Option<(usize, f32)> = None;
        for (i, n) in self.folk.iter().enumerate() {
            if n.leaving {
                continue;
            }
            let d = n.pos - p;
            let dist = d.length();
            // Shopkeepers can be reached across the counter.
            let reach = if n.fixed { 2.3 } else { 1.4 };
            if dist > reach {
                continue;
            }
            let facing = d.normalize_or_zero().dot(f);
            let score = dist - facing * 0.6;
            if facing > -0.3 && best.is_none_or(|(_, s)| score < s) {
                best = Some((i, score));
            }
        }
        best.map(|(i, _)| i)
    }
}

/// What a villager looks like dressed: worn gear as stacks.
pub fn outfit(v: Villager) -> [Option<Stack>; 5] {
    let mut e: [Option<Stack>; 5] = [None; 5];
    for &it in v.def().wears {
        if let Some(slot) = it.class().and_then(|c| c.slot()) {
            e[slot as usize] = Some(Stack::new(it, 1));
        }
    }
    e
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::town;

    #[test]
    fn every_villager_is_somewhere_by_day() {
        for &v in VILLAGERS {
            let seen = (360..1500)
                .step_by(30)
                .any(|m| v.spot(m as f32) != Spot::Away);
            assert!(seen, "{} never leaves home", v.name());
            let d = v.def();
            assert!(!d.chat.is_empty() && !d.close.is_empty());
            assert!(!d.haunts.is_empty(), "{} has nowhere to go", d.name);
            if let Some(p) = d.keeps {
                // Keepers are in their shop whenever it's open.
                let (a, b) = p.def().open;
                for m in (a as i32..b as i32).step_by(20) {
                    assert_eq!(v.spot(m as f32), Spot::Inside(p), "{} wandered off", d.name);
                }
            }
        }
    }

    #[test]
    fn haunts_are_reachable_from_the_bus_stop() {
        let w = town::generate(u32::MAX);
        for &v in VILLAGERS {
            for &(x, z) in v.def().haunts {
                let (x, z) = w.nearest_open(x, z);
                assert!(
                    find_path(&w, town::TOWN.arrive, (x, z)).is_some(),
                    "{} can't reach ({x}, {z})",
                    v.name()
                );
            }
        }
    }

    #[test]
    fn friendship_counts_hearts() {
        let mut f = Friends::default();
        assert_eq!(f.hearts(Villager::Pip), 0);
        assert!(f.add(Villager::Pip, 120));
        assert_eq!(f.hearts(Villager::Pip), 1);
        assert!(!f.add(Villager::Pip, 10));
        f.add(Villager::Pip, 5000);
        assert_eq!(f.hearts(Villager::Pip), 10);
        f.add(Villager::Pip, -5000);
        assert_eq!(f.hearts(Villager::Pip), 0);
    }
}
