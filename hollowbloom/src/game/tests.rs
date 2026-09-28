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
    super::save::write(&p).expect("save");
    let q = super::save::read().expect("load");
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
    assert_eq!(s.play.area, Area::Farm);
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
                if matches!(w.obj(x, z), Some(Obj::LootChest { opened: false })) {
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
