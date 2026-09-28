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

// Keepsakes for quests.

const LETTER: &[&str] = &[
    "................",
    "................",
    "................",
    ".KKKKKKKKKKKKKK.",
    ".KwyyyyyyyyyyyK.",
    ".KyKyyyyyyyyKyK.",
    ".KyyKyyyyyyKyyK.",
    ".KyyyKyyyyKyyyK.",
    ".KyyyyKrrKyyyyK.",
    ".KyyyyyrrryyyyK.",
    ".KyyyyyyrryyyyK.",
    ".KnyyyyyyyyyynK.",
    ".KnnnnnnnnnnnnK.",
    ".KKKKKKKKKKKKKK.",
    "................",
    "................",
];

const PARCEL: &[&str] = &[
    "................",
    "................",
    "......K..K......",
    ".....KuKKuK.....",
    "..KKKKKuuKKKKK..",
    "..KCCCCKuCCCCK..",
    "..KCyCCKuCCyCK..",
    "..KKKKKKKKKKKK..",
    "..KCCCCKuCCCCK..",
    "..KCCCCKuCCCCK..",
    "..KCCCCKuCCCCK..",
    "..KRRRRKuRRRRK..",
    "..KKKKKKKKKKKK..",
    "................",
    "................",
    "................",
];

const BASKET: &[&str] = &[
    "................",
    "......KKKK......",
    ".....K....K.....",
    "....K......K....",
    "...KKKKKKKKKK...",
    "..KrrwrrwrrwrK..",
    "..KwrrwrrwrrwK..",
    "..KKKKKKKKKKKK..",
    "..KCuCuCuCuCuK..",
    "..KuCuCuCuCuCK..",
    "...KCuCuCuCuK...",
    "...KuCuCuCuCK...",
    "....KKKKKKKK....",
    "................",
    "................",
    "................",
];

const LOCKET: &[&str] = &[
    "................",
    "....K......K....",
    "...K.K....K.K...",
    "...K..K..K..K...",
    "....K..KK..K....",
    ".....K....K.....",
    "....KKKK.KKKK...",
    "...KYyyYKYYYYK..",
    "..KYyYYYYYYYCK..",
    "..KYYYYbYYYYCK..",
    "...KYYYYYYYCK...",
    "....KYYYYYCK....",
    ".....KYYYCK.....",
    "......KYCK......",
    ".......KK.......",
    "................",
];

const TEDDY: &[&str] = &[
    "................",
    "...KKK....KKK...",
    "..KCCCK..KCCCK..",
    "..KCeCKKKKCeCK..",
    "...KCCCCCCCCK...",
    "..KCCCCCCCCCCK..",
    "..KCCKCCCCKCCK..",
    "..KCCKCCCCwCCK..",
    "..KCCCeeeeCCCK..",
    "..KCCCeKKeCCCK..",
    "...KCCCeeCCCK...",
    "....KCCCCCCK....",
    "...KCCCCCCCCK...",
    "..KCCKCCCCKCCK..",
    "...KK.KKKK.KK...",
    "................",
];

const MAP: &[&str] = &[
    "................",
    "................",
    "..KKKKKKKKKKKK..",
    ".KnyyyyyyyyyyyK.",
    ".KyyyKyyyyyyyyK.",
    ".KyyyyKyyyyrKrK.",
    ".KyyKyyKyyyyrKK.",
    ".KyyyyyyKyyrKrK.",
    ".KyyyyyyyKyyyyK.",
    ".KyyggyyyyKyyyK.",
    ".KyggggyyyyKyyK.",
    ".KyyggyyyyyyyyK.",
    ".KnyyyyyyyyyynK.",
    "..KKKKKKKKKKKK..",
    "................",
    "................",
];

const JELLY_HEART: &[&str] = &[
    "................",
    "................",
    "...KKK...KKK....",
    "..K000K.K001K...",
    ".K0w0011K1112K..",
    ".K0w001111112K..",
    ".K00111111112K..",
    ".K01111111122K..",
    "..K111111122K...",
    "...K1111122K....",
    "....K11122K.....",
    ".....K122K......",
    "......K2K.......",
    ".......K........",
    "................",
    "................",
];

const JAR: &[&str] = &[
    "................",
    ".....KKKKKK.....",
    ".....KCuuCK.....",
    "....KKKKKKKK....",
    "....KwSSSSSK....",
    "...KwS0000SSK...",
    "...KS001100SK...",
    "...KS011110SK...",
    "...KS011210SK...",
    "...KS012210SK...",
    "...KS011110SK...",
    "...KwS1111SSK...",
    "....KSSSSSSK....",
    ".....KKKKKK.....",
    "................",
    "................",
];

