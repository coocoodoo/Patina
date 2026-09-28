//! Items, crops, recipes and inventories.

use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ToolKind {
    Sword,
    Pickaxe,
    Axe,
    Hoe,
    Can,
}

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
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Kind {
    Material,
    Seed(Crop),
    Produce { hp: i32, energy: i32 },
    Food { hp: i32, energy: i32 },
    Tool(ToolKind, u8),
    Place(Placeable),
    Feather,
    HeartCrystal,
    SunStone,
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

use Kind::*;
use Placeable as P;
use ToolKind as T;

items! {
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

    TurnipSeeds = "turnip_seeds", "Turnip Seeds", "turnip_seeds", 99, 10, Seed(Crop::Turnip), "Plant on tilled soil. Grows in 4 days.";
    CarrotSeeds = "carrot_seeds", "Cave Carrot Seeds", "carrot_seeds", 99, 15, Seed(Crop::CaveCarrot), "A hardy root from the Hollow. 3 days.";
    GlowcapSpores = "glowcap_spores", "Glowcap Spores", "glowcap_spores", 99, 25, Seed(Crop::Glowcap), "Glows at night once grown. 5 days.";
    BerrySeeds = "berry_seeds", "Crystal Berry Seeds", "berry_seeds", 99, 35, Seed(Crop::CrystalBerry), "6 days, then fruits every 3 days.";
    MelonSeeds = "melon_seeds", "Moss Melon Seeds", "melon_seeds", 99, 40, Seed(Crop::MossMelon), "Slow and sweet. 8 days.";
    PumpkinSeeds = "pumpkin_seeds", "Spore Pumpkin Seeds", "pumpkin_seeds", 99, 50, Seed(Crop::SporePumpkin), "A fungal pumpkin. 9 days.";
    PepperSeeds = "pepper_seeds", "Ember Pepper Seeds", "pepper_seeds", 99, 45, Seed(Crop::EmberPepper), "5 days, then fruits every 2 days.";
    LilyBulb = "lily_bulb", "Frost Lily Bulb", "lily_bulb", 99, 60, Seed(Crop::FrostLily), "A cold, lovely flower. 7 days.";
    MoonbloomSeeds = "moonbloom_seeds", "Moonbloom Seeds", "moonbloom_seeds", 99, 120, Seed(Crop::Moonbloom), "Legendary. Glows. 12 days.";

    Turnip = "turnip", "Turnip", "turnip", 99, 35, Produce { hp: 8, energy: 20 }, "Crisp and humble.";
    CaveCarrot = "cave_carrot", "Cave Carrot", "cave_carrot", 99, 30, Produce { hp: 14, energy: 14 }, "Earthy and sweet.";
    Glowcap = "glowcap", "Glowcap", "glowcap", 99, 60, Produce { hp: 10, energy: 24 }, "A luminous mushroom.";
    CrystalBerry = "crystal_berry", "Crystal Berry", "crystal_berry", 99, 45, Produce { hp: 16, energy: 10 }, "Crunchy, cold, and faintly fizzy.";
    MossMelon = "moss_melon", "Moss Melon", "moss_melon", 99, 170, Produce { hp: 30, energy: 40 }, "Heavy with juice.";
    SporePumpkin = "spore_pumpkin", "Spore Pumpkin", "spore_pumpkin", 99, 220, Produce { hp: 36, energy: 50 }, "Soft pink flesh. Smells like rain.";
    EmberPepper = "ember_pepper", "Ember Pepper", "ember_pepper", 99, 80, Produce { hp: 22, energy: 18 }, "Warms you from the inside.";
    FrostLily = "frost_lily", "Frost Lily", "frost_lily", 99, 140, Produce { hp: 5, energy: 30 }, "Petals like snow.";
    Moonbloom = "moonbloom", "Moonbloom", "moonbloom", 99, 420, Produce { hp: 40, energy: 60 }, "Blooms only for the patient.";

    VeggieStew = "veggie_stew", "Veggie Stew", "veggie_stew", 20, 120, Food { hp: 50, energy: 50 }, "Hearty. Tastes like home.";
    GlowSoup = "glow_soup", "Glowcap Soup", "glow_soup", 20, 170, Food { hp: 60, energy: 60 }, "It glows. Just a little.";
    EmberCurry = "ember_curry", "Ember Curry", "ember_curry", 20, 260, Food { hp: 90, energy: 70 }, "Spicy enough to light a torch.";
    BerryTart = "berry_tart", "Berry Tart", "berry_tart", 20, 200, Food { hp: 70, energy: 40 }, "Flaky crust, jewel-bright berries.";
    PumpkinPie = "pumpkin_pie", "Pumpkin Pie", "pumpkin_pie", 20, 520, Food { hp: 120, energy: 100 }, "A feast in a slice.";
    HealingTonic = "healing_tonic", "Healing Tonic", "healing_tonic", 20, 90, Food { hp: 80, energy: 0 }, "Restores 80 HP.";
    StaminaTonic = "stamina_tonic", "Stamina Tonic", "stamina_tonic", 20, 90, Food { hp: 0, energy: 80 }, "Restores 80 energy.";

    Sword0 = "sword0", "Rusty Sword", "sword0", 1, 10, Tool(T::Sword, 0), "Old, but it remembers how to fight.";
    Sword1 = "sword1", "Copper Sword", "sword1", 1, 80, Tool(T::Sword, 1), "A bright, honest blade.";
    Sword2 = "sword2", "Iron Sword", "sword2", 1, 200, Tool(T::Sword, 2), "Heavy and dependable.";
    Sword3 = "sword3", "Golden Sword", "sword3", 1, 450, Tool(T::Sword, 3), "Shines like a sunrise.";
    Sword4 = "sword4", "Crystal Blade", "sword4", 1, 900, Tool(T::Sword, 4), "Sings when it strikes.";
    Sword5 = "sword5", "Ember Blade", "sword5", 1, 1600, Tool(T::Sword, 5), "Forged in the deep fire.";
    Pick0 = "pick0", "Rusty Pickaxe", "pick0", 1, 10, Tool(T::Pickaxe, 0), "Breaks rocks, walls and placed things.";
    Pick1 = "pick1", "Copper Pickaxe", "pick1", 1, 80, Tool(T::Pickaxe, 1), "Mines faster than rust.";
    Pick2 = "pick2", "Iron Pickaxe", "pick2", 1, 200, Tool(T::Pickaxe, 2), "Makes short work of stone.";
    Pick3 = "pick3", "Golden Pickaxe", "pick3", 1, 450, Tool(T::Pickaxe, 3), "Fast and shiny.";
    Pick4 = "pick4", "Crystal Pickaxe", "pick4", 1, 900, Tool(T::Pickaxe, 4), "Cuts rock like butter.";
    Pick5 = "pick5", "Ember Pickaxe", "pick5", 1, 1600, Tool(T::Pickaxe, 5), "Nothing is too hard.";
    Axe0 = "axe0", "Rusty Axe", "axe0", 1, 10, Tool(T::Axe, 0), "Chops trees, stumps and logs.";
    Axe1 = "axe1", "Copper Axe", "axe1", 1, 80, Tool(T::Axe, 1), "Chops a little faster.";
    Axe2 = "axe2", "Iron Axe", "axe2", 1, 200, Tool(T::Axe, 2), "Chops much faster.";
    Axe3 = "axe3", "Golden Axe", "axe3", 1, 450, Tool(T::Axe, 3), "Timber!";
    Axe4 = "axe4", "Crystal Axe", "axe4", 1, 900, Tool(T::Axe, 4), "Trees fall politely.";
    Axe5 = "axe5", "Ember Axe", "axe5", 1, 1600, Tool(T::Axe, 5), "One swing, usually.";
    Hoe = "hoe", "Hoe", "hoe", 1, 10, Tool(T::Hoe, 0), "Tills grass and soil for planting.";
    Can0 = "can0", "Watering Can", "can0", 1, 10, Tool(T::Can, 0), "Holds 25 water. Refill at the pond.";
    Can1 = "can1", "Copper Can", "can1", 1, 120, Tool(T::Can, 1), "Holds 50 water.";
    Can2 = "can2", "Crystal Can", "can2", 1, 600, Tool(T::Can, 2), "Holds 100 water.";

    Chest = "chest", "Chest", "chest", 20, 30, Place(P::Chest), "Stores 30 stacks. Place anywhere.";
    Torch = "torch", "Torch", "torch", 99, 5, Place(P::Torch), "A warm light for dark places.";
    Lamp = "lamp", "Glowcap Lamp", "lamp", 20, 60, Place(P::Lamp), "A soft, cozy glow for your farm.";
    Fence = "fence", "Fence", "fence", 99, 3, Place(P::Fence), "Keeps paths tidy. Joins its neighbours.";
    WoodPath = "wood_path", "Wood Floor", "wood_path", 99, 2, Place(P::WoodPath), "Lay planks on the ground.";
    StonePath = "stone_path", "Stone Path", "stone_path", 99, 2, Place(P::StonePath), "A cobbled path.";
    Sprinkler = "sprinkler", "Sprinkler", "sprinkler", 20, 60, Place(P::Sprinkler(0)), "Waters the 4 tiles next to it every morning.";
    QualitySprinkler = "quality_sprinkler", "Quality Sprinkler", "quality_sprinkler", 20, 150, Place(P::Sprinkler(1)), "Waters the 8 tiles around it every morning.";
    CrystalSprinkler = "crystal_sprinkler", "Crystal Sprinkler", "crystal_sprinkler", 20, 400, Place(P::Sprinkler(2)), "Waters a 5x5 square every morning.";
    StoneWall = "stone_wall", "Stone Wall", "stone_wall", 99, 4, Place(P::StoneWall), "A solid block. Build anything.";
    WoodWall = "wood_wall", "Wood Wall", "wood_wall", 99, 4, Place(P::WoodWall), "A solid block of planks.";
    Workbench = "workbench", "Workbench", "workbench", 5, 40, Place(P::Workbench), "Opens the crafting menu. Pretty, too.";
    FlowerPot = "flower_pot", "Flower Pot", "flower_pot", 20, 20, Place(P::FlowerPot), "A little colour for the porch.";
    Bench = "bench", "Bench", "bench", 20, 30, Place(P::Bench), "Sit a while.";

    Feather = "feather", "Homeward Feather", "feather", 20, 150, Feather, "Use in the Hollow to float back home.";
    HeartCrystal = "heart_crystal", "Heart Crystal", "heart_crystal", 20, 500, HeartCrystal, "Use to raise max HP by 10.";
    SunStone = "sun_stone", "Sun Stone", "stamina_gem", 20, 500, SunStone, "Use to raise max energy by 15.";
}

impl Item {
    pub fn def(self) -> &'static ItemDef {
        &ITEMS[self as usize]
    }

    pub fn from_key(key: &str) -> Option<Item> {
        ALL_ITEMS.iter().copied().find(|i| i.def().key == key)
    }

    pub fn tool(self) -> Option<(ToolKind, u8)> {
        match self.def().kind {
            Kind::Tool(k, t) => Some((k, t)),
            _ => None,
        }
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
}

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
            MossMelon => d(8, 0, Item::MossMelon, "young", false, 1),
            SporePumpkin => d(9, 0, Item::SporePumpkin, "young_fungal", false, 1),
            EmberPepper => d(5, 2, Item::EmberPepper, "young", false, 2),
            FrostLily => d(7, 0, Item::FrostLily, "young_frost", false, 1),
            Moonbloom => d(12, 0, Item::Moonbloom, "young_fungal", true, 1),
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

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Stack {
    pub item: Item,
    pub n: u16,
}

impl Stack {
    pub fn new(item: Item, n: u16) -> Self {
        Stack { item, n }
    }
}

#[derive(Serialize, Deserialize)]
struct StackRepr {
    item: String,
    n: u16,
}

impl Serialize for Stack {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        StackRepr {
            item: self.item.def().key.to_string(),
            n: self.n,
        }
        .serialize(s)
    }
}

