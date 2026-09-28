//! Offscreen modes: `--shots DIR` renders a tour of the game to PNG files and `--bench`
//! measures the renderer. Neither needs a display, so both run on CI machines.

use std::path::Path;
use std::time::Instant;

use glam::Vec2;

use crate::audio::Audio;
use crate::game::gear::{Affix, Gear, Rarity, Slot, Stat};
use crate::game::items::{ALL_CROPS, Crop, Item, Stack};
use crate::game::loot::{self, Fortune};
use crate::game::menus::{self, Menu, Pick, ShopTab, Tab};
use crate::game::play::{Play, Trans};
use crate::game::player::{Act, ActKind};
use crate::game::spells::Spell;
use crate::game::world::{Floor, Obj, WATERED};
use crate::game::{Game, Io, State};
use crate::input::Input;
use crate::render::Renderer;
use crate::shot::save_png;
use crate::util::Rng;

const W: usize = 480;
const H: usize = 270;

fn tick(game: &mut Game, input: &Input, audio: &Audio, frames: usize) {
    for _ in 0..frames {
        let mut io = Io {
            dt: 1.0 / 60.0,
            input,
            audio,
            view: (W, H),
            quit: false,
            toggle_fullscreen: false,
        };
        game.update(&mut io);
    }
}

fn snap(game: &mut Game, r: &mut Renderer, input: &Input, dir: &Path, name: &str) {
    if let Some(p) = game.play_mut() {
        p.player.hurt = 0.0;
        if p.area == crate::game::world::Area::Home {
            // The house is framed like a diorama, as in the game.
            let c = p.room_center();
            p.cam_pos = c + (p.player.world_pos() - c) * glam::Vec3::new(0.15, 0.0, 0.08);
            p.cam.dist = 17.5;
        } else {
            p.cam_pos = p.player.world_pos();
        }
        p.cam.target = p.cam_pos;
        p.cam.update(W, H);
    }
    game.draw(r, input);
    let path = dir.join(format!("{name}.png"));
    match save_png(&path, &r.fb, 2) {
        Ok(()) => println!("wrote {}", path.display()),
        Err(e) => eprintln!("failed to write {}: {e}", path.display()),
    }
}

fn play(game: &mut Game) -> &mut Play {
    game.play_mut().expect("playing")
}

/// Jumps straight to a Hollow floor, skipping the fade.
fn descend(game: &mut Game, input: &Input, audio: &Audio, depth: u32, waystone: bool) {
    let p = play(game);
    p.fade = None;
    p.start_fade(Trans::Descend {
        depth,
        via_waystone: waystone,
    });
    tick(game, input, audio, 60);
    let p = play(game);
    p.banner = None;
}

/// A piece of gear with hand-picked rolls, for showing off.
fn fancy(item: Item, level: u16, affixes: &[(Stat, i16)], enchants: &[(Stat, i16)]) -> Stack {
    let mut g = Gear::plain(level);
    g.quality = 92;
    for (i, (stat, val)) in affixes.iter().enumerate() {
        g.affixes[i] = Some(Affix {
            stat: *stat,
            val: *val,
        });
    }
    for (i, (stat, val)) in enchants.iter().enumerate() {
        g.enchants[i] = Some(Affix {
            stat: *stat,
            val: *val,
        });
    }
    g.update_rarity();
    Stack::with_gear(item, g)
}

/// Dresses the hero.
fn wear(p: &mut Play, items: &[Item]) {
    p.player.equip = [None; 5];
    for &it in items {
        if let Some(slot) = it.class().and_then(|c| c.slot()) {
            p.player.equip[slot as usize] = Some(Stack::new(it, 1));
        }
    }
    p.player.refresh();
}

