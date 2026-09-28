//! Scripted play-throughs that drive the real update loop with simulated input.

use glam::Vec2;

use super::items::{Crop, Item, Stack};
use super::menus::{Choice, Menu};
use super::play::{Play, Trans};
use super::world::{Area, Floor, Obj, WATERED, Wall};
use super::{Io, Settings};
use crate::audio::Audio;
use crate::input::{Input, KeyCode};

struct Sim {
    play: Play,
    input: Input,
    audio: Audio,
    settings: Settings,
}

impl Sim {
    fn new() -> Sim {
        let mut play = Play::new(4242);
        play.menu = Menu::None;
        Sim {
            play,
            input: Input::default(),
            audio: Audio::silent(),
            settings: Settings::default(),
        }
    }

    fn frames(&mut self, n: usize) {
        for _ in 0..n {
            let mut io = Io {
                dt: 1.0 / 60.0,
                input: &self.input,
                audio: &self.audio,
                view: (480, 270),
                quit: false,
                toggle_fullscreen: false,
            };
            self.play.update(&mut io, &mut self.settings);
            self.input.end_frame(1.0 / 60.0);
        }
    }

    /// Taps a key for one frame, then lets the game run.
    fn tap(&mut self, key: KeyCode, after: usize) {
        self.input.key_event(key, true, false);
        self.frames(1);
        self.input.key_event(key, false, false);
        self.frames(after);
    }

    fn select(&mut self, slot: usize) {
        self.play.player.sel = slot;
    }

    /// Puts the player on a tile, facing a direction, with nothing in the way.
    fn stand(&mut self, x: i32, z: i32, facing: Vec2) {
        let p = &mut self.play;
        p.player.pos = Vec2::new(x as f32 + 0.5, z as f32 + 0.5);
        p.player.facing = facing;
        p.player.act = None;
    }
}

fn clear_farm_tile(p: &mut Play, x: i32, z: i32) {
    p.farm.set_wall(x, z, Wall::None);
    p.farm.set_floor(x, z, Floor::Grass);
    p.farm.set_obj(x, z, None);
}

#[test]
fn till_plant_water_and_grow() {
    let mut s = Sim::new();
    for z in 15..18 {
        for x in 34..37 {
            clear_farm_tile(&mut s.play, x, z);
        }
    }
    s.stand(35, 15, Vec2::new(0.0, 1.0));
    let energy = s.play.player.energy;

    // Hoe.
    s.select(1);
    s.tap(KeyCode::KeyJ, 40);
    assert_eq!(
        s.play.farm.floor(35, 16),
        Floor::Tilled,
        "hoe should till the tile in front"
    );
    assert!(s.play.player.energy < energy, "tools cost energy");

    // Plant turnip seeds.
    s.select(5);
    let seeds = s.play.player.inv.count(Item::TurnipSeeds);
    s.tap(KeyCode::KeyJ, 5);
    assert!(matches!(
        s.play.farm.obj(35, 16),
        Some(Obj::Crop {
            crop: Crop::Turnip,
            days: 0,
            ..
        })
    ));
    assert_eq!(s.play.player.inv.count(Item::TurnipSeeds), seeds - 1);

    // Water.
    s.select(2);
    let water = s.play.player.water;
    s.tap(KeyCode::KeyJ, 40);
    assert!(
        s.play.farm.flag(35, 16, WATERED),
        "watering can should water the soil"
    );
    assert_eq!(s.play.player.water, water - 1);

    // Sleep through the night.
    let day = s.play.clock.day;
    s.play.start_fade(Trans::Sleep { passed_out: false });
    s.frames(60);
    assert_eq!(s.play.clock.day, day + 1);
    assert!(matches!(s.play.menu, Menu::Summary));
    assert!(matches!(
        s.play.farm.obj(35, 16),
        Some(Obj::Crop { days: 1, .. })
    ));
    assert!(
        !s.play.farm.flag(35, 16, WATERED) || s.play.rain,
        "water dries overnight unless it rains"
    );
    assert_eq!(s.play.player.energy, s.play.player.max_energy as f32);
    s.tap(KeyCode::KeyE, 2);
    assert!(matches!(s.play.menu, Menu::None));
}

#[test]
fn harvest_ripe_crop() {
    let mut s = Sim::new();
    clear_farm_tile(&mut s.play, 40, 16);
    s.play.farm.set_floor(40, 16, Floor::Tilled);
    s.play.farm.set_obj(
        40,
        16,
        Some(Obj::Crop {
            crop: Crop::CrystalBerry,
            days: 6,
            harvested: false,
        }),
    );
    clear_farm_tile(&mut s.play, 40, 15);
    s.stand(40, 15, Vec2::new(0.0, 1.0));
    s.frames(2);
    s.tap(KeyCode::KeyE, 2);
    assert!(
        s.play.player.inv.count(Item::CrystalBerry) >= 1,
        "interact harvests"
    );
    // Berries regrow instead of disappearing.
    assert!(matches!(
        s.play.farm.obj(40, 16),
        Some(Obj::Crop {
            harvested: true,
            days: 3,
            ..
        })
    ));
}

