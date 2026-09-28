//! Spells: taught by Hazel at the Starfall Spellery and cast with mana. You carry two at a
//! time (on Q and R), and only Hazel's attuning circle can change which two. Every cast is
//! practice: each level makes a spell stronger and a little cheaper to cast.

use serde::{Deserialize, Serialize};

use crate::palette::*;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, Serialize, Deserialize)]
pub enum Spell {
    Firebolt,
    FrostNova,
    ChainSpark,
    Starfall,
    Mend,
    Ward,
    Blink,
    Bloom,
}

pub const SPELLS: [Spell; 8] = [
    Spell::Firebolt,
    Spell::Mend,
    Spell::Bloom,
    Spell::FrostNova,
    Spell::Blink,
    Spell::ChainSpark,
    Spell::Ward,
    Spell::Starfall,
];

pub struct SpellDef {
    pub name: &'static str,
    pub icon: &'static str,
    /// Mana for a cast at level 1.
    pub cost: f32,
    /// Seconds before the slot is ready again.
    pub cooldown: f32,
    /// What Hazel charges to teach it, in copper.
    pub price: u64,
    /// How deep you must have been before she will teach it.
    pub depth: u32,
    /// Light, mid and dark colours of its magic.
    pub colors: [u8; 3],
    /// Its name's colour on parchment.
    pub ink: u8,
    /// Safe to cast in town and indoors.
    pub gentle: bool,
    pub desc: &'static str,
    /// What practice brings.
    pub grows: &'static str,
}

pub static SPELL_DEFS: [SpellDef; 8] = [
    SpellDef {
        name: "Firebolt",
        icon: "spell_firebolt",
        cost: 8.0,
        cooldown: 0.45,
        price: 400,
        depth: 0,
        colors: [CREAM, GOLD, ORANGE],
        ink: RUST,
        gentle: false,
        desc: "Hurls a ball of fire that sets foes burning.",
        grows: "Hotter and longer-burning. Splits in three at Lv 4, five at Lv 8.",
    },
    SpellDef {
        name: "Frost Nova",
        icon: "spell_frost_nova",
        cost: 14.0,
        cooldown: 3.0,
        price: 900,
        depth: 5,
        colors: [WHITE, SKY, BLUE],
        ink: BLUE,
        gentle: false,
        desc: "A ring of frost bursts out around you, hurting and slowing all it touches.",
        grows: "Wider and colder. Freezes foes solid from Lv 5.",
    },
    SpellDef {
        name: "Chain Spark",
        icon: "spell_chain_spark",
        cost: 12.0,
        cooldown: 1.2,
        price: 1600,
        depth: 12,
        colors: [WHITE, CREAM, GOLD],
        ink: CLAY,
        gentle: false,
        desc: "Lightning leaps from foe to foe.",
        grows: "Harder hitting, and it jumps to one more foe every two levels.",
    },
    SpellDef {
        name: "Starfall",
        icon: "spell_starfall",
        cost: 22.0,
        cooldown: 4.0,
        price: 3000,
        depth: 25,
        colors: [WHITE, CREAM, LAVENDER],
        ink: PURPLE,
        gentle: false,
        desc: "Stars rain down on the foes around you.",
        grows: "More stars with every level, and each one lands harder.",
    },
    SpellDef {
        name: "Mend",
        icon: "spell_mend",
        cost: 16.0,
        cooldown: 2.5,
        price: 450,
        depth: 0,
        colors: [WHITE, BLUSH, PINK],
        ink: CRIMSON,
        gentle: true,
        desc: "Soft light knits your wounds together.",
        grows: "Heals more at every level; from Lv 3 it also heals a share of your max HP.",
    },
    SpellDef {
        name: "Ward",
        icon: "spell_ward",
        cost: 18.0,
        cooldown: 6.0,
        price: 1800,
        depth: 16,
        colors: [WHITE, MINT, AQUA],
        ink: TEAL,
        gentle: true,
        desc: "A bubble of light that soaks up harm for a while.",
        grows: "Holds more and lasts longer. From Lv 6 it throws some of the harm back.",
    },
    SpellDef {
        name: "Blink",
        icon: "spell_blink",
        cost: 10.0,
        cooldown: 1.5,
        price: 1100,
        depth: 8,
        colors: [WHITE, LAVENDER, PURPLE],
        ink: GRAPE,
        gentle: true,
        desc: "Step through the air to a spot ahead, untouchable for a heartbeat.",
        grows: "Carries you further at every level.",
    },
    SpellDef {
        name: "Bloom",
        icon: "spell_bloom",
        cost: 12.0,
        cooldown: 3.0,
        price: 600,
        depth: 0,
        colors: [LIME, GREEN, PINK],
        ink: GREEN,
        gentle: true,
        desc: "Waters the crops around you and coaxes them to grow. In the Hollow, vines \
               tangle the foes near you.",
        grows: "Reaches further, with a better chance of a growth spurt.",
    },
];

impl Spell {
    pub fn def(self) -> &'static SpellDef {
        &SPELL_DEFS[self as usize]
    }
}

pub const MAX_LEVEL: u8 = 10;

/// Practice needed to go from `level` to the next (a cast is one, more if it hits things).
pub fn xp_to_next(level: u8) -> u32 {
    let l = level as u32;
    10 + l * l * 4
}

