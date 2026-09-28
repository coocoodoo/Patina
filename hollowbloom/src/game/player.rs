//! The farmer-adventurer.

use glam::{Vec2, Vec3};
use serde::{Deserialize, Serialize};

use super::draw::Swing;
use super::gear::{Class, Sheet, Slot, Stat};
use super::items::{Buff, Inventory, Item, Stack};

pub const BAG: usize = 40;
pub const HOTBAR: usize = 10;
pub const RADIUS: f32 = 0.26;
pub const SPEED: f32 = 3.7;

/// What a swing, cast or tool use does when it lands.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ActKind {
    Slash,
    Bolt,
    Blast,
    Mine,
    Chop,
    Till,
    Water,
    Reap,
    /// A spell leaving your hands (see `magic.rs`).
    Cast,
}

impl ActKind {
    /// Seconds for the whole motion before speed bonuses.
    pub fn base_duration(self) -> f32 {
        match self {
            ActKind::Slash => 0.3,
            ActKind::Bolt => 0.34,
            ActKind::Blast => 0.55,
            ActKind::Mine | ActKind::Chop => 0.4,
            ActKind::Till => 0.38,
            ActKind::Water => 0.42,
            ActKind::Reap => 0.34,
            ActKind::Cast => 0.32,
        }
    }

    /// Fraction of the motion at which it connects.
    pub fn impact(self) -> f32 {
        match self {
            ActKind::Slash | ActKind::Reap => 0.3,
            ActKind::Bolt | ActKind::Cast => 0.35,
            ActKind::Water => 0.4,
            ActKind::Blast => 0.55,
            _ => 0.6,
        }
    }

    pub fn swing(self) -> Swing {
        match self {
            ActKind::Slash | ActKind::Reap => Swing::Slash,
            ActKind::Water => Swing::Pour,
            ActKind::Bolt | ActKind::Cast => Swing::Cast,
            ActKind::Blast => Swing::Raise,
            _ => Swing::Chop,
        }
    }

    pub fn for_class(c: Class) -> Option<ActKind> {
        Some(match c {
            Class::Sword => ActKind::Slash,
            Class::Wand => ActKind::Bolt,
            Class::Staff => ActKind::Blast,
            Class::Pickaxe => ActKind::Mine,
            Class::Axe => ActKind::Chop,
            Class::Hoe => ActKind::Till,
            Class::Can => ActKind::Water,
            Class::Sickle => ActKind::Reap,
            _ => return None,
        })
    }
}

pub struct Act {
    pub kind: ActKind,
    pub t: f32,
    pub dur: f32,
    pub fired: bool,
    pub tile: (i32, i32),
    pub dir: Vec2,
}

impl Act {
    pub fn new(kind: ActKind, dur: f32, tile: (i32, i32), dir: Vec2) -> Act {
        Act {
            kind,
            t: 0.0,
            dur: dur.max(0.08),
            fired: false,
            tile,
            dir,
        }
    }

    pub fn progress(&self) -> f32 {
        (self.t / self.dur).clamp(0.0, 1.0)
    }
}

/// A food buff that is wearing off.
#[derive(Clone, Copy, Debug)]
pub struct Active {
    pub buff: Buff,
    pub left: f32,
    pub from: Item,
}

pub struct Player {
    pub pos: Vec2,
    pub vel: Vec2,
    pub facing: Vec2,
    pub yaw: f32,
    pub hp: i32,
    /// Max HP before gear (levels and heart crystals raise it).
    pub base_hp: i32,
    pub energy: f32,
    pub base_energy: i32,
    pub mana: f32,
    pub base_mana: i32,
    pub level: u32,
    pub xp: u32,
    pub inv: Inventory,
    /// Worn armour: head, chest, legs, feet, shield.
    pub equip: [Option<Stack>; 5],
    pub buffs: Vec<Active>,
    /// Stats from everything worn, the item in hand and food, refreshed every frame.
    pub sheet: Sheet,
    pub sel: usize,
    pub water: u32,
    pub act: Option<Act>,
    pub walk: f32,
    pub stride: f32,
    pub hurt: f32,
    pub flash: f32,
    pub dodge: f32,
    pub dodge_cd: f32,
    pub dodge_dir: Vec2,
    pub step_t: f32,
    pub eat_t: f32,
    pub regen_acc: f32,
    pub no_mana_t: f32,
    /// Harm the Ward spell will still soak up, for how many more seconds, at what level.
    pub ward: f32,
    pub ward_t: f32,
    pub ward_lv: u8,
}