#[test]
fn shipping_pays_overnight() {
    let mut s = Sim::new();
    s.play.shipping.push(Stack::new(Item::Glowcap, 10));
    let gold = s.play.gold;
    s.play.start_fade(Trans::Sleep { passed_out: false });
    s.frames(60);
    assert_eq!(s.play.gold, gold + 600);
    assert!(s.play.shipping.is_empty());
}

#[test]
fn chop_a_tree() {
    let mut s = Sim::new();
    clear_farm_tile(&mut s.play, 44, 30);
    clear_farm_tile(&mut s.play, 44, 31);
    s.play
        .farm
        .set_obj(44, 31, Some(Obj::Tree { var: 0, hp: 6 }));
    s.stand(44, 30, Vec2::new(0.0, 1.0));
    s.select(3);
    for _ in 0..4 {
        s.tap(KeyCode::KeyJ, 40);
    }
    assert!(
        matches!(s.play.farm.obj(44, 31), Some(Obj::Stump { .. })),
        "tree becomes a stump"
    );
    s.frames(90);
    assert!(
        s.play.player.inv.count(Item::Wood) >= 4,
        "wood is collected"
    );
}

#[test]
fn delve_fight_and_descend() {
    let mut s = Sim::new();
    s.play.start_fade(Trans::Descend {
        depth: 1,
        via_waystone: false,
    });
    s.frames(60);
    assert_eq!(s.play.area, Area::Hollow { depth: 1 });
    assert!(!s.play.foes.is_empty());
    assert_eq!(s.play.deepest, 1);

    // Put a slime right in front of the hero and swing until it pops.
    let p = s.play.player.pos;
    s.play.foes.truncate(1);
    s.play.foes[0].pos = p + Vec2::new(0.0, 0.9);
    s.play.foes[0].alert = false;
    s.play.player.facing = Vec2::new(0.0, 1.0);
    s.play.player.hurt = 100.0;
    s.select(0);
    for _ in 0..12 {
        if s.play.foes.is_empty() {
            break;
        }
        s.play.foes[0].pos = p + Vec2::new(0.0, 0.9);
        s.tap(KeyCode::KeyJ, 25);
    }
    assert!(s.play.foes.is_empty(), "the enemy should be defeated");
    assert_eq!(s.play.stats.kills, 1);
    assert!(s.play.player.xp > 0);

    // Walk to the stairs and go down.
    let (sx, sz) = s.play.level.as_ref().unwrap().stairs;
    s.stand(sx, sz - 1, Vec2::new(0.0, 1.0));
    s.frames(1);
    s.tap(KeyCode::KeyE, 60);
    assert_eq!(s.play.area, Area::Hollow { depth: 2 });
}

#[test]
fn mining_walls_yields_ore() {
    let mut s = Sim::new();
    s.play.start_fade(Trans::Descend {
        depth: 3,
        via_waystone: false,
    });
    s.frames(60);
    s.play.foes.clear();
    // Find a wall tile beside open floor and dig it out.
    let w = &mut s.play.level.as_mut().unwrap().world;
    let mut spot = None;
    'find: for z in 3..w.h - 3 {
        for x in 3..w.w - 3 {
            if w.wall(x, z) == Wall::Rock && w.wall(x, z - 1) == Wall::None && !w.blocked(x, z - 1)
            {
                spot = Some((x, z));
                break 'find;
            }
        }
    }
    let (x, z) = spot.expect("a wall to dig");
    w.set_wall(x, z, Wall::Ore(0));
    s.stand(x, z - 1, Vec2::new(0.0, 1.0));
    s.select(4);
    for _ in 0..12 {
        s.tap(KeyCode::KeyJ, 30);
        if s.play.world().wall(x, z) == Wall::None {
            break;
        }
    }
    assert_eq!(
        s.play.world().wall(x, z),
        Wall::None,
        "the vein should break"
    );
    s.frames(90);
    assert!(
        s.play.player.inv.count(Item::CopperOre) >= 1,
        "copper ore is collected"
    );
}

