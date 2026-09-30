//! Pixel art for the treasure of the Hollow: coins, gems, relics, monster bits, dozens of
//! seed packets and crops, dishes, scrolls, little stat symbols and UI bits.
//!
//! Same format as `sprites.rs`: rows of characters, digits `0`-`3` are recolourable slots.

use std::collections::HashMap;

use super::sprites::{
    self, BOWL, GEL, HEART_GEM, MELON, MUSHROOM, PACKET, PEPPER, POTION, TART, YOUNG, art,
};
use crate::palette::*;
use crate::render::{TexBank, TexId, Texture};

const NO: [u8; 4] = sprites::NO;

// ------------------------------------------------------------------------------------------
// Money, gems and treasures
// ------------------------------------------------------------------------------------------

pub const COIN: &[&str] = &[
    "..KKKK..", ".K0001K.", "K00w112K", "K0w1112K", "K011112K", "K111122K", ".K1222K.", "..KKKK..",
];

const FACET: &[&str] = &[
    "................",
    "................",
    "............w...",
    "....KKKKKKKK.www",
    "...K0w001112K.w.",
    "..K00KKKKKK12K..",
    "..K0111111122K..",
    "...K01111122K...",
    "....K011122K....",
    ".....K0122K.....",
    "......K12K......",
    ".......KK.......",
];

const ROUND_GEM: &[&str] = &[
    "................",
    "................",
    "................",
    "......KKKK......",
    "....KK0011KK....",
    "...K0ww01112K...",
    "...K0w011112K...",
    "..K0011111122K..",
    "..K0111111122K..",
    "..K1111111222K..",
    "...K11111222K...",
    "...K12222222K...",
    "....KK2222KK....",
    "......KKKK......",
];

const DIAMOND: &[&str] = &[
    "................",
    ".............w..",
    "............www.",
    "....KKKKKKKK.w..",
    "...K0w001112K...",
    "..KKKKKKKKKKKK..",
    "...K00111122K...",
    "....K011122K....",
    ".....K0122K.....",
    "......K12K......",
    ".......KK.......",
    "..w.............",
    ".www............",
    "..w.............",
];

const STAR: &[&str] = &[
    "................",
    ".......KK.......",
    "......K00K......",
    "......K01K......",
    ".KKKKKK011KKKKK.",
    ".K0000w0111112K.",
    "...K00011112K...",
    "....K001112K....",
    "...K0011K1122K..",
    "..K011K..K122K..",
    "..K11K....K22K..",
    "..KKK......KKK..",
];

const BUTTON: &[&str] = &[
    "................",
    "................",
    "................",
    "......KKKK......",
    "....KKPPPPKK....",
    "...KPbbPPPPcK...",
    "...KPbPPPPPcK...",
    "..KPPPKPPKPPcK..",
    "..KPPPPPPPPPcK..",
    "..KPPPKPPKPPcK..",
    "...KPPPPPPPcK...",
    "...KcPPPPPccK...",
    "....KKccccKK....",
    "......KKKK......",
];

const MARBLE: &[&str] = &[
    "................",
    "................",
    "................",
    "......KKKK......",
    "....KKSwSSKK....",
    "...KSwSSaaBBK...",
    "...KwSaaBBBBK...",
    "..KSSaBBBaaBBK..",
    "..KSaBBaaaBBBK..",
    "..KSaBaaBBBBIK..",
    "...KaBBBBBBIK...",
    "...KBBBBBIIIK...",
    "....KKIIIIKK....",
    "......KKKK......",
];

const DUCK: &[&str] = &[
    "................",
    "................",
    "................",
    "........KKKK....",
    ".......KYYYYK...",
    ".......KYKYYKK..",
    ".......KYYYYooK.",
    "..KK...KYYYKKK..",
    ".KYYK.KYYYYK....",
    ".KYYYKYYYYYYK...",
    ".KYYYYYYYYYYYK..",
    "..KYYYYYYYYYCK..",
    "...KCYYYYYYCK...",
    "....KKKKKKKK....",
];

const TEACUP: &[&str] = &[
    "................",
    "................",
    ".....K..K.......",
    "......K..K......",
    ".....K..K.......",
    "................",
    "..KKKK.KKKKK....",
    "..KwbK.KwwwKKK..",
    "..KwPPwPPwwK.K..",
    "..KwwwwwwwwK.K..",
    "..KbwwwwwwbKKK..",
    "...KbwwwwbK.....",
    "..KKKKKKKKKK....",
    ".KbbbbbbbbbbK...",
    "..KKKKKKKKKK....",
];

const BOAT: &[&str] = &[
    "................",
    ".......K........",
    ".......KK.......",
    ".......KwK......",
    ".......KwwK.....",
    ".......KwwwK....",
    ".......KwwwwK...",
    ".......KwwwwwK..",
    ".......KKKKKKK..",
    ".KKKKKKKKKKKKKK.",
    ".KrrrrrrrrrrrrK.",
    "..KrwrrrrrrrrK..",
    "...KccccccccK...",
    "....KKKKKKKK....",
    ".SS...SS...SS...",
];

const OLD_COIN: &[&str] = &[
    "................",
    "................",
    "................",
    ".....KKKKKK.....",
    "....KYYtCCCK....",
    "...KYCYYYYtuK...",
    "..KYCYYKKYYCuK..",
    "..KCYYK..KYYuK..",
    "..KtYYK..KYtuK..",
    "..KCCYYKKYYCuK..",
    "...KuCYtYYCuK...",
    "....KuuuuuuK....",
    ".....KKKKKK.....",
];

pub(super) const ACORN: &[&str] = &[
    "................",
    "................",
    ".......KK.......",
    "......KuK.......",
    "....KKKuKKK.....",
    "...KCuCuCuCK....",
    "..KuCuCuCuCuK...",
    "..KKKKKKKKKKK...",
    "...KyYYYYYYK....",
    "...KYyYYYYCK....",
    "...KYYYYYYCK....",
    "....KYYYYCK.....",
    ".....KYYCK......",
    "......KKK.......",
];

const MUSIC_BOX: &[&str] = &[
    "..........KK....",
    "..........K.K...",
    "........KKK.....",
    "...KKKKKKKKKK...",
    "..KCYCCCCCCYCK..",
    "..KuuuuuuuuuuK..",
    "..KKKKKKKKKKKK..",
    "..KPbPPPPPPbPK..",
    "..KPPPwPPwPPPK..",
    "..KPPPPPPPPPPKKK",
    "..KPbPPPPPPbPKuK",
    "..KccccccccccK..",
    "..KKKKKKKKKKKK..",
];

const FOSSIL: &[&str] = &[
    "................",
    "................",
    "....KKKKKKKK....",
    "...KnnhnnnnhK...",
    "..KnhnnKnnnnhK..",
    "..KnnnKsKnnnhK..",
    "..KnKKKsKKKnhK..",
    "..KnKssssssKhK..",
    "..KnnKssssKnhK..",
    "..KnnKsKKsKnhK..",
    "..KnKsKnnKsKhK..",
    "...KKKhhhhKKK...",
    "....KKKKKKKK....",
];