pub fn shots(dir: &str) {
    let dir = Path::new(dir);
    if let Err(e) = std::fs::create_dir_all(dir) {
        eprintln!("cannot create {}: {e}", dir.display());
        return;
    }
    // Keep screenshots away from real save files.
    // SAFETY: set before any other thread reads the environment.
    unsafe {
        std::env::set_var("HOLLOWBLOOM_DATA", dir.join("data"));
    }
    let audio = Audio::silent();
    let mut input = Input::default();
    let mut r = Renderer::new(W, H);
    let mut game = Game::new();

    // Title.
    tick(&mut game, &input, &audio, 90);
    input.mouse = Vec2::new(-10.0, -10.0);
    snap(&mut game, &mut r, &input, dir, "01_title");

    // A fresh farm, and the welcome.
    game.new_game(20260928);
    tick(&mut game, &input, &audio, 400);
    snap(&mut game, &mut r, &input, dir, "01b_welcome");
    // Set up to show the crops at every stage.
    {
        let p = play(&mut game);
        p.banner = None;
        p.menu = Menu::None;
        let crops = [
            Crop::Turnip,
            Crop::CaveCarrot,
            Crop::Glowcap,
            Crop::CrystalBerry,
            Crop::MossMelon,
            Crop::SporePumpkin,
            Crop::EmberPepper,
            Crop::FrostLily,
            Crop::Moonbloom,
        ];
        for (i, c) in crops.iter().enumerate() {
            let x = 22 + i as i32;
            for (k, z) in (15..18).enumerate() {
                p.farm.set_floor(x, z, Floor::Tilled);
                let days = [c.def().days, c.def().days / 2, 1][k];
                p.farm.set_obj(
                    x,
                    z,
                    Some(Obj::Crop {
                        crop: *c,
                        days,
                        harvested: false,
                    }),
                );
                if k == 0 {
                    p.farm.set_flag(x, z, WATERED, true);
                }
            }
        }
        p.farm.set_obj(21, 16, Some(Obj::Sprinkler { tier: 0 }));
        p.farm.set_obj(
            20,
            12,
            Some(Obj::Chest {
                items: vec![None; 30],
            }),
        );
        p.farm.set_obj(19, 14, Some(Obj::Lamp));
        for x in 18..22 {
            p.farm.set_obj(x, 18, Some(Obj::Fence));
        }
        p.player.pos = Vec2::new(26.5, 13.4);
        p.player.facing = Vec2::new(0.0, 1.0);
        p.player.sel = 1;
        p.clock.min = 8.0 * 60.0;
    }
    tick(&mut game, &input, &audio, 40);
    {
        let p = play(&mut game);
        p.player.act = Some(Act::new(ActKind::Till, 0.38, (26, 14), Vec2::new(0.0, 1.0)));
        p.player.act.as_mut().unwrap().t = 0.14;
    }
    snap(&mut game, &mut r, &input, dir, "02_farm_morning");

    {
        let p = play(&mut game);
        p.player.act = None;
        p.clock.min = 18.4 * 60.0;
        p.player.pos = Vec2::new(30.5, 12.5);
    }
    tick(&mut game, &input, &audio, 30);
    snap(&mut game, &mut r, &input, dir, "03_farm_sunset");

    {
        let p = play(&mut game);
        p.clock.min = 22.0 * 60.0;
        p.player.pos = Vec2::new(24.5, 13.0);
    }
    tick(&mut game, &input, &audio, 30);
    snap(&mut game, &mut r, &input, dir, "04_farm_night");

    // A big market garden: every crop there is, ripe.
    {
        let p = play(&mut game);
        p.clock.min = 10.0 * 60.0;
        for (i, c) in ALL_CROPS.iter().enumerate() {
            let (x, z) = (34 + (i % 10) as i32, 22 + (i / 10) as i32 * 2);
            for dz in 0..2 {
                p.farm.set_obj(x, z + dz, None);
                p.farm.set_wall(x, z + dz, crate::game::world::Wall::None);
                p.farm.set_floor(x, z + dz, Floor::Tilled);
                p.farm.set_flag(x, z + dz, WATERED, dz == 0);
            }
            p.farm.set_obj(
                x,
                z,
                Some(Obj::Crop {
                    crop: *c,
                    days: c.def().days,
                    harvested: false,
                }),
            );
            p.farm.set_obj(
                x,
                z + 1,
                Some(Obj::Crop {
                    crop: *c,
                    days: c.def().days * 2 / 3,
                    harvested: false,
                }),
            );
        }
        wear(
            p,
            &[
                Item::FlowerCrown,
                Item::CozySweater,
                Item::PumpkinBloomers,
                Item::FrogSlippers,
            ],
        );
        p.player.inv.slots[8] = Some(Stack::new(Item::GoldSickle, 1));
        p.player.sel = 8;
        p.player.pos = Vec2::new(38.5, 21.2);
        p.player.facing = Vec2::new(0.0, 1.0);
    }
    tick(&mut game, &input, &audio, 20);
    snap(&mut game, &mut r, &input, dir, "05_market_garden");

    // The bag: worn gear, loot and a legendary tooltip.
    let legendary = fancy(
        Item::StarlightSword,
        48,
        &[
            (Stat::Damage, 34),
            (Stat::Crit, 9),
            (Stat::Lifesteal, 5),
            (Stat::Shock, 18),
        ],
        &[(Stat::Haste, 11), (Stat::Burn, 20)],
    );
    {
        let p = play(&mut game);
        p.clock.min = 11.0 * 60.0;
        let mut rng = Rng::new(77);
        let f = Fortune {
            luck: 0.6,
            greed: 1.0,
        };
        let mut k = 10;
        for _ in 0..12 {
            p.player.inv.slots[k] = Some(loot::random_gear(30, f, &mut rng));
            k += 1;
        }
        for _ in 0..4 {
            p.player.inv.slots[k] = Some(loot::random_scroll(30, f, &mut rng));
            k += 1;
        }
        for (item, n) in [
            (Item::Ruby, 2),
            (Item::Moonstone, 1),
            (Item::RubberDuck, 1),
            (Item::Strawberry, 9),
            (Item::Shortcake, 2),
            (Item::GoldCoin, 1),
        ] {
            p.player.inv.slots[k] = Some(Stack::new(item, n));
            k += 1;
        }
        p.player.inv.slots[k] = Some(legendary);
        wear(
            p,
            &[
                Item::WizardHat,
                Item::MageRobe,
                Item::StarryLeggings,
                Item::FeatherBoots,
                Item::CrystalAegis,
            ],
        );
        p.money = 12_345;
        p.menu = Menu::inventory(false);
    }
    let l = menus::panel_layout(W as i32, H as i32);
    let g = menus::bag_grid(&l, 22);
    let slot = play(&mut game)
        .player
        .inv
        .slots
        .iter()
        .position(|s| *s == Some(legendary))
        .unwrap();
    let (sx, sy) = g.slot_pos(slot);
    input.mouse = Vec2::new(sx as f32 + 8.0, sy as f32 + 8.0);
    input.mouse_inside = false;
    tick(&mut game, &input, &audio, 5);
    snap(&mut game, &mut r, &input, dir, "06_bag_and_gear");
    {
        let p = play(&mut game);
        p.player.buffs.clear();
        p.player.add_buff(
            crate::game::items::Buff {
                stat: Stat::Luck,
                val: 12,
                secs: 240,
            },
            Item::Shortcake,
        );
        p.player.refresh();
        p.menu = Menu::Inventory {
            tab: Tab::Stats,
            cursor: 0,
            recipe: 0,
            scroll: 0,
            cat: 0,
        };
    }
    input.mouse = Vec2::new(-10.0, -10.0);
    tick(&mut game, &input, &audio, 5);
    snap(&mut game, &mut r, &input, dir, "07_stats");
    {
        let p = play(&mut game);
        p.menu = Menu::Inventory {
            tab: Tab::Craft,
            cursor: 0,
            recipe: 3,
            scroll: 0,
            cat: 2,
        };
        p.player.inv.add(Item::CopperOre, 30);
        p.player.inv.add(Item::Wood, 20);
    }
    tick(&mut game, &input, &audio, 5);
    snap(&mut game, &mut r, &input, dir, "08_crafting");
    {
        let p = play(&mut game);
        p.deepest = 34;
        p.menu = Menu::Shop {
            at: None,
            tab: ShopTab::Specials,
            cursor: 0,
            scroll: 0,
        };
    }
    tick(&mut game, &input, &audio, 5);
    snap(&mut game, &mut r, &input, dir, "09_shop_specials");

    // The enchanting table by the house.
    {
        let p = play(&mut game);
        let (ex, ez) = crate::game::farm::MARKS.enchant;
        p.player.pos = Vec2::new(ex as f32 - 0.5, ez as f32 + 1.4);
        p.player.facing = Vec2::new(0.7, -0.7).normalize();
        let sword = p
            .player
            .inv
            .slots
            .iter()
            .position(|s| {
                s.is_some_and(|s| s.item.class() == Some(crate::game::gear::Class::Sword))
            })
            .unwrap_or(10);
        let scroll = loot::scroll_of(
            crate::game::gear::Group::Weapon,
            30,
            Fortune {
                luck: 2.0,
                greed: 1.0,
            },
            &mut Rng::new(3),
        );
        p.player.inv.slots[39] = Some(scroll);
        p.menu = Menu::Enchant {
            gear: Some(Pick::Bag(sword)),
            scroll: Some(39),
            socket: 0,
            cursor: 48,
            msg: None,
            glow: 0.0,
        };
    }
    tick(&mut game, &input, &audio, 5);
    snap(&mut game, &mut r, &input, dir, "10_enchanting");
    {
        let p = play(&mut game);
        p.menu = Menu::None;
        p.clock.min = 17.0 * 60.0;
    }
    tick(&mut game, &input, &audio, 5);
    snap(&mut game, &mut r, &input, dir, "11_enchanting_table");

    // The Hollow, biome by biome, with a different outfit and weapon each time.
    let outfits: [(&[Item], Item, ActKind); 6] = [
        (
            &[
                Item::StrawHat,
                Item::FarmerTunic,
                Item::PatchedTrousers,
                Item::RainBoots,
                Item::PotLid,
            ],
            Item::CarrotBlade,
            ActKind::Slash,
        ),
        (
            &[
                Item::CatHood,
                Item::LeafTunic,
                Item::GrassSkirt,
                Item::BunnySlippers,
            ],
            Item::BubbleWand,
            ActKind::Bolt,
        ),
        (
            &[
                Item::MushroomCap,
                Item::WoollyPoncho,
                Item::LeatherLeggings,
                Item::LeatherBoots,
                Item::MushroomShield,
            ],
            Item::MushroomStaff,
            ActKind::Blast,
        ),
        (
            &[
                Item::EmberCrown,
                Item::EmberPlate,
                Item::EmberGreaves,
                Item::EmberTreads,
                Item::EmberBulwark,
            ],
            Item::Sword5,
            ActKind::Slash,
        ),
        (
            &[
                Item::FrogHood,
                Item::FrogRaincoat,
                Item::FrostLeggings,
                Item::FrogSlippers,
                Item::TurtleShell,
            ],
            Item::FrostWand,
            ActKind::Bolt,
        ),
        (
            &[
                Item::FrostHelm,
                Item::CrystalMail,
                Item::CrystalGreaves,
                Item::CrystalBoots,
                Item::CrystalAegis,
            ],
            Item::MoonbloomStaff,
            ActKind::Blast,
        ),
    ];
    for (i, depth) in [1u32, 14, 25, 33, 44, 56].into_iter().enumerate() {
        descend(&mut game, &input, &audio, depth, false);
        {
            let p = play(&mut game);
            let (fit, weapon, kind) = outfits[i];
            wear(p, fit);
            p.player.inv.slots[0] = Some(loot::roll_gear(
                weapon,
                depth as u16,
                0.8,
                &mut Rng::new(depth as u64),
            ));
            p.player.sel = 0;
            p.player.base_hp = 200;
            p.player.hp = 200;
            p.player.level = 12;
            p.player.mana = 99.0;
            // Pull a few enemies close so the shot has some life in it.
            let pp = p.player.pos;
            let mut k = 0;
            for f in p.foes.iter_mut() {
                if k < 3 && !f.boss {
                    let a = k as f32 * 2.1 + 0.6;
                    let target = pp + Vec2::new(a.cos(), a.sin()) * 2.3;
                    if !p
                        .level
                        .as_ref()
                        .unwrap()
                        .world
                        .blocked(target.x as i32, target.y as i32)
                    {
                        f.pos = target;
                        f.alert = true;
                        k += 1;
                    }
                }
            }
            let dir = Vec2::new(0.6, 0.8).normalize();
            p.player.facing = dir;
            p.player.hurt = 5.0;
            match kind {
                ActKind::Bolt => {
                    p.cast_bolt(
                        dir,
                        &mut Io {
                            dt: 1.0 / 60.0,
                            input: &input,
                            audio: &audio,
                            view: (W, H),
                            quit: false,
                            toggle_fullscreen: false,
                        },
                    );
                    let mut a = Act::new(kind, 0.34, p.player.tile(), dir);
                    a.t = 0.12;
                    a.fired = true;
                    p.player.act = Some(a);
                }
                ActKind::Blast => {
                    let mut a = Act::new(kind, 0.55, p.player.tile(), dir);
                    a.t = 0.2;
                    p.player.act = Some(a);
                }
                _ => {
                    let mut a = Act::new(kind, 0.3, p.player.tile(), dir);
                    a.t = 0.09;
                    a.fired = true;
                    p.player.act = Some(a);
                }
            }
            // Scatter a little loot.
            let at = p.player.world_pos() + glam::Vec3::new(-1.2, 0.0, 0.8);
            let mut rng = Rng::new(depth as u64 * 3);
            let mut goodies = loot::coin_stacks(234 + depth as u64 * 10);
            goodies.push(loot::random_gear(
                depth,
                Fortune {
                    luck: 3.0,
                    greed: 1.0,
                },
                &mut rng,
            ));
            for s in goodies {
                let mut d = crate::game::fx::Drop::new(s, at, &mut rng);
                d.pos += glam::Vec3::new(rng.range_f(-0.6, 0.6), 0.0, rng.range_f(-0.4, 0.4));
                d.vel = glam::Vec3::ZERO;
                d.pos.y = 0.1;
                d.age = 1.0;
                p.drops.push(d);
            }
        }
        tick(&mut game, &input, &audio, 3);
        snap(
            &mut game,
            &mut r,
            &input,
            dir,
            &format!("{:02}_hollow_floor_{depth}", 12 + i),
        );
    }

    // Hidden behind a wall: the hero shows through as a soft silhouette.
    descend(&mut game, &input, &audio, 3, false);
    {
        let p = play(&mut game);
        p.foes.clear();
        p.drops.clear();
        let w = &p.level.as_ref().unwrap().world;
        let mut spot = None;
        'find: for z in 4..w.h - 4 {
            for x in 4..w.w - 4 {
                let wall = |dz: i32| w.wall(x, z + dz) != crate::game::world::Wall::None;
                if !w.blocked(x, z)
                    && wall(1)
                    && !w.blocked(x - 1, z)
                    && !w.blocked(x + 1, z)
                    && !w.blocked(x, z - 1)
                {
                    spot = Some((x, z));
                    break 'find;
                }
            }
        }
        if let Some((x, z)) = spot {
            p.player.pos = Vec2::new(x as f32 + 0.5, z as f32 + 0.72);
            p.player.facing = Vec2::new(0.0, 1.0);
            p.player.act = None;
        }
    }
    tick(&mut game, &input, &audio, 2);
    snap(&mut game, &mut r, &input, dir, "18_behind_wall");

    // A guardian floor.
    descend(&mut game, &input, &audio, 10, false);
    {
        let p = play(&mut game);
        wear(
            p,
            &[
                Item::BunnyHood,
                Item::CopperMail,
                Item::CopperGreaves,
                Item::CopperSabatons,
                Item::CopperShield,
            ],
        );
        p.player.inv.slots[0] = Some(Stack::new(Item::MightyLeek, 1));
        let boss = p.foes.iter().position(|f| f.boss);
        if let (Some(b), Some(l)) = (boss, &p.level) {
            let (sx, sz) = l.stairs;
            let (px, pz) = l.world.nearest_open(sx, sz + 3);
            p.player.pos = Vec2::new(px as f32 + 0.5, pz as f32 + 0.5);
            let (bx, bz) = l.world.nearest_open(sx + 1, sz + 1);
            p.foes[b].pos = Vec2::new(bx as f32 + 0.5, bz as f32 + 0.5);
            p.foes[b].alert = true;
            p.foes[b].hp = p.foes[b].max_hp * 2 / 3;
            p.player.facing = (p.foes[b].pos - p.player.pos).normalize_or_zero();
        }
        p.player.hurt = 5.0;
    }
    tick(&mut game, &input, &audio, 3);
    snap(&mut game, &mut r, &input, dir, "19_guardian");

    // Down it goes: a shower of loot.
    {
        let p = play(&mut game);
        if let Some(b) = p.foes.iter().position(|f| f.boss) {
            p.foes[b].hp = 0;
            p.player.pos = p.foes[b].pos + Vec2::new(0.0, 2.2);
            p.player.facing = Vec2::new(0.0, -1.0);
            let mut io = Io {
                dt: 1.0 / 60.0,
                input: &input,
                audio: &audio,
                view: (W, H),
                quit: false,
                toggle_fullscreen: false,
            };
            p.reap(&mut io);
        }
        p.foes.clear();
        p.banner = None;
    }
    tick(&mut game, &input, &audio, 50);
    snap(&mut game, &mut r, &input, dir, "20_guardian_loot");

    // The same floor once the guardian is gone: the waystone haven.
    {
        let p = play(&mut game);
        p.drops.clear();
        if let Some(l) = &p.level {
            if let Some((wx, wz)) = l.waystone {
                p.player.pos = Vec2::new(wx as f32 + 1.5, wz as f32 + 1.8);
                p.player.facing = Vec2::new(-0.5, -1.0).normalize();
            }
        }
        p.menu = Menu::dialog_choice(
            "The waystone hums warmly. Return to the surface?",
            vec![
                ("Go home", crate::game::menus::Choice::ReturnHome),
                ("Keep delving", crate::game::menus::Choice::Close),
            ],
        );
    }
    tick(&mut game, &input, &audio, 120);
    snap(&mut game, &mut r, &input, dir, "21_waystone");

    // The morning report after a night's sleep.
    {
        let p = play(&mut game);
        p.menu = Menu::None;
        p.fade = None;
        p.start_fade(Trans::Home);
    }
    tick(&mut game, &input, &audio, 60);
    {
        let p = play(&mut game);
        p.shipping.push(Stack::new(Item::Glowcap, 6));
        p.shipping.push(Stack::new(Item::CrystalBerry, 9));
        p.shipping.push(Stack::new(Item::Starfruit, 2));
        p.fade = None;
        p.start_fade(Trans::Sleep { passed_out: false });
    }
    tick(&mut game, &input, &audio, 60);
    snap(&mut game, &mut r, &input, dir, "22_new_day");
    if let State::Play(p) = &game.state {
        println!("day {} money {}", p.clock.day, loot::money_text(p.money));
    }
    let _ = (Slot::Head, Rarity::Common);
    // The road to town, Bramblewick and its people.
    town_shots(dir.to_str().unwrap_or("shots"));
    folk_shots(dir.to_str().unwrap_or("shots"));
    magic_shots(dir.to_str().unwrap_or("shots"));
}

