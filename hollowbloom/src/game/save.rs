//! Save files: JSON in the platform's data directory.

use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use super::folk::Friends;
use super::items::{Item, Stack};
use super::play::{Play, Stats};
use super::player::{Player, PlayerSave};
use super::quests::{Journal, Quest};
use super::world::{Floor, Obj, Wall, World};

pub const VERSION: u32 = 5;

#[derive(Serialize, Deserialize)]
pub struct FarmSave {
    pub w: i32,
    pub h: i32,
    pub floor: String,
    pub wall: String,
    pub flags: String,
    pub objs: Vec<(i32, i32, Obj)>,
}

#[derive(Serialize, Deserialize)]
pub struct SaveData {
    pub version: u32,
    pub seed: u64,
    pub day: u32,
    pub minutes: f32,
    /// Money in copper (the key is from version 1, when there was only gold).
    pub gold: u64,
    pub deepest: u32,
    pub waystones: Vec<u32>,
    pub player: PlayerSave,
    pub farm: FarmSave,
    pub shipping: Vec<Stack>,
    pub rain: bool,
    pub stats: Stats,
    /// Burrowby's specials bought on the saved day.
    #[serde(default)]
    pub bought: Vec<usize>,
    /// Bramblewick: town projects, friendships, quests and records (version 3).
    #[serde(default)]
    pub restored: u32,
    #[serde(default)]
    pub friends: Friends,
    #[serde(default)]
    pub quests: Vec<Quest>,
    #[serde(default)]
    pub done: Vec<String>,
    #[serde(default)]
    pub journal: Journal,
    #[serde(default)]
    pub taken: Vec<u32>,
    #[serde(default)]
    pub marks: u32,
    #[serde(default)]
    pub rank: u8,
    #[serde(default)]
    pub spells: super::spells::Spellbook,
    /// Inside the farmhouse (version 4).
    #[serde(default)]
    pub house: super::home::HouseSave,
    #[serde(default)]
    pub warmed: u32,
    #[serde(default)]
    pub stargazed: u32,
    /// Recipes learned (version 5). Saves from before recipes had to be found know them
    /// all.
    #[serde(default)]
    pub recipes: Option<Vec<Item>>,
    /// Bombs the smith still has to sell on the saved day.
    #[serde(default)]
    pub bombs: Option<u8>,
}

const FLOORS: [(Floor, char); 15] = [
    (Floor::Void, '.'),
    (Floor::Grass, 'g'),
    (Floor::Path, 'p'),
    (Floor::Soil, 's'),
    (Floor::Tilled, 't'),
    (Floor::Sand, 'a'),
    (Floor::Water, 'w'),
    (Floor::Planks, 'W'),
    (Floor::Cobble, 'S'),
    (Floor::Cave, 'c'),
    (Floor::Lava, 'l'),
    (Floor::Street, 'r'),
    (Floor::Plaza, 'z'),
    (Floor::Tiles, 'T'),
    (Floor::Carpet, 'C'),
];

fn floor_char(f: Floor) -> char {
    FLOORS
        .iter()
        .find(|(k, _)| *k == f)
        .map_or('.', |(_, c)| *c)
}

fn char_floor(c: char) -> Floor {
    FLOORS
        .iter()
        .find(|(_, k)| *k == c)
        .map_or(Floor::Grass, |(f, _)| *f)
}

fn wall_char(w: Wall) -> char {
    match w {
        Wall::None => '.',
        Wall::Rock => 'r',
        Wall::Ore(n) => char::from(b'0' + n.min(9)),
        Wall::Bedrock => 'b',
        Wall::Cliff => 'c',
        Wall::Brick => 'S',
        Wall::Timber => 'W',
        Wall::Hedge => 'h',
        Wall::Paper(_) => '.',
    }
}

fn char_wall(c: char) -> Wall {
    match c {
        'r' => Wall::Rock,
        'b' => Wall::Bedrock,
        'c' => Wall::Cliff,
        'S' => Wall::Brick,
        'W' => Wall::Timber,
        'h' => Wall::Hedge,
        d if d.is_ascii_digit() => Wall::Ore(d as u8 - b'0'),
        _ => Wall::None,
    }
}