#[test]
fn guardian_waystone_and_home() {
    let mut s = Sim::new();
    s.play.start_fade(Trans::Descend {
        depth: 10,
        via_waystone: false,
    });
    s.frames(60);
    assert!(
        s.play.foes.iter().any(|f| f.boss),
        "floor 10 has a guardian"
    );
    let (wx, wz) = s
        .play
        .level
        .as_ref()
        .unwrap()
        .waystone
        .expect("waystone on floor 10");
    // The waystone refuses while the guardian lives.
    s.stand(wx, wz + 1, Vec2::new(0.0, -1.0));
    s.play.player.hurt = 100.0;
    s.frames(1);
    s.tap(KeyCode::KeyE, 2);
    assert!(s.play.waystones.is_empty());
    s.play.foes.retain(|f| !f.boss);
    s.play.foes.clear();
    s.tap(KeyCode::KeyE, 2);
    assert_eq!(s.play.waystones, vec![10]);
    // Choose "Go home".
    match &mut s.play.menu {
        Menu::Dialog { shown, choices, .. } => {
            *shown = 999.0;
            assert_eq!(choices[0].1, Choice::ReturnHome);
        }
        _ => panic!("expected the waystone dialog"),
    }
    s.tap(KeyCode::Enter, 60);
    assert_eq!(s.play.area, Area::Farm);
    // The Hollow entrance now offers floor 10.
    s.play.start_fade(Trans::Descend {
        depth: 10,
        via_waystone: true,
    });
    s.frames(60);
    assert!(
        !s.play.foes.iter().any(|f| f.boss),
        "no guardian when arriving by waystone"
    );
}

#[test]
fn crafting_consumes_ingredients() {
    let mut s = Sim::new();
    s.play.player.inv.add(Item::Wood, 25);
    s.play.menu = Menu::Inventory {
        craft: true,
        cursor: 0,
        recipe: 1,
        scroll: 0,
    };
    s.tap(KeyCode::Enter, 2);
    assert_eq!(s.play.player.inv.count(Item::Chest), 1);
    assert_eq!(s.play.player.inv.count(Item::Wood), 5);
}

#[test]
fn save_and_load_round_trip() {
    let dir = std::env::temp_dir().join(format!("hollowbloom-test-{}", std::process::id()));
    // SAFETY: tests in this binary that touch the data dir all go through this one test.
    unsafe {
        std::env::set_var("HOLLOWBLOOM_DATA", &dir);
    }
    let mut p = Play::new(99);
    p.gold = 1234;
    p.clock.day = 7;
    p.deepest = 23;
    p.waystones = vec![10, 20];
    p.player.inv.add(Item::MossMelon, 3);
    p.farm.set_obj(
        20,
        30,
        Some(Obj::Chest {
            items: vec![Some(Stack::new(Item::Bone, 4)), None],
        }),
    );
    super::save::write(&p).expect("save");
    let q = super::save::read().expect("load");
    assert_eq!(q.gold, 1234);
    assert_eq!(q.clock.day, 7);
    assert_eq!(q.deepest, 23);
    assert_eq!(q.waystones, vec![10, 20]);
    assert_eq!(q.player.inv.count(Item::MossMelon), 3);
    assert!(matches!(q.farm.obj(20, 30), Some(Obj::Chest { .. })));
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn fainting_sends_you_home() {
    let mut s = Sim::new();
    s.play.gold = 500;
    s.play.start_fade(Trans::Descend {
        depth: 4,
        via_waystone: false,
    });
    s.frames(60);
    let day = s.play.clock.day;
    s.play.player.hp = 0;
    s.frames(90);
    assert_eq!(s.play.area, Area::Farm);
    assert_eq!(s.play.clock.day, day + 1);
    assert_eq!(s.play.gold, 450, "a tenth of your gold is lost");
    assert!(s.play.player.hp > 0);
}

#[test]
fn staying_up_too_late() {
    let mut s = Sim::new();
    s.play.clock.min = super::play::DAY_END - 0.5;
    let day = s.play.clock.day;
    s.frames(90);
    assert_eq!(s.play.clock.day, day + 1);
    assert_eq!(s.play.clock.min, super::play::DAY_START);
}

#[test]
fn every_rendered_pixel_is_in_the_palette() {
    use crate::render::Renderer;
    let mut game = super::Game::new();
    let mut r = Renderer::new(320, 180);
    let input = Input::default();
    // Title, farm by day and night, and a Hollow floor.
    game.draw(&mut r, &input);
    assert!(r.fb.color.iter().all(|&c| c < 32));
    game.new_game(5);
    let audio = Audio::silent();
    let mut settings = Settings::default();
    for (min, depth) in [(480.0, 0), (1380.0, 0), (600.0, 17)] {
        if let Some(p) = game.play_mut() {
            p.clock.min = min;
            if depth > 0 {
                p.start_fade(Trans::Descend {
                    depth,
                    via_waystone: false,
                });
            }
            for _ in 0..70 {
                let mut io = Io {
                    dt: 1.0 / 60.0,
                    input: &input,
                    audio: &audio,
                    view: (320, 180),
                    quit: false,
                    toggle_fullscreen: false,
                };
                p.update(&mut io, &mut settings);
            }
        }
        game.draw(&mut r, &input);
        assert!(
            r.fb.color.iter().all(|&c| c < 32),
            "frame at {min} / floor {depth}"
        );
        let distinct = {
            let mut seen = [false; 256];
            r.fb.color.iter().for_each(|&c| seen[c as usize] = true);
            seen.iter().filter(|&&b| b).count()
        };
        assert!(
            distinct > 8,
            "the scene should use a good part of the palette"
        );
    }
}
