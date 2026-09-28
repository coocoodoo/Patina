//! Icons for the town: the bus sign, quest keepsakes, speech bubbles and hearts.
//! Same ASCII legend as `sprites`, with recolour slots '0'-'3'.

use std::collections::HashMap;

use super::sprites::{NO, art};
use crate::palette::*;
use crate::render::{TexBank, TexId, Texture};

const BUS: &[&str] = &[
    "................",
    "....KKKKKKKK....",
    "...KwwyyyyyyK...",
    "..KyyyyyyyyyyK..",
    "..KSSSSKKSSSSK..",
    "..KSwSSKKSwSSK..",
    "..KSSSSKKSSSSK..",
    "..KaaaaaaaaaaK..",
    "..KaYYaaaaYYaK..",
    "..KaaaaaaaaaaK..",
    "..KttttttttttK..",
    "..KkkkkkkkkkkK..",
    "...KKKKKKKKKK...",
    "...KK......KK...",
    "................",
    "................",
];

/// "!" in a speech bubble: this villager has something for you.
const BUBBLE_NEW: &[&str] = &[
    "..KKKKKKKK..",
    ".KwwwwwwwwK.",
    "KwwwwYYwwwwK",
    "KwwwwYYwwwwK",
    "KwwwwYYwwwwK",
    "KwwwwYYwwwwK",
    "KwwwwwwwwwwK",
    "KwwwwYYwwwwK",
    ".KwwwwwwwwK.",
    "..KKKwwKKK..",
    "....KwK.....",
    ".....K......",
];

/// "?" in a speech bubble: you can finish their request.
const BUBBLE_DONE: &[&str] = &[
    "..KKKKKKKK..",
    ".KYYYYYYYYK.",
    "KYYYwwwwYYYK",
    "KYYwwYYwwYYK",
    "KYYYYYYwwYYK",
    "KYYYYYwwYYYK",
    "KYYYYwwYYYYK",
    "KYYYYYYYYYYK",
    ".KYYYwwYYYK.",
    "..KKKYYKKK..",
    "....KYK.....",
    ".....K......",
];

/// Three dots: they're chatting.
const BUBBLE_TALK: &[&str] = &[
    "..KKKKKKKK..",
    ".KwwwwwwwwK.",
    "KwwwwwwwwwwK",
    "KwKKwKKwKKwK",
    "KwKKwKKwKKwK",
    "KwwwwwwwwwwK",
    ".KwwwwwwwwK.",
    "..KKKwwKKK..",
    "....KwK.....",
    ".....K......",
];

const HEART: &[&str] = &[
    ".KK.KK..", "K00K11K.", "K0011111", "K0111112", ".K11112K", "..K112K.", "...K2K..", "....K...",
];

const HEART_EMPTY: &[&str] = &[
    ".KK.KK..", "KkkKkkK.", "Kkk.kkkK", "Kk.....K", ".Kk...K.", "..Kk.K..", "...KK...", "........",
];

const STAR_BADGE: &[&str] = &[
    "...K....", "..K0K...", "KK010KK.", "K01110K.", ".K121K..", "K12K21K.", "KK.K.KK.", "........",
];

const SCROLL_NOTE: &[&str] = &[
    "................",
    "...KKKKKKKKKK...",
    "..KyyyyyyyyyyK..",
    "..KyKKKKKKKyyK..",
    "..KyyyyyyyyyyK..",
    "..KyKKKKKyyyyK..",
    "..KyyyyyyyyyyK..",
    "..KyKKKKKKKKyK..",
    "..KyyyyyyyyyyK..",
    "..KyKKKKyyyyyK..",
    "..KyyyyyyyyyyK..",
    "..KyyyyyyyrryK..",
    "..KyyyyyyyrryK..",
    "...KKKKKKKKKK...",
    "................",
    "................",
];

/// Everything this module draws, by name.
pub fn build(bank: &mut TexBank, m: &mut HashMap<&'static str, TexId>) {
    let mut add = |name: &'static str, t: Texture| {
        m.insert(name, bank.add(t));
    };
    add("bus", art(BUS, NO));
    add("bubble_new", art(BUBBLE_NEW, NO));
    add("bubble_done", art(BUBBLE_DONE, NO));
    add("bubble_talk", art(BUBBLE_TALK, NO));
    add("heart", art(HEART, [WHITE, PINK, CRIMSON, CLEAR]));
    add("heart_empty", art(HEART_EMPTY, NO));
    add("star_badge", art(STAR_BADGE, [CREAM, GOLD, CLAY, CLEAR]));
    add("request", art(SCROLL_NOTE, NO));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quest_art_is_tidy() {
        for (name, rows) in [
            ("bus", BUS),
            ("new", BUBBLE_NEW),
            ("done", BUBBLE_DONE),
            ("talk", BUBBLE_TALK),
            ("heart", HEART),
            ("empty", HEART_EMPTY),
            ("badge", STAR_BADGE),
            ("note", SCROLL_NOTE),
        ] {
            let w = rows[0].len();
            assert!(rows.iter().all(|r| r.len() == w), "{name} is ragged");
            assert!(rows.iter().all(|r| !r.contains(' ')), "{name} has spaces");
        }
    }
}
