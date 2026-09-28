//! Pixel art for potions, their ingredients and the spells of the Starfall Spellery.
//!
//! Same format as `sprites.rs`: rows of characters, digits `0`-`3` are recolourable slots.

use std::collections::HashMap;

use super::sprites::{NO, art};
use crate::palette::*;
use crate::render::{TexBank, TexId, Texture};

const SMALL_POTION: &[&str] = &[
    "................",
    "................",
    "................",
    "................",
    ".......KK.......",
    "......KuuK......",
    ".......KK.......",
    "......KwwK......",
    ".....KwKK1K.....",
    "....Kw11112K....",
    "....K1w1112K....",
    "....K111122K....",
    "....K211222K....",
    ".....K2222K.....",
    "......KKKK......",
];

const MEDIUM_POTION: &[&str] = &[
    "................",
    "......KKKK......",
    "......KuuK......",
    "......KuuK......",
    ".......KK.......",
    "......KwwK......",
    "......KwwK......",
    ".....K1ww1K.....",
    "....Kw11112K....",
    "....Kw11112K....",
    "....K111122K....",
    "....K111122K....",
    "....K211222K....",
    "....K222222K....",
    ".....KKKKKK.....",
];

const LARGE_POTION: &[&str] = &[
    "................",
    ".....KKKKKK.....",
    ".....KuuuuK.....",
    "......KKKK......",
    "......KwwK......",
    "....KKKwwKKK....",
    "...Kw1111112K...",
    "..Kw1w1111122K..",
    "..K1w11111122K..",
    "..K1111111122K..",
    "..K1111111222K..",
    "..K2111112222K..",
    "...K22222222K...",
    "....KKKKKKKK....",
];

const VIAL: &[&str] = &[
    "................",
    "................",
    "......KKKK......",
    "......KuuK......",
    "......KKKK......",
    ".......KwK......",
    "......KwyK......",
    "......KwyK......",
    ".....KwyyyK.....",
    ".....KwyyyK.....",
    ".....KyyyyK.....",
    ".....KSyySK.....",
    "......KSSK......",
    ".......KK.......",
];

const HEARTLEAF: &[&str] = &[
    "................",
    "................",
    "...KKK...KKK....",
    "..KlggK.KgglK...",
    ".KlgggrKrgglgK..",
    ".KglggrrrggltK..",
    ".KgglggrgggltK..",
    "..KgglgrggltK...",
    "...KgggrggtK....",
    "....KgggttK.....",
    ".....KggtK......",
    "......KgK.......",
    "......KgK.......",
    ".......K........",
];

const SP_FIRE: &[&str] = &[
    "................",
    "..........KK....",
    ".........KoK....",
    "......KK.KoK.K..",
    ".....KoKKoyoKoK.",
    "....KoyoKoyyooK.",
    "...KoyyyooywyoK.",
    "..KoyywyyyyyyoK.",
    "..KoyyyyyyyyyoK.",
    "..KooyyyyyyyooK.",
    "...KuoooyyoouK..",
    "....KuuoooouK...",
    ".....KKuuuuK....",
    "......KKKKK.....",
];

const SP_FROST: &[&str] = &[
    "................",
    ".......KK.......",
    ".....K.KwK.K....",
    "....KwKKwKKwK...",
    ".....KwKwKwK....",
    "..K...KwwwK...K.",
    ".KwKKKKwSwKKKKwK",
    "..KwwwwSBSwwwwK.",
    ".KwKKKKwSwKKKKwK",
    "..K...KwwwK...K.",
    ".....KwKwKwK....",
    "....KwKKwKKwK...",
    ".....K.KwK.K....",
    ".......KK.......",
];

const SP_SPARK: &[&str] = &[
    "................",
    "........KKKK....",
    ".......KywyK....",
    "......KywyK.....",
    ".....KywyK......",
    "....KywyKKKK....",
    "...KywwwwwyK....",
    "...KKKKywyK.....",
    "......KywK......",
    ".....KywK.......",
    "....KywK........",
    "...KyK..........",
    "...KK...........",
    "................",
];

const SP_STAR: &[&str] = &[
    "................",
    ".......KK.......",
    "......KyyK......",
    "......KyyK......",
    "..KKKKKywKKKKK..",
    "..KyyyywwwyyyK..",
    "...KyyywwyyyK...",
    "....KyyyyyyK....",
    "....KyyyYyyK....",
    "...KyyYKKYyyK...",
    "...KyYK..KYyK...",
    "...KKK....KKK...",
    ".LL.........LL..",
    "L..L.......L..L.",
];