const FANG: &[&str] = &[
    "................",
    "................",
    "....KKKKKKKK....",
    "...KwwwwwwyyK...",
    "...KwwwwwwyyK...",
    "....KwwwwwyK....",
    "....KwwwwwyK....",
    ".....KwwwyK.....",
    ".....KwwwyK.....",
    "......KwyK......",
    "......KwyK......",
    ".......KK.......",
    "................",
    "................",
    "................",
    "................",
];

const ORB: &[&str] = &[
    "................",
    "................",
    ".....KKKKKK.....",
    "....K0ww0011K...",
    "...K0w000111K...",
    "...K00001111K...",
    "..K0000111112K..",
    "..K0001111122K..",
    "..K0011111122K..",
    "..K0111111222K..",
    "...K11111222K...",
    "...K11112222K...",
    "....KK2222KK....",
    "......KKKK......",
    "................",
    "................",
];

const FLOWER: &[&str] = &[
    "................",
    "................",
    ".......KK.......",
    "......K00K......",
    "..KK..K01K..KK..",
    ".K00KK0110KK00K.",
    ".K0110011001110K",
    "..K111122111K...",
    "..K112w22w211K..",
    ".K0111122111K...",
    ".K00K110011KK00K",
    "..KK.K01K...KK..",
    ".....K00K.......",
    "......KK........",
    "................",
    "................",
];

const LANTERN: &[&str] = &[
    "................",
    "......KKKK......",
    ".....K....K.....",
    "....KKKKKKKK....",
    "....KDDDDDDK....",
    "...KwK0000KDK...",
    "...K0K0110K0K...",
    "...K0K1221K0K...",
    "...K0K1221K0K...",
    "...K0K0110K0K...",
    "...KwK0000KDK...",
    "....KDDDDDDK....",
    "....KKKKKKKK....",
    "......KDDK......",
    ".......KK.......",
    "................",
];

const SHARD: &[&str] = &[
    "................",
    ".......KK.......",
    ".......K0K......",
    "......K0w0K.....",
    "......K000K.....",
    "..KKKKK0101KKKK.",
    "..K00w0001111K..",
    "...K0000111K....",
    "....K011111K....",
    "....K11K2111K...",
    "...K12K..K212K..",
    "...KK.....KK2K..",
    "...........KK...",
    "................",
    "................",
    "................",
];

const SHEET: &[&str] = &[
    "................",
    "..KKKKKKKKKKK...",
    "..KwyyyyyyyynK..",
    "..KyyyyyyyyyyK..",
    "..KkkkkkkkkkkK..",
    "..KyyyKyyyyyyK..",
    "..KkkkKkkKkkkK..",
    "..KyyKKyyKyyyK..",
    "..KkKKKkKKkkkK..",
    "..KyKKyyKKyyyK..",
    "..KkkkkkkkkkkK..",
    "..KyyyyyyyyyyK..",
    "..KkkkkkkkkkkK..",
    "..KnyyyyyyyyuK..",
    "..KKKKKKKKKKKK..",
    "................",
];

const BADGE: &[&str] = &[
    "................",
    "....KKK..KKK....",
    "....KBBKKBBK....",
    ".....KBBBBK.....",
    ".....KBrrBK.....",
    "......KKKK......",
    ".....KYYYYK.....",
    "....KYyYYYCK....",
    "...KYYYSSYYCK...",
    "...KYYSSSSYCK...",
    "...KYYYSSYYCK...",
    "....KYYYYYCK....",
    ".....KCCCCK.....",
    "......KKKK......",
    "................",
    "................",
];

const WHISK: &[&str] = &[
    "................",
    ".....KKKKKK.....",
    "....KYyyyYYK....",
    "...KY.KYYK.YK...",
    "...KY.KYYK.YK...",
    "...KY.KYYK.YK...",
    "....KY.KK.YK....",
    ".....KY..YK.....",
    "......KYYK......",
    ".......KK.......",
    ".......KuK......",
    ".......KuK......",
    ".......KuK......",
    ".......KmK......",
    "........K.......",
    "................",
];

const BAG: &[&str] = &[
    "................",
    "....KKKKKKK.....",
    "...K.......K....",
    "..K.........K...",
    "..KKKKKKKKKKKK..",
    "..KaaaaaaaaaaK..",
    "..KtttttttttaK..",
    "..KtKKKKKKKtaK..",
    "..KtKyyyyyKtaK..",
    "..KtKyyrryKtaK..",
    "..KtKyyyyyKtaK..",
    "..KtKKKKKKKtaK..",
    "..KTTTTTTTTTTK..",
    "..KKKKKKKKKKKK..",
    "................",
    "................",
];