const CROWN: &[&str] = &[
    "................",
    "................",
    "................",
    "...K...KK...K...",
    "..K0K.K00K.K0K..",
    "..K0K.K0rK.K0K..",
    "..K00KK00KK00K..",
    "..K0000000000K..",
    "..K1y111r111yK..",
    "..K1111111111K..",
    "..K2222222222K..",
    "...KKKKKKKKKK...",
];

const SCALE: &[&str] = &[
    "................",
    "................",
    ".....KKKKKK.....",
    "....K001111K....",
    "...K0w011112K...",
    "...K01111112K...",
    "...K01111122K...",
    "...K11111222K...",
    "....K1112222K...",
    "....K111222K....",
    ".....K1222K.....",
    "......K22K......",
    ".......KK.......",
];

// ------------------------------------------------------------------------------------------
// Monster bits
// ------------------------------------------------------------------------------------------

const SHELL: &[&str] = &[
    "................",
    "................",
    "................",
    "...KKKKKKKKKK...",
    "..KbPbPbPbPbPK..",
    ".KbwbPbPbPbPbcK.",
    ".KbPbPbPbPbPbcK.",
    "..KbPbPbPbPbcK..",
    "...KbPbPbPbcK...",
    "....KbPbPbcK....",
    ".....KPPPcK.....",
    "....KbKKKKbK....",
    "....KKK..KKK....",
];

const CARAPACE: &[&str] = &[
    "................",
    "................",
    "......KKKK......",
    "....KK1KK1KK....",
    "...K01K00K11K...",
    "...K0wK11K12K...",
    "..K001K11K122K..",
    "..K011K11K122K..",
    "..K011K11K122K..",
    "..K111K11K222K..",
    "...K11K12K22K...",
    "....KK2KK2KK....",
    "......KKKK......",
];

const DUST: &[&str] = &[
    "................",
    "....w.......M...",
    "...wyw.....MwM..",
    "....w...a...M...",
    ".......awa......",
    "..M.....a.......",
    ".MwM............",
    "..M....KKKK.....",
    ".....KKaMMaKK...",
    "....KaMwMMMMaK..",
    "...KaMMMMaMMMaK.",
    "...KaaaMMMMaaaK.",
    "....KKKKKKKKKK..",
];

const HORN: &[&str] = &[
    "................",
    "...........KK...",
    "..........KyK...",
    ".........KyYK...",
    ".........KYYK...",
    "........KYYoK...",
    ".......KYYoK....",
    "......KYoorK....",
    ".....KooorK.....",
    "...KKoorrK......",
    "..KrrrrrK.......",
    "..KmmmmK........",
    "...KKKK.........",
];

const BOMB: &[&str] = &[
    "...........yY...",
    "..........Y.o...",
    ".........K..r...",
    "........KuK.....",
    ".......KKKKK....",
    ".....KKkkkkKK...",
    "....KkkDDkkkkK..",
    "...KkkDwDkkkkkK.",
    "...KkkDDkkkkkkK.",
    "...KkkkkkkkkkkK.",
    "...KkkkkkkkkkkK.",
    "....KkkkkkkkkK..",
    ".....KKkkkkKK...",
    ".......KKKK.....",
];

const GRAVE_DUST: &[&str] = &[
    "................",
    "................",
    "..........KKK...",
    ".........KwwnK..",
    "......KK.KnwK...",
    ".....KwwK.KK....",
    "......KK........",
    ".......KKKKK....",
    ".....KKLDLLLKK..",
    "....KLLwDLDLLLK.",
    "...KLDLLLLDLLDLK",
    "...KDDLDDDLDDDDK",
    "....KKKKKKKKKKK.",
];

const TOOTH: &[&str] = &[
    "................",
    "................",
    "......KKKK......",
    ".....KwwwyK.....",
    "....KwwwwynK....",
    "....KwwwyynK....",
    "....KwwyynnK....",
    ".....KwyynK.....",
    ".....KwyynK.....",
    "......KyynK.....",
    "......KynK......",
    ".......KnK......",
    "........K.......",
];

const CHITIN: &[&str] = &[
    "................",
    "................",
    "....KKKKKKK.....",
    "...K0w00001K....",
    "..K00011111K....",
    "..KKKKKKKKKKK...",
    "..K0w0011112K...",
    ".K00011111122K..",
    ".KKKKKKKKKKKKK..",
    ".K0000111122K...",
    "..K011112222K...",
    "...KKKKKKKKK....",
    "................",
];

// ------------------------------------------------------------------------------------------
// Crops
// ------------------------------------------------------------------------------------------

pub(super) const ROUND_FRUIT: &[&str] = &[
    "................",
    "................",
    ".......KK.......",
    "......K3K.KK....",
    "....KKK3KK33K...",
    "...K00K1K1KKK...",
    "..K0w0111111K...",
    "..K0011111112K..",
    "..K0111111112K..",
    "..K1111111122K..",
    "..K1111111222K..",
    "...K11112222K...",
    "....KK2222KK....",
    "......KKKK......",
];

pub(super) const ROOT: &[&str] = &[
    "................",
    "....KK...KK.....",
    "...K33K.K33K....",
    "...K333K333K....",
    "....KK333KK.....",
    ".....KK3KK......",
    "....KK000KK.....",
    "...K00w0011K....",
    "...K0w00011K....",
    "...K0000111K....",
    "....K00111K.....",
    ".....K011K......",
    "......K1K.......",
    ".......K........",
];

const EARS_ROOT: &[&str] = &[
    "..KK......KK....",
    ".K33K....K33K...",
    ".K3b3K..K3b3K...",
    ".K3b3K..K3b3K...",
    "..K33K..K33K....",
    "...KK3KK3KK.....",
    "....KK000KK.....",
    "...K00w0011K....",
    "...K0K00K11K....",
    "...K000P011K....",
    "....K00111K.....",
    ".....K011K......",
    "......K1K.......",
    ".......K........",
];

pub(super) const POTATO: &[&str] = &[
    "................",
    "................",
    "................",
    "................",
    ".....KKKKKK.....",
    "...KK000001KK...",
    "..K00w0000011K..",
    "..K0000200001K..",
    ".K002000000011K.",
    ".K0000000200111K",
    ".K1000020000111K",
    "..K11000001111K.",
    "...KK1111111KK..",
    ".....KKKKKKK....",
];

pub(super) const CABBAGE: &[&str] = &[
    "................",
    "................",
    "......KKKK......",
    "....KKlllgKK....",
    "...KllKlgglgK...",
    "..KlglKlglgggK..",
    "..KllgKgllKggK..",
    ".KlglgKglKglggK.",
    ".KllggKgKgglggK.",
    ".KglggKKgglgggK.",
    "..KgggglgggggK..",
    "...KKggggggKK...",
    ".....KKKKKK.....",
];

