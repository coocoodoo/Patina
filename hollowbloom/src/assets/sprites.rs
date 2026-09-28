//! Hand-drawn 16x16 pixel art: item icons, crop stages, decorations and UI bits.
//!
//! Art is written as rows of characters; the legend maps each character to a palette index.
//! Digits `0`-`3` are recolourable slots so one drawing serves several tiers or crops.

use std::collections::HashMap;

use crate::palette::*;
use crate::render::{TexBank, TexId, Texture};

pub const LEGEND: &[(char, u8)] = &[
    ('K', INK),
    ('k', SHADOW),
    ('w', WHITE),
    ('o', ORANGE),
    ('r', RED),
    ('p', PLUM),
    ('c', CRIMSON),
    ('P', PINK),
    ('s', SALMON),
    ('e', PEACH),
    ('n', SAND),
    ('h', KHAKI),
    ('R', ROSEWOOD),
    ('T', DEEP_TEAL),
    ('t', TEAL),
    ('g', GREEN),
    ('l', LIME),
    ('y', CREAM),
    ('Y', GOLD),
    ('C', CLAY),
    ('u', RUST),
    ('m', MAROON),
    ('v', GRAPE),
    ('V', PURPLE),
    ('L', LAVENDER),
    ('b', BLUSH),
    ('S', SKY),
    ('B', BLUE),
    ('I', INDIGO),
    ('D', SLATE),
    ('a', AQUA),
    ('M', MINT),
];

fn art(rows: &[&str], slots: [u8; 4]) -> Texture {
    let mut legend: Vec<(char, u8)> = LEGEND.to_vec();
    for (i, c) in slots.iter().enumerate() {
        legend.push((char::from(b'0' + i as u8), *c));
    }
    Texture::from_art(rows, &legend)
}

const NO: [u8; 4] = [CLEAR; 4];

const WOOD: &[&str] = &[
    "................",
    "................",
    "................",
    "..........KKKK..",
    "...KKKKKKKnnnnK.",
    "..KCCCCCCKnuuunK",
    ".KCCuCCCCKnunnuK",
    ".KCCCCCuCKnununK",
    ".KuCCCCCCKnunnuK",
    ".KuuuCuuuKnuuunK",
    ".KmuuuuuuuKnnnK.",
    "..KmmmmmmmmKKK..",
    "...KKKKKKKKK....",
];

const STONE: &[&str] = &[
    "................",
    "................",
    "................",
    "................",
    ".....KKKKK......",
    "...KKhhnnhKK....",
    "..KhhnnhhhhRK...",
    "..KhnnhhhhRRKK..",
    ".KhhhhhhRRRRRK..",
    ".KhhhhRRRRRkkK..",
    ".KRRRRRRRkkkK...",
    "..KkkkkkkkkK....",
    "...KKKKKKKK.....",
];

const FIBER: &[&str] = &[
    "................",
    "......K.K.K.....",
    ".....KlKlKlK....",
    ".....KlKlKlK....",
    ".....KgKlKgK....",
    ".....KgKgKgK....",
    ".....KgKgKgK....",
    "....KnnnnnnnK...",
    "....KhhhhhhhK...",
    ".....KgKgKgK....",
    ".....KgKgKgK....",
    "....KgK.KgK.K...",
    "....KgK.KgKKgK..",
    "....KK..KK..KK..",
];

const NUGGET: &[&str] = &[
    "................",
    "................",
    "................",
    "......KKKK......",
    "....KK1122KK....",
    "...K1w122223K...",
    "..K1122222233K..",
    "..K1222222333K..",
    ".K122222223333K.",
    ".K222222233333K.",
    ".K322223333333K.",
    "..K3333333333K..",
    "...KK333333KK...",
    ".....KKKKKK.....",
];

const GEM: &[&str] = &[
    "................",
    "......KKKK......",
    ".....K11w2K.....",
    "....K111222K....",
    "....K1w1222K....",
    "....K111223K....",
    "....K112223K....",
    "....K112233K....",
    "....K122233K....",
    "....K122333K....",
    "....K222333K....",
    "....K223333K....",
    ".....K2333K.....",
    "......KKKK......",
];