/// Jelly, sparks, the breeze in the grass, spells and potions.
pub fn magic_shots(dir: &str) {
    let dir = Path::new(dir);
    let _ = std::fs::create_dir_all(dir);
    let audio = Audio::silent();
    let input = Input::default();
    let mut r = Renderer::new(W, H);
    let mut game = Game::new();
    game.new_game(77);
    tick(&mut game, &input, &audio, 30);
    {
        let p = play(&mut game);
        p.menu = Menu::None;
        p.banner = None;
    }

    // A wobbly gathering: a slime from every biome, and a ghost drifting through.
    descend(&mut game, &input, &audio, 2, false);
    {
        let p = play(&mut game);
        p.foes.clear();
        p.drops.clear();
        let w = &p.level.as_ref().unwrap().world;
        let (px, pz) = p.player.tile();
        let mut spots = Vec::new();
        for dz in -3..=3 {
            for dx in -4..=4 {
                let (x, z) = (px + dx, pz + dz);
                if (dx, dz) != (0, 0) && !w.blocked(x, z) && (dx + dz) % 2 == 0 {
                    spots.push((x, z));
                }
            }
        }
        for (b, &(x, z)) in spots.iter().take(6).enumerate() {
            let mut f = crate::game::foes::Enemy::new(
                crate::game::dungeon::Foe::Slime,
                x as f32 + 0.5,
                z as f32 + 0.5,
                2,
                b,
                false,
                b as u32 * 7,
            );
            f.yaw = (p.player.pos.x - f.pos.x).atan2(p.player.pos.y - f.pos.y);
            f.anim = b as f32 * 0.9;
            p.foes.push(f);
        }
        if let Some(&(x, z)) = spots.get(7) {
            let mut g = crate::game::foes::Enemy::new(
                crate::game::dungeon::Foe::Ghost,
                x as f32 + 0.5,
                z as f32 + 0.5,
                2,
                2,
                false,
                99,
            );
            g.yaw = 0.4;
            p.foes.push(g);
        }
        p.player.facing = Vec2::new(0.0, 1.0);
    }
    tick(&mut game, &input, &audio, 2);
    {
        let p = play(&mut game);
        for f in p.foes.iter_mut() {
            f.alert = false;
        }
    }
    snap(&mut game, &mut r, &input, dir, "m01_jelly_slimes");

    // Steel on stone: sparks fly and light up the dark.
    {
        let p = play(&mut game);
        p.foes.clear();
        let at = p.player.world_pos() + glam::Vec3::new(0.4, 0.5, 0.9);
        for k in 0..3 {
            p.fx.sparks(
                at + glam::Vec3::new(k as f32 * 0.2, 0.0, 0.0),
                glam::Vec3::new(-0.3, 0.0, -1.0),
                12,
            );
        }
    }
    tick(&mut game, &input, &audio, 9);
    snap(&mut game, &mut r, &input, dir, "m02_sparks");

    // A breezy afternoon on the farm: tall grass, weeds and wild bushes.
    {
        let p = play(&mut game);
        p.fade = None;
        p.start_fade(Trans::Home);
    }
    tick(&mut game, &input, &audio, 60);
    {
        let p = play(&mut game);
        p.clock.min = 14.0 * 60.0;
        p.player.pos = Vec2::new(40.5, 30.5);
        p.player.facing = Vec2::new(0.0, 1.0);
        p.banner = None;
    }
    tick(&mut game, &input, &audio, 40);
    snap(&mut game, &mut r, &input, dir, "m03_breezy_farm");

    // Brewing: the Potions page of the crafting book.
    {
        let p = play(&mut game);
        for (item, n) in [
            (Item::Vial, 9),
            (Item::Heartleaf, 5),
            (Item::Blueberry, 6),
            (Item::SlimeGel, 4),
        ] {
            p.player.inv.add(item, n);
        }
        let cat = 1 + crate::game::items::CATS
            .iter()
            .position(|c| *c == crate::game::items::Cat::Potions)
            .unwrap();
        p.menu = Menu::Inventory {
            tab: Tab::Craft,
            cursor: 0,
            recipe: 1,
            scroll: 0,
            cat,
        };
    }
    tick(&mut game, &input, &audio, 3);
    snap(&mut game, &mut r, &input, dir, "m04_brewing_potions");

    // The Starfall Spellery, in town.
    {
        let p = play(&mut game);
        p.menu = Menu::None;
        p.clock.min = 16.5 * 60.0;
        p.ride_bus(true);
        p.bus = None;
        p.player.pos = Vec2::new(54.5, 42.6);
        p.player.facing = Vec2::new(0.0, -1.0);
    }
    tick(&mut game, &input, &audio, 30);
    play(&mut game).banner = None;
    snap(&mut game, &mut r, &input, dir, "m05_spellery_outside");
    {
        let p = play(&mut game);
        p.enter_place(crate::game::town::Place::Spellery);
    }
    tick(&mut game, &input, &audio, 40);
    {
        let p = play(&mut game);
        let (kx, kz) = p.room.as_ref().map_or((7, 1), |r| r.keeper);
        p.player.pos = Vec2::new(kx as f32 + 0.5, kz as f32 + 2.6);
        p.player.facing = Vec2::new(0.0, -1.0);
    }
    tick(&mut game, &input, &audio, 10);
    snap(&mut game, &mut r, &input, dir, "m06_spellery_inside");
    // Meeting Hazel: a free Firebolt.
    {
        let p = play(&mut game);
        if let Some(i) = p
            .folk
            .iter()
            .position(|n| n.who == crate::game::folk::Villager::Hazel)
        {
            let mut io = Io {
                dt: 1.0 / 60.0,
                input: &input,
                audio: &audio,
                view: (W, H),
                quit: false,
                toggle_fullscreen: false,
            };
            p.talk_to(i, &mut io);
        }
    }
    tick(&mut game, &input, &audio, 400);
    snap(&mut game, &mut r, &input, dir, "m07_meet_hazel");
    {
        let p = play(&mut game);
        p.money = 25_000;
        p.deepest = 26;
        for sp in [Spell::Mend, Spell::Bloom, Spell::ChainSpark] {
            p.spells.learn(sp);
        }
        p.spells.get_mut(Spell::Firebolt).unwrap().practise(130);
        p.spells.get_mut(Spell::Mend).unwrap().practise(30);
        p.menu = Menu::Spells { tab: 0, sel: 5 };
    }
    tick(&mut game, &input, &audio, 2);
    snap(&mut game, &mut r, &input, dir, "m08_spells_learn");
    {
        let p = play(&mut game);
        p.menu = Menu::Spells { tab: 1, sel: 3 };
    }
    tick(&mut game, &input, &audio, 2);
    snap(&mut game, &mut r, &input, dir, "m09_attuning_circle");

    // Spells in the Hollow.
    {
        let p = play(&mut game);
        p.menu = Menu::None;
        p.leave_place();
        p.spells.learn(Spell::Starfall);
        p.spells.learn(Spell::Ward);
        p.spells.learn(Spell::FrostNova);
        p.player.base_mana = 400;
        p.player.level = 14;
        p.fade = None;
        p.start_fade(Trans::Descend {
            depth: 14,
            via_waystone: false,
        });
    }
    tick(&mut game, &input, &audio, 60);
    let casts: [(&str, Spell, usize); 5] = [
        ("m10_firebolt", Spell::Firebolt, 9),
        ("m11_chain_spark", Spell::ChainSpark, 22),
        ("m12_starfall", Spell::Starfall, 42),
        ("m13_frost_nova", Spell::FrostNova, 26),
        ("m14_ward", Spell::Ward, 30),
    ];
    for (name, spell, wait) in casts {
        {
            let p = play(&mut game);
            p.banner = None;
            p.player.mana = 400.0;
            p.player.hp = p.player.max_hp();
            p.spells.slots = [Some(spell), Some(Spell::Mend)];
            p.spells.cool = [0.0; 2];
            p.player.act = None;
            // Bring a few foes close.
            let pp = p.player.pos;
            let mut k = 0;
            for f in p.foes.iter_mut() {
                if k < 4 && !f.boss {
                    let a = k as f32 * 1.1 - 0.4;
                    let target = pp + Vec2::new(a.cos(), a.sin() * 0.8 + 0.6) * 2.6;
                    if !p
                        .level
                        .as_ref()
                        .unwrap()
                        .world
                        .blocked(target.x as i32, target.y as i32)
                    {
                        f.pos = target;
                        f.alert = true;
                        f.hp = f.max_hp * 10;
                        k += 1;
                    }
                }
            }
            p.player.facing = Vec2::new(0.4, 0.9).normalize();
            p.aim = None;
            let mut io = Io {
                dt: 1.0 / 60.0,
                input: &input,
                audio: &audio,
                view: (W, H),
                quit: false,
                toggle_fullscreen: false,
            };
            p.begin_cast(0, &mut io);
        }
        tick(&mut game, &input, &audio, wait);
        snap(&mut game, &mut r, &input, dir, name);
    }

    // Bloom on the farm: water and a growth spurt.
    {
        let p = play(&mut game);
        p.fade = None;
        p.start_fade(Trans::Home);
    }
    tick(&mut game, &input, &audio, 60);
    {
        let p = play(&mut game);
        p.clock.min = 11.0 * 60.0;
        p.banner = None;
        p.player.pos = Vec2::new(26.5, 14.5);
        p.player.facing = Vec2::new(0.0, 1.0);
        for z in 13..18 {
            for x in 23..30 {
                if p.farm.obj(x, z).is_none() {
                    p.farm.set_floor(x, z, Floor::Tilled);
                    p.farm.set_obj(
                        x,
                        z,
                        Some(Obj::Crop {
                            crop: Crop::Strawberry,
                            days: 3,
                            harvested: false,
                        }),
                    );
                }
            }
        }
        p.spells.slots = [Some(Spell::Bloom), Some(Spell::Mend)];
        p.spells.get_mut(Spell::Bloom).unwrap().practise(400);
        p.spells.cool = [0.0; 2];
        p.player.mana = 300.0;
        let mut io = Io {
            dt: 1.0 / 60.0,
            input: &input,
            audio: &audio,
            view: (W, H),
            quit: false,
            toggle_fullscreen: false,
        };
        p.begin_cast(0, &mut io);
    }
    tick(&mut game, &input, &audio, 16);
    snap(&mut game, &mut r, &input, dir, "m15_bloom");
}