pub fn save_farm(w: &World) -> FarmSave {
    let mut objs = Vec::new();
    for z in 0..w.h {
        for x in 0..w.w {
            if let Some(o) = w.obj(x, z) {
                objs.push((x, z, o.clone()));
            }
        }
    }
    FarmSave {
        w: w.w,
        h: w.h,
        floor: w.floor.iter().map(|f| floor_char(*f)).collect(),
        wall: w.wall.iter().map(|x| wall_char(*x)).collect(),
        flags: w.flags.iter().map(|f| char::from(b'0' + (f & 7))).collect(),
        objs,
    }
}

pub fn load_farm(s: &FarmSave, into: &mut World) {
    if s.w != into.w || s.h != into.h {
        return;
    }
    for (i, c) in s.floor.chars().enumerate().take(into.floor.len()) {
        into.floor[i] = char_floor(c);
    }
    for (i, c) in s.wall.chars().enumerate().take(into.wall.len()) {
        into.wall[i] = char_wall(c);
    }
    for (i, c) in s.flags.chars().enumerate().take(into.flags.len()) {
        into.flags[i] = (c as u8).saturating_sub(b'0');
    }
    for o in into.objs.iter_mut() {
        *o = None;
    }
    for (x, z, o) in &s.objs {
        into.set_obj(*x, *z, Some(o.clone()));
    }
    into.touch_all();
}

pub fn dir() -> Option<PathBuf> {
    if let Some(p) = std::env::var_os("HOLLOWBLOOM_DATA") {
        return Some(PathBuf::from(p));
    }
    #[cfg(windows)]
    {
        std::env::var_os("APPDATA").map(|a| PathBuf::from(a).join("Hollowbloom"))
    }
    #[cfg(target_os = "macos")]
    {
        std::env::var_os("HOME")
            .map(|h| PathBuf::from(h).join("Library/Application Support/Hollowbloom"))
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        if let Some(x) = std::env::var_os("XDG_DATA_HOME") {
            return Some(PathBuf::from(x).join("hollowbloom"));
        }
        std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".local/share/hollowbloom"))
    }
    #[cfg(not(any(windows, unix)))]
    {
        None
    }
}

fn path() -> Option<PathBuf> {
    dir().map(|d| d.join("save.json"))
}

pub fn exists() -> bool {
    path().is_some_and(|p| p.exists())
}

pub fn write(p: &Play) -> Result<(), String> {
    let data = SaveData {
        version: VERSION,
        seed: p.seed,
        day: p.clock.day,
        minutes: p.clock.min,
        gold: p.money,
        deepest: p.deepest,
        waystones: p.waystones.clone(),
        player: p.player.save(),
        farm: save_farm(&p.farm),
        shipping: p.shipping.clone(),
        rain: p.rain,
        stats: p.stats.clone(),
        bought: p.bought.clone(),
        restored: p.restored,
        friends: p.friends.clone(),
        quests: p.quests.clone(),
        done: p.done.clone(),
        journal: p.journal.clone(),
        taken: p.taken.clone(),
        marks: p.marks,
        rank: p.rank,
        spells: p.spells.clone(),
        house: p.house.save(),
        warmed: p.warmed,
        stargazed: p.stargazed,
        recipes: Some(p.known.iter().copied().collect()),
        bombs: Some(p.bomb_stock),
    };
    let json = serde_json::to_string(&data).map_err(|e| e.to_string())?;
    let path = path().ok_or("no data directory")?;
    if let Some(d) = path.parent() {
        fs::create_dir_all(d).map_err(|e| e.to_string())?;
    }
    let tmp = path.with_extension("json.tmp");
    fs::write(&tmp, json).map_err(|e| e.to_string())?;
    fs::rename(&tmp, &path).map_err(|e| e.to_string())
}

