//! Gear art: an icon for every weapon, piece of armour and tool, and what each looks like
//! on the hero in 3D: blades, wands and staffs in hand, hats on the head, boots on the feet,
//! shields on the arm, and clothes swapped onto the body.

use std::collections::HashMap;
use std::f32::consts::{FRAC_PI_2, PI};

use glam::{Mat4, Vec3};

use super::models::{HERO, Look, body_tex, lathe, limb_tex, skin_box};
use super::sprites::{AXE, CAN, HOE, NO, PICKAXE, SWORD, art};
use crate::palette::*;
use crate::render::{Mesh, TexBank, TexId, Texture, UvRect};

// ------------------------------------------------------------------------------------------
// Icons
// ------------------------------------------------------------------------------------------

const SABRE: &[&str] = &[
    "................",
    "............KK..",
    "...........K0K..",
    "..........K01K..",
    "..........K01K..",
    ".........K012K..",
    "........K012K...",
    ".......K012K....",
    "..KK..K012K.....",
    "..K3KK012K......",
    "...K3312K.......",
    "...KuK3K........",
    "..KuK.KK........",
    ".KuK............",
    ".KK.............",
    "................",
];

const TWIG_SWORD: &[&str] = &[
    "................",
    "............K...",
    "...........KlK..",
    "..........KlgK..",
    ".........KCKK...",
    "........KCuK....",
    ".......KCuK.....",
    "......KCuK......",
    ".....KCuK.......",
    "..KK.KuK........",
    ".KgKKCK.........",
    "..KlKuK.........",
    "...KuKK.........",
    "..KuK...........",
    ".KmK............",
    ".KK.............",
];

const CARROT_BLADE: &[&str] = &[
    "................",
    "............K...",
    "...........KoK..",
    "..........KoCK..",
    ".........KoYoK..",
    "........KooCK...",
    ".......KoYoK....",
    "......KooCK.....",
    ".....KoYoK......",
    "....KooCK.......",
    "..KKKCoK........",
    ".KglKKK.........",
    "..KglK..........",
    ".KlKgK..........",
    "KlK.KK..........",
    "KK..............",
];

const LEEK: &[&str] = &[
    "............KK..",
    "...........KlgK.",
    "..........KlggK.",
    ".........KlglK..",
    "........KlggK...",
    ".......KlglK....",
    "......KwllK.....",
    ".....KwwwK......",
    "....KwwwK.......",
    "...KwwyK........",
    "..KwwyK.........",
    ".KnyyK..........",
    ".KnnK...........",
    "..KK............",
];

const BAGUETTE: &[&str] = &[
    "................",
    "............KK..",
    "...........KYYK.",
    "..........KYnYK.",
    ".........KYnYK..",
    "........KYYnK...",
    ".......KYnYK....",
    "......KYnYK.....",
    ".....KYYnK......",
    "....KYnYK.......",
    "...KCYnK........",
    "..KCCYK.........",
    ".KCCCK..........",
    ".KuuK...........",
    "..KK............",
];

const WAND_GEM: &[&str] = &[
    "................",
    "...........KKK..",
    "..........K01wK.",
    ".........K0112K.",
    "........KK122K..",
    ".......K3KK2K...",
    "......K3K.KK....",
    ".....K3K........",
    "....K3K.........",
    "...K3K..........",
    "..K3K...........",
    ".KuK............",
    ".KK.............",
];

const WAND_STAR: &[&str] = &[
    "..........K.....",
    ".........K0K....",
    "......KKK001KKK.",
    "......K0000111K.",
    ".......K01112K..",
    ".......K11112K..",
    "......K12K.K12K.",
    "......KKK...KKK.",
    ".....K3K........",
    "....K3K.........",
    "...K3K..........",
    "..K3K...........",
    ".K3K............",
    ".KK.............",
];

const WAND_BUBBLE: &[&str] = &[
    "..........KKK...",
    ".........KSwSK..",
    "........KS...SK.",
    "........Kw...SK.",
    "........KS...SK.",
    ".........KSSSK..",
    "..KK.....KuKK...",
    ".KwSK...KuK.....",
    ".KSSK..KuK......",
    "..KK..KuK.......",
    ".....KuK........",
    "....KuK.........",
    "...KuK..........",
    "..KuK...........",
    "..KK............",
];

const WAND_MUSHROOM: &[&str] = &[
    "................",
    "..........KKKK..",
    "........KKaMMaKK",
    ".......KaMwMMMaK",
    ".......KMMMaMMaK",
    "........KKnnnKK.",
    "........KunnK...",
    ".......KuKKK....",
    "......KuK.......",
    ".....KuK........",
    "....KuK.........",
    "...KuK..........",
    "..KuK...........",
    "..KK............",
];

const WAND_CANDY: &[&str] = &[
    "..........KKKK..",
    ".........KPwPPK.",
    "........KPwbwPPK",
    "........KPbwbwPK",
    "........KPPbwPPK",
    ".........KPPPPK.",
    "........KwKKKK..",
    ".......KrK......",
    "......KwK.......",
    ".....KrK........",
    "....KwK.........",
    "...KrK..........",
    "..KwK...........",
    "..KK............",
];

const WAND_MOON: &[&str] = &[
    "...........KK...",
    "..........KbbK..",
    ".........KbLLK..",
    "........KbLKK...",
    "........KbLK....",
    "........KbLLK...",
    ".........KbLLKK.",
    "........KKKLLK..",
    ".......KyK.KK...",
    "......KyK.......",
    ".....KyK........",
    "....KyK.........",
    "...KyK..........",
    "..KyK...........",
    "..KK............",
];

const STAFF_ORB: &[&str] = &[
    "..........KKKK..",
    ".........K01wK..",
    "........K0011KK.",
    "........K0112K..",
    ".........K122K..",
    "........KKKKK...",
    ".......K3K......",
    "......K3K.......",
    "......K3K.......",
    ".....K3K........",
    "....K3K.........",
    "....K3K.........",
    "...K3K..........",
    "..K3K...........",
    "..K3K...........",
    "..KK............",
];

const STAFF_SUNFLOWER: &[&str] = &[
    "........K.KK.K..",
    ".......KYKYYKYK.",
    "......KYYKCCKYYK",
    ".......KYKCCKYK.",
    "......KYYKCCKYYK",
    ".......KYKKKKYK.",
    "........K.KgK.K.",
    "..........KgK...",
    ".........KgK....",
    "........KgKlK...",
    ".......KgKlK....",
    "......KgK.K.....",
    ".....KgK........",
    "....KgK.........",
    "...KgK..........",
    "...KK...........",
];

const STAFF_MUSHROOM: &[&str] = &[
    "........KKKKKK..",
    "......KKrrwrrrKK",
    ".....KrwrrrrwrrK",
    ".....KrrrrwrrrrK",
    "......KKccccccK.",
    "........KKnnKK..",
    ".........KnnK...",
    "........KuKK....",
    ".......KuK......",
    "......KuK.......",
    ".....KuK........",
    "....KuK.........",
    "...KuK..........",
    "..KuK...........",
    "..KK............",
];

const STAFF_BLOOM: &[&str] = &[
    ".........K..K...",
    "........KbKKbK..",
    ".......KbLKKLbK.",
    "......KKLLyyLLKK",
    ".......KbLyyLbK.",
    "......KbLLKKLLbK",
    ".......KKbKKbKK.",
    ".........KgK....",
    "........KgKlK...",
    ".......KgKlK....",
    "......KgK.K.....",
    ".....KgK........",
    "....KgK.........",
    "...KgK..........",
    "...KK...........",
];

const SHIELD_ROUND: &[&str] = &[
    "................",
    ".....KKKKKK.....",
    "....K333333K....",
    "...K30000113K...",
    "..K3001111123K..",
    "..K3011111123K..",
    "..K3011yy1123K..",
    "..K3011yy1223K..",
    "..K3011111223K..",
    "..K3012222223K..",
    "...K32222223K...",
    "....K333333K....",
    ".....KKKKKK.....",
];

const SHIELD_KITE: &[&str] = &[
    "................",
    "...KKKKKKKKKK...",
    "...K33333333K...",
    "...K30001113K...",
    "...K30011123K...",
    "...K30113123K...",
    "...K31133323K...",
    "...K31113223K...",
    "....K311223K....",
    "....K312223K....",
    ".....K3223K.....",
    "......K33K......",
    ".......KK.......",
];

const POT_LID: &[&str] = &[
    "................",
    "................",
    "................",
    ".......KK.......",
    "......KkkK......",
    ".....KKhhKK.....",
    "...KKhnnnnhKK...",
    "..KhnwnnnnnnhK..",
    ".KhnnnnnnnnnnhK.",
    ".KhhhhhhhhhhhhK.",
    "..KRRRRRRRRRRK..",
    "...KKKKKKKKKK...",
];

const TURTLE: &[&str] = &[
    "................",
    "................",
    ".....KKKKKK.....",
    "...KKtggggtKK...",
    "..KtlllttlllgtK.",
    ".KtllltgglllltK.",
    ".KttttgllgtttgK.",
    ".KglltgllgtllgK.",
    ".KglltttttlllgK.",
    "..KgtlllllltgK..",
    "...KKggggggKK...",
    ".....KKKKKK.....",
];

const LEAF_SHIELD: &[&str] = &[
    "................",
    ".......KK.......",
    "......KllK......",
    ".....KlglgK.....",
    "....KlggKglK....",
    "...KlgggKgglK...",
    "...KlggKKKglK...",
    "..KlgggKgKgglK..",
    "..KlggKKgKKglK..",
    "...KlggKgKglK...",
    "....KlggKgglK...",
    ".....KlgKgK.....",
    "......KKgKK.....",
    ".......KgK......",
    "........K.......",
];

const MUSHROOM_SHIELD: &[&str] = &[
    "................",
    "................",
    ".....KKKKKK.....",
    "...KKrrwwrrKK...",
    "..KrrwwrrrrwrK..",
    "..KrrrrrrrwwrK..",
    ".KrwrrrrrrrrrrK.",
    ".KrrrrwwrrrrcrK.",
    ".KrrrrwwrrrcccK.",
    ".KcrrrrrrrccccK.",
    "..KccrrrrccccK..",
    "...KKccccccKK...",
    ".....KKKKKK.....",
];

const STRAW_HAT: &[&str] = &[
    "................",
    "................",
    "................",
    "................",
    ".....KKKKKK.....",
    "....KYnnnnYK....",
    "....KnYnnnnK....",
    "...KKrrrrrrKK...",
    ".KKnYnnnnnnYnKK.",
    "KnnYnnYnnYnnnYnK",
    ".KKhhhhhhhhhhKK.",
    "...KKKKKKKKKK...",
];

const FLOWER_CROWN: &[&str] = &[
    "................",
    "................",
    "................",
    "................",
    "...KK..KK..KK...",
    "..KPPKKyYKKSSK..",
    "..KPyPKYYKKSwSK.",
    "...KPKgKKgKKSK..",
    "..KggKglgKlgKgK.",
    ".KglgKgKgKgKglK.",
    "..KKKKKKKKKKKK..",
];

const LEAF_CAP: &[&str] = &[
    "................",
    "................",
    "........KK......",
    ".......KgK......",
    "......KgK.KK....",
    ".....KKKKKllK...",
    "....KllgglKKK...",
    "...KlglglgglK...",
    "..KlggglgglgK...",
    "..KggglgglgggK..",
    ".KKKKKKKKKKKKKK.",
];

const HOOD_EARS: &[&str] = &[
    "................",
    "..KK........KK..",
    "..K0K......K0K..",
    "..K03K....K30K..",
    "..K000KKKK000K..",
    "..K0000000000K..",
    ".K000000000000K.",
    ".K00KKKKKKKK00K.",
    ".K0KeeeeeeeeK0K.",
    ".K0KeKeeeeKeK0K.",
    ".K0KeeeeeeeeK0K.",
    ".K1KeseeeeseK1K.",
    ".K11KeeeeeeK11K.",
    "..KK1KKKKKK1KK..",
    "....KK....KK....",
];

const BUNNY_HOOD: &[&str] = &[
    "...KK......KK...",
    "..KwbK....KbwK..",
    "..KwbK....KbwK..",
    "..KwbK....KbwK..",
    "..KwwK....KwwK..",
    "..KwwKKKKKKwwK..",
    ".KwwwwwwwwwwwwK.",
    ".KwwKKKKKKKKwwK.",
    ".KwKeeeeeeeeKwK.",
    ".KwKeKeeeeKeKwK.",
    ".KwKeseeeeseKwK.",
    ".KnKeeeeeeeeKnK.",
    ".KnnKeeeeeeKnnK.",
    "..KKnKKKKKKnKK..",
    "....KK....KK....",
];

const FROG_HOOD: &[&str] = &[
    "................",
    "...KKK....KKK...",
    "..KwwgK..KgwwK..",
    "..KwKgKKKKgKwK..",
    "..KggggggggggK..",
    ".KgllllllllllgK.",
    ".KglKKKKKKKKlgK.",
    ".KgKeeeeeeeeKgK.",
    ".KgKeKeeeeKeKgK.",
    ".KgKeseeeeseKgK.",
    ".KgKeeeeeeeeKgK.",
    ".KggKeeeeeeKggK.",
    "..KKgKKKKKKgKK..",
    "....KK....KK....",
];

const HELM: &[&str] = &[
    "................",
    "................",
    "......KKKK......",
    "....KK0011KK....",
    "...K00111112K...",
    "...K01111112K...",
    "..K3333333333K..",
    "..K1111111112K..",
    "..K1KKKKKKKK2K..",
    "..K1112KK2112K..",
    "..K1112KK2122K..",
    "...KK12KK21KK...",
    ".....KKKKKK.....",
];

const HELM_HORNS: &[&str] = &[
    "................",
    "..K..........K..",
    "..KwK......KwK..",
    "...KwKKKKKKwK...",
    "....KK0011KK....",
    "...K00111112K...",
    "...K01111112K...",
    "..K3333333333K..",
    "..K1111111112K..",
    "..K1KKKKKKKK2K..",
    "..K1112KK2112K..",
    "..K1112KK2122K..",
    "...KK12KK21KK...",
    ".....KKKKKK.....",
];

const CAP_HAT: &[&str] = &[
    "................",
    "................",
    "................",
    ".....KKKKKK.....",
    "...KKrrwwrrKK...",
    "..KrwwrrrrwwrK..",
    ".KrrrrrwwrrrrrK.",
    ".KrwwrrwwrrrwrK.",
    "KrrwwrrrrrrrrrcK",
    "KcrrrrrrwwrrrccK",
    ".KKccccccccccKK.",
    "...KnnnnnnnnK...",
    "....KKKKKKKK....",
];

const MINER_HAT: &[&str] = &[
    "................",
    "................",
    "......KKKK......",
    "....KKYYYYKK....",
    "...KYYwwYYYYK...",
    "..KYYKyyKYYYYK..",
    "..KYKyywyKYYYK..",
    "..KYKyyyyKYYYK..",
    "..KYYKKKKYYYYK..",
    ".KKKKKKKKKKKKKK.",
    ".KCCCCCCCCCCCCK.",
    "..KKKKKKKKKKKK..",
];

const WIZARD_HAT: &[&str] = &[
    "...........KK...",
    "..........K1K...",
    ".........K11K...",
    "........K101K...",
    ".......K1111K...",
    "......K1y111K...",
    "......K11121K...",
    ".....K1112y1K...",
    "....K11211112K..",
    "....K1y111112K..",
    "...K333333333K..",
    ".KK22222222222K.",
    "K22222222222222K",
    ".KKKKKKKKKKKKKK.",
];