/// Renders the hero in an outfit into a cell of a sheet.
#[allow(clippy::too_many_arguments)]
fn portrait(
    a: &crate::assets::Assets,
    r: &mut Renderer,
    sheet: &mut crate::render::Frame,
    (x0, y0): (usize, usize),
    equip: &[Option<Stack>; 5],
    held: Option<&Stack>,
    yaw: f32,
    bg: u8,
) {
    portrait_from(a, r, sheet, (x0, y0), equip, held, yaw, bg, 30.0, None);
}

/// Like `portrait`, looking down at `pitch` degrees, and optionally as a villager.
#[allow(clippy::too_many_arguments)]
fn portrait_from(
    a: &crate::assets::Assets,
    r: &mut Renderer,
    sheet: &mut crate::render::Frame,
    (x0, y0): (usize, usize),
    equip: &[Option<Stack>; 5],
    held: Option<&Stack>,
    yaw: f32,
    bg: u8,
    pitch: f32,
    who: Option<crate::game::folk::Villager>,
) {
    use crate::game::draw::{Pose, draw_humanoid};
    use crate::game::scene::dress;
    use crate::render::{DrawOpts, Light, Mode};
    use glam::Vec3;

    let (cw, ch) = (r.width(), r.height());
    r.cam.target = Vec3::new(0.0, 0.5, 0.0);
    r.cam.pitch = pitch.to_radians();
    r.cam.dist = 3.3;
    r.cam.update(cw, ch);
    r.fb.clear(bg);
    r.remap.clear();
    let o = DrawOpts {
        light: Light::Fixed(1.0, crate::palette::NEUTRAL),
        mode: Mode::Lit,
        tag: 1,
        ..Default::default()
    };
    match who {
        Some(v) => {
            let (dressed, remap) = crate::game::scene::villager_dress(a, v);
            draw_humanoid(
                r,
                a,
                &a.folk[v as usize],
                Vec3::ZERO,
                yaw,
                &Pose::default(),
                &o,
                &dressed.outfit_with(&remap),
            );
        }
        None => {
            let dressed = dress(a, equip, held);
            draw_humanoid(
                r,
                a,
                &a.hero,
                Vec3::ZERO,
                yaw,
                &Pose::default(),
                &o,
                &dressed.outfit(Some(&a.sprout)),
            );
        }
    }
    r.fb.outline(crate::palette::INK);
    for y in 0..ch {
        for x in 0..cw {
            sheet.color[(y0 + y) * sheet.w + x0 + x] = r.fb.color[y * cw + x];
        }
    }
}

fn outfit(items: &[Item]) -> [Option<Stack>; 5] {
    let mut e = [None; 5];
    for &it in items {
        if let Some(slot) = it.class().and_then(|c| c.slot()) {
            e[slot as usize] = Some(Stack::new(it, 1));
        }
    }
    e
}

