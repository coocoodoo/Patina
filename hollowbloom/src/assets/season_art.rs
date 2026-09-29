//! Icons for the seasons' crops: a seed packet and a picture of what each one gives, from a
//! few new drawings (flowers, bells, cherries, a pineapple, a pumpkin...) and some of
//! `item_art`'s recoloured; and the dishes they cook into. Same ASCII legend as `sprites`,
//! with recolour slots '0'-'3'.

use std::collections::HashMap;

use super::item_art::{
    ACORN, BAKED, CABBAGE, CAKE, CUP, EGGPLANT, GLASS, PEAR, POTATO, ROOT, ROSE, ROUND_FRUIT,
    SPRIG, TULIP,
};
use super::sprites::{self, BOWL, NO, PACKET, TART, art};
use crate::palette::*;
use crate::render::{TexBank, TexId, Texture};

/// A round flower head of petals (slots: light and mid petals, the middle, the stem).
const DAISY: &[&str] = &[
    "................",
    ".....KK..KK.....",
    "....K00KK01K....",
    "..KKK001101KKK..",
    ".K000KK11KK012K.",
    ".K0011K22K1122K.",
    "..KK1K2ww2K1KK..",
    "..KK0K2w22K0KK..",
    ".K001K2222K112K.",
    ".K0112KK22K122K.",
    "..KK112K3K122K..",
    "....K12K3K2K....",
    ".....KK.K3KKK...",
    "..KK....K3K33K..",
    ".K333KKK3K33K...",
    "..KK33333KKK....",
];

/// Two bell flowers nodding from an arched stem (slots: light, mid and dark bells, stem).
const BELL: &[&str] = &[
    "................",
    "....KKKKK.......",
    "...K33333K......",
    "..K3KK..K3K.....",
    ".K3K.....K3K....",
    "..KK.....KKKK...",
    "..KKK.....K00K..",
    ".K00K....K0w01K.",
    "K0w01K...K0001K.",
    "K00011K..K01112K",
    "K011122K.K11122K",
    ".K11122K.KK222K.",
    "..K2K2K..K2KK2K.",
    "...K.K....K..K..",
    "................",
    "................",
];

/// A pair of cherries on their stems (slots: light, mid, dark, stem).
const CHERRY: &[&str] = &[
    "................",
    "..........KKK...",
    ".........K33K...",
    "........K3K3K...",
    ".......K3KK33K..",
    "......K3K..K3K..",
    ".....K3K....K3K.",
    "...KKKK....KKK..",
    "..K00w0K..K0w0K.",
    ".K000001KK00001K",
    ".K000011KK00011K",
    ".K0001122K00112K",
    "..K01122KK1122K.",
    "...KK22K..KK22K.",
    ".....KK.....KK..",
    "................",
];

/// Stalks under a spread of leaves, for rhubarb (slots: light, mid, dark, leaves).
const STALKS: &[&str] = &[
    "................",
    "..KKKK...KKKK...",
    ".K3333K.K3333K..",
    "K33K3333333K33K.",
    ".KK3KK333KK3KK..",
    "...K..K3K..K....",
    "....K0KK1K......",
    "....K01K11K.....",
    "...K001K11K.....",
    "...K01K011K.....",
    "..K001K01K......",
    "..K01K011K......",
    ".K001K01K.......",
    ".K01K011K.......",
    ".KKKKKKK........",
    "................",
];

/// A slice of melon (slots: light and mid flesh, rind, dark rind).
const SLICE: &[&str] = &[
    "................",
    "................",
    "................",
    "..KKKKKKKKKKKK..",
    ".K000000000000K.",
    ".K0K00w0K000K0K.",
    "..K000000000K0K.",
    "..K0K000K00000K.",
    "...K00000000KK..",
    "...KK0000K00K...",
    "....K1100011K...",
    ".....K11111K....",
    ".....KK222KK....",
    "......K222K.....",
    ".......KKK......",
    "................",
];

/// A pineapple under its spiky crown (slots: light, mid, dark, leaves).
const PINEAPPLE: &[&str] = &[
    "....KK...KK.....",
    "...K33K.K33K....",
    "....K33K33K.....",
    "..KK.K333K.KK...",
    ".K33KK333KK33K..",
    "..KK33K3K33KK...",
    "....KKKKKKK.....",
    "...K0w0K00KK....",
    "..K00K001K01K...",
    "..K0K001K011K...",
    "..K001K01K12K...",
    "..K01K011K12K...",
    "..K0011K11K2K...",
    "...K11K112KK....",
    "....KK222KK.....",
    "......KKK.......",
];