pub fn read() -> Result<Play, String> {
    let path = path().ok_or("no data directory")?;
    let text = fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let d: SaveData = serde_json::from_str(&text).map_err(|e| e.to_string())?;
    let mut p = Play::new(d.seed);
    p.clock.day = d.day.max(1);
    p.clock.min = d
        .minutes
        .clamp(super::play::DAY_START, super::play::DAY_END - 60.0);
    p.money = d.gold;
    p.deepest = d.deepest;
    p.waystones = d.waystones;
    load_farm(&d.farm, &mut p.farm);
    // Farms from before enchanting get a table by the house.
    let has_table = p
        .farm
        .objs
        .iter()
        .any(|o| matches!(o, Some(Obj::EnchantTable)));
    let (ex, ez) = super::farm::MARKS.enchant;
    if !has_table && p.farm.obj(ex, ez).is_none() && p.farm.wall(ex, ez) == Wall::None {
        p.farm.set_obj(ex, ez, Some(Obj::EnchantTable));
    }
    // Farms from before the bus get the road to town.
    if !p.farm.objs.iter().any(|o| matches!(o, Some(Obj::BusStop))) {
        super::farm::lay_road(&mut p.farm);
    }
    p.restored = d.restored;
    p.friends = d.friends;
    p.friends.fix();
    // Quests whose story no longer exists are dropped quietly.
    p.quests = d
        .quests
        .into_iter()
        .filter(|q| q.request().is_some() || q.story().is_some())
        .collect();
    p.done = d.done;
    p.journal = d.journal;
    p.taken = d.taken;
    p.marks = d.marks;
    p.rank = d.rank.min(super::quests::RANKS.len() as u8 - 1);
    p.spells = d.spells;
    p.spells.fix();
    // Farms from before the house had rooms get a freshly furnished one.
    p.house = super::home::House::load(&d.house);
    p.warmed = d.warmed;
    p.stargazed = d.stargazed;
    p.town = super::town::generate(p.restored);
    let (dx, dz) = super::farm::MARKS.door;
    let pos = glam::Vec2::new(dx as f32 + 0.5, dz as f32 + 0.6);
    p.player = Player::load(d.player, pos);
    p.known = match d.recipes {
        Some(list) => list.into_iter().collect(),
        None => super::items::RECIPES.iter().map(|r| r.out).collect(),
    };
    p.bag_seen = p.player.inv.distinct();
    p.bomb_stock = d.bombs.unwrap_or(super::bombs::BOMBS_PER_DAY);
    p.shipping = d.shipping;
    p.rain = d.rain;
    p.stats = d.stats;
    p.bought = d.bought;
    // Farm chests may hold gear from older saves without rolls.
    for o in p.farm.objs.iter_mut().flatten() {
        if let Obj::Chest { items } = o {
            for s in items.iter_mut().flatten() {
                s.normalize();
            }
        }
    }
    p.cam_pos = p.player.world_pos();
    p.banner = Some(super::play::Banner {
        title: format!("Day {}", p.clock.day),
        sub: "Welcome back".into(),
        t: 0.0,
    });
    Ok(p)
}

#[derive(Serialize, Deserialize)]
struct SettingsFile {
    music: f32,
    sfx: f32,
    fullscreen: bool,
    shake: bool,
    #[serde(default = "yes")]
    shadows: bool,
    #[serde(default = "yes")]
    ao: bool,
}

fn yes() -> bool {
    true
}

pub fn load_settings() -> super::Settings {
    let mut s = super::Settings::default();
    if let Some(p) = dir().map(|d| d.join("settings.json")) {
        if let Ok(t) = fs::read_to_string(p) {
            if let Ok(f) = serde_json::from_str::<SettingsFile>(&t) {
                s.music = f.music;
                s.sfx = f.sfx;
                s.fullscreen = f.fullscreen;
                s.shake = f.shake;
                s.shadows = f.shadows;
                s.ao = f.ao;
            }
        }
    }
    s
}

pub fn save_settings(s: &super::Settings) {
    let Some(d) = dir() else { return };
    let _ = fs::create_dir_all(&d);
    let f = SettingsFile {
        music: s.music,
        sfx: s.sfx,
        fullscreen: s.fullscreen,
        shake: s.shake,
        shadows: s.shadows,
        ao: s.ao,
    };
    if let Ok(j) = serde_json::to_string_pretty(&f) {
        let _ = fs::write(d.join("settings.json"), j);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn farm_round_trip() {
        let mut a = super::super::farm::generate(3);
        a.set_obj(
            10,
            10,
            Some(Obj::Chest {
                items: vec![Some(Stack::new(super::super::items::Item::Wood, 5)), None],
            }),
        );
        a.set_wall(12, 12, Wall::Brick);
        let s = save_farm(&a);
        let json = serde_json::to_string(&s).unwrap();
        let back: FarmSave = serde_json::from_str(&json).unwrap();
        let mut b = super::super::farm::generate(99);
        load_farm(&back, &mut b);
        assert_eq!(a.floor, b.floor);
        assert_eq!(a.wall, b.wall);
        assert!(matches!(b.obj(10, 10), Some(Obj::Chest { .. })));
    }
}