/// `--wardrobe FILE`: the hero in a dozen themed outfits, and next to it (`FILE` with
/// `_all` added) a contact sheet of every single piece of gear.
pub fn wardrobe(path: &str) {
    use crate::game::items::ALL_ITEMS;
    use crate::palette::*;
    use crate::render::Frame;
    use crate::ui::Canvas;

    let a = crate::assets::Assets::new();
    let looks: [(&str, &[Item], Item); 12] = [
        (
            "Farmer",
            &[
                Item::StrawHat,
                Item::FarmerTunic,
                Item::PatchedTrousers,
                Item::RainBoots,
            ],
            Item::Hoe,
        ),
        (
            "Frog Friend",
            &[
                Item::FrogHood,
                Item::FrogRaincoat,
                Item::FrostLeggings,
                Item::FrogSlippers,
            ],
            Item::FrogCan,
        ),
        (
            "Kitty Knight",
            &[
                Item::CatHood,
                Item::CopperMail,
                Item::CopperGreaves,
                Item::CopperSabatons,
                Item::CopperShield,
            ],
            Item::CarrotBlade,
        ),
        (
            "Bunny Mage",
            &[
                Item::BunnyHood,
                Item::MageRobe,
                Item::StarryLeggings,
                Item::BunnySlippers,
            ],
            Item::CandyWand,
        ),
        (
            "Stargazer",
            &[
                Item::WizardHat,
                Item::StarRobe,
                Item::StarryLeggings,
                Item::FeatherBoots,
            ],
            Item::MoonpetalWand,
        ),
        (
            "Mushroom Scout",
            &[
                Item::MushroomCap,
                Item::LeafTunic,
                Item::GrassSkirt,
                Item::LeatherBoots,
                Item::MushroomShield,
            ],
            Item::MushroomStaff,
        ),
        (
            "Garden Party",
            &[
                Item::FlowerCrown,
                Item::CozySweater,
                Item::PumpkinBloomers,
                Item::FrogSlippers,
            ],
            Item::SunflowerStaff,
        ),
        (
            "Miner",
            &[
                Item::MinerHat,
                Item::LeatherVest,
                Item::LeatherLeggings,
                Item::IronBoots,
                Item::PotLid,
            ],
            Item::MolePick,
        ),
        (
            "Frost Knight",
            &[
                Item::FrostHelm,
                Item::FrostCoat,
                Item::FrostLeggings,
                Item::FrostWalkers,
                Item::FrostWard,
            ],
            Item::FrostFang,
        ),
        (
            "Ember Lord",
            &[
                Item::EmberCrown,
                Item::EmberPlate,
                Item::EmberGreaves,
                Item::EmberTreads,
                Item::EmberBulwark,
            ],
            Item::Sword5,
        ),
        (
            "Crystal Paladin",
            &[
                Item::CrystalCirclet,
                Item::CrystalMail,
                Item::CrystalGreaves,
                Item::CrystalBoots,
                Item::CrystalAegis,
            ],
            Item::StarlightSword,
        ),
        (
            "Turtle Tank",
            &[
                Item::IronHelm,
                Item::IronPlate,
                Item::IronGreaves,
                Item::IronBoots,
                Item::TurtleShell,
            ],
            Item::MightyLeek,
        ),
    ];
    let (cw, ch) = (88usize, 112usize);
    let cols = 6;
    let mut sheet = Frame::new(cols * cw, looks.len().div_ceil(cols) * ch);
    sheet.clear(INK);
    let mut r = Renderer::new(cw, ch - 12);
    let bgs = [SLATE, DEEP_TEAL, GRAPE, INDIGO, PLUM, SHADOW];
    for (k, (_, items, held)) in looks.iter().enumerate() {
        let (x0, y0) = ((k % cols) * cw, (k / cols) * ch);
        let s = Stack::new(*held, 1);
        portrait(
            &a,
            &mut r,
            &mut sheet,
            (x0, y0),
            &outfit(items),
            Some(&s),
            0.5,
            bgs[k % bgs.len()],
        );
    }
    let font = crate::assets::font::Font::regular();
    let darken = r.sh.darken;
    {
        let mut c = Canvas {
            fb: &mut sheet,
            font: &font,
            darken: &darken,
        };
        for (k, (name, _, _)) in looks.iter().enumerate() {
            let (x0, y0) = ((k % cols) * cw, (k / cols) * ch);
            c.text_center(
                x0 as i32 + cw as i32 / 2,
                (y0 + ch - 11) as i32,
                name,
                CREAM,
            );
        }
    }
    match save_png(Path::new(path), &sheet, 2) {
        Ok(()) => println!("wrote {path}"),
        Err(e) => eprintln!("failed to write {path}: {e}"),
    }

    // Every piece of gear on its own.
    let items: Vec<Item> = ALL_ITEMS
        .iter()
        .copied()
        .filter(|i| i.class().is_some())
        .collect();
    let (cw, ch) = (64usize, 80usize);
    let cols = 12;
    let mut all = Frame::new(cols * cw, items.len().div_ceil(cols) * ch);
    all.clear(INK);
    let mut r = Renderer::new(cw, ch);
    let base = outfit(&[Item::FarmerTunic, Item::PatchedTrousers, Item::RainBoots]);
    for (k, item) in items.iter().enumerate() {
        let mut equip = base;
        let s = Stack::new(*item, 1);
        let held = match item.class().and_then(|c| c.slot()) {
            Some(slot) => {
                equip[slot as usize] = Some(s);
                None
            }
            None => Some(s),
        };
        let (x0, y0) = ((k % cols) * cw, (k / cols) * ch);
        portrait(
            &a,
            &mut r,
            &mut all,
            (x0, y0),
            &equip,
            held.as_ref(),
            0.55,
            SLATE,
        );
        let icon = a.tex(a.icon(item.def().icon));
        for y in 0..16 {
            for x in 0..16 {
                let c = icon.get(x, y);
                if c != CLEAR {
                    all.color[(y0 + ch - 17 + y as usize) * all.w + x0 + 1 + x as usize] = c;
                }
            }
        }
    }
    let all_path = match path.rsplit_once('.') {
        Some((stem, ext)) => format!("{stem}_all.{ext}"),
        None => format!("{path}_all"),
    };
    match save_png(Path::new(&all_path), &all, 2) {
        Ok(()) => println!("wrote {all_path}"),
        Err(e) => eprintln!("failed to write {all_path}: {e}"),
    }

    // Every hat from the game's own camera, from the front and both sides, then everyone
    // in town who wears one: nothing should poke through a crown.
    let hats: Vec<Item> = items
        .iter()
        .copied()
        .filter(|i| i.class().and_then(|c| c.slot()) == Some(Slot::Head))
        .collect();
    let wearers: Vec<crate::game::folk::Villager> = crate::game::folk::VILLAGERS
        .iter()
        .copied()
        .filter(|v| crate::game::folk::outfit(*v)[Slot::Head as usize].is_some())
        .collect();
    let (cw, ch) = (64usize, 64usize);
    let views = [0.0f32, 0.9, -0.9, std::f32::consts::PI];
    let cols = views.len() * 3;
    let n = hats.len() + wearers.len();
    let mut sheet = Frame::new(cols * cw, n.div_ceil(3) * ch);
    sheet.clear(INK);
    let mut r = Renderer::new(cw, ch);
    let base = outfit(&[Item::FarmerTunic, Item::PatchedTrousers, Item::RainBoots]);
    for k in 0..n {
        let (row, block) = (k / 3, k % 3);
        for (j, yaw) in views.iter().enumerate() {
            let x0 = (block * views.len() + j) * cw;
            let y0 = row * ch;
            let mut equip = base;
            let who = if k < hats.len() {
                equip[Slot::Head as usize] = Some(Stack::new(hats[k], 1));
                None
            } else {
                Some(wearers[k - hats.len()])
            };
            portrait_from(
                &a,
                &mut r,
                &mut sheet,
                (x0, y0),
                &equip,
                None,
                *yaw,
                SLATE,
                47.0,
                who,
            );
        }
    }
    let hats_path = match path.rsplit_once('.') {
        Some((stem, ext)) => format!("{stem}_hats.{ext}"),
        None => format!("{path}_hats"),
    };
    match save_png(Path::new(&hats_path), &sheet, 2) {
        Ok(()) => println!("wrote {hats_path}"),
        Err(e) => eprintln!("failed to write {hats_path}: {e}"),
    }
}

/// The light maps as an image: rows are light levels, columns palette colours, in three
/// blocks for cold, neutral and warm light.
pub fn palette_chart(path: &str) {
    let sh = crate::palette::Shading::new();
    let mut fb = crate::render::Frame::new(3 * (32 * 6 + 4), 32 * 4 + 8);
    fb.clear(crate::palette::INK);
    for (b, warm) in [0usize, 4, 8].into_iter().enumerate() {
        for lv in 0..32 {
            for i in 0..32u8 {
                let c = sh.shade(warm, lv, i);
                for yy in 0..4 {
                    for xx in 0..6 {
                        let x = b * (32 * 6 + 4) + i as usize * 6 + xx;
                        fb.color[(8 + lv * 4 + yy) * fb.w + x] = c;
                    }
                }
            }
        }
        // Mark the full-light row.
        for x in 0..32 * 6 {
            fb.color[(8 + 16 * 4 - 1) * fb.w + b * (32 * 6 + 4) + x] = crate::palette::WHITE;
        }
    }
    let _ = save_png(Path::new(path), &fb, 3);
}

pub fn bench() {
    let audio = Audio::silent();
    let input = Input::default();
    let mut r = Renderer::new(W, H);
    let mut game = Game::new();
    game.new_game(1);
    descend(&mut game, &input, &audio, 12, false);
    let frames = 240;
    let t0 = Instant::now();
    let mut draw_time = 0.0;
    for _ in 0..frames {
        tick(&mut game, &input, &audio, 1);
        let t = Instant::now();
        game.draw(&mut r, &input);
        draw_time += t.elapsed().as_secs_f64();
    }
    let total = t0.elapsed().as_secs_f64();
    println!(
        "{frames} frames at {W}x{H}: {:.2} ms/frame total, {:.2} ms/frame drawing",
        total * 1000.0 / frames as f64,
        draw_time * 1000.0 / frames as f64
    );
    // The busiest place: the plaza at noon, everyone out and about.
    {
        let p = play(&mut game);
        p.clock.min = 720.0;
        p.restored = u32::MAX;
        p.ride_bus(true);
        p.bus = None;
        p.player.pos = Vec2::new(35.5, 29.5);
    }
    let t0 = Instant::now();
    let mut draw_time = 0.0;
    for _ in 0..frames {
        tick(&mut game, &input, &audio, 1);
        let t = Instant::now();
        game.draw(&mut r, &input);
        draw_time += t.elapsed().as_secs_f64();
    }
    let total = t0.elapsed().as_secs_f64();
    println!(
        "{frames} frames in town: {:.2} ms/frame total, {:.2} ms/frame drawing",
        total * 1000.0 / frames as f64,
        draw_time * 1000.0 / frames as f64
    );
}