const POD: &[&str] = &[
    "................",
    "........KK......",
    ".......KgK......",
    "......KKgKK.....",
    ".....KllllgK....",
    "....KlwlllggK...",
    "....KlllgYgggK..",
    "...KllgYYYggK...",
    "...KlgYYyYggK...",
    "...KlgYYYYggK...",
    "...KllgYYggK....",
    "....KllgggK.....",
    ".....KKgggK.....",
    ".......KKK......",
    "................",
    "................",
];

const COG: &[&str] = &[
    "................",
    ".......KK.......",
    "....KK.KYK.KK...",
    "...KYYKKYYKYYK..",
    "...KYyYYYYYYCK..",
    "....KYYKKKYCK...",
    "..KKYYK...KYCKK.",
    "..KYYK.....KYCK.",
    "..KYYK.....KYCK.",
    "..KKYYK...KYCKK.",
    "....KYYKKKYCK...",
    "...KYYCCCCCCCK..",
    "...KYCKKCCKCCK..",
    "....KK.KCK.KK...",
    ".......KK.......",
    "................",
];

const LEAF: &[&str] = &[
    "................",
    "...........KK...",
    "..........K0K...",
    ".......KKK00K...",
    ".....KK000011K..",
    "....K00w001112K.",
    "...K0w00111122K.",
    "...K000111122K..",
    "..K000111122K...",
    "..K00111122K....",
    "..K0111122K.....",
    "..K111222K......",
    ".KuKK22KK.......",
    "KuK..KK.........",
    "KK..............",
    "................",
];

const ACORN: &[&str] = &[
    "................",
    ".......KK.......",
    "......KuuK......",
    "....KKKKKKKK....",
    "...KCuCuCuCuK...",
    "...KuCuCuCuCK...",
    "...KKKKKKKKKK...",
    "....K001111K....",
    "....K0w01112K...",
    "....K0011112K...",
    "....K0111122K...",
    ".....K11122K....",
    "......K122K.....",
    ".......KKK......",
    "................",
    "................",
];

const KEY: &[&str] = &[
    "................",
    "................",
    "...KKKK.........",
    "..K0ww0K........",
    ".K0K..K1K.......",
    ".K0K..K1KKKKKKK.",
    ".K01KK12000000K.",
    "..K1112KK1KK1K..",
    "...KKKK.K1KK1K..",
    "........KK.KK...",
    "................",
    "................",
    "................",
    "................",
    "................",
    "................",
];

const WATCH: &[&str] = &[
    "................",
    ".......KK.......",
    "......KYYK......",
    ".....KKYYKK.....",
    "....KYYYYYYK....",
    "...KYwwwwwwYK...",
    "..KYwwwwKwwwYK..",
    "..KYwwwwKwwwYK..",
    "..KYwwwwKKKwYK..",
    "..KYwwwwwwwwYK..",
    "..KYwwwwwwwwYK..",
    "...KYwwwwwwYK...",
    "....KYYYYYYK....",
    ".....KKKKKK.....",
    "................",
    "................",
];

const GLASSES: &[&str] = &[
    "................",
    "................",
    "................",
    "................",
    "................",
    "..KKKKK..KKKKK..",
    ".KYSSSSKKYSSSSK.",
    "KKYSwSSYYSwSSKKK",
    "K.YSSSSYKSSSSY.K",
    "..KYSSYK.KYSSYK.",
    "...KKKK...KKKK..",
    "................",
    "................",
    "................",
    "................",
    "................",
];

const BELL: &[&str] = &[
    "................",
    ".......KK.......",
    "......KYYK......",
    ".....KYYYYK.....",
    "....KYyYYYCK....",
    "....KYyYYYCK....",
    "...KYYyYYYYCK...",
    "...KYYYYYYYCK...",
    "..KYYYYYYYYYCK..",
    "..KYYYYYYYYYCK..",
    ".KYYYYYYYYYYYCK.",
    ".KKKKKKKKKKKKKK.",
    ".......KuK......",
    ".......KKK......",
    "................",
    "................",
];

const QUILL: &[&str] = &[
    "................",
    "............KK..",
    "...........K01K.",
    "..........K011K.",
    ".........K0112K.",
    "........K0112K..",
    ".......K0112K...",
    "......K0112K....",
    ".....K0112K.....",
    "....K0112K......",
    "....K012K.......",
    "...KK12K........",
    "...KuKK.........",
    "..KuK...........",
    "..KK............",
    "................",
];