impl<'de> Deserialize<'de> for Stack {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let r = StackRepr::deserialize(d)?;
        let item = Item::from_key(&r.item)
            .ok_or_else(|| serde::de::Error::custom(format!("unknown item {}", r.item)))?;
        Ok(Stack { item, n: r.n })
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
    pub fn add(&mut self, item: Item, mut n: u16) -> u16 {
        let max = item.def().stack;
        for s in self.slots.iter_mut().flatten() {
            if s.item == item && s.n < max {
                let k = (max - s.n).min(n);
                s.n += k;
                n -= k;
                if n == 0 {
                    return 0;
                }
            }
        }
        for s in self.slots.iter_mut() {
            if s.is_none() {
                let k = max.min(n);
                *s = Some(Stack::new(item, k));
                n -= k;
                if n == 0 {
                    return 0;
                }
            }
        }
        n
    }

    pub fn can_fit(&self, item: Item, n: u16) -> bool {
        let max = item.def().stack;
        let mut room = 0u32;
        for s in &self.slots {
            match s {
                None => room += max as u32,
                Some(s) if s.item == item => room += (max - s.n.min(max)) as u32,
                _ => {}
            }
        }
        room >= n as u32
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

    pub fn best_tool(&self, kind: ToolKind) -> Option<u8> {
        self.slots
            .iter()
            .flatten()
            .filter_map(|s| s.item.tool())
            .filter(|(k, _)| *k == kind)
            .map(|(_, t)| t)
            .max()
    }
}

// ------------------------------------------------------------------------------------------
// Recipes
// ------------------------------------------------------------------------------------------

pub struct Recipe {
    pub out: Item,
    pub n: u16,
    pub needs: &'static [(Item, u16)],
}

use Item as I;

pub static RECIPES: &[Recipe] = &[
    Recipe {
        out: I::Torch,
        n: 4,
        needs: &[(I::Wood, 2), (I::SlimeGel, 1)],
    },
    Recipe {
        out: I::Chest,
        n: 1,
        needs: &[(I::Wood, 20)],
    },
    Recipe {
        out: I::Fence,
        n: 4,
        needs: &[(I::Wood, 4)],
    },
    Recipe {
        out: I::WoodPath,
        n: 4,
        needs: &[(I::Wood, 2)],
    },
    Recipe {
        out: I::StonePath,
        n: 4,
        needs: &[(I::Stone, 2)],
    },
    Recipe {
        out: I::StoneWall,
        n: 2,
        needs: &[(I::Stone, 4)],
    },
    Recipe {
        out: I::WoodWall,
        n: 2,
        needs: &[(I::Wood, 4)],
    },
    Recipe {
        out: I::Lamp,
        n: 1,
        needs: &[(I::Stone, 6), (I::Glowcap, 1)],
    },
    Recipe {
        out: I::FlowerPot,
        n: 1,
        needs: &[(I::Stone, 4), (I::Fiber, 2)],
    },
    Recipe {
        out: I::Bench,
        n: 1,
        needs: &[(I::Wood, 10)],
    },
    Recipe {
        out: I::Workbench,
        n: 1,
        needs: &[(I::Wood, 15), (I::Stone, 5)],
    },
    Recipe {
        out: I::Sprinkler,
        n: 1,
        needs: &[(I::CopperOre, 5), (I::IronOre, 2)],
    },
    Recipe {
        out: I::QualitySprinkler,
        n: 1,
        needs: &[(I::IronOre, 5), (I::GoldOre, 2), (I::Crystal, 1)],
    },
    Recipe {
        out: I::CrystalSprinkler,
        n: 1,
        needs: &[(I::GoldOre, 5), (I::Crystal, 4), (I::FrostGem, 1)],
    },
    Recipe {
        out: I::HealingTonic,
        n: 1,
        needs: &[(I::CaveCarrot, 2), (I::SlimeGel, 2)],
    },
    Recipe {
        out: I::StaminaTonic,
        n: 1,
        needs: &[(I::Turnip, 2), (I::Spore, 1)],
    },
    Recipe {
        out: I::VeggieStew,
        n: 1,
        needs: &[(I::Turnip, 1), (I::CaveCarrot, 1)],
    },
    Recipe {
        out: I::GlowSoup,
        n: 1,
        needs: &[(I::Glowcap, 2), (I::Turnip, 1)],
    },
    Recipe {
        out: I::BerryTart,
        n: 1,
        needs: &[(I::CrystalBerry, 3), (I::SlimeGel, 1)],
    },
    Recipe {
        out: I::EmberCurry,
        n: 1,
        needs: &[(I::EmberPepper, 1), (I::CaveCarrot, 1), (I::Turnip, 1)],
    },
    Recipe {
        out: I::PumpkinPie,
        n: 1,
        needs: &[(I::SporePumpkin, 1), (I::CrystalBerry, 2)],
    },
    Recipe {
        out: I::Sword1,
        n: 1,
        needs: &[(I::CopperOre, 10), (I::Wood, 5)],
    },
    Recipe {
        out: I::Pick1,
        n: 1,
        needs: &[(I::CopperOre, 8), (I::Wood, 5)],
    },
    Recipe {
        out: I::Axe1,
        n: 1,
        needs: &[(I::CopperOre, 8), (I::Wood, 5)],
    },
    Recipe {
        out: I::Can1,
        n: 1,
        needs: &[(I::CopperOre, 10)],
    },
    Recipe {
        out: I::Sword2,
        n: 1,
        needs: &[(I::IronOre, 12), (I::CopperOre, 5)],
    },
    Recipe {
        out: I::Pick2,
        n: 1,
        needs: &[(I::IronOre, 10), (I::CopperOre, 4)],
    },
    Recipe {
        out: I::Axe2,
        n: 1,
        needs: &[(I::IronOre, 10), (I::CopperOre, 4)],
    },
    Recipe {
        out: I::Sword3,
        n: 1,
        needs: &[(I::GoldOre, 14), (I::IronOre, 6)],
    },
    Recipe {
        out: I::Pick3,
        n: 1,
        needs: &[(I::GoldOre, 12), (I::IronOre, 5)],
    },
    Recipe {
        out: I::Axe3,
        n: 1,
        needs: &[(I::GoldOre, 12), (I::IronOre, 5)],
    },
    Recipe {
        out: I::Sword4,
        n: 1,
        needs: &[(I::Crystal, 14), (I::GoldOre, 6)],
    },
    Recipe {
        out: I::Pick4,
        n: 1,
        needs: &[(I::Crystal, 12), (I::GoldOre, 5)],
    },
    Recipe {
        out: I::Axe4,
        n: 1,
        needs: &[(I::Crystal, 12), (I::GoldOre, 5)],
    },
    Recipe {
        out: I::Can2,
        n: 1,
        needs: &[(I::Crystal, 6), (I::GoldOre, 4)],
    },
    Recipe {
        out: I::Sword5,
        n: 1,
        needs: &[(I::EmberOre, 16), (I::Crystal, 8)],
    },
    Recipe {
        out: I::Pick5,
        n: 1,
        needs: &[(I::EmberOre, 14), (I::Crystal, 6)],
    },
    Recipe {
        out: I::Axe5,
        n: 1,
        needs: &[(I::EmberOre, 14), (I::Crystal, 6)],
    },
    Recipe {
        out: I::HeartCrystal,
        n: 1,
        needs: &[(I::Amber, 3), (I::FrostGem, 2), (I::Moonbloom, 1)],
    },
];

impl Recipe {
    pub fn can_craft(&self, inv: &Inventory) -> bool {
        self.needs.iter().all(|(i, n)| inv.count(*i) >= *n as u32) && inv.can_fit(self.out, self.n)
    }
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
    fn crop_stages() {
        assert_eq!(Crop::Turnip.stage(0), 0);
        assert_eq!(Crop::Turnip.stage(4), 3);
        assert!(Crop::Moonbloom.stage(11) < 3);
    }

    #[test]
    fn stack_serde() {
        let s = Stack::new(Item::Glowcap, 7);
        let j = serde_json::to_string(&s).unwrap();
        let back: Stack = serde_json::from_str(&j).unwrap();
        assert_eq!(back, s);
    }
}