/// `--town-shots DIR`: the road to town, Bramblewick from above and street level, and the
/// inside of every shop. Handy while building the town.
pub fn town_shots(dir: &str) {
    use crate::game::town::{self, PLACES};
    use crate::game::world::Area;
    let dir = Path::new(dir);
    if let Err(e) = std::fs::create_dir_all(dir) {
        eprintln!("cannot create {}: {e}", dir.display());
        return;
    }
    // SAFETY: set before any other thread reads the environment.
    unsafe {
        std::env::set_var("HOLLOWBLOOM_DATA", dir.join("data"));
    }
    let audio = Audio::silent();
    let input = Input::default();
    let mut r = Renderer::new(W, H);
    let mut game = Game::new();
    game.new_game(20260928);
    tick(&mut game, &input, &audio, 30);
    {
        let p = play(&mut game);
        p.menu = Menu::None;
        p.banner = None;
        let (sx, sz) = crate::game::farm::MARKS.stop;
        p.player.pos = Vec2::new(sx as f32 + 1.0, sz as f32 + 1.6);
        p.player.facing = Vec2::new(1.0, 0.0);
    }
    snap(&mut game, &mut r, &input, dir, "t01_farm_bus_stop");
    // Wave the bus down and watch it pull in.
    {
        let p = play(&mut game);
        let mut io = Io {
            dt: 1.0 / 60.0,
            input: &input,
            audio: &audio,
            view: (W, H),
            quit: false,
            toggle_fullscreen: false,
        };
        p.call_bus(&mut io);
    }
    tick(&mut game, &input, &audio, 110);
    snap(&mut game, &mut r, &input, dir, "t02_bus_arrives");
    tick(&mut game, &input, &audio, 150);
    {
        let p = play(&mut game);
        p.banner = None;
    }
    snap(&mut game, &mut r, &input, dir, "t03_town_arrival");
    // Every restoration done, for the postcard views.
    for (name, done) in [("before", 0u32), ("after", u32::MAX)] {
        {
            let p = play(&mut game);
            p.restored = done;
            p.town = town::generate(done);
            p.bus = None;
            p.clock.min = 700.0;
        }
        // The whole town from high above.
        let mut big = Renderer::new(1440, 1000);
        if let State::Play(p) = &mut game.state {
            let a = &game.assets;
            big.cam.target = glam::Vec3::new(36.0, 0.0, 27.0);
            big.cam.dist = 118.0;
            big.cam.pitch = 62f32.to_radians();
            big.cam.update(1440, 1000);
            let env = p.env();
            crate::game::draw::draw_world(&mut big, a, &mut p.town, &env, &[]);
            big.ambient_occlusion(crate::game::draw::AO_STRENGTH);
            crate::game::draw::draw_grass(&mut big, a, &p.town, &env);
            big.fb.outline(crate::palette::INK);
        }
        let path = dir.join(format!("t04_town_overview_{name}.png"));
        let _ = save_png(&path, &big.fb, 1);
        println!("wrote {}", path.display());
    }
    let spots: [(&str, (f32, f32), f32); 6] = [
        ("t05_plaza", (35.5, 30.5), 700.0),
        ("t06_main_street_west", (12.0, 21.5), 760.0),
        ("t07_main_street_east", (44.0, 21.5), 820.0),
        ("t08_town_hall", (34.0, 12.5), 900.0),
        ("t09_tavern_evening", (54.0, 35.0), 1230.0),
        ("t10_wishing_tree", (65.0, 27.0), 640.0),
    ];
    for (name, (x, z), min) in spots {
        let p = play(&mut game);
        p.area = Area::Town;
        p.player.pos = Vec2::new(x, z);
        p.player.facing = Vec2::new(0.0, -1.0);
        p.clock.min = min;
        snap(&mut game, &mut r, &input, dir, name);
    }
    for (i, place) in PLACES.iter().enumerate() {
        {
            let p = play(&mut game);
            p.clock.min = 720.0;
            p.enter_place(*place);
            let (ex, ez) = p.room.as_ref().unwrap().exit;
            p.player.pos = Vec2::new(ex as f32 + 0.5, ez as f32 - 2.5);
        }
        let p = play(&mut game);
        p.cam_pos = p.room_center();
        p.cam.target = p.cam_pos;
        p.cam.update(W, H);
        game.draw(&mut r, &input);
        let path = dir.join(format!("t{:02}_inside_{:?}.png", 11 + i, place).to_lowercase());
        let _ = save_png(&path, &r.fb, 2);
        println!("wrote {}", path.display());
        play(&mut game).leave_place();
    }
}

/// A frame's worth of input and audio, for poking the game directly.
fn mk_io<'a>(input: &'a Input, audio: &'a Audio) -> Io<'a> {
    Io {
        dt: 1.0 / 60.0,
        input,
        audio,
        view: (W, H),
        quit: false,
        toggle_fullscreen: false,
    }
}

/// `--folk-shots DIR`: villagers about town, a chat, a quest offer and hand-in, the boards,
/// the journal and a shop with its keeper.
pub fn folk_shots(dir: &str) {
    use crate::game::folk::Villager;
    use crate::game::quests::QuestId;
    use crate::game::talk::Say;
    use crate::game::town::{self, Place};
    let dir = Path::new(dir);
    if let Err(e) = std::fs::create_dir_all(dir) {
        eprintln!("cannot create {}: {e}", dir.display());
        return;
    }
    // SAFETY: set before any other thread reads the environment.
    unsafe {
        std::env::set_var("HOLLOWBLOOM_DATA", dir.join("data"));
    }
    let audio = Audio::silent();
    let input = Input::default();
    let mut r = Renderer::new(W, H);
    let mut game = Game::new();
    game.new_game(20260928);
    tick(&mut game, &input, &audio, 5);
    let io = mk_io;
    {
        let p = play(&mut game);
        p.menu = Menu::None;
        p.banner = None;
        p.clock.min = 600.0;
        p.deepest = 12;
        p.restored = town::FOUNTAIN | town::GARDENS;
        p.ride_bus(true);
        p.bus = None;
        p.banner = None;
        p.player.pos = Vec2::new(35.5, 29.6);
        p.player.facing = Vec2::new(0.0, -1.0);
        p.arrive_folk();
    }
    tick(&mut game, &input, &audio, 240);
    snap(&mut game, &mut r, &input, dir, "f01_townsfolk");
    // Find Pip and have a chat.
    {
        let p = play(&mut game);
        if let Some(i) = p.folk.iter().position(|n| n.who == Villager::Pip) {
            let at = p.folk[i].pos;
            p.player.pos = at + Vec2::new(0.0, 1.0);
            p.player.facing = Vec2::new(0.0, -1.0);
            let mut io = io(&input, &audio);
            p.talk_to(i, &mut io);
            if let Menu::Talk { shown, text, .. } = &mut p.menu {
                *shown = text.chars().count() as f32;
            }
        }
    }
    snap(&mut game, &mut r, &input, dir, "f02_meet_pip");
    {
        let p = play(&mut game);
        let mut io = io(&input, &audio);
        if let Menu::Talk { who, choices, .. } = &p.menu {
            let who = *who;
            if let Some((_, say)) = choices.iter().find(|(_, s)| matches!(s, Say::Offer(_))) {
                let say = *say;
                if let Some((text, choices)) = p.talk_choice(who, say, &mut io) {
                    let n = text.chars().count() as f32;
                    p.menu = Menu::Talk {
                        who,
                        text,
                        shown: n,
                        choices,
                        sel: 0,
                        blip: 0.0,
                    };
                }
            }
        }
    }
    snap(&mut game, &mut r, &input, dir, "f03_quest_offer");
    // Take a handful of quests, and finish one.
    {
        let p = play(&mut game);
        let mut io = io(&input, &audio);
        p.menu = Menu::None;
        for n in &mut p.folk {
            n.talking = false;
        }
        for k in [
            "pip_teddy",
            "thistle_hello",
            "rowan_slimes",
            "posy_turnips",
            "fern_locket",
        ] {
            p.accept(QuestId::Story(k.to_string()), &mut io);
        }
        p.player.inv.add(Item::Turnip, 6);
        if let Some(q) = p.quests.iter_mut().find(|q| q.key() == "rowan_slimes") {
            q.n = 7;
        }
        p.toasts.clear();
    }
    snap(&mut game, &mut r, &input, dir, "f04_quest_tracker");
    {
        let p = play(&mut game);
        p.menu = Menu::Journal { tab: 0, sel: 2 };
    }
    snap(&mut game, &mut r, &input, dir, "f05_journal");
    {
        let p = play(&mut game);
        p.friends.met.fill(true);
        for (i, v) in crate::game::folk::VILLAGERS.iter().enumerate() {
            p.friends.add(*v, (i as i32 * 57) % 900);
        }
        p.menu = Menu::Journal { tab: 1, sel: 0 };
    }
    snap(&mut game, &mut r, &input, dir, "f06_friends");
    {
        let p = play(&mut game);
        p.menu = Menu::Board {
            guild: false,
            sel: 0,
        };
    }
    snap(&mut game, &mut r, &input, dir, "f07_request_board");
    {
        let p = play(&mut game);
        let mut io = io(&input, &audio);
        p.player.inv.add(Item::Teddy, 1);
        if let Some(i) = p.quests.iter().position(|q| q.key() == "pip_teddy") {
            p.finish_quest(i, &mut io);
        }
        p.menu = Menu::Cheer;
    }
    tick(&mut game, &input, &audio, 50);
    snap(&mut game, &mut r, &input, dir, "f08_quest_complete");
    // A shop with its keeper behind the counter.
    for (name, place) in [
        ("f09_bakery", Place::Bakery),
        ("f10_armory", Place::Armory),
        ("f11_smithy", Place::Smithy),
        ("f12_tavern_evening", Place::Tavern),
    ] {
        {
            let p = play(&mut game);
            p.menu = Menu::None;
            p.cheer = None;
            p.clock.min = if place == Place::Tavern {
                1230.0
            } else {
                700.0
            };
            p.enter_place(place);
            let (ex, ez) = p.room.as_ref().unwrap().exit;
            p.player.pos = Vec2::new(ex as f32 + 0.5, ez as f32 - 4.5);
            p.player.facing = Vec2::new(0.0, -1.0);
        }
        tick(&mut game, &input, &audio, 30);
        snap(&mut game, &mut r, &input, dir, name);
        play(&mut game).leave_place();
    }
    {
        let p = play(&mut game);
        p.clock.min = 700.0;
        p.enter_place(Place::Armory);
        p.menu = Menu::shop_at(Place::Armory);
        p.menu = match std::mem::replace(&mut p.menu, Menu::None) {
            Menu::Shop {
                at, cursor, scroll, ..
            } => Menu::Shop {
                at,
                tab: crate::game::menus::ShopTab::Specials,
                cursor,
                scroll,
            },
            m => m,
        };
    }
    snap(&mut game, &mut r, &input, dir, "f13_armory_shop");
    // A keepsake waiting in the Hollow for whoever asked for it.
    {
        let p = play(&mut game);
        p.menu = Menu::None;
        p.leave_place();
        let mut io = io(&input, &audio);
        p.accept(QuestId::Story("fern_locket".into()), &mut io);
        p.fade = None;
        p.start_fade(Trans::Descend {
            depth: 3,
            via_waystone: false,
        });
    }
    tick(&mut game, &input, &audio, 60);
    {
        let p = play(&mut game);
        p.banner = None;
        p.toasts.clear();
        if let Some(d) = p.drops.iter().find(|d| d.stack.item == Item::Locket) {
            let (x, z) = (d.pos.x, d.pos.z);
            let w = &p.level.as_ref().unwrap().world;
            let (tx, tz) = w.nearest_open(x as i32 - 1, z as i32 + 2);
            p.player.pos = Vec2::new(tx as f32 + 0.5, tz as f32 + 0.5);
            p.player.facing = Vec2::new(1.0, -1.0).normalize();
            p.foes.clear();
            for d in p.drops.iter_mut() {
                d.age = 5.0;
            }
        }
    }
    tick(&mut game, &input, &audio, 20);
    snap(&mut game, &mut r, &input, dir, "f14_keepsake_in_the_hollow");
}