const STRAWBERRY: &[&str] = &[
    "................",
    "......K..K......",
    "....KKgKKgKK....",
    "...KglglgglgK...",
    "...KKrKgKKrKK...",
    "..KrrwrrrrrrrK..",
    "..KrwryrrryrrK..",
    "..KrrrrrrrrrcK..",
    "...KryrrryrcK...",
    "...KrrrrrrrcK...",
    "....KryrrccK....",
    ".....KrrccK.....",
    "......KccK......",
    ".......KK.......",
];

const GRAIN: &[&str] = &[
    "................",
    "....K...K...K...",
    "...K0K.K0K.K0K..",
    "...K1K.K1K.K1K..",
    "...K01KK01KK01K.",
    "...K10KK10K.K1K.",
    "....K1K1KKKK1K..",
    "....KK1K1K1KK...",
    ".....K1K1K1K....",
    ".....K22222K....",
    ".....K1K1K1K....",
    "....K1K.K1K.K...",
    "....KK..KK......",
];

const CORN: &[&str] = &[
    "................",
    ".........KK.....",
    "........KyyK....",
    ".......KyYyYK...",
    "......KyYyYyK...",
    "....K.KYyYyYK...",
    "...KgKKyYyYK....",
    "...KglKYyYyK....",
    "....KglKYyK.....",
    ".....KglKK......",
    "......KgK.......",
    ".......KK.......",
];

pub(super) const EGGPLANT: &[&str] = &[
    "................",
    "..........KK....",
    ".........KgK....",
    "........KggKK...",
    "......KKgglgK...",
    ".....KVVKKKK....",
    "....KVLVVVK.....",
    "...KVLVVVVK.....",
    "..KVLVVVVvK.....",
    "..KVVVVVvvK.....",
    "..KVVVVvvK......",
    "...KvvvvK.......",
    "....KKKK........",
];

const GARLIC: &[&str] = &[
    "................",
    ".......KK.......",
    "......KgK.......",
    "......KnK.......",
    ".....KwwnK......",
    "....KwwnwnK.....",
    "...KwwnwwnnK....",
    "...KwnwwnwnK....",
    "..KwwnwwnwnnK...",
    "..KwnwwnwwnnK...",
    "...KnnwnnwnK....",
    "....KKKKKKK.....",
    "....K.K.K.......",
];

const SUNFLOWER: &[&str] = &[
    "......K..K......",
    "....KKYKKYKK....",
    "...KYYKYYKYYK...",
    "..KKYKuuuuKYKK..",
    "..KYYuCuCuuYYK..",
    "...KYuuCuCuYK...",
    "..KYYuCuuCuYYK..",
    "..KKYKuuuuKYKK..",
    "...KYYKYYKYYK...",
    "....KKYKKYKK....",
    "......KgK.......",
    "......KgK.KK....",
    "......KgKKlK....",
    "......KgKlK.....",
    "......KgKK......",
    "......KK........",
];

pub(super) const ROSE: &[&str] = &[
    "................",
    "......KKKK......",
    ".....KrcrrK.....",
    "....KrrKcrrK....",
    "....KcrrrcrK....",
    "....KrcrrrcK....",
    ".....KrrcrK.....",
    "......KKKK......",
    ".....KgKgK......",
    "..KK..KgK.......",
    ".KglKKgK........",
    "..KKKgK.........",
    ".....KgK........",
    "......KgK.......",
    "......KK........",
];

const POD: &[&str] = &[
    "................",
    "................",
    "................",
    "...KKKKKKKKK....",
    "..K000000000KKK.",
    ".K0KKKKKKKKK0gK.",
    ".K0K1wK1wK1wK1K.",
    ".K0K11K11K11K1K.",
    ".K0KKKKKKKKKK1K.",
    "..K1111111111K..",
    "...KKKKKKKKKK...",
];

pub(super) const PEAR: &[&str] = &[
    "................",
    ".......KK.......",
    "......KuK.KK....",
    ".......KKK3K....",
    "......K00KK.....",
    ".....K0w01K.....",
    ".....K0011K.....",
    "....K001111K....",
    "...K00111112K...",
    "...K01111112K...",
    "...K01111122K...",
    "...K11111222K...",
    "....K112222K....",
    ".....KKKKKK.....",
];

const GOURD: &[&str] = &[
    "................",
    ".......KK.......",
    "......KuK.......",
    "....KKKKKKK.....",
    "...K0011111K....",
    "..K0w0111KK1K...",
    "..K001111KMK1K..",
    ".K00111KKaMwK2K.",
    ".K0111KaaMMKK2K.",
    ".K0111KaMaK1122K",
    ".K1111KKKK11122K",
    "..K11111112222K.",
    "...KK11122222K..",
    ".....KKKKKKKK...",
];

pub(super) const TULIP: &[&str] = &[
    "................",
    "................",
    "....K..KK..K....",
    "...K0KK01KK1K...",
    "...K00K01K11K...",
    "...K000011112K..",
    "...K001111122K..",
    "....K0111122K...",
    ".....K11122K....",
    "......KKKKK.....",
    ".......K3K......",
    "...KK..K3K......",
    "..K33K.K3K.KK...",
    "...KK3KK3KK33K..",
    ".....KK3K33K....",
    ".......KK.......",
];

pub(super) const SPRIG: &[&str] = &[
    "................",
    ".......KK.......",
    "......K00K......",
    ".....K0w01K.....",
    "..KK.K0011K.KK..",
    ".K00KK0112KK01K.",
    ".K0w0K0112K001K.",
    "..K001K12K0112K.",
    "...K0112K2K12K..",
    "....KK12K12KK...",
    ".......K2K......",
    ".......K2K......",
    "......K2K.......",
    "......KK........",
];

const PUFF: &[&str] = &[
    "................",
    "................",
    "................",
    ".....KKKKKK.....",
    "...KK0w00001KK..",
    "..K0ww00100011K.",
    "..K0w00000011K..",
    ".K0001000010001K",
    ".K0000000100011K",
    ".K1001000000011K",
    "..K11000010011K.",
    "...KK11111111K..",
    ".....KKKKKKKK...",
    ".......K22K.....",
    "......KK22KK....",
];

// Young crop stages that are not just a recolour.
const YOUNG_VINE: &[&str] = &[
    "................",
    "................",
    "........K.......",
    ".......KuK......",
    "....KK.KuK......",
    "...KllKKuKKK....",
    "....KlgKuglK....",
    ".....KKgulKK....",
    "...KK.KugKKK....",
    "..KllKKuKllK....",
    "...KlgKugKK.....",
    "....KKguKK......",
    ".....KKuKK......",
    "......KuK.......",
    "......KgK.......",
    "......KK........",
];