const GEL: &[&str] = &[
    "................",
    "................",
    "................",
    "................",
    "......KKKK......",
    "....KK1w11KK....",
    "...K1w111122K...",
    "...K11111222K...",
    "..K1111122222K..",
    "..K1112222223K..",
    "..K1222222233K..",
    "...K22223333K...",
    "....KKKKKKKK....",
];

const WING: &[&str] = &[
    "................",
    "................",
    "..K.............",
    "..KK............",
    "..KvK.....K.....",
    "..KvvK...KvK....",
    "..KvVvK.KvVvK...",
    "..KvVVvKvVVVvK..",
    "..KvVVVvVVVVVvK.",
    "..KvVVVVVVVVVVK.",
    "...KvVVVVVVVVK..",
    "...KvVKvVVKvVK..",
    "....KK.KvK.KK...",
    "........K.......",
];

const BONE: &[&str] = &[
    "................",
    "................",
    "................",
    "................",
    ".KKK........KKK.",
    "KwwnK......KwwnK",
    "KwnnKKKKKKKKwnnK",
    ".KnwwwwwwwwwwnK.",
    ".KnnnnnnnnnnnnK.",
    "KwnnKKKKKKKKwnnK",
    "KnnhK......KnnhK",
    ".KKK........KKK.",
];

const SPORE: &[&str] = &[
    "................",
    "................",
    ".......K........",
    "......KaK.......",
    "...K..KaK..K....",
    "..KaK.KMK.KaK...",
    "...KKKMMMKKK....",
    "....KMMwMMK.....",
    "..KKMMwwwMMKK...",
    ".KaMMMMwMMMMaK..",
    "..KKMMMMMMMKK...",
    "....KMMMMMK.....",
    "...KKKKMKKKK....",
    "..KaK.KaK.KaK...",
    "...K..KaK..K....",
    ".......K........",
];

const HEART_GEM: &[&str] = &[
    "................",
    "................",
    "...KKK...KKK....",
    "..K12wK.K122K...",
    ".K1w122K12223K..",
    ".K11222222223K..",
    ".K12222222233K..",
    ".K12222222333K..",
    "..K222222333K...",
    "...K2222333K....",
    "....K22333K.....",
    ".....K233K......",
    "......K3K.......",
    ".......K........",
];

const FEATHER: &[&str] = &[
    "................",
    "...........KK...",
    "..........K1wK..",
    ".........K112K..",
    "........K1122K..",
    ".......K11223K..",
    "......K11223K...",
    ".....K11223K....",
    "....K11223K.....",
    "...K1223K.......",
    "...K223K........",
    "..KuKKK.........",
    ".Ku.............",
    ".K..............",
];

const PACKET: &[&str] = &[
    "................",
    "................",
    "....KKKKKKKK....",
    "...KnnnnnnnnK...",
    "...KhhhhhhhhK...",
    "...KnnnnnnnnK...",
    "...Kn222222nK...",
    "...Kn222222nK...",
    "...Kn211112nK...",
    "...Kn111111nK...",
    "...Kn131111nK...",
    "...Kn211112nK...",
    "...KnnnnnnnnK...",
    "...KhhhhhhhhK...",
    "....KKKKKKKK....",
];

const TURNIP: &[&str] = &[
    "................",
    "......K..K......",
    ".....KgKKgK.....",
    "....KglgKlgK....",
    ".....KglggK.....",
    "......KggK......",
    ".....KbwwwK.....",
    "....KbwwwwwK....",
    "...KbwwwwwwbK...",
    "...KbwwwwwbbK...",
    "...KPbwwwbbPK...",
    "....KPbbbbPK....",
    ".....KPPPPK.....",
    "......KKKK......",
    ".......K........",
];