/// A spell you know, and how practised you are at it.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct Known {
    pub spell: Spell,
    pub level: u8,
    pub xp: u32,
}

impl Known {
    pub fn new(spell: Spell) -> Known {
        Known {
            spell,
            level: 1,
            xp: 0,
        }
    }

    /// How much stronger practice has made it: 1 at level 1, and 18% more every level.
    pub fn power(&self) -> f32 {
        1.0 + 0.18 * (self.level as f32 - 1.0)
    }

    /// Mana for one cast: 5% cheaper at every level.
    pub fn cost(&self) -> f32 {
        self.spell.def().cost * (1.0 - 0.05 * (self.level as f32 - 1.0))
    }

    /// Adds practice. Returns true when it goes up a level.
    pub fn practise(&mut self, xp: u32) -> bool {
        if self.level >= MAX_LEVEL {
            return false;
        }
        self.xp += xp;
        let mut up = false;
        while self.level < MAX_LEVEL && self.xp >= xp_to_next(self.level) {
            self.xp -= xp_to_next(self.level);
            self.level += 1;
            up = true;
        }
        if self.level >= MAX_LEVEL {
            self.xp = 0;
        }
        up
    }
}

/// Everything the hero knows of magic.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Spellbook {
    pub known: Vec<Known>,
    /// The two spells ready to hand, cast with Q and R.
    pub slots: [Option<Spell>; 2],
    /// Seconds until each slot is ready again.
    #[serde(skip)]
    pub cool: [f32; 2],
}

impl Spellbook {
    pub fn knows(&self, s: Spell) -> bool {
        self.known.iter().any(|k| k.spell == s)
    }

    pub fn get(&self, s: Spell) -> Option<&Known> {
        self.known.iter().find(|k| k.spell == s)
    }

    pub fn get_mut(&mut self, s: Spell) -> Option<&mut Known> {
        self.known.iter_mut().find(|k| k.spell == s)
    }

    /// Learns a spell, putting it in an empty slot if there is one.
    pub fn learn(&mut self, s: Spell) -> bool {
        if self.knows(s) {
            return false;
        }
        self.known.push(Known::new(s));
        if let Some(slot) = self.slots.iter_mut().find(|x| x.is_none()) {
            *slot = Some(s);
        }
        true
    }

    /// Readies a known spell in a slot at the attuning circle. A spell already in the other
    /// slot trades places with this one.
    pub fn attune(&mut self, slot: usize, s: Spell) -> bool {
        if slot > 1 || !self.knows(s) {
            return false;
        }
        if self.slots[1 - slot] == Some(s) {
            self.slots.swap(0, 1);
        } else {
            self.slots[slot] = Some(s);
        }
        self.cool = [0.0; 2];
        true
    }

    /// Takes away anything that no longer makes sense (older or edited saves).
    pub fn fix(&mut self) {
        let mut seen = Vec::new();
        self.known.retain(|k| {
            let fresh = !seen.contains(&k.spell);
            seen.push(k.spell);
            fresh
        });
        for k in &mut self.known {
            k.level = k.level.clamp(1, MAX_LEVEL);
        }
        for i in 0..2 {
            if let Some(s) = self.slots[i] {
                if !self.knows(s) || (i == 1 && self.slots[0] == Some(s)) {
                    self.slots[i] = None;
                }
            }
        }
    }

    pub fn tick(&mut self, dt: f32) {
        for c in &mut self.cool {
            *c = (*c - dt).max(0.0);
        }
    }

    /// The best level of any spell you know.
    pub fn best_level(&self) -> u8 {
        self.known.iter().map(|k| k.level).max().unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn practice_levels_up_and_cheapens_spells() {
        let mut k = Known::new(Spell::Firebolt);
        let c1 = k.cost();
        let p1 = k.power();
        let mut ups = 0;
        for _ in 0..5000 {
            if k.practise(1) {
                ups += 1;
            }
        }
        assert_eq!(k.level, MAX_LEVEL);
        assert_eq!(ups, MAX_LEVEL as u32 - 1);
        assert!(k.cost() < c1 * 0.6, "mana cost falls a little every level");
        assert!(k.power() > p1 * 2.5, "and the spell grows stronger");
        // Levels come steadily, not all at once.
        let mut fresh = Known::new(Spell::Mend);
        fresh.practise(xp_to_next(1) - 1);
        assert_eq!(fresh.level, 1);
        fresh.practise(1);
        assert_eq!(fresh.level, 2);
    }

    #[test]
    fn two_slots_only_and_swapping() {
        let mut b = Spellbook::default();
        assert!(b.learn(Spell::Firebolt));
        assert!(b.learn(Spell::Mend));
        assert!(b.learn(Spell::Bloom));
        assert!(!b.learn(Spell::Bloom), "already known");
        assert_eq!(b.slots, [Some(Spell::Firebolt), Some(Spell::Mend)]);
        assert!(b.attune(1, Spell::Bloom));
        assert_eq!(b.slots, [Some(Spell::Firebolt), Some(Spell::Bloom)]);
        // Choosing the spell from the other slot swaps them round.
        assert!(b.attune(0, Spell::Bloom));
        assert_eq!(b.slots, [Some(Spell::Bloom), Some(Spell::Firebolt)]);
        assert!(
            !b.attune(0, Spell::Starfall),
            "unknown spells can't be readied"
        );
        assert!(!b.attune(2, Spell::Mend));
    }
}