/// Runs frames with the input moving on between them, so presses and releases register.
fn step(game: &mut Game, input: &mut Input, audio: &Audio, frames: usize) {
    for _ in 0..frames {
        tick(game, input, audio, 1);
        input.end_frame(1.0 / 60.0);
    }
}

/// A picture of the room you're in, framed the way the game frames it.
fn snap_room(game: &mut Game, r: &mut Renderer, input: &Input, dir: &Path, name: &str) {
    if let Some(p) = game.play_mut() {
        let c = p.room_center();
        p.cam_pos = c + (p.player.world_pos() - c) * glam::Vec3::new(0.15, 0.0, 0.08);
        p.cam.target = p.cam_pos;
        p.cam.dist = 17.5;
        p.cam.update(W, H);
    }
    game.draw(r, input);
    let path = dir.join(format!("{name}.png"));
    match save_png(&path, &r.fb, 2) {
        Ok(()) => println!("wrote {}", path.display()),
        Err(e) => eprintln!("failed to write {}: {e}", path.display()),
    }
}

/// Fishing, the farmhouse, cooking and the furniture shop.
pub fn home_shots(dir: &str) {
    use crate::game::dungeon::Foe;
    use crate::game::fish::{Hooked, Phase};
    use crate::game::foes::Enemy;
    use crate::game::home::{self, Furn, Rug};
    use crate::game::town::Place;
    use crate::input::KeyCode;
    let dir = Path::new(dir);
    if let Err(e) = std::fs::create_dir_all(dir) {
        eprintln!("cannot create {}: {e}", dir.display());
        return;
    }
    // SAFETY: set before any other thread reads the environment.
    unsafe {
        std::env::set_var("HOLLOWBLOOM_DATA", dir.join("data"));
    }
    let audio = Audio::silent();
    let mut input = Input::default();
    let mut r = Renderer::new(W, H);
    let mut game = Game::new();
    game.new_game(20261001);
    tick(&mut game, &input, &audio, 30);

    // An afternoon by the farm pond with a coral rod from the Hollow.
    {
        let p = play(&mut game);
        p.menu = Menu::None;
        p.banner = None;
        p.clock.min = 960.0;
        p.rain = false;
        for z in 29..34 {
            for x in 19..23 {
                if p.farm.floor(x, z) != Floor::Water {
                    p.farm.set_obj(x, z, None);
                }
            }
        }
        p.player.pos = Vec2::new(20.5, 31.5);
        p.player.facing = Vec2::new(-1.0, 0.0);
        p.player.inv.slots[0] = Some(fancy(
            Item::CoralRod,
            10,
            &[(Stat::Angler, 9), (Stat::Lure, 11)],
            &[(Stat::Luck, 3)],
        ));
        p.player.sel = 0;
        p.toasts.clear();
    }
    input.key_event(KeyCode::KeyJ, true, false);
    step(&mut game, &mut input, &audio, 22);
    snap(&mut game, &mut r, &input, dir, "h01_winding_up_a_cast");
    input.key_event(KeyCode::KeyJ, false, false);
    step(&mut game, &mut input, &audio, 50);
    {
        let p = play(&mut game);
        if let Some(f) = p.fishing.as_mut() {
            f.bite_in = 30.0;
        }
    }
    step(&mut game, &mut input, &audio, 40);
    snap(&mut game, &mut r, &input, dir, "h02_line_in_the_pond");
    {
        let p = play(&mut game);
        if let Some(f) = p.fishing.as_mut() {
            f.bite_in = 0.0;
        }
    }
    step(&mut game, &mut input, &audio, 3);
    snap(&mut game, &mut r, &input, dir, "h03_a_bite");
    {
        let p = play(&mut game);
        if let Some(f) = p.fishing.as_mut() {
            f.hooked = Some(Hooked::Fish(Item::LilyKoi, 44));
        }
    }
    input.key_event(KeyCode::KeyJ, true, false);
    step(&mut game, &mut input, &audio, 1);
    input.key_event(KeyCode::KeyJ, false, false);
    step(&mut game, &mut input, &audio, 1);
    // Reel for a while, keeping the koi mostly in the bar.
    for k in 0..70 {
        let hold = (k / 9) % 2 == 0;
        input.key_event(KeyCode::KeyJ, hold, false);
        {
            let p = play(&mut game);
            if let Some(f) = p.fishing.as_mut() {
                if f.phase == Phase::Reel {
                    f.fish_to = (f.zone + f.zone_h * 0.6).min(0.95);
                    f.progress = f.progress.min(0.7);
                }
            }
        }
        step(&mut game, &mut input, &audio, 1);
    }
    snap(&mut game, &mut r, &input, dir, "h04_reeling_in");
    input.key_event(KeyCode::KeyJ, true, false);
    for _ in 0..400 {
        let p = play(&mut game);
        match p.fishing.as_mut() {
            Some(f) if f.phase == Phase::Reel => {
                f.fish_y = (f.zone + f.zone_h * 0.5).min(1.0);
                f.fish_to = f.fish_y;
            }
            _ => break,
        }
        step(&mut game, &mut input, &audio, 1);
    }
    input.key_event(KeyCode::KeyJ, false, false);
    step(&mut game, &mut input, &audio, 14);
    snap(&mut game, &mut r, &input, dir, "h05_caught_a_lily_koi");
    step(&mut game, &mut input, &audio, 120);

    // Water folk round a pond deep in the Hollow.
    let mut found = false;
    for depth in [
        12u32, 13, 14, 15, 16, 17, 18, 19, 21, 22, 23, 24, 3, 4, 5, 6,
    ] {
        descend(&mut game, &input, &audio, depth, false);
        let p = play(&mut game);
        let Some(l) = p.level.as_ref() else { continue };
        let w = &l.world;
        let mut spot = None;
        'find: for z in 2..w.h - 2 {
            for x in 2..w.w - 2 {
                if w.floor(x, z) == Floor::Water
                    && w.floor(x, z + 1) != Floor::Water
                    && !w.blocked(x, z + 2)
                    && w.floor(x, z + 2) != Floor::Water
                {
                    spot = Some((x, z));
                    break 'find;
                }
            }
        }
        let Some((wx, wz)) = spot else { continue };
        p.foes.clear();
        p.drops.clear();
        p.player.pos = Vec2::new(wx as f32 + 0.5, wz as f32 + 2.5);
        p.player.facing = Vec2::new(0.0, -1.0);
        let biome = crate::game::dungeon::biome_for(depth);
        let (px, pz) = p.player.tile();
        let w = &p.level.as_ref().unwrap().world;
        let mut open = Vec::new();
        for dz in -3..=2 {
            for dx in -4..=4 {
                let (x, z) = (px + dx, pz + dz);
                if (dx.abs() >= 2 || dz.abs() >= 2)
                    && !w.blocked(x, z)
                    && w.floor(x, z) != Floor::Water
                {
                    open.push((x, z));
                }
            }
        }
        for (i, foe) in [Foe::Frog, Foe::Jelly, Foe::Puffer, Foe::Frog]
            .into_iter()
            .enumerate()
        {
            let Some(&(x, z)) = open.get(i * 3 % open.len().max(1)) else {
                break;
            };
            let mut f = Enemy::new(
                foe,
                x as f32 + 0.5,
                z as f32 + 0.5,
                depth,
                biome,
                false,
                i as u32 * 5,
            );
            f.yaw = (p.player.pos.x - f.pos.x).atan2(p.player.pos.y - f.pos.y);
            f.anim = i as f32 * 0.7;
            p.foes.push(f);
        }
        found = true;
        break;
    }
    if found {
        tick(&mut game, &input, &audio, 2);
        {
            let p = play(&mut game);
            for f in p.foes.iter_mut() {
                f.alert = false;
            }
        }
        snap(
            &mut game,
            &mut r,
            &input,
            dir,
            "h06_water_folk_by_a_hollow_pond",
        );
    }

    // Home, as you find it on the first morning.
    {
        let p = play(&mut game);
        p.foes.clear();
        p.fade = None;
        p.level = None;
        p.area = crate::game::world::Area::Farm;
        p.clock.min = 480.0;
        p.player.inv.slots[0] = None;
        p.enter_house();
        p.banner = None;
        p.toasts.clear();
        p.player.pos = Vec2::new(7.5, 8.2);
        p.player.facing = Vec2::new(0.0, -1.0);
    }
    tick(&mut game, &input, &audio, 5);
    snap_room(
        &mut game,
        &mut r,
        &input,
        dir,
        "h07_home_on_the_first_morning",
    );

    // Months later: a charming home.
    {
        let p = play(&mut game);
        p.house = home::House::new();
        let w = &mut p.house.world;
        for z in 1..10 {
            for x in 1..14 {
                w.set_obj(x, z, None);
            }
        }
        let pieces: &[(Furn, u8, i32, i32)] = &[
            (Furn::Wardrobe, 0, 1, 1),
            (Furn::CanopyBed, 0, 2, 1),
            (Furn::FloorLamp, 0, 4, 1),
            (Furn::Bookshelf, 0, 5, 1),
            (Furn::Clock, 0, 6, 1),
            (Furn::Fireplace, 0, 8, 1),
            (Furn::Range, 0, 10, 1),
            (Furn::Counter, 0, 12, 1),
            (Furn::Icebox, 0, 13, 1),
            (Furn::RoundTable, 0, 7, 5),
            (Furn::Chair, 1, 6, 5),
            (Furn::Chair, 3, 8, 5),
            (Furn::Sofa, 0, 2, 4),
            (Furn::Armchair, 1, 1, 6),
            (Furn::Globe, 0, 4, 4),
            (Furn::Piano, 0, 2, 8),
            (Furn::FishTank, 0, 10, 4),
            (Furn::FishBowl, 0, 12, 4),
            (Furn::Telescope, 0, 13, 6),
            (Furn::Fern, 0, 13, 8),
            (Furn::Cactus, 0, 1, 8),
            (Furn::Vase, 0, 9, 8),
            (Furn::Plush, 0, 11, 8),
            (Furn::Candelabra, 0, 4, 8),
        ];
        for &(f, rot, x, z) in pieces {
            home::put(w, f, rot, x, z);
        }
        let tank = [
            Item::LilyKoi,
            Item::GoldenCarp,
            Item::Bluegill,
            Item::PrismGuppy,
            Item::MoonJelly,
            Item::RainbowTrout,
        ];
        if let Some(crate::game::world::Obj::Furniture { fish, .. }) = w.obj_mut(10, 4) {
            fish.extend(tank);
        }
        if let Some(crate::game::world::Obj::Furniture { fish, .. }) = w.obj_mut(12, 4) {
            fish.extend([Item::SunnyMinnow, Item::BubbleGoby]);
        }
        p.house.rugs = vec![
            Rug {
                x: 6,
                z: 4,
                kind: 0,
            },
            Rug {
                x: 2,
                z: 5,
                kind: 2,
            },
            Rug {
                x: 10,
                z: 6,
                kind: 3,
            },
        ];
        p.house.art = vec![(7, 3), (12, 2), (1, 0)];
        p.house.redecorate(5, 0);
        for it in tank {
            p.journal.fish.push(it);
            p.journal.records.push((it, 40));
        }
        for it in [
            Item::SunnyMinnow,
            Item::BubbleGoby,
            Item::PondPerch,
            Item::BrookTrout,
        ] {
            p.journal.fish.push(it);
        }
        p.stats.caught = 57;
        p.stats.cooked = 23;
        p.clock.min = 700.0;
        p.player.pos = Vec2::new(7.5, 7.6);
        p.player.facing = Vec2::new(0.0, -1.0);
        p.toasts.clear();
    }
    tick(&mut game, &input, &audio, 3);
    snap_room(&mut game, &mut r, &input, dir, "h08_a_charming_home");
    {
        let p = play(&mut game);
        p.clock.min = 1310.0;
    }
    tick(&mut game, &input, &audio, 3);
    snap_room(&mut game, &mut r, &input, dir, "h09_cozy_at_night");

    // Carrying an armchair to its new spot.
    {
        let p = play(&mut game);
        p.clock.min = 760.0;
        p.player.inv.slots[1] = Some(Stack::new(Item::Armchair, 1));
        p.player.sel = 1;
        p.player.pos = Vec2::new(6.5, 8.5);
        p.player.facing = Vec2::new(-1.0, 0.0);
        p.sel_name_t = 5.0;
    }
    tick(&mut game, &input, &audio, 3);
    snap_room(&mut game, &mut r, &input, dir, "h10_placing_an_armchair");

    // Looking after the fish.
    {
        let p = play(&mut game);
        p.player.inv.slots[1] = Some(Stack::new(Item::CrystalTetra, 2));
        p.player.inv.slots[2] = Some(Stack::new(Item::SilverDace, 1));
        p.player.pos = Vec2::new(10.5, 6.5);
        p.player.facing = Vec2::new(0.0, -1.0);
        p.menu = Menu::Tank {
            x: 10,
            z: 4,
            cursor: 1,
        };
    }
    snap_room(&mut game, &mut r, &input, dir, "h11_the_fish_tank");

    // Grilling a trout on the copper range.
    {
        let p = play(&mut game);
        p.menu = Menu::None;
        p.player.inv.add(Item::BrookTrout, 2);
        p.player.inv.add(Item::Garlic, 2);
        p.player.pos = Vec2::new(10.5, 2.4);
        p.player.facing = Vec2::new(0.0, -1.0);
        let k = crate::game::items::RECIPES
            .iter()
            .position(|r| r.out == Item::GrilledTrout)
            .unwrap_or(0);
        let mut io = mk_io(&input, &audio);
        p.start_cooking(k, (0, 0, 0), &mut io);
    }
    tick(&mut game, &input, &audio, 45);
    snap_room(&mut game, &mut r, &input, dir, "h12_cooking_at_the_range");
    tick(&mut game, &input, &audio, 60);

    // The Fishdex.
    {
        let p = play(&mut game);
        p.menu = Menu::Journal { tab: 3, sel: 4 };
    }
    snap_room(&mut game, &mut r, &input, dir, "h13_fishdex");

    // Wren's shop, stocked for a charming home.
    {
        let p = play(&mut game);
        p.menu = Menu::None;
        p.leave_house();
        p.clock.min = 720.0;
        p.money = 12000;
        p.enter_place(Place::Nook);
        let (ex, ez) = p.room.as_ref().unwrap().exit;
        p.player.pos = Vec2::new(ex as f32 + 0.5, ez as f32 - 3.5);
        p.player.facing = Vec2::new(0.0, -1.0);
        p.banner = None;
    }
    tick(&mut game, &input, &audio, 30);
    snap_room(&mut game, &mut r, &input, dir, "h14_wrens_cozy_nook");
    {
        let p = play(&mut game);
        p.menu = Menu::shop_at(Place::Nook);
    }
    snap_room(&mut game, &mut r, &input, dir, "h15_nook_decor");
    {
        let p = play(&mut game);
        p.menu = Menu::None;
        p.leave_place();
        p.enter_place(Place::Smithy);
        // Scrolled down to the fishing rods.
        let rows = crate::game::shops::rows(p, Some(Place::Smithy), ShopTab::Goods);
        let at = rows
            .iter()
            .position(|(s, _, _)| s.item == Item::BambooRod)
            .unwrap_or(0);
        p.menu = Menu::Shop {
            at: Some(Place::Smithy),
            tab: ShopTab::Goods,
            cursor: at,
            scroll: at.saturating_sub(1),
        };
    }
    snap_room(&mut game, &mut r, &input, dir, "h16_rods_at_the_smithy");
}