const CARROT: &[&str] = &[
    "................",
    ".........K.K....",
    "........KgKgK...",
    ".......KglKgK...",
    "......KglgglK...",
    "......KKgggKK...",
    ".....KooKKKK....",
    "....KoYoooK.....",
    "....KYoooCK.....",
    "...KoooooCK.....",
    "...KooCoCK......",
    "..KoooCCK.......",
    "..KoCCCK........",
    ".KoCCK..........",
    ".KCK............",
    "..K.............",
];

const MUSHROOM: &[&str] = &[
    "................",
    "................",
    ".....KKKKKK.....",
    "...KK1020111KK..",
    "..K1001111111K..",
    ".K102011101113K.",
    ".K111111111133K.",
    ".K311111113333K.",
    "..KK33333333KK..",
    "....KKnnnnKK....",
    ".....KnwnnK.....",
    ".....KnnnhK.....",
    ".....KnnhhK.....",
    ".....KhhhhK.....",
    "......KKKK......",
];

const BERRIES: &[&str] = &[
    "................",
    ".........K......",
    "........KgK.....",
    ".......KgK......",
    "....KKKKgKKK....",
    "...K1w2KK1w2K...",
    "...K1223K1223K..",
    "...K2233K2233K..",
    "....K33KKK33K...",
    ".....KK1w2KK....",
    "......K1223K....",
    "......K2233K....",
    ".......K33K.....",
    "........KK......",
];

const MELON: &[&str] = &[
    "................",
    "................",
    ".......KuK......",
    ".....KKKKKK.....",
    "...KK121121KK...",
    "..K1y12112113K..",
    ".K112112112133K.",
    ".K121121121333K.",
    ".K211211213T3TK.",
    ".K3112112133TTK.",
    "..K32332333TTK..",
    "...KK3333TTKK...",
    ".....KKKKKK.....",
];

const PUMPKIN: &[&str] = &[
    "................",
    "................",
    ".......KgK......",
    "......KgK.......",
    "...KKKKKKKKKK...",
    "..K1011K10111K..",
    ".K101111K11112K.",
    ".K11112K111122K.",
    ".K1w112K11w122K.",
    ".K11112K111122K.",
    ".K21122K211222K.",
    "..K2222K22222K..",
    "...KKKKKKKKKK...",
];

const PEPPER: &[&str] = &[
    "................",
    "........KK......",
    ".......KgK......",
    "......KKgKK.....",
    ".....K11g11K....",
    "....K1w11112K...",
    "....K1w11112K...",
    "....K1111112K...",
    "....K1111122K...",
    ".....K11112K....",
    ".....K11122K....",
    "......K112K.....",
    "......K12K......",
    ".......KK.......",
];

const FLOWER: &[&str] = &[
    "................",
    "......KK........",
    ".....K00K.KK....",
    "..KK.K01KK00K...",
    ".K00KK01K000K...",
    ".K000K11K01K....",
    "..K0112211K.....",
    "...KK1221K00K...",
    "..K001111K000K..",
    ".K000K11KK01K...",
    ".K01K.KgK.KK....",
    "..KK..KgK.......",
    "......KgKK......",
    ".......KgK......",
    "........K.......",
];

const BOWL: &[&str] = &[
    "................",
    "................",
    "................",
    "......K.K.......",
    ".....K.K.K......",
    "......K.K.......",
    "...KKKKKKKKKK...",
    "..K1210231201K..",
    "..KnnnnnnnnnnK..",
    "..KhnnnnnnnnhK..",
    "...KhnnnnnnhK...",
    "....KhhhhhhK....",
    ".....KKKKKK.....",
];

const TART: &[&str] = &[
    "................",
    "................",
    "................",
    "................",
    ".....KKKKKK.....",
    "...KK121w21KK...",
    "..K21w2212w21K..",
    "..KYYYYYYYYYYK..",
    "..KYnYnYnYnYnK..",
    "..KCYYYYYYYYCK..",
    "...KCCCCCCCCK...",
    "....KKKKKKKK....",
];

