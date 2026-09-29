//! The playing state: the farm, the Hollow, and everything the player does in them.

use glam::{Vec2, Vec3};

use super::canyon::QUICKSAND_DRAG;
use super::combat::{Bolt, Flash};
use super::draw::Env;
use super::dungeon::{self, Level, is_waystone_floor, ore_item};
use super::farm::{self, MARKS, tillable};
use super::foes::{Call, Enemy, HOWL_REACH, MEND_AGAIN, MEND_REACH, St};
use super::fx::{Drop, Fx, Rune, Shot, ShotKind};
use super::gear::{Class, Rarity, Stat};
use super::items::{Crop, Inventory, Item, Kind, Placeable, Stack};
use super::loot::{self, Fortune};
use super::menus::Menu;
use super::pets::{Cat, Hatch, Spider};
use super::player::{Act, ActKind, HOTBAR, Player, RADIUS};
use super::town::{self, BUILDINGS, Place, Room};
use super::travel::{Bus, area_world, area_world_mut};
use super::world::{Area, FERTILE, Floor, Obj, WATERED, Wall, World};
use super::{Io, Settings};
use crate::assets::BIOME_STYLES;
use crate::audio::{Sfx, Song};
use crate::input::{Action, KeyCode};
use crate::palette::*;
use crate::render::Camera;
use crate::util::{Rng, approach, damp, hash2, wrap_angle};

pub const MIN_PER_SEC: f32 = 1.55;
pub const DAY_START: f32 = 360.0;
pub const DAY_END: f32 = 1560.0;
/// Days in each season: spring from day 1, then summer, autumn and winter, and round again.
pub const SEASON_DAYS: u32 = 28;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Season {
    Spring,
    Summer,
    Autumn,
    Winter,
}