/// Light and shade: the sun's shadows through the day and a full moon, and ambient
/// occlusion on the farm, in town, in the Hollow and indoors.
pub fn light_shots(dir: &str) {
    use crate::game::town::Place;
    let dir = Path::new(dir);
    if let Err(e) = std::fs::create_dir_all(dir) {
        eprintln!("cannot create {}: {e}", dir.display());
        return;
    }
    // SAFETY: set before any other thread reads the environment.
    unsafe {
        std::env::set_var("HOLLOWBLOOM_DATA", dir.join("data"));
    }
    let audio = Audio::silent();
    let input = Input::default();
    let mut r = Renderer::new(W, H);
    let mut game = Game::new();
    game.new_game(20261003);
    tick(&mut game, &input, &audio, 30);
    {
        let p = play(&mut game);
        p.menu = Menu::None;
        p.banner = None;
        p.rain = false;
        // A few things lying about by the house.
        let (dx, dz) = crate::game::farm::MARKS.door;
        let mut rng = Rng::new(3);
        for (k, it) in [Item::Turnip, Item::Wood, Item::CopperOre, Item::Stone]
            .into_iter()
            .enumerate()
        {
            let at = glam::Vec3::new(dx as f32 + 1.2 + k as f32 * 0.6, 0.0, dz as f32 + 2.3);
            let mut d = crate::game::fx::Drop::new(Stack::new(it, 1), at, &mut rng);
            d.pos = at + glam::Vec3::Y * 0.12;
            d.vel = glam::Vec3::ZERO;
            d.age = 3.0;
            p.drops.push(d);
        }
        p.player.pos = Vec2::new(dx as f32 + 0.5, dz as f32 + 2.2);
        p.player.facing = Vec2::new(0.0, 1.0);
    }
    for (name, min, day) in [
        ("l01_farm_morning", 430.0, 1),
        ("l02_farm_noon", 760.0, 1),
        ("l03_farm_evening", 1110.0, 1),
        ("l04_farm_full_moon", 1330.0, 5),
        ("l05_farm_new_moon", 1330.0, 1),
    ] {
        {
            let p = play(&mut game);
            p.clock.min = min;
            p.clock.day = day;
            p.toasts.clear();
        }
        tick(&mut game, &input, &audio, 2);
        snap(&mut game, &mut r, &input, dir, name);
    }
    // Bramblewick's plaza mid-morning: the buildings throw long shadows.
    {
        let p = play(&mut game);
        p.clock.min = 560.0;
        p.clock.day = 2;
        p.area = crate::game::world::Area::Town;
        p.drops.clear();
        p.player.pos = Vec2::new(35.5, 30.5);
        p.player.facing = Vec2::new(0.0, 1.0);
        p.arrive_folk();
    }
    tick(&mut game, &input, &audio, 2);
    snap(&mut game, &mut r, &input, dir, "l06_town_morning");
    {
        let p = play(&mut game);
        p.clock.min = 900.0;
        p.player.pos = Vec2::new(12.0, 21.5);
    }
    tick(&mut game, &input, &audio, 2);
    snap(&mut game, &mut r, &input, dir, "l07_town_afternoon");
    // Underground and indoors: ambient occlusion alone.
    for (name, depth) in [("l08_hollow_mossy", 3u32), ("l09_hollow_frost", 44)] {
        descend(&mut game, &input, &audio, depth, false);
        tick(&mut game, &input, &audio, 2);
        snap(&mut game, &mut r, &input, dir, name);
    }
    {
        let p = play(&mut game);
        p.foes.clear();
        p.level = None;
        p.area = crate::game::world::Area::Farm;
        p.clock.min = 700.0;
        p.enter_house();
        p.banner = None;
        p.player.pos = Vec2::new(7.5, 7.5);
    }
    tick(&mut game, &input, &audio, 2);
    snap(&mut game, &mut r, &input, dir, "l10_home");
    {
        let p = play(&mut game);
        p.leave_house();
        p.area = crate::game::world::Area::Town;
        p.enter_place(Place::Nook);
        p.banner = None;
        let (ex, ez) = p.room.as_ref().unwrap().exit;
        p.player.pos = Vec2::new(ex as f32 + 0.5, ez as f32 - 3.5);
    }
    tick(&mut game, &input, &audio, 2);
    snap(&mut game, &mut r, &input, dir, "l11_nook");
}