const POTION: &[&str] = &[
    "................",
    "......KKKK......",
    "......KuuK......",
    ".......KK.......",
    "......KwwK......",
    ".....KwKKwK.....",
    "....Kw1111wK....",
    "...Kw1w11112K...",
    "...K11w11112K...",
    "...K11111122K...",
    "...K21111122K...",
    "....K222222K....",
    ".....KKKKKK.....",
];

const SWORD: &[&str] = &[
    "................",
    "............KK..",
    "...........K10K.",
    "..........K102K.",
    ".........K102K..",
    "........K102K...",
    ".......K102K....",
    "..KK..K102K.....",
    "..KYKK102K......",
    "...KYY12K.......",
    "....KYYK........",
    "...KuKKYK.......",
    "..KuK..KYK......",
    ".KuK....KK......",
    ".KK.............",
];

const PICKAXE: &[&str] = &[
    "................",
    "....KKKKKK......",
    "...K00111KK.....",
    "..K0KKKKK12K....",
    "..KK....KuK12K..",
    "........KuK.K2K.",
    ".......KuK...KK.",
    "......KuK.......",
    ".....KuK........",
    "....KuK.........",
    "...KuK..........",
    "..KuK...........",
    "..KK............",
];

const AXE: &[&str] = &[
    "................",
    ".........KKK....",
    "........K001K...",
    ".......K00111K..",
    "......KuK0111K..",
    ".....KuK.K112K..",
    "....KuK...K22K..",
    "...KuK.....KK...",
    "..KuK...........",
    ".KuK............",
    ".KK.............",
];

const HOE: &[&str] = &[
    "................",
    "...........KKKK.",
    "..........K0011K",
    "..........K1112K",
    "..........KuKKK.",
    ".........KuK....",
    "........KuK.....",
    ".......KuK......",
    "......KuK.......",
    ".....KuK........",
    "....KuK.........",
    "...KuK..........",
    "..KuK...........",
    "..KK............",
];

const CAN: &[&str] = &[
    "................",
    "................",
    ".....KKKKK......",
    "....K.....K.....",
    "....K.....K.....",
    "..KKKKKKKKKK..K.",
    "..K00111112K.K1K",
    "..K01111112KK1K.",
    "..K01111112K1K..",
    "..K11111112K1K..",
    "..K11111122KK...",
    "..K11111122K....",
    "..K22222222K....",
    "...KKKKKKKK.....",
];

const CHEST: &[&str] = &[
    "................",
    "................",
    "................",
    "...KKKKKKKKKK...",
    "..KCYCCCCCCYCK..",
    "..KCCCCCCCCCCK..",
    "..KuuuuuuuuuuK..",
    "..KKKKKYYKKKKK..",
    "..KCCCCYYCCCCK..",
    "..KCCCCCCCCCCK..",
    "..KuCCCCCCCCuK..",
    "..KuuuuuuuuuuK..",
    "...KKKKKKKKKK...",
];

const TORCH: &[&str] = &[
    "................",
    ".......KK.......",
    "......KoYK......",
    ".....KoYyYK.....",
    ".....KoyyoK.....",
    "......KYoK......",
    "......KKKK......",
    "......KuCK......",
    "......KuCK......",
    "......KuCK......",
    "......KuCK......",
    "......KmuK......",
    "......KmuK......",
    ".......KK.......",
];

const LAMP: &[&str] = &[
    "................",
    ".....KKKKKK.....",
    "....KaMMMaaK....",
    "...KaMwMaaatK...",
    "...KtaaaaatTK...",
    "....KKKKKKKK....",
    "......KkkK......",
    "......KhkK......",
    "......KhkK......",
    "......KhkK......",
    "......KhkK......",
    ".....KKhkKK.....",
    "....KhhhkkkK....",
    "....KKKKKKKK....",
];