/// The part of the player that goes into a save file.
#[derive(Serialize, Deserialize)]
pub struct PlayerSave {
    pub hp: i32,
    /// Base max HP (the name is kept from older saves).
    pub max_hp: i32,
    pub energy: f32,
    pub max_energy: i32,
    #[serde(default)]
    pub mana: f32,
    #[serde(default)]
    pub max_mana: i32,
    pub level: u32,
    pub xp: u32,
    pub inv: Inventory,
    #[serde(default)]
    pub equip: Vec<Option<Stack>>,
    pub sel: usize,
    pub water: u32,
}

pub fn xp_needed(level: u32) -> u32 {
    20 + level * level * 8
}

pub const BASE_MANA: i32 = 30;

impl Player {
    pub fn new(pos: Vec2) -> Player {
        let mut inv = Inventory::new(BAG);
        let kit = [
            (Item::Sword0, 1),
            (Item::Hoe, 1),
            (Item::Can0, 1),
            (Item::Axe0, 1),
            (Item::Pick0, 1),
            (Item::TurnipSeeds, 12),
            (Item::CarrotSeeds, 4),
            (Item::Turnip, 3),
            (Item::Sickle, 1),
            (Item::TwigWand, 1),
        ];
        for (i, (item, n)) in kit.into_iter().enumerate() {
            inv.slots[i] = Some(Stack::new(item, n));
        }
        let mut equip: [Option<Stack>; 5] = [None; 5];
        equip[Slot::Head as usize] = Some(Stack::new(Item::StrawHat, 1));
        equip[Slot::Chest as usize] = Some(Stack::new(Item::FarmerTunic, 1));
        equip[Slot::Legs as usize] = Some(Stack::new(Item::PatchedTrousers, 1));
        equip[Slot::Feet as usize] = Some(Stack::new(Item::RainBoots, 1));
        let mut p = Player {
            pos,
            vel: Vec2::ZERO,
            facing: Vec2::new(0.0, 1.0),
            yaw: 0.0,
            hp: 60,
            base_hp: 60,
            energy: 100.0,
            base_energy: 100,
            mana: BASE_MANA as f32,
            base_mana: BASE_MANA,
            level: 1,
            xp: 0,
            inv,
            equip,
            buffs: Vec::new(),
            sheet: Sheet::default(),
            sel: 0,
            water: 0,
            act: None,
            walk: 0.0,
            stride: 0.0,
            hurt: 0.0,
            flash: 0.0,
            dodge: 0.0,
            dodge_cd: 0.0,
            dodge_dir: Vec2::ZERO,
            step_t: 0.0,
            eat_t: 0.0,
            regen_acc: 0.0,
            no_mana_t: 0.0,
            ward: 0.0,
            ward_t: 0.0,
            ward_lv: 0,
        };
        p.refresh();
        p.hp = p.max_hp();
        p.energy = p.max_energy() as f32;
        p.mana = p.max_mana() as f32;
        p.water = p.can_capacity();
        p
    }

    pub fn save(&self) -> PlayerSave {
        PlayerSave {
            hp: self.hp,
            max_hp: self.base_hp,
            energy: self.energy,
            max_energy: self.base_energy,
            mana: self.mana,
            max_mana: self.base_mana,
            level: self.level,
            xp: self.xp,
            inv: self.inv.clone(),
            equip: self.equip.to_vec(),
            sel: self.sel,
            water: self.water,
        }
    }