const SP_MEND: &[&str] = &[
    "................",
    "................",
    "...KKK...KKK....",
    "..KPPPK.KPPPK...",
    ".KPbwPPKPPPPrK..",
    ".KPwPPPPPPPPrK..",
    ".KPPPPKwKPPPrK..",
    "..KPPKwwwKPrK...",
    "...KPPKwKPrK....",
    "....KPPPPrK.....",
    ".....KPPrK......",
    "......KrK.......",
    ".......K........",
    "................",
];

const SP_WARD: &[&str] = &[
    "................",
    ".....KKKKKK.....",
    "....KaMMMMaK....",
    "...KaMwwMMMaK...",
    "..KaMwMMMMMMaK..",
    "..KaMMMKKMMMaK..",
    "..KaMMKSSKMMaK..",
    "..KaMMKSSKMMaK..",
    "..KaMMMKKMMMaK..",
    "..KtaMMMMMMatK..",
    "...KtaMMMMatK...",
    "....KtaaaatK....",
    ".....KKKKKK.....",
    "................",
];

const SP_BLINK: &[&str] = &[
    "................",
    "........KKKK....",
    "......KKLLLLK...",
    ".....KLLKKKLLK..",
    "....KLK....KLK..",
    "....KLK..KK.KLK.",
    "...KLK..KwLK.KK.",
    "...KLK..KLLK....",
    "...KLK...KK.....",
    "....KLK.........",
    "....KLLKK...K...",
    ".....KLLLKKwLK..",
    "......KKLLLLLK..",
    "........KKKKK...",
];

const SP_BLOOM: &[&str] = &[
    "................",
    "..........KK....",
    ".........KSSK...",
    "........KSwSSK..",
    "........KSSSSK..",
    ".........KSSK...",
    "....KKK...KK....",
    "...KlllK..KK....",
    "...KllgKKKlK....",
    "....KKlgKllK....",
    "......KgKKK.....",
    "......KgK.......",
    "....KKKgKKK.....",
    "...KnnnnnnnK....",
];

const SPELLBOOK: &[&str] = &[
    "................",
    "...KKKKKKKKKK...",
    "..KVVVVVVVVVVK..",
    "..KVLLLLLLLLVKK.",
    "..KVLyKyyKyLVKwK",
    "..KVLKyyyyKLVKwK",
    "..KVLyyYYyyLVKwK",
    "..KVLKyyyyKLVKwK",
    "..KVLyKyyKyLVKwK",
    "..KVLLLLLLLLVKwK",
    "..KVVVVVVVVVVKwK",
    "..KvvvvvvvvvvKK.",
    "...KKKKKKKKKK...",
    "................",
];

pub fn build(bank: &mut TexBank, m: &mut HashMap<&'static str, TexId>) {
    let mut add = |name: &'static str, t: Texture| {
        m.insert(name, bank.add(t));
    };
    // Potions: small, medium and large, for health, mana and energy.
    for (names, light, dark) in [
        (["hp_potion_s", "hp_potion_m", "hp_potion_l"], RED, CRIMSON),
        (["mp_potion_s", "mp_potion_m", "mp_potion_l"], SKY, BLUE),
        (["ep_potion_s", "ep_potion_m", "ep_potion_l"], GOLD, ORANGE),
    ] {
        for (name, rows) in names
            .into_iter()
            .zip([SMALL_POTION, MEDIUM_POTION, LARGE_POTION])
        {
            add(name, art(rows, [CLEAR, light, dark, CLEAR]));
        }
    }
    add("vial", art(VIAL, NO));
    add("heartleaf", art(HEARTLEAF, NO));
    // Spells.
    add("spell_firebolt", art(SP_FIRE, NO));
    add("spell_frost_nova", art(SP_FROST, NO));
    add("spell_chain_spark", art(SP_SPARK, NO));
    add("spell_starfall", art(SP_STAR, NO));
    add("spell_mend", art(SP_MEND, NO));
    add("spell_ward", art(SP_WARD, NO));
    add("spell_blink", art(SP_BLINK, NO));
    add("spell_bloom", art(SP_BLOOM, NO));
    add("spellbook", art(SPELLBOOK, NO));
}