/// A plump ribbed pumpkin (slots: light, mid, dark, stem).
const PUMPKIN: &[&str] = &[
    "................",
    "......KK........",
    ".....K33K.......",
    "......K3KK......",
    "...KKKKKKKKK....",
    "..K00K00K001K...",
    ".K0w0K00K0011K..",
    ".K000K00K0011K..",
    "K0000K00K00112K.",
    "K0001K00K01112K.",
    "K0011K01K11122K.",
    ".K011K11K1122K..",
    ".K112K11K1222K..",
    "..KK22K2K22KK...",
    "....KKKKKKK.....",
    "................",
];

/// A bunch of grapes with a leaf (slots: light, mid, dark, leaf).
const GRAPES: &[&str] = &[
    "................",
    "........KK......",
    ".......K3K..KK..",
    "......K3K..K33K.",
    "....KKKKKKK333K.",
    "...K0wK01K0KKK..",
    "...K00K01K01K...",
    "..KK11KKKKK11K..",
    ".K0w0K0w0K0w0K..",
    ".K001K001K011K..",
    "..KKKKKKKKKKK...",
    "...K0wK0w0K.....",
    "...K01K011K.....",
    "....KKK0wK......",
    ".....KK01K......",
    "......KKK.......",
];

/// Two pieces of candy corn (slots: tip, middle, base).
const CANDY_CORN: &[&str] = &[
    "................",
    "................",
    ".....K.....K....",
    "....K0K...K0K...",
    "...K0w0K.K0w0K..",
    "...K000K.K000K..",
    "..K11111KK1111K.",
    "..K11111KK1111K.",
    ".K1111111K11111K",
    ".K2222222K22222K",
    ".K2222222K22222K",
    ".KKKKKKKKKKKKKKK",
    "................",
    "................",
    "................",
    "................",
];

/// Holly leaves round a cluster of berries (slots: light and mid leaf, berries, dark leaf).
const HOLLY: &[&str] = &[
    "................",
    "...KK......KK...",
    "..K00K....K01K..",
    ".K0w01K..K0011K.",
    "K0K001KKK001K1K.",
    "K00K011K1011KK..",
    ".K0011K222K11K..",
    "..KKK1K2w2K1KK..",
    "....KK22322KK...",
    "..KKK2w2K2w2K...",
    ".K001K22KK222K..",
    ".K011KKK.KK1KK..",
    "..K12K.....K1K..",
    "...KK.......K...",
    "................",
    "................",
];