const CIRCLET: &[&str] = &[
    "................",
    "................",
    "................",
    "................",
    "................",
    "................",
    ".......KK.......",
    "......K33K......",
    "..KKKKK33KKKKK..",
    ".K0001K33K1112K.",
    ".K0111KKKK1122K.",
    "..KKKKKKKKKKKK..",
];

const CROWN_HAT: &[&str] = &[
    "................",
    "................",
    "................",
    "...K...KK...K...",
    "..K0K.K00K.K0K..",
    "..K0K.K03K.K0K..",
    "..K00KK00KK00K..",
    "..K0000000000K..",
    "..K1y1113111yK..",
    "..K1111111111K..",
    "..K2222222222K..",
    "...KKKKKKKKKK...",
];

const TUNIC: &[&str] = &[
    "................",
    "...KKKK..KKKK...",
    "..K0000KK0000K..",
    ".K000000000001K.",
    "K00K00000001K01K",
    "K01K00000011K11K",
    "KKKK01000011KKKK",
    "...K00000011K...",
    "...K33333333K...",
    "...K01000112K...",
    "...K00000112K...",
    "...K11111122K...",
    "...KKKKKKKKKK...",
];

const SWEATER: &[&str] = &[
    "................",
    "...KKKK..KKKK...",
    "..K3333KK3333K..",
    ".K000000000002K.",
    "K11K11111111K12K",
    "K00K00000002K02K",
    "KKKK11111112KKKK",
    "...K00000002K...",
    "...K11111112K...",
    "...K00000002K...",
    "...K33333333K...",
    "...KKKKKKKKKK...",
];

const MAIL: &[&str] = &[
    "................",
    "...KKKK..KKKK...",
    "..K0101KK1010K..",
    ".K010101010102K.",
    "K01K10101010K12K",
    "K10K01010102K22K",
    "KKKK10101012KKKK",
    "...K01010102K...",
    "...K33333333K...",
    "...K01010112K...",
    "...K10101122K...",
    "...KKKKKKKKKK...",
];

const PLATE: &[&str] = &[
    "................",
    "...KKKK..KKKK...",
    "..K3333KK3333K..",
    ".K300000000113K.",
    "K33K0w000011K33K",
    "K33K00000112K33K",
    "KKKK00001112KKKK",
    "...K30001113K...",
    "...K33333333K...",
    "...K01111112K...",
    "...K11111122K...",
    "...KKKKKKKKKK...",
];

const ROBE: &[&str] = &[
    "...KKKK..KKKK...",
    "..K0000KK0000K..",
    ".K000000000002K.",
    "K00K00300000K02K",
    "K01K00000302K12K",
    "KKKK00000012KKKK",
    "...K03000012K...",
    "...K00000312K...",
    "...K00300012K...",
    "..K0000000112K..",
    "..K0030000112K..",
    ".K000000031122K.",
    ".K000000001122K.",
    ".KKKKKKKKKKKKKK.",
];

const COAT: &[&str] = &[
    "................",
    "...KKKK..KKKK...",
    "..K0000KK0000K..",
    ".K000000300002K.",
    "K00K00003001K02K",
    "K01K00000301K12K",
    "KKKK00003011KKKK",
    "...K00000312K...",
    "...K00003012K...",
    "...K00000312K...",
    "...K00003112K...",
    "...KKKKKKKKKK...",
];

const PONCHO: &[&str] = &[
    "................",
    "......KKKK......",
    ".....K0000K.....",
    "....K000000K....",
    "...K00000000K...",
    "..K3333333333K..",
    ".K000000000001K.",
    "K00000000000112K",
    "K33333333333333K",
    "K00000000000112K",
    "KK0KK0KK0KK0KKKK",
    ".K.K..K..K..K...",
];

const TROUSERS: &[&str] = &[
    "................",
    "................",
    "...KKKKKKKKKK...",
    "...K33333333K...",
    "...K00000012K...",
    "...K00000012K...",
    "...K0001K012K...",
    "...K001KK012K...",
    "...K001KK012K...",
    "...K001KK012K...",
    "...K012KK112K...",
    "...KKKKKKKKKK...",
];

const TROUSERS_PATCHED: &[&str] = &[
    "................",
    "................",
    "...KKKKKKKKKK...",
    "...K33333333K...",
    "...K00000012K...",
    "...K0ss00012K...",
    "...K0ss1K012K...",
    "...K001KK012K...",
    "...K001KKY12K...",
    "...K001KKYY2K...",
    "...K012KK112K...",
    "...KKKKKKKKKK...",
];

const GREAVES: &[&str] = &[
    "................",
    "................",
    "...KKKKKKKKKK...",
    "...K22222222K...",
    "...K00000112K...",
    "...K0w01K012K...",
    "...K001KK012K...",
    "...K333KK333K...",
    "...K001KK012K...",
    "...K001KK012K...",
    "...K012KK122K...",
    "...KKKKKKKKKK...",
];

const GRASS_SKIRT: &[&str] = &[
    "................",
    "................",
    "...KKKKKKKKKK...",
    "...KuCuCuCuCK...",
    "...KglgglgglK...",
    "..KglgKglgKglK..",
    "..KgKlgKgKlgKK..",
    "..KgKgKglKgKgK..",
    "..KlKgKgKgKlgK..",
    "..KgKlKgKlKgKK..",
    "..KKKKKKKKKKKK..",
];

const BLOOMERS: &[&str] = &[
    "................",
    "................",
    "................",
    "...KKKKKKKKKK...",
    "...KggggggggK...",
    "..KoYoCoYoCooK..",
    ".KoYoCoYoKoCooK.",
    ".KoYoCoKKoCoCoK.",
    ".KooCooKKooCooK.",
    "..KooCoKKoCooK..",
    "...KKKK..KKKK...",
];

const BOOTS: &[&str] = &[
    "................",
    "................",
    "....KKKKKK......",
    "....K0001K......",
    "....K0012K......",
    "....K0012K......",
    "....K0012K......",
    "....K00112KKKK..",
    "....K0011111w1K.",
    "....K0011111112K",
    "....K3333333333K",
    "....KKKKKKKKKKKK",
];

const FROG_SLIPPERS: &[&str] = &[
    "................",
    "................",
    "................",
    "................",
    "........KKKKKK..",
    ".......KwKwwKwK.",
    ".......KKgKKgKK.",
    "..KKKKKgggggggK.",
    "..KllllllllllgK.",
    ".KglllKlllllggK.",
    ".KgggggggggggK..",
    ".KnnnnnnnnnnnK..",
    "..KKKKKKKKKKK...",
];

const BUNNY_SLIPPERS: &[&str] = &[
    "................",
    "................",
    "..........KK.KK.",
    ".........KbKKbK.",
    ".........KbKKbK.",
    "........KwwwwwwK",
    "..KKKKKKwwwwKwwK",
    "..KwwwwwwwwwwwwK",
    ".KwwwwwwwwwwPwwK",
    ".KnwwwwwwwwwwwK.",
    ".KnnnnnnnnnnnnK.",
    "..KKKKKKKKKKKK..",
];

const WINGED_BOOTS: &[&str] = &[
    "................",
    "................",
    "....KKKKKK......",
    "....KwwwSK.KK...",
    "....KwwSBKKwwK..",
    "....KwwSBKwwK...",
    "....KwwSBKKK....",
    "....KwwSSBKKKK..",
    "....KwwSSSSSwSK.",
    "....KwwSSSSSSSBK",
    "....KnnnnnnnnnnK",
    "....KKKKKKKKKKKK",
];

const SPROUT_HOE: &[&str] = &[
    "................",
    "...........KKKK.",
    "..........K0011K",
    "..........K1112K",
    "..........KuKKK.",
    ".........KuK....",
    "...KK...KuK.....",
    "..KllK.KuK......",
    "...KlgKuK.......",
    ".....KuK........",
    "....KuK.........",
    "...KuK..........",
    "..KuK...........",
    "..KK............",
];

const DUCK_CAN: &[&str] = &[
    "................",
    "................",
    "........KKK.....",
    ".......KYYYK....",
    ".......KYKYKK...",
    "...KK..KYYYooK..",
    "..KYYK.KYYKKK...",
    "..KYKKKKYYK.....",
    "..KYYYYYYYYYK...",
    "..KYYYYYYYYYYK..",
    "..KYYYYYYYYYCK..",
    "...KYYYYYYYCK...",
    "....KKKKKKKK....",
];

const TEAPOT_CAN: &[&str] = &[
    "................",
    "................",
    ".......KK.......",
    "......KbbK......",
    "....KKKKKKKK....",
    "...KPPPPPPPPK...",
    "..KPwPPPPPPPPKKK",
    "KKKPPbPPbPPPPKbK",
    "KbKPPPPPPPPPPKK.",
    "KbKPPbPPbPPPPK..",
    ".KKPPPPPPPPPPK..",
    "...KcPPPPPPcK...",
    "....KKKKKKKK....",
];

const FROG_CAN: &[&str] = &[
    "................",
    "...KKK...KKK....",
    "..KwwgK.KgwwK...",
    "..KwKgKKKgKwK...",
    "..KgggggggggKKK.",
    "..KgllllllllKlgK",
    "..KglKgggKllKKK.",
    "..KgllKKKlllK...",
    "..KgllllllllK...",
    "..KggllllllgK...",
    "...KgggggggK....",
    "....KKKKKKK.....",
];

const CLOUD_CAN: &[&str] = &[
    "................",
    "................",
    "......KKKK......",
    "....KKwwwwK.....",
    "...KwwwwwwwKK...",
    "..KwwSwwwwwwwK..",
    ".KwwwwwwwSwwwwK.",
    ".KwSwwwwwwwwwSK.",
    "..KSSSSSSSSSSK..",
    "...KKKKKKKKKK...",
    "....B...B...B...",
    "...B...B...B....",
];

const SICKLE: &[&str] = &[
    "................",
    "....KKKKK.......",
    "...K00011KK.....",
    "..K0KKKK112K....",
    "..K1K...KK2K....",
    "..KK.....K2K....",
    ".........KuK....",
    "........KuK.....",
    ".......KuK......",
    "......KuK.......",
    ".....KuK........",
    "....KmK.........",
    "....KK..........",
];

// ------------------------------------------------------------------------------------------
// Looks in 3D
/// A fishing rod: 0-1 the pole, 2 the grip, 3 the reel.
const ROD: &[&str] = &[
    "..............KK",
    ".............K0K",
    "............K01K",
    "...........K01Kw",
    "..........K01K.w",
    ".........K01K..w",
    "........K01K...w",
    ".......K01K....w",
    "......K01K.....w",
    ".....K01K......w",
    "....K01K.....KrK",
    "...K01K3K....KwK",
    "..K22K333K....K.",
    ".K22K.K3K.......",
    "K22K...K........",
    "KKK.............",
];

// The biome sets: the Marble Labyrinth's bronze and marble, the Sunscorch Canyon's linen,
// gold and lapis, and the Starless Rift's obsidian and amethyst.
const PLUMED_HELM: &[&str] = &[
    "................",
    "....KKKKKKK.....",
    "...K3333333K....",
    "..K33c3333c3K...",
    "..KKKKKKKKK3K...",
    "...KK00111KKK...",
    "..K001111112K...",
    "..K011111112K...",
    "..K1KKKK1112K...",
    "..K1K..K1122K...",
    "..K11KK11122K...",
    "..K11K.K1122K...",
    "...K1K.K122K....",
    "....KK..KKK.....",
];

const CUIRASS: &[&str] = &[
    "................",
    "...KKKK..KKKK...",
    "..K3333KK3333K..",
    ".K300010010013K.",
    "K33K0110011K33K.",
    "K33K0000001K33K.",
    "KKKK1000101KKKK.",
    "...K0101011K....",
    "...K1000011K....",
    "...K3333333K....",
    "...K3K3K3K3K....",
    "...K3K3K3K3K....",
    "...KKKKKKKKK....",
];

const GREAVES_BRONZE: &[&str] = &[
    "................",
    "...KKKK..KKKK...",
    "..K0012KK0012K..",
    "..K3333KK3333K..",
    "..K0112KK0112K..",
    "..K0112KK0112K..",
    "..Kw112KKw112K..",
    "..K0112KK0112K..",
    "..K3333KK3333K..",
    "..K0112KK0112K..",
    "..K0122KK0122K..",
    "...K12K..K12K...",
    "....KK....KK....",
];

const WINGED_SANDAL: &[&str] = &[
    "................",
    ".KK.............",
    "K33K............",
    "K333KK..........",
    ".K3333K.........",
    "..KK333KKK......",
    "....KK3K1K......",
    "....K1K1KK......",
    "....K101KKK.....",
    "....K1K1K1KKK...",
    "....K01K1K011KK.",
    "....K111111K011K",
    "....KhhhhhhhhhhK",
    "....KKKKKKKKKKK.",
];

const ASPIS: &[&str] = &[
    "................",
    ".....KKKKKK.....",
    "...KK222222KK...",
    "..K2233333322K..",
    "..K2311111132K..",
    ".K231KKKKK1322K.",
    ".K231K111K1322K.",
    ".K231K1K1K1322K.",
    ".K231K1KKK1322K.",
    ".K231K11111322K.",
    "..K2311111132K..",
    "..K2233333322K..",
    "...KK222222KK...",
    ".....KKKKKK.....",
];

const LABRYS: &[&str] = &[
    "................",
    ".KK..........KK.",
    "K01K...KK...K10K",
    "K011K.K33K.K110K",
    "K0111KK33KK1110K",
    "K0111111111110K.",
    "K0112KK33KK2110K",
    "K012K.K33K.K210K",
    "K01K..K33K..K10K",
    ".KK...K33K...KK.",
    "......K33K......",
    "......K33K......",
    "......K33K......",
    "......KKKK......",
];

const QUILL: &[&str] = &[
    "................",
    "...........KK...",
    "..........K01K..",
    ".........K0112K.",
    "........K01212K.",
    ".......K011212K.",
    "......K01212K...",
    ".....K0112KK....",
    "....K0112K......",
    "....KuK2K.......",
    "...KuKKK........",
    "..KuK...........",
    ".KuK............",
    ".KK.............",
];

const NEMES: &[&str] = &[
    "................",
    ".......KK.......",
    "......K33K......",
    "....KKK33KKK....",
    "...K0000000K....",
    "..K011111110K...",
    "..K000000000K...",
    ".K01KKKKKKK10K..",
    ".K00KeeeeeK00K..",
    ".K01KeKeKeK10K..",
    "K000KeeeeeK000K.",
    "K011KKeeeKK110K.",
    "K000K.KKK.K000K.",
    "KKKKK.....KKKKK.",
];

const WRAPS: &[&str] = &[
    "................",
    "...KKKK..KKKK...",
    "..K0000KK0000K..",
    ".KYtYtYttYtYtYK.",
    "K0KBYBYBBYBYBK0K",
    "K00KKYYYYYYKK01K",
    "KKKK10000012KKKK",
    "...K12222222K...",
    "...K00000001K...",
    "...K12222222K...",
    "...K00000012K...",
    "...KKKKKKKKKK...",
];

const KILT: &[&str] = &[
    "................",
    "................",
    "...KKKKKKKKKK...",
    "...K333BB333K...",
    "...K01010102K...",
    "..K0101010102K..",
    "..K0101010102K..",
    ".K010101010102K.",
    ".K010101010122K.",
    ".KKKKKKKKKKKKKK.",
    "...KeeK..KeeK...",
    "...KKK....KKK...",
];