    pub fn load(s: PlayerSave, pos: Vec2) -> Player {
        let mut p = Player::new(pos);
        p.base_hp = s.max_hp.max(10);
        p.base_energy = s.max_energy.max(10);
        p.base_mana = if s.max_mana > 0 {
            s.max_mana
        } else {
            BASE_MANA
        };
        p.level = s.level.max(1);
        p.xp = s.xp;
        p.inv = s.inv;
        p.inv.slots.resize(BAG, None);
        p.inv.normalize();
        // Older saves have no equipment record: start them in the farming clothes.
        if !s.equip.is_empty() {
            p.equip = [None; 5];
            for (i, e) in s.equip.into_iter().take(5).enumerate() {
                p.equip[i] = e.filter(|st| {
                    st.item.class().and_then(|c| c.slot()).map(|sl| sl as usize) == Some(i)
                });
                if let Some(st) = &mut p.equip[i] {
                    st.normalize();
                }
            }
        }
        p.sel = s.sel.min(HOTBAR - 1);
        p.refresh();
        p.hp = s.hp.clamp(1, p.max_hp());
        p.energy = s.energy.min(p.max_energy() as f32);
        p.mana = if s.max_mana > 0 {
            s.mana.clamp(0.0, p.max_mana() as f32)
        } else {
            p.max_mana() as f32
        };
        p.water = s.water.min(p.can_capacity());
        p
    }

    pub fn world_pos(&self) -> Vec3 {
        Vec3::new(self.pos.x, 0.0, self.pos.y)
    }

    pub fn tile(&self) -> (i32, i32) {
        (self.pos.x.floor() as i32, self.pos.y.floor() as i32)
    }

    pub fn held(&self) -> Option<Item> {
        self.inv.slots[self.sel].map(|s| s.item)
    }

    pub fn held_stack(&self) -> Option<&Stack> {
        self.inv.slots[self.sel].as_ref()
    }

    pub fn held_class(&self) -> Option<Class> {
        self.held().and_then(|i| i.class())
    }

    pub fn worn(&self, slot: Slot) -> Option<&Stack> {
        self.equip[slot as usize].as_ref()
    }

    /// Adds up everything worn, the item in hand and food buffs, then keeps health, energy
    /// and mana inside their (possibly new) limits.
    pub fn refresh(&mut self) {
        let mut sheet = Sheet::default();
        for s in self.equip.iter().flatten() {
            s.add_stats(&mut sheet);
        }
        if let Some(h) = self.held_stack() {
            if h.item.class().is_some_and(|c| !c.is_armor()) {
                h.add_stats(&mut sheet);
            }
        }
        for b in &self.buffs {
            sheet.add(b.buff.stat, b.buff.val as i32);
        }
        self.sheet = sheet;
        self.hp = self.hp.min(self.max_hp());
        self.energy = self.energy.min(self.max_energy() as f32);
        self.mana = self.mana.min(self.max_mana() as f32);
    }

    pub fn stat(&self, s: Stat) -> i32 {
        self.sheet.get(s)
    }

    pub fn max_hp(&self) -> i32 {
        self.base_hp + self.stat(Stat::Vitality).max(0)
    }

    pub fn max_energy(&self) -> i32 {
        self.base_energy + self.stat(Stat::Stamina).max(0)
    }

    pub fn max_mana(&self) -> i32 {
        self.base_mana + self.level as i32 * 2 + self.stat(Stat::Wisdom).max(0)
    }

    /// Defense from every worn piece plus Defense stats.
    pub fn defense(&self) -> i32 {
        let worn: i32 = self
            .equip
            .iter()
            .flatten()
            .filter_map(|s| s.main_value())
            .sum();
        worn + self.stat(Stat::Defense)
    }

    /// Chance to block a hit, 0..1. A shield brings its own.
    pub fn block_chance(&self) -> f32 {
        let shield = self
            .worn(Slot::Shield)
            .map_or(0, |s| 8 + s.gear.map_or(0, |g| g.level as i32 / 6).min(12));
        (shield + self.stat(Stat::Block)).clamp(0, 60) as f32 / 100.0
    }

    pub fn dodge_chance(&self) -> f32 {
        self.sheet.frac(Stat::Dodge, 40)
    }

    pub fn crit_chance(&self) -> f32 {
        (5 + self.stat(Stat::Crit)).clamp(0, 75) as f32 / 100.0
    }

    pub fn crit_mult(&self) -> f32 {
        1.6 + self.stat(Stat::CritDmg).clamp(0, 300) as f32 / 100.0
    }

    /// Multiplier on how long swings, casts and tool uses take.
    pub fn haste(&self) -> f32 {
        1.0 / (1.0 + self.sheet.frac(Stat::Haste, 60))
    }

    pub fn move_speed(&self) -> f32 {
        SPEED * (1.0 + self.sheet.frac(Stat::Swift, 60))
    }

