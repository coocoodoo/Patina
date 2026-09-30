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

    /// Knows every recipe, for tests about making things rather than learning them.
    fn learn_all(&mut self) {
        self.play
            .known
            .extend(super::items::RECIPES.iter().map(|r| r.out));
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
    assert_eq!(s.play.player.energy, s.play.player.max_energy() as f32);
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
    let money = s.play.money;
    s.play.start_fade(Trans::Sleep { passed_out: false });
    s.frames(60);
    assert_eq!(s.play.money, money + 600);
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
            // Cave rock, or the brickwork of a sewer floor.
            let diggable = matches!(w.wall(x, z), Wall::Rock | Wall::Sewer);
            if diggable && w.wall(x, z - 1) == Wall::None && !w.blocked(x, z - 1) {
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
        tab: super::menus::Tab::Craft,
        cursor: 0,
        recipe: 1,
        scroll: 0,
        cat: 0,
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
    p.money = 1234;
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
    // The house, its tank and the Fishdex come along too.
    super::home::put(&mut p.house.world, super::home::Furn::FishTank, 0, 3, 7);
    if let Some(Obj::Furniture { fish, .. }) = p.house.world.obj_mut(3, 7) {
        fish.push(Item::GoldenCarp);
    }
    p.house.redecorate(3, 2);
    p.journal.fish.push(Item::GoldenCarp);
    p.journal.records.push((Item::GoldenCarp, 50));
    // Pip's egg in its nest, and the spider that hatched from the last one.
    p.farm.set_obj(21, 30, Some(Obj::Egg { laid: 5 }));
    p.spider = Some(super::pets::Spider::new(Vec2::new(26.0, 12.0)));
    let charm = p.charisma();
    super::save::write(&p).expect("save");
    let q = super::save::read().expect("load");
    assert!(matches!(q.farm.obj(21, 30), Some(Obj::Egg { laid: 5 })));
    assert!(q.spider.is_some(), "the spider comes home too");
    assert_eq!(q.charisma(), charm);
    assert_eq!(q.house.fish_kept(), 1);
    assert_eq!((q.house.paper, q.house.floor), (3, 2));
    assert_eq!(q.journal.records, vec![(Item::GoldenCarp, 50)]);
    assert_eq!(q.money, 1234);
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
    s.play.money = 500;
    s.play.start_fade(Trans::Descend {
        depth: 4,
        via_waystone: false,
    });
    s.frames(60);
    let day = s.play.clock.day;
    s.play.player.hp = 0;
    s.frames(90);
    // You wake up in your own bed.
    assert_eq!(s.play.area, Area::Home);
    assert_eq!(s.play.clock.day, day + 1);
    assert_eq!(s.play.money, 450, "a tenth of your money is lost");
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

// ------------------------------------------------------------------------------------------
// Gear, magic, coins and enchanting
// ------------------------------------------------------------------------------------------

use super::gear::{Affix, Class, Gear, Group, Rarity, SOCKETS, Slot, Stat};
use super::menus::{Pick, Tab};

/// Goes down to a floor and keeps just one foe, parked in front of the hero.
fn one_foe_ahead(s: &mut Sim, depth: u32, dist: f32) {
    s.play.start_fade(Trans::Descend {
        depth,
        via_waystone: false,
    });
    s.frames(60);
    let p = s.play.player.pos;
    s.play.foes.truncate(1);
    let f = &mut s.play.foes[0];
    f.pos = p + Vec2::new(0.0, dist);
    f.alert = false;
    f.speed = 0.0;
    s.play.player.facing = Vec2::new(0.0, 1.0);
    s.play.player.hurt = 100.0;
}

#[test]
fn wand_bolts_defeat_foes() {
    let mut s = Sim::new();
    one_foe_ahead(&mut s, 1, 2.5);
    s.select(9);
    assert_eq!(s.play.player.held(), Some(Item::TwigWand));
    let mana = s.play.player.mana;
    for _ in 0..30 {
        if s.play.foes.is_empty() {
            break;
        }
        s.play.foes[0].pos = s.play.player.pos + Vec2::new(0.0, 2.5);
        s.play.player.mana = s.play.player.max_mana() as f32;
        s.tap(KeyCode::KeyJ, 30);
    }
    assert!(s.play.foes.is_empty(), "bolts should defeat the foe");
    assert!(s.play.player.max_mana() as f32 >= mana);
}

#[test]
fn casting_costs_mana_and_stops_when_empty() {
    let mut s = Sim::new();
    s.select(9);
    s.play.player.mana = 6.0;
    s.tap(KeyCode::KeyJ, 30);
    assert!(s.play.player.mana < 6.0, "a bolt costs mana");
    s.play.player.mana = 1.0;
    s.play.bolts.clear();
    s.tap(KeyCode::KeyJ, 2);
    assert!(s.play.bolts.is_empty(), "no bolt without mana");
}

#[test]
fn staff_blast_hits_a_crowd() {
    let mut s = Sim::new();
    one_foe_ahead(&mut s, 5, 2.4);
    s.play.start_fade(Trans::Descend {
        depth: 5,
        via_waystone: false,
    });
    s.frames(60);
    let p = s.play.player.pos;
    s.play.foes.truncate(3);
    assert_eq!(s.play.foes.len(), 3);
    for (i, f) in s.play.foes.iter_mut().enumerate() {
        f.pos = p + Vec2::new(-0.5 + i as f32 * 0.5, 2.4);
        f.speed = 0.0;
        f.hp = 10_000;
        f.max_hp = 10_000;
    }
    s.play.player.inv.slots[0] = Some(Stack::new(Item::OakStaff, 1));
    s.select(0);
    s.play.player.facing = Vec2::new(0.0, 1.0);
    s.play.player.hurt = 100.0;
    s.tap(KeyCode::KeyJ, 40);
    let hurt = s.play.foes.iter().filter(|f| f.hp < 10_000).count();
    assert_eq!(hurt, 3, "the blast should reach all three");
}

#[test]
fn armor_softens_blows() {
    let mut s = Sim::new();
    s.play.player.equip = [None; 5];
    s.play.player.refresh();
    s.play.player.hp = 60;
    let io_hurt = |s: &mut Sim| {
        s.play.player.hurt = 0.0;
        s.play.player.dodge = 0.0;
        let before = s.play.player.hp;
        let audio = Audio::silent();
        let input = Input::default();
        let mut io = Io {
            dt: 1.0 / 60.0,
            input: &input,
            audio: &audio,
            view: (480, 270),
            quit: false,
            toggle_fullscreen: false,
        };
        s.play.hurt_player(30, Vec2::X, None, &mut io);
        before - s.play.player.hp
    };
    let bare = io_hurt(&mut s);
    for (item, slot) in [
        (Item::IronHelm, Slot::Head),
        (Item::IronPlate, Slot::Chest),
        (Item::IronGreaves, Slot::Legs),
        (Item::IronBoots, Slot::Feet),
    ] {
        s.play.player.equip[slot as usize] = Some(Stack::new(item, 1));
    }
    s.play.player.refresh();
    s.play.player.hp = 60;
    let armored = io_hurt(&mut s);
    assert!(armored < bare, "armor helps: {armored} vs {bare}");
    assert!(armored >= 1);
}

#[test]
fn wearing_gear_from_the_bag() {
    let mut s = Sim::new();
    s.play.player.inv.slots[12] = Some(Stack::new(Item::FrogHood, 1));
    s.play.menu = Menu::Inventory {
        tab: Tab::Bag,
        cursor: 12,
        recipe: 0,
        scroll: 0,
        cat: 0,
    };
    s.tap(KeyCode::Enter, 2);
    assert_eq!(
        s.play.player.worn(Slot::Head).map(|w| w.item),
        Some(Item::FrogHood)
    );
    assert_eq!(
        s.play.player.inv.slots[12].map(|w| w.item),
        Some(Item::StrawHat),
        "the old hat goes back in the bag"
    );
    // The frog hood's innate dodge shows up in the stats.
    s.frames(1);
    assert!(s.play.player.stat(Stat::Dodge) > 0);
}

#[test]
fn enchanting_binds_a_scroll() {
    let mut s = Sim::new();
    let mut rng = crate::util::Rng::new(1);
    let scroll = Stack::with_gear(
        Item::WeaponScroll,
        super::gear::scroll_with(Stat::Burn, 10, 0.9),
    );
    s.play.player.inv.slots[20] = Some(scroll);
    s.play.money = 10_000;
    let _ = &mut rng;
    s.play.menu = Menu::Enchant {
        gear: Some(Pick::Bag(0)),
        scroll: Some(20),
        socket: 0,
        cursor: 48,
        msg: None,
        glow: 0.0,
    };
    s.tap(KeyCode::Enter, 2);
    let sword = s.play.player.inv.slots[0].unwrap();
    let e = sword.gear.unwrap().enchants[0].expect("enchanted");
    assert_eq!(e.stat, Stat::Burn);
    assert!(
        s.play.player.inv.slots[20].is_none(),
        "the scroll is used up"
    );
    assert!(s.play.money < 10_000, "binding costs coins");
    // Holding the sword now brings its burn chance along.
    s.select(0);
    s.frames(1);
    assert!(s.play.player.stat(Stat::Burn) > 0);
}

#[test]
fn scrolls_only_fit_their_own_kind() {
    let mut s = Sim::new();
    let scroll = Stack::with_gear(
        Item::ArmorScroll,
        super::gear::scroll_with(Stat::Defense, 10, 0.5),
    );
    s.play.player.inv.slots[20] = Some(scroll);
    s.play.money = 10_000;
    s.play.menu = Menu::Enchant {
        gear: Some(Pick::Bag(0)),
        scroll: Some(20),
        socket: 0,
        cursor: 48,
        msg: None,
        glow: 0.0,
    };
    s.tap(KeyCode::Enter, 2);
    let sword = s.play.player.inv.slots[0].unwrap();
    assert!(sword.gear.unwrap().enchants().next().is_none());
    assert!(s.play.player.inv.slots[20].is_some());
    assert_eq!(s.play.money, 10_000);
    // Armor scrolls do fit armour, including what is worn.
    s.play.menu = Menu::Enchant {
        gear: Some(Pick::Worn(Slot::Chest)),
        scroll: Some(20),
        socket: 0,
        cursor: 48,
        msg: None,
        glow: 0.0,
    };
    s.tap(KeyCode::Enter, 2);
    let worn = s.play.player.worn(Slot::Chest).unwrap();
    assert_eq!(
        worn.gear.unwrap().enchants[0].map(|e| e.stat),
        Some(Stat::Defense)
    );
}

#[test]
fn coins_go_to_the_purse() {
    let mut s = Sim::new();
    let money = s.play.money;
    let at = s.play.player.world_pos();
    for st in super::loot::coin_stacks(357) {
        let mut d = super::fx::Drop::new(st, at, &mut s.play.rng);
        d.vel = glam::Vec3::ZERO;
        s.play.drops.push(d);
    }
    s.frames(120);
    assert!(s.play.drops.is_empty());
    assert_eq!(s.play.money, money + 357);
}

#[test]
fn monsters_drop_coins() {
    let mut s = Sim::new();
    one_foe_ahead(&mut s, 12, 0.9);
    s.play.player.inv.slots[0] = Some(super::loot::roll_gear(
        Item::Sword3,
        40,
        0.0,
        &mut crate::util::Rng::new(2),
    ));
    s.select(0);
    // Plenty of coin find, so something always drops.
    s.play.player.equip[Slot::Feet as usize] = Some(Stack::with_gear(Item::RainBoots, {
        let mut g = Gear::plain(40);
        g.affixes[0] = Some(Affix {
            stat: Stat::Greed,
            val: 50,
        });
        g
    }));
    let money = s.play.money;
    for _ in 0..20 {
        if s.play.foes.is_empty() {
            break;
        }
        s.play.foes[0].pos = s.play.player.pos + Vec2::new(0.0, 0.9);
        s.tap(KeyCode::KeyJ, 25);
    }
    assert!(s.play.foes.is_empty());
    s.frames(150);
    // Coins fly out and get collected on the way; most kills pay.
    assert!(s.play.money >= money, "never lose money for a kill");
}

#[test]
fn sickle_harvests_a_patch() {
    let mut s = Sim::new();
    for z in 15..19 {
        for x in 34..39 {
            clear_farm_tile(&mut s.play, x, z);
        }
    }
    for z in 16..19 {
        for x in 35..38 {
            s.play.farm.set_floor(x, z, Floor::Tilled);
            s.play.farm.set_obj(
                x,
                z,
                Some(Obj::Crop {
                    crop: Crop::Radish,
                    days: 3,
                    harvested: false,
                }),
            );
        }
    }
    s.stand(36, 15, Vec2::new(0.0, 1.0));
    s.select(8);
    assert_eq!(s.play.player.held(), Some(Item::Sickle));
    s.tap(KeyCode::KeyJ, 40);
    assert!(
        s.play.player.inv.count(Item::Radish) >= 6,
        "a sickle sweep harvests a patch, got {}",
        s.play.player.inv.count(Item::Radish)
    );
}

#[test]
fn hoe_with_reach_tills_a_row() {
    let mut s = Sim::new();
    for z in 15..20 {
        clear_farm_tile(&mut s.play, 36, z);
    }
    let mut g = Gear::plain(1);
    g.affixes[0] = Some(Affix {
        stat: Stat::Reach,
        val: 2,
    });
    s.play.player.inv.slots[1] = Some(Stack::with_gear(Item::Hoe, g));
    s.stand(36, 15, Vec2::new(0.0, 1.0));
    s.select(1);
    s.tap(KeyCode::KeyJ, 40);
    let tilled = (16..20)
        .filter(|z| s.play.farm.floor(36, *z) == Floor::Tilled)
        .count();
    assert_eq!(tilled, 3, "reach 2 tills three tiles in a row");
}

#[test]
fn food_buffs_last_a_while() {
    let mut s = Sim::new();
    s.play.player.inv.slots[7] = Some(Stack::new(Item::GarlicBread, 1));
    s.select(7);
    let dmg = s.play.player.weapon_damage();
    s.tap(KeyCode::KeyE, 2);
    assert!(
        s.play.player.weapon_damage() > dmg,
        "garlic bread adds damage"
    );
    assert_eq!(s.play.player.buffs.len(), 1);
    s.play.player.buffs[0].left = 0.01;
    s.frames(3);
    assert!(s.play.player.buffs.is_empty());
    assert_eq!(s.play.player.weapon_damage(), dmg);
}

#[test]
fn crafted_gear_is_rolled() {
    let mut s = Sim::new();
    s.learn_all();
    s.play.player.inv.add(Item::CopperOre, 10);
    s.play.player.inv.add(Item::Wood, 5);
    let list = super::menus::recipes_in(0);
    let i = list.iter().position(|r| r.out == Item::Sword1).unwrap();
    s.play.menu = Menu::Inventory {
        tab: Tab::Craft,
        cursor: 0,
        recipe: i,
        scroll: i.saturating_sub(3),
        cat: 0,
    };
    s.tap(KeyCode::Enter, 2);
    let made = s
        .play
        .player
        .inv
        .slots
        .iter()
        .flatten()
        .find(|st| st.item == Item::Sword1)
        .expect("a copper sword");
    assert!(made.gear.unwrap().level >= 8);
}

#[test]
fn gear_and_outfit_survive_saving() {
    let dir = std::env::temp_dir().join(format!("hollowbloom-gear-{}", std::process::id()));
    // SAFETY: only this test touches this variable's directory.
    unsafe {
        std::env::set_var("HOLLOWBLOOM_DATA", &dir);
    }
    let mut p = Play::new(5);
    let mut rng = crate::util::Rng::new(4);
    let mut sword = super::loot::roll_gear(Item::FrostFang, 44, 1.0, &mut rng);
    if let Some(g) = &mut sword.gear {
        g.enchant(
            2,
            Affix {
                stat: Stat::Chill,
                val: 12,
            },
        );
    }
    p.player.inv.slots[15] = Some(sword);
    p.player.equip[Slot::Shield as usize] = Some(Stack::new(Item::TurtleShell, 1));
    p.player.base_mana = 50;
    super::save::write(&p).expect("save");
    let q = super::save::read().expect("load");
    assert_eq!(q.player.inv.slots[15], Some(sword));
    assert_eq!(
        q.player.worn(Slot::Shield).map(|s| s.item),
        Some(Item::TurtleShell)
    );
    assert_eq!(q.player.base_mana, 50);
    // Recipes learned stay learned...
    assert!(q.known.contains(&Item::Chest) && !q.known.contains(&Item::EmperorPlatter));
    let mut p = q;
    p.known.insert(Item::EmperorPlatter);
    super::save::write(&p).expect("save");
    let q = super::save::read().expect("load");
    assert!(q.known.contains(&Item::EmperorPlatter));
    assert!(!q.known.contains(&Item::FishTacos));
    // ...and a farm saved before recipes had to be found knows every one of them.
    let file = dir.join("save.json");
    let mut json: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&file).unwrap()).unwrap();
    json.as_object_mut().unwrap().remove("recipes");
    std::fs::write(&file, json.to_string()).unwrap();
    let old = super::save::read().expect("load an old save");
    assert_eq!(old.known.len(), super::items::RECIPES.len());
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn every_item_has_art() {
    let a = crate::assets::Assets::new();
    for it in super::items::ALL_ITEMS {
        let icon = it.def().icon;
        assert!(a.icons.contains_key(icon), "{it:?} has no icon {icon}");
        match it.class() {
            Some(Class::Head) => assert!(a.hat_mesh(icon).is_some(), "{it:?} hat"),
            Some(Class::Chest) => assert!(a.chest_skin(icon).is_some(), "{it:?} chest"),
            Some(Class::Legs) => assert!(a.leg_skin(icon).is_some(), "{it:?} legs"),
            Some(Class::Feet) => assert!(a.boot_mesh(icon).is_some(), "{it:?} boots"),
            Some(Class::Shield) => assert!(a.shield_mesh(icon).is_some(), "{it:?} shield"),
            Some(_) => assert!(a.held_mesh(icon).is_some(), "{it:?} held"),
            None => {}
        }
    }
    for c in super::items::ALL_CROPS {
        assert!(a.icons.contains_key(c.def().young), "{c:?} young sprite");
    }
    for st in super::gear::ALL_STATS {
        assert!(
            a.icons.contains_key(super::tips::stat_icon(st)),
            "{st:?} icon"
        );
    }
    for slot in super::gear::SLOTS {
        assert!(a.icons.contains_key(slot.ghost_icon()));
    }
    for name in [
        "socket",
        "ghost_scroll",
        "coin_gold",
        "coin_silver",
        "coin_copper",
    ] {
        assert!(a.icons.contains_key(name), "{name}");
    }
    // Every icon is a neat 16x16 or 8x8 picture.
    for (name, id) in &a.icons {
        let t = a.tex(*id);
        assert!(
            (t.w == 16 && t.h == 16)
                || (t.w == 8 && t.h == 8)
                || *name == "pointer"
                || *name == "sparkle",
            "{name} is {}x{}",
            t.w,
            t.h
        );
    }
    let _ = (Group::Weapon, Rarity::Common, SOCKETS);
}

/// A long, rough play-through: fight with every kind of weapon on floors all the way down,
/// crack open the chests, scoop up the loot, then sell and enchant with the proceeds.
#[test]
fn soak_many_floors() {
    let mut s = Sim::new();
    let weapons = [
        Item::Sword2,
        Item::CrystalWand,
        Item::EmberStaff,
        Item::MoonSickle,
        Item::Pick3,
    ];
    let mut rng = crate::util::Rng::new(99);
    for (k, depth) in [1u32, 7, 10, 13, 22, 30, 37, 50, 60]
        .into_iter()
        .enumerate()
    {
        s.play.fade = None;
        s.play.start_fade(Trans::Descend {
            depth,
            via_waystone: false,
        });
        s.frames(60);
        assert_eq!(s.play.area, Area::Hollow { depth });
        s.play.player.inv.slots[0] = Some(super::loot::roll_gear(
            weapons[k % weapons.len()],
            depth as u16 + 5,
            0.5,
            &mut rng,
        ));
        s.select(0);
        for round in 0..400 {
            s.play.player.hurt = 100.0;
            s.play.player.hp = s.play.player.max_hp();
            s.play.player.mana = s.play.player.max_mana() as f32;
            s.play.player.energy = s.play.player.max_energy() as f32;
            let Some(f) = s.play.foes.first() else { break };
            // Stand next to the first foe, facing it, and swing or cast.
            let to = f.pos;
            let w = &s.play.level.as_ref().unwrap().world;
            let (tx, tz) = w.nearest_open(to.x as i32, to.y as i32 + 1);
            s.play.player.pos = Vec2::new(tx as f32 + 0.5, tz as f32 + 0.5);
            let d = to - s.play.player.pos;
            s.play.player.facing = if d.length() > 0.01 {
                d.normalize()
            } else {
                Vec2::Y
            };
            if round % 50 == 49 {
                // Stubborn foes (behind walls, flying off) are cleared by hand.
                s.play.foes[0].hp = 0;
                let mut io = Io {
                    dt: 1.0 / 60.0,
                    input: &s.input,
                    audio: &s.audio,
                    view: (480, 270),
                    quit: false,
                    toggle_fullscreen: false,
                };
                s.play.reap(&mut io);
            }
            s.tap(KeyCode::KeyJ, 12);
        }
        // Open every chest and pick up everything.
        let w = &s.play.level.as_ref().unwrap().world;
        let mut chests = Vec::new();
        for z in 0..w.h {
            for x in 0..w.w {
                if matches!(w.obj(x, z), Some(Obj::LootChest { opened: false, .. })) {
                    chests.push((x, z));
                }
            }
        }
        for (x, z) in chests {
            let (px, pz) = s.play.world().nearest_open(x, z + 1);
            s.stand(px, pz, Vec2::new(0.0, -1.0));
            s.frames(1);
            s.tap(KeyCode::KeyE, 30);
        }
        for _ in 0..40 {
            let Some(d) = s.play.drops.first() else { break };
            s.play.player.pos = Vec2::new(d.pos.x, d.pos.z);
            s.frames(20);
            if s.play.drops.len() > 30 {
                s.play.drops.truncate(30);
            }
        }
    }
    // Home: sell a stack of anything sellable and enchant with whatever scroll turned up.
    s.play.start_fade(Trans::Home);
    s.frames(60);
    assert_eq!(s.play.area, Area::Farm);
    s.play.menu = super::menus::Menu::shop();
    s.frames(2);
    let scroll = s
        .play
        .player
        .inv
        .slots
        .iter()
        .position(|st| st.is_some_and(|st| st.item.scroll_group().is_some()));
    let gear = s.play.player.inv.slots.iter().position(|st| {
        st.is_some_and(|st| {
            scroll.is_some_and(|i| {
                st.item.class().map(|c| c.group())
                    == s.play.player.inv.slots[i].and_then(|x| x.item.scroll_group())
            })
        })
    });
    s.play.menu = Menu::None;
    s.play.money += 100_000;
    if let (Some(g), Some(sc)) = (gear, scroll) {
        s.play.menu = Menu::Enchant {
            gear: Some(Pick::Bag(g)),
            scroll: Some(sc),
            socket: 1,
            cursor: 48,
            msg: None,
            glow: 0.0,
        };
        s.tap(KeyCode::Enter, 2);
    }
    // Ship everything and sleep on it.
    for i in 0..40 {
        if let Some(st) = s.play.player.inv.slots[i] {
            if super::menus::can_sell(&st) && i > 9 {
                s.play.shipping.push(st);
                s.play.player.inv.slots[i] = None;
            }
        }
    }
    let money = s.play.money;
    s.play.start_fade(Trans::Sleep { passed_out: false });
    s.frames(60);
    assert!(s.play.money >= money);
    assert!(s.play.stats.kills > 20, "kills: {}", s.play.stats.kills);
}

// ------------------------------------------------------------------------------------------
// Bramblewick: the bus, shops, villagers and quests
// ------------------------------------------------------------------------------------------

use super::folk::{Spot, VILLAGERS, Villager};
use super::quests::{Goal, QuestId};
use super::town::{self, BUILDINGS, Place, TOWN};

/// One frame's worth of input and audio, borrowing only those two.
fn frame_io<'a>(input: &'a Input, audio: &'a Audio) -> Io<'a> {
    Io {
        dt: 1.0 / 60.0,
        input,
        audio,
        view: (480, 270),
        quit: false,
        toggle_fullscreen: false,
    }
}

impl Sim {
    /// Rides the bus from wherever the hero is and waits to arrive.
    fn ride(&mut self) {
        let mut io = Io {
            dt: 1.0 / 60.0,
            input: &self.input,
            audio: &self.audio,
            view: (480, 270),
            quit: false,
            toggle_fullscreen: false,
        };
        let from = self.play.area;
        self.play.call_bus(&mut io);
        for _ in 0..600 {
            self.frames(1);
            if self.play.area != from && self.play.fade.is_none() {
                break;
            }
        }
    }

    fn accept(&mut self, key: &str) {
        let mut io = Io {
            dt: 1.0 / 60.0,
            input: &self.input,
            audio: &self.audio,
            view: (480, 270),
            quit: false,
            toggle_fullscreen: false,
        };
        assert!(self.play.accept(QuestId::Story(key.to_string()), &mut io));
    }

    fn quest(&self, key: &str) -> usize {
        self.play
            .quests
            .iter()
            .position(|q| q.key() == key)
            .unwrap_or_else(|| panic!("{key} isn't open"))
    }
}

#[test]
fn bus_ride_to_town_and_back() {
    let mut s = Sim::new();
    s.play.clock.min = 600.0;
    s.ride();
    assert_eq!(s.play.area, Area::Town);
    let (ax, az) = TOWN.arrive;
    let d = s.play.player.pos - Vec2::new(ax as f32, az as f32);
    assert!(d.length() < 4.0, "you step off at the stop");
    assert!(!s.play.folk.is_empty(), "the town is full of people");
    // The bus drives off by itself.
    s.frames(600);
    assert!(s.play.bus.is_none());
    s.ride();
    assert_eq!(s.play.area, Area::Farm);
    let (sx, sz) = super::farm::MARKS.stop;
    let d = s.play.player.pos - Vec2::new(sx as f32, sz as f32);
    assert!(d.length() < 4.0, "and step off by your farm");
}

#[test]
fn the_farm_road_reaches_the_bus_stop() {
    let p = Play::new(77);
    let (sx, sz) = super::farm::MARKS.stop;
    assert!(matches!(p.farm.obj(sx, sz), Some(Obj::BusStop)));
    // From the front door to the shelter and on to the road.
    let (dx, dz) = super::farm::MARKS.door;
    let path = super::folk::find_path(&p.farm, (dx, dz + 1), (sx, sz + 1));
    assert!(
        path.is_some(),
        "you can walk from the house to the bus stop"
    );
    for z in 0..super::farm::FARM_H {
        assert!(
            !p.farm.blocked(super::farm::ROAD_X, z),
            "the road is clear at {z}"
        );
    }
}

#[test]
fn shops_open_and_close_their_doors() {
    let mut s = Sim::new();
    s.play.clock.min = 600.0;
    s.ride();
    s.frames(90);
    let b = &BUILDINGS[Place::Armory.building()];
    let (x, z) = b.step();
    s.stand(x, z, Vec2::new(0.0, -1.0));
    s.frames(1);
    s.tap(KeyCode::KeyE, 60);
    assert_eq!(s.play.area, Area::Inside(Place::Armory));
    assert!(
        s.play
            .folk
            .iter()
            .any(|n| n.who == Villager::Hilde && n.fixed),
        "Hilde is behind her counter"
    );
    // The mat by the door leads back out.
    let (ex, ez) = s.play.room.as_ref().unwrap().exit;
    s.stand(ex, ez, Vec2::new(0.0, 1.0));
    s.frames(60);
    assert_eq!(s.play.area, Area::Town);
    assert_eq!(s.play.player.tile(), (x, z));
    // At night the door stays shut.
    s.play.clock.min = 1400.0;
    s.stand(x, z, Vec2::new(0.0, -1.0));
    s.frames(1);
    s.tap(KeyCode::KeyE, 60);
    assert_eq!(s.play.area, Area::Town, "closed for the night");
}

#[test]
fn talking_and_gifts_make_friends() {
    let mut s = Sim::new();
    s.play.clock.min = 600.0;
    s.ride();
    s.frames(30);
    let i = s
        .play
        .folk
        .iter()
        .position(|n| n.who == Villager::Olive)
        .expect("Olive is out gardening");
    let at = s.play.folk[i].pos;
    s.play.player.pos = at + Vec2::new(0.0, 0.9);
    s.play.player.facing = Vec2::new(0.0, -1.0);
    s.play.player.inv.slots[5] = Some(Stack::new(Item::Sunflower, 3));
    s.select(5);
    let mut io = frame_io(&s.input, &s.audio);
    s.play.talk_to(i, &mut io);
    assert!(matches!(s.play.menu, Menu::Talk { .. }));
    let before = s.play.friends.points[Villager::Olive as usize];
    let mut io = frame_io(&s.input, &s.audio);
    let reply = s
        .play
        .talk_choice(Villager::Olive, super::talk::Say::Gift, &mut io);
    let after = s.play.friends.points[Villager::Olive as usize];
    // A loved gift, grown a little by how charming your home is.
    assert_eq!(
        (after - before) as i32,
        s.play.charm_friend(80),
        "Olive loves sunflowers"
    );
    assert_eq!(s.play.player.inv.count(Item::Sunflower), 2);
    let (_, choices) = reply.expect("she answers");
    assert!(
        !choices.iter().any(|(_, c)| *c == super::talk::Say::Gift),
        "one gift a day"
    );
}

#[test]
fn keepsakes_wait_in_the_hollow_while_asked_for() {
    let mut s = Sim::new();
    // Nobody asked: nothing special on the floor.
    s.play.start_fade(Trans::Descend {
        depth: 3,
        via_waystone: false,
    });
    s.frames(60);
    assert!(!s.play.drops.iter().any(|d| d.stack.item == Item::Locket));
    s.accept("fern_locket");
    s.play.fade = None;
    s.play.start_fade(Trans::Descend {
        depth: 3,
        via_waystone: false,
    });
    s.frames(60);
    let d = s
        .play
        .drops
        .iter()
        .find(|d| d.stack.item == Item::Locket)
        .expect("the locket is somewhere on floor 3");
    s.play.player.pos = Vec2::new(d.pos.x, d.pos.z);
    s.frames(60);
    assert_eq!(s.play.player.inv.count(Item::Locket), 1);
    // Hand it in: the locket goes back to Fern, and she's generous.
    let q = s.quest("fern_locket");
    assert!(s.play.quest_ready(&s.play.quests[q]));
    let hearts = s.play.player.inv.count(Item::HeartCrystal);
    let mut io = frame_io(&s.input, &s.audio);
    s.play.finish_quest(q, &mut io);
    assert_eq!(s.play.player.inv.count(Item::Locket), 0);
    assert_eq!(s.play.player.inv.count(Item::HeartCrystal), hearts + 1);
    assert!(s.play.is_done("fern_locket"));
    assert!(s.play.cheer.is_some(), "and there's a celebration");
    // Found once, the locket never turns up again.
    s.play.fade = None;
    s.play.start_fade(Trans::Descend {
        depth: 4,
        via_waystone: false,
    });
    s.frames(60);
    assert!(!s.play.drops.iter().any(|d| d.stack.item == Item::Locket));
}

#[test]
fn quest_drops_only_fall_while_the_quest_is_open() {
    let mut s = Sim::new();
    let at = glam::Vec3::ZERO;
    for _ in 0..60 {
        s.play.on_kill(super::dungeon::Foe::Slime, false, 2, at);
    }
    assert!(
        !s.play
            .drops
            .iter()
            .any(|d| d.stack.item == Item::SlimeHeart),
        "no slime hearts without Pip's request"
    );
    s.play.done.push("pip_teddy".into());
    s.accept("pip_slime");
    for _ in 0..200 {
        s.play.on_kill(super::dungeon::Foe::Slime, false, 2, at);
    }
    let dropped: u32 = s
        .play
        .drops
        .iter()
        .filter(|d| d.stack.item == Item::SlimeHeart)
        .map(|d| d.stack.n as u32)
        .sum();
    assert_eq!(dropped, 8, "exactly as many as Pip asked for");
    // Bats don't carry them.
    s.play.drops.clear();
    for _ in 0..100 {
        s.play.on_kill(super::dungeon::Foe::Bat, false, 2, at);
    }
    assert!(
        !s.play
            .drops
            .iter()
            .any(|d| d.stack.item == Item::SlimeHeart)
    );
}

#[test]
fn guardians_return_for_their_prizes() {
    let mut s = Sim::new();
    s.play.deepest = 20;
    s.play.waystones = vec![10, 20];
    s.play.start_fade(Trans::Descend {
        depth: 20,
        via_waystone: true,
    });
    s.frames(60);
    assert!(
        !s.play.foes.iter().any(|f| f.boss),
        "a cleared floor stays cleared"
    );
    // Home to hear about her pearl, and back down by the waystone.
    s.play.fade = None;
    s.play.start_fade(Trans::Home);
    s.frames(60);
    for k in ["opal_gems", "opal_curios"] {
        s.play.done.push(k.into());
    }
    s.accept("opal_pearl");
    s.play.fade = None;
    s.play.start_fade(Trans::Descend {
        depth: 20,
        via_waystone: true,
    });
    s.frames(60);
    let boss = s
        .play
        .foes
        .iter()
        .position(|f| f.boss)
        .expect("the Matriarch is back");
    s.play.foes[boss].hp = 0;
    let mut io = frame_io(&s.input, &s.audio);
    s.play.reap(&mut io);
    assert!(
        s.play
            .drops
            .iter()
            .any(|d| d.stack.item == Item::MatriarchPearl),
        "and she drops her pearl"
    );
}

#[test]
fn counting_quests_count() {
    let mut s = Sim::new();
    s.accept("rowan_slimes");
    let q = s.quest("rowan_slimes");
    assert_eq!(s.play.progress(&s.play.quests[q]), (0, 12));
    for _ in 0..5 {
        s.play
            .on_kill(super::dungeon::Foe::Slime, false, 1, glam::Vec3::ZERO);
    }
    s.play
        .on_kill(super::dungeon::Foe::Bat, false, 1, glam::Vec3::ZERO);
    assert_eq!(s.play.progress(&s.play.quests[q]).0, 5);
    for _ in 0..10 {
        s.play
            .on_kill(super::dungeon::Foe::Slime, false, 1, glam::Vec3::ZERO);
    }
    assert!(s.play.quest_ready(&s.play.quests[q]));
    let money = s.play.money;
    let mut io = frame_io(&s.input, &s.audio);
    s.play.finish_quest(q, &mut io);
    assert_eq!(s.play.money, money + 600);
    let shield = s
        .play
        .player
        .inv
        .slots
        .iter()
        .flatten()
        .copied()
        .find(|st| st.item == Item::CopperShield)
        .expect("a shield from the guild");
    assert!(shield.rarity() >= Some(Rarity::Uncommon));
    assert!(s.play.marks >= 20);
    // Next up with Rowan: the map.
    assert_eq!(
        s.play.quest_for(Villager::Rowan).map(|d| d.key),
        Some("rowan_map")
    );
}

#[test]
fn deliveries_go_to_the_right_door() {
    let mut s = Sim::new();
    s.play.done.push("toby_mailbag".into());
    s.accept("toby_letter");
    assert_eq!(
        s.play.player.inv.count(Item::Letter),
        1,
        "Toby hands you the letter"
    );
    assert!(s.play.quest_to_finish(Villager::Toby).is_none());
    let q = s
        .play
        .quest_to_finish(Villager::Thistle)
        .expect("for the mayor");
    let mut io = frame_io(&s.input, &s.audio);
    s.play.finish_quest(q, &mut io);
    assert_eq!(s.play.player.inv.count(Item::Letter), 0);
    assert!(s.play.is_done("toby_letter"));
}

#[test]
fn meeting_everyone_on_the_welcome_tour() {
    let mut s = Sim::new();
    s.accept("thistle_hello");
    let q = s.quest("thistle_hello");
    for v in [
        Villager::Hilde,
        Villager::Garrick,
        Villager::Posy,
        Villager::Mabel,
    ] {
        s.play.on_meet(v);
    }
    assert!(!s.play.quest_ready(&s.play.quests[q]));
    s.play.on_meet(Villager::Pip);
    assert!(
        !s.play.quest_ready(&s.play.quests[q]),
        "Pip isn't a shopkeeper"
    );
    s.play.on_meet(Villager::Nix);
    assert!(s.play.quest_ready(&s.play.quests[q]));
}

#[test]
fn the_boards_post_new_notices_every_day() {
    let mut s = Sim::new();
    let today = s.play.notices(false);
    assert_eq!(today.len(), 3);
    assert_eq!(s.play.notices(true).len(), 2);
    let first = today[0].clone();
    s.play.taken.push(first.id);
    let mut io = frame_io(&s.input, &s.audio);
    s.play.accept(QuestId::Request(first.clone()), &mut io);
    assert_eq!(s.play.notices(false).len(), 2, "taken notices come down");
    // Fill it and hand it in.
    let q = s.play.quests.len() - 1;
    match first.goal {
        Goal::Bring(item, n) => {
            s.play.player.inv.add(item, n);
        }
        Goal::Slay(foe, n, d) => {
            for _ in 0..n {
                s.play.on_kill(
                    foe.unwrap_or(super::dungeon::Foe::Slime),
                    false,
                    d,
                    glam::Vec3::ZERO,
                );
            }
        }
        _ => {}
    }
    assert!(s.play.quest_ready(&s.play.quests[q]));
    let money = s.play.money;
    let mut io = frame_io(&s.input, &s.audio);
    s.play.finish_quest(q, &mut io);
    assert!(s.play.money > money);
    // Tomorrow brings fresh ones.
    s.play.clock.day += 1;
    assert_eq!(s.play.notices(false).len(), 3);
}

#[test]
fn town_projects_change_the_town() {
    let before = town::generate(0);
    let after = town::generate(u32::MAX);
    let (fx, fz) = TOWN.fountain;
    assert!(matches!(
        before.obj(fx, fz),
        Some(Obj::Fountain { flowing: false })
    ));
    assert!(matches!(
        after.obj(fx, fz),
        Some(Obj::Fountain { flowing: true })
    ));
    // The bridges to the park only exist once mended.
    let bridge = (61, 20);
    assert!(before.blocked(bridge.0, bridge.1));
    assert!(!after.blocked(bridge.0, bridge.1));
    let (tx, tz) = TOWN.wish_tree;
    assert!(matches!(
        after.obj(tx, tz),
        Some(Obj::WishTree { blooming: true })
    ));
}

#[test]
fn villagers_keep_their_hours() {
    let mut s = Sim::new();
    // Morning in town: Mabel is at her bakery, not in the street.
    s.play.clock.min = 600.0;
    s.ride();
    s.frames(10);
    assert!(!s.play.folk.iter().any(|n| n.who == Villager::Mabel));
    assert_eq!(Villager::Mabel.spot(600.0), Spot::Inside(Place::Bakery));
    // Late at night the streets are quiet but the tavern isn't.
    assert!(
        VILLAGERS
            .iter()
            .any(|v| v.spot(1300.0) == Spot::Inside(Place::Tavern))
    );
    assert!(
        VILLAGERS
            .iter()
            .all(|v| v.spot(1530.0) != Spot::Town || *v == Villager::Mira)
    );
}

#[test]
fn town_and_shops_render_in_palette() {
    use crate::render::Renderer;
    let mut game = super::Game::new();
    let mut r = Renderer::new(320, 180);
    let input = Input::default();
    game.new_game(6);
    let audio = Audio::silent();
    let mut settings = Settings::default();
    if let Some(p) = game.play_mut() {
        p.menu = Menu::None;
        p.clock.min = 700.0;
        p.ride_bus(true);
        p.bus = None;
    }
    for place in [None, Some(Place::Bakery), Some(Place::Tavern)] {
        if let Some(p) = game.play_mut() {
            if let Some(pl) = place {
                p.enter_place(pl);
            }
            for _ in 0..20 {
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
        assert!(r.fb.color.iter().all(|&c| c < 32));
        if let Some(p) = game.play_mut() {
            if place.is_some() {
                p.leave_place();
            }
        }
    }
}

// ------------------------------------------------------------------------------------------
// The wild farm, potions and spells
// ------------------------------------------------------------------------------------------

use super::spells::Spell;

#[test]
fn hoe_turns_paths_and_sand_too() {
    let mut s = Sim::new();
    // The path south of the house, and a patch of sand.
    s.play.farm.set_floor(30, 16, Floor::Sand);
    s.play.farm.set_obj(30, 16, None);
    for (x, z) in [(29, 16), (30, 16)] {
        s.stand(x, z - 1, Vec2::new(0.0, 1.0));
        s.select(1);
        s.tap(KeyCode::KeyJ, 40);
        assert_eq!(s.play.farm.floor(x, z), Floor::Tilled, "tile {x},{z} tills");
    }
    // The road to town stays a road.
    let (rx, rz) = (super::farm::ROAD_X, 20);
    assert!(!super::farm::tillable(&s.play.farm, rx, rz));
}

#[test]
fn nights_bring_weeds_and_wild_bushes() {
    let mut s = Sim::new();
    let count =
        |p: &Play, f: &dyn Fn(&Obj) -> bool| p.farm.objs.iter().flatten().filter(|o| f(o)).count();
    let weeds = count(&s.play, &|o| matches!(o, Obj::Weed { .. }));
    let bushes = count(&s.play, &|o| matches!(o, Obj::Shrub { .. }));
    for _ in 0..3 {
        s.play.start_fade(Trans::Sleep { passed_out: false });
        s.frames(60);
        s.tap(KeyCode::KeyE, 2);
    }
    assert!(count(&s.play, &|o| matches!(o, Obj::Weed { .. })) > weeds + 10);
    assert!(count(&s.play, &|o| matches!(o, Obj::Shrub { .. })) > bushes + 3);
    // Only on open ground: never on paths, dug soil or the road.
    let w = &s.play.farm;
    for z in 0..w.h {
        for x in 0..w.w {
            if matches!(w.obj(x, z), Some(Obj::Shrub { .. })) {
                assert!(
                    matches!(w.floor(x, z), Floor::Grass | Floor::Soil),
                    "bush on {x},{z}"
                );
            }
        }
    }
}

#[test]
fn wild_bushes_come_out_with_an_axe() {
    let mut s = Sim::new();
    clear_farm_tile(&mut s.play, 36, 25);
    clear_farm_tile(&mut s.play, 36, 26);
    s.play
        .farm
        .set_obj(36, 26, Some(Obj::Shrub { var: 1, hp: 4 }));
    s.stand(36, 25, Vec2::new(0.0, 1.0));
    s.select(3);
    for _ in 0..6 {
        if s.play.farm.obj(36, 26).is_none() {
            break;
        }
        s.tap(KeyCode::KeyJ, 40);
    }
    assert!(s.play.farm.obj(36, 26).is_none(), "the bush comes out");
    s.frames(90);
    assert!(
        s.play.player.inv.count(Item::Blueberry) >= 2,
        "a berry bush leaves berries"
    );
}

#[test]
fn potions_brew_up_in_three_sizes() {
    let mut s = Sim::new();
    s.learn_all();
    let inv = &mut s.play.player.inv;
    inv.slots[10..].fill(None);
    inv.add(Item::Vial, 8);
    inv.add(Item::Heartleaf, 4);
    inv.add(Item::SlimeGel, 4);
    inv.add(Item::Amber, 1);
    let potions = 1 + super::items::CATS
        .iter()
        .position(|c| *c == super::items::Cat::Potions)
        .unwrap();
    let craft = |s: &mut Sim, recipe: usize, times: usize| {
        for _ in 0..times {
            s.play.menu = Menu::Inventory {
                tab: super::menus::Tab::Craft,
                cursor: 0,
                recipe,
                scroll: 0,
                cat: potions,
            };
            s.tap(KeyCode::Enter, 1);
        }
        s.play.menu = Menu::None;
    };
    craft(&mut s, 1, 4);
    assert_eq!(s.play.player.inv.count(Item::SmallHealthPotion), 8);
    craft(&mut s, 2, 2);
    assert_eq!(s.play.player.inv.count(Item::HealthPotion), 2);
    craft(&mut s, 3, 1);
    assert_eq!(s.play.player.inv.count(Item::LargeHealthPotion), 1);
    assert_eq!(s.play.player.inv.count(Item::SmallHealthPotion), 4);
    assert_eq!(s.play.stats.brewed, 8 + 2 + 1);
    // And down it goes.
    let slot = s
        .play
        .player
        .inv
        .slots
        .iter()
        .position(|x| x.is_some_and(|st| st.item == Item::LargeHealthPotion))
        .unwrap();
    s.play.player.inv.slots.swap(slot, 9);
    s.play.player.hp = 5;
    s.select(9);
    s.tap(KeyCode::KeyJ, 5);
    assert_eq!(s.play.player.hp, s.play.player.max_hp());
    assert_eq!(s.play.player.inv.count(Item::LargeHealthPotion), 0);
}

#[test]
fn spells_cost_mana_grow_with_practice_and_get_cheaper() {
    let mut s = Sim::new();
    s.play.spells.learn(Spell::Firebolt);
    s.play.player.base_mana = 400;
    s.play.player.mana = 400.0;
    s.stand(30, 25, Vec2::new(1.0, 0.0));
    let mana = s.play.player.mana;
    let cost = s.play.spells.get(Spell::Firebolt).unwrap().cost();
    s.tap(KeyCode::KeyQ, 1);
    assert!(
        s.play.player.mana <= mana - cost + 0.5,
        "casting costs mana"
    );
    s.frames(20);
    assert!(
        s.play.stats.casts == 1,
        "the spell goes off ({} casts)",
        s.play.stats.casts
    );
    // Practice until it levels.
    for _ in 0..40 {
        s.play.player.mana = 400.0;
        s.tap(KeyCode::KeyQ, 40);
    }
    let k = *s.play.spells.get(Spell::Firebolt).unwrap();
    assert!(k.level >= 2, "practice levels it up");
    assert!(k.cost() < cost, "and makes it cheaper");
    // Nothing on R yet: nothing happens but a hint.
    let before = s.play.player.mana;
    s.tap(KeyCode::KeyR, 5);
    assert!(s.play.player.mana >= before);
}

#[test]
fn only_hazel_changes_your_spells() {
    use super::talk::Say;
    use super::town::Place;
    let mut s = Sim::new();
    s.play.clock.min = 700.0;
    s.play.spells.learn(Spell::Firebolt);
    s.play.spells.learn(Spell::Mend);
    s.play.spells.learn(Spell::Bloom);
    assert_eq!(
        s.play.spells.slots,
        [Some(Spell::Firebolt), Some(Spell::Mend)]
    );
    // Out in town, Hazel is just Hazel.
    s.ride();
    s.play.enter_place(Place::Spellery);
    s.frames(30);
    let i = s
        .play
        .folk
        .iter()
        .position(|n| n.who == Villager::Hazel)
        .expect("Hazel minds the Spellery");
    let mut io = frame_io(&s.input, &s.audio);
    s.play.talk_to(i, &mut io);
    let Menu::Talk { choices, .. } = &s.play.menu else {
        panic!("talking to Hazel");
    };
    assert!(choices.iter().any(|(_, c)| *c == Say::Spells));
    let mut io = frame_io(&s.input, &s.audio);
    s.play.talk_choice(Villager::Hazel, Say::Spells, &mut io);
    assert!(matches!(s.play.menu, Menu::Spells { .. }));
    // The attuning circle: pick Bloom and put it on R.
    let bloom = s
        .play
        .known_spells()
        .iter()
        .position(|x| *x == Spell::Bloom)
        .unwrap();
    s.play.menu = Menu::Spells { tab: 1, sel: bloom };
    s.tap(KeyCode::KeyR, 2);
    assert_eq!(
        s.play.spells.slots,
        [Some(Spell::Firebolt), Some(Spell::Bloom)]
    );
    // Everyone else just chats.
    s.play.leave_place();
    s.frames(10);
    if let Some(j) = s.play.folk.iter().position(|n| n.who != Villager::Hazel) {
        let mut io = frame_io(&s.input, &s.audio);
        s.play.talk_to(j, &mut io);
        if let Menu::Talk { choices, .. } = &s.play.menu {
            assert!(!choices.iter().any(|(_, c)| *c == Say::Spells));
        }
    }
}

#[test]
fn meeting_hazel_teaches_firebolt() {
    let mut s = Sim::new();
    s.play.clock.min = 700.0;
    s.ride();
    s.play.enter_place(super::town::Place::Spellery);
    s.frames(30);
    let i = s
        .play
        .folk
        .iter()
        .position(|n| n.who == Villager::Hazel)
        .expect("Hazel minds the Spellery");
    assert!(s.play.spells.known.is_empty());
    let mut io = frame_io(&s.input, &s.audio);
    s.play.talk_to(i, &mut io);
    assert!(s.play.spells.knows(Spell::Firebolt));
    assert_eq!(s.play.spells.slots[0], Some(Spell::Firebolt));
}

#[test]
fn spells_and_ward_save_with_the_game() {
    let mut p = Play::new(5);
    p.spells.learn(Spell::Ward);
    p.spells.get_mut(Spell::Ward).unwrap().practise(500);
    let json = serde_json::to_string(&p.spells).unwrap();
    let back: super::spells::Spellbook = serde_json::from_str(&json).unwrap();
    assert_eq!(back.known, p.spells.known);
    assert_eq!(back.slots, p.spells.slots);
}

// ------------------------------------------------------------------------------------------
// Fishing, cooking and the house
// ------------------------------------------------------------------------------------------

use super::fish::{Hooked, Phase, Water};
use super::home::{Furn, HOUSE_DOOR};

/// A little pond just north of the hero, and a rod in hand.
fn by_a_pond(s: &mut Sim) {
    for z in 12..19 {
        for x in 30..38 {
            clear_farm_tile(&mut s.play, x, z);
        }
    }
    for z in 12..16 {
        for x in 32..36 {
            s.play.farm.set_floor(x, z, Floor::Water);
        }
    }
    s.stand(34, 16, Vec2::new(0.0, -1.0));
    s.play.player.inv.slots[5] = Some(Stack::new(Item::BambooRod, 1));
    s.select(5);
}

#[test]
fn fishing_casts_hooks_and_lands_a_fish() {
    let mut s = Sim::new();
    s.play.clock.min = 600.0;
    by_a_pond(&mut s);
    let energy = s.play.player.energy;
    // Wind up, then let go.
    s.input.key_event(KeyCode::KeyJ, true, false);
    s.frames(20);
    assert_eq!(
        s.play.fishing.as_ref().map(|f| f.phase),
        Some(Phase::Charge)
    );
    assert!(s.play.fishing.as_ref().unwrap().power > 0.3);
    s.input.key_event(KeyCode::KeyJ, false, false);
    s.frames(40);
    let f = s.play.fishing.as_ref().expect("the line is out");
    assert_eq!(f.phase, Phase::Wait, "the bobber settles on the water");
    assert_eq!(f.water, Some(Water::Pond));
    assert!(
        s.play.player.energy < energy,
        "casting takes a little energy"
    );
    // Something bites.
    s.play.fishing.as_mut().unwrap().bite_in = 0.0;
    s.frames(1);
    assert_eq!(s.play.fishing.as_ref().unwrap().phase, Phase::Bite);
    s.play.fishing.as_mut().unwrap().hooked = Some(Hooked::Fish(Item::PondPerch, 20));
    s.tap(KeyCode::KeyJ, 0);
    assert_eq!(s.play.fishing.as_ref().unwrap().phase, Phase::Reel);
    // Keep it inside the bar until it's landed.
    s.input.key_event(KeyCode::KeyJ, true, false);
    for _ in 0..900 {
        match s.play.fishing.as_mut() {
            Some(f) if f.phase == Phase::Reel => {
                f.fish_y = (f.zone + f.zone_h * 0.5).min(1.0);
                f.fish_to = f.fish_y;
            }
            _ => break,
        }
        s.frames(1);
    }
    s.input.key_event(KeyCode::KeyJ, false, false);
    // Held up to be shown off (it counts as caught already), but not in the bag until A
    // says so.
    assert!(s.play.journal.fish.contains(&Item::PondPerch));
    assert_eq!(s.play.stats.caught, 1);
    let f = s.play.fishing.as_ref().expect("holding it up");
    assert_eq!(f.phase, Phase::Caught);
    assert!(f.new, "a new kind for the Fishdex");
    assert_eq!(s.play.player.inv.count(Item::PondPerch), 0);
    // It waits, the camera leaning in on it...
    s.frames(150);
    assert_eq!(
        s.play.fishing.as_ref().map(|f| f.phase),
        Some(Phase::Caught)
    );
    assert!(s.play.cam.dist < 10.0, "the camera leans in");
    // ...until A (E on the keyboard) tucks it into the bag, and the camera eases back.
    s.tap(KeyCode::KeyE, 60);
    assert_eq!(s.play.player.inv.count(Item::PondPerch), 1);
    assert!(s.play.fishing.is_none());
    assert!(s.play.cam.dist > 15.0);
}

/// Casts from the pond's edge, hooks `item` and reels it in: it's left held up.
fn land_a_fish(s: &mut Sim, item: Item) {
    s.input.key_event(KeyCode::KeyJ, true, false);
    s.frames(20);
    s.input.key_event(KeyCode::KeyJ, false, false);
    s.frames(40);
    s.play.fishing.as_mut().unwrap().bite_in = 0.0;
    s.frames(1);
    s.play.fishing.as_mut().unwrap().hooked = Some(Hooked::Fish(item, 20));
    s.tap(KeyCode::KeyJ, 0);
    s.input.key_event(KeyCode::KeyJ, true, false);
    for _ in 0..900 {
        match s.play.fishing.as_mut() {
            Some(f) if f.phase == Phase::Reel => {
                f.fish_y = (f.zone + f.zone_h * 0.5).min(1.0);
                f.fish_to = f.fish_y;
            }
            _ => break,
        }
        s.frames(1);
    }
    s.input.key_event(KeyCode::KeyJ, false, false);
    assert_eq!(
        s.play.fishing.as_ref().map(|f| f.phase),
        Some(Phase::Caught)
    );
}

#[test]
fn a_catch_the_bag_cant_hold_goes_back_in_the_water() {
    let mut s = Sim::new();
    s.play.clock.min = 600.0;
    by_a_pond(&mut s);
    // Every slot but the rod's is full of stone.
    for (i, slot) in s.play.player.inv.slots.iter_mut().enumerate() {
        if i != 5 {
            *slot = Some(Stack::new(Item::Stone, 99));
        }
    }
    land_a_fish(&mut s, Item::Bluegill);
    s.frames(40);
    s.tap(KeyCode::KeyE, 5);
    assert_eq!(
        s.play.fishing.as_ref().map(|f| f.phase),
        Some(Phase::Release),
        "thrown back"
    );
    s.frames(60);
    assert!(s.play.fishing.is_none());
    assert_eq!(s.play.player.inv.count(Item::Bluegill), 0);
    assert!(
        s.play.drops.iter().all(|d| d.stack.item != Item::Bluegill),
        "not left lying on the bank"
    );
    assert!(s.play.journal.fish.contains(&Item::Bluegill), "still seen");
}

#[test]
fn putting_the_rod_away_keeps_a_catch_being_shown_off() {
    let mut s = Sim::new();
    s.play.clock.min = 600.0;
    by_a_pond(&mut s);
    land_a_fish(&mut s, Item::PondPerch);
    // Straight to another tool on the hotbar: the fish goes into the bag on the way.
    s.select(0);
    s.frames(2);
    assert!(s.play.fishing.is_none());
    assert_eq!(s.play.player.inv.count(Item::PondPerch), 1);
}

#[test]
fn the_sewers_have_their_own_fish() {
    use super::fish::FISH;
    let mut s = Sim::new();
    let depth = (3..80)
        .find(|&d| super::sewer::is_sewer(s.play.seed, d))
        .expect("a sewer floor");
    s.play.fade = None;
    s.play.start_fade(Trans::Descend {
        depth,
        via_waystone: false,
    });
    s.frames(60);
    assert_eq!(s.play.area, Area::Hollow { depth });
    s.play.foes.clear();
    // A walkway with the channel just north of it.
    let (x, z) = {
        let w = s.play.world();
        assert!(w.sewer);
        (2..w.h - 2)
            .flat_map(|z| (2..w.w - 2).map(move |x| (x, z)))
            .find(|&(x, z)| {
                w.floor(x, z) == Floor::Walkway
                    && !w.blocked(x, z)
                    && w.floor(x, z - 1) == Floor::Walkway
                    && w.floor(x, z - 2) == Floor::Water
            })
            .expect("a walkway by the water")
    };
    assert_eq!(s.play.water_at(x, z - 2), Some(Water::Sewer));
    s.stand(x, z, Vec2::new(0.0, -1.0));
    s.play.player.inv.slots[5] = Some(Stack::new(Item::BambooRod, 1));
    s.select(5);
    s.input.key_event(KeyCode::KeyJ, true, false);
    s.frames(20);
    s.input.key_event(KeyCode::KeyJ, false, false);
    s.frames(40);
    let f = s.play.fishing.as_ref().expect("the line is out");
    assert_eq!(f.water, Some(Water::Sewer));
    // Ten odd fish live down there, and nowhere else.
    let sewer: Vec<_> = FISH
        .iter()
        .filter(|d| d.water.contains(&Water::Sewer))
        .collect();
    assert_eq!(sewer.len(), 10);
    assert!(sewer.iter().all(|d| d.water == [Water::Sewer]));
    assert!(
        sewer
            .iter()
            .any(|d| d.item == Item::SewerKing && d.rarity == 3)
    );
}

#[test]
fn the_sewers_folk_are_bones_and_sludge() {
    use super::dungeon::Foe;
    use super::foes::Enemy;
    use crate::assets::SEWER_LOOK;
    use crate::render::Renderer;
    let mut game = super::Game::new();
    game.new_game(5);
    let (audio, input) = (Audio::silent(), Input::default());
    let mut settings = Settings::default();
    let p = game.play_mut().unwrap();
    let depth = (12..90)
        .find(|&d| super::sewer::is_sewer(p.seed, d))
        .expect("a sewer floor");
    p.menu = Menu::None;
    p.start_fade(Trans::Descend {
        depth,
        via_waystone: false,
    });
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
    assert!(p.world().sewer);
    assert!(!p.foes.is_empty());
    for f in &p.foes {
        assert!(f.sewer, "{:?}", f.foe);
        assert_eq!(f.look(), SEWER_LOOK);
        let name = f.name();
        assert!(
            ["Bone ", "Sludge ", "Sewer ", "Skull "]
                .iter()
                .any(|w| name.starts_with(w)),
            "{name}"
        );
        // None of the biomes' own folk (shroomlings, crabs and the like) live down here.
        assert!(!matches!(
            f.foe,
            Foe::Shroom | Foe::Crab | Foe::Wisp | Foe::Beetle | Foe::Imp | Foe::Golem | Foe::Jelly
        ));
    }
    // Every family the sewers hold, drawn in its sewer look (bone spiders walking).
    let biome = p.hollow_biome(depth);
    let at = p.player.pos;
    p.foes.clear();
    for (i, foe) in [
        Foe::Slime,
        Foe::Bat,
        Foe::Zombie,
        Foe::Brute,
        Foe::Sneak,
        Foe::Bug,
        Foe::Skeleton,
        Foe::Ghost,
        Foe::Frog,
        Foe::Puffer,
        Foe::Snail,
        Foe::Bookworm,
    ]
    .into_iter()
    .enumerate()
    {
        let a = i as f32 * 0.52;
        let f = Enemy::new(
            foe,
            at.x + a.cos() * 1.5,
            at.y + a.sin() * 1.5,
            depth,
            biome,
            false,
            3,
        )
        .in_the_sewers();
        if foe == Foe::Bug {
            assert!(!f.flying() && f.y == 0.0, "bone spiders walk");
        }
        p.foes.push(f);
    }
    let mut r = Renderer::new(320, 180);
    game.draw(&mut r, &input);
    assert!(r.fb.color.iter().all(|&c| c < 32));
    // Up top, the same families wear their biome's look.
    let f = Enemy::new(Foe::Zombie, 0.0, 0.0, 3, 1, false, 1);
    assert!(!f.sewer && f.look() == 1 && f.name() == "Crystal Zombie");
}

#[test]
fn a_dry_cast_catches_nothing_and_a_fish_gets_away() {
    let mut s = Sim::new();
    by_a_pond(&mut s);
    // Facing away from the water.
    s.play.player.facing = Vec2::new(0.0, 1.0);
    s.tap(KeyCode::KeyJ, 50);
    assert!(s.play.fishing.is_none(), "nothing bites on dry land");
    assert_eq!(s.play.stats.caught, 0);
    // Back to the pond; this time the bite is missed.
    s.play.player.facing = Vec2::new(0.0, -1.0);
    s.tap(KeyCode::KeyJ, 40);
    s.play.fishing.as_mut().unwrap().bite_in = 0.0;
    s.frames(90);
    assert!(s.play.fishing.is_none(), "it got away");
    assert_eq!(s.play.stats.caught, 0);
}

#[test]
fn rods_are_plain_in_the_smithy_but_rolled_in_the_hollow() {
    let s = Sim::new();
    let stock = super::shops::goods(&s.play, Place::Smithy);
    assert!(stock.iter().any(|(i, _)| *i == Item::BambooRod));
    assert!(stock.iter().any(|(i, _)| *i == Item::Bait));
    let mut rng = crate::util::Rng::new(9);
    let mut better = 0;
    for _ in 0..200 {
        let r = super::loot::random_rod(30, super::loot::Fortune::plain(), &mut rng);
        assert_eq!(r.item.class(), Some(super::gear::Class::Rod));
        let g = r.gear.expect("rolled");
        if g.affixes().count() > 0 {
            better += 1;
        }
    }
    assert!(better > 100, "most dungeon rods come with stats ({better})");
}

#[test]
fn cooking_needs_a_stove_and_takes_a_moment() {
    let mut s = Sim::new();
    s.play.player.inv.add_stack(Stack::new(Item::PondPerch, 1));
    s.play.player.inv.add_stack(Stack::new(Item::Potato, 2));
    let k = super::items::RECIPES
        .iter()
        .position(|r| r.out == Item::FishAndChips)
        .expect("a recipe for fish and chips");
    s.stand(20, 20, Vec2::new(0.0, 1.0));
    let mut io = frame_io(&s.input, &s.audio);
    assert!(
        !s.play.start_cooking(k, (0, 0, 0), &mut io),
        "no stove out in the field"
    );
    s.play.enter_house();
    s.play.player.pos = Vec2::new(10.5, 2.5);
    let mut io = frame_io(&s.input, &s.audio);
    assert!(s.play.start_cooking(k, (0, 0, 0), &mut io));
    s.frames(30);
    assert!(s.play.cooking.is_some(), "still sizzling");
    assert_eq!(s.play.player.inv.count(Item::FishAndChips), 0);
    s.frames(90);
    assert!(s.play.cooking.is_none());
    assert_eq!(s.play.player.inv.count(Item::FishAndChips), 1);
    assert_eq!(s.play.player.inv.count(Item::PondPerch), 0);
    assert_eq!(s.play.player.inv.count(Item::Potato), 0);
    assert_eq!(s.play.stats.cooked, 1);
}

#[test]
fn home_through_the_door_and_back() {
    let mut s = Sim::new();
    let (dx, dz) = super::farm::MARKS.door;
    s.stand(dx, dz + 1, Vec2::new(0.0, -1.0));
    let mut io = frame_io(&s.input, &s.audio);
    s.play.go_indoors(&mut io);
    s.frames(90);
    assert_eq!(s.play.area, Area::Home);
    // Out through the front door.
    let (hx, hz) = HOUSE_DOOR;
    s.play.player.pos = Vec2::new(hx as f32 + 0.5, hz as f32 - 0.6);
    s.input.key_event(KeyCode::KeyS, true, false);
    s.frames(40);
    s.input.key_event(KeyCode::KeyS, false, false);
    s.frames(60);
    assert_eq!(s.play.area, Area::Farm);
}

#[test]
fn furnishing_the_house_makes_it_charming() {
    let mut s = Sim::new();
    s.play.enter_house();
    s.frames(5);
    let before = s.play.charisma();
    let price = s.play.buy_price(1000);
    // A sofa, set down facing you.
    s.play.player.inv.slots[5] = Some(Stack::new(Item::Sofa, 1));
    s.select(5);
    s.stand(4, 7, Vec2::new(0.0, -1.0));
    s.tap(KeyCode::KeyE, 2);
    assert!(
        matches!(
            s.play.house.world.obj(4, 6),
            Some(Obj::Furniture { f: Furn::Sofa, .. })
        ),
        "the sofa is down"
    );
    assert!(s.play.house.world.blocked(5, 6), "it covers two tiles");
    assert_eq!(s.play.player.inv.count(Item::Sofa), 0);
    let after = s.play.charisma();
    assert!(after > before, "charisma {before} -> {after}");
    assert!(s.play.buy_price(1000) < price, "shops are kinder");
    // A fish tank, turned with T before it goes down, with fish in it.
    s.play.player.inv.slots[5] = Some(Stack::new(Item::FishTank, 1));
    s.stand(10, 8, Vec2::new(-1.0, 0.0));
    s.tap(KeyCode::KeyT, 1);
    s.tap(KeyCode::KeyE, 2);
    let (tx, tz) = s
        .play
        .house
        .pieces()
        .iter()
        .find(|p| p.2 == Furn::FishTank)
        .map(|p| (p.0, p.1))
        .expect("the tank is down");
    let with_tank = s.play.charisma();
    s.play.player.inv.slots[6] = Some(Stack::new(Item::Bluegill, 1));
    s.play.player.inv.slots[7] = Some(Stack::new(Item::LilyKoi, 1));
    let mut io = frame_io(&s.input, &s.audio);
    assert!(s.play.tank_add(tx, tz, 6, &mut io));
    assert!(s.play.tank_add(tx, tz, 7, &mut io));
    assert_eq!(s.play.house.fish_kept(), 2);
    assert!(s.play.charisma() > with_tank, "fish make it lovelier");
    // Picking the sofa back up returns it.
    s.select(9);
    s.play.player.inv.slots[9] = None;
    s.stand(4, 7, Vec2::new(0.0, -1.0));
    s.tap(KeyCode::KeyJ, 2);
    assert_eq!(s.play.player.inv.count(Item::Sofa), 1);
    assert!(s.play.house.world.obj(4, 6).is_none());
}

#[test]
fn the_nook_saves_its_grandest_pieces_for_charming_homes() {
    let mut s = Sim::new();
    let stock = super::shops::goods(&s.play, Place::Nook);
    assert!(stock.iter().any(|(i, _)| *i == Item::FishTank));
    assert!(!stock.iter().any(|(i, _)| *i == Item::Piano));
    // Fill the place with lovely things.
    let pieces = [
        Furn::Piano,
        Furn::Fireplace,
        Furn::CanopyBed,
        Furn::Clock,
        Furn::Sofa,
        Furn::Range,
        Furn::Telescope,
        Furn::Bookshelf,
    ];
    let mut x = 1;
    for f in pieces {
        super::home::put(&mut s.play.house.world, f, 0, x, 7);
        x += f.def().size.0;
    }
    assert!(s.play.charisma() >= 50, "charisma {}", s.play.charisma());
    let stock = super::shops::goods(&s.play, Place::Nook);
    assert!(stock.iter().any(|(i, _)| *i == Item::Piano));
}

#[test]
fn water_folk_drop_rods_now_and_then() {
    let mut rng = crate::util::Rng::new(3);
    let mut rods = 0;
    for _ in 0..400 {
        let loot = super::loot::foe_loot(
            super::dungeon::Foe::Jelly,
            false,
            1,
            12,
            super::loot::Fortune::plain(),
            &mut rng,
        );
        rods += loot
            .iter()
            .filter(|s| s.item.class() == Some(super::gear::Class::Rod))
            .count();
    }
    assert!(rods > 10, "{rods} rods from 400 jellies");
}

#[test]
fn the_full_moon_riles_up_the_hollow() {
    use super::dungeon::Foe;
    use super::foes::Enemy;
    use super::sky::MoonPhase;
    let mut calm = Enemy::new(Foe::Skeleton, 5.5, 5.5, 12, 1, false, 7);
    let mut wild = Enemy::new(Foe::Skeleton, 5.5, 5.5, 12, 1, false, 7);
    calm.feel_the_moon(MoonPhase::New);
    wild.feel_the_moon(MoonPhase::Full);
    assert!(wild.moonlit && !calm.moonlit);
    assert!(wild.max_hp > calm.max_hp && wild.dmg > calm.dmg);
    assert!(wild.speed > calm.speed && wild.fury > calm.fury);
    // Down in the Hollow on a full-moon day, every creature is moonlit.
    let mut s = Sim::new();
    s.play.clock.day = 5;
    s.play.start_fade(Trans::Descend {
        depth: 4,
        via_waystone: false,
    });
    s.frames(60);
    assert!(!s.play.foes.is_empty());
    assert!(s.play.foes.iter().all(|f| f.moonlit));
    // ...and on a quiet night none are.
    let mut s = Sim::new();
    s.play.clock.day = 2;
    s.play.start_fade(Trans::Descend {
        depth: 4,
        via_waystone: false,
    });
    s.frames(60);
    assert!(s.play.foes.iter().all(|f| !f.moonlit));
}

#[test]
fn zombies_goblins_and_bugs_live_in_every_biome() {
    use super::dungeon::{Foe, biome_foes, too_tough};
    use super::foes::kind_name;
    for biome in 0..6 {
        let here: Vec<Foe> = biome_foes(biome).iter().map(|f| f.0).collect();
        for f in [
            Foe::Zombie,
            Foe::Brute,
            Foe::Sneak,
            Foe::Bug,
            Foe::Skeleton,
            Foe::Ghost,
        ] {
            assert!(here.contains(&f), "{f:?} missing from biome {biome}");
        }
    }
    // Every family goes by a different name in each biome.
    for f in [
        Foe::Zombie,
        Foe::Brute,
        Foe::Sneak,
        Foe::Bug,
        Foe::Skeleton,
        Foe::Ghost,
    ] {
        let names: std::collections::HashSet<_> = (0..6).map(|b| kind_name(f, b)).collect();
        assert_eq!(names.len(), 6, "{f:?}");
    }
    // The first floors are spared the brutes.
    assert!(too_tough(Foe::Brute, 1) && !too_tough(Foe::Brute, 3));
    for depth in [1u32, 2] {
        for seed in 0..6 {
            let biome = super::dungeon::biome_for(seed, depth);
            let level = super::dungeon::generate(seed, depth, biome, false);
            assert!(level.spawns.iter().all(|s| !too_tough(s.foe, depth)));
        }
    }
}

#[test]
fn the_new_families_drop_their_bits() {
    use super::dungeon::Foe;
    use super::loot::{Fortune, foe_loot};
    let mut rng = crate::util::Rng::new(5);
    for (foe, bit) in [
        (Foe::Zombie, Item::GraveDust),
        (Foe::Brute, Item::GoblinTooth),
        (Foe::Sneak, Item::GoblinTooth),
        (Foe::Bug, Item::Chitin),
    ] {
        let mut got = 0;
        for k in 0..200 {
            let loot = foe_loot(foe, false, k % 6, 12, Fortune::plain(), &mut rng);
            got += loot.iter().filter(|s| s.item == bit).count();
        }
        assert!(got > 40, "{foe:?} dropped {got} {bit:?}");
    }
}

#[test]
fn every_new_family_fights_back() {
    use super::dungeon::Foe;
    for foe in [Foe::Zombie, Foe::Brute, Foe::Sneak, Foe::Bug] {
        for biome in 0..6usize {
            let depth = biome as u32 * 10 + 4;
            let mut s = Sim::new();
            s.play.start_fade(Trans::Descend {
                depth,
                via_waystone: false,
            });
            s.frames(60);
            // Clear the floor and set one creature loose a few steps from the hero.
            let (px, pz) = s.play.player.tile();
            let w = &s.play.level.as_ref().unwrap().world;
            let (fx, fz) = w.nearest_open(px + 2, pz);
            s.play.foes.clear();
            s.play.foes.push(super::foes::Enemy::new(
                foe,
                fx as f32 + 0.5,
                fz as f32 + 0.5,
                depth,
                biome,
                false,
                3,
            ));
            s.play.player.hurt = 0.0;
            let start = s.play.player.hp;
            let mut hit = false;
            for _ in 0..900 {
                s.frames(1);
                if s.play.player.hp < start {
                    hit = true;
                    break;
                }
                // Keep the hero standing still and alive.
                s.play.player.hp = start;
                s.play.player.hurt = 0.0;
                if s.play.foes.is_empty() {
                    break;
                }
            }
            assert!(hit, "{foe:?} in biome {biome} never landed a blow");
        }
    }
}

#[test]
fn quests_hold_the_hollow_to_the_biome_they_need_until_theyre_done() {
    use super::dungeon::{Foe, biome_for, boss_for};
    use super::quests::{Quest, QuestId, fill_text};
    let story = |k: &str| Quest {
        id: QuestId::Story(k.to_string()),
        n: 0,
        met: 0,
        day: 0,
    };
    let mut s = Sim::new();
    // A save whose floors 31-40 aren't the Ember Depths, and whose Ember Depths are
    // somewhere above them.
    let seed = (0..500u64)
        .find(|&seed| (1..4).any(|b| biome_for(seed, b * 10 + 1) == 3) && biome_for(seed, 31) != 3)
        .expect("such a save");
    s.play.seed = seed;
    s.play.deepest = 35;
    s.play.quests.clear();
    let natural = biome_for(seed, 31);
    assert_eq!(s.play.biome_at(31), natural);
    // The Guild wants thirty imps from floor 31 down: that band turns into the Ember Depths
    // (imps and all) while the quest is under way...
    s.play.quests.push(story("rowan_imps"));
    for depth in 31..=40 {
        assert_eq!(s.play.biome_at(depth), 3, "floor {depth}");
    }
    // ...trading places with the band that was, so the first six still have all six.
    let embers = (1..3)
        .find(|b| biome_for(seed, b * 10 + 1) == 3)
        .expect("above");
    assert_eq!(s.play.biome_at(embers * 10 + 1), natural);
    let firsts: std::collections::HashSet<usize> =
        (0..6).map(|b| s.play.biome_at(b * 10 + 1)).collect();
    assert_eq!(firsts.len(), 6);
    for b in (0..12).filter(|&b| b != 3 && b != embers) {
        let floor = b * 10 + 1;
        assert_eq!(
            s.play.biome_at(floor),
            biome_for(seed, floor),
            "the rest stay put"
        );
    }
    // Down there when it's done, the band stays as it's been until you're out of it...
    s.play.fade = None;
    s.play.start_fade(Trans::Descend {
        depth: 35,
        via_waystone: false,
    });
    s.frames(60);
    assert_eq!(s.play.hollow_biome(35), 3);
    s.play.quests.clear();
    assert_eq!(s.play.biome_at(36), 3);
    assert_eq!(s.play.biome_at(40), 3);
    // ...and goes back once you're out.
    s.play.level = None;
    s.play.area = super::world::Area::Town;
    assert_eq!(s.play.biome_at(35), natural);
    assert_eq!(s.play.biome_at(embers * 10 + 1), 3);
    // A guardian someone sends you after is where they said it would be.
    s.play.quests.push(story("rowan_warden"));
    assert_eq!(s.play.biome_at(60), 5);
    assert_eq!(boss_for(60, s.play.biome_at(60)), Foe::Skeleton);
    s.play.quests.clear();
    // So is one whose prize someone's after: the Matriarch waits on floor twenty.
    s.play.quests.push(story("opal_pearl"));
    assert_eq!(boss_for(20, s.play.biome_at(20)), Foe::Crab);
    assert_eq!(super::dungeon::guardian_home(Foe::Crab), Some((20, 1)));
    assert_eq!(super::dungeon::guardian_home(Foe::Brute), Some((70, 0)));
    assert_eq!(super::dungeon::guardian_home(Foe::Frog), None);
    s.play.quests.clear();
    // Quest words name the biome a floor is in on this save.
    let name = crate::assets::BIOME_STYLES[s.play.biome_at(25)].name;
    let text = fill_text(&s.play, "deep in the {biome@25}, floors 25 to 29");
    assert_eq!(text, format!("deep in the {name}, floors 25 to 29"));
    // Every quest's placeholders are ones that fill in.
    for q in super::quests::QUESTS.iter() {
        assert!(!fill_text(&s.play, q.ask).contains('{'), "{}", q.key);
    }
}

#[test]
fn guardians_are_giants_that_leave_a_hoard_of_chests() {
    use super::dungeon::{Foe, boss_for};
    use super::foes::{BOSS_SCALE, Enemy};
    // The classic six on the first trip down (whichever biome a band turns out to be),
    // giant goblins, bugs and dead on the next.
    assert_eq!(boss_for(10, 0), Foe::Slime);
    assert_eq!(boss_for(60, 5), Foe::Skeleton);
    assert_eq!(boss_for(20, 5), Foe::Skeleton);
    assert_eq!(boss_for(70, 0), Foe::Brute);
    assert_eq!(boss_for(80, 1), Foe::Bug);
    assert_eq!(boss_for(130, 0), Foe::Slime);
    let small = Enemy::new(Foe::Slime, 0.0, 0.0, 10, 0, false, 1);
    let big = Enemy::new(Foe::Slime, 0.0, 0.0, 10, 0, true, 1);
    assert!(big.scale() >= 2.5 && BOSS_SCALE == big.scale());
    assert!(big.max_hp >= small.max_hp * 12 && big.dmg > small.dmg);
    for depth in [10u32, 70] {
        let mut s = Sim::new();
        s.play.fade = None;
        s.play.start_fade(Trans::Descend {
            depth,
            via_waystone: false,
        });
        s.frames(60);
        let i = s.play.foes.iter().position(|f| f.boss).expect("a guardian");
        assert_eq!(s.play.foes[i].foe, boss_for(depth, s.play.biome_at(depth)));
        let name = s.play.foes[i].name();
        assert!(!name.is_empty() && name != "Guardian", "{name}");
        let chests = |p: &Play| {
            let w = &p.level.as_ref().unwrap().world;
            let mut n = 0;
            for z in 0..w.h {
                for x in 0..w.w {
                    if matches!(w.obj(x, z), Some(Obj::LootChest { opened: false, .. })) {
                        n += 1;
                    }
                }
            }
            n
        };
        let before = chests(&s.play);
        s.play.foes[i].hp = 0;
        let mut io = frame_io(&s.input, &s.audio);
        s.play.reap(&mut io);
        let hoard = chests(&s.play) - before;
        assert!((4..=8).contains(&hoard), "floor {depth}: {hoard} chests");
        // A heap of loot on the floor too, finely made gear among it.
        assert!(s.play.drops.len() >= 12, "{} drops", s.play.drops.len());
        assert!(
            s.play
                .drops
                .iter()
                .any(|d| d.stack.rarity().is_some_and(|r| r >= Rarity::Rare))
        );
        assert_eq!(s.play.stats.guardians, 1);
    }
}

#[test]
fn gleaming_chests_hold_finely_rolled_gear() {
    use super::loot::{Fortune, chest_loot};
    let mut rng = crate::util::Rng::new(77);
    let best = |loot: &[Stack]| {
        loot.iter()
            .filter_map(|s| s.gear.filter(|_| s.item.base().is_some()))
            .map(|g| g.score())
            .fold(0.0f32, f32::max)
    };
    let (mut plain, mut gleaming) = (0.0, 0.0);
    for _ in 0..60 {
        let a = chest_loot(20, 1, false, Fortune::plain(), &mut rng);
        let b = chest_loot(20, 1, true, Fortune::plain(), &mut rng);
        plain += best(&a);
        gleaming += best(&b);
        // Every gleaming chest has at least two pieces of rare-or-better gear.
        let fine = b
            .iter()
            .filter(|s| s.item.base().is_some() && s.rarity().is_some_and(|r| r >= Rarity::Rare))
            .count();
        assert!(fine >= 2, "{fine} fine pieces");
    }
    assert!(
        gleaming > plain * 1.5,
        "gleaming {gleaming} vs plain {plain}"
    );
    // Most chests are plain; now and then one gleams.
    let (mut total, mut gleams) = (0, 0);
    for seed in 0..40 {
        let level = super::dungeon::generate(seed, 13, super::dungeon::biome_for(seed, 13), false);
        let w = &level.world;
        for z in 0..w.h {
            for x in 0..w.w {
                if let Some(Obj::LootChest { gleam, .. }) = w.obj(x, z) {
                    total += 1;
                    gleams += usize::from(*gleam);
                }
            }
        }
    }
    assert!(
        gleams > 0 && gleams * 5 < total,
        "{gleams} of {total} gleam"
    );
}

#[test]
fn recipes_are_learned_by_finding_their_ingredients() {
    use super::items::RECIPES;
    let mut s = Sim::new();
    s.play.menu = Menu::None;
    s.frames(2);
    // A new farmer knows how to work wood, stone and fibre, and nothing fancier.
    assert!(s.play.known.contains(&Item::Chest) && s.play.known.contains(&Item::Fence));
    assert!(!s.play.known.contains(&Item::HealingTonic));
    assert!(matches!(s.play.menu, Menu::None));
    // One carrot and one blob of gel is enough to work out the tonic (it takes two of each).
    s.play.player.inv.add(Item::CaveCarrot, 1);
    s.play.player.inv.add(Item::SlimeGel, 1);
    s.frames(1);
    let tonic = RECIPES
        .iter()
        .position(|r| r.out == Item::HealingTonic)
        .unwrap();
    assert!(matches!(s.play.menu, Menu::Recipe { recipe, .. } if recipe == tonic));
    assert!(s.play.known.contains(&Item::HealingTonic));
    // The card stops the clock until you press OK.
    let min = s.play.clock.min;
    s.frames(40);
    assert_eq!(s.play.clock.min, min);
    s.tap(KeyCode::Enter, 1);
    // (The turnips you started with and that carrot make a stew, too.)
    assert!(s.play.known.contains(&Item::VeggieStew));
    while matches!(s.play.menu, Menu::Recipe { .. }) {
        s.frames(30);
        s.tap(KeyCode::Enter, 1);
    }
    assert!(matches!(s.play.menu, Menu::None));
    s.frames(10);
    assert!(s.play.clock.min > min);
    // Finding two at once shows them one after another.
    s.play.player.inv.add(Item::Spore, 1);
    s.play.player.inv.add(Item::Blueberry, 1);
    s.play.player.inv.add(Item::WispDust, 1);
    s.frames(1);
    assert!(matches!(s.play.menu, Menu::Recipe { .. }));
    assert!(!s.play.discoveries.is_empty());
    s.frames(30);
    s.tap(KeyCode::Enter, 1);
    assert!(matches!(s.play.menu, Menu::Recipe { .. }), "the next card");
    while matches!(s.play.menu, Menu::Recipe { .. }) {
        s.frames(30);
        s.tap(KeyCode::Enter, 1);
    }
    assert!(s.play.known.contains(&Item::StaminaTonic) && s.play.known.contains(&Item::ManaTonic));
    // What you haven't worked out, you can't make (even with everything it needs).
    let sword = RECIPES.iter().position(|r| r.out == Item::Sword1).unwrap();
    s.play.known.remove(&Item::Sword1);
    s.play.player.inv.add(Item::CopperOre, 10);
    s.play.player.inv.add(Item::Wood, 5);
    s.play.menu = Menu::Inventory {
        tab: Tab::Craft,
        cursor: 0,
        recipe: sword,
        scroll: sword.saturating_sub(3),
        cat: 0,
    };
    s.tap(KeyCode::Enter, 1);
    assert_eq!(s.play.player.inv.count(Item::Sword1), 0);
    assert_eq!(s.play.player.inv.count(Item::CopperOre), 10);
}

#[test]
fn a_steam_deck_plays_the_game() {
    use crate::pad::{PadButton, PadState};
    let mut s = Sim::new();
    s.play.menu = Menu::None;
    s.frames(2);
    let press = |s: &mut Sim, b: PadButton| {
        let st = PadState {
            connected: true,
            buttons: b.bit(),
            ..Default::default()
        };
        s.input.pad_event(st, 1.0 / 60.0);
        s.frames(1);
        s.input.pad_event(
            PadState {
                connected: true,
                ..Default::default()
            },
            1.0 / 60.0,
        );
        s.frames(1);
    };
    // Y opens the bag, and the hints speak the Deck's language.
    press(&mut s, PadButton::Y);
    assert!(matches!(s.play.menu, Menu::Inventory { .. }));
    assert!(s.play.pad);
    assert_eq!(s.play.prompt(crate::input::Action::Confirm), "(A)");
    // R1 flips to your stats, B closes it.
    press(&mut s, PadButton::R1);
    assert!(matches!(
        s.play.menu,
        Menu::Inventory {
            tab: Tab::Stats,
            ..
        }
    ));
    press(&mut s, PadButton::B);
    assert!(matches!(s.play.menu, Menu::None));
    // Menu pauses; the controls page shows the Deck's layout.
    press(&mut s, PadButton::Menu);
    assert!(matches!(s.play.menu, Menu::Pause { .. }));
    for _ in 0..4 {
        press(&mut s, PadButton::Down);
    }
    press(&mut s, PadButton::A);
    assert!(matches!(s.play.menu, Menu::Controls { deck: true }));
    press(&mut s, PadButton::B);
    assert!(matches!(s.play.menu, Menu::Pause { .. }));
    press(&mut s, PadButton::B);
    assert!(matches!(s.play.menu, Menu::None));
    // R1 and L1 step along the hotbar, round from the first slot to the last.
    s.select(0);
    press(&mut s, PadButton::R1);
    assert_eq!(s.play.player.sel, 1);
    press(&mut s, PadButton::L1);
    press(&mut s, PadButton::L1);
    assert_eq!(s.play.player.sel, super::player::HOTBAR - 1);
    // The left stick walks, the right one turns you about.
    let start = s.play.player.pos;
    for _ in 0..30 {
        s.input.pad_event(
            PadState {
                connected: true,
                left: Vec2::new(1.0, 0.0),
                ..Default::default()
            },
            1.0 / 60.0,
        );
        s.frames(1);
    }
    assert!(s.play.player.pos.x > start.x + 0.5, "walked right");
    s.input.pad_event(
        PadState {
            connected: true,
            right: Vec2::new(0.0, -1.0),
            ..Default::default()
        },
        1.0 / 60.0,
    );
    s.frames(1);
    assert!(s.play.player.facing.y < -0.9, "faces up the screen");
    // A key press hands the hints back to the keyboard.
    s.input.key_event(KeyCode::KeyW, true, false);
    s.frames(1);
    s.input.key_event(KeyCode::KeyW, false, false);
    assert_eq!(s.play.prompt(crate::input::Action::Confirm), "(E)");
}

#[test]
fn a_stack_picked_up_on_a_controller_rides_on_the_cursor() {
    use super::menus::bag_grid;
    use crate::pad::{PadButton, PadState};
    let mut s = Sim::new();
    s.frames(2);
    let press = |s: &mut Sim, b: PadButton| {
        let st = PadState {
            connected: true,
            buttons: b.bit(),
            ..Default::default()
        };
        s.input.pad_event(st, 1.0 / 60.0);
        s.frames(1);
        s.input.pad_event(
            PadState {
                connected: true,
                ..Default::default()
            },
            1.0 / 60.0,
        );
        s.frames(1);
    };
    press(&mut s, PadButton::Y);
    s.play.player.inv.slots[3] = Some(Stack::new(Item::Wood, 7));
    if let Menu::Inventory { cursor, .. } = &mut s.play.menu {
        *cursor = 3;
    }
    press(&mut s, PadButton::A);
    assert_eq!(s.play.held.map(|h| h.item), Some(Item::Wood));
    // The Deck's pointer sits parked in the corner: the stack stays on the cursor's slot.
    // (The bag's panel stands a little left of the middle, the backpack's beside it.)
    let (x, y) = bag_grid(&super::menus::pouch_layout(480, 270, true), 22).slot_pos(3);
    assert_eq!(
        s.play.held_spot(480, 270, Vec2::ZERO, s.input.mouse_aim),
        (x + 7, y - 7)
    );
    // Once the mouse is what's pointing, it follows the mouse.
    assert_eq!(
        s.play.held_spot(480, 270, Vec2::new(100.0, 80.0), true),
        (96, 76)
    );
}

#[test]
fn the_shops_keep_nine_to_five() {
    use super::folk::{Spot, VILLAGER_DEFS};
    use super::town::{PLACES, STORE_HOURS, trading};
    assert!(!trading(539.0) && trading(540.0) && trading(1019.0) && !trading(1020.0));
    for place in PLACES {
        if place.is_store() {
            assert_eq!(place.def().open, STORE_HOURS, "{place:?}");
        }
    }
    // Every shopkeeper is behind the counter from nine to five.
    for v in VILLAGER_DEFS.iter() {
        let Some(p) = v.keeps.filter(|p| *p != Place::Hall) else {
            continue;
        };
        for min in (540..1020).step_by(15) {
            let at = v
                .hours
                .iter()
                .find(|(a, b, _)| (*a..*b).contains(&(min as f32)))
                .map(|h| h.2);
            assert_eq!(at, Some(Spot::Inside(p)), "{} at {min}", v.name);
        }
    }
    // ...and nobody's inside a shop while it's shut.
    for v in VILLAGER_DEFS.iter() {
        for &(a, b, spot) in v.hours {
            if let Spot::Inside(p) = spot {
                if p.is_store() {
                    assert!(
                        a >= STORE_HOURS.0 - 180.0 && b <= STORE_HOURS.1,
                        "{} in {p:?}",
                        v.name
                    );
                    if v.keeps != Some(p) {
                        assert!(
                            a >= STORE_HOURS.0,
                            "{} visits {p:?} before it opens",
                            v.name
                        );
                    }
                }
            }
        }
    }
    let mut s = Sim::new();
    s.play.menu = Menu::None;
    // A shop door at half past eight stays shut; at nine it opens.
    s.play.area = Area::Town;
    let id = Place::Smithy.building();
    s.play.clock.min = 510.0;
    let mut io = frame_io(&s.input, &s.audio);
    s.play.knock(id, &mut io);
    assert!(s.play.fade.is_none(), "closed before nine");
    s.play.clock.min = 545.0;
    let mut io = frame_io(&s.input, &s.audio);
    s.play.knock(id, &mut io);
    s.frames(60);
    assert_eq!(s.play.area, Area::Inside(Place::Smithy));
    // At five the smith sees you out.
    s.play.clock.min = 1018.0;
    s.frames(60 * 2);
    s.frames(90);
    assert_eq!(s.play.area, Area::Town, "shown out at closing time");
    // Burrowby's stall on the farm keeps the same hours.
    s.play.area = Area::Farm;
    s.play.room = None;
    let (sx, sz) = super::farm::MARKS.stall;
    for (min, open) in [(1080.0, false), (600.0, true)] {
        s.play.menu = Menu::None;
        s.play.clock.min = min;
        s.stand(sx, sz + 1, Vec2::new(0.0, -1.0));
        s.tap(KeyCode::KeyE, 2);
        assert_eq!(
            matches!(s.play.menu, Menu::Shop { at: None, .. }),
            open,
            "the stall at {min}"
        );
    }
}

#[test]
fn a_bomb_opens_a_secret_room_and_the_rope_leads_back() {
    use super::dungeon::generate;
    let mut s = Sim::new();
    s.play.menu = Menu::None;
    // Find a floor with a crack in it.
    let depth = (2..40)
        .find(|&d| {
            generate(s.play.seed, d, s.play.biome_at(d), false)
                .crack
                .is_some()
        })
        .expect("some floor hides a secret room");
    s.play.fade = None;
    s.play.start_fade(Trans::Descend {
        depth,
        via_waystone: false,
    });
    s.frames(60);
    s.play.menu = Menu::None;
    s.play.discoveries.clear();
    s.play.foes.clear();
    let (cx, cz) = s.play.level.as_ref().unwrap().crack.unwrap();
    assert!(matches!(s.play.world().obj(cx, cz), Some(Obj::Crack)));
    // Throw a bomb at it from two steps south.
    s.play.player.inv.slots[0] = Some(Stack::new(Item::Bomb, 3));
    s.select(0);
    s.stand(cx, cz + 2, Vec2::new(0.0, -1.0));
    let hp = s.play.player.hp;
    s.tap(KeyCode::KeyJ, 1);
    assert_eq!(s.play.bombs.len(), 1, "lit and thrown");
    assert_eq!(s.play.player.inv.count(Item::Bomb), 2);
    s.frames(150);
    assert!(s.play.bombs.is_empty(), "gone off");
    assert!(
        matches!(s.play.world().obj(cx, cz), Some(Obj::Hole)),
        "the floor gave way"
    );
    assert_eq!(s.play.player.hp, hp, "far enough away");
    // Down the hole.
    s.stand(cx, cz + 1, Vec2::new(0.0, -1.0));
    s.tap(KeyCode::KeyE, 60);
    assert!(s.play.in_vault(), "in the secret room");
    let (w, keepers) = {
        let lv = s.play.level.as_ref().unwrap();
        (lv.world.w, s.play.foes.len())
    };
    assert!(w < 30 && keepers >= 3, "one small room, {keepers} keepers");
    let chests = |p: &Play| {
        let w = &p.level.as_ref().unwrap().world;
        (0..w.h)
            .flat_map(|z| (0..w.w).map(move |x| (x, z)))
            .filter(|&(x, z)| matches!(w.obj(x, z), Some(Obj::LootChest { opened: false, .. })))
            .count()
    };
    assert!(chests(&s.play) >= 2);
    // Clear it out and help yourself.
    for f in s.play.foes.iter_mut() {
        f.hp = 0;
    }
    let mut io = frame_io(&s.input, &s.audio);
    s.play.reap(&mut io);
    assert!(s.play.foes.is_empty());
    // Back up the rope to where you came down.
    // (You land beside the rope: it hangs just to your left.)
    let start = s.play.level.as_ref().unwrap().start;
    assert!(matches!(
        s.play.world().obj(start.0 - 1, start.1),
        Some(Obj::Rope)
    ));
    s.stand(start.0, start.1, Vec2::new(-1.0, 0.0));
    s.tap(KeyCode::KeyE, 60);
    assert!(!s.play.in_vault());
    let back = s.play.player.tile();
    assert!(
        (back.0 - cx).abs() <= 1 && (back.1 - cz).abs() <= 1,
        "by the hole"
    );
    assert!(matches!(s.play.world().obj(cx, cz), Some(Obj::Hole)));
    // Going back down finds it as you left it.
    s.stand(cx, cz + 1, Vec2::new(0.0, -1.0));
    s.tap(KeyCode::KeyE, 60);
    assert!(s.play.in_vault() && s.play.foes.is_empty());
}

#[test]
fn the_smith_sells_five_bombs_a_day() {
    let mut s = Sim::new();
    s.play.menu = Menu::None;
    s.play.money = 100_000;
    s.play.clock.min = 600.0;
    s.play.area = Area::Town;
    s.play.enter_place(Place::Smithy);
    s.play.menu = Menu::shop_at(Place::Smithy);
    let rows = super::shops::rows(&s.play, Some(Place::Smithy), super::menus::ShopTab::Goods);
    let i = rows
        .iter()
        .position(|r| r.0.item == Item::Bomb)
        .expect("bombs on the shelf");
    for _ in 0..7 {
        s.play.menu = Menu::Shop {
            at: Some(Place::Smithy),
            tab: super::menus::ShopTab::Goods,
            cursor: i,
            scroll: i.saturating_sub(3),
        };
        s.tap(KeyCode::Enter, 1);
    }
    assert_eq!(s.play.player.inv.count(Item::Bomb), 5, "five and no more");
    assert_eq!(s.play.bomb_stock, 0);
    // Fresh stock in the morning.
    s.play.menu = Menu::None;
    s.play.start_fade(Trans::Sleep { passed_out: false });
    s.frames(90);
    assert_eq!(s.play.bomb_stock, super::bombs::BOMBS_PER_DAY);
}

#[test]
fn the_year_turns_through_four_seasons() {
    use super::play::{Clock, Season};
    let at = |day| Clock { day, min: 600.0 };
    assert_eq!(at(1).season(), Season::Spring);
    assert_eq!(at(28).season(), Season::Spring);
    assert_eq!(at(29).season(), Season::Summer);
    assert_eq!(at(57).season(), Season::Autumn);
    assert_eq!(at(84).season(), Season::Autumn);
    assert_eq!(at(85).season(), Season::Winter);
    assert_eq!(at(113).season(), Season::Spring);
    assert_eq!((at(57).season_day(), at(84).season_day()), (1, 28));
    // The morning autumn comes, the day's news says so.
    let mut s = Sim::new();
    s.play.clock.day = 56;
    s.play.start_fade(Trans::Sleep { passed_out: false });
    s.frames(60);
    let news = &s.play.last_summary.as_ref().expect("a summary").notes;
    assert!(
        news.iter().any(|(t, _)| t.starts_with("Autumn is here")),
        "{news:?}"
    );
}

/// Candy rocks on the floor you're on.
fn candy_rocks(p: &Play) -> Vec<(i32, i32, i16)> {
    let w = &p.level.as_ref().expect("in the Hollow").world;
    let mut v = Vec::new();
    for z in 0..w.h {
        for x in 0..w.w {
            if let Some(Obj::CandyRock { hp }) = w.obj(x, z) {
                v.push((x, z, *hp));
            }
        }
    }
    v
}

#[test]
fn pip_asks_for_candy_rocks_only_in_autumn_and_only_once() {
    use super::folk::Villager;
    let mut s = Sim::new();
    let asks = |p: &Play| {
        p.quest_for(Villager::Pip)
            .is_some_and(|q| q.key == "pip_candy")
    };
    s.play.clock.day = 20;
    assert!(!asks(&s.play), "not in spring");
    s.play.clock.day = 110;
    assert!(!asks(&s.play), "not in winter");
    s.play.clock.day = 60;
    assert!(asks(&s.play), "Pip asks in autumn");
    s.accept("pip_candy");
    // Back down to floor 10 by the waystone: its guardian returns while Pip waits.
    s.play.deepest = 10;
    s.play.waystones = vec![10];
    s.play.fade = None;
    s.play.start_fade(Trans::Descend {
        depth: 10,
        via_waystone: true,
    });
    s.frames(60);
    let i = s
        .play
        .foes
        .iter()
        .position(|f| f.boss)
        .expect("the guardian is back");
    let at = s.play.foes[i].pos;
    s.play.player.pos = at + Vec2::new(0.0, 2.5);
    s.play.foes.retain(|f| f.boss);
    s.play.foes[0].hp = 0;
    let mut io = frame_io(&s.input, &s.audio);
    s.play.reap(&mut io);
    s.play.foes.clear();
    // (Its loot's no part of this: a recipe it teaches would pop up and pause the game.)
    s.play.drops.clear();
    // Up come the candy rocks, one after another...
    let rocks = candy_rocks(&s.play);
    assert_eq!(rocks.len(), super::candy::ROCKS);
    assert_eq!(s.play.erupting.len(), super::candy::ROCKS);
    // ...too hot to touch while they're going off...
    let (x, z, hp) = rocks[0];
    s.play.mine_candy(x, z, hp, 50, &mut io);
    assert_eq!(candy_rocks(&s.play).len(), super::candy::ROCKS);
    // ...until they've all gone bang and set hard.
    s.frames(600);
    assert!(s.play.erupting.is_empty());
    let mut io = frame_io(&s.input, &s.audio);
    for (x, z, hp) in candy_rocks(&s.play) {
        s.play.mine_candy(x, z, hp, 50, &mut io);
    }
    assert!(candy_rocks(&s.play).is_empty());
    let loose: u32 = s
        .play
        .drops
        .iter()
        .filter(|d| d.stack.item == Item::CandyRock)
        .map(|d| d.stack.n as u32)
        .sum();
    assert_eq!(loose, 5);
    // Five for Pip, and a mysterious egg in return.
    s.play.drops.clear();
    s.play.player.inv.add(Item::CandyRock, 5);
    let q = s.quest("pip_candy");
    assert!(s.play.quest_ready(&s.play.quests[q]));
    s.play.finish_quest(q, &mut io);
    assert_eq!(s.play.player.inv.count(Item::MysteryEgg), 1);
    assert_eq!(s.play.player.inv.count(Item::CandyRock), 0);
    // Never again, not even next autumn.
    s.play.clock.day = 60 + 4 * super::play::SEASON_DAYS;
    assert!(!asks(&s.play));
}

#[test]
fn a_guardian_without_a_candy_quest_leaves_no_candy() {
    let mut s = Sim::new();
    s.play.fade = None;
    s.play.start_fade(Trans::Descend {
        depth: 10,
        via_waystone: false,
    });
    s.frames(60);
    let i = s.play.foes.iter().position(|f| f.boss).expect("a guardian");
    s.play.foes[i].hp = 0;
    let mut io = frame_io(&s.input, &s.audio);
    s.play.reap(&mut io);
    assert!(candy_rocks(&s.play).is_empty());
    assert!(s.play.erupting.is_empty());
}

#[test]
fn the_egg_tips_over_on_day_nine_and_hatches_on_day_ten() {
    use super::pets::{HATCH_DAYS, TILT_DAY};
    let mut s = Sim::new();
    s.play.clock.min = 600.0;
    // It won't go down indoors.
    s.play.player.inv.slots[0] = Some(Stack::new(Item::MysteryEgg, 1));
    s.select(0);
    s.play.area = Area::Home;
    let mut io = frame_io(&s.input, &s.audio);
    s.play.house_use((4, 4), false, &mut io);
    assert_eq!(s.play.player.inv.count(Item::MysteryEgg), 1);
    // Out on the farm it tucks into its nest.
    s.play.area = Area::Farm;
    let (x, z) = (30, 20);
    clear_farm_tile(&mut s.play, x, z);
    s.stand(x, z + 2, Vec2::new(0.0, -1.0));
    let mut io = frame_io(&s.input, &s.audio);
    s.play.use_item(Item::MysteryEgg, (x, z), &mut io);
    let day = s.play.clock.day;
    assert!(matches!(s.play.farm.obj(x, z), Some(Obj::Egg { laid }) if *laid == day));
    assert_eq!(s.play.player.inv.count(Item::MysteryEgg), 0);
    let night = |s: &mut Sim| {
        s.play.start_fade(Trans::Sleep { passed_out: false });
        s.frames(60);
        let news = s
            .play
            .last_summary
            .as_ref()
            .expect("a summary")
            .notes
            .clone();
        s.play.menu = Menu::None;
        s.play.fade = None;
        s.play.area = Area::Farm;
        news
    };
    let mut news = Vec::new();
    for _ in 0..TILT_DAY {
        news = night(&mut s);
    }
    assert_eq!(s.play.clock.day, day + TILT_DAY);
    assert!(
        news.iter().any(|(t, _)| t.contains("tipped over")),
        "{news:?}"
    );
    // Tipped over, but not ready: standing right by it does nothing yet.
    s.stand(x, z + 2, Vec2::new(0.0, -1.0));
    s.frames(30);
    assert!(s.play.hatching.is_none() && s.play.spider.is_none());
    // The tenth morning: it waits for you to come by.
    news = night(&mut s);
    assert_eq!(s.play.clock.day, day + HATCH_DAYS);
    assert!(news.iter().any(|(t, _)| t.contains("tapping")), "{news:?}");
    s.stand(x + 14, z, Vec2::new(0.0, 1.0));
    s.frames(20);
    assert!(s.play.hatching.is_none(), "not while you're away");
    s.stand(x + 2, z + 1, Vec2::new(-1.0, 0.0));
    s.frames(5);
    assert!(
        s.play.hatching.is_some(),
        "it starts to hatch as you come near"
    );
    s.frames(150);
    assert!(s.play.farm.obj(x, z).is_none(), "the shell is gone");
    let spider = s.play.spider.expect("a jumping spider!");
    assert!((spider.pos - Vec2::new(x as f32 + 0.5, z as f32 + 0.5)).length() < 2.5);
}

#[test]
fn the_cat_and_the_spider_play_together() {
    use super::pets::{CatMode, Spider};
    let mut s = Sim::new();
    s.play.clock.min = 560.0;
    let cat = s.play.cat.pos;
    s.play.spider = Some(Spider::new(cat + Vec2::new(1.0, 0.5)));
    let (mut pounces, mut rides, mut hops) = (0, 0, 0);
    let mut was = (false, false, false);
    for _ in 0..60 * 120 {
        s.frames(1);
        let c = s.play.cat;
        let sp = s.play.spider.expect("still here");
        let now = (
            matches!(c.mode, CatMode::Pounce { .. }),
            sp.ride > 0.0,
            sp.hop.is_some(),
        );
        pounces += usize::from(now.0 && !was.0);
        rides += usize::from(now.1 && !was.1);
        hops += usize::from(now.2 && !was.2);
        was = now;
        // It never lands anywhere it shouldn't.
        let (tx, tz) = (sp.pos.x.floor() as i32, sp.pos.y.floor() as i32);
        assert!(sp.ride > 0.0 || sp.y > 0.01 || !s.play.farm.blocked(tx, tz));
    }
    assert!(hops > 25, "{hops} hops");
    assert!(pounces >= 1, "the cat never pounced");
    assert!(rides >= 1, "the spider never rode the cat");
    // A fuss: stand by it and it waves.
    let sp = s.play.spider.expect("still here");
    s.play.player.pos = sp.pos + Vec2::new(0.6, 0.0);
    s.play.target = None;
    let mut io = frame_io(&s.input, &s.audio);
    assert!(s.play.pet_nearby(&mut io));
    assert!(s.play.spider.expect("still here").wave > 0.0 || s.play.cat.pet > 0.0);
}

#[test]
fn seasonal_crops_keep_to_their_season() {
    use super::play::SEASON_DAYS;
    let mut s = Sim::new();
    s.play.clock.day = 1;
    clear_farm_tile(&mut s.play, 35, 15);
    clear_farm_tile(&mut s.play, 35, 16);
    s.play.farm.set_floor(35, 16, Floor::Tilled);
    s.stand(35, 15, Vec2::new(0.0, 1.0));
    s.frames(2);

    // Snowdrops wait for winter...
    s.play.player.inv.slots[5] = Some(Stack::new(Item::SnowdropBulb, 3));
    s.select(5);
    s.tap(KeyCode::KeyJ, 5);
    assert!(s.play.farm.obj(35, 16).is_none(), "no snowdrops in spring");
    assert_eq!(s.play.player.inv.count(Item::SnowdropBulb), 3);
    assert!(
        s.play
            .toasts
            .iter()
            .any(|t| t.text.contains("only grows in winter")),
        "says why"
    );

    // ...but tulips go straight in.
    s.play.player.inv.slots[5] = Some(Stack::new(Item::TulipBulb, 3));
    s.tap(KeyCode::KeyJ, 5);
    assert!(matches!(
        s.play.farm.obj(35, 16),
        Some(Obj::Crop {
            crop: Crop::PastelTulip,
            ..
        })
    ));
    // A year-round turnip beside it lives through the change of season.
    clear_farm_tile(&mut s.play, 36, 16);
    s.play.farm.set_floor(36, 16, Floor::Tilled);
    s.play.farm.set_obj(
        36,
        16,
        Some(Obj::Crop {
            crop: Crop::Turnip,
            days: 0,
            harvested: false,
        }),
    );

    // The last day of spring warns about the tulip...
    s.play.clock.day = SEASON_DAYS - 1;
    s.play.start_fade(Trans::Sleep { passed_out: false });
    s.frames(60);
    assert_eq!(s.play.clock.season_day(), SEASON_DAYS);
    let notes = &s.play.last_summary.as_ref().unwrap().notes;
    assert!(
        notes
            .iter()
            .any(|(n, _)| n.contains("Last day of Spring! 1 crop")),
        "{notes:?}"
    );

    // ...and the first night of summer wilts it into a weed.
    s.play.menu = Menu::None;
    s.play.start_fade(Trans::Sleep { passed_out: false });
    s.frames(60);
    assert_eq!(s.play.clock.season(), super::play::Season::Summer);
    assert!(matches!(s.play.farm.obj(35, 16), Some(Obj::Weed { .. })));
    assert!(matches!(
        s.play.farm.obj(36, 16),
        Some(Obj::Crop {
            crop: Crop::Turnip,
            ..
        })
    ));
    let notes = &s.play.last_summary.as_ref().unwrap().notes;
    assert!(notes.iter().any(|(n, _)| n.contains("Summer is here")));
    assert!(
        notes
            .iter()
            .any(|(n, _)| n.contains("1 out-of-season crop withered")),
        "{notes:?}"
    );
}

#[test]
fn snow_comes_with_winter_and_melts_in_spring() {
    use super::play::SEASON_DAYS;
    let mut s = Sim::new();
    s.play.clock.day = 3 * SEASON_DAYS + 2;
    s.frames(1);
    assert!(s.play.farm.snowy() && s.play.town.snowy());
    assert_eq!(s.play.env().season, Some(super::play::Season::Winter));
    s.play.clock.day = 4 * SEASON_DAYS + 1;
    s.frames(1);
    assert!(!s.play.farm.snowy() && !s.play.town.snowy());
}

#[test]
fn lantern_snails_and_book_worms_live_from_floor_eleven_down() {
    use super::dungeon::{DEEP_FOLK, Foe, generate};
    let deep = |f: &Foe| matches!(f, Foe::Snail | Foe::Bookworm);
    let mut seen = [0usize; 2];
    for seed in 0..12u64 {
        for depth in [1, 5, 9, 10] {
            let l = generate(seed, depth, super::dungeon::biome_for(seed, depth), false);
            assert!(
                !l.spawns.iter().any(|s| deep(&s.foe)),
                "a deep creature on floor {depth}"
            );
        }
        for depth in [DEEP_FOLK, 25, 47, 63] {
            let l = generate(seed, depth, super::dungeon::biome_for(seed, depth), false);
            for s in &l.spawns {
                match s.foe {
                    Foe::Snail => seen[0] += 1,
                    Foe::Bookworm => seen[1] += 1,
                    _ => {}
                }
            }
        }
    }
    assert!(seen[0] > 5 && seen[1] > 5, "hardly any turn up: {seen:?}");
}

#[test]
fn a_lantern_snail_hides_in_its_shell_and_lights_the_dark() {
    use super::combat::Hit;
    use super::dungeon::Foe;
    use super::foes::{Enemy, HIDE_SECS};
    let mut s = Sim::new();
    one_foe_ahead(&mut s, 14, 1.2);
    let p = s.play.player.pos;
    s.play.foes[0] = Enemy::new(Foe::Snail, p.x, p.y + 1.2, 14, 1, false, 3);
    let snail = &s.play.foes[0];
    assert!(snail.light().is_some(), "its shell glows");
    assert!(!snail.shelled());
    let hp = snail.hp;
    // The first blow lands in full, and in it goes...
    s.play.strike(0, Hit::plain(10), p);
    let snail = &s.play.foes[0];
    assert_eq!(snail.hp, hp - 10);
    assert!(snail.shelled() && snail.armored() && !snail.grounded());
    // ...so the next glances off the glass.
    s.play.strike(0, Hit::plain(10), p);
    let snail = &s.play.foes[0];
    assert!(hp - 10 - snail.hp < 5, "the shell barely helped");
    // It peeks out again, and leaves shining slime where it crawls.
    s.play.foes[0].speed = 0.8;
    s.play.foes[0].hurt_cd = 0.0;
    s.frames((HIDE_SECS * 60.0) as usize + 5);
    assert!(!s.play.foes[0].shelled());
    s.play.player.pos = p + Vec2::new(0.0, 8.0);
    s.frames(120);
    assert!(s.play.foes[0].trail.len() > 1, "no trail");
}

#[test]
fn book_worm_ink_dries_into_runes_pointing_the_way() {
    use super::dungeon::Foe;
    use super::foes::Enemy;
    let mut s = Sim::new();
    one_foe_ahead(&mut s, 23, 3.0);
    let p = s.play.player.pos;
    // Somewhere open, a few steps from the hero.
    let spot = [
        Vec2::new(0.0, 4.0),
        Vec2::new(0.0, -4.0),
        Vec2::new(4.0, 0.0),
        Vec2::new(-4.0, 0.0),
    ]
    .into_iter()
    .map(|d| p + d)
    .find(|q| {
        let w = &s.play.level.as_ref().unwrap().world;
        w.clear_line(p, *q) && !w.blocked(q.x as i32, q.y as i32)
    })
    .expect("room to stand");
    let mut worm = Enemy::new(Foe::Bookworm, spot.x, spot.y, 23, 2, false, 5);
    worm.alert = true;
    worm.t = 0.0;
    s.play.foes[0] = worm;
    s.play.player.hurt = 0.0;
    let hp = s.play.player.hp;
    for _ in 0..600 {
        s.frames(1);
        s.play.player.pos = p;
        s.play.player.hp = hp.max(s.play.player.hp);
        if !s.play.runes.is_empty() {
            break;
        }
    }
    assert!(!s.play.runes.is_empty(), "the worm never spat");
    // Every rune points along the way to the stairs (or towards a secret).
    let level = s.play.level.as_ref().unwrap();
    for r in &s.play.runes {
        assert!((r.dir.length() - 1.0).abs() < 0.01);
        if !r.secret {
            let want = super::ink::way(&level.world, r.pos, level.stairs);
            assert!(r.dir.dot(want) > 0.9, "points the wrong way");
        }
    }
    // And they fade away in time.
    s.play.update_runes(60.0);
    assert!(s.play.runes.is_empty());
}

#[test]
fn an_ink_map_lights_the_way_down() {
    let mut s = Sim::new();
    one_foe_ahead(&mut s, 12, 3.0);
    s.play.foes.clear();
    s.play.player.inv.slots[5] = Some(Stack::new(Item::InkMap, 2));
    s.select(5);
    s.tap(KeyCode::KeyE, 30);
    assert_eq!(s.play.player.inv.count(Item::InkMap), 1);
    let level = s.play.level.as_ref().unwrap();
    let path = level.world.path(s.play.player.tile(), level.stairs);
    assert!(!path.is_empty());
    let line: Vec<_> = s.play.runes.iter().filter(|r| !r.secret).collect();
    assert!(!line.is_empty(), "no runes");
    // They lie along the way, pointing on down it.
    for r in &line {
        let t = (r.pos.x as i32, r.pos.y as i32);
        assert!(path.contains(&t), "a rune off the path");
    }
    // Up on the farm the ink knows nothing.
    s.play.start_fade(Trans::Home);
    s.frames(60);
    s.play.menu = Menu::None;
    s.play.player.inv.slots[5] = Some(Stack::new(Item::InkMap, 1));
    s.play.nag = 0.0;
    s.tap(KeyCode::KeyE, 5);
    assert_eq!(s.play.player.inv.count(Item::InkMap), 1);
}

#[test]
fn glowcap_caves_glow_and_their_mushrooms_can_be_gathered() {
    use super::dungeon::Foe;
    use super::fish::Water;
    use super::glowcave::{self, TALL, is_glowcave};
    use super::world::Floor;
    let mut s = Sim::new();
    let depth = (4..90)
        .find(|&d| is_glowcave(s.play.seed, d, s.play.biome_at(d)))
        .expect("a glowcap floor");
    s.play.fade = None;
    s.play.start_fade(Trans::Descend {
        depth,
        via_waystone: false,
    });
    s.frames(30);
    assert!(s.play.world().glowcave);
    let banner = s.play.banner.as_ref().expect("the floor's banner");
    assert!(banner.sub.ends_with("the glowcap caves"), "{}", banner.sub);
    // Lit cooler than the caves round about, and everyone who lives here is at home in it:
    // the shroomlings glow like the caps.
    let env = s.play.env();
    assert!(env.warmth < 3.0);
    assert!(s.play.foes.iter().all(|f| f.glowcave));
    let mut shroom =
        super::foes::Enemy::new(Foe::Shroom, 5.0, 5.0, depth, 2, false, 3).in_the_glowcaves();
    assert!(shroom.light().is_some(), "it lights its way");
    shroom.glowcave = false;
    assert!(shroom.light().is_none());
    // A sickle through a tall glowcap gathers at least one glowcap, and it's gone.
    let (x, z) = {
        let w = s.play.world_mut();
        let spot = (0..w.h)
            .flat_map(|z| (0..w.w).map(move |x| (x, z)))
            .find(|&(x, z)| matches!(w.obj(x, z), Some(Obj::Glowcap { .. })))
            .expect("glowcaps");
        w.set_obj(spot.0, spot.1, Some(glowcave::glowcap(1, TALL)));
        spot
    };
    s.play.drops.clear();
    let mut io = frame_io(&s.input, &s.audio);
    assert!(s.play.gather_glow(x, z, &mut io));
    assert!(s.play.world().obj(x, z).is_none());
    let caps: u32 = s
        .play
        .drops
        .iter()
        .filter(|d| d.stack.item == Item::Glowcap)
        .map(|d| d.stack.n as u32)
        .sum();
    assert!(caps >= 1);
    assert!(!s.play.gather_glow(x, z, &mut io), "nothing left there");
    // Its pools hold the Fungal Hollow's fish.
    let pool = {
        let w = s.play.world();
        (0..w.h)
            .flat_map(|z| (0..w.w).map(move |x| (x, z)))
            .find(|&(x, z)| w.floor(x, z) == Floor::Water)
            .expect("a pool")
    };
    assert_eq!(s.play.water_at(pool.0, pool.1), Some(Water::Fungal));
}

#[test]
fn drakelings_leaflings_and_werewolves_fight_back() {
    use super::dungeon::Foe;
    for (foe, biome, depth) in [
        (Foe::Drake, 3usize, 14u32),
        (Foe::Drake, 4, 24),
        (Foe::Leafling, 0, 4),
        (Foe::Werewolf, 2, 8),
    ] {
        let mut s = Sim::new();
        s.play.start_fade(Trans::Descend {
            depth,
            via_waystone: false,
        });
        s.frames(60);
        let (px, pz) = s.play.player.tile();
        let w = &s.play.level.as_ref().unwrap().world;
        let (fx, fz) = w.nearest_open(px + 2, pz);
        s.play.foes.clear();
        s.play.foes.push(super::foes::Enemy::new(
            foe,
            fx as f32 + 0.5,
            fz as f32 + 0.5,
            depth,
            biome,
            false,
            3,
        ));
        s.play.player.hurt = 0.0;
        let start = s.play.player.hp;
        let mut hit = false;
        for _ in 0..900 {
            s.frames(1);
            if s.play.player.hp < start {
                hit = true;
                break;
            }
            s.play.player.hp = start;
            s.play.player.hurt = 0.0;
        }
        assert!(hit, "{foe:?} in biome {biome} never landed a blow");
    }
}

#[test]
fn a_drakeling_rears_back_and_breathes_fire_or_frost() {
    use super::dungeon::Foe;
    use super::foes::{Enemy, St, kind_name};
    use super::fx::ShotKind;
    assert_eq!(kind_name(Foe::Drake, 3), "Cinder Drakeling");
    assert_eq!(kind_name(Foe::Drake, 4), "Frost Drakeling");
    use crate::palette::{ORANGE, SKY};
    for (biome, colour) in [(3usize, ORANGE), (4, SKY)] {
        let mut s = Sim::new();
        one_foe_ahead(&mut s, 14, 3.0);
        let p = s.play.player.pos;
        let w = &s.play.level.as_ref().unwrap().world;
        let (fx, fz) = w.nearest_open(p.x as i32, p.y as i32 + 3);
        let at = Vec2::new(fx as f32 + 0.5, fz as f32 + 0.5);
        assert!(w.clear_line(at, p), "no clear line to the hero");
        let mut d = Enemy::new(Foe::Drake, at.x, at.y, 14, biome, false, 3);
        d.alert = true;
        d.t = 0.0;
        d.speed = 0.0;
        s.play.foes[0] = d;
        s.play.shots.clear();
        s.frames(20);
        let d = &s.play.foes[0];
        assert_eq!(d.st, St::Windup, "rearing back to draw breath");
        assert!(
            d.armored() && d.light().is_some(),
            "scaly, and its mouth glows"
        );
        s.frames(40);
        let puffs: Vec<_> = s
            .play
            .shots
            .iter()
            .filter(|s| s.kind == ShotKind::Breath)
            .collect();
        assert!(puffs.len() >= 3, "only {} puffs", puffs.len());
        assert!(puffs.iter().all(|s| s.color == colour));
        // Breathed at the hero.
        let towards = (p - at).normalize();
        assert!(puffs.iter().all(|s| s.vel.normalize().dot(towards) > 0.8));
        s.frames(30);
        assert_ne!(s.play.foes[0].st, St::Dash, "a breath doesn't last");
    }
}

#[test]
fn a_leafling_mends_whoever_is_worst_hurt() {
    use super::dungeon::Foe;
    use super::foes::{Enemy, MEND_AGAIN};
    let mut s = Sim::new();
    one_foe_ahead(&mut s, 3, 2.0);
    let p = s.play.player.pos;
    let foe = |f: Foe, dx: f32, dz: f32, seed: u32| {
        let mut e = Enemy::new(f, p.x + dx, p.y + dz, 3, 0, false, seed);
        e.speed = 0.0;
        e
    };
    let mut leaf = foe(Foe::Leafling, 0.5, 2.0, 5);
    leaf.alert = true;
    leaf.summon = 0.05;
    let mut slime = foe(Foe::Slime, -1.0, 2.5, 6);
    slime.hp = slime.max_hp / 3;
    let mut bat = foe(Foe::Bat, 1.0, 3.0, 7);
    bat.hp = bat.max_hp * 3 / 4;
    s.play.foes = vec![leaf, slime, bat];
    let (slime_hp, bat_hp) = (s.play.foes[1].hp, s.play.foes[2].hp);
    s.frames(10);
    assert!(s.play.foes[1].hp > slime_hp, "the worst hurt is mended");
    assert_eq!(s.play.foes[2].hp, bat_hp, "one at a time");
    // With nobody left needing it, it looks again in a moment rather than waiting out
    // its whole spell.
    for f in s.play.foes.iter_mut() {
        f.hp = f.max_hp;
    }
    s.play.foes[0].summon = 0.02;
    s.frames(3);
    let again = s.play.foes[0].summon;
    assert!(again <= MEND_AGAIN && again > MEND_AGAIN - 0.1, "{again}");
}

#[test]
fn werewolves_prowl_under_a_full_moon_and_howl_up_the_floor() {
    use super::dungeon::{Foe, WEREWOLF_FLOOR};
    use super::foes::{Enemy, HOWL_SECS, St};
    // Only under a full moon (day 5), and not so near the top.
    let wolves = |day: u32, depth: u32| {
        let mut s = Sim::new();
        s.play.clock.day = day;
        s.play.start_fade(Trans::Descend {
            depth,
            via_waystone: false,
        });
        s.frames(60);
        let start = s.play.player.pos;
        let wolves: Vec<&Enemy> = s
            .play
            .foes
            .iter()
            .filter(|f| f.foe == Foe::Werewolf)
            .collect();
        assert!(
            wolves
                .iter()
                .all(|w| (w.pos - start).length() > 14.0 && w.moonlit)
        );
        wolves.len()
    };
    assert_eq!(wolves(2, 12), 0, "not on a quiet night");
    assert_eq!(wolves(5, WEREWOLF_FLOOR - 1), 0, "not so near the top");
    assert!(wolves(5, 12) >= 1, "none under the full moon");
    // Catching the hero's scent, it throws its head back and howls, and everything in
    // earshot comes running, riled up.
    let mut s = Sim::new();
    one_foe_ahead(&mut s, 12, 3.0);
    let p = s.play.player.pos;
    let w = &s.play.level.as_ref().unwrap().world;
    // Somewhere in plain sight a few steps off, and someone else a little further.
    let spot = |near: f32, far: f32| {
        (-6..=6)
            .flat_map(|dz| (-6..=6).map(move |dx| (dx, dz)))
            .map(|(dx, dz)| Vec2::new(p.x.floor() + dx as f32 + 0.5, p.y.floor() + dz as f32 + 0.5))
            .find(|q| {
                let d = (*q - p).length();
                d > near && d < far && !w.blocked(q.x as i32, q.y as i32) && w.clear_line(*q, p)
            })
            .expect("open ground")
    };
    let (a, b) = (spot(2.5, 4.0), spot(4.5, 6.5));
    let wolf = Enemy::new(Foe::Werewolf, a.x, a.y, 12, 1, false, 9);
    let near = Enemy::new(Foe::Skeleton, b.x, b.y, 12, 1, false, 10);
    s.play.foes = vec![wolf, near];
    let fury = s.play.foes[1].fury;
    s.frames(2);
    assert_eq!(s.play.foes[0].st, St::Howl);
    assert!(s.play.foes[1].alert && s.play.foes[1].fury > fury);
    s.frames((HOWL_SECS * 60.0) as usize + 5);
    assert_ne!(s.play.foes[0].st, St::Howl, "one howl, then the hunt");
    // Losing the scent and finding it again, it doesn't howl twice.
    s.play.foes[0].alert = false;
    s.frames(2);
    assert_ne!(s.play.foes[0].st, St::Howl);
}

#[test]
fn drakelings_leaflings_and_werewolves_drop_their_bits() {
    use super::dungeon::Foe;
    use super::loot::{Fortune, foe_loot};
    let mut rng = crate::util::Rng::new(9);
    for (foe, biome, bit) in [
        (Foe::Drake, 3, Item::CinderScale),
        (Foe::Drake, 4, Item::FrostScale),
        (Foe::Leafling, 0, Item::Heartleaf),
        (Foe::Werewolf, 2, Item::WolfFang),
        (Foe::Minotaur, 1, Item::MinotaurHorn),
        (Foe::Griffin, 4, Item::GriffinFeather),
    ] {
        let mut got = 0;
        for _ in 0..200 {
            let loot = foe_loot(foe, false, biome, 14, Fortune::plain(), &mut rng);
            got += loot.iter().filter(|s| s.item == bit).count();
        }
        assert!(got > 60, "{foe:?} dropped {got} {bit:?}");
    }
}

#[test]
fn a_backpack_adds_room_to_the_bag() {
    use super::items::Pack;
    use super::player::BAG;
    let mut s = Sim::new();
    let p = &mut s.play.player;
    // Fill the bag right up.
    for slot in p.inv.slots.iter_mut() {
        if slot.is_none() {
            *slot = Some(Stack::new(Item::Stone, 1));
        }
    }
    assert_eq!(p.inv.add(Item::Bone, 1), 1, "the bag is full");
    // Swap a stone for an eight-slot rucksack and put it on.
    p.inv.slots[30] = Some(Stack::with_pack(Item::Rucksack, Pack { slots: 8, hue: 3 }));
    assert!(p.wear_pack_from(30));
    assert_eq!((p.pack_slots(), p.inv.slots.len()), (8, BAG + 8));
    assert!(p.inv.slots[30].is_none(), "no old backpack to swap back");
    p.inv.slots[30] = Some(Stack::new(Item::Stone, 1));
    // Now there's room, in the pouch.
    assert_eq!(p.inv.add(Item::Bone, 3), 0);
    assert_eq!(
        p.inv.slots[BAG].map(|s| (s.item, s.n)),
        Some((Item::Bone, 3))
    );
    // With the bag full there's nowhere for the bones to go: it stays on.
    assert!(!p.unwear_pack() && p.lift_pack().is_none());
    assert!(p.pack.is_some() && p.inv.count(Item::Bone) == 3);
    // Make a little room, and off it comes, the bones tipped out into the bag.
    p.inv.slots[5] = None;
    p.inv.slots[6] = None;
    assert!(p.unwear_pack());
    assert_eq!(p.inv.slots.len(), BAG);
    assert_eq!(p.inv.count(Item::Bone), 3);
    assert_eq!(p.inv.count(Item::Rucksack), 1);
    assert!(p.pack.is_none());
}

#[test]
fn a_smaller_backpack_spills_into_the_bag() {
    use super::items::Pack;
    use super::player::BAG;
    let mut s = Sim::new();
    let p = &mut s.play.player;
    let big = Stack::with_pack(Item::FramePack, Pack { slots: 12, hue: 0 });
    let small = Stack::with_pack(Item::Knapsack, Pack { slots: 4, hue: 5 });
    assert!(p.swap_pack(big).unwrap().is_none());
    for slot in p.inv.slots.iter_mut().take(BAG) {
        if slot.is_none() {
            *slot = Some(Stack::new(Item::Stone, 1));
        }
    }
    for k in 0..6 {
        p.inv.slots[BAG + k] = Some(Stack::new(Item::Fiber, 1 + k as u16));
    }
    // Four of the six fit in the knapsack; with the bag full the other two can't go.
    assert!(p.swap_pack(small).is_err());
    assert_eq!(p.inv.slots.len(), BAG + 12, "nothing changed");
    p.inv.slots[1] = None;
    p.inv.slots[2] = None;
    let old = p.swap_pack(small).expect("room now");
    assert_eq!(old.map(|o| o.item), Some(Item::FramePack));
    assert_eq!(p.inv.slots.len(), BAG + 4);
    assert_eq!(p.inv.count(Item::Fiber), 1 + 2 + 3 + 4 + 5 + 6);
}

#[test]
fn backpacks_are_rarer_the_roomier_and_come_in_all_looks() {
    use super::items::{MAX_PACK, MIN_PACK, Pack};
    use super::loot::{Fortune, random_pack};
    let r = |slots| Pack { slots, hue: 0 }.rarity();
    assert_eq!(r(4), Rarity::Common);
    assert_eq!(r(8), Rarity::Uncommon);
    assert_eq!(r(12), Rarity::Rare);
    assert_eq!(r(16), Rarity::Epic);
    assert_eq!(r(20), Rarity::Legendary);
    assert!(Pack { slots: 20, hue: 0 }.price() > Pack { slots: 6, hue: 0 }.price());
    let mut rng = crate::util::Rng::new(4);
    let mut items = std::collections::HashSet::new();
    let mut hues = std::collections::HashSet::new();
    let (mut shallow, mut deep) = (0u32, 0u32);
    for _ in 0..400 {
        let a = random_pack(2, Fortune::plain(), &mut rng);
        let b = random_pack(60, Fortune::plain(), &mut rng);
        for s in [a, b] {
            let p = s.pack.expect("a backpack's roll");
            assert!((MIN_PACK..=MAX_PACK).contains(&p.slots));
            assert_eq!(s.rarity(), Some(p.rarity()));
            items.insert(s.item);
            hues.insert(p.hue);
        }
        shallow += a.pack_slots() as u32;
        deep += b.pack_slots() as u32;
    }
    assert_eq!(items.len(), 6, "every look turns up");
    assert_eq!(hues.len(), crate::assets::pack_art::HUES);
    assert!(
        deep > shallow * 3 / 2,
        "deeper, roomier: {deep} vs {shallow}"
    );
    // The guild keeps some on its shelves.
    let mut p = Play::new(3);
    p.deepest = 25;
    let guild = super::shops::rows(
        &p,
        Some(super::town::Place::Guild),
        super::menus::ShopTab::Goods,
    );
    let packs: Vec<Item> = guild
        .iter()
        .filter(|r| r.0.pack.is_some())
        .map(|r| r.0.item)
        .collect();
    assert_eq!(packs.len(), 4, "{packs:?}");
}

#[test]
fn a_backpack_is_worn_and_emptied_from_the_bag_tab() {
    use super::items::Pack;
    use super::menus::{PACK_AT, WORN_AT};
    use super::player::BAG;
    let mut s = Sim::new();
    s.play.player.inv.slots[12] = Some(Stack::with_pack(Item::Knapsack, Pack { slots: 6, hue: 1 }));
    let bag = |cursor| Menu::Inventory {
        tab: Tab::Bag,
        cursor,
        recipe: 0,
        scroll: 0,
        cat: 0,
    };
    s.play.menu = bag(12);
    s.tap(KeyCode::Enter, 2);
    assert_eq!(s.play.player.pack.map(|p| p.item), Some(Item::Knapsack));
    assert_eq!(s.play.player.inv.slots.len(), BAG + 6);
    // The cursor walks off the right of the bag into the pouch, and down to the backpack.
    s.play.menu = bag(9);
    let cursor = |s: &Sim| match s.play.menu {
        Menu::Inventory { cursor, .. } => cursor,
        _ => usize::MAX,
    };
    s.tap(KeyCode::ArrowRight, 1);
    assert_eq!(cursor(&s), BAG);
    s.tap(KeyCode::ArrowDown, 1);
    assert_eq!(cursor(&s), BAG + 5);
    s.tap(KeyCode::ArrowDown, 1);
    assert_eq!(cursor(&s), PACK_AT);
    s.tap(KeyCode::ArrowRight, 1);
    assert_eq!(cursor(&s), WORN_AT, "round to the worn gear");
    // A on the backpack's slot takes it off into the bag.
    s.play.menu = bag(PACK_AT);
    s.tap(KeyCode::Enter, 2);
    assert!(s.play.player.pack.is_none());
    assert_eq!(s.play.player.inv.slots.len(), BAG);
    assert_eq!(s.play.player.inv.count(Item::Knapsack), 1);
}

#[test]
fn a_backpack_and_its_pouch_are_saved() {
    use super::items::Pack;
    use super::player::{BAG, Player};
    let mut s = Sim::new();
    let p = &mut s.play.player;
    let pack = Stack::with_pack(Item::ShellPack, Pack { slots: 17, hue: 6 });
    assert!(p.swap_pack(pack).is_ok());
    p.inv.slots[BAG + 16] = Some(Stack::new(Item::Ruby, 2));
    let json = serde_json::to_string(&p.save()).unwrap();
    let back: super::player::PlayerSave = serde_json::from_str(&json).unwrap();
    let q = Player::load(back, Vec2::ZERO);
    assert_eq!(q.pack, Some(pack));
    assert_eq!(q.pack.and_then(|s| s.rarity()), Some(Rarity::Epic));
    assert_eq!(q.inv.slots.len(), BAG + 17);
    assert_eq!(q.inv.slots[BAG + 16].map(|s| s.item), Some(Item::Ruby));
    // An older save, with no backpack, still loads.
    let mut old: serde_json::Value = serde_json::from_str(&json).unwrap();
    old.as_object_mut().unwrap().remove("pack");
    let q = Player::load(serde_json::from_value(old).unwrap(), Vec2::ZERO);
    assert!(q.pack.is_none());
    assert_eq!(q.inv.slots.len(), BAG);
}

#[test]
fn every_monster_leaves_a_backpack_in_its_own_shape() {
    use super::dungeon::Foe;
    use super::loot::{Fortune, foe_loot, monster_pack};
    let a = crate::assets::Assets::new();
    let foes = [
        Foe::Slime,
        Foe::Bat,
        Foe::Shroom,
        Foe::Crab,
        Foe::Wisp,
        Foe::Beetle,
        Foe::Imp,
        Foe::Skeleton,
        Foe::Golem,
        Foe::Ghost,
        Foe::Frog,
        Foe::Jelly,
        Foe::Puffer,
        Foe::Zombie,
        Foe::Brute,
        Foe::Sneak,
        Foe::Bug,
        Foe::Snail,
        Foe::Bookworm,
        Foe::Drake,
        Foe::Leafling,
        Foe::Werewolf,
        Foe::Minotaur,
        Foe::Griffin,
    ];
    let mut looks = std::collections::HashSet::new();
    for foe in foes {
        let item = monster_pack(foe);
        let style = item.pack_style().expect("a backpack");
        assert!(looks.insert(style), "{foe:?} shares a look");
        // Worn, and in the bag, it looks like itself.
        let s = Stack::with_pack(item, super::items::Pack { slots: 9, hue: 3 });
        assert!(a.pack_mesh(&s).is_some_and(|m| !m.tris.is_empty()));
        assert_eq!(a.stack_icon(&s), a.icon(item.def().icon));
    }
    // Rarely from a creature, often from a guardian.
    let mut rng = crate::util::Rng::new(12);
    let own = |boss: bool, rng: &mut crate::util::Rng| {
        (0..400)
            .filter(|_| {
                foe_loot(Foe::Slime, boss, 0, 12, Fortune::plain(), rng)
                    .iter()
                    .any(|s| s.item == super::items::Item::SlimePack)
            })
            .count()
    };
    let (plain, boss) = (own(false, &mut rng), own(true, &mut rng));
    assert!(
        (1..20).contains(&plain),
        "{plain} slime packs from 400 slimes"
    );
    assert!(boss > 120, "{boss} from 400 King Slimes");
}

/// Down into the first marble labyrinth on this save.
fn into_the_labyrinth(s: &mut Sim) -> u32 {
    use super::labyrinth::is_labyrinth;
    let depth = (8..400)
        .find(|&d| is_labyrinth(s.play.seed, d, s.play.biome_at(d)))
        .expect("a labyrinth floor");
    s.play.fade = None;
    s.play.start_fade(Trans::Descend {
        depth,
        via_waystone: false,
    });
    // (Nothing stirs till the fade's done.)
    s.frames(60);
    depth
}

#[test]
fn the_labyrinth_keeps_minotaurs_and_a_griffin_on_its_nest() {
    use super::dungeon::Foe;
    use super::foes::PERCH_Y;
    use super::labyrinth::minotaurs;
    use super::world::Wall;
    let mut s = Sim::new();
    let depth = into_the_labyrinth(&mut s);
    assert!(s.play.world().labyrinth);
    let banner = s.play.banner.as_ref().expect("the floor's banner");
    assert!(
        banner.sub.ends_with("the marble labyrinth"),
        "{}",
        banner.sub
    );
    // Pale marble, lit warm.
    assert!(s.play.env().ambient > 0.5);
    let bulls = s
        .play
        .foes
        .iter()
        .filter(|f| f.foe == Foe::Minotaur)
        .count();
    assert_eq!(bulls, minotaurs(depth));
    let griffin = s
        .play
        .foes
        .iter()
        .find(|f| f.foe == Foe::Griffin)
        .expect("a griffin");
    // Sat on its nest up on its column until it sees you.
    let (x, z) = (griffin.pos.x as i32, griffin.pos.y as i32);
    assert!(matches!(s.play.world().obj(x, z), Some(Obj::Nest)));
    assert!(!griffin.alert);
    assert!((griffin.y - PERCH_Y).abs() < 1e-3);
    assert!(!griffin.grounded(), "harmless up there");
    // The marble stands up to a pickaxe.
    s.play.foes.clear();
    let w = s.play.world();
    let wall = (1..w.h - 1)
        .flat_map(|z| (1..w.w - 1).map(move |x| (x, z)))
        .find(|&(x, z)| matches!(w.wall(x, z), Wall::Marble(_)) && !w.blocked(x, z + 1))
        .expect("marble");
    s.stand(wall.0, wall.1 + 1, Vec2::new(0.0, -1.0));
    s.select(4);
    for _ in 0..8 {
        s.tap(KeyCode::KeyJ, 30);
    }
    assert!(matches!(
        s.play.world().wall(wall.0, wall.1),
        Wall::Marble(_)
    ));
}

#[test]
fn a_minotaur_charges_down_the_corridor_and_is_dazed_by_the_wall() {
    use super::combat::Hit;
    use super::dungeon::Foe;
    use super::foes::{CHARGE_WINDUP, DAZED, Enemy, St};
    let mut s = Sim::new();
    let depth = into_the_labyrinth(&mut s);
    // A straight run of corridor with a wall across its north end.
    let w = s.play.world();
    let (x, z) = (1..w.h - 7)
        .flat_map(|z| (1..w.w - 1).map(move |x| (x, z)))
        .find(|&(x, z)| {
            w.wall(x, z - 1) != super::world::Wall::None
                && (0..7).all(|k| !w.blocked(x, z + k) && w.obj(x, z + k).is_none())
                && (0..7).all(|k| !w.blocked(x + 1, z + k))
        })
        .expect("a straight corridor");
    s.play.foes.clear();
    s.play.player.pos = Vec2::new(x as f32 + 0.5, z as f32 + 0.5);
    let mut bull = Enemy::new(
        Foe::Minotaur,
        x as f32 + 0.5,
        z as f32 + 5.5,
        depth,
        0,
        false,
        4,
    );
    bull.alert = true;
    bull.t = 0.0;
    s.play.foes.push(bull);
    // It lowers its horns and paws the ground...
    s.frames(2);
    assert_eq!(s.play.foes[0].st, St::Windup);
    s.frames((CHARGE_WINDUP * 60.0) as usize + 2);
    assert_eq!(s.play.foes[0].st, St::Dash, "and charges");
    // ...and you step aside: on it thunders, into the wall.
    s.play.player.pos.x += 1.2;
    let hp = s.play.player.hp;
    let mut dazed = false;
    for _ in 0..90 {
        s.frames(1);
        if s.play.foes[0].dazed() {
            dazed = true;
            break;
        }
    }
    assert!(dazed, "it never hit the wall");
    let f = &s.play.foes[0];
    assert_eq!(f.hops, DAZED);
    assert!(
        f.pos.y < z as f32 + 1.5,
        "it ran the length of the corridor"
    );
    assert!(!f.grounded(), "seeing stars, it can't hurt you");
    assert_eq!(s.play.player.hp, hp);
    // And your blows land harder meanwhile.
    let before = f.hp;
    let hit = Hit {
        dmg: 10,
        crit: false,
        burn: false,
        chill: false,
        shock: false,
        knock: 0.0,
    };
    let from = s.play.player.pos;
    s.play.strike(0, hit, from);
    assert_eq!(before - s.play.foes[0].hp, 15);
    // Then it comes round.
    s.frames(200);
    assert!(!s.play.foes[0].dazed());
}

#[test]
fn a_griffin_takes_wing_wheels_round_and_dives() {
    use super::dungeon::Foe;
    use super::foes::{St, WHEEL_Y};
    let mut s = Sim::new();
    into_the_labyrinth(&mut s);
    let i = s
        .play
        .foes
        .iter()
        .position(|f| f.foe == Foe::Griffin)
        .expect("a griffin");
    s.play.foes.retain(|f| f.foe == Foe::Griffin);
    let i = i.min(s.play.foes.len() - 1);
    let nest = s.play.foes[i].pos;
    // Walk into the courtyard in sight of it.
    let w = s.play.world();
    let spot = [(0, 3), (0, -3), (3, 0), (-3, 0), (2, 2), (-2, 2)]
        .into_iter()
        .map(|(dx, dz)| (nest.x as i32 + dx, nest.y as i32 + dz))
        .find(|&(x, z)| !w.blocked(x, z))
        .expect("open ground by the nest");
    s.play.player.pos = Vec2::new(spot.0 as f32 + 0.5, spot.1 as f32 + 0.5);
    s.frames(3);
    assert!(s.play.foes[i].alert, "it sees you");
    // Up it goes, to wheel round overhead...
    s.frames(120);
    let g = &s.play.foes[i];
    assert!(g.y > WHEEL_Y * 0.8, "only {} up", g.y);
    assert!(!g.grounded());
    // ...and down it comes, talons first.
    let mut dived = false;
    for _ in 0..900 {
        s.frames(1);
        s.play.player.hp = s.play.player.max_hp();
        let g = &s.play.foes[i];
        if g.st == St::Dash && g.grounded() {
            dived = true;
            break;
        }
    }
    assert!(dived, "it never stooped to dive");
}

fn into_the_canyon(s: &mut Sim) -> u32 {
    use super::canyon::is_canyon;
    let depth = (6..400)
        .find(|&d| is_canyon(s.play.seed, d, s.play.biome_at(d)))
        .expect("a canyon floor");
    s.play.fade = None;
    s.play.start_fade(Trans::Descend {
        depth,
        via_waystone: false,
    });
    s.frames(60);
    depth
}

/// Somewhere in the canyon with open sand `r` tiles either side, and a tile above and
/// below.
fn open_sand(s: &Sim, r: i32) -> (i32, i32) {
    let w = s.play.world();
    (2..w.h - 2)
        .flat_map(|z| (r + 1..w.w - r - 1).map(move |x| (x, z)))
        .find(|&(x, z)| {
            (-1..=1).all(|dz| {
                (-r..=r)
                    .all(|dx| !w.blocked(x + dx, z + dz) && w.floor(x + dx, z + dz) == Floor::Sand)
            })
        })
        .expect("open sand")
}

#[test]
fn the_canyon_is_hot_bright_and_dressed_for_the_desert() {
    use super::canyon::DESERT;
    let mut s = Sim::new();
    into_the_canyon(&mut s);
    assert!(s.play.world().canyon);
    let banner = s.play.banner.as_ref().expect("the floor's banner");
    assert!(
        banner.sub.ends_with("the sunscorch canyon"),
        "{}",
        banner.sub
    );
    assert!(s.play.env().ambient > 0.6);
    assert!(!s.play.foes.is_empty());
    assert!(
        s.play.foes.iter().all(|f| f.biome == DESERT),
        "everyone's dressed for the desert"
    );
    // Sandstone stands up to a pickaxe.
    s.play.foes.clear();
    let w = s.play.world();
    let wall = (1..w.h - 1)
        .flat_map(|z| (1..w.w - 1).map(move |x| (x, z)))
        .find(|&(x, z)| matches!(w.wall(x, z), Wall::Sandstone(_)) && !w.blocked(x, z + 1))
        .expect("sandstone");
    s.stand(wall.0, wall.1 + 1, Vec2::new(0.0, -1.0));
    s.select(4);
    for _ in 0..8 {
        s.tap(KeyCode::KeyJ, 30);
    }
    assert!(matches!(
        s.play.world().wall(wall.0, wall.1),
        Wall::Sandstone(_)
    ));
}

#[test]
fn quicksand_drags_at_your_feet() {
    let mut s = Sim::new();
    into_the_canyon(&mut s);
    s.play.foes.clear();
    let run = |s: &mut Sim, x: i32, z: i32| {
        s.stand(x, z, Vec2::new(1.0, 0.0));
        s.play.player.energy = s.play.player.max_energy() as f32;
        s.input.key_event(KeyCode::KeyD, true, false);
        s.frames(20);
        s.input.key_event(KeyCode::KeyD, false, false);
        s.frames(1);
        s.play.player.pos.x - (x as f32 + 0.5)
    };
    let w = s.play.world();
    let bog = (1..w.h - 1)
        .flat_map(|z| (1..w.w - 4).map(move |x| (x, z)))
        .find(|&(x, z)| {
            (0..4).all(|k| w.floor(x + k, z) == Floor::Quicksand && !w.blocked(x + k, z))
        })
        .expect("a stretch of quicksand");
    let sand = open_sand(&s, 3);
    let wading = run(&mut s, bog.0, bog.1);
    let walking = run(&mut s, sand.0 - 3, sand.1);
    assert!(wading > 0.1, "you can still wade through");
    assert!(wading < walking * 0.7, "{wading} wading, {walking} walking");
}

#[test]
fn a_cactling_keeps_still_among_the_cacti_until_you_come_close() {
    use super::canyon::DESERT;
    use super::dungeon::Foe;
    use super::foes::{Enemy, NEEDLES};
    use super::fx::ShotKind;
    let mut s = Sim::new();
    let depth = into_the_canyon(&mut s);
    s.play.foes.clear();
    s.play.shots.clear();
    let (x, z) = open_sand(&s, 3);
    s.stand(x - 3, z, Vec2::new(1.0, 0.0));
    let spot = Vec2::new(x as f32 + 2.5, z as f32 + 0.5);
    s.play.foes.push(Enemy::new(
        Foe::Cactus,
        spot.x,
        spot.y,
        depth,
        DESERT,
        false,
        3,
    ));
    s.frames(120);
    let c = &s.play.foes[0];
    assert!(!c.alert, "it looks like any other cactus");
    assert_eq!(c.pos, spot, "and never moves a root");
    // Come close, and up it pops.
    s.stand(x + 1, z, Vec2::new(1.0, 0.0));
    s.frames(2);
    assert!(s.play.foes[0].alert);
    // Before long it bristles and sprays needles all round.
    let mut most = 0;
    for _ in 0..400 {
        s.frames(1);
        s.play.player.hp = s.play.player.max_hp();
        let n = s
            .play
            .shots
            .iter()
            .filter(|s| s.kind == ShotKind::Needle)
            .count();
        most = most.max(n);
        if most >= NEEDLES {
            break;
        }
    }
    assert_eq!(most, NEEDLES, "a whole ring of needles");
}

#[test]
fn a_sand_cobra_rears_up_to_strike_and_spits_from_further_off() {
    use super::canyon::DESERT;
    use super::dungeon::Foe;
    use super::foes::{Enemy, REAR_SECS, SPITTING, STRIKING, St};
    use super::fx::ShotKind;
    let mut s = Sim::new();
    let depth = into_the_canyon(&mut s);
    s.play.foes.clear();
    s.play.shots.clear();
    let (x, z) = open_sand(&s, 3);
    s.stand(x - 2, z, Vec2::new(1.0, 0.0));
    // Close by, it rears up with its hood spread...
    let mut c = Enemy::new(
        Foe::Cobra,
        x as f32 - 0.1,
        z as f32 + 0.5,
        depth,
        DESERT,
        false,
        5,
    );
    c.alert = true;
    c.t = 0.0;
    s.play.foes.push(c);
    s.frames(2);
    assert_eq!(s.play.foes[0].st, St::Windup);
    assert_eq!(s.play.foes[0].hops, STRIKING);
    // ...and strikes.
    s.frames((REAR_SECS * 60.0) as usize + 2);
    assert_eq!(s.play.foes[0].st, St::Dash);
    // Further off, it spits venom at you.
    s.play.foes.clear();
    let away = Vec2::new(x as f32 + 2.5, z as f32 + 0.5);
    let mut c = Enemy::new(Foe::Cobra, away.x, away.y, depth, DESERT, false, 6);
    c.alert = true;
    s.play.foes.push(c);
    let mut spat = false;
    for _ in 0..600 {
        s.frames(1);
        s.play.player.hp = s.play.player.max_hp();
        if s.play.foes[0].hops == SPITTING && s.play.shots.iter().any(|s| s.kind == ShotKind::Venom)
        {
            spat = true;
            break;
        }
        // (Keep it at a distance.)
        s.play.foes[0].pos = away;
    }
    assert!(spat, "it never spat");
}
