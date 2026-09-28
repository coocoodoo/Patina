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
        p.cam_pos = p.player.world_pos();
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
    use crate::game::draw::{Pose, draw_humanoid};
    use crate::game::scene::dress;
    use crate::render::{DrawOpts, Light, Mode};
    use glam::Vec3;

    let (cw, ch) = (r.width(), r.height());
    r.cam.target = Vec3::new(0.0, 0.42, 0.0);
    r.cam.pitch = 30f32.to_radians();
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
    let _ = Slot::Head;
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
}
