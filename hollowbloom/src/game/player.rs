//! The farmer-adventurer.

use glam::{Vec2, Vec3};
use serde::{Deserialize, Serialize};

use super::draw::Swing;
use super::items::{Inventory, Item, ToolKind};

pub const BAG: usize = 40;
pub const HOTBAR: usize = 10;
pub const RADIUS: f32 = 0.26;
pub const SPEED: f32 = 3.7;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ActKind {
    Sword(u8),
    Pick(u8),
    Axe(u8),
    Hoe,
    Water,
}

impl ActKind {
    pub fn duration(self) -> f32 {
        match self {
            ActKind::Sword(t) => 0.3 - t as f32 * 0.012,
            ActKind::Pick(t) | ActKind::Axe(t) => 0.42 - t as f32 * 0.03,
            ActKind::Hoe => 0.38,
            ActKind::Water => 0.42,
        }
    }

    /// Fraction of the swing at which it connects.
    pub fn impact(self) -> f32 {
        match self {
            ActKind::Sword(_) => 0.3,
            ActKind::Water => 0.4,
            _ => 0.6,
        }
    }

    pub fn swing(self) -> Swing {
        match self {
            ActKind::Sword(_) => Swing::Slash,
            ActKind::Water => Swing::Pour,
            _ => Swing::Chop,
        }
    }
}

pub struct Act {
    pub kind: ActKind,
    pub t: f32,
    pub fired: bool,
    pub tile: (i32, i32),
    pub dir: Vec2,
}

impl Act {
    pub fn progress(&self) -> f32 {
        (self.t / self.kind.duration()).clamp(0.0, 1.0)
    }
}

pub struct Player {
    pub pos: Vec2,
    pub vel: Vec2,
    pub facing: Vec2,
    pub yaw: f32,
    pub hp: i32,
    pub max_hp: i32,
    pub energy: f32,
    pub max_energy: i32,
    pub level: u32,
    pub xp: u32,
    pub inv: Inventory,
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
}

/// The part of the player that goes into a save file.
#[derive(Serialize, Deserialize)]
pub struct PlayerSave {
    pub hp: i32,
    pub max_hp: i32,
    pub energy: f32,
    pub max_energy: i32,
    pub level: u32,
    pub xp: u32,
    pub inv: Inventory,
    pub sel: usize,
    pub water: u32,
}

pub fn xp_needed(level: u32) -> u32 {
    20 + level * level * 8
}

impl Player {
    pub fn new(pos: Vec2) -> Player {
        let mut inv = Inventory::new(BAG);
        for (i, item) in [
            Item::Sword0,
            Item::Hoe,
            Item::Can0,
            Item::Axe0,
            Item::Pick0,
            Item::TurnipSeeds,
        ]
        .into_iter()
        .enumerate()
        {
            inv.slots[i] = Some(super::items::Stack::new(
                item,
                if item == Item::TurnipSeeds { 12 } else { 1 },
            ));
        }
        inv.slots[6] = Some(super::items::Stack::new(Item::CarrotSeeds, 4));
        inv.slots[7] = Some(super::items::Stack::new(Item::Turnip, 3));
        Player {
            pos,
            vel: Vec2::ZERO,
            facing: Vec2::new(0.0, 1.0),
            yaw: 0.0,
            hp: 60,
            max_hp: 60,
            energy: 100.0,
            max_energy: 100,
            level: 1,
            xp: 0,
            inv,
            sel: 0,
            water: 25,
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
        }
    }

    pub fn save(&self) -> PlayerSave {
        PlayerSave {
            hp: self.hp,
            max_hp: self.max_hp,
            energy: self.energy,
            max_energy: self.max_energy,
            level: self.level,
            xp: self.xp,
            inv: self.inv.clone(),
            sel: self.sel,
            water: self.water,
        }
    }

    pub fn load(s: PlayerSave, pos: Vec2) -> Player {
        let mut p = Player::new(pos);
        p.hp = s.hp.max(1);
        p.max_hp = s.max_hp.max(10);
        p.energy = s.energy;
        p.max_energy = s.max_energy.max(10);
        p.level = s.level.max(1);
        p.xp = s.xp;
        p.inv = s.inv;
        p.inv.slots.resize(BAG, None);
        p.sel = s.sel.min(HOTBAR - 1);
        p.water = s.water;
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

    pub fn can_capacity(&self) -> u32 {
        match self.inv.best_tool(ToolKind::Can) {
            Some(2) => 100,
            Some(1) => 50,
            _ => 25,
        }
    }

    pub fn sword_damage(&self, tier: u8) -> i32 {
        [6, 11, 18, 28, 42, 60][tier.min(5) as usize] + self.level as i32 - 1
    }

    /// Adds experience; returns the number of levels gained.
    pub fn gain_xp(&mut self, xp: u32) -> u32 {
        self.xp += xp;
        let mut ups = 0;
        while self.xp >= xp_needed(self.level) {
            self.xp -= xp_needed(self.level);
            self.level += 1;
            self.max_hp += 6;
            self.hp = self.max_hp;
            ups += 1;
        }
        ups
    }

    pub fn invulnerable(&self) -> bool {
        self.hurt > 0.0 || self.dodge > 0.0
    }
}