const SANDAL: &[&str] = &[
    "................",
    "................",
    "................",
    "....KKK.........",
    "....K0K.........",
    "....K1KKK.......",
    "....K010KK......",
    "....K1K3K1KK....",
    "....K0K1K01KK...",
    "....K11K1K011K..",
    "....K111110111K.",
    "....K2222222222K",
    "....KKKKKKKKKKK.",
];

const SCARAB: &[&str] = &[
    ".....K....K.....",
    "......K..K......",
    ".....KKKKKK.....",
    ".....K3333K.....",
    "...KKKKKKKKKK...",
    "..K0011K11112K..",
    ".K00111K111122K.",
    "K3K0111K11112K3K",
    ".K01111K111122K.",
    "K3K1111K11122K3K",
    ".K01111K111222K.",
    "..K111KKK1122K..",
    "...KK2K.K22KK...",
    ".....KK.KK......",
];

const KHOPESH: &[&str] = &[
    "................",
    "........KKKK....",
    ".......K0011K...",
    "......K01..12K..",
    "......K1K...K2K.",
    ".......KK...K2K.",
    "...........K12K.",
    "..........K112K.",
    ".........K112K..",
    "..KK....K112K...",
    "..K3K..K112K....",
    "...K3KK112K.....",
    "....K33KKK......",
    "...K3K3K........",
    "..K3K.KK........",
    "..KK............",
];

const COBRA_STAFF: &[&str] = &[
    ".......KKKK.....",
    "......K0011K....",
    ".....K011112K...",
    ".....K1B11B2K...",
    ".....K111112K...",
    "......K1r12K....",
    "......KK1KK.....",
    "........K1K.....",
    ".......K1K......",
    ".......KuK......",
    "......KuK.......",
    ".....KuK........",
    "....KuK.........",
    "...KuK..........",
    "..KuK...........",
    "..KK............",
];

const EYE_CIRCLET: &[&str] = &[
    "................",
    "................",
    "................",
    "................",
    "................",
    "......KKKK......",
    ".....KwwwwK.....",
    "..KKKKw33wKKKK..",
    ".K0001w3Kw1112K.",
    ".K0111KwwK1122K.",
    "..KKKKKKKKKKKK..",
];

const OBSIDIAN_PLATE: &[&str] = &[
    "................",
    "...KKKK..KKKK...",
    "..K0000KK0000K..",
    ".K011111111112K.",
    "K00K1113111K12K.",
    "K01K1133311K22K.",
    "KKKK1113111KKKK.",
    "...K1111111K....",
    "...K3131313K....",
    "...K1111112K....",
    "...K1111122K....",
    "...KKKKKKKKK....",
];

const VOIDWEAVE: &[&str] = &[
    "................",
    "................",
    "...KKKKKKKKKK...",
    "...K33333333K...",
    "...K01w00012K...",
    "...K00000w12K...",
    "...K0w01K012K...",
    "...K001KK0w2K...",
    "...K0w1KK012K...",
    "...K001KKw12K...",
    "...K012KK112K...",
    "...KKKKKKKKKK...",
];

const VOIDWALKERS: &[&str] = &[
    "................",
    "................",
    "....KKKKKK......",
    "....K0w01K......",
    "....K0012K......",
    "....KL012K......",
    "....K0012K......",
    "....K00112KKKK..",
    "....K001111L11K.",
    "....K0011111112K",
    "....K3333333333K",
    "....KKKKKKKKKKKK",
];

const EYE_KITE: &[&str] = &[
    "................",
    "...KKKKKKKKKK...",
    "...K33333333K...",
    "...K30011113K...",
    "...K3KKKKK13K...",
    "...K3KwllwK3K...",
    "...K3KlKKlK3K...",
    "...K3KwllwK3K...",
    "....K3KKKK3K....",
    "....K311113K....",
    ".....K3113K.....",
    "......K33K......",
    ".......KK.......",
];

const RIFTBLADE: &[&str] = &[
    "................",
    "............KK..",
    "...........K01K.",
    "..........K012K.",
    ".........K012K..",
    "........K012K...",
    ".......K012K....",
    "..KK..K012K.....",
    "..K3KK012K......",
    "...K3312K.......",
    "....K33K........",
    "...KkKK3K.......",
    "..KkK..K3K......",
    ".KkK....KK......",
    ".KK.............",
];

const EYE_WAND: &[&str] = &[
    "................",
    "..........KKK...",
    "...KK....K000K..",
    "..KVVK..K01110K.",
    "...KVVKK011K10K.",
    "....KKK011KK10K.",
    ".......K0111120K",
    ".......K001100K.",
    "......KKK0000K..",
    ".....K3K.KKKK...",
    "....K3K.........",
    "...K3K..........",
    "..K3K...........",
    ".K3K............",
    ".KK.............",
];

// ------------------------------------------------------------------------------------------

#[derive(Clone, Copy)]
enum Tip {
    Gem,
    Star,
    Bubble,
    Mushroom,
    Candy,
    Moon,
    /// A griffin's golden feather.
    Feather,
    /// A gazer's eye, with a pair of little bat's wings.
    Eye,
}

#[derive(Clone, Copy)]
enum Top {
    Orb,
    Sunflower,
    Mushroom,
    Bloom,
    /// A golden cobra rearing up with its hood spread.
    Cobra,
}

#[derive(Clone, Copy)]
enum CanStyle {
    Plain,
    Duck,
    Teapot,
    Frog,
    Cloud,
}

#[derive(Clone, Copy)]
enum Ears {
    Cat,
    Bunny,
    Frog,
}

#[derive(Clone, Copy)]
enum Pattern {
    Plain,
    Stripes,
    Chain,
    Plate,
    Stars,
    Buttons,
    Zigzag,
    /// A hero's muscles carved in marble, and a fringe of gilded strips below.
    Cuirass,
    /// A broad jewelled collar of gold and lapis.
    Collar,
    /// Runes glowing on black glass.
    Runes,
}

#[derive(Clone, Copy)]
enum LegPattern {
    Plain,
    Patched,
    Skirt,
    Plated,
    Striped,
    Stars,
    /// A pleated linen kilt to the knee on a golden belt, bare shins below.
    Kilt,
}

#[derive(Clone, Copy)]
enum L {
    Blade([u8; 3], u8, u8, f32, f32),
    Carrot,
    Leek,
    Baguette,
    Twig,
    Wand(Tip, [u8; 3], u8),
    Staff(Top, [u8; 3], u8),
    Pick([u8; 3]),
    Axe([u8; 3]),
    Hoe([u8; 3]),
    Sickle([u8; 3]),
    Can(CanStyle, [u8; 3]),
    Straw,
    Flowers,
    Hood(Ears, [u8; 3], u8),
    MushroomCap([u8; 3]),
    LeafCap,
    Helm([u8; 3], u8, bool),
    Miner,
    Wizard([u8; 3], u8),
    Circlet([u8; 3], u8),
    Crown([u8; 3], u8),
    Chest([u8; 3], u8, Pattern),
    Legs([u8; 2], u8, LegPattern),
    Boot([u8; 3], u8, f32),
    FrogBoot,
    BunnyBoot,
    WingBoot,
    Round([u8; 3], u8),
    Kite([u8; 3], u8),
    PotLid,
    Turtle,
    LeafShield,
    MushroomShield,
    /// Pole colours, grip and reel.
    Rod([u8; 3], u8, u8),
    /// A bronze helm with cheek guards and a crest from brow to nape, in the crest's colour.
    Crested([u8; 3], u8),
    /// A striped headcloth falling to the shoulders, a cobra on the brow: gold, then blue.
    Nemes([u8; 3], u8),
    /// A circlet set with a staring eye.
    EyeCirclet([u8; 3]),
    /// Sandals, strapped up the ankle, with little wings at the heel or without.
    Sandal([u8; 3], bool),
    /// A round shield with the maze beaten into it, and a rim.
    Aspis([u8; 3], u8),
    /// A beetle of a shield, its wings folded, with a golden rim, head and legs.
    Scarab([u8; 3], u8),
    /// A kite shield with an eye glaring out of it.
    EyeKite([u8; 3], u8),
    /// A double axe: blades, and the haft.
    Labrys([u8; 3], u8),
    /// A hooked sickle-sword: blade, and the grip.
    Khopesh([u8; 3], u8),
}