const STONE: &[&str] = &[
    "................",
    "................",
    "......KKKK......",
    "....KK0000KK....",
    "...K00ww0011K...",
    "..K00w001111K...",
    "..K0001111w12K..",
    "..K011w11112K...",
    "..K0111ww112K...",
    "...K111111122K..",
    "...K11122222K...",
    "....KK2222KK....",
    "......KKKK......",
    "................",
    "................",
    "................",
];

const BEETLE: &[&str] = &[
    "................",
    ".....K....K.....",
    "......K..K......",
    ".....KKKKKK.....",
    "....KDDDDDDK....",
    "...KK00K0000K...",
    "..K00w0K00001K..",
    "..K0000K00011K..",
    "..K0000K00011K..",
    "..K0010K00111K..",
    "..K0001K01112K..",
    "...K001K1112K...",
    "....KKKKKKKK....",
    "...K..K..K..K...",
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
    // Keepsakes.
    add("letter", art(LETTER, NO));
    add("parcel", art(PARCEL, NO));
    add("pie_basket", art(BASKET, NO));
    add("locket", art(LOCKET, NO));
    add("teddy", art(TEDDY, NO));
    add("map_scrap", art(MAP, NO));
    add(
        "slime_heart",
        art(JELLY_HEART, [BLUSH, PINK, CRIMSON, CLEAR]),
    );
    add("royal_jelly", art(JELLY_HEART, [CREAM, GOLD, CLAY, CLEAR]));
    add("ember_heart", art(JELLY_HEART, [GOLD, ORANGE, RED, CLEAR]));
    add("moonmoss", art(JAR, [MINT, AQUA, TEAL, CLEAR]));
    add("rainbow_spore", art(JAR, [BLUSH, LAVENDER, PURPLE, CLEAR]));
    add("glow_oil", art(JAR, [CREAM, GOLD, ORANGE, CLEAR]));
    add("bat_fang", art(FANG, NO));
    add("singing_crystal", art(ORB, [WHITE, AQUA, TEAL, CLEAR]));
    add("matriarch_pearl", art(ORB, [WHITE, BLUSH, LAVENDER, CLEAR]));
    add("ember_gem", art(ORB, [GOLD, ORANGE, RED, CLEAR]));
    add("frost_core", art(ORB, [WHITE, SKY, BLUE, CLEAR]));
    add("frost_blossom", art(FLOWER, [WHITE, SKY, BLUE, CLEAR]));
    add("ghost_lantern", art(LANTERN, [MINT, AQUA, WHITE, CLEAR]));
    add("star_shard", art(SHARD, [CREAM, GOLD, CLAY, CLEAR]));
    add("song_page", art(SHEET, NO));
    add("knight_badge", art(BADGE, NO));
    add("golden_whisk", art(WHISK, NO));
    add("mailbag", art(BAG, NO));
    add("seed_pod", art(POD, NO));
    add("cog", art(COG, NO));
    add("wish_leaf", art(LEAF, [BLUSH, PINK, CRIMSON, CLEAR]));
    add("capwood_acorn", art(ACORN, [LIME, GREEN, TEAL, CLEAR]));
    add("warden_key", art(KEY, [WHITE, SAND, KHAKI, CLEAR]));
    add("pocket_watch", art(WATCH, NO));
    add("spectacles", art(GLASSES, NO));
    add("bell_clapper", art(BELL, NO));
    add("spring_stone", art(STONE, [WHITE, AQUA, BLUE, CLEAR]));
    add("glow_beetle", art(BEETLE, [LIME, GREEN, TEAL, CLEAR]));
    add("phoenix_quill", art(QUILL, [GOLD, ORANGE, RED, CLEAR]));
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
            ("letter", LETTER),
            ("parcel", PARCEL),
            ("basket", BASKET),
            ("locket", LOCKET),
            ("teddy", TEDDY),
            ("map", MAP),
            ("jelly", JELLY_HEART),
            ("jar", JAR),
            ("fang", FANG),
            ("orb", ORB),
            ("flower", FLOWER),
            ("lantern", LANTERN),
            ("shard", SHARD),
            ("sheet", SHEET),
            ("badge", BADGE),
            ("whisk", WHISK),
            ("bag", BAG),
            ("pod", POD),
            ("cog", COG),
            ("leaf", LEAF),
            ("acorn", ACORN),
            ("key", KEY),
            ("watch", WATCH),
            ("glasses", GLASSES),
            ("bell", BELL),
            ("quill", QUILL),
            ("stone", STONE),
            ("beetle", BEETLE),
        ] {
            let w = rows[0].len();
            assert!(rows.iter().all(|r| r.len() == w), "{name} is ragged");
            assert!(rows.iter().all(|r| !r.contains(' ')), "{name} has spaces");
        }
    }
}