const YOUNG_LEAFY: &[&str] = &[
    "................",
    "................",
    "................",
    "................",
    "................",
    "......KKKK......",
    "....KKllglKK....",
    "...KlglKlgllK...",
    "..KllgKlglKglK..",
    "..KgKlglKgglgK..",
    ".KlglKgKlgKgllK.",
    ".KggKlggKglKggK.",
    "..KKggKKggKKgK..",
    "....KKgKKgKKK...",
    "......KgK.......",
    "......KK........",
];

const YOUNG_GRAIN: &[&str] = &[
    "................",
    "................",
    "...K.....K......",
    "..KlK...KlK..K..",
    "..KlK...KlK.KlK.",
    "..KgK.K.KgK.KlK.",
    "..KgKKlKKgK.KgK.",
    "...KgKlKKgK.KgK.",
    "...KgKgKKgKKgK..",
    "....KgKgKgKKgK..",
    "....KgKgKgKgK...",
    ".....KgggggK....",
    "......KgggK.....",
    "......KgggK.....",
    ".......KgK......",
    ".......KK.......",
];

const YOUNG_BUD: &[&str] = &[
    "................",
    "................",
    "................",
    "......KKK.......",
    ".....KbPbK......",
    ".....KPbPK......",
    "......KgK.......",
    "..KK..KgK..KK...",
    ".KllK.KgK.KllK..",
    "..KlgKKgKKglK...",
    "...KKgKgKgKK....",
    ".....KKgKK......",
    "......KgK.......",
    "......KgK.......",
    "......KgK.......",
    "......KK........",
];

// ------------------------------------------------------------------------------------------
// Kitchen
// ------------------------------------------------------------------------------------------

const BREAD: &[&str] = &[
    "................",
    "................",
    "................",
    "....KKKKKKKK....",
    "...K00100000K...",
    "..K0120120121K..",
    "..K1201201211K..",
    ".K111111111111K.",
    ".KCCCCCCCCCCCCK.",
    ".KuuuuuuuuuuuuK.",
    "..KKKKKKKKKKKK..",
];

pub(super) const BAKED: &[&str] = &[
    "................",
    "................",
    "................",
    "......KKKK......",
    "....KKyyyyKK....",
    "...KyyyYYyyyK...",
    "..KhyyYyyYyyhK..",
    "..KhhyyYYyyhhK..",
    ".KuhhhhhhhhhhuK.",
    ".KuuhhhhhhhhuuK.",
    "..KuuuuuuuuuuK..",
    "...KKKKKKKKKK...",
];

const MUFFIN: &[&str] = &[
    "................",
    "................",
    ".....KKKKKK.....",
    "...KKnBnnBnKK...",
    "..KnnBnnnnBnnK..",
    "..KnBnnBnnnBnK..",
    "..KKnnnnnnnnKK..",
    "...KbKbKbKbKK...",
    "...KbbbbbbbbK...",
    "...KbKbKbKbbK...",
    "....KbbbbbbK....",
    "....KKKKKKKK....",
];

pub(super) const CUP: &[&str] = &[
    "................",
    "......K..K......",
    ".......K..K.....",
    "......K..K......",
    "................",
    "...KKKKKKKKK....",
    "...KwMMMMMwKKK..",
    "...KwaaaaawK.K..",
    "...KwwwwwwwK.K..",
    "...KbwwwwwbKKK..",
    "....KbwwwbK.....",
    "...KKKKKKKKK....",
    "..KwwwwwwwwwK...",
    "...KKKKKKKKK....",
];

pub(super) const GLASS: &[&str] = &[
    "................",
    "..........K.....",
    ".........K......",
    "....KKKKKKKK....",
    "....KwYYYYwK....",
    "....KwoYYoYK....",
    "....KwYYYYYK....",
    "....KwYoYYYK....",
    ".....KYYYYK.....",
    ".....KwYYoK.....",
    "......KKKK......",
    ".......KK.......",
    ".......KK.......",
    ".....KKKKKK.....",
];

const POPCORN: &[&str] = &[
    "................",
    "................",
    "....KK.KK.KK....",
    "...KwyKywKywK...",
    "..KwyywyywyyK...",
    "..KKKKKKKKKKKK..",
    "..KrwrwrwrwrwK..",
    "...KrwrwrwrwK...",
    "...KrwrwrwrwK...",
    "....KrwrwrwK....",
    "....KKKKKKKK....",
];

pub(super) const CAKE: &[&str] = &[
    "................",
    "................",
    "........K.......",
    ".......K3K......",
    "......KK3KK.....",
    "....KK00000KK...",
    "..KK000000000K..",
    "..K0000000000K..",
    "..K1111111111K..",
    "..K2222222222K..",
    "..K1111111111K..",
    "..K1111111111K..",
    "..KKKKKKKKKKKK..",
];

const SKEWER: &[&str] = &[
    "............K...",
    "...........KnK..",
    "........KKKKK...",
    ".......KrrwrK...",
    ".......KrrrcK...",
    "......KKKKKK....",
    ".....KMwaK......",
    ".....KaaaK......",
    "....KKKKK.......",
    "...KyyYK........",
    "...KYYCK........",
    "..KKKKK.........",
    ".KnK............",
    "KnK.............",
    "KK..............",
];

const COOKIE: &[&str] = &[
    "................",
    "................",
    "................",
    ".....KKKKKK.....",
    "...KKYYyYYYKK...",
    "..KYyYKYYKYYCK..",
    "..KYYYYYYYYYCK..",
    ".KYKYYYKYYYKYCK.",
    ".KYYYYYYYYYYYCK.",
    ".KYYKYYYKYYYCCK.",
    "..KCYYYYYYYCCK..",
    "...KKCCCCCCKK...",
    ".....KKKKKK.....",
];

// ------------------------------------------------------------------------------------------
// Magic
// ------------------------------------------------------------------------------------------

const SCROLL: &[&str] = &[
    "................",
    "..KKKKKKKKKKKK..",
    ".KyynnnnnnnnnhK.",
    ".KynnnnnnnnnnhK.",
    "..KKKKKKKKKKKK..",
    "...KnhhhhhhnK...",
    "...KnnnnnnnnK...",
    "..KK00K22K00KK..",
    "..K1100K2K0011K.",
    "...KKKK22KKKK...",
    "...KnhhhhhhnK...",
    "...KnnnnnnnnK...",
    "..KKKKKKKKKKKK..",
    ".KyynnnnnnnnnhK.",
    ".KynnnnnnnnnnhK.",
    "..KKKKKKKKKKKK..",
];

const ENCHANT_TABLE: &[&str] = &[
    "................",
    "..w.........M...",
    ".wyw...KK..MwM..",
    "..w...KLLK..M...",
    ".....KLwLLK.....",
    "....KKLLLLKK....",
    "..KKVVVVVVVVKK..",
    ".KvVVLVVVVLVVvK.",
    ".KvvvvvvvvvvvvK.",
    "..KKKKKKKKKKKK..",
    "...KuK....KuK...",
    "...KuK....KuK...",
    "..KKuKK..KKuKK..",
    "..KuuuK..KuuuK..",
    "...KKK....KKK...",
];