const FENCE: &[&str] = &[
    "................",
    "................",
    "..KK........KK..",
    ".KCCK......KCCK.",
    ".KCuK......KCuK.",
    "KKCuKKKKKKKKCuKK",
    "KCCCCCCCCCCCCCCK",
    "KuuuuuuuuuuuuuuK",
    "KKCuKKKKKKKKCuKK",
    "KKCuKKKKKKKKCuKK",
    "KCCCCCCCCCCCCCCK",
    "KuuuuuuuuuuuuuuK",
    "KKCuKKKKKKKKCuKK",
    ".KCuK......KCuK.",
    ".KuuK......KuuK.",
    "..KK........KK..",
];

const PLANK_TILE: &[&str] = &[
    "................",
    "................",
    "..KKKKKKKKKKKK..",
    "..KCCCCCCCCCCK..",
    "..KYCCCCYCCCCK..",
    "..KuuuuuuuuuuK..",
    "..KCCCYCCCCCCK..",
    "..KCCCCCCYCCCK..",
    "..KuuuuuuuuuuK..",
    "..KCYCCCCCCCCK..",
    "..KCCCCCCCYCCK..",
    "..KuuuuuuuuuuK..",
    "..KKKKKKKKKKKK..",
];

const COBBLE_TILE: &[&str] = &[
    "................",
    "................",
    "..KKKKKKKKKKKK..",
    "..KnnhKKnnnhKK..",
    "..KnhhKKnhhhKK..",
    "..KKKKnnhKKKKK..",
    "..KnhKnhhKnnhK..",
    "..KhhKKKKKnhhK..",
    "..KKKnnnhKKKKK..",
    "..KnnnhhhKnhKK..",
    "..KnhhhhKKhhKK..",
    "..KKKKKKKKKKKK..",
];

const SPRINKLER: &[&str] = &[
    "................",
    "................",
    ".......KK.......",
    "......K00K......",
    ".....K0112K.....",
    "..K..K1112K..K..",
    ".KSK.KK12KK.KSK.",
    "..K.KK1122KK.K..",
    "...K00111122K...",
    "...K01111122K...",
    "...K11111222K...",
    "....K112222K....",
    ".....KKKKKK.....",
];

const WALL_BLOCK: &[&str] = &[
    "................",
    "..KKKKKKKKKKKK..",
    "..K0000K00000K..",
    "..K1111K11111K..",
    "..KKKKKKKKKKKK..",
    "..K00K0000K00K..",
    "..K11K1111K11K..",
    "..KKKKKKKKKKKK..",
    "..K0000K00000K..",
    "..K1111K11111K..",
    "..KKKKKKKKKKKK..",
    "..K2222222222K..",
    "..K3333333333K..",
    "..KKKKKKKKKKKK..",
];

const WORKBENCH: &[&str] = &[
    "................",
    "................",
    "....KK...KK.....",
    "...KhhK.KSSK....",
    "..KKKKKKKKKKKK..",
    ".KYCCCCCCCCCCYK.",
    ".KCCCCCCCCCCCCK.",
    ".KuuuuuuuuuuuuK.",
    ".KKKKKKKKKKKKKK.",
    "..KuK......KuK..",
    "..KuKKKKKKKKuK..",
    "..KuKCCCCCCKuK..",
    "..KuKKKKKKKKuK..",
    "..KmK......KmK..",
    "..KKK......KKK..",
];

const FLOWER_POT: &[&str] = &[
    "................",
    "...KK.KK.KK.....",
    "..KPPKyyKbbK....",
    "..KPwKywKbwK....",
    "...KKgKKgKK.....",
    "....KgKgKgK.....",
    ".....KgggK......",
    "...KKKKKKKKK....",
    "...KCCCCCCCK....",
    "....KCuCCCK.....",
    "....KuCCCuK.....",
    "....KuuuuuK.....",
    ".....KKKKK......",
];

const BENCH: &[&str] = &[
    "................",
    "................",
    "................",
    "..KKKKKKKKKKKK..",
    "..KCCCCCCCCCCK..",
    "..KuuuuuuuuuuK..",
    "..KKKKKKKKKKKK..",
    "..KCCCCCCCCCCK..",
    ".KYCCCCCCCCCCYK.",
    ".KuuuuuuuuuuuuK.",
    ".KKKKKKKKKKKKKK.",
    "..KuK......KuK..",
    "..KuK......KuK..",
    "..KKK......KKK..",
];