/// (icon name, icon template or `None` when `sprites.rs` already draws it, slots, look)
type Entry = (&'static str, Option<&'static [&'static str]>, [u8; 4], L);

const RUSTY: [u8; 3] = [KHAKI, ROSEWOOD, RUST];
const COPPER: [u8; 3] = [CREAM, GOLD, CLAY];
const IRON: [u8; 3] = [WHITE, SKY, INDIGO];
const GOLDEN: [u8; 3] = [WHITE, CREAM, GOLD];
const CRYSTAL: [u8; 3] = [WHITE, MINT, AQUA];
const EMBER: [u8; 3] = [CREAM, GOLD, RED];
const FROST: [u8; 3] = [WHITE, SKY, BLUE];
const BRONZE: [u8; 3] = [CREAM, GOLD, CLAY];
const MARBLE: [u8; 3] = [WHITE, SAND, KHAKI];
const LINEN: [u8; 3] = [WHITE, SAND, KHAKI];
const LAPIS: [u8; 3] = [SKY, BLUE, INDIGO];
const OBSIDIAN: [u8; 3] = [SLATE, SHADOW, INK];

fn s4(c: [u8; 3], d: u8) -> [u8; 4] {
    [c[0], c[1], c[2], d]
}

fn table() -> Vec<Entry> {
    use L::*;
    vec![
        // Swords.
        ("twig_sword", Some(TWIG_SWORD), NO, Twig),
        ("sword0", None, NO, Blade(RUSTY, GOLD, RUST, 0.34, 0.06)),
        ("carrot_blade", Some(CARROT_BLADE), NO, Carrot),
        ("baguette", Some(BAGUETTE), NO, Baguette),
        ("sword1", None, NO, Blade(COPPER, GOLD, RUST, 0.35, 0.06)),
        ("mighty_leek", Some(LEEK), NO, Leek),
        ("sword2", None, NO, Blade(IRON, GOLD, RUST, 0.37, 0.06)),
        (
            "capwood_sabre",
            Some(SABRE),
            [CREAM, SAND, KHAKI, RED],
            Blade([CREAM, SAND, KHAKI], RED, RUST, 0.38, 0.07),
        ),
        ("sword3", None, NO, Blade(GOLDEN, GOLD, RUST, 0.38, 0.065)),
        ("sword4", None, NO, Blade(CRYSTAL, GOLD, RUST, 0.41, 0.06)),
        (
            "frost_fang",
            Some(SWORD),
            s4(FROST, CLEAR),
            Blade(FROST, SKY, INDIGO, 0.42, 0.06),
        ),
        ("sword5", None, NO, Blade(EMBER, GOLD, RUST, 0.42, 0.07)),
        (
            "bone_sabre",
            Some(SABRE),
            [WHITE, SAND, KHAKI, SHADOW],
            Blade([WHITE, SAND, KHAKI], SHADOW, SHADOW, 0.42, 0.07),
        ),
        (
            "starlight_sword",
            Some(SWORD),
            [WHITE, CREAM, LAVENDER, CLEAR],
            Blade([WHITE, CREAM, LAVENDER], LAVENDER, INDIGO, 0.46, 0.07),
        ),
        // Wands.
        (
            "twig_wand",
            Some(WAND_GEM),
            [LIME, GREEN, TEAL, CLAY],
            Wand(Tip::Gem, [LIME, GREEN, TEAL], CLAY),
        ),
        (
            "bubble_wand",
            Some(WAND_BUBBLE),
            NO,
            Wand(Tip::Bubble, [WHITE, SKY, AQUA], RUST),
        ),
        (
            "glowcap_wand",
            Some(WAND_MUSHROOM),
            NO,
            Wand(Tip::Mushroom, [WHITE, MINT, AQUA], RUST),
        ),
        (
            "candy_wand",
            Some(WAND_CANDY),
            NO,
            Wand(Tip::Candy, [WHITE, BLUSH, PINK], WHITE),
        ),
        (
            "crystal_wand",
            Some(WAND_GEM),
            s4(CRYSTAL, SLATE),
            Wand(Tip::Gem, CRYSTAL, SLATE),
        ),
        (
            "ember_wand",
            Some(WAND_GEM),
            s4(EMBER, MAROON),
            Wand(Tip::Gem, EMBER, MAROON),
        ),
        (
            "frost_wand",
            Some(WAND_GEM),
            s4(FROST, INDIGO),
            Wand(Tip::Gem, FROST, INDIGO),
        ),
        (
            "star_wand",
            Some(WAND_STAR),
            [WHITE, CREAM, GOLD, LAVENDER],
            Wand(Tip::Star, [WHITE, CREAM, GOLD], LAVENDER),
        ),
        (
            "moonpetal_wand",
            Some(WAND_MOON),
            NO,
            Wand(Tip::Moon, [WHITE, BLUSH, LAVENDER], CREAM),
        ),
        // Staffs.
        (
            "oak_staff",
            Some(STAFF_ORB),
            [SAND, CLAY, RUST, RUST],
            Staff(Top::Orb, [SAND, CLAY, RUST], RUST),
        ),
        (
            "sunflower_staff",
            Some(STAFF_SUNFLOWER),
            NO,
            Staff(Top::Sunflower, [CREAM, GOLD, RUST], GREEN),
        ),
        (
            "mossy_staff",
            Some(STAFF_ORB),
            [LIME, GREEN, TEAL, CLAY],
            Staff(Top::Orb, [LIME, GREEN, TEAL], CLAY),
        ),
        (
            "mushroom_staff",
            Some(STAFF_MUSHROOM),
            NO,
            Staff(Top::Mushroom, [WHITE, RED, CRIMSON], RUST),
        ),
        (
            "crystal_staff",
            Some(STAFF_ORB),
            s4(CRYSTAL, SLATE),
            Staff(Top::Orb, CRYSTAL, SLATE),
        ),
        (
            "ember_staff",
            Some(STAFF_ORB),
            s4(EMBER, MAROON),
            Staff(Top::Orb, EMBER, MAROON),
        ),
        (
            "frost_staff",
            Some(STAFF_ORB),
            s4(FROST, INDIGO),
            Staff(Top::Orb, FROST, INDIGO),
        ),
        (
            "moonbloom_staff",
            Some(STAFF_BLOOM),
            NO,
            Staff(Top::Bloom, [BLUSH, LAVENDER, PURPLE], GREEN),
        ),
        // Shields.
        ("pot_lid", Some(POT_LID), NO, PotLid),
        (
            "wood_buckler",
            Some(SHIELD_ROUND),
            [SAND, CLAY, RUST, KHAKI],
            Round([SAND, CLAY, RUST], KHAKI),
        ),
        ("leaf_shield", Some(LEAF_SHIELD), NO, LeafShield),
        (
            "copper_shield",
            Some(SHIELD_ROUND),
            [GOLD, ORANGE, RUST, MAROON],
            Round([GOLD, ORANGE, RUST], MAROON),
        ),
        ("turtle_shell", Some(TURTLE), NO, Turtle),
        (
            "iron_shield",
            Some(SHIELD_KITE),
            s4(IRON, SLATE),
            Kite(IRON, SLATE),
        ),
        ("mushroom_shield", Some(MUSHROOM_SHIELD), NO, MushroomShield),
        (
            "gold_shield",
            Some(SHIELD_KITE),
            [CREAM, GOLD, CLAY, RUST],
            Kite([CREAM, GOLD, CLAY], RUST),
        ),
        (
            "crystal_aegis",
            Some(SHIELD_KITE),
            s4(CRYSTAL, TEAL),
            Kite(CRYSTAL, TEAL),
        ),
        (
            "frost_ward",
            Some(SHIELD_ROUND),
            s4(FROST, INDIGO),
            Round(FROST, INDIGO),
        ),
        (
            "ember_bulwark",
            Some(SHIELD_KITE),
            s4(EMBER, MAROON),
            Kite(EMBER, MAROON),
        ),
        // Headgear.
        ("straw_hat", Some(STRAW_HAT), NO, Straw),
        ("flower_crown", Some(FLOWER_CROWN), NO, Flowers),
        ("leaf_cap", Some(LEAF_CAP), NO, LeafCap),
        (
            "cat_hood",
            Some(HOOD_EARS),
            [PEACH, CLAY, RUST, PINK],
            Hood(Ears::Cat, [PEACH, CLAY, RUST], PINK),
        ),
        (
            "frog_hood",
            Some(FROG_HOOD),
            NO,
            Hood(Ears::Frog, [LIME, GREEN, TEAL], WHITE),
        ),
        (
            "copper_helm",
            Some(HELM),
            [GOLD, ORANGE, RUST, MAROON],
            Helm([GOLD, ORANGE, RUST], MAROON, false),
        ),
        (
            "mushroom_cap",
            Some(CAP_HAT),
            NO,
            MushroomCap([WHITE, RED, CRIMSON]),
        ),
        ("miner_hat", Some(MINER_HAT), NO, Miner),
        (
            "iron_helm",
            Some(HELM),
            s4(IRON, SLATE),
            Helm(IRON, SLATE, false),
        ),
        (
            "wizard_hat",
            Some(WIZARD_HAT),
            [LAVENDER, PURPLE, GRAPE, GOLD],
            Wizard([LAVENDER, PURPLE, GRAPE], GOLD),
        ),
        (
            "bunny_hood",
            Some(BUNNY_HOOD),
            NO,
            Hood(Ears::Bunny, [WHITE, SAND, KHAKI], BLUSH),
        ),
        (
            "crystal_circlet",
            Some(CIRCLET),
            [WHITE, SKY, BLUE, MINT],
            Circlet([WHITE, SKY, BLUE], MINT),
        ),
        (
            "frost_helm",
            Some(HELM_HORNS),
            s4(FROST, INDIGO),
            Helm(FROST, INDIGO, true),
        ),
        (
            "ember_crown",
            Some(CROWN_HAT),
            [GOLD, ORANGE, RED, CRIMSON],
            Crown([GOLD, ORANGE, RED], CRIMSON),
        ),
        // Chest.
        (
            "cozy_sweater",
            Some(SWEATER),
            [CREAM, SALMON, CRIMSON, PLUM],
            Chest([CREAM, SALMON, CRIMSON], SALMON, Pattern::Stripes),
        ),
        (
            "farmer_tunic",
            Some(TUNIC),
            [SKY, BLUE, INDIGO, CLAY],
            Chest([SKY, BLUE, INDIGO], CLAY, Pattern::Plain),
        ),
        (
            "leaf_tunic",
            Some(TUNIC),
            [LIME, GREEN, TEAL, RUST],
            Chest([LIME, GREEN, TEAL], RUST, Pattern::Plain),
        ),
        (
            "leather_vest",
            Some(TUNIC),
            [SAND, CLAY, RUST, MAROON],
            Chest([SAND, CLAY, RUST], MAROON, Pattern::Plain),
        ),
        (
            "copper_mail",
            Some(MAIL),
            [GOLD, ORANGE, RUST, MAROON],
            Chest([GOLD, ORANGE, RUST], MAROON, Pattern::Chain),
        ),
        (
            "frog_raincoat",
            Some(COAT),
            [LIME, GREEN, TEAL, GOLD],
            Chest([LIME, GREEN, TEAL], GOLD, Pattern::Buttons),
        ),
        (
            "mage_robe",
            Some(ROBE),
            [LAVENDER, PURPLE, GRAPE, CREAM],
            Chest([LAVENDER, PURPLE, GRAPE], GOLD, Pattern::Stars),
        ),
        (
            "iron_plate",
            Some(PLATE),
            [WHITE, SKY, INDIGO, GOLD],
            Chest(IRON, GOLD, Pattern::Plate),
        ),
        (
            "woolly_poncho",
            Some(PONCHO),
            [CREAM, SAND, KHAKI, RED],
            Chest([CREAM, SAND, KHAKI], RED, Pattern::Zigzag),
        ),
        (
            "crystal_mail",
            Some(MAIL),
            s4(CRYSTAL, TEAL),
            Chest(CRYSTAL, TEAL, Pattern::Chain),
        ),
        (
            "frost_coat",
            Some(COAT),
            [WHITE, SKY, BLUE, CREAM],
            Chest(FROST, WHITE, Pattern::Buttons),
        ),
        (
            "ember_plate",
            Some(PLATE),
            [CREAM, GOLD, RED, MAROON],
            Chest(EMBER, MAROON, Pattern::Plate),
        ),
        (
            "star_robe",
            Some(ROBE),
            [SKY, INDIGO, SLATE, GOLD],
            Chest([SKY, INDIGO, SLATE], GOLD, Pattern::Stars),
        ),
        // Legs.
        (
            "patched_trousers",
            Some(TROUSERS_PATCHED),
            [SKY, BLUE, INDIGO, RUST],
            Legs([BLUE, INDIGO], SALMON, LegPattern::Patched),
        ),
        (
            "grass_skirt",
            Some(GRASS_SKIRT),
            NO,
            Legs([GREEN, TEAL], LIME, LegPattern::Skirt),
        ),
        (
            "leather_leggings",
            Some(TROUSERS),
            [SAND, CLAY, RUST, MAROON],
            Legs([CLAY, RUST], MAROON, LegPattern::Plain),
        ),
        (
            "copper_greaves",
            Some(GREAVES),
            [GOLD, ORANGE, RUST, CREAM],
            Legs([ORANGE, RUST], GOLD, LegPattern::Plated),
        ),
        (
            "pumpkin_bloomers",
            Some(BLOOMERS),
            NO,
            Legs([ORANGE, CLAY], GOLD, LegPattern::Striped),
        ),
        (
            "iron_greaves",
            Some(GREAVES),
            [WHITE, SKY, INDIGO, WHITE],
            Legs([SKY, INDIGO], WHITE, LegPattern::Plated),
        ),
        (
            "starry_leggings",
            Some(TROUSERS),
            [SKY, INDIGO, SLATE, GOLD],
            Legs([INDIGO, SLATE], GOLD, LegPattern::Stars),
        ),
        (
            "crystal_greaves",
            Some(GREAVES),
            [WHITE, MINT, AQUA, WHITE],
            Legs([MINT, AQUA], WHITE, LegPattern::Plated),
        ),
        (
            "frost_leggings",
            Some(TROUSERS),
            [WHITE, SKY, BLUE, WHITE],
            Legs([SKY, BLUE], WHITE, LegPattern::Plain),
        ),
        (
            "ember_greaves",
            Some(GREAVES),
            [CREAM, GOLD, RED, GOLD],
            Legs([RED, MAROON], GOLD, LegPattern::Plated),
        ),
        // Boots.
        (
            "rain_boots",
            Some(BOOTS),
            [CREAM, GOLD, CLAY, RUST],
            Boot([CREAM, GOLD, CLAY], RUST, 0.15),
        ),
        ("frog_slippers", Some(FROG_SLIPPERS), NO, FrogBoot),
        (
            "leather_boots",
            Some(BOOTS),
            [SAND, CLAY, RUST, MAROON],
            Boot([SAND, CLAY, RUST], MAROON, 0.11),
        ),
        ("bunny_slippers", Some(BUNNY_SLIPPERS), NO, BunnyBoot),
        (
            "copper_sabatons",
            Some(BOOTS),
            [GOLD, ORANGE, RUST, MAROON],
            Boot([GOLD, ORANGE, RUST], MAROON, 0.12),
        ),
        (
            "iron_boots",
            Some(BOOTS),
            s4(IRON, SLATE),
            Boot(IRON, SLATE, 0.12),
        ),
        ("feather_boots", Some(WINGED_BOOTS), NO, WingBoot),
        (
            "crystal_boots",
            Some(BOOTS),
            s4(CRYSTAL, TEAL),
            Boot(CRYSTAL, TEAL, 0.12),
        ),
        (
            "frost_walkers",
            Some(BOOTS),
            s4(FROST, INDIGO),
            Boot(FROST, INDIGO, 0.13),
        ),
        (
            "ember_treads",
            Some(BOOTS),
            s4(EMBER, MAROON),
            Boot(EMBER, MAROON, 0.12),
        ),
        // Hoes.
        ("hoe", None, NO, Hoe(IRON)),
        (
            "sprout_hoe",
            Some(SPROUT_HOE),
            [WHITE, LIME, GREEN, CLEAR],
            Hoe([WHITE, LIME, GREEN]),
        ),
        ("copper_hoe", Some(HOE), s4(COPPER, CLEAR), Hoe(COPPER)),
        (
            "iron_hoe",
            Some(HOE),
            [WHITE, SKY, SLATE, CLEAR],
            Hoe([WHITE, SKY, SLATE]),
        ),
        ("gold_hoe", Some(HOE), s4(GOLDEN, CLEAR), Hoe(GOLDEN)),
        ("crystal_hoe", Some(HOE), s4(CRYSTAL, CLEAR), Hoe(CRYSTAL)),
        ("ember_hoe", Some(HOE), s4(EMBER, CLEAR), Hoe(EMBER)),
        // Watering cans.
        ("can0", None, NO, Can(CanStyle::Plain, IRON)),
        (
            "duck_can",
            Some(DUCK_CAN),
            NO,
            Can(CanStyle::Duck, [CREAM, GOLD, ORANGE]),
        ),
        ("can1", None, NO, Can(CanStyle::Plain, COPPER)),
        (
            "teapot_can",
            Some(TEAPOT_CAN),
            NO,
            Can(CanStyle::Teapot, [BLUSH, PINK, CRIMSON]),
        ),
        (
            "iron_can",
            Some(CAN),
            [WHITE, SKY, SLATE, CLEAR],
            Can(CanStyle::Plain, [WHITE, SKY, SLATE]),
        ),
        (
            "frog_can",
            Some(FROG_CAN),
            NO,
            Can(CanStyle::Frog, [LIME, GREEN, TEAL]),
        ),
        ("can2", None, NO, Can(CanStyle::Plain, CRYSTAL)),
        (
            "cloud_can",
            Some(CLOUD_CAN),
            NO,
            Can(CanStyle::Cloud, [WHITE, SKY, BLUE]),
        ),
        // Sickles.
        ("sickle", Some(SICKLE), s4(IRON, CLEAR), Sickle(IRON)),
        (
            "copper_sickle",
            Some(SICKLE),
            s4(COPPER, CLEAR),
            Sickle(COPPER),
        ),
        (
            "iron_sickle",
            Some(SICKLE),
            [WHITE, SKY, SLATE, CLEAR],
            Sickle([WHITE, SKY, SLATE]),
        ),
        (
            "gold_sickle",
            Some(SICKLE),
            s4(GOLDEN, CLEAR),
            Sickle(GOLDEN),
        ),
        (
            "moon_sickle",
            Some(SICKLE),
            [WHITE, BLUSH, LAVENDER, CLEAR],
            Sickle([WHITE, BLUSH, LAVENDER]),
        ),
        // Axes and pickaxes.
        ("axe0", None, NO, Axe(RUSTY)),
        ("axe1", None, NO, Axe(COPPER)),
        (
            "beaver_axe",
            Some(AXE),
            [SAND, CLAY, RUST, CLEAR],
            Axe([SAND, CLAY, RUST]),
        ),
        ("axe2", None, NO, Axe(IRON)),
        ("axe3", None, NO, Axe(GOLDEN)),
        ("axe4", None, NO, Axe(CRYSTAL)),
        ("axe5", None, NO, Axe(EMBER)),
        ("pick0", None, NO, Pick(RUSTY)),
        ("pick1", None, NO, Pick(COPPER)),
        ("pick2", None, NO, Pick(IRON)),
        (
            "mole_pick",
            Some(PICKAXE),
            [BLUSH, PINK, CRIMSON, CLEAR],
            Pick([BLUSH, PINK, CRIMSON]),
        ),
        ("pick3", None, NO, Pick(GOLDEN)),
        ("pick4", None, NO, Pick(CRYSTAL)),
        ("pick5", None, NO, Pick(EMBER)),
        // Fishing rods.
        (
            "bamboo_rod",
            Some(ROD),
            [CREAM, KHAKI, CLAY, SLATE],
            Rod([CREAM, SAND, KHAKI], CLAY, SLATE),
        ),
        (
            "willow_rod",
            Some(ROD),
            [PEACH, CLAY, GREEN, SKY],
            Rod([PEACH, CLAY, RUST], GREEN, SKY),
        ),
        (
            "oak_rod",
            Some(ROD),
            [GOLD, RUST, MAROON, GOLD],
            Rod([GOLD, RUST, MAROON], MAROON, GOLD),
        ),
        (
            "coral_rod",
            Some(ROD),
            [BLUSH, PINK, WHITE, SALMON],
            Rod([BLUSH, PINK, CRIMSON], WHITE, SALMON),
        ),
        (
            "kelp_rod",
            Some(ROD),
            [LIME, GREEN, TEAL, MINT],
            Rod([LIME, GREEN, DEEP_TEAL], TEAL, MINT),
        ),
        (
            "pearl_rod",
            Some(ROD),
            [WHITE, BLUSH, SKY, WHITE],
            Rod([WHITE, BLUSH, LAVENDER], SKY, WHITE),
        ),
        (
            "crystal_rod",
            Some(ROD),
            [WHITE, MINT, TEAL, AQUA],
            Rod([WHITE, MINT, AQUA], TEAL, AQUA),
        ),
        (
            "jelly_rod",
            Some(ROD),
            [LAVENDER, PINK, BLUSH, PURPLE],
            Rod([LAVENDER, PINK, PURPLE], BLUSH, PURPLE),
        ),
        (
            "ember_rod",
            Some(ROD),
            [GOLD, ORANGE, MAROON, RED],
            Rod([GOLD, ORANGE, RED], MAROON, GOLD),
        ),
        (
            "frost_rod",
            Some(ROD),
            [WHITE, SKY, INDIGO, BLUE],
            Rod([WHITE, SKY, BLUE], INDIGO, WHITE),
        ),
        (
            "star_rod",
            Some(ROD),
            [CREAM, GOLD, INDIGO, LAVENDER],
            Rod([CREAM, GOLD, LAVENDER], INDIGO, GOLD),
        ),
        (
            "leviathan_rod",
            Some(ROD),
            [AQUA, TEAL, INDIGO, GOLD],
            Rod([AQUA, TEAL, DEEP_TEAL], INDIGO, GOLD),
        ),
        // The Marble Labyrinth.
        (
            "plumed_helm",
            Some(PLUMED_HELM),
            [CREAM, GOLD, CLAY, RED],
            Crested(BRONZE, RED),
        ),
        (
            "marble_cuirass",
            Some(CUIRASS),
            [WHITE, SAND, KHAKI, GOLD],
            Chest(MARBLE, GOLD, Pattern::Cuirass),
        ),
        (
            "hoplite_greaves",
            Some(GREAVES_BRONZE),
            [CREAM, GOLD, CLAY, MAROON],
            Legs([GOLD, CLAY], CREAM, LegPattern::Plated),
        ),
        (
            "winged_sandals",
            Some(WINGED_SANDAL),
            [CREAM, GOLD, CLAY, WHITE],
            Sandal(BRONZE, true),
        ),
        (
            "labyrinth_aspis",
            Some(ASPIS),
            [CREAM, GOLD, CLAY, MAROON],
            Aspis(BRONZE, MAROON),
        ),
        (
            "labrys",
            Some(LABRYS),
            [CREAM, GOLD, CLAY, RUST],
            Labrys(BRONZE, RUST),
        ),
        (
            "griffin_quill",
            Some(QUILL),
            [CREAM, GOLD, CLAY, WHITE],
            Wand(Tip::Feather, BRONZE, RUST),
        ),
        // The Sunscorch Canyon.
        (
            "pharaoh_nemes",
            Some(NEMES),
            [GOLD, BLUE, INDIGO, CREAM],
            Nemes([CREAM, GOLD, CLAY], BLUE),
        ),
        (
            "sunscorch_wraps",
            Some(WRAPS),
            [WHITE, SAND, KHAKI, GOLD],
            Chest(LINEN, GOLD, Pattern::Collar),
        ),
        (
            "scarab_kilt",
            Some(KILT),
            [WHITE, SAND, KHAKI, GOLD],
            Legs([WHITE, SAND], GOLD, LegPattern::Kilt),
        ),
        (
            "dune_sandals",
            Some(SANDAL),
            [SAND, CLAY, RUST, GOLD],
            Sandal([SAND, CLAY, RUST], false),
        ),
        (
            "scarab_shield",
            Some(SCARAB),
            [SKY, BLUE, INDIGO, GOLD],
            Scarab(LAPIS, GOLD),
        ),
        (
            "khopesh",
            Some(KHOPESH),
            [CREAM, GOLD, CLAY, BLUE],
            Khopesh(BRONZE, BLUE),
        ),
        (
            "cobra_staff",
            Some(COBRA_STAFF),
            [CREAM, GOLD, CLAY, BLUE],
            Staff(Top::Cobra, BRONZE, RUST),
        ),
        // The Starless Rift.
        (
            "gazer_circlet",
            Some(EYE_CIRCLET),
            [LAVENDER, PURPLE, GRAPE, LIME],
            EyeCirclet(OBSIDIAN),
        ),
        (
            "obsidian_plate",
            Some(OBSIDIAN_PLATE),
            [SLATE, SHADOW, INK, LAVENDER],
            Chest(OBSIDIAN, LAVENDER, Pattern::Runes),
        ),
        (
            "voidweave_leggings",
            Some(VOIDWEAVE),
            [PURPLE, GRAPE, INK, LAVENDER],
            Legs([GRAPE, INK], LAVENDER, LegPattern::Stars),
        ),
        (
            "voidwalkers",
            Some(VOIDWALKERS),
            [LAVENDER, PURPLE, GRAPE, INK],
            Boot([LAVENDER, PURPLE, GRAPE], INK, 0.14),
        ),
        (
            "obsidian_bulwark",
            Some(EYE_KITE),
            [SLATE, SHADOW, INK, LAVENDER],
            EyeKite(OBSIDIAN, LAVENDER),
        ),
        (
            "riftblade",
            Some(RIFTBLADE),
            [LAVENDER, PURPLE, INK, GRAPE],
            Blade([LAVENDER, PURPLE, GRAPE], GRAPE, INK, 0.44, 0.07),
        ),
        (
            "gazer_wand",
            Some(EYE_WAND),
            [WHITE, LIME, GREEN, GRAPE],
            Wand(Tip::Eye, [WHITE, LIME, GREEN], GRAPE),
        ),
    ]
}

/// Every piece of gear's look, keyed by icon name.
#[derive(Default)]
pub struct GearArt {
    pub held: HashMap<&'static str, Mesh>,
    pub hats: HashMap<&'static str, Mesh>,
    pub boots: HashMap<&'static str, Mesh>,
    pub shields: HashMap<&'static str, Mesh>,
    /// Body and arm textures for chest armour.
    pub chest: HashMap<&'static str, (TexId, TexId)>,
    pub legs: HashMap<&'static str, TexId>,
}

/// A little shaded metal texture: `[highlight, base, shadow]`.
fn metal(bank: &mut TexBank, c: [u8; 3]) -> TexId {
    let mut t = Texture::new(4, 4, c[1]);
    t.set(0, 0, c[0]);
    t.set(1, 1, c[0]);
    t.set(0, 2, c[0]);
    t.set(3, 3, c[2]);
    t.set(2, 3, c[2]);
    t.set(3, 1, c[2]);
    bank.add(t)
}

fn solid(bank: &mut TexBank, c: u8) -> TexId {
    bank.add(Texture::new(4, 4, c))
}

/// Dots of `dot` on `base` (mushroom caps).
fn spotty(bank: &mut TexBank, base: u8, dot: u8) -> TexId {
    let mut t = Texture::new(8, 8, base);
    for (x, y) in [
        (1, 1),
        (2, 1),
        (5, 3),
        (6, 3),
        (5, 4),
        (2, 5),
        (3, 6),
        (7, 7),
        (0, 6),
    ] {
        t.set(x, y, dot);
    }
    bank.add(t)
}

/// A box with the whole of a small texture on every face.
fn bx(m: &mut Mesh, min: Vec3, max: Vec3, tex: TexId) {
    skin_box(m, min, max, tex, &Texture::new(4, 4, 0));
}

fn v(x: f32, y: f32, z: f32) -> Vec3 {
    Vec3::new(x, y, z)
}

// ------------------------------------------------------------------------------------------
// Things held in the hand: grip at the origin, pointing down the arm (-y)
// ------------------------------------------------------------------------------------------

fn handle(m: &mut Mesh, tex: TexId, len: f32) {
    bx(m, v(-0.02, -len, -0.02), v(0.02, 0.04, 0.02), tex);
}

fn held_mesh(bank: &mut TexBank, look: L) -> Option<Mesh> {
    let mut m = Mesh::new();
    match look {
        L::Blade(c, guard, grip, len, w) => {
            let blade = metal(bank, c);
            let (g, h) = (solid(bank, guard), solid(bank, grip));
            bx(&mut m, v(-0.02, -0.08, -0.02), v(0.02, 0.04, 0.02), h);
            bx(&mut m, v(-0.02, 0.04, -0.025), v(0.02, 0.07, 0.025), g);
            bx(&mut m, v(-0.085, -0.105, -0.028), v(0.085, -0.07, 0.028), g);
            let tip = -0.1 - len;
            bx(
                &mut m,
                v(-w * 0.5, tip + 0.05, -0.013),
                v(w * 0.5, -0.1, 0.013),
                blade,
            );
            bx(
                &mut m,
                v(-w * 0.25, tip, -0.01),
                v(w * 0.25, tip + 0.05, 0.01),
                blade,
            );
        }
        L::Carrot => {
            let (o, g) = (metal(bank, [GOLD, ORANGE, CLAY]), solid(bank, GREEN));
            handle(&mut m, solid(bank, CLAY), 0.06);
            for (i, w) in [0.1f32, 0.085, 0.07, 0.05, 0.03].iter().enumerate() {
                let y = -0.08 - i as f32 * 0.07;
                bx(
                    &mut m,
                    v(-w * 0.5, y - 0.07, -w * 0.4),
                    v(w * 0.5, y, w * 0.4),
                    o,
                );
            }
            for a in [-0.5f32, 0.0, 0.5] {
                let mut leaf = Mesh::new();
                bx(&mut leaf, v(-0.012, 0.0, -0.012), v(0.012, 0.12, 0.012), g);
                m.append(
                    &leaf,
                    Mat4::from_translation(v(0.0, 0.02, 0.0)) * Mat4::from_rotation_z(a),
                );
            }
        }
        L::Leek => {
            let (w, g) = (
                metal(bank, [WHITE, WHITE, SAND]),
                metal(bank, [LIME, GREEN, TEAL]),
            );
            bx(&mut m, v(-0.035, -0.22, -0.035), v(0.035, 0.05, 0.035), w);
            bx(&mut m, v(-0.03, -0.44, -0.03), v(0.03, -0.22, 0.03), g);
            let mut leaf = Mesh::new();
            bx(&mut leaf, v(-0.02, -0.18, -0.008), v(0.02, 0.0, 0.008), g);
            m.append(
                &leaf,
                Mat4::from_translation(v(0.0, -0.4, 0.0)) * Mat4::from_rotation_z(0.4),
            );
            m.append(
                &leaf,
                Mat4::from_translation(v(0.0, -0.4, 0.0)) * Mat4::from_rotation_z(-0.4),
            );
            bx(
                &mut m,
                v(-0.03, 0.05, -0.03),
                v(0.03, 0.08, 0.03),
                solid(bank, SAND),
            );
        }
        L::Baguette => {
            let mut t = Texture::new(4, 4, GOLD);
            t.set(1, 0, SAND);
            t.set(2, 2, SAND);
            t.set(0, 3, CLAY);
            let crust = bank.add(t);
            bx(&mut m, v(-0.04, -0.52, -0.035), v(0.04, 0.06, 0.035), crust);
        }
        L::Twig => {
            let (b, g) = (solid(bank, CLAY), solid(bank, LIME));
            bx(&mut m, v(-0.018, -0.42, -0.018), v(0.018, 0.04, 0.018), b);
            let mut leaf = Mesh::new();
            bx(&mut leaf, v(0.0, -0.01, -0.03), v(0.08, 0.01, 0.03), g);
            m.append(
                &leaf,
                Mat4::from_translation(v(0.015, -0.3, 0.0)) * Mat4::from_rotation_z(-0.5),
            );
            bx(&mut m, v(0.018, -0.2, -0.01), v(0.06, -0.18, 0.01), b);
        }
        L::Wand(tip, c, stick) => {
            let s = solid(bank, stick);
            bx(&mut m, v(-0.015, -0.3, -0.015), v(0.015, 0.04, 0.015), s);
            let t = metal(bank, c);
            let at = Mat4::from_translation(v(0.0, -0.34, 0.0));
            let mut h = Mesh::new();
            match tip {
                Tip::Gem => {
                    bx(&mut h, v(-0.04, -0.04, -0.04), v(0.04, 0.04, 0.04), t);
                    m.append(
                        &h,
                        at * Mat4::from_rotation_z(PI / 4.0) * Mat4::from_rotation_y(PI / 4.0),
                    );
                }
                Tip::Star => {
                    bx(&mut h, v(-0.07, -0.025, -0.012), v(0.07, 0.025, 0.012), t);
                    for a in [0.0, 1.26, 2.51] {
                        m.append(&h, at * Mat4::from_rotation_z(a));
                    }
                }
                Tip::Bubble => {
                    bx(&mut h, v(-0.015, -0.015, -0.015), v(0.015, 0.015, 0.015), t);
                    for k in 0..8 {
                        let a = k as f32 / 8.0 * PI * 2.0;
                        m.append(
                            &h,
                            at * Mat4::from_translation(v(
                                a.cos() * 0.06,
                                a.sin() * 0.06 - 0.03,
                                0.0,
                            )),
                        );
                    }
                }
                Tip::Mushroom => {
                    lathe(
                        &mut h,
                        Vec3::ZERO,
                        &[(0.07, 0.0), (0.05, 0.04), (0.0, 0.055)],
                        6,
                        0.0,
                        t,
                        true,
                    );
                    m.append(&h, at * Mat4::from_rotation_x(PI));
                }
                Tip::Candy => {
                    let stripe = solid(bank, c[0]);
                    bx(&mut h, v(-0.045, -0.045, -0.03), v(0.045, 0.045, 0.03), t);
                    bx(
                        &mut h,
                        v(-0.047, -0.01, -0.032),
                        v(0.047, 0.01, 0.032),
                        stripe,
                    );
                    m.append(&h, at * Mat4::from_translation(v(0.0, -0.02, 0.0)));
                }
                Tip::Moon => {
                    bx(&mut h, v(-0.02, -0.02, -0.015), v(0.02, 0.02, 0.015), t);
                    for k in 0..5 {
                        let a = PI * 0.25 + k as f32 / 4.0 * PI;
                        m.append(
                            &h,
                            at * Mat4::from_translation(v(a.cos() * 0.06, -a.sin() * 0.06, 0.0)),
                        );
                    }
                }
                Tip::Feather => {
                    // A quill, and the vane either side of it, curving a little.
                    let quill = solid(bank, c[0]);
                    bx(
                        &mut h,
                        v(-0.008, -0.22, -0.008),
                        v(0.008, 0.02, 0.008),
                        quill,
                    );
                    bx(&mut h, v(-0.05, -0.2, -0.01), v(-0.008, -0.02, 0.01), t);
                    bx(&mut h, v(0.008, -0.18, -0.01), v(0.04, -0.03, 0.01), t);
                    bx(&mut h, v(-0.03, -0.25, -0.008), v(0.012, -0.19, 0.008), t);
                    m.append(&h, at * Mat4::from_rotation_z(0.15));
                }
                Tip::Eye => {
                    // An eyeball glaring out along the wand, on a pair of bat's wings.
                    let (white, iris, pupil, wing) = (
                        solid(bank, c[0]),
                        solid(bank, c[1]),
                        solid(bank, INK),
                        solid(bank, stick),
                    );
                    lathe(
                        &mut h,
                        Vec3::ZERO,
                        &[
                            (0.0, -0.06),
                            (0.045, -0.042),
                            (0.06, 0.0),
                            (0.045, 0.042),
                            (0.0, 0.06),
                        ],
                        6,
                        0.0,
                        white,
                        false,
                    );
                    bx(
                        &mut h,
                        v(-0.028, -0.028, 0.05),
                        v(0.028, 0.028, 0.062),
                        iris,
                    );
                    bx(
                        &mut h,
                        v(-0.008, -0.022, 0.062),
                        v(0.008, 0.022, 0.066),
                        pupil,
                    );
                    for sx in [-1.0f32, 1.0] {
                        let span = |a: f32, b: f32| (sx * a).min(sx * b)..(sx * a).max(sx * b);
                        let (x0, x1) = (span(0.05, 0.14), span(0.08, 0.13));
                        bx(
                            &mut h,
                            v(x0.start, 0.0, -0.004),
                            v(x0.end, 0.025, 0.004),
                            wing,
                        );
                        bx(
                            &mut h,
                            v(x1.start, -0.03, -0.003),
                            v(x1.end, 0.0, 0.003),
                            wing,
                        );
                    }
                    m.append(&h, at * Mat4::from_translation(v(0.0, -0.03, 0.0)));
                }
            }
        }
        L::Staff(top, c, stick) => {
            let s = solid(bank, stick);
            bx(&mut m, v(-0.022, -0.6, -0.022), v(0.022, 0.24, 0.022), s);
            let t = metal(bank, c);
            let at = Mat4::from_translation(v(0.0, -0.66, 0.0));
            let mut h = Mesh::new();
            match top {
                Top::Orb => {
                    lathe(
                        &mut h,
                        Vec3::ZERO,
                        &[
                            (0.0, -0.08),
                            (0.06, -0.06),
                            (0.08, 0.0),
                            (0.06, 0.06),
                            (0.0, 0.08),
                        ],
                        6,
                        0.3,
                        t,
                        false,
                    );
                    m.append(&h, at);
                    let ring = solid(bank, stick);
                    bx(
                        &mut m,
                        v(-0.035, -0.6, -0.035),
                        v(0.035, -0.57, 0.035),
                        ring,
                    );
                }
                Top::Sunflower => {
                    let (petal, heart) = (solid(bank, GOLD), solid(bank, RUST));
                    bx(&mut h, v(-0.03, -0.03, -0.012), v(0.03, 0.03, 0.02), heart);
                    let mut p = Mesh::new();
                    bx(&mut p, v(0.03, -0.02, -0.01), v(0.1, 0.02, 0.01), petal);
                    for k in 0..8 {
                        h.append(&p, Mat4::from_rotation_z(k as f32 * PI / 4.0));
                    }
                    m.append(&h, at * Mat4::from_rotation_x(FRAC_PI_2));
                }
                Top::Mushroom => {
                    let cap = spotty(bank, c[1], c[0]);
                    lathe(
                        &mut h,
                        Vec3::ZERO,
                        &[(0.12, 0.0), (0.1, 0.05), (0.0, 0.08)],
                        7,
                        0.0,
                        cap,
                        true,
                    );
                    m.append(
                        &h,
                        at * Mat4::from_translation(v(0.0, 0.02, 0.0)) * Mat4::from_rotation_x(PI),
                    );
                }
                Top::Bloom => {
                    let (petal, heart) = (metal(bank, c), solid(bank, CREAM));
                    bx(
                        &mut h,
                        v(-0.025, -0.025, -0.025),
                        v(0.025, 0.025, 0.025),
                        heart,
                    );
                    let mut p = Mesh::new();
                    bx(&mut p, v(0.0, -0.015, -0.03), v(0.09, 0.015, 0.03), petal);
                    for k in 0..5 {
                        h.append(
                            &p,
                            Mat4::from_rotation_y(k as f32 * PI * 0.4)
                                * Mat4::from_rotation_z(-0.5),
                        );
                    }
                    m.append(&h, at * Mat4::from_rotation_x(PI));
                }
                Top::Cobra => {
                    // The neck rising from the staff's end, the hood spread wide, the head
                    // at the top with its lapis eyes.
                    let eye = solid(bank, BLUE);
                    bx(&mut h, v(-0.025, -0.1, -0.025), v(0.025, 0.02, 0.025), t);
                    bx(&mut h, v(-0.08, -0.15, -0.012), v(0.08, -0.04, 0.012), t);
                    bx(&mut h, v(-0.055, -0.18, -0.012), v(0.055, -0.15, 0.012), t);
                    bx(&mut h, v(-0.035, -0.23, -0.03), v(0.035, -0.17, 0.05), t);
                    for x in [-0.02f32, 0.02] {
                        bx(
                            &mut h,
                            v(x - 0.008, -0.215, 0.05),
                            v(x + 0.008, -0.195, 0.054),
                            eye,
                        );
                    }
                    m.append(&h, at);
                }
            }
        }
        // The chopping tools' heads lie in the plane they swing through, with the business
        // end towards -z, the way the swing carries it, so a pick lands on its point and an
        // axe on its edge.
        L::Pick(c) => {
            let (h, t) = (solid(bank, RUST), metal(bank, c));
            handle(&mut m, h, 0.4);
            bx(&mut m, v(-0.03, -0.44, -0.2), v(0.03, -0.38, 0.2), t);
            bx(&mut m, v(-0.02, -0.42, -0.24), v(0.02, -0.34, -0.18), t);
            bx(&mut m, v(-0.02, -0.42, 0.18), v(0.02, -0.34, 0.24), t);
        }
        L::Axe(c) => {
            let (h, t) = (solid(bank, RUST), metal(bank, c));
            handle(&mut m, h, 0.4);
            bx(&mut m, v(-0.025, -0.46, -0.16), v(0.025, -0.3, 0.0), t);
            bx(&mut m, v(-0.02, -0.43, 0.0), v(0.02, -0.33, 0.05), t);
        }
        L::Hoe(c) => {
            let (h, t) = (solid(bank, RUST), metal(bank, c));
            handle(&mut m, h, 0.42);
            bx(&mut m, v(-0.02, -0.46, -0.14), v(0.02, -0.4, 0.02), t);
            bx(&mut m, v(-0.03, -0.5, -0.15), v(0.03, -0.4, -0.1), t);
        }
        L::Sickle(c) => {
            let (h, t) = (solid(bank, RUST), metal(bank, c));
            handle(&mut m, h, 0.16);
            let mut seg = Mesh::new();
            bx(&mut seg, v(-0.03, -0.012, -0.012), v(0.03, 0.012, 0.012), t);
            for k in 0..7 {
                let a = -0.3 + k as f32 * 0.45;
                let p = v(0.1 - a.cos() * 0.12, -0.2 - a.sin() * 0.12, 0.0);
                m.append(
                    &seg,
                    Mat4::from_translation(p) * Mat4::from_rotation_z(a + FRAC_PI_2),
                );
            }
        }
        L::Can(style, c) => {
            let body = metal(bank, c);
            let h = solid(bank, RUST);
            match style {
                CanStyle::Plain => {
                    bx(&mut m, v(-0.08, -0.2, -0.08), v(0.08, -0.04, 0.08), body);
                    bx(&mut m, v(-0.015, -0.04, -0.015), v(0.015, 0.03, 0.015), h);
                    bx(&mut m, v(0.06, -0.16, -0.015), v(0.2, -0.13, 0.015), body);
                }
                CanStyle::Duck => {
                    let beak = solid(bank, ORANGE);
                    let eye = solid(bank, INK);
                    bx(&mut m, v(-0.09, -0.2, -0.08), v(0.08, -0.06, 0.08), body);
                    bx(&mut m, v(0.02, -0.1, -0.05), v(0.12, 0.0, 0.05), body);
                    bx(&mut m, v(0.12, -0.06, -0.025), v(0.2, -0.03, 0.025), beak);
                    bx(&mut m, v(0.09, -0.03, 0.05), v(0.11, -0.01, 0.052), eye);
                    bx(&mut m, v(0.09, -0.03, -0.052), v(0.11, -0.01, -0.05), eye);
                    bx(&mut m, v(-0.015, -0.04, -0.015), v(0.015, 0.03, 0.015), h);
                }
                CanStyle::Teapot => {
                    lathe(
                        &mut m,
                        v(0.0, -0.2, 0.0),
                        &[
                            (0.06, 0.0),
                            (0.1, 0.05),
                            (0.09, 0.12),
                            (0.04, 0.15),
                            (0.0, 0.15),
                        ],
                        7,
                        0.0,
                        body,
                        true,
                    );
                    bx(&mut m, v(0.08, -0.12, -0.015), v(0.19, -0.1, 0.015), body);
                    bx(&mut m, v(-0.015, -0.05, -0.015), v(0.015, 0.03, 0.015), h);
                }
                CanStyle::Frog => {
                    let (w, k) = (solid(bank, WHITE), solid(bank, INK));
                    bx(&mut m, v(-0.09, -0.2, -0.08), v(0.09, -0.06, 0.08), body);
                    for z in [-0.045, 0.045] {
                        bx(&mut m, v(0.0, -0.06, z - 0.03), v(0.06, 0.0, z + 0.03), w);
                        bx(
                            &mut m,
                            v(0.06, -0.04, z - 0.012),
                            v(0.065, -0.02, z + 0.012),
                            k,
                        );
                    }
                    bx(&mut m, v(0.08, -0.16, -0.015), v(0.2, -0.13, 0.015), body);
                    bx(&mut m, v(-0.015, -0.06, -0.015), v(0.015, 0.03, 0.015), h);
                }
                CanStyle::Cloud => {
                    let drop = solid(bank, BLUE);
                    for (x, y, r) in [
                        (-0.05, -0.12, 0.06),
                        (0.04, -0.11, 0.07),
                        (0.0, -0.06, 0.06),
                        (0.1, -0.14, 0.05),
                    ] {
                        bx(
                            &mut m,
                            v(x - r, y - r * 0.8, -r),
                            v(x + r, y + r * 0.8, r),
                            body,
                        );
                    }
                    for (x, z) in [(-0.05, 0.02), (0.03, -0.03), (0.09, 0.03)] {
                        bx(
                            &mut m,
                            v(x - 0.012, -0.28, z - 0.012),
                            v(x + 0.012, -0.24, z + 0.012),
                            drop,
                        );
                    }
                    bx(&mut m, v(-0.015, -0.03, -0.015), v(0.015, 0.03, 0.015), h);
                }
            }
        }
        L::Labrys(c, haft) => {
            let (b, h) = (metal(bank, c), solid(bank, haft));
            // A long haft, and a crescent blade to either side of its head, widening out to
            // the edge.
            bx(&mut m, v(-0.02, -0.58, -0.02), v(0.02, 0.06, 0.02), h);
            bx(&mut m, v(-0.03, -0.56, -0.03), v(0.03, -0.42, 0.03), b);
            for sx in [-1.0f32, 1.0] {
                for (x0, x1, hh) in [
                    (0.03, 0.08, 0.045),
                    (0.08, 0.13, 0.07),
                    (0.13, 0.165, 0.095),
                ] {
                    let (a, z) = ((sx * x0).min(sx * x1), (sx * x0).max(sx * x1));
                    bx(&mut m, v(a, -0.49 - hh, -0.012), v(z, -0.49 + hh, 0.012), b);
                }
            }
        }
        L::Khopesh(c, grip) => {
            let (b, g) = (metal(bank, c), solid(bank, grip));
            bx(&mut m, v(-0.02, -0.08, -0.02), v(0.02, 0.05, 0.02), g);
            bx(&mut m, v(-0.05, -0.105, -0.025), v(0.05, -0.08, 0.025), b);
            // A straight neck, then the blade hooking out and round and back.
            bx(&mut m, v(-0.018, -0.3, -0.012), v(0.018, -0.1, 0.012), b);
            let mut seg = Mesh::new();
            bx(
                &mut seg,
                v(-0.032, -0.024, -0.012),
                v(0.032, 0.024, 0.012),
                b,
            );
            for k in 0..9 {
                let a = FRAC_PI_2 - k as f32 / 8.0 * 2.9;
                let p = v(a.cos() * 0.12, -0.41 + a.sin() * 0.12, 0.0);
                let along = a - FRAC_PI_2;
                m.append(
                    &seg,
                    Mat4::from_translation(p) * Mat4::from_rotation_z(along),
                );
            }
        }
        L::Rod(c, grip, reel) => {
            let pole = metal(bank, c);
            let (g, r) = (solid(bank, grip), solid(bank, reel));
            let len = crate::game::fish::ROD_LEN;
            // A grip, then the pole tapering away down the arm.
            bx(&mut m, v(-0.028, -0.22, -0.028), v(0.028, 0.07, 0.028), g);
            let segs = 5;
            for k in 0..segs {
                let t0 = k as f32 / segs as f32;
                let t1 = (k + 1) as f32 / segs as f32;
                let y0 = -0.22 - t0 * (len - 0.22);
                let y1 = -0.22 - t1 * (len - 0.22);
                let w = 0.02 * (1.0 - t0 * 0.65);
                bx(&mut m, v(-w, y1, -w), v(w, y0, w), pole);
                // A little guide ring for the line.
                bx(
                    &mut m,
                    v(-w * 0.6, y1 + 0.01, w),
                    v(w * 0.6, y1 + 0.03, w + 0.025),
                    r,
                );
            }
            // The reel under the grip, with its little crank.
            bx(&mut m, v(-0.045, -0.13, 0.03), v(0.045, -0.02, 0.1), r);
            bx(&mut m, v(0.045, -0.09, 0.05), v(0.08, -0.07, 0.07), g);
        }
        _ => return None,
    }
    Some(m)
}

// ------------------------------------------------------------------------------------------
// Hats: origin at the top of the head, +z is the face
// ------------------------------------------------------------------------------------------

/// A crown that hugs the head: like `lathe`, but every ring is a squircle just big enough
/// (at size 1) to hide the corners of the head's box, so the head never pokes through a
/// hat. `prof` is (size, height) pairs from the bottom up; a size of 0 closes it to a point.
fn hug(m: &mut Mesh, base: Vec3, prof: &[(f32, f32)], tex: TexId, cap_bottom: bool) {
    // The head's box is 0.52 x 0.46 across; this squircle clears its corners.
    const A: f32 = 0.31;
    const B: f32 = 0.28;
    const SEG: usize = 16;
    let pt = |s: f32, y: f32, j: usize| {
        let a = j as f32 / SEG as f32 * PI * 2.0;
        let (c, sn) = (a.cos(), a.sin());
        base + v(
            s * A * c.signum() * c.abs().sqrt(),
            y,
            -s * B * sn.signum() * sn.abs().sqrt(),
        )
    };
    let top = prof.last().map_or(0.0, |p| p.1);
    let tv = |y: f32| (top - y) * 16.0;
    for i in 0..prof.len().saturating_sub(1) {
        let ((s0, y0), (s1, y1)) = (prof[i], prof[i + 1]);
        for j in 0..SEG {
            let u0 = j as f32 * 2.0;
            let u1 = u0 + 2.0;
            let p = [
                pt(s0, y0, j),
                pt(s0, y0, j + 1),
                pt(s1, y1, j + 1),
                pt(s1, y1, j),
            ];
            if s1 <= 1e-4 {
                m.tri(
                    [p[0], p[1], p[2]],
                    [
                        glam::Vec2::new(u0, tv(y0)),
                        glam::Vec2::new(u1, tv(y0)),
                        glam::Vec2::new((u0 + u1) * 0.5, tv(y1)),
                    ],
                    tex,
                );
            } else {
                m.quad(p, UvRect::new(u0, tv(y1), u1, tv(y0)), tex);
            }
        }
    }
    if cap_bottom {
        let (s0, y0) = prof[0];
        let c = base + Vec3::Y * y0;
        for j in 0..SEG {
            let (p0, p1) = (pt(s0, y0, j), pt(s0, y0, j + 1));
            let uv = glam::Vec2::new(2.0, 2.0);
            m.tri([c, p1, p0], [uv, uv, uv], tex);
        }
    }
}

/// A point on the squircle `hug` builds, `a` radians round from the front, at size `s`.
fn hug_point(a: f32, s: f32, y: f32) -> Vec3 {
    let (sn, c) = (a.sin(), a.cos());
    v(
        s * 0.31 * sn.signum() * sn.abs().sqrt(),
        y,
        s * 0.28 * c.signum() * c.abs().sqrt(),
    )
}

fn hood(bank: &mut TexBank, m: &mut Mesh, c: [u8; 3]) {
    let t = metal(bank, c);
    bx(m, v(-0.29, -0.02, -0.26), v(0.29, 0.05, 0.25), t);
    bx(m, v(-0.29, -0.4, -0.27), v(0.29, 0.0, -0.22), t);
    for x in [-0.3f32, 0.26] {
        bx(m, v(x, -0.36, -0.25), v(x + 0.04, 0.0, 0.2), t);
    }
    // A soft brim framing the face.
    bx(m, v(-0.29, -0.06, 0.2), v(0.29, 0.0, 0.26), t);
}

fn hat_mesh(bank: &mut TexBank, look: L) -> Option<Mesh> {
    let mut m = Mesh::new();
    match look {
        L::Straw => {
            let straw = metal(bank, [CREAM, SAND, KHAKI]);
            let ribbon = solid(bank, RED);
            lathe(
                &mut m,
                v(0.0, -0.12, 0.0),
                &[(0.39, 0.0), (0.39, 0.025)],
                12,
                0.0,
                straw,
                true,
            );
            hug(
                &mut m,
                v(0.0, -0.1, 0.0),
                &[(1.0, 0.0), (0.98, 0.12), (0.62, 0.17), (0.0, 0.19)],
                straw,
                false,
            );
            hug(
                &mut m,
                v(0.0, -0.09, 0.0),
                &[(1.03, 0.0), (1.02, 0.05)],
                ribbon,
                false,
            );
        }
        L::Flowers => {
            let leaf = solid(bank, GREEN);
            let cols = [PINK, WHITE, SKY, GOLD];
            let heart = solid(bank, GOLD);
            for k in 0..12 {
                let a = k as f32 / 12.0 * PI * 2.0;
                let p = hug_point(a, 1.02, -0.06);
                if k % 2 == 0 {
                    let t = solid(bank, cols[(k / 2) % 4]);
                    bx(&mut m, p - Vec3::splat(0.045), p + Vec3::splat(0.045), t);
                    bx(
                        &mut m,
                        p + v(-0.015, 0.045, -0.015),
                        p + v(0.015, 0.055, 0.015),
                        heart,
                    );
                } else {
                    bx(
                        &mut m,
                        p - v(0.03, 0.02, 0.03),
                        p + v(0.03, 0.02, 0.03),
                        leaf,
                    );
                }
            }
        }
        L::Hood(ears, c, inner) => {
            hood(bank, &mut m, c);
            let (outer, inn) = (metal(bank, c), solid(bank, inner));
            match ears {
                Ears::Cat => {
                    // Pointy ears built up in steps, pink on the inside.
                    for s in [-1.0f32, 1.0] {
                        let mut e = Mesh::new();
                        bx(&mut e, v(-0.09, 0.0, -0.035), v(0.09, 0.07, 0.035), outer);
                        bx(&mut e, v(-0.06, 0.07, -0.03), v(0.06, 0.13, 0.03), outer);
                        bx(&mut e, v(-0.03, 0.13, -0.025), v(0.03, 0.18, 0.025), outer);
                        bx(&mut e, v(-0.055, 0.01, 0.035), v(0.055, 0.07, 0.04), inn);
                        bx(&mut e, v(-0.03, 0.07, 0.03), v(0.03, 0.12, 0.035), inn);
                        m.append(
                            &e,
                            Mat4::from_translation(v(s * 0.16, 0.04, -0.02))
                                * Mat4::from_rotation_z(s * -0.3),
                        );
                    }
                }
                Ears::Bunny => {
                    for s in [-1.0f32, 1.0] {
                        let mut e = Mesh::new();
                        bx(&mut e, v(-0.045, 0.0, -0.025), v(0.045, 0.3, 0.025), outer);
                        bx(&mut e, v(-0.025, 0.03, 0.025), v(0.025, 0.26, 0.03), inn);
                        m.append(
                            &e,
                            Mat4::from_translation(v(s * 0.1, 0.04, -0.05))
                                * Mat4::from_rotation_z(s * -0.25),
                        );
                    }
                }
                Ears::Frog => {
                    let pupil = solid(bank, INK);
                    for s in [-1.0f32, 1.0] {
                        let p = v(s * 0.14, 0.1, 0.1);
                        bx(&mut m, p - Vec3::splat(0.065), p + Vec3::splat(0.065), inn);
                        bx(
                            &mut m,
                            p + v(-0.025, -0.01, 0.065),
                            p + v(0.025, 0.035, 0.07),
                            pupil,
                        );
                        bx(
                            &mut m,
                            p + v(-0.075, -0.07, -0.07),
                            p + v(0.075, -0.03, 0.07),
                            outer,
                        );
                    }
                }
            }
        }
        L::MushroomCap(c) => {
            let cap = spotty(bank, c[1], c[0]);
            let rim = solid(bank, c[2]);
            hug(
                &mut m,
                v(0.0, -0.12, 0.0),
                &[(1.12, 0.0), (1.06, 0.12), (0.72, 0.2), (0.0, 0.25)],
                cap,
                false,
            );
            hug(
                &mut m,
                v(0.0, -0.13, 0.0),
                &[(1.13, 0.0), (1.13, 0.02)],
                rim,
                true,
            );
        }
        L::LeafCap => {
            let (g, stem) = (metal(bank, [LIME, GREEN, TEAL]), solid(bank, GREEN));
            hug(
                &mut m,
                v(0.0, -0.12, 0.0),
                &[(1.02, 0.0), (1.0, 0.13), (0.55, 0.17), (0.0, 0.19)],
                g,
                true,
            );
            bx(&mut m, v(-0.015, 0.05, -0.015), v(0.015, 0.14, 0.015), stem);
            let mut leaf = Mesh::new();
            bx(&mut leaf, v(0.0, -0.01, -0.04), v(0.14, 0.01, 0.04), g);
            m.append(
                &leaf,
                Mat4::from_translation(v(0.0, 0.12, 0.0)) * Mat4::from_rotation_z(0.4),
            );
        }
        L::Helm(c, band, horns) => {
            let (t, b) = (metal(bank, c), solid(bank, band));
            hug(
                &mut m,
                v(0.0, -0.22, 0.0),
                &[(1.04, 0.0), (1.04, 0.22), (0.72, 0.32), (0.0, 0.36)],
                t,
                false,
            );
            hug(
                &mut m,
                v(0.0, -0.1, 0.0),
                &[(1.07, 0.0), (1.07, 0.05)],
                b,
                false,
            );
            bx(&mut m, v(-0.03, -0.3, 0.28), v(0.03, -0.06, 0.33), t);
            if horns {
                let ice = metal(bank, [WHITE, WHITE, SKY]);
                for s in [-1.0f32, 1.0] {
                    let mut h = Mesh::new();
                    bx(&mut h, v(-0.035, 0.0, -0.035), v(0.035, 0.1, 0.035), ice);
                    bx(&mut h, v(-0.02, 0.1, -0.02), v(0.02, 0.18, 0.02), ice);
                    m.append(
                        &h,
                        Mat4::from_translation(v(s * 0.29, 0.0, 0.0))
                            * Mat4::from_rotation_z(s * -0.7),
                    );
                }
            }
        }
        L::Miner => {
            let (y, brim, lamp) = (
                metal(bank, [CREAM, GOLD, CLAY]),
                solid(bank, CLAY),
                solid(bank, CREAM),
            );
            hug(
                &mut m,
                v(0.0, -0.14, 0.0),
                &[(1.03, 0.0), (1.0, 0.15), (0.6, 0.25), (0.0, 0.28)],
                y,
                false,
            );
            bx(&mut m, v(-0.26, -0.15, 0.22), v(0.26, -0.12, 0.4), brim);
            bx(&mut m, v(-0.06, -0.06, 0.27), v(0.06, 0.04, 0.34), lamp);
        }
        L::Wizard(c, band) => {
            let (t, b) = (metal(bank, c), solid(bank, band));
            lathe(
                &mut m,
                v(0.0, -0.1, 0.0),
                &[(0.42, 0.0), (0.42, 0.03)],
                10,
                0.0,
                t,
                true,
            );
            hug(
                &mut m,
                v(0.0, -0.08, 0.0),
                &[
                    (1.0, 0.0),
                    (0.98, 0.09),
                    (0.55, 0.25),
                    (0.3, 0.44),
                    (0.0, 0.6),
                ],
                t,
                false,
            );
            hug(
                &mut m,
                v(0.0, -0.07, 0.0),
                &[(1.03, 0.0), (1.02, 0.06)],
                b,
                false,
            );
            let star = solid(bank, CREAM);
            bx(&mut m, v(-0.03, 0.08, 0.26), v(0.03, 0.14, 0.29), star);
        }
        L::Circlet(c, gem) => {
            let (t, g) = (metal(bank, c), solid(bank, gem));
            hug(
                &mut m,
                v(0.0, -0.2, 0.0),
                &[(1.02, 0.0), (1.02, 0.05)],
                t,
                false,
            );
            bx(&mut m, v(-0.04, -0.21, 0.28), v(0.04, -0.13, 0.32), g);
        }
        L::Crown(c, gem) => {
            let (t, g) = (metal(bank, c), solid(bank, gem));
            hug(
                &mut m,
                v(0.0, -0.12, 0.0),
                &[(1.02, 0.0), (1.02, 0.09)],
                t,
                false,
            );
            for k in 0..6 {
                let a = k as f32 / 6.0 * PI * 2.0;
                let p = hug_point(a, 1.0, 0.02);
                bx(&mut m, p - v(0.03, 0.05, 0.03), p + v(0.03, 0.05, 0.03), t);
            }
            bx(&mut m, v(-0.035, -0.1, 0.28), v(0.035, -0.03, 0.31), g);
        }
        L::Crested(c, crest) => {
            let (t, cr) = (metal(bank, c), solid(bank, crest));
            hug(
                &mut m,
                v(0.0, -0.24, 0.0),
                &[(1.04, 0.0), (1.04, 0.24), (0.72, 0.34), (0.0, 0.38)],
                t,
                false,
            );
            // Cheek guards down either side of the face, and a nose guard between.
            for (x0, x1) in [(-0.31, -0.17), (0.17, 0.31)] {
                bx(&mut m, v(x0, -0.4, 0.12), v(x1, -0.2, 0.31), t);
            }
            bx(&mut m, v(-0.03, -0.33, 0.28), v(0.03, -0.1, 0.33), t);
            // The crest, brow to nape, tallest in the middle, then down the back.
            for k in 0..8 {
                let f = k as f32 / 7.0;
                let z = 0.2 - f * 0.46;
                let top = 0.2 + (f * PI).sin() * 0.13;
                bx(
                    &mut m,
                    v(-0.035, 0.0, z - 0.035),
                    v(0.035, top, z + 0.035),
                    cr,
                );
            }
            bx(&mut m, v(-0.035, -0.32, -0.33), v(0.035, 0.1, -0.26), cr);
        }
        L::Nemes(c, stripe) => {
            // Bands of gold and blue.
            let bands = bank.add({
                let mut t = Texture::new(4, 4, c[1]);
                for x in 0..4 {
                    t.set(x, 2, stripe);
                    t.set(x, 3, stripe);
                }
                t
            });
            let tall = bank.add({
                let mut t = Texture::new(4, 8, c[1]);
                for y in [1, 2, 5, 6] {
                    for x in 0..4 {
                        t.set(x, y, stripe);
                    }
                }
                t
            });
            let (gold, brow) = (metal(bank, c), solid(bank, c[0]));
            // The cloth over the crown, and a band across the brow.
            hug(
                &mut m,
                v(0.0, -0.2, 0.0),
                &[(1.08, 0.0), (1.06, 0.2), (0.7, 0.3), (0.0, 0.32)],
                bands,
                false,
            );
            hug(
                &mut m,
                v(0.0, -0.21, 0.0),
                &[(1.1, 0.0), (1.1, 0.05)],
                brow,
                false,
            );
            // Falling behind the ears and down the back, and a lappet down in front of each
            // shoulder.
            for (x0, x1, l0, l1) in [(-0.37, -0.28, -0.37, -0.23), (0.28, 0.37, 0.23, 0.37)] {
                bx(&mut m, v(x0, -0.62, -0.24), v(x1, -0.18, 0.14), tall);
                bx(&mut m, v(l0, -0.86, 0.1), v(l1, -0.44, 0.2), tall);
            }
            bx(&mut m, v(-0.3, -0.6, -0.31), v(0.3, -0.18, -0.24), tall);
            // The cobra rearing on the brow.
            bx(&mut m, v(-0.025, -0.17, 0.31), v(0.025, 0.02, 0.35), gold);
            bx(&mut m, v(-0.045, -0.02, 0.3), v(0.045, 0.045, 0.34), gold);
        }
        L::EyeCirclet(c) => {
            let t = metal(bank, c);
            let (white, iris, pupil, gem) = (
                solid(bank, WHITE),
                solid(bank, LIME),
                solid(bank, INK),
                solid(bank, LAVENDER),
            );
            hug(
                &mut m,
                v(0.0, -0.2, 0.0),
                &[(1.02, 0.0), (1.02, 0.05)],
                t,
                false,
            );
            // A setting on the brow, and the eye in it, staring out with a slit of a pupil.
            bx(&mut m, v(-0.075, -0.235, 0.28), v(0.075, -0.1, 0.31), t);
            bx(
                &mut m,
                v(-0.058, -0.215, 0.31),
                v(0.058, -0.12, 0.325),
                white,
            );
            bx(&mut m, v(-0.03, -0.2, 0.325), v(0.03, -0.135, 0.333), iris);
            bx(
                &mut m,
                v(-0.008, -0.195, 0.333),
                v(0.008, -0.14, 0.337),
                pupil,
            );
            for x in [-0.2f32, 0.2] {
                bx(
                    &mut m,
                    v(x - 0.025, -0.2, 0.21),
                    v(x + 0.025, -0.15, 0.26),
                    gem,
                );
            }
        }
        _ => return None,
    }
    Some(m)
}

// ------------------------------------------------------------------------------------------
// Boots: origin at the sole, the toe points to +z
// ------------------------------------------------------------------------------------------

fn boot_mesh(bank: &mut TexBank, look: L) -> Option<Mesh> {
    let mut m = Mesh::new();
    match look {
        L::Boot(c, sole, h) => {
            let (t, s) = (metal(bank, c), solid(bank, sole));
            bx(&mut m, v(-0.075, 0.02, -0.07), v(0.075, h, 0.1), t);
            bx(&mut m, v(-0.078, 0.0, -0.072), v(0.078, 0.025, 0.105), s);
            bx(
                &mut m,
                v(-0.08, h - 0.02, -0.074),
                v(0.08, h + 0.005, 0.074),
                s,
            );
        }
        L::FrogBoot => {
            let (g, w, k) = (
                metal(bank, [LIME, GREEN, TEAL]),
                solid(bank, WHITE),
                solid(bank, INK),
            );
            bx(&mut m, v(-0.075, 0.0, -0.07), v(0.075, 0.08, 0.11), g);
            for x in [-0.035f32, 0.035] {
                bx(
                    &mut m,
                    v(x - 0.028, 0.08, 0.04),
                    v(x + 0.028, 0.13, 0.095),
                    w,
                );
                bx(
                    &mut m,
                    v(x - 0.012, 0.095, 0.095),
                    v(x + 0.012, 0.12, 0.1),
                    k,
                );
            }
        }
        L::BunnyBoot => {
            let (w, p, k) = (
                metal(bank, [WHITE, WHITE, SAND]),
                solid(bank, BLUSH),
                solid(bank, INK),
            );
            bx(&mut m, v(-0.075, 0.0, -0.07), v(0.075, 0.08, 0.11), w);
            for x in [-0.035f32, 0.035] {
                bx(&mut m, v(x - 0.018, 0.08, 0.0), v(x + 0.018, 0.17, 0.03), w);
                bx(&mut m, v(x - 0.01, 0.09, 0.03), v(x + 0.01, 0.15, 0.035), p);
            }
            bx(&mut m, v(-0.015, 0.05, 0.11), v(0.015, 0.07, 0.115), p);
            bx(&mut m, v(-0.05, 0.06, 0.11), v(-0.03, 0.075, 0.115), k);
            bx(&mut m, v(0.03, 0.06, 0.11), v(0.05, 0.075, 0.115), k);
        }
        L::WingBoot => {
            let (w, s) = (metal(bank, [WHITE, WHITE, SKY]), solid(bank, SAND));
            bx(&mut m, v(-0.075, 0.02, -0.07), v(0.075, 0.13, 0.1), w);
            bx(&mut m, v(-0.078, 0.0, -0.072), v(0.078, 0.025, 0.105), s);
            let mut wing = Mesh::new();
            bx(&mut wing, v(0.0, -0.02, -0.08), v(0.012, 0.02, 0.0), w);
            m.append(
                &wing,
                Mat4::from_translation(v(0.075, 0.1, 0.0)) * Mat4::from_rotation_x(0.5),
            );
            m.append(
                &wing,
                Mat4::from_translation(v(0.075, 0.07, -0.01)) * Mat4::from_rotation_x(0.2),
            );
        }
        L::Sandal(c, wings) => {
            let (strap, sole) = (metal(bank, c), solid(bank, c[2]));
            bx(&mut m, v(-0.078, 0.0, -0.075), v(0.078, 0.03, 0.11), sole);
            // Straps over the foot and wound up round the ankle.
            for z in [0.07f32, 0.01] {
                bx(
                    &mut m,
                    v(-0.08, 0.03, z - 0.018),
                    v(0.08, 0.07, z + 0.018),
                    strap,
                );
            }
            bx(&mut m, v(-0.08, 0.1, -0.075), v(0.08, 0.13, 0.05), strap);
            bx(&mut m, v(-0.08, 0.16, -0.075), v(0.08, 0.19, 0.04), strap);
            if wings {
                // A little white wing sweeping back from each side of the heel.
                let w = solid(bank, WHITE);
                for (x0, x1) in [(-0.094, -0.082), (0.082, 0.094)] {
                    bx(&mut m, v(x0, 0.13, -0.15), v(x1, 0.19, -0.04), w);
                    bx(&mut m, v(x0, 0.17, -0.19), v(x1, 0.23, -0.1), w);
                    bx(&mut m, v(x0, 0.21, -0.22), v(x1, 0.25, -0.15), w);
                }
            }
        }
        _ => return None,
    }
    Some(m)
}

// ------------------------------------------------------------------------------------------
// Shields: centred on the origin, the face looking down +z
// ------------------------------------------------------------------------------------------

fn disc(m: &mut Mesh, r: f32, depth: f32, tex: TexId, z: f32) {
    let mut d = Mesh::new();
    lathe(
        &mut d,
        Vec3::ZERO,
        &[(r, 0.0), (r, depth)],
        10,
        0.0,
        tex,
        true,
    );
    m.append(
        &d,
        Mat4::from_translation(v(0.0, 0.0, z)) * Mat4::from_rotation_x(FRAC_PI_2),
    );
}

fn dome(m: &mut Mesh, r: f32, h: f32, tex: TexId) {
    let mut d = Mesh::new();
    lathe(
        &mut d,
        Vec3::ZERO,
        &[(r, 0.0), (r * 0.8, h * 0.6), (0.0, h)],
        8,
        0.0,
        tex,
        true,
    );
    m.append(&d, Mat4::from_rotation_x(FRAC_PI_2));
}

fn shield_mesh(bank: &mut TexBank, look: L) -> Option<Mesh> {
    let mut m = Mesh::new();
    match look {
        L::Round(c, rim) => {
            let (t, r) = (metal(bank, c), solid(bank, rim));
            disc(&mut m, 0.2, 0.03, r, -0.015);
            disc(&mut m, 0.17, 0.02, t, 0.0);
            let boss = solid(bank, CREAM);
            bx(&mut m, v(-0.04, -0.04, 0.01), v(0.04, 0.04, 0.05), boss);
        }
        L::Kite(c, rim) => {
            let (t, r) = (metal(bank, c), solid(bank, rim));
            for (y, w, h) in [
                (0.1, 0.34, 0.12),
                (-0.02, 0.28, 0.12),
                (-0.12, 0.18, 0.08),
                (-0.18, 0.08, 0.05),
            ] {
                bx(
                    &mut m,
                    v(-w * 0.5, y - h * 0.5, -0.02),
                    v(w * 0.5, y + h * 0.5, 0.02),
                    t,
                );
            }
            bx(&mut m, v(-0.02, -0.2, 0.02), v(0.02, 0.16, 0.035), r);
            bx(&mut m, v(-0.15, 0.06, 0.02), v(0.15, 0.1, 0.035), r);
        }
        L::PotLid => {
            let (lid, knob) = (metal(bank, [WHITE, SAND, KHAKI]), solid(bank, SHADOW));
            disc(&mut m, 0.19, 0.025, lid, -0.01);
            dome(&mut m, 0.15, 0.05, lid);
            bx(&mut m, v(-0.03, -0.03, 0.05), v(0.03, 0.03, 0.09), knob);
        }
        L::Turtle => {
            let mut t = Texture::new(8, 8, GREEN);
            for i in 0..8 {
                t.set(i, 3, TEAL);
                t.set(3, i, TEAL);
                t.set(i, 7, TEAL);
            }
            t.set(1, 1, LIME);
            t.set(5, 5, LIME);
            let shell = bank.add(t);
            let rim = solid(bank, GOLD);
            disc(&mut m, 0.21, 0.02, rim, -0.02);
            dome(&mut m, 0.19, 0.1, shell);
        }
        L::LeafShield => {
            let (g, vein) = (metal(bank, [LIME, GREEN, TEAL]), solid(bank, LIME));
            let mut p = Mesh::new();
            bx(&mut p, v(-0.13, -0.13, -0.015), v(0.13, 0.13, 0.015), g);
            m.append(&p, Mat4::from_rotation_z(PI / 4.0));
            bx(&mut m, v(-0.015, -0.2, 0.015), v(0.015, 0.2, 0.025), vein);
        }
        L::MushroomShield => {
            let cap = spotty(bank, RED, WHITE);
            let rim = solid(bank, CRIMSON);
            disc(&mut m, 0.21, 0.02, rim, -0.02);
            dome(&mut m, 0.2, 0.09, cap);
        }
        L::Aspis(c, rim) => {
            let (edge, face) = (metal(bank, c), bank.add(aspis_face(c, rim)));
            disc(&mut m, 0.215, 0.03, edge, -0.015);
            disc(&mut m, 0.19, 0.02, face, 0.0);
        }
        L::Scarab(c, rim) => {
            let (shell, gold) = (metal(bank, c), metal(bank, [CREAM, rim, CLAY]));
            // Its two wing cases, meeting down the middle.
            for sx in [-1.0f32, 1.0] {
                let mut wing = Mesh::new();
                dome(&mut wing, 0.1, 0.08, shell);
                m.append(
                    &wing,
                    Mat4::from_translation(v(sx * 0.075, -0.03, 0.0))
                        * Mat4::from_scale(v(0.85, 1.9, 1.0)),
                );
            }
            // Its head and little horns up top, and three legs a side.
            bx(&mut m, v(-0.075, 0.16, -0.01), v(0.075, 0.24, 0.05), gold);
            for (x0, x1) in [(-0.06, -0.035), (0.035, 0.06)] {
                bx(&mut m, v(x0, 0.24, 0.0), v(x1, 0.3, 0.03), gold);
            }
            for (y, a) in [(0.1f32, 0.45f32), (-0.03, 0.0), (-0.16, -0.45)] {
                for sx in [-1.0f32, 1.0] {
                    let mut leg = Mesh::new();
                    bx(
                        &mut leg,
                        v(0.0, -0.014, -0.012),
                        v(0.09, 0.014, 0.012),
                        gold,
                    );
                    m.append(
                        &leg,
                        Mat4::from_translation(v(sx * 0.15, y, 0.01))
                            * Mat4::from_rotation_z(if sx > 0.0 { a } else { PI - a }),
                    );
                }
            }
        }
        L::EyeKite(c, rim) => {
            let (t, r) = (metal(bank, c), solid(bank, rim));
            for (y, w, h) in [
                (0.1, 0.34, 0.12),
                (-0.02, 0.28, 0.12),
                (-0.12, 0.18, 0.08),
                (-0.18, 0.08, 0.05),
            ] {
                bx(
                    &mut m,
                    v(-w * 0.5, y - h * 0.5, -0.02),
                    v(w * 0.5, y + h * 0.5, 0.02),
                    t,
                );
            }
            // A rune-bordered eye glaring out of the middle.
            let (white, iris, pupil) = (solid(bank, WHITE), solid(bank, LIME), solid(bank, INK));
            bx(&mut m, v(-0.1, -0.005, 0.02), v(0.1, 0.105, 0.03), r);
            bx(&mut m, v(-0.08, 0.01, 0.03), v(0.08, 0.09, 0.038), white);
            bx(
                &mut m,
                v(-0.035, 0.015, 0.038),
                v(0.035, 0.085, 0.044),
                iris,
            );
            bx(&mut m, v(-0.01, 0.02, 0.044), v(0.01, 0.08, 0.048), pupil);
            for (x0, x1) in [(-0.13, -0.11), (0.11, 0.13)] {
                bx(&mut m, v(x0, -0.06, 0.02), v(x1, 0.14, 0.03), r);
            }
        }
        _ => return None,
    }
    Some(m)
}

/// The face of a bronze aspis: the maze beaten into its middle, and a meander running round
/// inside the rim, in `ink`.
fn aspis_face(c: [u8; 3], ink: u8) -> Texture {
    const MAZE: [&str; 7] = [
        "#######", "#.....#", "#.###.#", "#.#.#.#", "#.#...#", "#.#####", "#......",
    ];
    let mut t = Texture::new(16, 16, c[1]);
    for y in 0..16 {
        for x in 0..16 {
            let (dx, dy) = (x as f32 - 7.5, y as f32 - 7.5);
            let r = dx.hypot(dy);
            if (5.6..7.6).contains(&r) {
                let a = (dy.atan2(dx) + PI) / (2.0 * PI);
                let step = (a * 20.0) as i32;
                let outer = r > 6.6;
                let dark = match step % 4 {
                    0 => true,
                    1 => outer,
                    2 => false,
                    _ => !outer,
                };
                t.set(x, y, if dark { ink } else { c[0] });
            } else if r > 7.6 {
                t.set(x, y, c[2]);
            }
        }
    }
    for (row, line) in MAZE.iter().enumerate() {
        for (col, ch) in line.chars().enumerate() {
            if ch == '#' {
                t.set(4 + col as i32, 4 + row as i32, ink);
            }
        }
    }
    t
}

// ------------------------------------------------------------------------------------------
// Clothes: textures swapped onto the hero's body, arms and legs
// ------------------------------------------------------------------------------------------

fn chest_skin(bank: &mut TexBank, c: [u8; 3], belt: u8, pat: Pattern) -> (TexId, TexId) {
    let look = Look {
        shirt: c,
        belt,
        ..HERO
    };
    let mut t = body_tex(&look);
    // Front (0,0) 6x5, back (6,0), sides (12,0) 4x5.
    match pat {
        Pattern::Plain => {}
        Pattern::Stripes => {
            for y in [1, 3] {
                for x in 0..16 {
                    t.set(x, y, c[0]);
                }
            }
        }
        Pattern::Chain => {
            for y in 0..5 {
                for x in 0..16 {
                    if (x + y) % 2 == 0 && y != 3 {
                        t.set(x, y, c[0]);
                    }
                }
            }
        }
        Pattern::Plate => {
            for y in 0..3 {
                t.set(2, y, c[0]);
                t.set(3, y, c[0]);
            }
            t.set(0, 0, belt);
            t.set(5, 0, belt);
        }
        Pattern::Stars => {
            for (x, y) in [(1, 1), (4, 2), (2, 4), (7, 1), (10, 3), (13, 2)] {
                t.set(x, y, CREAM);
            }
            for x in 0..16 {
                t.set(x, 3, c[1]);
            }
        }
        Pattern::Buttons => {
            for y in [1, 2, 4] {
                t.set(3, y, belt);
            }
        }
        Pattern::Zigzag => {
            for x in 0..16 {
                t.set(x, if x % 2 == 0 { 1 } else { 2 }, belt);
            }
        }
        Pattern::Cuirass => {
            // The chest's curve and a line down the middle, gilt at the shoulders...
            for x in 1..5 {
                t.set(x, 1, c[2]);
            }
            t.set(2, 2, c[2]);
            t.set(3, 2, c[2]);
            t.set(0, 0, belt);
            t.set(5, 0, belt);
            // ...and a fringe of gilded strips all round below the belt.
            for x in 0..16 {
                t.set(x, 4, if x % 2 == 0 { belt } else { c[2] });
            }
        }
        Pattern::Collar => {
            for x in 0..16 {
                t.set(x, 0, if x % 2 == 0 { belt } else { TEAL });
                t.set(x, 1, if x % 2 == 0 { BLUE } else { belt });
            }
        }
        Pattern::Runes => {
            // A rune glowing on the breast, and more scattered round.
            for (x, y) in [(2, 1), (3, 1), (2, 2), (3, 0), (1, 3), (4, 3)] {
                t.set(x, y, belt);
            }
            for (x, y) in [(8, 1), (10, 2), (7, 3), (13, 1), (14, 3)] {
                t.set(x, y, belt);
            }
        }
    }
    let arm = limb_tex(c[1], c[2], HERO.skin[1]);
    (bank.add(t), bank.add(arm))
}

fn leg_skin(bank: &mut TexBank, c: [u8; 2], accent: u8, pat: LegPattern) -> TexId {
    let mut t = limb_tex(c[0], c[1], HERO.boots);
    match pat {
        LegPattern::Plain => {}
        LegPattern::Patched => t.set(1, 1, accent),
        LegPattern::Skirt => {
            for x in 0..4 {
                t.set(x, 0, if x % 2 == 0 { accent } else { c[0] });
                t.set(x, 1, if x % 2 == 1 { accent } else { c[1] });
            }
        }
        LegPattern::Plated => {
            t.set(0, 1, accent);
            t.set(1, 1, accent);
        }
        LegPattern::Striped => {
            t.set(1, 0, accent);
            t.set(1, 1, accent);
            t.set(1, 2, accent);
        }
        LegPattern::Stars => {
            t.set(0, 0, accent);
            t.set(2, 2, accent);
        }
        LegPattern::Kilt => {
            for x in 0..4 {
                t.set(x, 0, accent);
                t.set(x, 1, if x % 2 == 0 { c[0] } else { c[1] });
                t.set(x, 2, HERO.skin[0]);
            }
        }
    }
    bank.add(t)
}

/// Builds every gear icon (into `icons`) and every gear look.
pub fn build(bank: &mut TexBank, icons: &mut HashMap<&'static str, TexId>) -> GearArt {
    let mut g = GearArt::default();
    for (name, tpl, slots, look) in table() {
        if let Some(rows) = tpl {
            icons.insert(name, bank.add(art(rows, slots)));
        }
        if let Some(m) = held_mesh(bank, look) {
            g.held.insert(name, m);
        }
        if let Some(m) = hat_mesh(bank, look) {
            g.hats.insert(name, m);
        }
        if let Some(m) = boot_mesh(bank, look) {
            g.boots.insert(name, m);
        }
        if let Some(m) = shield_mesh(bank, look) {
            g.shields.insert(name, m);
        }
        match look {
            L::Chest(c, belt, pat) => {
                g.chest.insert(name, chest_skin(bank, c, belt, pat));
            }
            L::Legs(c, accent, pat) => {
                g.legs.insert(name, leg_skin(bank, c, accent, pat));
            }
            _ => {}
        }
    }
    g
}

/// Names of every gear look (for tests).
#[cfg(test)]
pub fn names() -> Vec<&'static str> {
    table().into_iter().map(|e| e.0).collect()
}

#[cfg(test)]
pub fn templates() -> Vec<&'static [&'static str]> {
    table().into_iter().filter_map(|e| e.1).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gear_art_is_tidy() {
        let names = names();
        let mut seen = std::collections::HashSet::new();
        for n in &names {
            assert!(seen.insert(*n), "{n} listed twice");
        }
        for rows in templates() {
            assert!(rows.len() <= 16);
            assert!(rows.iter().all(|r| r.chars().count() <= 16), "{rows:?}");
        }
        let mut bank = TexBank::default();
        let mut icons = HashMap::new();
        let g = build(&mut bank, &mut icons);
        assert!(g.held.len() + g.hats.len() + g.boots.len() + g.shields.len() >= 90);
        assert!(bank_ok(&bank, &icons));
    }

    fn bank_ok(bank: &TexBank, icons: &HashMap<&'static str, TexId>) -> bool {
        icons
            .values()
            .all(|id| bank.get(*id).data.iter().all(|&c| c < 32 || c == CLEAR))
    }
}