// ------------------------------------------------------------------------------------------
// Little stat symbols (8x8) and empty-slot outlines
// ------------------------------------------------------------------------------------------

const ST_DAMAGE: &[&str] = &[
    "......KK", ".....KwK", "....KwK.", "KK.KwK..", ".KKwK...", "..KYK...", ".KuKK...", "KuK.....",
];
const ST_CRIT: &[&str] = &[
    "...KK...", "..KYYK..", "KKKYYKKK", "KYYYYYYK", ".KYYYYK.", ".KYKKYK.", "KYK..KYK", "KK....KK",
];
const ST_CRITDMG: &[&str] = &[
    "K..KK..K", ".KKooKK.", ".KoyyoK.", "KoyYYyoK", "KoyYYyoK", ".KoyyoK.", ".KKooKK.", "K..KK..K",
];
const ST_HASTE: &[&str] = &[
    "........", "....KKK.", "..KKwwSK", ".KwwSSK.", "KwSSSK..", ".KKSK...", "...K....", "........",
];
const ST_LIFESTEAL: &[&str] = &[
    ".KK.KK..", "KPPKPPK.", "KPwPPPK.", "KPPPPPK.", ".KPPPK..", "..KPK.K.", "...K.KrK", ".....KK.",
];
const ST_BURN: &[&str] = &[
    "...K....", "..KoK...", "..KoKK..", ".KoYoK..", ".KoYYoK.", "KoYyYoK.", "KoYyyYoK", ".KKKKKK.",
];
const ST_CHILL: &[&str] = &[
    "...K....", ".K.S.K..", "..KSK...", "KSSwSSK.", "..KSK...", ".K.S.K..", "...K....", "........",
];
const ST_SHOCK: &[&str] = &[
    "....KKK.", "...KYK..", "..KYK...", ".KYYYYK.", "...KYK..", "..KYK...", ".KYK....", ".KK.....",
];
const ST_DEFENSE: &[&str] = &[
    "KKKKKKK.", "KSSwSSK.", "KSwSSBK.", "KSSSSBK.", ".KSSBK..", ".KSSBK..", "..KBK...", "...K....",
];
const ST_VITALITY: &[&str] = &[
    ".KK.KK..", "KPPKPPK.", "KPwPPcK.", "KPPPPcK.", ".KPPcK..", "..KcK...", "...K....", "........",
];
const ST_STAMINA: &[&str] = &[
    "...K....", ".K.K.K..", "..KYK...", "KKYyYKK.", "..KYK...", ".K.K.K..", "...K....", "........",
];
const ST_WISDOM: &[&str] = &[
    "..KKK...", ".KLLK...", "KLLK....", "KLK.....", "KLLK....", ".KLLKKK.", "..KKLK..", "........",
];
const ST_REGEN: &[&str] = &[
    "..KKK...", "..KlK...", "KKKlKKK.", "KlllllK.", "KKKlKKK.", "..KlK...", "..KKK...", "........",
];
const ST_SWIFT: &[&str] = &[
    "........", "..KK....", "..KMK...", "..KMK...", "..KMMKK.", ".KMMMMMK", ".KKKKKKK", "........",
];
const ST_DODGE: &[&str] = &[
    "........", "KK...KK.", "KaK.KaK.", "KaaKaaK.", ".KaKaK..", "KaaKaaK.", "KKK.KKK.", "........",
];
const ST_BLOCK: &[&str] = &[
    "..KKK...", ".KSSSK..", "KSKKKSK.", "KSKwKSK.", "KSKKKSK.", ".KSSSK..", "..KKK...", "........",
];
const ST_THORNS: &[&str] = &[
    "K...K...", ".KlK.K..", "..KlK...", "KlKgKlK.", "..KgK...", ".K.g.K..", "..KgK...", "...K....",
];
const ST_LUCK: &[&str] = &[
    ".KK.KK..", "KllKllK.", "KlgKglK.", ".KKgKK..", "KlgKglK.", "KllKllK.", ".KKgKK..", "...KgK..",
];
const ST_GREED: &[&str] = &[
    "..KKKK..", ".KYyYYK.", ".KKKKKK.", ".KYyYYK.", ".KKKKKK.", ".KYyYYK.", ".KCCCCK.", "..KKKK..",
];
const ST_POWER: &[&str] = &[
    "KKKKKK..", "KeeeesK.", "KKKKKK..", "..KuK...", "..KuK...", "..KuK...", "..KKK...", "........",
];
const ST_FRUGAL: &[&str] = &[
    "CCCC....", "...C....", "..C.....", ".C......", "CCCC.CCC", "......C.", ".....C..", ".....CCC",
];
const ST_REACH: &[&str] = &[
    "...K....", "..KMK...", ".KKMKK..", "KMMMMMK.", ".KKMKK..", "..KMK...", "...K....", "........",
];
const ST_CAPACITY: &[&str] = &[
    "...K....", "..KSK...", "..KSK...", ".KSwSK..", "KSwSSBK.", "KSSSSBK.", ".KSBBK..", "..KKK...",
];
const ST_BOUNTY: &[&str] = &[
    "..KrK...", ".KrrKgK.", "KKKKKKKK", "KuCuCuCK", ".KuCuCK.", ".KCuCuK.", "..KKKK..", "........",
];
const ST_GROWTH: &[&str] = &[
    "........", ".KK.KK..", "KllKllK.", ".KlKlK..", "..KgK...", "..KgK...", "KKKKKKK.", "KuuuuuK.",
];
const ST_FORAGE: &[&str] = &[
    "...K....", "..KuK...", ".KCuCK..", "KKKKKKK.", ".KyYYK..", ".KYYCK..", "..KCK...", "...K....",
];
const ST_FOCUS: &[&str] = &[
    "........", "..KKKK..", ".KwwwwK.", "KwwLLwwK", "KwwLKwwK", ".KwwwwK.", "..KKKK..", "........",
];
const ST_SPIRIT: &[&str] = &[
    "...K....", "..KaK...", ".KaMaK..", "KaMwMaK.", "KaMMMaK.", ".KaaaK..", "..KKK...", "........",
];
const SOCKET: &[&str] = &[
    "...K....", "..KLK...", ".KL.LK..", "KL...LK.", ".KL.LK..", "..KLK...", "...K....", "........",
];