    /// Luck as used by loot rolls.
    pub fn luck(&self) -> f32 {
        self.sheet.frac(Stat::Luck, 150)
    }

    pub fn greed(&self) -> f32 {
        1.0 + self.sheet.frac(Stat::Greed, 300)
    }

    /// Base damage of a weapon in hand (or of bare hands and tools).
    pub fn weapon_damage(&self) -> i32 {
        let base = match self.held_stack() {
            Some(s) => match s.item.class() {
                Some(Class::Sword | Class::Wand | Class::Staff | Class::Sickle) => {
                    s.main_value().unwrap_or(4)
                }
                Some(Class::Axe | Class::Pickaxe) => 3 + s.gear.map_or(1, |g| g.level as i32) / 2,
                _ => 3,
            },
            None => 3,
        };
        base + self.stat(Stat::Damage).max(0) + (self.level as i32 - 1)
    }

    /// Mining or chopping power of the tool in hand.
    pub fn tool_power(&self) -> i32 {
        let base = self.held_stack().and_then(|s| match s.item.class() {
            Some(Class::Axe | Class::Pickaxe) => s.main_value(),
            _ => None,
        });
        base.unwrap_or(1) + self.stat(Stat::Power).max(0)
    }

    /// Extra tiles for hoes, cans and sickles.
    pub fn reach(&self) -> i32 {
        self.stat(Stat::Reach).clamp(0, 4)
    }

    /// Energy a tool use costs after thrift and the tool's level.
    pub fn tool_cost(&self, base: f32) -> f32 {
        let lvl = self
            .held_stack()
            .and_then(|s| s.gear)
            .map_or(1, |g| g.level) as f32;
        base * (1.0 - lvl.min(60.0) / 240.0) * (1.0 - self.sheet.frac(Stat::Frugal, 75))
    }

    pub fn mana_cost(&self, base: f32) -> f32 {
        base * (1.0 - self.sheet.frac(Stat::Focus, 60))
    }

    /// The watering can water supply: the biggest can in the bag.
    pub fn can_capacity(&self) -> u32 {
        self.inv
            .gear_of(Class::Can)
            .map(|s| {
                let mut sheet = Sheet::default();
                s.add_stats(&mut sheet);
                s.main_value().unwrap_or(20) + sheet.get(Stat::Capacity).max(0)
            })
            .max()
            .unwrap_or(20)
            .max(1) as u32
    }

    /// Adds experience; returns the number of levels gained.
    pub fn gain_xp(&mut self, xp: u32) -> u32 {
        self.xp += xp;
        let mut ups = 0;
        while self.xp >= xp_needed(self.level) {
            self.xp -= xp_needed(self.level);
            self.level += 1;
            self.base_hp += 6;
            self.hp = self.max_hp();
            self.mana = self.max_mana() as f32;
            ups += 1;
        }
        ups
    }

    pub fn invulnerable(&self) -> bool {
        self.hurt > 0.0 || self.dodge > 0.0
    }

    /// Eats a buff (the same stat refreshes instead of stacking).
    pub fn add_buff(&mut self, buff: Buff, from: Item) {
        if let Some(a) = self.buffs.iter_mut().find(|a| a.buff.stat == buff.stat) {
            a.buff = buff;
            a.left = buff.secs as f32;
            a.from = from;
        } else {
            self.buffs.push(Active {
                buff,
                left: buff.secs as f32,
                from,
            });
        }
    }

    /// Puts armour on from a bag slot, swapping with what was worn.
    pub fn equip_from(&mut self, bag_slot: usize) -> bool {
        let Some(s) = self.inv.slots[bag_slot] else {
            return false;
        };
        let Some(slot) = s.item.class().and_then(|c| c.slot()) else {
            return false;
        };
        self.inv.slots[bag_slot] = self.equip[slot as usize].take();
        self.equip[slot as usize] = Some(s);
        self.refresh();
        true
    }

    /// Takes armour off into the bag.
    pub fn unequip(&mut self, slot: Slot) -> bool {
        let Some(s) = self.equip[slot as usize] else {
            return false;
        };
        if self.inv.add_stack(s) == 0 {
            self.equip[slot as usize] = None;
            self.refresh();
            true
        } else {
            false
        }
    }
}