const COIN: &[&str] = &[
    "..KKKK..", ".KYyyYK.", "KYyYYYuK", "KYyYYYuK", "KYYYYYuK", "KYYYYuuK", ".KuuuuK.", "..KKKK..",
];

const POINTER: &[&str] = &[
    "K.........",
    "KK........",
    "KwK.......",
    "KwwK......",
    "KwwwK.....",
    "KwwwwK....",
    "KwwwwwK...",
    "KwwwKKKK..",
    "KwKwK.....",
    "KK.KwK....",
    "....KK....",
];

const SEEDLING: &[&str] = &[
    "................",
    "................",
    "................",
    "................",
    "................",
    "................",
    "................",
    "................",
    "................",
    "................",
    "................",
    "................",
    "......K.K.......",
    ".....KlKlK......",
    "......KgK.......",
    "......KgK.......",
];

const SPROUT: &[&str] = &[
    "................",
    "................",
    "................",
    "................",
    "................",
    "................",
    "................",
    "................",
    "........K.......",
    "...KK..KlK......",
    "..KllKKlgK......",
    "...KlgKgK.KK....",
    "....KKgKKKlgK...",
    "......KgKlgK....",
    "......KgKKK.....",
    "......KgK.......",
];

const YOUNG: &[&str] = &[
    "................",
    "................",
    "................",
    "................",
    ".......KK.......",
    "......KllK......",
    "..KK..KlgK.KK...",
    ".KllK.KlgKKllK..",
    ".KlgglKgKlggK...",
    "..KKggKgKgKK....",
    "...KKlgggK......",
    "..KllKgKgKKK....",
    ".KlggKgKggllK...",
    "..KKKKgKKKKK....",
    "......KgK.......",
    "......KgK.......",
];

const TUFT: &[&str] = &[
    "................",
    "................",
    "................",
    "................",
    "................",
    "................",
    "................",
    "................",
    "...t......t.....",
    "..tlt..t..tlt...",
    "..tlgt.tltlgt...",
    ".tlgtgtlgtgt.t..",
    ".tgtgtlgtgtgtlt.",
    "tlgtgtggtgtgtgt.",
    "tggtgtggtggtggt.",
    "................",
];

const BLOSSOM: &[&str] = &[
    "................",
    "................",
    "................",
    "................",
    "................",
    "................",
    "................",
    "................",
    "....0.......0...",
    "...010.....010..",
    "....0...0...0...",
    "....t..010..t...",
    "...lt...0..tl...",
    "....tl..t.lt....",
    ".....t.ltl.t....",
    "......tt.tt.....",
];

const SPARKLE: &[&str] = &["..w..", "..w..", "wwyww", "..w..", "..w.."];