const GHOST_HEAD: &[&str] = &[
    "................",
    "................",
    "................",
    "......RRRR......",
    ".....R....R.....",
    "....R......R....",
    "....R......R....",
    "..RRRRRRRRRRRR..",
    ".R............R.",
    "..RRRRRRRRRRRR..",
];
const GHOST_CHEST: &[&str] = &[
    "................",
    "....RRR..RRR....",
    "..RR...RR...RR..",
    ".R............R.",
    "R..R........R..R",
    "R..R........R..R",
    ".RR.R......R.RR.",
    "....R......R....",
    "....R......R....",
    "....R......R....",
    "....RRRRRRRR....",
];
const GHOST_LEGS: &[&str] = &[
    "................",
    "................",
    "....RRRRRRRR....",
    "....R......R....",
    "....R......R....",
    "....R..RR..R....",
    "....R..RR..R....",
    "....R..RR..R....",
    "....R..RR..R....",
    "....RRRRRRRR....",
];
const GHOST_FEET: &[&str] = &[
    "................",
    "................",
    "................",
    "....RRRR........",
    "....R..R........",
    "....R..R........",
    "....R..R........",
    "....R..RRRRR....",
    "....R......RR...",
    "....RRRRRRRRR...",
];
const GHOST_SHIELD: &[&str] = &[
    "................",
    "................",
    "...RRRRRRRRRR...",
    "...R........R...",
    "...R........R...",
    "...R........R...",
    "....R......R....",
    ".....R....R.....",
    "......R..R......",
    ".......RR.......",
];
const GHOST_SCROLL: &[&str] = &[
    "................",
    "................",
    "..RRRRRRRRRRRR..",
    "..R..........R..",
    "..RRRRRRRRRRRR..",
    "...R........R...",
    "...R.RRRRRR.R...",
    "...R........R...",
    "...R.RRRRR..R...",
    "..RRRRRRRRRRRR..",
    "..R..........R..",
    "..RRRRRRRRRRRR..",
];

// The new biomes' treasures.
const LAUREL: &[&str] = &[
    "................",
    "................",
    "....KK....KK....",
    "...K01K..K10K...",
    "..K01KK..KK10K..",
    "..K1K......K1K..",
    ".K01K......K10K.",
    ".K1K........K1K.",
    ".K01K......K10K.",
    "..K1K......K1K..",
    "..K01KK..KK10K..",
    "...K121KK121K...",
    "....KKK33KKK....",
    "......K33K......",
    ".......KK.......",
];

const SPOOL: &[&str] = &[
    "................",
    "................",
    "....KKKKKKKK....",
    "....K333333K....",
    "....KKKKKKKK....",
    ".....K0101K.....",
    ".....K1010K.....",
    ".....K0101K.....",
    ".....K1012K.....",
    ".....K0122K.....",
    "....KKKKKKKK..K.",
    "....K333333K.K1K",
    "....KKKKKKKKK1K.",
    ".........KK11K..",
    "..........KKK...",
];

const AMULET: &[&str] = &[
    "......K..K......",
    ".....K3KK3K.....",
    "....K3K..K3K....",
    "...K3K....K3K...",
    "...K3K....K3K...",
    "....K3K..K3K....",
    ".....KKKKKK.....",
    "....K3KKKK3K....",
    "...K00K1K112K...",
    "..K001K1K1122K..",
    "..K011K1K1122K..",
    "...K11K1K122K...",
    "....KK1K12KK....",
    "......KKKK......",
];

const SUN_DISC: &[&str] = &[
    "................",
    ".......KK.......",
    "..K...K33K...K..",
    ".K3K.KKKKKK.K3K.",
    "..KKK000011KKK..",
    "...K00001112K...",
    "..K0000011112K..",
    "KK30000111112KKK",
    "K3K0011111122K3K",
    "..K0111111122K..",
    "...K11111222K...",
    "..KKK222222KKK..",
    ".K3K.KKKKKK.K3K.",
    "..K...K33K...K..",
    ".......KK.......",
];

const VOID_PEARL: &[&str] = &[
    "................",
    "................",
    "................",
    ".....KKKKKK.....",
    "....K0w1111K....",
    "...K0w111111K...",
    "...K01113112K...",
    "...K01133312K...",
    "...K11113112K...",
    "...K11111122K...",
    "....K112222K....",
    ".....KKKKKK.....",
];

const RIFT_STAR: &[&str] = &[
    "................",
    "..........K.....",
    ".........K3K....",
    "..........K.KK..",
    ".......KKKKK0K..",
    "......K00112K...",
    ".....K00112K....",
    "....K00112K.....",
    "...K0012KK......",
    "..K0112K........",
    "..K012K....K....",
    "..K12K....K3K...",
    "...KK......K....",
];