/// Everything this module draws, by name.
pub fn build(bank: &mut TexBank, m: &mut HashMap<&'static str, TexId>) {
    let mut add = |name: &'static str, t: Texture| {
        m.insert(name, bank.add(t));
    };
    // Recolourings of drawings made for other crops.
    let lettuce = |c| match c {
        LIME => CREAM,
        GREEN => LIME,
        _ => c,
    };
    let kale = |c| match c {
        LIME => TEAL,
        GREEN => DEEP_TEAL,
        _ => c,
    };
    let zucchini = |c| match c {
        PURPLE => GREEN,
        LAVENDER => LIME,
        GRAPE => TEAL,
        _ => c,
    };
    let snowy = |c| match c {
        RED => WHITE,
        CRIMSON => SKY,
        _ => c,
    };
    let chestnut = |c| match c {
        CREAM => PEACH,
        GOLD => CLAY,
        CLAY => RUST,
        RUST => MAROON,
        _ => c,
    };

    // Seed packets.
    let packets: [(&'static str, [u8; 3]); 40] = [
        ("tulip_bulb", [BLUSH, GREEN, PINK]),
        ("daffodil_bulb", [GOLD, GREEN, CREAM]),
        ("bluebell_bulb", [BLUE, GREEN, SKY]),
        ("lavender_seeds", [LAVENDER, GREEN, PURPLE]),
        ("cherry_pits", [RED, GREEN, PINK]),
        ("cloudberry_seeds", [PEACH, GREEN, CREAM]),
        ("rhubarb_crown", [PINK, GREEN, CRIMSON]),
        ("onion_seeds", [WHITE, GREEN, LIME]),
        ("lettuce_seeds", [LIME, GREEN, WHITE]),
        ("baby_carrot_seeds", [ORANGE, GREEN, GOLD]),
        ("watermelon_seeds", [GREEN, RED, LIME]),
        ("honeydew_seeds", [LIME, GREEN, CREAM]),
        ("pineapple_top", [GOLD, GREEN, CREAM]),
        ("peach_pit", [PEACH, GREEN, SALMON]),
        ("raspberry_canes", [PINK, GREEN, BLUSH]),
        ("bell_pepper_seeds", [ORANGE, GREEN, GOLD]),
        ("zucchini_seeds", [GREEN, GOLD, LIME]),
        ("hibiscus_seeds", [PINK, GREEN, GOLD]),
        ("poppy_seeds", [RED, GREEN, SALMON]),
        ("cosmos_seeds", [LAVENDER, GREEN, GOLD]),
        ("harvest_pumpkin_seeds", [ORANGE, GREEN, GOLD]),
        ("squash_seeds", [PEACH, GREEN, CLAY]),
        ("sweet_potato_slips", [SALMON, GREEN, ROSEWOOD]),
        ("cranberry_seeds", [RED, GREEN, CRIMSON]),
        ("grape_vine", [PURPLE, GREEN, LAVENDER]),
        ("crabapple_seeds", [RED, GREEN, GOLD]),
        ("chestnut_sapling", [CLAY, GREEN, RUST]),
        ("beet_seeds", [CRIMSON, GREEN, PLUM]),
        ("mum_seeds", [GOLD, GREEN, ORANGE]),
        ("candy_corn_kernels", [ORANGE, GREEN, WHITE]),
        ("snowdrop_bulb", [WHITE, GREEN, SKY]),
        ("snow_rose_seeds", [SKY, GREEN, WHITE]),
        ("poinsettia_cutting", [RED, GREEN, GOLD]),
        ("mistletoe_sprig", [GREEN, WHITE, LIME]),
        ("holly_seeds", [RED, GREEN, TEAL]),
        ("kale_seeds", [TEAL, GREEN, DEEP_TEAL]),
        ("parsnip_seeds", [CREAM, GREEN, KHAKI]),
        ("sprout_seeds", [LIME, GREEN, TEAL]),
        ("snow_melon_seeds", [SKY, GREEN, WHITE]),
        ("frostberry_seeds", [WHITE, GREEN, SKY]),
    ];
    for (name, [a, b, c]) in packets {
        add(name, art(PACKET, [CLEAR, a, b, c]));
    }

    // What they give.
    add("pastel_tulip", art(TULIP, [WHITE, BLUSH, PINK, GREEN]));
    add("daffodil", art(DAISY, [CREAM, GOLD, ORANGE, GREEN]));
    add("bluebell", art(BELL, [SKY, BLUE, INDIGO, GREEN]));
    add("lavender", art(SPRIG, [LAVENDER, PURPLE, GRAPE, CLEAR]));
    add("sweet_cherry", art(CHERRY, [SALMON, RED, CRIMSON, GREEN]));
    add(
        "cloudberry",
        art(sprites::BERRIES, [CLEAR, CREAM, PEACH, CLAY]),
    );
    add("rhubarb", art(STALKS, [PINK, CRIMSON, MAROON, GREEN]));
    add("spring_onion", art(ROOT, [WHITE, SAND, CLEAR, LIME]));
    add("butter_lettuce", art(CABBAGE, NO).map_colors(lettuce));
    add("baby_carrot", art(ROOT, [GOLD, ORANGE, CLEAR, GREEN]));
    add("watermelon", art(SLICE, [SALMON, RED, GREEN, DEEP_TEAL]));
    add("honeydew", art(sprites::MELON, [CLEAR, CREAM, LIME, GREEN]));
    add("pineapple", art(PINEAPPLE, [CREAM, GOLD, CLAY, GREEN]));
    add("peach", art(ROUND_FRUIT, [PEACH, SALMON, CLAY, GREEN]));
    add(
        "raspberry",
        art(sprites::BERRIES, [CLEAR, BLUSH, PINK, CRIMSON]),
    );
    add(
        "bell_pepper",
        art(sprites::PEPPER, [CLEAR, GOLD, ORANGE, CLEAR]),
    );
    add("zucchini", art(EGGPLANT, NO).map_colors(zucchini));
    add("hibiscus", art(DAISY, [BLUSH, PINK, GOLD, GREEN]));
    add("poppy", art(DAISY, [SALMON, RED, INK, GREEN]));
    add("cosmos", art(DAISY, [LAVENDER, PURPLE, GOLD, GREEN]));
    add("pumpkin", art(PUMPKIN, [GOLD, ORANGE, CLAY, GREEN]));
    add("butternut", art(PEAR, [CREAM, PEACH, CLAY, GREEN]));
    add(
        "sweet_potato",
        art(POTATO, [SALMON, ROSEWOOD, MAROON, CLEAR]),
    );
    add(
        "cranberry",
        art(sprites::BERRIES, [CLEAR, RED, CRIMSON, PLUM]),
    );
    add("grapes", art(GRAPES, [LAVENDER, PURPLE, GRAPE, GREEN]));
    add("crabapple", art(ROUND_FRUIT, [GOLD, RED, MAROON, GREEN]));
    add("chestnut", art(ACORN, NO).map_colors(chestnut));
    add("beetroot", art(ROOT, [CRIMSON, PLUM, CLEAR, GREEN]));
    add("chrysanthemum", art(DAISY, [GOLD, ORANGE, RUST, GREEN]));
    add("candy_corn", art(CANDY_CORN, [WHITE, ORANGE, GOLD, GREEN]));
    add("snowdrop", art(BELL, [WHITE, SKY, BLUE, GREEN]));
    add("snow_rose", art(ROSE, NO).map_colors(snowy));
    add("poinsettia", art(DAISY, [RED, CRIMSON, GOLD, GREEN]));
    add("mistletoe", art(HOLLY, [LIME, GREEN, WHITE, TEAL]));
    add("holly_berry", art(HOLLY, [LIME, GREEN, RED, TEAL]));
    add("kale", art(CABBAGE, NO).map_colors(kale));
    add("parsnip", art(ROOT, [CREAM, KHAKI, CLEAR, GREEN]));
    add(
        "brussels_sprouts",
        art(sprites::BERRIES, [CLEAR, LIME, GREEN, TEAL]),
    );
    add("snow_melon", art(sprites::MELON, [CLEAR, WHITE, SKY, BLUE]));
    add(
        "frostberry",
        art(sprites::BERRIES, [CLEAR, WHITE, SKY, BLUE]),
    );

    // Seasonal dishes.
    add("cherry_tart", art(TART, [CLEAR, SALMON, RED, CLEAR]));
    add("spring_salad", art(BOWL, [CREAM, LIME, ORANGE, GREEN]));
    add(
        "lavender_tea",
        art(CUP, NO).map_colors(|c| match c {
            MINT => LAVENDER,
            AQUA => PURPLE,
            _ => c,
        }),
    );
    add("rhubarb_crumble", art(TART, [CLEAR, PINK, CRIMSON, CLEAR]));
    add(
        "watermelon_slush",
        art(GLASS, NO).map_colors(|c| match c {
            GOLD => PINK,
            ORANGE => RED,
            _ => c,
        }),
    );
    add("pineapple_cake", art(CAKE, [CREAM, GOLD, ORANGE, RED]));
    add("peach_cobbler", art(TART, [CLEAR, PEACH, CLAY, CLEAR]));
    add("stuffed_peppers", art(BOWL, [GOLD, ORANGE, GREEN, RED]));
    add("pumpkin_soup", art(BOWL, [GOLD, ORANGE, CREAM, CLAY]));
    add(
        "candied_sweet_potato",
        art(BAKED, NO).map_colors(|c| match c {
            CREAM => PEACH,
            GOLD => SALMON,
            KHAKI => ROSEWOOD,
            RUST => MAROON,
            _ => c,
        }),
    );
    add(
        "grape_juice",
        art(GLASS, NO).map_colors(|c| match c {
            GOLD => PURPLE,
            ORANGE => GRAPE,
            _ => c,
        }),
    );
    add("roasted_chestnuts", art(BOWL, [CLAY, RUST, SAND, MAROON]));
    add("kale_stew", art(BOWL, [LIME, TEAL, CREAM, DEEP_TEAL]));
    add("holly_cake", art(CAKE, [WHITE, SAND, RED, GREEN]));
    add(
        "snow_rose_tea",
        art(CUP, NO).map_colors(|c| match c {
            MINT => WHITE,
            AQUA => SKY,
            _ => c,
        }),
    );
    add("frostberry_sorbet", art(BOWL, [WHITE, SKY, BLUSH, BLUE]));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn season_art_is_tidy() {
        for (name, rows) in [
            ("daisy", DAISY),
            ("bell", BELL),
            ("cherry", CHERRY),
            ("stalks", STALKS),
            ("slice", SLICE),
            ("pineapple", PINEAPPLE),
            ("pumpkin", PUMPKIN),
            ("grapes", GRAPES),
            ("candy_corn", CANDY_CORN),
            ("holly", HOLLY),
        ] {
            let w = rows[0].len();
            assert_eq!(w, 16, "{name} is {w} wide");
            assert!(rows.len() <= 16, "{name} is too tall");
            assert!(rows.iter().all(|r| r.len() == w), "{name} is ragged");
        }
    }
}