/// Builds every sprite and registers it under a name.
pub fn build(bank: &mut TexBank) -> HashMap<&'static str, TexId> {
    let mut m: HashMap<&'static str, TexId> = HashMap::new();
    let mut add = |name: &'static str, t: Texture| {
        m.insert(name, bank.add(t));
    };

    // Materials.
    add("wood", art(WOOD, NO));
    add("stone", art(STONE, NO));
    add("fiber", art(FIBER, NO));
    add("copper_ore", art(NUGGET, [CLEAR, GOLD, ORANGE, RUST]));
    add("iron_ore", art(NUGGET, [CLEAR, WHITE, SKY, INDIGO]));
    add("gold_ore", art(NUGGET, [CLEAR, CREAM, GOLD, CLAY]));
    add("ember_ore", art(NUGGET, [CLEAR, GOLD, RED, MAROON]));
    add("crystal", art(GEM, [CLEAR, MINT, AQUA, TEAL]));
    add("amber", art(GEM, [CLEAR, CREAM, GOLD, CLAY]));
    add("frost_gem", art(GEM, [CLEAR, WHITE, SKY, BLUE]));
    add("slime_gel", art(GEL, [CLEAR, LIME, GREEN, TEAL]));
    add("bat_wing", art(WING, NO));
    add("bone", art(BONE, NO));
    add("spore", art(SPORE, NO));
    add(
        "heart_crystal",
        art(HEART_GEM, [CLEAR, BLUSH, PINK, CRIMSON]),
    );
    add("stamina_gem", art(HEART_GEM, [CLEAR, CREAM, GOLD, CLAY]));
    add("feather", art(FEATHER, [CLEAR, WHITE, SKY, BLUE]));

    // Seeds: packet with the crop's colours (produce, leaf, accent).
    add("turnip_seeds", art(PACKET, [CLEAR, WHITE, GREEN, PINK]));
    add("carrot_seeds", art(PACKET, [CLEAR, ORANGE, GREEN, CLAY]));
    add("glowcap_spores", art(PACKET, [CLEAR, AQUA, TEAL, MINT]));
    add("berry_seeds", art(PACKET, [CLEAR, SKY, GREEN, BLUE]));
    add("melon_seeds", art(PACKET, [CLEAR, LIME, TEAL, GREEN]));
    add("pumpkin_seeds", art(PACKET, [CLEAR, PINK, GREEN, BLUSH]));
    add("pepper_seeds", art(PACKET, [CLEAR, RED, GREEN, ORANGE]));
    add("lily_bulb", art(PACKET, [CLEAR, WHITE, SKY, AQUA]));
    add(
        "moonbloom_seeds",
        art(PACKET, [CLEAR, LAVENDER, BLUSH, PURPLE]),
    );

    // Produce.
    add("turnip", art(TURNIP, NO));
    add("cave_carrot", art(CARROT, NO));
    add("glowcap", art(MUSHROOM, [MINT, AQUA, WHITE, TEAL]));
    add("crystal_berry", art(BERRIES, [CLEAR, SKY, BLUE, INDIGO]));
    add("moss_melon", art(MELON, [CLEAR, LIME, GREEN, TEAL]));
    add("spore_pumpkin", art(PUMPKIN, [BLUSH, PINK, CRIMSON, CLEAR]));
    add("ember_pepper", art(PEPPER, [CLEAR, RED, CRIMSON, CLEAR]));
    add("frost_lily", art(FLOWER, [WHITE, SKY, AQUA, CLEAR]));
    add("moonbloom", art(FLOWER, [BLUSH, LAVENDER, CREAM, CLEAR]));

    // Cooking.
    add("veggie_stew", art(BOWL, [ORANGE, GOLD, GREEN, CLAY]));
    add("glow_soup", art(BOWL, [AQUA, MINT, TEAL, WHITE]));
    add("ember_curry", art(BOWL, [RED, ORANGE, GOLD, MAROON]));
    add("berry_tart", art(TART, [CLEAR, SKY, BLUE, CLEAR]));
    add("pumpkin_pie", art(TART, [CLEAR, PINK, CRIMSON, CLEAR]));
    add("healing_tonic", art(POTION, [CLEAR, RED, CRIMSON, CLEAR]));
    add("stamina_tonic", art(POTION, [CLEAR, LIME, GREEN, CLEAR]));

    // Tools, per tier: rusty, copper, iron, gold, crystal, ember.
    const TIERS: [[u8; 4]; 6] = [
        [KHAKI, ROSEWOOD, RUST, CLEAR],
        [CREAM, GOLD, CLAY, CLEAR],
        [WHITE, SKY, INDIGO, CLEAR],
        [WHITE, CREAM, GOLD, CLEAR],
        [WHITE, MINT, AQUA, CLEAR],
        [CREAM, GOLD, RED, CLEAR],
    ];
    const SWORDS: [&str; 6] = ["sword0", "sword1", "sword2", "sword3", "sword4", "sword5"];
    const PICKS: [&str; 6] = ["pick0", "pick1", "pick2", "pick3", "pick4", "pick5"];
    const AXES: [&str; 6] = ["axe0", "axe1", "axe2", "axe3", "axe4", "axe5"];
    for t in 0..6 {
        let [a, b, c, _] = TIERS[t];
        add(SWORDS[t], art(SWORD, [a, b, c, CLEAR]));
        add(PICKS[t], art(PICKAXE, [a, b, c, CLEAR]));
        add(AXES[t], art(AXE, [a, b, c, CLEAR]));
    }
    add("hoe", art(HOE, [WHITE, SKY, INDIGO, CLEAR]));
    add("can0", art(CAN, [WHITE, SKY, INDIGO, CLEAR]));
    add("can1", art(CAN, [CREAM, GOLD, CLAY, CLEAR]));
    add("can2", art(CAN, [WHITE, MINT, AQUA, CLEAR]));

    // Placeables.
    add("chest", art(CHEST, NO));
    add("torch", art(TORCH, NO));
    add("lamp", art(LAMP, NO));
    add("fence", art(FENCE, NO));
    add("wood_path", art(PLANK_TILE, NO));
    add("stone_path", art(COBBLE_TILE, NO));
    add("sprinkler", art(SPRINKLER, [GOLD, ORANGE, RUST, CLEAR]));
    add(
        "quality_sprinkler",
        art(SPRINKLER, [WHITE, SKY, INDIGO, CLEAR]),
    );
    add(
        "crystal_sprinkler",
        art(SPRINKLER, [WHITE, AQUA, TEAL, CLEAR]),
    );
    add(
        "stone_wall",
        art(WALL_BLOCK, [SAND, KHAKI, ROSEWOOD, SHADOW]),
    );
    add("wood_wall", art(WALL_BLOCK, [CLAY, RUST, RUST, MAROON]));
    add("workbench", art(WORKBENCH, NO));
    add("flower_pot", art(FLOWER_POT, NO));
    add("bench", art(BENCH, NO));

    // Crop stages and decorations (billboards).
    add("seedling", art(SEEDLING, NO));
    add("sprout", art(SPROUT, NO));
    add("young", art(YOUNG, NO));
    add(
        "young_fungal",
        art(YOUNG, NO).map_colors(|c| match c {
            LIME => LAVENDER,
            GREEN => PURPLE,
            _ => c,
        }),
    );
    add(
        "young_frost",
        art(YOUNG, NO).map_colors(|c| match c {
            LIME => MINT,
            GREEN => AQUA,
            _ => c,
        }),
    );
    add("tuft", art(TUFT, NO));
    add(
        "tuft_dry",
        art(TUFT, NO).map_colors(|c| match c {
            LIME => CREAM,
            GREEN => LIME,
            _ => c,
        }),
    );
    add("blossom_pink", art(BLOSSOM, [PINK, SKY, CLEAR, CLEAR]));
    add("blossom_white", art(BLOSSOM, [WHITE, GOLD, CLEAR, CLEAR]));
    add("blossom_blue", art(BLOSSOM, [SKY, LAVENDER, CLEAR, CLEAR]));
    add("blossom_gold", art(BLOSSOM, [GOLD, WHITE, CLEAR, CLEAR]));

    // UI.
    add("coin", art(COIN, NO));
    add("pointer", art(POINTER, NO));
    add("sparkle", art(SPARKLE, NO));
    m
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn art_rows_fit_16() {
        for (name, rows) in [
            ("wood", WOOD),
            ("nugget", NUGGET),
            ("sword", SWORD),
            ("fence", FENCE),
            ("young", YOUNG),
            ("melon", MELON),
        ] {
            for r in rows {
                assert!(r.chars().count() <= 16, "{name}: row too long: {r}");
            }
        }
    }

    #[test]
    fn sprites_use_palette_only() {
        let mut bank = TexBank::default();
        let m = build(&mut bank);
        assert!(m.len() > 60);
        for id in m.values() {
            assert!(bank.get(*id).data.iter().all(|&c| c < 32 || c == CLEAR));
        }
    }
}