/// Adds every icon in this file to the sprite table.
pub fn build(bank: &mut TexBank, m: &mut HashMap<&'static str, TexId>) {
    let mut add = |name: &'static str, t: Texture| {
        m.insert(name, bank.add(t));
    };

    // Coins.
    add("coin_copper", art(COIN, [PEACH, CLAY, RUST, CLEAR]));
    add("coin_silver", art(COIN, [WHITE, SKY, BLUE, CLEAR]));
    add("coin_gold", art(COIN, [CREAM, GOLD, CLAY, CLEAR]));

    // Gems.
    add("ruby", art(FACET, [PINK, RED, MAROON, CLEAR]));
    add("sapphire", art(FACET, [SKY, BLUE, INDIGO, CLEAR]));
    add("emerald", art(FACET, [LIME, GREEN, TEAL, CLEAR]));
    add("topaz", art(FACET, [CREAM, ORANGE, RUST, CLEAR]));
    add("amethyst", art(FACET, [LAVENDER, PURPLE, GRAPE, CLEAR]));
    add("moonstone", art(ROUND_GEM, [WHITE, BLUSH, LAVENDER, CLEAR]));
    add("star_diamond", art(DIAMOND, [WHITE, SKY, AQUA, CLEAR]));

    // Relics.
    add("lost_button", art(BUTTON, NO));
    add("glass_marble", art(MARBLE, NO));
    add("rubber_duck", art(DUCK, NO));
    add("chipped_teacup", art(TEACUP, NO));
    add("toy_boat", art(BOAT, NO));
    add("ancient_coin", art(OLD_COIN, NO));
    add("golden_acorn", art(ACORN, NO));
    add("music_box", art(MUSIC_BOX, NO));
    add("star_fossil", art(FOSSIL, NO));
    add("tiny_crown", art(CROWN, [GOLD, CREAM, CLAY, CLEAR]));
    add("moon_pearl", art(ROUND_GEM, [WHITE, CREAM, BLUSH, CLEAR]));
    add("dragon_scale", art(SCALE, [SALMON, RED, MAROON, CLEAR]));
    add("golden_laurel", art(LAUREL, [CREAM, GOLD, CLAY, RUST]));
    add("ariadnes_thread", art(SPOOL, [CREAM, GOLD, CLAY, RUST]));
    add("scarab_amulet", art(AMULET, [SKY, BLUE, INDIGO, GOLD]));
    add("sun_disc", art(SUN_DISC, [CREAM, GOLD, CLAY, ORANGE]));
    add(
        "void_pearl",
        art(VOID_PEARL, [SLATE, SHADOW, INK, LAVENDER]),
    );
    add(
        "rift_star",
        art(RIFT_STAR, [BLUSH, LAVENDER, PURPLE, WHITE]),
    );

    // Monster bits.
    add("crab_shell", art(SHELL, NO));
    add(
        "beetle_shell",
        art(CARAPACE, [LAVENDER, PURPLE, GRAPE, CLEAR]),
    );
    add("shroom_cap", art(MUSHROOM, [WHITE, RED, WHITE, CRIMSON]));
    add("wisp_dust", art(DUST, NO));
    add("imp_horn", art(HORN, NO));
    add("ectoplasm", art(GEL, [CLEAR, WHITE, BLUSH, LAVENDER]));
    add(
        "golem_heart",
        art(HEART_GEM, [CLEAR, SAND, KHAKI, ROSEWOOD]),
    );
    add("grave_dust", art(GRAVE_DUST, NO));
    add("bomb", art(BOMB, NO));
    add("goblin_tooth", art(TOOTH, NO));
    add("chitin", art(CHITIN, [LIME, GREEN, TEAL, CLEAR]));

    // Seeds.
    let packets: [(&'static str, [u8; 3]); 29] = [
        ("potato_seeds", [SAND, GREEN, KHAKI]),
        ("radish_seeds", [PINK, GREEN, WHITE]),
        ("cabbage_seeds", [LIME, GREEN, TEAL]),
        ("tomato_seeds", [RED, GREEN, CRIMSON]),
        ("strawberry_seeds", [RED, GREEN, CREAM]),
        ("wheat_seeds", [GOLD, CLAY, CREAM]),
        ("corn_seeds", [GOLD, GREEN, CREAM]),
        ("blueberry_seeds", [BLUE, GREEN, INDIGO]),
        ("eggplant_seeds", [PURPLE, GREEN, LAVENDER]),
        ("garlic_seeds", [WHITE, GREEN, SAND]),
        ("sunflower_seeds", [GOLD, GREEN, RUST]),
        ("rose_seeds", [CRIMSON, GREEN, PINK]),
        ("pea_seeds", [LIME, GREEN, BLUSH]),
        ("mossberry_seeds", [SALMON, TEAL, RED]),
        ("bunnyroot_seeds", [BLUSH, LIME, WHITE]),
        ("prism_pear_seeds", [MINT, AQUA, WHITE]),
        ("geode_gourd_seeds", [LAVENDER, SLATE, AQUA]),
        ("puffball_spores", [WHITE, LAVENDER, CREAM]),
        ("jelly_spores", [PINK, PURPLE, BLUSH]),
        ("truffle_spores", [SHADOW, PURPLE, KHAKI]),
        ("flame_tulip_seeds", [ORANGE, RED, GOLD]),
        ("lava_lemon_seeds", [GOLD, RED, ORANGE]),
        ("magma_melon_seeds", [RED, MAROON, GOLD]),
        ("snow_pea_seeds", [MINT, SKY, WHITE]),
        ("ice_plum_seeds", [SKY, BLUE, WHITE]),
        ("frost_mint_seeds", [MINT, TEAL, WHITE]),
        ("starfruit_seeds", [GOLD, TEAL, CREAM]),
        ("ghost_pepper_seeds", [WHITE, SLATE, BLUSH]),
        ("ancient_grain_seeds", [MINT, TEAL, GOLD]),
    ];
    for (name, [a, b, c]) in packets {
        add(name, art(PACKET, [CLEAR, a, b, c]));
    }

    // Produce.
    add("potato", art(POTATO, [SAND, KHAKI, ROSEWOOD, CLEAR]));
    add("radish", art(ROOT, [PINK, CRIMSON, CLEAR, GREEN]));
    add("cabbage", art(CABBAGE, NO));
    add("tomato", art(ROUND_FRUIT, [SALMON, RED, CRIMSON, GREEN]));
    add("strawberry", art(STRAWBERRY, NO));
    add("wheat", art(GRAIN, [CREAM, GOLD, RUST, CLEAR]));
    add("corn", art(CORN, NO));
    add(
        "blueberry",
        art(sprites::BERRIES, [CLEAR, SKY, INDIGO, SLATE]),
    );
    add("eggplant", art(EGGPLANT, NO));
    add("garlic", art(GARLIC, NO));
    add("sunflower", art(SUNFLOWER, NO));
    add("rose", art(ROSE, NO));
    add("sweet_pea", art(POD, [LIME, GREEN, CLEAR, CLEAR]));
    add(
        "mossberry",
        art(sprites::BERRIES, [CLEAR, SALMON, RED, MAROON]),
    );
    add("bunnyroot", art(EARS_ROOT, [BLUSH, PINK, CLEAR, LIME]));
    add("prism_pear", art(PEAR, [WHITE, MINT, AQUA, TEAL]));
    add("geode_gourd", art(GOURD, [LAVENDER, SLATE, INDIGO, CLEAR]));
    add("puffball", art(PUFF, [WHITE, CREAM, KHAKI, CLEAR]));
    add("jelly_shroom", art(MUSHROOM, [WHITE, PINK, BLUSH, CRIMSON]));
    add("truffle", art(POTATO, [SHADOW, INK, KHAKI, CLEAR]));
    add("flame_tulip", art(TULIP, [GOLD, ORANGE, RED, GREEN]));
    add("lava_lemon", art(ROUND_FRUIT, [CREAM, GOLD, ORANGE, RED]));
    add("magma_melon", art(MELON, [CLEAR, GOLD, RED, MAROON]));
    add("snow_pea", art(POD, [MINT, AQUA, CLEAR, CLEAR]));
    add("ice_plum", art(ROUND_FRUIT, [WHITE, SKY, BLUE, TEAL]));
    add("frost_mint", art(SPRIG, [WHITE, MINT, AQUA, CLEAR]));
    add("starfruit", art(STAR, [CREAM, GOLD, ORANGE, CLEAR]));
    add("ghost_pepper", art(PEPPER, [CLEAR, WHITE, BLUSH, CLEAR]));
    add("ancient_grain", art(GRAIN, [MINT, TEAL, GOLD, CLEAR]));

    // Young plants.
    add("young_vine", art(YOUNG_VINE, NO));
    add("young_leafy", art(YOUNG_LEAFY, NO));
    add("young_grain", art(YOUNG_GRAIN, NO));
    add("young_bud", art(YOUNG_BUD, NO));
    add(
        "young_ember",
        art(YOUNG, NO).map_colors(|c| match c {
            LIME => GOLD,
            GREEN => ORANGE,
            _ => c,
        }),
    );
    add(
        "young_crystal",
        art(YOUNG, NO).map_colors(|c| match c {
            LIME => SKY,
            GREEN => BLUE,
            _ => c,
        }),
    );
    add(
        "young_ruin",
        art(YOUNG, NO).map_colors(|c| match c {
            GREEN => TEAL,
            _ => c,
        }),
    );

    // Kitchen.
    add("mana_tonic", art(POTION, [CLEAR, LAVENDER, PURPLE, CLEAR]));
    add("baked_potato", art(BAKED, NO));
    add("fresh_bread", art(BREAD, [CREAM, GOLD, SAND, CLEAR]));
    add("garlic_bread", art(BREAD, [CREAM, GOLD, GREEN, CLEAR]));
    add("garden_salad", art(BOWL, [LIME, GREEN, RED, TEAL]));
    add("tomato_soup", art(BOWL, [RED, ORANGE, GREEN, CRIMSON]));
    add("shortcake", art(CAKE, [WHITE, SAND, PINK, RED]));
    add("carrot_cake", art(CAKE, [CREAM, CLAY, WHITE, ORANGE]));
    add("blueberry_muffin", art(MUFFIN, NO));
    add("corn_chowder", art(BOWL, [GOLD, CREAM, GREEN, CLAY]));
    add("mint_tea", art(CUP, NO));
    add("popcorn", art(POPCORN, NO));
    add("mushroom_skewer", art(SKEWER, NO));
    add("truffle_risotto", art(BOWL, [CREAM, SAND, SHADOW, KHAKI]));
    add("starfruit_tart", art(TART, [CLEAR, CREAM, GOLD, CLEAR]));
    add("lava_lemonade", art(GLASS, NO));
    add("plum_pudding", art(BOWL, [SKY, BLUE, WHITE, INDIGO]));
    add("spooky_chili", art(BOWL, [RED, MAROON, WHITE, PLUM]));
    add("sunflower_cookies", art(COOKIE, NO));
    add("pear_crumble", art(TART, [CLEAR, MINT, AQUA, CLEAR]));

    // Magic.
    add("weapon_scroll", art(SCROLL, [RED, CRIMSON, GOLD, CLEAR]));
    add("armor_scroll", art(SCROLL, [SKY, BLUE, GOLD, CLEAR]));
    add("tool_scroll", art(SCROLL, [LIME, GREEN, GOLD, CLEAR]));
    add("enchant_table", art(ENCHANT_TABLE, NO));
    add("wish_star", art(STAR, [WHITE, LAVENDER, PURPLE, CLEAR]));

    // Stat symbols.
    for (name, rows) in [
        ("st_damage", ST_DAMAGE),
        ("st_crit", ST_CRIT),
        ("st_critdmg", ST_CRITDMG),
        ("st_haste", ST_HASTE),
        ("st_lifesteal", ST_LIFESTEAL),
        ("st_burn", ST_BURN),
        ("st_chill", ST_CHILL),
        ("st_shock", ST_SHOCK),
        ("st_defense", ST_DEFENSE),
        ("st_vitality", ST_VITALITY),
        ("st_stamina", ST_STAMINA),
        ("st_wisdom", ST_WISDOM),
        ("st_regen", ST_REGEN),
        ("st_swift", ST_SWIFT),
        ("st_dodge", ST_DODGE),
        ("st_block", ST_BLOCK),
        ("st_thorns", ST_THORNS),
        ("st_luck", ST_LUCK),
        ("st_greed", ST_GREED),
        ("st_power", ST_POWER),
        ("st_frugal", ST_FRUGAL),
        ("st_reach", ST_REACH),
        ("st_capacity", ST_CAPACITY),
        ("st_bounty", ST_BOUNTY),
        ("st_growth", ST_GROWTH),
        ("st_forage", ST_FORAGE),
        ("st_focus", ST_FOCUS),
        ("st_spirit", ST_SPIRIT),
        ("socket", SOCKET),
    ] {
        add(name, art(rows, NO));
    }
    for (name, rows) in [
        ("ghost_head", GHOST_HEAD),
        ("ghost_chest", GHOST_CHEST),
        ("ghost_legs", GHOST_LEGS),
        ("ghost_feet", GHOST_FEET),
        ("ghost_shield", GHOST_SHIELD),
        ("ghost_scroll", GHOST_SCROLL),
    ] {
        add(name, art(rows, NO));
    }
}