impl Season {
    pub fn name(self) -> &'static str {
        match self {
            Season::Spring => "Spring",
            Season::Summer => "Summer",
            Season::Autumn => "Autumn",
            Season::Winter => "Winter",
        }
    }

    /// What the morning it begins feels like.
    pub fn greeting(self) -> &'static str {
        match self {
            Season::Spring => "the farm is waking up.",
            Season::Summer => "long, golden days ahead.",
            Season::Autumn => "the leaves are turning.",
            Season::Winter => "frost on the windows.",
        }
    }

    /// Its bit, as crops keep their seasons (`items::SPRING`...).
    pub fn bit(self) -> u8 {
        1 << self as u8
    }

    /// Readable on the HUD's paper.
    pub fn color(self) -> u8 {
        match self {
            Season::Spring => DEEP_TEAL,
            Season::Summer => CLAY,
            Season::Autumn => CRIMSON,
            Season::Winter => INDIGO,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Clock {
    pub day: u32,
    pub min: f32,
}

impl Clock {
    pub fn label(&self) -> String {
        let m = (self.min as u32 / 10) * 10;
        let h24 = (m / 60) % 24;
        let mm = m % 60;
        let (h12, ap) = match h24 {
            0 => (12, "am"),
            1..=11 => (h24, "am"),
            12 => (12, "pm"),
            _ => (h24 - 12, "pm"),
        };
        format!("{h12}:{mm:02}{ap}")
    }

    pub fn weekday(&self) -> &'static str {
        ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"][((self.day.max(1) - 1) % 7) as usize]
    }

    pub fn season(&self) -> Season {
        match (self.day.max(1) - 1) / SEASON_DAYS % 4 {
            0 => Season::Spring,
            1 => Season::Summer,
            2 => Season::Autumn,
            _ => Season::Winter,
        }
    }

    /// How far into its season today is, from 1 to `SEASON_DAYS`.
    pub fn season_day(&self) -> u32 {
        (self.day.max(1) - 1) % SEASON_DAYS + 1
    }
}

/// What happens when a screen fade reaches black.
#[derive(Clone, Copy, Debug)]
pub enum Trans {
    Descend {
        depth: u32,
        via_waystone: bool,
    },
    Home,
    Sleep {
        passed_out: bool,
    },
    Faint,
    /// Riding the bus to town (or home).
    Bus {
        to_town: bool,
    },
    /// Through a door into a building.
    Enter(Place),
    /// Back out into the street.
    Leave,
    /// In or out of the farmhouse.
    House {
        enter: bool,
    },
    /// Down a blasted hole into a floor's secret room, or back up the rope.
    Vault {
        enter: bool,
    },
}

pub struct Fade {
    pub t: f32,
    pub action: Trans,
    pub fired: bool,
}

pub struct Toast {
    pub text: String,
    pub icon: Option<Item>,
    pub n: u32,
    pub t: f32,
    pub color: u8,
}

pub struct Banner {
    pub title: String,
    pub sub: String,
    pub t: f32,
}

pub struct Play {
    pub seed: u64,
    pub farm: World,
    /// Bramblewick, rebuilt from its plan whenever you arrive.
    pub town: World,
    /// The inside of the building you're in.
    pub room: Option<Room>,
    /// Inside the farmhouse: furniture, rugs, pictures, wallpaper.
    pub house: super::home::House,
    /// A line in the water.
    pub fishing: Option<super::fish::Fishing>,
    /// How far the camera has leaned in on a catch being shown off (0..1).
    pub catch_zoom: f32,
    /// A dish on the stove.
    pub cooking: Option<super::home::Cooking>,
    /// The globe spinning, the day the fire last warmed you and the day you last saw a
    /// shooting star.
    pub house_spin: f32,
    pub warmed: u32,
    pub stargazed: u32,
    /// Town projects finished (bits from `town`).
    pub restored: u32,
    pub bus: Option<Bus>,
    /// How long you've been leaning on a shop door.
    pub door_push: f32,
    /// Cool-down for gentle reminders.
    pub nag: f32,
    /// Townsfolk where you can see them.
    pub folk: Vec<super::folk::Npc>,
    pub friends: super::folk::Friends,
    /// Quests you've taken on, and the story quests you've finished.
    pub quests: Vec<super::quests::Quest>,
    pub done: Vec<String>,
    pub journal: super::quests::Journal,
    /// Notices already taken from the boards.
    pub taken: Vec<u32>,
    /// Lantern Guild marks and rank.
    pub marks: u32,
    pub rank: u8,
    /// A quest just finished, to celebrate.
    pub cheer: Option<super::quests::Cheer>,
    /// The day the Wishing Tree last blessed you.
    pub blessed: u32,
    /// The hour the town clock last struck.
    pub last_hour: i32,
    pub level: Option<Level>,
    pub area: Area,
    pub player: Player,
    pub foes: Vec<Enemy>,
    pub drops: Vec<Drop>,
    pub shots: Vec<Shot>,
    /// Book-worm ink dried into runes on the floor (see `ink`).
    pub runes: Vec<Rune>,
    /// The hero's wand bolts.
    pub bolts: Vec<Bolt>,
    /// Brief flashes of light from magic.
    pub flashes: Vec<Flash>,
    /// Spells you know and the two you have ready.
    pub spells: super::spells::Spellbook,
    /// The spell on its way out of your hands.
    pub casting: Option<super::spells::Spell>,
    /// Starfall's stars, and Chain Spark's lightning.
    pub stars: Vec<super::magic::Star>,
    pub zaps: Vec<super::magic::Zap>,
    /// Burrowby's specials already bought today.
    pub bought: Vec<usize>,
    pub fx: Fx,
    pub rng: Rng,
    pub clock: Clock,
    /// Money, counted in copper (10 copper = 1 silver, 100 copper = 1 gold).
    pub money: u64,
    pub deepest: u32,
    pub waystones: Vec<u32>,
    pub shipping: Vec<Stack>,
    pub menu: Menu,
    pub held: Option<Stack>,
    pub cam: Camera,
    pub cam_pos: Vec3,
    pub shake: f32,
    pub fade: Option<Fade>,
    pub toasts: Vec<Toast>,
    pub banner: Option<Banner>,
    pub cat: Cat,
    /// The jumping spider, once Pip's egg has hatched, and the egg while it hatches.
    pub spider: Option<Spider>,
    pub hatching: Option<Hatch>,
    pub rain: bool,
    pub time: f32,
    pub target: Option<(i32, i32)>,
    pub target_ok: bool,
    /// The ground point under the mouse, when aiming with it.
    pub aim: Option<Vec2>,
    pub hint: Option<String>,
    pub sel_name_t: f32,
    pub revealed: Vec<bool>,
    pub show_map: bool,
    pub boss_seen: Option<String>,
    pub stats: Stats,
    /// Recipes learned so far (by what they make), ones just learned and still to be shown,
    /// and the different things in the bag when that was last checked.
    pub known: std::collections::BTreeSet<Item>,
    pub discoveries: std::collections::VecDeque<usize>,
    pub bag_seen: Vec<Item>,
    /// Playing on a controller (the Steam Deck's, or any pad): hints show its buttons.
    pub pad: bool,
    /// Candy rocks bursting up out of the floor where a guardian fell.
    pub erupting: Vec<super::candy::Eruption>,
    /// Lit bombs, and how many the smith has left to sell today.
    pub bombs: Vec<super::bombs::Bomb>,
    pub bomb_stock: u8,
    /// The floor above while you're down in its secret room, and the secret room as you
    /// left it while you're back up top.
    pub below: Option<Box<super::bombs::Stash>>,
    pub vault: Option<Box<super::bombs::Stash>>,
    pub quit_to_title: bool,
    pub last_summary: Option<super::menus::Summary>,
    pub saved_day: u32,
}

#[derive(Default, Clone, serde::Serialize, serde::Deserialize)]
pub struct Stats {
    pub kills: u32,
    pub harvested: u32,
    pub earned: u64,
    pub floors: u32,
    #[serde(default)]
    pub quests: u32,
    #[serde(default)]
    pub casts: u32,
    #[serde(default)]
    pub brewed: u32,
    #[serde(default)]
    pub caught: u32,
    #[serde(default)]
    pub cooked: u32,
    /// Guardians felled, and gleaming chests opened.
    #[serde(default)]
    pub guardians: u32,
    #[serde(default)]
    pub gleams: u32,
}

impl Play {
    pub fn new(seed: u64) -> Play {
        let farm = farm::generate(seed);
        let spawn = MARKS.spawn;
        let player = Player::new(Vec2::new(spawn.0 as f32 + 0.5, spawn.1 as f32 + 0.5));
        let mut p = Play {
            seed,
            farm,
            town: town::generate(0),
            room: None,
            house: super::home::House::new(),
            fishing: None,
            catch_zoom: 0.0,
            cooking: None,
            house_spin: 0.0,
            warmed: 0,
            stargazed: 0,
            restored: 0,
            bus: None,
            door_push: 0.0,
            nag: 0.0,
            folk: Vec::new(),
            friends: super::folk::Friends::default(),
            quests: Vec::new(),
            done: Vec::new(),
            journal: super::quests::Journal::default(),
            taken: Vec::new(),
            marks: 0,
            rank: 0,
            cheer: None,
            blessed: 0,
            last_hour: -1,
            level: None,
            area: Area::Farm,
            player,
            foes: Vec::new(),
            drops: Vec::new(),
            shots: Vec::new(),
            runes: Vec::new(),
            bolts: Vec::new(),
            flashes: Vec::new(),
            spells: Default::default(),
            casting: None,
            stars: Vec::new(),
            zaps: Vec::new(),
            bought: Vec::new(),
            fx: Fx::default(),
            rng: Rng::new(seed ^ 0xC0FFEE),
            clock: Clock {
                day: 1,
                min: DAY_START + 60.0,
            },
            money: 150,
            deepest: 0,
            waystones: Vec::new(),
            shipping: Vec::new(),
            menu: Menu::None,
            held: None,
            cam: Camera::default(),
            cam_pos: Vec3::ZERO,
            shake: 0.0,
            fade: None,
            toasts: Vec::new(),
            banner: None,
            cat: Cat::new(Vec2::new(26.5, 11.5)),
            spider: None,
            hatching: None,
            rain: false,
            time: 0.0,
            target: None,
            target_ok: false,
            aim: None,
            hint: None,
            sel_name_t: 0.0,
            revealed: Vec::new(),
            show_map: true,
            boss_seen: None,
            stats: Stats::default(),
            known: Default::default(),
            discoveries: Default::default(),
            bag_seen: Vec::new(),
            pad: false,
            erupting: Vec::new(),
            bombs: Vec::new(),
            bomb_stock: super::bombs::BOMBS_PER_DAY,
            below: None,
            vault: None,
            quit_to_title: false,
            last_summary: None,
            saved_day: 0,
        };
        p.known = super::items::starter_recipes(&p.player.inv);
        p.bag_seen = p.player.inv.distinct();
        p.cam_pos = p.player.world_pos();
        p.banner = Some(Banner {
            title: "Hollowbloom Farm".into(),
            sub: "Day 1 - welcome home".into(),
            t: 0.0,
        });
        p.menu = Menu::dialog(
            "Welcome home! This little farm sits right on top of the Hollow, a cave that goes \
             down forever. Seeds from the deep grow up here, so delve, dig and plant.\n\
             Till with the hoe, plant, water every day and sleep to let things grow. The \
             Hollow is full of treasure: gear, coins and scrolls you can bind at the \
             enchanting table by the house.\n\
             Follow the path east to the bus stop: the bus goes to Bramblewick, where the \
             townsfolk have shops, stories and plenty they'd love a hand with (L: quests).",
        );
        p
    }

    /// The hero's luck, for loot rolls.
    pub fn fortune(&self) -> Fortune {
        Fortune {
            luck: self.player.luck(),
            greed: self.player.greed(),
        }
    }

    pub fn world(&self) -> &World {
        area_world(
            self.area,
            &self.farm,
            &self.town,
            &self.house.world,
            &self.level,
            &self.room,
        )
    }

    pub fn world_mut(&mut self) -> &mut World {
        area_world_mut(
            self.area,
            &mut self.farm,
            &mut self.town,
            &mut self.house.world,
            &mut self.level,
            &mut self.room,
        )
    }

    pub fn depth(&self) -> u32 {
        match self.area {
            Area::Hollow { depth } => depth,
            _ => 0,
        }
    }

    pub fn toast(&mut self, text: impl Into<String>, icon: Option<Item>, n: u32) {
        self.toast_colored(text, icon, n, CREAM);
    }

    pub fn toast_colored(
        &mut self,
        text: impl Into<String>,
        icon: Option<Item>,
        n: u32,
        color: u8,
    ) {
        let text = text.into();
        if let Some(t) = self
            .toasts
            .iter_mut()
            .find(|t| t.icon.is_some() && t.icon == icon && t.text == text && t.t < 2.0)
        {
            t.n += n;
            t.t = 0.0;
            return;
        }
        self.toasts.push(Toast {
            text,
            icon,
            n,
            t: 0.0,
            color,
        });
        if self.toasts.len() > 5 {
            self.toasts.remove(0);
        }
    }

    pub fn start_fade(&mut self, action: Trans) {
        if self.fade.is_none() {
            self.fade = Some(Fade {
                t: 0.0,
                action,
                fired: false,
            });
        }
    }

    // --------------------------------------------------------------------------------------
    // Environment
    // --------------------------------------------------------------------------------------

    /// Tonight's moon.
    pub fn moon(&self) -> super::sky::MoonPhase {
        super::sky::MoonPhase::of_day(self.clock.day)
    }

    /// Snow lies on the farm and in town all winter, and melts with the spring.
    pub fn sync_season(&mut self) {
        let winter = self.clock.season() == Season::Winter;
        self.farm.set_snow(winter);
        self.town.set_snow(winter);
    }

    pub fn env(&self) -> Env {
        match self.area {
            Area::Home => {
                // Daylight through the windows, lamps and the fire after dark.
                let night = self.sky_night();
                Env {
                    ambient: 0.84 - night * 0.3,
                    warmth: 4.8 + night * 1.2,
                    clear: INK,
                    time: self.time,
                    night: 1.0,
                    wind: 0.0,
                    push: self.player.pos,
                    spin: self.house_spin,
                    season: None,
                }
            }
            Area::Inside(_) => Env {
                ambient: 0.8,
                warmth: 4.6,
                clear: INK,
                time: self.time,
                night: 1.0,
                wind: 0.0,
                push: self.player.pos,
                spin: 0.0,
                season: None,
            },
            Area::Farm | Area::Town => {
                const KEYS: [(f32, f32, f32); 9] = [
                    (360.0, 0.72, 5.4),
                    (430.0, 1.0, 4.6),
                    (720.0, 1.06, 4.0),
                    (1020.0, 1.0, 4.4),
                    (1110.0, 0.92, 6.4),
                    (1180.0, 0.72, 6.0),
                    (1250.0, 0.46, 2.0),
                    (1320.0, 0.44, 1.3),
                    (1560.0, 0.42, 1.1),
                ];
                let m = self.clock.min;
                let mut amb = KEYS[0].1;
                let mut warm = KEYS[0].2;
                for w in KEYS.windows(2) {
                    let (a, b) = (w[0], w[1]);
                    if m >= a.0 && m <= b.0 {
                        let k = (m - a.0) / (b.0 - a.0);
                        amb = a.1 + (b.1 - a.1) * k;
                        warm = a.2 + (b.2 - a.2) * k;
                    }
                }
                if m > KEYS[8].0 {
                    amb = KEYS[8].1;
                    warm = KEYS[8].2;
                }
                if self.rain {
                    amb *= 0.84;
                    warm = (warm - 0.9).max(0.5);
                }
                if self.clock.season() == Season::Winter {
                    // Snow throws the moonlight back: crisp days, and pale blue nights.
                    amb = amb.max(0.6);
                    warm = (warm - 1.4).max(0.0);
                }
                let night = ((m - 1150.0) / 110.0).clamp(0.0, 1.0);
                Env {
                    ambient: amb,
                    warmth: warm,
                    clear: if night > 0.5 { INK } else { DEEP_TEAL },
                    time: self.time,
                    night,
                    // Blustery in the rain, calmer at night.
                    wind: if self.rain { 1.6 } else { 1.0 - night * 0.35 },
                    push: self.player.pos,
                    spin: 0.0,
                    season: Some(self.clock.season()),
                }
            }
            Area::Hollow { depth } => {
                let st = &BIOME_STYLES[self.hollow_biome(depth)];
                // The glowcap caves are lit cool, and the mushrooms' own colours glow in it;
                // the labyrinth's marble shows pale and warm.
                let glowing = self.level.as_ref().is_some_and(|l| l.world.glowcave);
                let marble = self.level.as_ref().is_some_and(|l| l.world.labyrinth);
                let canyon = self.level.as_ref().is_some_and(|l| l.world.canyon);
                let (ambient, warmth, clear) = if glowing {
                    (0.42, 2.0, INK)
                } else if marble {
                    (0.56, 4.8, INK)
                } else if canyon {
                    // Sun falling into it from far above: hot, and lit orange.
                    (0.68, 5.4, INK)
                } else {
                    (st.ambient, st.warmth, st.clear)
                };
                Env {
                    ambient,
                    warmth,
                    clear,
                    time: self.time,
                    night: 1.0,
                    wind: 0.0,
                    push: self.player.pos,
                    spin: 0.0,
                    season: None,
                }
            }
        }
    }

    pub fn music(&self) -> (Option<Song>, i32, f32) {
        let night = self.clock.min > 1170.0;
        match self.area {
            Area::Town if night => (Some(Song::Night), 0, 1.0),
            Area::Town => (Some(Song::Town), 0, 1.0),
            Area::Inside(_) => (Some(Song::Shop), 0, 1.0),
            Area::Home if night => (Some(Song::Night), 0, 0.92),
            Area::Home => (Some(Song::Haven), 0, 1.0),
            Area::Farm if night => (Some(Song::Night), 0, 1.0),
            Area::Farm if self.clock.min < 720.0 => (Some(Song::Morning), 0, 1.0),
            Area::Farm => (Some(Song::Afternoon), 0, 1.0),
            Area::Hollow { depth } => {
                if self.foes.iter().any(|f| f.boss && f.alert) {
                    return (Some(Song::Boss), 0, 1.0);
                }
                if is_waystone_floor(depth) && !self.foes.iter().any(|f| f.boss) {
                    return (Some(Song::Haven), 0, 1.0);
                }
                // Three themes, each biome of a pair in its own key and pace.
                match self.hollow_biome(depth) {
                    0 => (Some(Song::Burrows), 0, 1.0),
                    1 => (Some(Song::Glimmer), 0, 1.0),
                    2 => (Some(Song::Burrows), -2, 1.06),
                    3 => (Some(Song::Depths), 0, 1.06),
                    4 => (Some(Song::Glimmer), -2, 0.92),
                    _ => (Some(Song::Depths), -3, 0.94),
                }
            }
        }
    }

    // --------------------------------------------------------------------------------------
    // Travel
    // --------------------------------------------------------------------------------------

    fn enter_hollow(&mut self, depth: u32, via_waystone: bool) {
        self.forget_vault();
        let biome = self.biome_at(depth);
        let mut level = dungeon::generate(self.seed, depth, biome, via_waystone);
        // A guardian comes back to its floor while someone needs something it carries.
        let rematch = via_waystone && self.guardian_wanted(depth);
        if rematch {
            level.spawns.push(dungeon::Spawn {
                foe: dungeon::boss_for(depth, biome),
                x: level.lair.0,
                z: level.lair.1,
                boss: true,
            });
        }
        self.room = None;
        self.bus = None;
        self.folk.clear();
        self.foes.clear();
        self.drops.clear();
        self.shots.clear();
        self.runes.clear();
        self.bolts.clear();
        self.erupting.clear();
        let moon = self.moon();
        if moon.full() {
            // Under a full moon more of the Hollow's chests gleam.
            let w = &mut level.world;
            for z in 0..w.h {
                for x in 0..w.w {
                    if let Some(Obj::LootChest { opened: false, .. }) = w.obj(x, z) {
                        if self.rng.chance(0.15) {
                            w.set_obj(
                                x,
                                z,
                                Some(Obj::LootChest {
                                    opened: false,
                                    gleam: true,
                                }),
                            );
                        }
                    }
                }
            }
        }
        for (i, s) in level.spawns.iter().enumerate() {
            let mut f = Enemy::new(
                s.foe,
                s.x,
                s.z,
                depth,
                biome,
                s.boss,
                hash2(depth as i32, i as i32, self.seed as u32),
            );
            if level.world.sewer {
                f = f.in_the_sewers();
            }
            if level.world.glowcave {
                f = f.in_the_glowcaves();
            }
            f.feel_the_moon(moon);
            self.foes.push(f);
        }
        if moon.full() {
            // Under a full moon, werewolves prowl the Hollow, well away from the way in.
            let n = dungeon::werewolves(depth);
            let mut rng = Rng::new(self.seed ^ (depth as u64 * 0x57A8) ^ self.clock.day as u64);
            for (k, (x, z)) in dungeon::prowls(&level, n, &mut rng).into_iter().enumerate() {
                let seed = hash2(depth as i32, 900 + k as i32, self.seed as u32);
                let mut f = Enemy::new(dungeon::Foe::Werewolf, x, z, depth, biome, false, seed);
                if level.world.sewer {
                    f = f.in_the_sewers();
                }
                f.feel_the_moon(moon);
                self.foes.push(f);
            }
        }
        self.player.pos = Vec2::new(level.start.0 as f32 + 0.5, level.start.1 as f32 + 0.5);
        self.player.act = None;
        self.revealed = vec![false; (level.world.w * level.world.h) as usize];
        let (sewer, glowcave, labyrinth, canyon) = (
            level.world.sewer,
            level.world.glowcave,
            level.world.labyrinth,
            level.world.canyon,
        );
        self.level = Some(level);
        self.area = Area::Hollow { depth };
        self.cam_pos = self.player.world_pos();
        if depth > self.deepest {
            self.deepest = depth;
        }
        self.stats.floors += 1;
        self.boss_seen = None;
        self.banner = Some(Banner {
            title: format!("Floor {depth}"),
            sub: if rematch {
                "The guardian has returned!".to_string()
            } else if moon.full() {
                format!(
                    "{} - full moon: the Hollow is wild!",
                    BIOME_STYLES[biome].name
                )
            } else if sewer {
                format!("{} - the old sewers", BIOME_STYLES[biome].name)
            } else if glowcave {
                format!("{} - the glowcap caves", BIOME_STYLES[biome].name)
            } else if labyrinth {
                format!("{} - the marble labyrinth", BIOME_STYLES[biome].name)
            } else if canyon {
                format!("{} - the sunscorch canyon", BIOME_STYLES[biome].name)
            } else {
                BIOME_STYLES[biome].name.to_string()
            },
            t: 0.0,
        });
        self.hide_keepsakes(depth);
    }

    /// Puts the keepsakes someone asked you to find somewhere out of the way on this floor.
    fn hide_keepsakes(&mut self, depth: u32) {
        let items = self.keepsakes_here(depth);
        if items.is_empty() {
            return;
        }
        let Some(level) = &self.level else { return };
        let w = &level.world;
        let (sx, sz) = level.start;
        let mut rng = Rng::new(self.seed ^ (depth as u64 * 7717) ^ self.clock.day as u64);
        let mut spots = Vec::new();
        for z in 1..w.h - 1 {
            for x in 1..w.w - 1 {
                let far = (x - sx).abs() + (z - sz).abs() > 14;
                if far && !w.blocked(x, z) && w.obj(x, z).is_none() {
                    spots.push((x, z));
                }
            }
        }
        for item in items {
            if spots.is_empty() {
                break;
            }
            let (x, z) = spots.swap_remove(rng.below(spots.len()));
            let mut d = Drop::new(Stack::new(item, 1), tile_center(x, z), &mut self.rng);
            d.vel = Vec3::ZERO;
            d.pos = tile_center(x, z) + Vec3::Y * 0.3;
            self.drops.push(d);
        }
    }

    fn go_home(&mut self) {
        self.forget_vault();
        self.level = None;
        self.room = None;
        self.bus = None;
        self.foes.clear();
        self.drops.clear();
        self.shots.clear();
        self.runes.clear();
        self.bolts.clear();
        self.area = Area::Farm;
        let (x, z) = MARKS.hollow;
        self.player.pos = Vec2::new(x as f32 + 0.5, z as f32 + 1.6);
        self.player.facing = Vec2::new(0.0, 1.0);
        self.player.act = None;
        self.cam_pos = self.player.world_pos();
        self.banner = Some(Banner {
            title: "Home Sweet Farm".into(),
            sub: format!("Deepest floor: {}", self.deepest),
            t: 0.0,
        });
    }

    /// Ends the day: growth, sales, rest.
    fn sleep(&mut self, passed_out: bool) {
        let day = self.clock.day;
        let mut earned = 0u64;
        for s in self.shipping.drain(..) {
            earned += s.value();
        }
        self.money += earned;
        self.bought.clear();
        self.bomb_stock = super::bombs::BOMBS_PER_DAY;
        self.stats.earned += earned;
        self.on_ship(earned);
        self.friends.new_day();
        let today = (day + 1) * 16;
        self.taken.retain(|id| *id >= today);
        let next = Clock {
            day: day + 1,
            min: DAY_START,
        };
        let turned = next.season() != self.clock.season();
        let night = farm::new_day(&mut self.farm, day + 1, false, next.season().bit(), turned);
        self.rain = Rng::new(self.seed ^ ((day as u64 + 1) * 31)).chance(0.18);
        if self.rain {
            // Rain waters everything for the new day.
            let (w, h) = (self.farm.w, self.farm.h);
            for z in 0..h {
                for x in 0..w {
                    if self.farm.floor(x, z) == Floor::Tilled {
                        self.farm.set_flag(x, z, WATERED, true);
                    }
                }
            }
        }
        self.clock.day += 1;
        self.clock.min = DAY_START;
        let p = &mut self.player;
        p.buffs.clear();
        p.refresh();
        p.energy = if passed_out {
            p.max_energy() as f32 * 0.6
        } else {
            p.max_energy() as f32
        };
        p.hp = p.max_hp();
        p.mana = p.max_mana() as f32;
        p.water = p.can_capacity();
        p.facing = Vec2::new(0.0, 1.0);
        p.act = None;
        self.player.pos = self.bedside();
        self.forget_vault();
        self.level = None;
        self.room = None;
        self.bus = None;
        self.drop_rod();
        self.cooking = None;
        self.foes.clear();
        self.shots.clear();
        self.runes.clear();
        self.bolts.clear();
        self.stars.clear();
        self.zaps.clear();
        self.drops.clear();
        p_ward_off(&mut self.player);
        self.area = Area::Home;
        self.cam_pos = self.player.world_pos();
        let mut notes = Vec::new();
        if self.clock.season_day() == 1 {
            let s = self.clock.season();
            notes.push((
                format!("{} is here - {}", s.name(), s.greeting()),
                s.color(),
            ));
        }
        if night.withered > 0 {
            notes.push((
                format!(
                    "{} out-of-season crop{} withered overnight.",
                    night.withered,
                    if night.withered == 1 { "" } else { "s" }
                ),
                ROSEWOOD,
            ));
        }
        // The last day of a season: warn about anything that won't live through the night.
        if self.clock.season_day() == SEASON_DAYS {
            let next = Clock {
                day: self.clock.day + 1,
                min: DAY_START,
            }
            .season();
            let doomed = self
                .farm
                .objs
                .iter()
                .flatten()
                .filter(|o| matches!(o, Obj::Crop { crop, .. } if !crop.grows_in(next.bit())))
                .count();
            if doomed > 0 {
                notes.push((
                    format!(
                        "Last day of {}! {doomed} crop{} won't survive into {}.",
                        self.clock.season().name(),
                        if doomed == 1 { "" } else { "s" },
                        next.name()
                    ),
                    CRIMSON,
                ));
            }
        }
        notes.extend(self.egg_news());
        self.last_summary = Some(super::menus::Summary {
            day: self.clock.day,
            earned,
            grown: night.grown,
            ready: night.ready,
            sprouted: night.sprouted,
            rain: self.rain,
            passed_out,
            fainted: false,
            notes,
        });
        self.menu = Menu::Summary;
    }

    fn faint(&mut self) {
        let lost = (self.money / 10).min(1000);
        self.money -= lost;
        self.sleep(true);
        if let Some(s) = &mut self.last_summary {
            s.fainted = true;
        }
        self.toast(
            format!("Lost {} while you were out", loot::money_text(lost)),
            None,
            0,
        );
        self.player.hp = self.player.max_hp() / 2;
    }

    // --------------------------------------------------------------------------------------
    // Update
    // --------------------------------------------------------------------------------------

    pub fn update(&mut self, io: &mut Io, settings: &mut Settings) {
        let dt = io.dt;
        self.time += dt;
        self.pad = io.input.pad_active;
        self.update_camera(io.view, dt);
        self.sync_season();

        // Fades run even over menus.
        if let Some(f) = &mut self.fade {
            f.t += dt * 2.4;
            if f.t >= 1.0 && !f.fired {
                f.fired = true;
                let action = f.action;
                match action {
                    Trans::Descend {
                        depth,
                        via_waystone,
                    } => self.enter_hollow(depth, via_waystone),
                    Trans::Home => self.go_home(),
                    Trans::Sleep { passed_out } => self.sleep(passed_out),
                    Trans::Faint => self.faint(),
                    Trans::Bus { to_town } => self.ride_bus(to_town),
                    Trans::Enter(place) => self.enter_place(place),
                    Trans::Leave => self.leave_place(),
                    Trans::House { enter: true } => self.enter_house(),
                    Trans::House { enter: false } => self.leave_house(),
                    Trans::Vault { enter: true } => self.enter_vault(),
                    Trans::Vault { enter: false } => self.leave_vault(),
                }
                if !matches!(action, Trans::Enter(_) | Trans::Leave | Trans::House { .. }) {
                    io.audio.play(Sfx::Stairs);
                }
            }
            if let Some(f) = &self.fade {
                if f.t >= 2.0 {
                    self.fade = None;
                }
            }
        }
        let (song, tr, tempo) = if matches!(self.menu, Menu::Summary) {
            (Some(Song::Title), 0, 1.0)
        } else {
            self.music()
        };
        io.audio.music(song, tr, tempo);

        for t in &mut self.toasts {
            t.t += dt;
        }
        self.toasts.retain(|t| t.t < 3.2);
        if let Some(b) = &mut self.banner {
            b.t += dt;
            if b.t > 3.5 {
                self.banner = None;
            }
        }
        self.sel_name_t += dt;
        self.nag = (self.nag - dt).max(0.0);

        self.update_bus(io);
        self.closing_time(io);
        // A newly learned recipe stops everything to show itself off.
        if matches!(self.menu, Menu::None) && self.fade.is_none() && self.cooking.is_none() {
            self.discover_recipes();
            if let Some(recipe) = self.discoveries.pop_front() {
                self.menu = Menu::Recipe { recipe, t: 0.0 };
                io.audio.play(Sfx::Discover);
            }
        }
        if !matches!(self.menu, Menu::None) {
            self.update_menu(io, settings);
            self.update_folk(dt, false);
            self.fx.update(dt);
            return;
        }
        if self.fade.is_some() || self.bus.as_ref().is_some_and(|b| b.boarding) {
            self.update_folk(dt, false);
            self.fx.update(dt);
            return;
        }
        let input = io.input;
        if input.pressed(Action::Menu) {
            self.menu = Menu::pause();
            io.audio.play(Sfx::UiSelect);
            return;
        }
        if input.pressed(Action::Inventory) {
            self.menu = Menu::inventory(false);
            io.audio.play(Sfx::UiSelect);
            return;
        }
        if input.pressed(Action::Crafting) {
            self.menu = Menu::inventory(true);
            io.audio.play(Sfx::UiSelect);
            return;
        }
        if input.pressed(Action::Map) {
            self.show_map = !self.show_map;
        }
        if input.pressed(Action::Quests) {
            self.menu = Menu::Journal { tab: 0, sel: 0 };
            io.audio.play(Sfx::UiSelect);
            return;
        }

        // Time, and the town clock striking the hours once it's mended.
        self.clock.min += dt * MIN_PER_SEC;
        let hour = (self.clock.min / 60.0) as i32;
        if hour != self.last_hour {
            if self.last_hour >= 0 && self.in_town() && self.restored & super::town::CLOCK != 0 {
                io.audio.play_at(Sfx::Bell, 0.5, 1.0);
            }
            // Your own grandfather clock chimes too.
            let clock = self
                .house
                .pieces()
                .iter()
                .any(|p| p.2 == super::home::Furn::Clock);
            if self.last_hour >= 0 && self.area == Area::Home && clock {
                io.audio.play_at(Sfx::Bell, 0.3, 1.3);
            }
            self.last_hour = hour;
        }
        self.house_spin = (self.house_spin - dt).max(0.0);
        if self.clock.min >= DAY_END {
            self.clock.min = DAY_END;
            self.toast("You're exhausted... you collapse.", None, 0);
            self.start_fade(Trans::Sleep { passed_out: true });
        }

        // Hotbar.
        if let Some(n) = input.number_pressed() {
            self.select(n, io);
        }
        if input.wheel != 0.0 {
            let dir = if input.wheel > 0.0 { HOTBAR - 1 } else { 1 };
            let n = (self.player.sel + dir) % HOTBAR;
            self.select(n, io);
        }
        // ] and [ (R1 and L1 on a controller) step along the hotbar; Q and R cast your two
        // spells.
        let step = input.hotbar_step();
        if step != 0 {
            let n = (self.player.sel as i32 + step).rem_euclid(HOTBAR as i32) as usize;
            self.select(n, io);
        }
        for (slot, action) in [Action::Spell1, Action::Spell2].into_iter().enumerate() {
            if input.pressed(action) {
                self.begin_cast(slot, io);
            }
        }

        self.player.refresh();
        if self.cooking.is_some() {
            self.update_cooking(io);
        } else {
            self.update_player(io);
        }
        self.update_doors(io);
        self.update_folk(dt, true);
        self.update_foes(io);
        self.update_runes(dt);
        self.update_statuses(dt, io);
        self.update_bolts(dt, io);
        self.update_spells(dt, io);
        self.update_drops(io);
        self.update_bombs(dt, io);
        self.update_eruptions(dt, io);
        self.update_cat(dt);
        self.update_spider(dt);
        self.update_egg(dt, io);
        self.update_vitals(dt);
        self.fx.update(dt);
        self.shake = (self.shake - dt * 2.5).max(0.0);
        if let Area::Hollow { .. } = self.area {
            self.reveal();
        }
        if self.player.hp <= 0 && self.fade.is_none() {
            self.player.hp = 0;
            io.audio.play(Sfx::PlayerHurt);
            self.toast("You fainted!", None, 0);
            self.start_fade(Trans::Faint);
        }
    }

    /// Five o'clock: the tills shut, and shopkeepers see you out of their shops.
    fn closing_time(&mut self, io: &mut Io) {
        if town::trading(self.clock.min) || self.fade.is_some() {
            return;
        }
        if matches!(self.menu, Menu::Shop { .. }) {
            self.close_menu(io);
            self.toast("Closing time! Shops trade from 9am to 5pm.", None, 0);
        }
        if let Area::Inside(place) = self.area {
            if place.is_store() {
                if !matches!(self.menu, Menu::None) {
                    self.close_menu(io);
                }
                self.toast(
                    format!("{} is closing - see you at 9am!", place.def().name),
                    None,
                    0,
                );
                self.start_fade(Trans::Leave);
            }
        }
    }

    /// The key or button for an action, for on-screen hints: "E" on the keyboard, "A" on
    /// the Steam Deck.
    pub fn key(&self, a: Action) -> &'static str {
        if self.pad {
            crate::input::pad_name(a)
        } else {
            crate::input::key_name(a)
        }
    }

    /// The same in brackets: "(E)", "(A)".
    pub fn prompt(&self, a: Action) -> String {
        format!("({})", self.key(a))
    }

    /// Holding a piece of furniture at home (B turns it rather than rolling).
    fn holding_furniture(&self) -> bool {
        self.area == Area::Home
            && self
                .player
                .held()
                .is_some_and(|i| matches!(i.def().kind, Kind::Place(Placeable::Furniture(_))))
    }

    /// Learns every recipe whose ingredients are all in the bag, one of each being enough,
    /// and queues a card for each to show. Only looks again when what's in the bag changes.
    pub fn discover_recipes(&mut self) {
        let bag = self.player.inv.distinct();
        if bag == self.bag_seen {
            return;
        }
        for (k, r) in super::items::RECIPES.iter().enumerate() {
            if !self.known.contains(&r.out) && r.hinted_by(&bag) {
                self.known.insert(r.out);
                self.discoveries.push_back(k);
            }
        }
        self.bag_seen = bag;
    }

    /// Regeneration, mana, food buffs and magic flashes.
    fn update_vitals(&mut self, dt: f32) {
        let p = &mut self.player;
        // Health: the Regen stat every five seconds, heartsip, and a slow trickle at home.
        let mut rate = p.stat(Stat::Regen).max(0) as f32 / 5.0;
        if !matches!(self.area, Area::Hollow { .. }) {
            rate += 1.0;
        }
        p.regen_acc += rate * dt;
        if p.regen_acc >= 1.0 {
            let heal = p.regen_acc.floor();
            p.regen_acc -= heal;
            if p.hp > 0 {
                p.hp = (p.hp + heal as i32).min(p.max_hp());
            }
        }
        let spirit = p.stat(Stat::Spirit).max(0) as f32;
        p.mana = (p.mana + (2.5 + spirit * 0.3) * dt).min(p.max_mana() as f32);
        p.no_mana_t = (p.no_mana_t - dt).max(0.0);
        let mut ended = Vec::new();
        for b in &mut p.buffs {
            b.left -= dt;
            if b.left <= 0.0 {
                ended.push(b.from);
            }
        }
        p.buffs.retain(|b| b.left > 0.0);
        if !ended.is_empty() {
            p.refresh();
        }
        for item in ended {
            self.toast(format!("{} wore off", item.def().name), None, 0);
        }
        for f in &mut self.flashes {
            f.t += dt;
        }
        self.flashes.retain(|f| f.t < 0.35);
    }

    fn select(&mut self, n: usize, io: &Io) {
        if n != self.player.sel {
            self.player.sel = n;
            self.sel_name_t = 0.0;
            io.audio.play_at(Sfx::UiMove, 0.6, 1.0);
        }
    }

    fn update_camera(&mut self, view: (usize, usize), dt: f32) {
        /// How close the camera comes to show off a catch.
        const CATCH_DIST: f32 = 8.5;
        let p = self.player.world_pos()
            + Vec3::new(self.player.facing.x * 0.6, 0.0, self.player.facing.y * 0.4);
        let k = damp(7.0, dt);
        self.cam_pos += (p - self.cam_pos) * k;
        let mut t = self.cam_pos;
        match self.area {
            Area::Farm | Area::Town => {
                let w = self.world();
                let (w, h) = (w.w as f32, w.h as f32);
                t.x = t.x.clamp(10.5, w - 10.5);
                t.z = t.z.clamp(7.0, h - 3.5);
            }
            Area::Inside(_) | Area::Home => {
                // Rooms are small dioramas: the camera stands back and leans a little
                // towards you.
                let c = self.room_center();
                t = c + (self.player.world_pos() - c) * Vec3::new(0.15, 0.0, 0.08);
            }
            _ => {}
        }
        let far = if matches!(self.area, Area::Inside(_) | Area::Home) {
            17.5
        } else {
            15.5
        };
        // Showing off a catch, the camera leans in on the hero and the fish held up (and
        // eases back out once it's put away).
        let shown = self.fishing.as_ref().and_then(|f| f.showing()).is_some();
        let rate = if shown { 2.2 } else { 2.8 };
        let want = if shown { 1.0 } else { 0.0 };
        self.catch_zoom = approach(self.catch_zoom, want, dt * rate);
        let z = self.catch_zoom * self.catch_zoom * (3.0 - 2.0 * self.catch_zoom);
        self.cam.dist = far - (far - CATCH_DIST) * z;
        if z > 0.0 {
            let p = &self.player;
            let lift = self.fishing.as_ref().map_or(0.7, |f| f.lift(self.time));
            let held = super::fish::rod_tip(p.world_pos(), p.yaw, lift) - Vec3::Y * 0.37;
            let focus = (p.world_pos() + Vec3::Y * 0.7).lerp(held, 0.5);
            t = t.lerp(focus, z);
        }
        if self.shake > 0.0 {
            let s = self.shake * self.shake * 0.25;
            t.x += (self.time * 71.0).sin() * s;
            t.z += (self.time * 53.0).cos() * s;
        }
        self.cam.target = t;
        self.cam.update(view.0.max(1), view.1.max(1));
    }

    fn update_player(&mut self, io: &mut Io) {
        let dt = io.dt;
        let input = io.input;
        let p = &mut self.player;
        p.hurt = (p.hurt - dt).max(0.0);
        p.flash = (p.flash - dt).max(0.0);
        p.dodge_cd = (p.dodge_cd - dt).max(0.0);
        p.eat_t = (p.eat_t - dt).max(0.0);
        let mut mv = input.move_axis();
        let busy = p.act.is_some();
        let still = self.fishing.as_ref().is_some_and(|f| f.holds_still());
        if still {
            mv = Vec2::ZERO;
        }
        // Dodge roll (at home, holding furniture, that button turns it instead).
        let turning = self.holding_furniture();
        let p = &mut self.player;
        if input.pressed(Action::Dodge) && p.dodge_cd <= 0.0 && p.dodge <= 0.0 && !still && !turning
        {
            self.fishing = None;
            let p = &mut self.player;
            let dir = if mv.length_squared() > 0.0 {
                mv
            } else {
                p.facing
            };
            p.dodge = 0.26;
            p.dodge_cd = 0.6;
            p.dodge_dir = dir.normalize_or_zero();
            p.act = None;
            io.audio.play(Sfx::Dodge);
            self.fx
                .burst(p.world_pos() + Vec3::Y * 0.05, 6, &[SAND, WHITE], 1.5, 0.8);
        }
        let world = area_world(
            self.area,
            &self.farm,
            &self.town,
            &self.house.world,
            &self.level,
            &self.room,
        );
        let p = &mut self.player;
        if p.dodge > 0.0 {
            p.dodge -= dt;
            let d = p.dodge_dir * 9.0 * dt;
            p.pos = world.move_circle(p.pos, d, RADIUS);
            mv = Vec2::ZERO;
        } else if busy {
            mv *= 0.25;
        }
        let tired = p.energy <= 0.0;
        // Quicksand drags at your feet.
        let (tx, tz) = (p.pos.x.floor() as i32, p.pos.y.floor() as i32);
        let sinking = world.floor(tx, tz) == Floor::Quicksand;
        let speed = p.move_speed()
            * if tired { 0.65 } else { 1.0 }
            * if sinking { QUICKSAND_DRAG } else { 1.0 };
        if mv.length_squared() > 0.0 {
            p.pos = world.move_circle(p.pos, mv * speed * dt, RADIUS);
            if !busy {
                p.facing = mv.normalize();
            }
            p.walk += dt * 11.0;
            p.stride = (p.stride + dt * 6.0).min(1.0);
            p.step_t -= dt;
            if p.step_t <= 0.0 {
                p.step_t = 0.3;
                io.audio.play_at(Sfx::Step, 0.5, 0.9 + (p.walk % 0.3));
            }
        } else {
            p.stride = (p.stride - dt * 6.0).max(0.0);
        }
        if p.vel.length_squared() > 0.001 {
            p.pos = world.move_circle(p.pos, p.vel * dt, RADIUS);
            p.vel *= (1.0 - dt * 9.0).max(0.0);
        }
        // Face the mouse when aiming with it.
        if input.mouse_aim {
            if let Some(g) = self.cam.ground(input.mouse.x, input.mouse.y, 0.0) {
                let d = Vec2::new(g.x, g.z) - p.pos;
                if d.length_squared() > 0.04
                    && (busy || mv.length_squared() == 0.0 || input.down(Action::Use))
                {
                    p.facing = d.normalize();
                }
            }
        }
        // Or with the right stick.
        if let Some(d) = input.aim_axis() {
            p.facing = d;
        }
        let want = p.facing.x.atan2(p.facing.y);
        p.yaw += wrap_angle(want - p.yaw) * damp(18.0, dt);

        // Current action.
        if let Some(act) = &mut p.act {
            act.t += dt;
            let prog = act.progress();
            let fire = !act.fired && prog >= act.kind.impact();
            if fire {
                act.fired = true;
            }
            let done = prog >= 1.0;
            let (kind, tile, dir) = (act.kind, act.tile, act.dir);
            if done {
                p.act = None;
            }
            if fire {
                self.fire(kind, tile, dir, io);
            }
        }
        // Aim at the tile in front (or under the mouse) before acting on it.
        self.update_target(io);
        // The rod has a rhythm of its own: winding up, waiting, reeling.
        if self.update_fishing(io) {
            return;
        }
        // Turning the next piece of furniture before it goes down.
        if self.area == Area::Home
            && (input.key_pressed(KeyCode::KeyT)
                || (self.holding_furniture() && input.pressed(Action::Turn)))
        {
            self.house.turn = (self.house.turn + 1) % 4;
            io.audio.play_at(Sfx::UiMove, 0.6, 1.2);
        }
        // Start actions.
        let p = &self.player;
        if p.act.is_none() && p.dodge <= 0.0 {
            if input.pressed(Action::Interact) {
                self.interact(io);
            } else if input.down(Action::Use) && (input.pressed(Action::Use) || self.held_repeats())
            {
                self.use_held(io);
            }
        }
    }

    fn held_repeats(&self) -> bool {
        self.player
            .held_class()
            .is_some_and(|c| ActKind::for_class(c).is_some())
    }

    /// The tile the player is working on: the one under the mouse when it is close,
    /// otherwise the one in front.
    fn update_target(&mut self, io: &Io) {
        let input = io.input;
        let p = &self.player;
        let (px, pz) = p.tile();
        let front = {
            let f = p.facing;
            if f.x.abs() > f.y.abs() {
                (px + f.x.signum() as i32, pz)
            } else {
                (px, pz + f.y.signum() as i32)
            }
        };
        let mut t = front;
        self.aim = None;
        if input.mouse_aim {
            if let Some(g) = self.cam.ground(input.mouse.x, input.mouse.y, 0.0) {
                self.aim = Some(Vec2::new(g.x, g.z));
                let (mx, mz) = (g.x.floor() as i32, g.z.floor() as i32);
                if (mx - px).abs() <= 1 && (mz - pz).abs() <= 1 {
                    t = (mx, mz);
                } else {
                    let d = Vec2::new(g.x, g.z) - p.pos;
                    let a = d.y.atan2(d.x);
                    let oct = ((a / (std::f32::consts::PI / 4.0)).round() as i32).rem_euclid(8);
                    let (dx, dz) = [
                        (1, 0),
                        (1, 1),
                        (0, 1),
                        (-1, 1),
                        (-1, 0),
                        (-1, -1),
                        (0, -1),
                        (1, -1),
                    ][oct as usize];
                    t = (px + dx, pz + dz);
                }
            }
        }
        self.target = Some(t);
        self.target_ok = self.can_act_on(t);
        self.hint = self.interact_hint(t);
    }

    fn can_act_on(&self, (x, z): (i32, i32)) -> bool {
        if self.area == Area::Home {
            return self.house_act_ok(x, z);
        }
        if self.in_town() {
            return false;
        }
        let w = self.world();
        let Some(item) = self.player.held() else {
            return false;
        };
        match item.def().kind {
            Kind::Gear(b) => match b.class {
                Class::Hoe => self.area == Area::Farm && tillable(w, x, z),
                Class::Can => w.floor(x, z) == Floor::Tilled || w.floor(x, z) == Floor::Water,
                Class::Sickle => w.obj(x, z).is_some_and(|o| match o {
                    Obj::Weed { .. }
                    | Obj::Flower { .. }
                    | Obj::Shrub { .. }
                    | Obj::Glowcap { .. }
                    | Obj::ShelfFungus { .. } => true,
                    Obj::Crop { crop, days, .. } => crop.stage(*days) == 3,
                    _ => false,
                }),
                Class::Pickaxe => {
                    matches!(
                        w.wall(x, z),
                        Wall::Rock | Wall::Ore(_) | Wall::Brick | Wall::Timber | Wall::Sewer
                    ) || w.obj(x, z).is_some_and(|o| {
                        matches!(
                            o,
                            Obj::Rock { .. }
                                | Obj::Boulder { .. }
                                | Obj::Crystal { .. }
                                | Obj::CandyRock { .. }
                                | Obj::Stalagmite { .. }
                                | Obj::Lamp
                                | Obj::Sprinkler { .. }
                                | Obj::FlowerPot { .. }
                                | Obj::Torch
                                | Obj::Chest { .. }
                                | Obj::Pot { .. }
                                | Obj::Crate { .. }
                                | Obj::Keg { .. }
                                | Obj::EnchantTable
                        )
                    }) || matches!(w.floor(x, z), Floor::Planks | Floor::Cobble)
                }
                Class::Axe => {
                    w.obj(x, z).is_some_and(|o| {
                        matches!(
                            o,
                            Obj::Tree { .. }
                                | Obj::Pine { .. }
                                | Obj::Stump { .. }
                                | Obj::Log { .. }
                                | Obj::Fence
                                | Obj::Bench
                                | Obj::Workbench
                                | Obj::Chest { .. }
                                | Obj::Crate { .. }
                                | Obj::Keg { .. }
                                | Obj::Weed { .. }
                                | Obj::Shrub { .. }
                        )
                    }) || w.wall(x, z) == Wall::Timber
                        // Broken planks on a sewer walkway (not the ones out in the water).
                        || (matches!(w.obj(x, z), Some(Obj::Debris { .. }))
                            && w.floor(x, z) != Floor::Water)
                }
                _ => false,
            },
            Kind::Seed(crop) => {
                self.area == Area::Farm
                    && w.floor(x, z) == Floor::Tilled
                    && w.obj(x, z).is_none()
                    && crop.grows_in(self.clock.season().bit())
            }
            Kind::Place(pl) => self.can_place(pl, x, z),
            _ => false,
        }
    }

    fn can_place(&self, pl: Placeable, x: i32, z: i32) -> bool {
        if self.area == Area::Home {
            return self.house_target_ok(pl, x, z);
        }
        if matches!(
            pl,
            Placeable::Furniture(_) | Placeable::Rug(_) | Placeable::WallArt(_)
        ) {
            return false;
        }
        let w = self.world();
        if !w.inside(x, z) || w.wall(x, z) != Wall::None {
            return false;
        }
        let floor = w.floor(x, z);
        if matches!(floor, Floor::Water | Floor::Lava | Floor::Void) {
            return false;
        }
        let in_hollow = self.area != Area::Farm;
        match pl {
            Placeable::WoodPath | Placeable::StonePath => {
                w.obj(x, z).is_none()
                    && !matches!(floor, Floor::Tilled | Floor::Planks | Floor::Cobble)
            }
            _ => {
                if w.obj(x, z).is_some() {
                    return false;
                }
                if in_hollow
                    && !matches!(
                        pl,
                        Placeable::Torch | Placeable::StoneWall | Placeable::WoodWall
                    )
                {
                    return false;
                }
                // Keep solid things off the player.
                let solid = !matches!(pl, Placeable::Torch);
                if solid {
                    let c = Vec2::new(x as f32 + 0.5, z as f32 + 0.5);
                    let d = (self.player.pos - c).abs();
                    if d.x < 0.5 + RADIUS && d.y < 0.5 + RADIUS {
                        return false;
                    }
                }
                true
            }
        }
    }

    fn interact_hint(&self, (x, z): (i32, i32)) -> Option<String> {
        if let Some(i) = self.npc_near() {
            let n = &self.folk[i];
            let v = n.who;
            let keeper = n.fixed && v.def().keeps.is_some_and(|p| p != super::town::Place::Hall);
            return Some(if keeper {
                format!("Talk to {} / shop", v.def().name)
            } else {
                super::talk::near_text(v)
            });
        }
        let w = self.world();
        let (ax, az) = w.anchor(x, z);
        let o = w.obj(ax, az)?;
        let s = match o {
            Obj::Building { id } => {
                let b = &BUILDINGS[*id as usize];
                return Some(match b.place {
                    Some(p) => format!("Enter {}", p.def().name),
                    None => format!("Knock at {}", b.name),
                });
            }
            Obj::BusStop => "Wait for the bus",
            Obj::Board => "Read the notices",
            Obj::Fountain { .. } => "Make a wish",
            Obj::WishTree { .. } => "Touch the Wishing Tree",
            Obj::Well => "Peek in",
            Obj::House => "Go inside",
            // Plain decor has nothing to do with E (J picks it up; the HUD says so).
            Obj::Furniture { f, .. } if f.def().use_ == super::home::Use::Nothing => return None,
            Obj::Furniture { f, .. } if self.area == Area::Home => super::home::use_hint(*f),
            Obj::Furniture { .. } => "Admire",
            Obj::Bin => "Ship items",
            Obj::Hollow => "Enter the Hollow",
            Obj::Stall => "Shop",
            Obj::Chest { .. } => "Open chest",
            Obj::LootChest { opened: false, .. } => "Open",
            Obj::Hole => "Climb down",
            Obj::Rope => "Climb up",
            Obj::Crack => "Look",
            Obj::Sign { .. } => "Read",
            Obj::StairsDown => "Descend",
            Obj::Waystone => "Touch the waystone",
            Obj::Workbench => "Craft",
            Obj::EnchantTable => "Enchant",
            Obj::Crop { crop, days, .. } if crop.stage(*days) == 3 => "Harvest",
            Obj::Bench => "Sit",
            _ => return None,
        };
        Some(s.to_string())
    }

    // --------------------------------------------------------------------------------------
    // Using things
    // --------------------------------------------------------------------------------------

    fn use_held(&mut self, io: &mut Io) {
        let tile = self.target.unwrap_or(self.player.tile());
        if self.area == Area::Home {
            // Empty hands can still pick things up about the house.
            self.house_use(tile, true, io);
            return;
        }
        let Some(item) = self.player.held() else {
            return;
        };
        let dir = self.player.facing;
        let kind = item.class().and_then(ActKind::for_class);
        let Some(kind) = kind else {
            self.use_item(item, tile, io);
            return;
        };
        if self.in_town() {
            if self.nag <= 0.0 {
                self.nag = 2.5;
                self.toast("Better keep that put away in town.", None, 0);
                io.audio.play(Sfx::Denied);
            }
            return;
        }
        let p = &mut self.player;
        match kind {
            ActKind::Bolt | ActKind::Blast => {
                let cost = p.mana_cost(if kind == ActKind::Bolt { 5.0 } else { 12.0 });
                if p.mana < cost {
                    if p.no_mana_t <= 0.0 {
                        p.no_mana_t = 1.2;
                        self.toast("Not enough mana.", None, 0);
                        io.audio.play(Sfx::Denied);
                    }
                    return;
                }
                p.mana -= cost;
            }
            ActKind::Slash => {}
            _ => {
                let base = match kind {
                    ActKind::Water => 1.4,
                    ActKind::Reap => 1.2,
                    _ => 2.0,
                };
                let cost = p.tool_cost(base);
                p.energy = (p.energy - cost).max(-10.0);
            }
        }
        let dur = kind.base_duration() * p.haste();
        p.act = Some(Act::new(kind, dur, tile, dir));
        match kind {
            ActKind::Slash | ActKind::Reap => io.audio.play(Sfx::Swing),
            ActKind::Bolt | ActKind::Blast => io.audio.play_at(Sfx::Swing, 0.5, 1.4),
            _ => {}
        }
    }

    /// Non-tool items: plant, place, eat, special.
    pub(crate) fn use_item(&mut self, item: Item, (x, z): (i32, i32), io: &mut Io) {
        let sel = self.player.sel;
        match item.def().kind {
            Kind::Seed(crop) => {
                if self.area != Area::Farm {
                    self.toast("Seeds need farm soil.", None, 0);
                    io.audio.play(Sfx::Denied);
                } else if !crop.grows_in(self.clock.season().bit()) {
                    if self.nag <= 0.0 {
                        self.nag = 2.0;
                        let name = crop.def().produce.def().name;
                        self.toast(
                            format!("{name} only grows in {}.", crop.seasons_text()),
                            Some(item),
                            0,
                        );
                    }
                    io.audio.play(Sfx::Denied);
                } else if self.world().floor(x, z) == Floor::Tilled
                    && self.world().obj(x, z).is_none()
                {
                    self.player.inv.take_one(sel);
                    self.farm.set_obj(
                        x,
                        z,
                        Some(Obj::Crop {
                            crop,
                            days: 0,
                            harvested: false,
                        }),
                    );
                    io.audio.play(Sfx::Plant);
                    self.fx
                        .burst(tile_center(x, z), 5, &[RUST, CLAY, GREEN], 1.2, 1.5);
                } else {
                    io.audio.play(Sfx::Denied);
                }
            }
            Kind::Place(pl) => {
                if matches!(
                    pl,
                    Placeable::Furniture(_) | Placeable::Rug(_) | Placeable::WallArt(_)
                ) && self.area != Area::Home
                {
                    self.toast("That goes inside your house.", None, 0);
                    io.audio.play(Sfx::Denied);
                    return;
                }
                if !self.can_place(pl, x, z) {
                    io.audio.play(Sfx::Denied);
                    if self.area != Area::Farm
                        && !matches!(
                            pl,
                            Placeable::Torch | Placeable::StoneWall | Placeable::WoodWall
                        )
                    {
                        self.toast("That belongs on the farm.", None, 0);
                    }
                    return;
                }
                self.player.inv.take_one(sel);
                let day = self.clock.day;
                let w = self.world_mut();
                match pl {
                    Placeable::Egg => w.set_obj(x, z, Some(Obj::Egg { laid: day })),
                    Placeable::WoodPath => w.set_floor(x, z, Floor::Planks),
                    Placeable::StonePath => w.set_floor(x, z, Floor::Cobble),
                    Placeable::StoneWall => w.set_wall(x, z, Wall::Brick),
                    Placeable::WoodWall => w.set_wall(x, z, Wall::Timber),
                    Placeable::Chest => w.set_obj(
                        x,
                        z,
                        Some(Obj::Chest {
                            items: vec![None; 30],
                        }),
                    ),
                    Placeable::Torch => w.set_obj(x, z, Some(Obj::Torch)),
                    Placeable::Lamp => w.set_obj(x, z, Some(Obj::Lamp)),
                    Placeable::Fence => w.set_obj(x, z, Some(Obj::Fence)),
                    Placeable::Sprinkler(t) => w.set_obj(x, z, Some(Obj::Sprinkler { tier: t })),
                    Placeable::Workbench => w.set_obj(x, z, Some(Obj::Workbench)),
                    Placeable::FlowerPot => w.set_obj(x, z, Some(Obj::FlowerPot { var: 0 })),
                    Placeable::Bench => w.set_obj(x, z, Some(Obj::Bench)),
                    Placeable::EnchantTable => w.set_obj(x, z, Some(Obj::EnchantTable)),
                    Placeable::Furniture(_) | Placeable::Rug(_) | Placeable::WallArt(_) => {}
                }
                io.audio.play(Sfx::Place);
                if pl == Placeable::Egg {
                    // Tucked into a nest of fallen leaves.
                    let c = tile_center(x, z);
                    self.fx
                        .burst(c + Vec3::Y * 0.1, 12, &[ORANGE, GOLD, RUST, CLAY], 1.4, 1.2);
                    self.fx
                        .motes(c + Vec3::Y * 0.3, 8, &[MINT, LAVENDER, WHITE], 0.25);
                    self.toast_colored(
                        "You tuck the egg into a nest of leaves. Now... wait.",
                        Some(item),
                        0,
                        LAVENDER,
                    );
                } else {
                    self.fx
                        .burst(tile_center(x, z), 6, &[SAND, KHAKI], 1.2, 1.0);
                }
            }
            Kind::Produce { hp, energy } => self.eat(item, hp, energy, 0, None, io),
            Kind::Potion { hp, mana, energy } => self.drink(item, hp, mana, energy, io),
            Kind::Food {
                hp,
                energy,
                mana,
                buff,
            } => self.eat(item, hp, energy, mana, buff, io),
            Kind::Gear(b) if b.class.is_armor() => {
                // Wear it straight from the hotbar.
                if self.player.equip_from(sel) {
                    io.audio.play(Sfx::Equip);
                    self.toast(format!("Wearing {}", item.def().name), Some(item), 0);
                    self.fx.motes(
                        self.player.world_pos() + Vec3::Y * 0.5,
                        8,
                        &[WHITE, CREAM, SKY],
                        0.3,
                    );
                }
            }
            Kind::Scroll(_) => {
                self.toast("Bind scrolls at an enchanting table.", None, 0);
            }
            Kind::Pack { .. } => {
                // On your back it goes, straight from the hotbar.
                if self.player.wear_pack_from(sel) {
                    io.audio.play(Sfx::Equip);
                    self.toast(format!("Wearing the {}", item.def().name), Some(item), 0);
                    self.fx.motes(
                        self.player.world_pos() + Vec3::Y * 0.5,
                        8,
                        &[WHITE, CREAM, SAND],
                        0.3,
                    );
                } else {
                    io.audio.play(Sfx::Denied);
                    self.toast("No room in your bag for what's in your backpack!", None, 0);
                }
            }
            Kind::Bomb => self.throw_bomb(io),
            Kind::InkMap => self.read_ink_map(io),
            Kind::Fish => {
                if self.nag <= 0.0 {
                    self.nag = 2.0;
                    self.toast(
                        "Cook it at your stove, or show it off in a fish tank!",
                        None,
                        0,
                    );
                }
            }
            Kind::Wallpaper(_) | Kind::Flooring(_) => {
                self.toast("Use it inside your house.", None, 0);
            }
            Kind::Feather => {
                if let Area::Hollow { .. } = self.area {
                    self.player.inv.take_one(sel);
                    io.audio.play(Sfx::Waystone);
                    self.fx
                        .motes(self.player.world_pos(), 20, &[WHITE, SKY, MINT], 0.5);
                    self.start_fade(Trans::Home);
                } else {
                    self.toast("It only works in the Hollow.", None, 0);
                }
            }
            Kind::HeartCrystal => {
                self.player.inv.take_one(sel);
                self.player.base_hp += 10;
                self.player.hp = self.player.max_hp();
                io.audio.play(Sfx::LevelUp);
                self.fx
                    .motes(self.player.world_pos(), 16, &[PINK, BLUSH, WHITE], 0.4);
                self.toast("Max HP +10", None, 0);
            }
            Kind::SunStone => {
                self.player.inv.take_one(sel);
                self.player.base_energy += 15;
                self.player.energy = self.player.max_energy() as f32;
                io.audio.play(Sfx::LevelUp);
                self.fx
                    .motes(self.player.world_pos(), 16, &[GOLD, CREAM, WHITE], 0.4);
                self.toast("Max energy +15", None, 0);
            }
            Kind::WishStar => {
                self.player.inv.take_one(sel);
                self.player.base_mana += 10;
                self.player.mana = self.player.max_mana() as f32;
                io.audio.play(Sfx::LevelUp);
                self.fx
                    .motes(self.player.world_pos(), 16, &[LAVENDER, BLUSH, WHITE], 0.4);
                self.toast("Max mana +10", None, 0);
            }
            _ => {}
        }
    }

    /// Drinks a potion. Quicker than eating, so it works in a pinch mid-fight.
    fn drink(&mut self, item: Item, hp: i32, mana: i32, energy: i32, io: &mut Io) {
        let p = &mut self.player;
        if p.eat_t > 0.0 {
            return;
        }
        let needs = (hp > 0 && p.hp < p.max_hp())
            || (mana > 0 && p.mana < p.max_mana() as f32)
            || (energy > 0 && p.energy < p.max_energy() as f32);
        if !needs {
            self.toast("You don't need that right now.", None, 0);
            return;
        }
        let sel = p.sel;
        p.inv.take_one(sel);
        p.hp = (p.hp + hp).min(p.max_hp());
        p.mana = (p.mana + mana as f32).min(p.max_mana() as f32);
        p.energy = (p.energy + energy as f32).min(p.max_energy() as f32);
        p.eat_t = 0.3;
        io.audio.play(Sfx::Gulp);
        let pos = p.world_pos() + Vec3::Y * 0.9;
        let at = p.world_pos();
        let mut colors = Vec::new();
        if hp > 0 {
            self.fx.popup(pos, format!("+{hp}"), PINK);
            colors.extend([PINK, RED]);
        }
        if mana > 0 {
            self.fx
                .popup(pos + Vec3::new(-0.3, 0.2, 0.0), format!("+{mana}"), SKY);
            colors.extend([SKY, BLUE]);
        }
        if energy > 0 {
            self.fx
                .popup(pos + Vec3::new(0.3, 0.2, 0.0), format!("+{energy}"), GOLD);
            colors.extend([GOLD, CREAM]);
        }
        colors.push(WHITE);
        self.fx.motes(at + Vec3::Y * 0.3, 14, &colors, 0.35);
        self.fx.ring(at, 2.2, 14, &colors);
        self.flashes.push(Flash {
            pos: at + Vec3::Y * 0.8,
            t: 0.12,
            warmth: if hp > 0 { 6.5 } else { 2.5 },
        });
        self.toast(format!("Drank {}", item.def().name), Some(item), 0);
    }

    fn eat(
        &mut self,
        item: Item,
        hp: i32,
        energy: i32,
        mana: i32,
        buff: Option<super::items::Buff>,
        io: &mut Io,
    ) {
        let p = &mut self.player;
        if p.eat_t > 0.0 {
            return;
        }
        let full = p.hp >= p.max_hp()
            && p.energy >= p.max_energy() as f32
            && (mana == 0 || p.mana >= p.max_mana() as f32);
        if full && buff.is_none() {
            self.toast("You're full.", None, 0);
            return;
        }
        let sel = p.sel;
        p.inv.take_one(sel);
        if let Some(b) = buff {
            p.add_buff(b, item);
            p.refresh();
        }
        p.hp = (p.hp + hp).min(p.max_hp());
        p.energy = (p.energy + energy as f32).min(p.max_energy() as f32);
        p.mana = (p.mana + mana as f32).min(p.max_mana() as f32);
        p.eat_t = 0.6;
        io.audio.play(Sfx::Eat);
        let pos = p.world_pos() + Vec3::Y * 0.9;
        if hp > 0 {
            self.fx.popup(pos, format!("+{hp}"), PINK);
        }
        if energy > 0 {
            self.fx
                .popup(pos + Vec3::new(0.3, 0.2, 0.0), format!("+{energy}"), GOLD);
        }
        if mana > 0 {
            self.fx.popup(
                pos + Vec3::new(-0.3, 0.2, 0.0),
                format!("+{mana}"),
                LAVENDER,
            );
        }
        match buff {
            Some(b) => {
                self.fx.motes(
                    self.player.world_pos() + Vec3::Y * 0.4,
                    10,
                    &[b.stat.def().color, WHITE],
                    0.3,
                );
                self.toast(
                    format!("{}: {}", item.def().name, b.stat.line(b.val as i32)),
                    Some(item),
                    0,
                );
            }
            None => self.toast(format!("Ate {}", item.def().name), None, 0),
        }
    }

    fn interact(&mut self, io: &mut Io) {
        if let Some(i) = self.npc_near() {
            io.audio.play(Sfx::UiSelect);
            self.talk_to(i, io);
            return;
        }
        let Some((tx, tz)) = self.target else { return };
        // Indoors, a piece in hand goes down before anything nearby gets used.
        if self.area == Area::Home {
            let placing = match self.player.held().map(|i| i.def().kind) {
                Some(Kind::Place(pl)) => self.house_target_ok(pl, tx, tz),
                Some(Kind::Wallpaper(_) | Kind::Flooring(_)) => true,
                _ => false,
            };
            if placing {
                self.house_use((tx, tz), false, io);
                return;
            }
        }
        // Check the target tile, then the tile we stand on, then anything adjacent.
        let mut cands = vec![(tx, tz)];
        let (px, pz) = self.player.tile();
        for dz in -1..=1 {
            for dx in -1..=1 {
                if (px + dx, pz + dz) != (tx, tz) {
                    cands.push((px + dx, pz + dz));
                }
            }
        }
        for (x, z) in cands {
            if self.interact_at(x, z, io) {
                return;
            }
        }
        // Pet the cat (or the spider).
        if self.pet_nearby(io) {
            return;
        }
        // Nothing there: use the held item (eat, plant, place).
        if self.area == Area::Home {
            self.house_use((tx, tz), false, io);
            return;
        }
        if let Some(item) = self.player.held() {
            if item.class().and_then(ActKind::for_class).is_none() {
                self.use_item(item, (tx, tz), io);
            }
        }
    }

    fn interact_at(&mut self, x: i32, z: i32, io: &mut Io) -> bool {
        let (ax, az) = self.world().anchor(x, z);
        let Some(obj) = self.world().obj(ax, az).cloned() else {
            return false;
        };
        match obj {
            Obj::House => {
                self.go_indoors(io);
                return true;
            }
            Obj::Egg { laid } => {
                self.listen_to_egg(laid, io);
                return true;
            }
            Obj::Furniture { .. } if self.area == Area::Home => {
                return self.use_furniture(ax, az, io);
            }
            Obj::Furniture { .. } => {
                // A piece on display in Wren's shop.
                if self.nag <= 0.0 {
                    self.nag = 1.5;
                    self.toast("On display - Wren sells these at the counter.", None, 0);
                }
                return true;
            }
            Obj::Bin => self.menu = Menu::Ship { cursor: 0 },
            Obj::Stall => {
                if town::trading(self.clock.min) {
                    self.menu = Menu::shop();
                } else {
                    io.audio.play(Sfx::Denied);
                    self.toast("Burrowby's stall is closed. Open 9am to 5pm.", None, 0);
                }
            }
            Obj::Hollow => {
                let mut floors = vec![1];
                floors.extend(self.waystones.iter().copied());
                self.menu = Menu::Descend { floors, sel: 0 };
            }
            Obj::Chest { .. } => {
                self.menu = Menu::Chest {
                    x: ax,
                    z: az,
                    cursor: 0,
                }
            }
            Obj::Workbench => self.menu = Menu::inventory(true),
            Obj::EnchantTable => self.menu = Menu::enchant(),
            Obj::Counter | Obj::Fixture { var: 7 } => {
                // Whoever minds the counter.
                if let Some(i) = self.folk.iter().position(|n| n.fixed) {
                    self.talk_to(i, io);
                } else {
                    self.toast("Nobody's minding the counter.", None, 0);
                }
                return true;
            }
            Obj::BusStop => {
                let text = if self.area == Area::Farm {
                    "The little bus to Bramblewick stops here all day long. Wave it down?"
                } else {
                    "The bus back to Hollowbloom Farm leaves from here. Wave it down?"
                };
                self.menu = Menu::dialog_choice(
                    text,
                    vec![
                        ("Ride the bus", super::menus::Choice::Bus),
                        ("Not now", super::menus::Choice::Close),
                    ],
                );
            }
            Obj::Building { id } => {
                self.knock(id as usize, io);
                return true;
            }
            Obj::Board => self.open_board(io),
            Obj::Fountain { .. } => self.wish(io),
            Obj::WishTree { blooming } => self.wish_tree(blooming, io),
            Obj::Well => {
                self.toast("It's very deep. Something glints far below...", None, 0);
                io.audio.play_at(Sfx::Water, 0.4, 0.7);
            }
            Obj::Sign { text } => {
                let msg = match text {
                    0 => {
                        "THE HOLLOW\nMind your step. It goes down forever. Waystones every ten floors will bring you home."
                    }
                    2 => {
                        "BUS STOP\nBramblewick - shops, friends and odd jobs. Buses all day, every day. Wave from the shelter!"
                    }
                    _ => "BURROWBY'S GOODS\nSeeds, supplies and a warm hello.",
                };
                self.menu = Menu::dialog(msg);
            }
            Obj::LootChest {
                opened: false,
                gleam,
            } => {
                self.world_mut().set_obj(
                    ax,
                    az,
                    Some(Obj::LootChest {
                        opened: true,
                        gleam,
                    }),
                );
                self.open_loot(ax, az, gleam, io);
                io.audio.play(Sfx::Chest);
            }
            Obj::Hole => self.climb_down(io),
            Obj::Rope => self.climb_up(io),
            Obj::Crack => {
                self.toast(
                    "The floor's cracked here... something's hollow underneath. A bomb would open it up!",
                    None,
                    0,
                );
            }
            Obj::StairsDown => {
                if self.foes.iter().any(|f| f.boss) {
                    self.toast("The guardian blocks the way down.", None, 0);
                    io.audio.play(Sfx::Denied);
                } else {
                    let d = self.depth() + 1;
                    self.start_fade(Trans::Descend {
                        depth: d,
                        via_waystone: false,
                    });
                }
            }
            Obj::Waystone => {
                if self.foes.iter().any(|f| f.boss) {
                    self.toast("The waystone is silent while the guardian lives.", None, 0);
                    io.audio.play(Sfx::Denied);
                } else {
                    let d = self.depth();
                    if !self.waystones.contains(&d) {
                        self.waystones.push(d);
                        self.waystones.sort();
                        io.audio.play(Sfx::Waystone);
                        self.fx.motes(
                            tile_center(ax, az) + Vec3::Y * 0.5,
                            24,
                            &[MINT, AQUA, WHITE],
                            0.5,
                        );
                        self.toast(format!("Waystone {d} attuned!"), None, 0);
                    }
                    self.menu = Menu::dialog_choice(
                        "The waystone hums warmly. Return to the surface?",
                        vec![
                            ("Go home", super::menus::Choice::ReturnHome),
                            ("Keep delving", super::menus::Choice::Close),
                        ],
                    );
                }
            }
            Obj::Crop { crop, days, .. } if crop.stage(days) == 3 => {
                self.harvest(ax, az, crop, io);
            }
            Obj::Bench => {
                self.player.pos = Vec2::new(ax as f32 + 0.5, az as f32 + 0.75);
                self.player.facing = Vec2::new(0.0, 1.0);
                self.fx.motes(
                    self.player.world_pos() + Vec3::Y * 0.6,
                    4,
                    &[PINK, WHITE],
                    0.2,
                );
                self.toast("Ahh, a nice rest.", None, 0);
                self.player.energy =
                    (self.player.energy + 5.0).min(self.player.max_energy() as f32);
            }
            _ => return false,
        }
        if !matches!(self.menu, Menu::None) {
            io.audio.play(Sfx::UiSelect);
        }
        true
    }

    fn harvest(&mut self, x: i32, z: i32, crop: Crop, io: &mut Io) {
        let def = crop.def();
        let mut n = 1 + self.rng.below(def.yield_max as usize) as u16;
        if self.rng.chance(self.player.sheet.frac(Stat::Bounty, 90)) {
            n += 1;
            self.fx
                .popup(tile_center(x, z) + Vec3::Y * 0.8, "Bounty!", ORANGE);
        }
        let mut left = self.player.inv.add(def.produce, n);
        if left > 0 {
            let at = tile_center(x, z);
            while left > 0 {
                self.drops
                    .push(Drop::item(def.produce, 1, at, &mut self.rng));
                left -= 1;
            }
        } else {
            self.toast(def.produce.def().name, Some(def.produce), n as u32);
        }
        // Crops sometimes give a seed back.
        if crop != Crop::Turnip
            && self
                .rng
                .chance(0.12 + self.player.sheet.frac(Stat::Forage, 60))
        {
            if let Some(s) = Item::seed_of(crop) {
                if self.player.inv.add(s, 1) == 0 {
                    self.toast(s.def().name, Some(s), 1);
                }
            }
        }
        if def.regrow > 0 {
            self.farm.set_obj(
                x,
                z,
                Some(Obj::Crop {
                    crop,
                    days: def.days - def.regrow,
                    harvested: true,
                }),
            );
        } else {
            self.farm.set_obj(x, z, None);
        }
        self.stats.harvested += n as u32;
        self.on_harvest(crop, n);
        io.audio.play(Sfx::Harvest);
        self.fx.motes(
            tile_center(x, z) + Vec3::Y * 0.3,
            8,
            &[CREAM, WHITE, LIME],
            0.3,
        );
    }

    fn open_loot(&mut self, x: i32, z: i32, gleam: bool, io: &mut Io) {
        let depth = self.depth().max(1);
        let at = tile_center(x, z);
        let fortune = self.fortune();
        let loot = loot::chest_loot(
            depth,
            self.hollow_biome(depth),
            gleam,
            fortune,
            &mut self.rng,
        );
        self.spill(loot, at, io);
        self.fx
            .motes(at + Vec3::Y * 0.4, 16, &[GOLD, CREAM, WHITE], 0.4);
        if gleam {
            // A gleaming chest: a fountain of gold and a fanfare.
            self.fx
                .motes(at + Vec3::Y * 0.5, 30, &[GOLD, CREAM, WHITE, ORANGE], 0.8);
            self.fx.popup_big(at + Vec3::Y * 1.1, "Gleaming!", GOLD);
            self.stats.gleams += 1;
            io.audio.play(Sfx::Rare);
        }
    }

    /// Throws loot out onto the floor, with a chime for anything rare.
    pub fn spill(&mut self, loot: Vec<Stack>, at: Vec3, io: &mut Io) {
        let mut best = None;
        for s in loot {
            best = best.max(s.rarity());
            self.drops.push(Drop::new(s, at, &mut self.rng));
        }
        if best >= Some(Rarity::Rare) {
            io.audio.play(Sfx::Rare);
        }
    }

    /// The moment a swing, cast or tool use connects.
    fn fire(&mut self, kind: ActKind, (x, z): (i32, i32), dir: Vec2, io: &mut Io) {
        match kind {
            ActKind::Slash => {
                let dmg = self.player.weapon_damage();
                let lvl = self
                    .player
                    .held_stack()
                    .and_then(|s| s.gear)
                    .map_or(1, |g| g.level);
                let reach = 1.35 + (lvl as f32 / 60.0).min(1.0) * 0.25;
                let hits = self.melee(dmg, reach, 1.15, dir, 6.0, io);
                // Cut grass and smash pots in the arc.
                let p = self.player.pos;
                for (dx, dz) in [(0.0, 0.0), (0.7, 0.0), (-0.7, 0.0), (0.0, 0.7), (0.0, -0.7)] {
                    let q = p + dir * 0.9 + Vec2::new(dx, dz) * 0.5;
                    let (tx, tz) = (q.x.floor() as i32, q.y.floor() as i32);
                    self.hit_soft(tx, tz, io);
                }
                if hits == 0 {
                    self.hit_soft(x, z, io);
                    // Steel on stone.
                    let q = p + dir.normalize_or_zero() * 0.95;
                    let (tx, tz) = (q.x.floor() as i32, q.y.floor() as i32);
                    if !self.strike_hard(tx, tz, io) {
                        self.strike_hard(x, z, io);
                    }
                }
            }
            ActKind::Bolt => self.cast_bolt(dir, io),
            ActKind::Blast => {
                let at = self.blast_point(dir);
                self.cast_blast(at, io);
            }
            ActKind::Mine => {
                let dmg = self.player.weapon_damage();
                if self.melee(dmg, 1.0, 0.9, dir, 4.0, io) == 0 {
                    let power = self.player.tool_power();
                    self.mine(x, z, power, io);
                }
            }
            ActKind::Chop => {
                let dmg = self.player.weapon_damage();
                if self.melee(dmg, 1.0, 0.9, dir, 4.0, io) == 0 {
                    let power = self.player.tool_power();
                    self.chop(x, z, power, io);
                }
            }
            ActKind::Till => {
                let n = self
                    .player
                    .held_stack()
                    .and_then(|s| s.main_value())
                    .unwrap_or(1)
                    + self.player.reach();
                for (tx, tz) in self.line((x, z), n) {
                    self.till(tx, tz, io);
                }
            }
            ActKind::Water => {
                let n = 1 + self.player.reach();
                if self.world().floor(x, z) == Floor::Water {
                    self.water(x, z, io);
                } else {
                    for (tx, tz) in self.line((x, z), n) {
                        if self.world().floor(tx, tz) != Floor::Water {
                            self.water(tx, tz, io);
                        }
                    }
                }
            }
            ActKind::Reap => self.reap_patch((x, z), dir, io),
            ActKind::Cast => self.release_spell(dir, io),
        }
    }

    /// Where a staff blast lands: under the mouse when it is close, otherwise ahead.
    fn blast_point(&self, dir: Vec2) -> Vec2 {
        let p = self.player.pos;
        let ahead = p + dir.normalize_or_zero() * 2.4;
        let mut at = match self.aim {
            Some(a) if (a - p).length() < 4.5 => a,
            _ => ahead,
        };
        // Keep it on this side of walls.
        let w = self.world();
        if !w.clear_line(p, at) || w.opaque(at.x.floor() as i32, at.y.floor() as i32) {
            at = p + dir.normalize_or_zero() * 1.2;
        }
        at
    }

    /// `n` tiles in a row, starting at `start` and heading away from the hero.
    fn line(&self, start: (i32, i32), n: i32) -> Vec<(i32, i32)> {
        let (px, pz) = self.player.tile();
        let (mut dx, mut dz) = (start.0 - px, start.1 - pz);
        if dx == 0 && dz == 0 {
            let f = self.player.facing;
            if f.x.abs() > f.y.abs() {
                dx = f.x.signum() as i32;
            } else {
                dz = f.y.signum() as i32;
            }
        } else if dx != 0 && dz != 0 {
            // Diagonal: go along the stronger facing axis.
            let f = self.player.facing;
            if f.x.abs() > f.y.abs() {
                dz = 0;
            } else {
                dx = 0;
            }
        }
        (0..n.max(1))
            .map(|k| (start.0 + dx.signum() * k, start.1 + dz.signum() * k))
            .collect()
    }

    /// A sickle sweep: harvests ripe crops and cuts grass in a patch in front, and nicks
    /// anything in the way.
    fn reap_patch(&mut self, (x, z): (i32, i32), dir: Vec2, io: &mut Io) {
        let dmg = self.player.weapon_damage();
        self.melee(dmg, 1.4, 1.3, dir, 4.0, io);
        let r = 1 + self.player.reach() / 2;
        let mut any = false;
        for dz in -r..=r {
            for dx in -r..=r {
                let (tx, tz) = (x + dx, z + dz);
                match self.world().obj(tx, tz).cloned() {
                    Some(Obj::Crop { crop, days, .. }) if crop.stage(days) == 3 => {
                        if self.area == Area::Farm {
                            self.harvest(tx, tz, crop, io);
                            any = true;
                        }
                    }
                    Some(Obj::Weed { .. } | Obj::Flower { .. }) => {
                        self.hit_soft(tx, tz, io);
                        any = true;
                    }
                    Some(Obj::Shrub { .. }) => {
                        self.cut_shrub(tx, tz, 2, io);
                        any = true;
                    }
                    Some(Obj::Glowcap { .. } | Obj::ShelfFungus { .. }) => {
                        any |= self.gather_glow(tx, tz, io);
                    }
                    _ => {}
                }
            }
        }
        if !any {
            self.hit_soft(x, z, io);
        }
    }

    /// Sword/any-hit things: weeds, flowers, pots, crates.
    pub fn hit_soft(&mut self, x: i32, z: i32, io: &mut Io) {
        let Some(o) = self.world().obj(x, z).cloned() else {
            return;
        };
        let at = tile_center(x, z);
        match o {
            Obj::Weed { .. } => {
                self.world_mut().set_obj(x, z, None);
                self.fx
                    .burst(at + Vec3::Y * 0.2, 8, &[GREEN, LIME, TEAL], 1.8, 1.8);
                if self.rng.chance(0.8) {
                    self.drops
                        .push(Drop::item(Item::Fiber, 1, at, &mut self.rng));
                }
                if self
                    .rng
                    .chance(0.04 + self.player.sheet.frac(Stat::Forage, 60) * 0.5)
                {
                    let seed = self.forage_seed();
                    self.drops.push(Drop::item(seed, 1, at, &mut self.rng));
                }
                // Heartleaf hides among the weeds.
                if self
                    .rng
                    .chance(0.1 + self.player.sheet.frac(Stat::Forage, 60) * 0.3)
                {
                    self.drops
                        .push(Drop::item(Item::Heartleaf, 1, at, &mut self.rng));
                }
                io.audio.play_at(Sfx::Swing, 0.5, 1.4);
            }
            Obj::Flower { .. } => {
                self.world_mut().set_obj(x, z, None);
                self.fx
                    .burst(at + Vec3::Y * 0.2, 8, &[PINK, WHITE, SKY], 1.8, 1.8);
                if self.rng.chance(0.5) {
                    self.drops
                        .push(Drop::item(Item::Fiber, 1, at, &mut self.rng));
                }
            }
            Obj::Shrub { .. } => self.cut_shrub(x, z, 1, io),
            Obj::Pot { .. } | Obj::Crate { .. } | Obj::Keg { .. } => {
                self.world_mut().set_obj(x, z, None);
                io.audio.play(Sfx::Break);
                let col = match o {
                    Obj::Pot { .. } => [CLAY, GOLD, RUST],
                    Obj::Keg { .. } => [RUST, CLAY, SLATE],
                    _ => [CLAY, RUST, SAND],
                };
                self.fx.burst(at + Vec3::Y * 0.3, 12, &col, 2.5, 2.0);
                let depth = self.depth().max(1);
                let fortune = self.fortune();
                let loot = loot::pot_loot(depth, self.hollow_biome(depth), fortune, &mut self.rng);
                self.spill(loot, at, io);
            }
            _ => {}
        }
    }

    /// Whacks a wild bush. It comes out after a few hits, leaving fiber and sticks, and
    /// blueberries if it was a berry bush.
    fn cut_shrub(&mut self, x: i32, z: i32, dmg: i32, io: &mut Io) {
        let Some(Obj::Shrub { var, hp }) = self.world().obj(x, z).cloned() else {
            return;
        };
        let at = tile_center(x, z);
        let leaves = if var == 2 {
            [TEAL, DEEP_TEAL, GREEN]
        } else {
            [GREEN, LIME, TEAL]
        };
        self.fx.burst(at + Vec3::Y * 0.4, 7, &leaves, 1.8, 1.4);
        io.audio.play(Sfx::Chop);
        let hp = hp - dmg.max(1) as i16;
        if hp > 0 {
            self.world_mut().set_obj(x, z, Some(Obj::Shrub { var, hp }));
            return;
        }
        self.world_mut().set_obj(x, z, None);
        io.audio.play_at(Sfx::Break, 0.6, 1.3);
        self.fx.burst(at + Vec3::Y * 0.5, 14, &leaves, 2.4, 1.6);
        let fiber = 1 + self.rng.below(2) as u16;
        self.drops
            .push(Drop::item(Item::Fiber, fiber, at, &mut self.rng));
        if self.rng.chance(0.6) {
            let n = 1 + u16::from(self.bounty());
            self.drops
                .push(Drop::item(Item::Wood, n, at, &mut self.rng));
        }
        let forage = self.player.sheet.frac(Stat::Forage, 60);
        match var {
            1 => {
                let n = 2 + self.rng.below(3) as u16;
                self.drops
                    .push(Drop::item(Item::Blueberry, n, at, &mut self.rng));
            }
            2 if self.rng.chance(0.35 + forage) => {
                self.drops
                    .push(Drop::item(Item::Heartleaf, 1, at, &mut self.rng));
            }
            _ => {}
        }
        if self.rng.chance(0.25 + forage) {
            self.drops
                .push(Drop::item(Item::Heartleaf, 1, at, &mut self.rng));
        }
        if self.rng.chance(0.06 + forage * 0.5) {
            let seed = self.forage_seed();
            self.drops.push(Drop::item(seed, 1, at, &mut self.rng));
        }
    }

    /// A seed found while working: from the Hollow's biome, or the valley's own on the farm.
    fn forage_seed(&mut self) -> Item {
        match self.area {
            Area::Hollow { depth } => loot::biome_seed(self.hollow_biome(depth), &mut self.rng),
            _ => {
                let seeds = [
                    Item::TurnipSeeds,
                    Item::RadishSeeds,
                    Item::WheatSeeds,
                    Item::SeedPotato,
                    Item::PeaSeeds,
                    Item::GarlicBulb,
                    Item::RoseSeeds,
                    Item::CarrotSeeds,
                ];
                seeds[self.rng.below(seeds.len())]
            }
        }
    }

    /// The way out of something at `at` towards the hero, along the ground.
    pub fn facing_out(&self, at: Vec3) -> Vec3 {
        let d = self.player.world_pos() - at;
        let v = Vec3::new(d.x, 0.0, d.z);
        if v.length_squared() < 1e-6 {
            Vec3::Z
        } else {
            v.normalize()
        }
    }

    /// A blade or tool glancing off something hard in front of the hero: sparks fly.
    /// Returns true if there was something hard there.
    fn strike_hard(&mut self, x: i32, z: i32, io: &mut Io) -> bool {
        let w = self.world();
        let hard_wall = matches!(
            w.wall(x, z),
            Wall::Rock
                | Wall::Ore(_)
                | Wall::Brick
                | Wall::Bedrock
                | Wall::Cliff
                | Wall::Sewer
                | Wall::Marble(_)
                | Wall::Sandstone(_)
        ) && w.inside(x, z);
        let hard_obj = w.obj(x, z).is_some_and(|o| {
            matches!(
                o,
                Obj::Rock { .. }
                    | Obj::Boulder { .. }
                    | Obj::Crystal { .. }
                    | Obj::Stalagmite { .. }
            )
        });
        if !hard_wall && !hard_obj {
            return false;
        }
        let at = tile_center(x, z);
        let out = self.facing_out(at);
        let (reach, high) = if hard_wall { (0.5, 0.5) } else { (0.3, 0.3) };
        self.fx.sparks(at + out * reach + Vec3::Y * high, out, 6);
        io.audio
            .play_at(Sfx::Clang, 0.5, 1.05 + self.rng.f32() * 0.2);
        true
    }

    /// A lucky extra drop (Bounty) when breaking something.
    fn bounty(&mut self) -> bool {
        let b = self.player.sheet.frac(Stat::Bounty, 90);
        b > 0.0 && self.rng.chance(b)
    }

    fn mine(&mut self, x: i32, z: i32, power: i32, io: &mut Io) {
        let at = tile_center(x, z);
        let depth = self.depth();
        let wall = self.world().wall(x, z);
        if matches!(
            wall,
            Wall::Rock | Wall::Ore(_) | Wall::Brick | Wall::Timber | Wall::Sewer
        ) {
            let hard = match wall {
                Wall::Rock | Wall::Sewer => 3 + depth as i32 / 6,
                Wall::Ore(o) => 5 + depth as i32 / 6 + o as i32,
                _ => 4,
            };
            let w = self.world_mut();
            let i = w.idx(x, z);
            let dmg = w.wall_dmg.entry(i).or_insert(0);
            *dmg += power as i16;
            let broke = *dmg as i32 >= hard;
            // Struck on the side facing the hero.
            let out = self.facing_out(at);
            let face = at + out * 0.5 + Vec3::Y * 0.5;
            let chips = match wall {
                Wall::Ore(o) => crate::assets::ORE_COLORS[o as usize % 6].to_vec(),
                Wall::Timber => vec![CLAY, RUST],
                _ => vec![KHAKI, ROSEWOOD, SAND],
            };
            self.fx.burst(face, 5, &chips, 2.0, 1.5);
            io.audio.play(Sfx::Mine);
            if wall != Wall::Timber {
                let n = if matches!(wall, Wall::Ore(_)) { 11 } else { 7 };
                self.fx.sparks(face, out, n);
                io.audio
                    .play_at(Sfx::Clang, 0.45, 0.9 + self.rng.f32() * 0.25);
            }
            if broke {
                self.world_mut().set_wall(x, z, Wall::None);
                io.audio.play(Sfx::Break);
                self.fx.burst(at + Vec3::Y * 0.5, 14, &chips, 3.0, 2.0);
                let forage = self.player.sheet.frac(Stat::Forage, 60);
                match wall {
                    Wall::Ore(o) => {
                        let n = 1 + self.rng.below(2) as u16 + u16::from(self.bounty());
                        self.drops
                            .push(Drop::item(ore_item(o), n, at, &mut self.rng));
                        if self.rng.chance(0.5) {
                            self.drops
                                .push(Drop::item(Item::Stone, 1, at, &mut self.rng));
                        }
                        if self.rng.chance(0.06 + forage) {
                            let gem = loot::random_gem(depth, &mut self.rng);
                            self.drops.push(Drop::item(gem, 1, at, &mut self.rng));
                            self.fx.motes(at + Vec3::Y * 0.5, 8, &[WHITE, CREAM], 0.3);
                        }
                    }
                    Wall::Rock | Wall::Sewer => {
                        if self.rng.chance(0.55) {
                            let n = 1 + u16::from(self.bounty());
                            self.drops
                                .push(Drop::item(Item::Stone, n, at, &mut self.rng));
                        }
                        if self.rng.chance(0.03) {
                            self.drops
                                .push(Drop::item(Item::Amber, 1, at, &mut self.rng));
                        }
                        if self.rng.chance(0.012 + forage * 0.25) {
                            let gem = loot::random_gem(depth, &mut self.rng);
                            self.drops.push(Drop::item(gem, 1, at, &mut self.rng));
                        }
                        if self.rng.chance(0.004 + forage * 0.02) {
                            let relic = loot::random_relic(depth, &mut self.rng);
                            self.drops.push(Drop::item(relic, 1, at, &mut self.rng));
                        }
                    }
                    Wall::Brick => {
                        self.drops
                            .push(Drop::item(Item::StoneWall, 1, at, &mut self.rng))
                    }
                    Wall::Timber => {
                        self.drops
                            .push(Drop::item(Item::WoodWall, 1, at, &mut self.rng))
                    }
                    _ => {}
                }
                self.shake = self.shake.max(0.3);
            }
            return;
        }
        let Some(o) = self.world().obj(x, z).cloned() else {
            // Pick up placed floors.
            let f = self.world().floor(x, z);
            if matches!(f, Floor::Planks | Floor::Cobble) {
                let back = if self.area == Area::Farm {
                    Floor::Grass
                } else {
                    Floor::Cave
                };
                self.world_mut().set_floor(x, z, back);
                let item = if f == Floor::Planks {
                    Item::WoodPath
                } else {
                    Item::StonePath
                };
                self.drops.push(Drop::item(item, 1, at, &mut self.rng));
                io.audio.play(Sfx::Place);
            }
            return;
        };
        if matches!(
            o,
            Obj::Rock { .. } | Obj::Boulder { .. } | Obj::Crystal { .. } | Obj::Stalagmite { .. }
        ) {
            let out = self.facing_out(at);
            let high = if matches!(o, Obj::Rock { .. }) {
                0.2
            } else {
                0.4
            };
            self.fx.sparks(at + out * 0.3 + Vec3::Y * high, out, 7);
            io.audio
                .play_at(Sfx::Clang, 0.4, 0.95 + self.rng.f32() * 0.25);
        }
        match o {
            Obj::Rock { hp, var } => {
                let hp = hp - power as i16;
                io.audio.play(Sfx::Mine);
                self.fx
                    .burst(at + Vec3::Y * 0.2, 5, &[SAND, KHAKI, ROSEWOOD], 2.0, 1.5);
                if hp <= 0 {
                    self.world_mut().set_obj(x, z, None);
                    io.audio.play(Sfx::Break);
                    let n = 1 + self.rng.below(2) as u16 + u16::from(self.bounty());
                    self.drops
                        .push(Drop::item(Item::Stone, n, at, &mut self.rng));
                    if self.area == Area::Farm && self.rng.chance(0.15) {
                        self.drops
                            .push(Drop::item(Item::CopperOre, 1, at, &mut self.rng));
                    }
                    if self
                        .rng
                        .chance(0.015 + self.player.sheet.frac(Stat::Forage, 60) * 0.3)
                    {
                        let gem = loot::random_gem(depth.max(1), &mut self.rng);
                        self.drops.push(Drop::item(gem, 1, at, &mut self.rng));
                    }
                } else {
                    self.world_mut().set_obj(x, z, Some(Obj::Rock { hp, var }));
                }
            }
            Obj::Boulder { hp } => {
                let hp = hp - power as i16;
                io.audio.play(Sfx::Mine);
                self.fx
                    .burst(at + Vec3::Y * 0.4, 6, &[SAND, KHAKI, ROSEWOOD], 2.0, 1.5);
                if hp <= 0 {
                    self.world_mut().set_obj(x, z, None);
                    io.audio.play(Sfx::Break);
                    self.shake = 0.5;
                    let n = 5 + self.rng.below(4) as u16;
                    self.drops
                        .push(Drop::item(Item::Stone, n, at, &mut self.rng));
                    if self.rng.chance(0.5) {
                        self.drops
                            .push(Drop::item(Item::IronOre, 1, at, &mut self.rng));
                    }
                } else {
                    self.world_mut().set_obj(x, z, Some(Obj::Boulder { hp }));
                }
            }
            Obj::CandyRock { hp } => self.mine_candy(x, z, hp, power, io),
            Obj::Crystal { hp, var } => {
                let hp = hp - power as i16;
                io.audio.play_at(Sfx::Mine, 1.0, 1.3);
                self.fx
                    .burst(at + Vec3::Y * 0.4, 6, &[MINT, AQUA, WHITE], 2.0, 1.5);
                if hp <= 0 {
                    self.world_mut().set_obj(x, z, None);
                    let item = match var {
                        3 | 5 => Item::Amber,
                        4 => Item::FrostGem,
                        _ => Item::Crystal,
                    };
                    let n = 1 + self.rng.below(2) as u16 + u16::from(self.bounty());
                    self.drops.push(Drop::item(item, n, at, &mut self.rng));
                    if self.rng.chance(0.08) {
                        let gem = loot::random_gem(depth.max(1), &mut self.rng);
                        self.drops.push(Drop::item(gem, 1, at, &mut self.rng));
                    }
                } else {
                    self.world_mut()
                        .set_obj(x, z, Some(Obj::Crystal { hp, var }));
                }
            }
            Obj::Stalagmite { .. } => {
                self.world_mut().set_obj(x, z, None);
                io.audio.play(Sfx::Break);
                self.fx
                    .burst(at + Vec3::Y * 0.4, 10, &[KHAKI, ROSEWOOD], 2.5, 2.0);
                self.drops
                    .push(Drop::item(Item::Stone, 1, at, &mut self.rng));
            }
            Obj::Pot { .. } | Obj::Crate { .. } | Obj::Keg { .. } => self.hit_soft(x, z, io),
            Obj::Lamp
            | Obj::Sprinkler { .. }
            | Obj::FlowerPot { .. }
            | Obj::Torch
            | Obj::Chest { .. }
            | Obj::EnchantTable => {
                self.pick_up(x, z, o, io);
            }
            _ => {}
        }
    }

    fn chop(&mut self, x: i32, z: i32, power: i32, io: &mut Io) {
        let at = tile_center(x, z);
        if self.world().wall(x, z) == Wall::Timber {
            self.world_mut().set_wall(x, z, Wall::None);
            self.drops
                .push(Drop::item(Item::WoodWall, 1, at, &mut self.rng));
            io.audio.play(Sfx::Chop);
            return;
        }
        let Some(o) = self.world().obj(x, z).cloned() else {
            return;
        };
        let leaves = [GREEN, LIME, TEAL];
        match o {
            Obj::Tree { hp, var } | Obj::Pine { hp, var } => {
                let hp = hp - power as i16;
                io.audio.play(Sfx::Chop);
                self.fx.burst(at + Vec3::Y * 1.0, 8, &leaves, 1.8, 1.0);
                self.fx
                    .burst(at + Vec3::Y * 0.3, 4, &[CLAY, RUST], 1.5, 1.5);
                if hp <= 0 {
                    self.world_mut().set_obj(x, z, Some(Obj::Stump { hp: 6 }));
                    io.audio.play(Sfx::Break);
                    self.shake = 0.5;
                    let n = 4 + self.rng.below(3) as u16 + 3 * u16::from(self.bounty());
                    self.drops
                        .push(Drop::item(Item::Wood, n, at, &mut self.rng));
                    if self.rng.chance(0.25) {
                        self.drops
                            .push(Drop::item(Item::Fiber, 2, at, &mut self.rng));
                    }
                    if self
                        .rng
                        .chance(0.05 + self.player.sheet.frac(Stat::Forage, 60))
                    {
                        let seed = self.forage_seed();
                        self.drops.push(Drop::item(seed, 1, at, &mut self.rng));
                    }
                    if self.rng.chance(0.01) {
                        self.drops
                            .push(Drop::item(Item::GoldenAcorn, 1, at, &mut self.rng));
                    }
                    self.fx.burst(at + Vec3::Y * 1.2, 20, &leaves, 2.5, 1.0);
                } else {
                    let o = if matches!(o, Obj::Tree { .. }) {
                        Obj::Tree { hp, var }
                    } else {
                        Obj::Pine { hp, var }
                    };
                    self.world_mut().set_obj(x, z, Some(o));
                }
            }
            Obj::Stump { hp } | Obj::Log { hp } => {
                let hp = hp - power as i16;
                io.audio.play(Sfx::Chop);
                self.fx
                    .burst(at + Vec3::Y * 0.2, 5, &[CLAY, RUST, SAND], 1.8, 1.5);
                if hp <= 0 {
                    self.world_mut().set_obj(x, z, None);
                    io.audio.play(Sfx::Break);
                    let n = if matches!(o, Obj::Log { .. }) {
                        8
                    } else {
                        2 + self.rng.below(2) as u16
                    };
                    self.drops
                        .push(Drop::item(Item::Wood, n, at, &mut self.rng));
                } else {
                    let o = if matches!(o, Obj::Log { .. }) {
                        Obj::Log { hp }
                    } else {
                        Obj::Stump { hp }
                    };
                    self.world_mut().set_obj(x, z, Some(o));
                }
            }
            Obj::Weed { .. } => self.hit_soft(x, z, io),
            Obj::Shrub { .. } => self.cut_shrub(x, z, power * 2, io),
            Obj::Crate { .. } | Obj::Keg { .. } => self.hit_soft(x, z, io),
            // Old planks split into something you can still use.
            Obj::Debris { .. } if self.world().floor(x, z) != Floor::Water => {
                self.world_mut().set_obj(x, z, None);
                io.audio.play(Sfx::Chop);
                self.fx
                    .burst(at + Vec3::Y * 0.15, 8, &[CLAY, RUST, SAND], 1.8, 1.5);
                self.drops
                    .push(Drop::item(Item::Wood, 1, at, &mut self.rng));
            }
            Obj::Fence | Obj::Bench | Obj::Workbench | Obj::Chest { .. } => {
                self.pick_up(x, z, o, io)
            }
            _ => {}
        }
    }

    /// Returns a placed object to the player as an item.
    fn pick_up(&mut self, x: i32, z: i32, o: Obj, io: &mut Io) {
        let at = tile_center(x, z);
        let item = match &o {
            Obj::Lamp => Item::Lamp,
            Obj::Sprinkler { tier } => [
                Item::Sprinkler,
                Item::QualitySprinkler,
                Item::CrystalSprinkler,
            ][*tier as usize % 3],
            Obj::FlowerPot { .. } => Item::FlowerPot,
            Obj::Torch => Item::Torch,
            Obj::Fence => Item::Fence,
            Obj::Bench => Item::Bench,
            Obj::Workbench => Item::Workbench,
            Obj::Chest { items } => {
                for s in items.iter().flatten() {
                    self.drops.push(Drop::new(*s, at, &mut self.rng));
                }
                Item::Chest
            }
            Obj::EnchantTable => Item::EnchantTable,
            _ => return,
        };
        self.world_mut().set_obj(x, z, None);
        self.drops.push(Drop::item(item, 1, at, &mut self.rng));
        io.audio.play(Sfx::Place);
        self.fx
            .burst(at + Vec3::Y * 0.3, 6, &[SAND, KHAKI], 1.5, 1.5);
    }

    fn till(&mut self, x: i32, z: i32, io: &mut Io) {
        if self.area != Area::Farm {
            return;
        }
        let at = tile_center(x, z);
        if let Some(Obj::Weed { .. } | Obj::Flower { .. }) = self.farm.obj(x, z) {
            self.hit_soft(x, z, io);
        }
        if tillable(&self.farm, x, z) && self.farm.obj(x, z).is_none() {
            self.farm.set_floor(x, z, Floor::Tilled);
            io.audio.play(Sfx::Till);
            self.fx
                .burst(at + Vec3::Y * 0.05, 8, &[RUST, CLAY, MAROON], 1.6, 1.8);
            let forage = self.player.sheet.frac(Stat::Forage, 60);
            if self.rng.chance(0.03 + forage * 0.4) {
                let seed = self.forage_seed();
                self.drops.push(Drop::item(seed, 1, at, &mut self.rng));
            }
            if self.rng.chance(0.004 + forage * 0.02) {
                let relic = loot::random_relic(1, &mut self.rng);
                self.drops.push(Drop::item(relic, 1, at, &mut self.rng));
            }
        }
    }

    fn water(&mut self, x: i32, z: i32, io: &mut Io) {
        let at = tile_center(x, z);
        let floor = self.world().floor(x, z);
        if floor == Floor::Water {
            self.player.water = self.player.can_capacity();
            io.audio.play(Sfx::Refill);
            self.fx
                .burst(at + Vec3::Y * 0.1, 10, &[WHITE, SKY, BLUE], 1.8, 2.2);
            self.toast("Watering can refilled", None, 0);
            return;
        }
        if self.player.water == 0 {
            self.toast("Your can is empty. Refill it at the pond.", None, 0);
            io.audio.play(Sfx::Denied);
            return;
        }
        self.player.water -= 1;
        io.audio.play(Sfx::Water);
        self.fx
            .burst(at + Vec3::Y * 0.4, 10, &[WHITE, SKY, BLUE], 1.2, 0.5);
        if floor == Floor::Tilled {
            self.world_mut().set_flag(x, z, WATERED, true);
            let growth = self.player.sheet.frac(Stat::Growth, 60);
            if growth > 0.0 && self.rng.chance(growth) {
                self.world_mut().set_flag(x, z, FERTILE, true);
                self.fx
                    .motes(at + Vec3::Y * 0.2, 5, &[LIME, MINT, WHITE], 0.25);
            }
        }
    }

    // --------------------------------------------------------------------------------------
    // Foes, drops, the cat
    // --------------------------------------------------------------------------------------

    fn update_foes(&mut self, io: &mut Io) {
        if self.foes.is_empty() && self.shots.is_empty() {
            return;
        }
        let dt = io.dt;
        let depth = self.depth();
        let Some(level) = &self.level else { return };
        let world = &level.world;
        let ppos = self.player.pos;
        let mut spawns = Vec::new();
        let mut shots = std::mem::take(&mut self.shots);
        let before = shots.len();
        let mut calls = Vec::new();
        for (i, f) in self.foes.iter_mut().enumerate() {
            let was_alert = f.alert;
            // Chilled creatures move (and think) at half speed.
            let fdt = if f.chill > 0.0 { dt * 0.5 } else { dt };
            f.update(
                fdt,
                world,
                ppos,
                &mut shots,
                &mut spawns,
                &mut self.fx,
                &mut self.rng,
                depth,
            );
            if !was_alert && f.alert {
                io.audio.play_at(Sfx::Alert, 0.6, 1.0);
                if f.boss {
                    self.boss_seen = Some(f.name().to_string());
                }
            }
            if let Some(c) = f.call.take() {
                calls.push((i, c));
            }
        }
        // (A drakeling's breath roars rather than zaps: see `answer`.)
        if shots[before..].iter().any(|s| s.kind != ShotKind::Breath) {
            io.audio.play(Sfx::Shoot);
        }
        for (i, c) in calls {
            self.answer(i, c, io);
        }
        // Keep enemies from stacking up.
        let n = self.foes.len();
        for i in 0..n {
            for j in (i + 1)..n {
                let d = self.foes[j].pos - self.foes[i].pos;
                let min = self.foes[i].radius + self.foes[j].radius;
                let l = d.length();
                if l < min && l > 1e-4 {
                    let push = d / l * (min - l) * 0.5;
                    self.foes[i].pos -= push;
                    self.foes[j].pos += push;
                }
            }
        }
        if !spawns.is_empty() && self.foes.len() < 40 {
            let moon = self.moon();
            for mut f in spawns {
                f.feel_the_moon(moon);
                self.foes.push(f);
            }
        }
        // Contact damage.
        let mut hurt: Option<(i32, Vec2, Option<usize>)> = None;
        for (i, f) in self.foes.iter().enumerate() {
            let d = ppos - f.pos;
            if d.length() < f.radius + RADIUS && f.grounded() && f.flash <= 0.0 && f.hp > 0 {
                hurt = Some((f.dmg, d.normalize_or_zero(), Some(i)));
            }
        }
        // Bats back off after touching you.
        if hurt.is_some() {
            for f in self.foes.iter_mut() {
                if f.foe == dungeon::Foe::Bat && (f.pos - ppos).length() < 0.8 {
                    f.st = St::Rest;
                    f.t = 0.8;
                }
            }
        }
        let world = &self.level.as_ref().unwrap().world;
        let mut splats = Vec::new();
        shots.retain_mut(|s| {
            if !s.update(dt, world) {
                // Ink splashes where it lands (just short of any wall it hit).
                if s.kind == ShotKind::Ink {
                    splats.push(s.pos - s.vel * dt);
                }
                return false;
            }
            if (s.pos - ppos).length() < s.radius + RADIUS {
                hurt = Some((s.dmg, s.vel.normalize_or_zero(), None));
                if s.kind == ShotKind::Ink {
                    splats.push(ppos);
                }
                return false;
            }
            true
        });
        self.shots = shots;
        for at in splats {
            self.splash_ink(at);
        }
        if let Some((dmg, dir, from)) = hurt {
            self.hurt_player(dmg, dir, from, io);
            self.reap(io);
        }
    }

    /// Answers what a creature has just done that reaches past itself (see `Call`).
    fn answer(&mut self, i: usize, call: Call, io: &mut Io) {
        let (pos, at, biome) = {
            let f = &self.foes[i];
            (f.pos, f.world_pos(), f.biome)
        };
        match call {
            Call::Howl => {
                let pitch = 0.95 + self.rng.f32() * 0.1;
                io.audio.play_at(Sfx::Howl, 0.9, pitch);
                self.fx.popup_big(at + Vec3::Y * 1.4, "Awoooo!", CREAM);
                self.fx
                    .ring(at + Vec3::Y * 1.0, 4.0, 24, &[WHITE, CREAM, SAND]);
                // Everything in earshot comes running, and riled up (a second howl riles
                // them further; a third, no more).
                for (j, f) in self.foes.iter_mut().enumerate() {
                    if j != i && (f.pos - pos).length() < HOWL_REACH {
                        f.alert = true;
                        if f.fury < 1.6 {
                            f.fury += 0.2;
                            f.speed *= 1.08;
                        }
                    }
                }
            }
            Call::Breath => {
                let pitch = if biome % 6 == 3 { 0.9 } else { 1.15 };
                io.audio.play_at(Sfx::Breath, 0.8, pitch);
            }
            Call::Mend => {
                // Whoever's worst hurt close by (not itself).
                let hurt = |f: &Enemy| f.hp as f32 / f.max_hp.max(1) as f32;
                let patient = self
                    .foes
                    .iter()
                    .enumerate()
                    .filter(|(j, f)| {
                        *j != i
                            && f.hp > 0
                            && f.hp < f.max_hp
                            && (f.pos - pos).length() < MEND_REACH
                    })
                    .min_by(|a, b| hurt(a.1).total_cmp(&hurt(b.1)))
                    .map(|(j, _)| j);
                let Some(j) = patient else {
                    // Nobody needs it yet: it looks again in a moment.
                    self.foes[i].summon = MEND_AGAIN;
                    return;
                };
                let f = &mut self.foes[j];
                let heal = (f.max_hp / 4).max(4).min(f.max_hp - f.hp);
                f.hp += heal;
                let to = f.world_pos() + Vec3::Y * (0.4 * f.scale());
                let ground = Vec3::new(f.pos.x, 0.05, f.pos.y);
                self.fx.popup(to + Vec3::Y * 0.4, format!("+{heal}"), LIME);
                self.fx.motes(to, 14, &[WHITE, LIME, GREEN], 0.4);
                self.fx.ring(ground, 1.6, 14, &[WHITE, LIME, GREEN]);
                // A stream of green light from the leafling to whoever it mends.
                let from = at + Vec3::Y * 0.1;
                for k in 1..6 {
                    self.fx
                        .motes(from.lerp(to, k as f32 / 6.0), 1, &[LIME, MINT], 0.05);
                }
                io.audio.play_at(Sfx::Magic, 0.45, 1.5);
            }
            Call::Charge => {
                let pitch = 0.9 + self.rng.f32() * 0.15;
                io.audio.play_at(Sfx::Bellow, 0.9, pitch);
            }
            Call::Crash => {
                // The walls shake with it.
                io.audio.play_at(Sfx::Crash, 1.0, 1.0);
                let near = (self.player.pos - pos).length();
                self.shake = self.shake.max((0.6 - near * 0.05).max(0.15));
                self.fx.popup(at + Vec3::Y * 1.3, "Dazed!", GOLD);
            }
            Call::Screech => {
                let pitch = 0.95 + self.rng.f32() * 0.1;
                io.audio.play_at(Sfx::Screech, 0.7, pitch);
            }
        }
    }

    fn update_drops(&mut self, io: &mut Io) {
        let dt = io.dt;
        let world = area_world(
            self.area,
            &self.farm,
            &self.town,
            &self.house.world,
            &self.level,
            &self.room,
        );
        let ppos = self.player.pos;
        let mut picked: Vec<usize> = Vec::new();
        for (i, d) in self.drops.iter_mut().enumerate() {
            let can = d.is_coin() || self.player.inv.can_fit_stack(&d.stack);
            if d.update(dt, world, ppos, can) {
                picked.push(i);
            }
        }
        for i in picked.into_iter().rev() {
            let d = self.drops.remove(i);
            let item = d.stack.item;
            if let Some(v) = item.coin_value() {
                let worth = v as u64 * d.stack.n as u64;
                self.money += worth;
                self.stats.earned += worth;
                self.toast(item.def().name, Some(item), d.stack.n as u32);
                io.audio
                    .play_at(Sfx::Coin, 0.6, 0.95 + self.rng.f32() * 0.15);
                continue;
            }
            let left = self.player.inv.add_stack(d.stack);
            let got = d.stack.n - left;
            if got > 0 {
                self.on_pickup(item);
                let name = d.stack.name();
                let color = d.stack.rarity().map_or(CREAM, |r| r.color());
                self.toast_colored(name, Some(item), got as u32, color);
                let chime = d.stack.rarity().is_some_and(|r| r >= Rarity::Rare);
                if chime {
                    io.audio.play(Sfx::Rare);
                } else {
                    io.audio
                        .play_at(Sfx::Pickup, 0.8, 1.0 + self.rng.f32() * 0.1);
                }
            }
        }
    }

    fn reveal(&mut self) {
        let Some(l) = &self.level else { return };
        let w = &l.world;
        let (px, pz) = self.player.tile();
        let r = 7;
        for z in (pz - r).max(0)..=(pz + r).min(w.h - 1) {
            for x in (px - r).max(0)..=(px + r).min(w.w - 1) {
                if (x - px).pow(2) + (z - pz).pow(2) <= r * r {
                    let i = (z * w.w + x) as usize;
                    if i < self.revealed.len() {
                        self.revealed[i] = true;
                    }
                }
            }
        }
    }
}

pub fn tile_center(x: i32, z: i32) -> Vec3 {
    Vec3::new(x as f32 + 0.5, 0.0, z as f32 + 0.5)
}

/// Moves the stack in `slot` of `from` into `to`, as much as fits.
pub fn transfer(from: &mut Inventory, slot: usize, to: &mut Inventory) {
    if let Some(s) = from.slots[slot] {
        let left = to.add_stack(s);
        from.slots[slot] = if left > 0 {
            Some(Stack { n: left, ..s })
        } else {
            None
        };
    }
}

/// The Ward bubble pops (a new day, a fall in the Hollow).
fn p_ward_off(p: &mut Player) {
    p.ward = 0.0;
    p.ward_t = 0.0;
}