#[cfg(test)]
pub fn all_templates() -> Vec<(&'static str, &'static [&'static str], usize)> {
    vec![
        ("coin", COIN, 8),
        ("facet", FACET, 16),
        ("round_gem", ROUND_GEM, 16),
        ("diamond", DIAMOND, 16),
        ("star", STAR, 16),
        ("button", BUTTON, 16),
        ("marble", MARBLE, 16),
        ("duck", DUCK, 16),
        ("teacup", TEACUP, 16),
        ("boat", BOAT, 16),
        ("old_coin", OLD_COIN, 16),
        ("acorn", ACORN, 16),
        ("music_box", MUSIC_BOX, 16),
        ("fossil", FOSSIL, 16),
        ("crown", CROWN, 16),
        ("scale", SCALE, 16),
        ("shell", SHELL, 16),
        ("carapace", CARAPACE, 16),
        ("dust", DUST, 16),
        ("horn", HORN, 16),
        ("grave_dust", GRAVE_DUST, 16),
        ("bomb", BOMB, 16),
        ("tooth", TOOTH, 16),
        ("chitin", CHITIN, 16),
        ("round_fruit", ROUND_FRUIT, 16),
        ("root", ROOT, 16),
        ("ears_root", EARS_ROOT, 16),
        ("potato", POTATO, 16),
        ("cabbage", CABBAGE, 16),
        ("strawberry", STRAWBERRY, 16),
        ("grain", GRAIN, 16),
        ("corn", CORN, 16),
        ("eggplant", EGGPLANT, 16),
        ("garlic", GARLIC, 16),
        ("sunflower", SUNFLOWER, 16),
        ("rose", ROSE, 16),
        ("pod", POD, 16),
        ("pear", PEAR, 16),
        ("gourd", GOURD, 16),
        ("tulip", TULIP, 16),
        ("sprig", SPRIG, 16),
        ("puff", PUFF, 16),
        ("young_vine", YOUNG_VINE, 16),
        ("young_leafy", YOUNG_LEAFY, 16),
        ("young_grain", YOUNG_GRAIN, 16),
        ("young_bud", YOUNG_BUD, 16),
        ("bread", BREAD, 16),
        ("baked", BAKED, 16),
        ("muffin", MUFFIN, 16),
        ("cup", CUP, 16),
        ("glass", GLASS, 16),
        ("popcorn", POPCORN, 16),
        ("cake", CAKE, 16),
        ("skewer", SKEWER, 16),
        ("cookie", COOKIE, 16),
        ("scroll", SCROLL, 16),
        ("enchant_table", ENCHANT_TABLE, 16),
        ("st_damage", ST_DAMAGE, 8),
        ("st_frugal", ST_FRUGAL, 8),
        ("socket", SOCKET, 8),
        ("ghost_chest", GHOST_CHEST, 16),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn item_art_is_tidy() {
        for (name, rows, size) in all_templates() {
            assert!(rows.len() <= size, "{name} has too many rows");
            for r in rows {
                assert!(r.chars().count() <= size, "{name}: row too long: {r}");
            }
        }
        let mut bank = TexBank::default();
        let mut m = HashMap::new();
        build(&mut bank, &mut m);
        assert!(m.len() > 150);
        for id in m.values() {
            assert!(bank.get(*id).data.iter().all(|&c| c < 32 || c == CLEAR));
        }
    }
}
