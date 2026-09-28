//! Offscreen modes: `--shots DIR` renders a tour of the game to PNG files and `--bench`
//! measures the renderer. Neither needs a display, so both run on CI machines.

use std::path::Path;
use std::time::Instant;

use glam::Vec2;

use crate::audio::Audio;
use crate::game::items::{Crop, Item, Stack};
use crate::game::menus::Menu;
use crate::game::play::{Play, Trans};
use crate::game::player::{Act, ActKind};
use crate::game::world::{Floor, Obj, WATERED};
use crate::game::{Game, Io, State};
use crate::input::Input;
use crate::render::Renderer;
use crate::shot::save_png;

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

    // A fresh farm, set up to show the crops at every stage.
    game.new_game(20260928);
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
        p.player.act = Some(Act {
            kind: ActKind::Hoe,
            t: 0.14,
            fired: true,
            tile: (26, 14),
            dir: Vec2::new(0.0, 1.0),
        });
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

    // Menus.
    {
        let p = play(&mut game);
        p.clock.min = 11.0 * 60.0;
        for (i, item) in [
            Item::Wood,
            Item::Stone,
            Item::CopperOre,
            Item::SlimeGel,
            Item::Glowcap,
            Item::CaveCarrot,
            Item::Torch,
            Item::Crystal,
        ]
        .into_iter()
        .enumerate()
        {
            p.player.inv.slots[12 + i] = Some(Stack::new(item, 3 + i as u16 * 4));
        }
        p.player.inv.slots[25] = Some(Stack::new(Item::HealingTonic, 2));
        p.menu = Menu::inventory(false);
    }
    tick(&mut game, &input, &audio, 5);
    snap(&mut game, &mut r, &input, dir, "05_inventory");
    {
        let p = play(&mut game);
        p.menu = Menu::Inventory {
            craft: true,
            cursor: 0,
            recipe: 1,
            scroll: 0,
        };
    }
    tick(&mut game, &input, &audio, 5);
    snap(&mut game, &mut r, &input, dir, "06_crafting");
    {
        let p = play(&mut game);
        p.deepest = 12;
        p.menu = Menu::Shop {
            cursor: 2,
            sell: false,
            scroll: 0,
        };
    }
    tick(&mut game, &input, &audio, 5);
    snap(&mut game, &mut r, &input, dir, "07_shop");
    {
        let p = play(&mut game);
        p.menu = Menu::None;
        p.player.inv.slots[0] = Some(Stack::new(Item::Sword2, 1));
        p.player.sel = 0;
        p.player.max_hp = 120;
        p.player.hp = 120;
        p.player.level = 8;
    }

    // The Hollow, biome by biome.
    for (i, depth) in [1u32, 14, 25, 33, 44, 56].into_iter().enumerate() {
        descend(&mut game, &input, &audio, depth, false);
        {
            let p = play(&mut game);
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
            p.player.act = Some(Act {
                kind: ActKind::Sword(2),
                t: 0.09,
                fired: true,
                tile: p.player.tile(),
                dir: Vec2::new(0.6, 0.8).normalize(),
            });
            p.player.facing = Vec2::new(0.6, 0.8).normalize();
            p.player.hurt = 5.0;
        }
        tick(&mut game, &input, &audio, 2);
        snap(
            &mut game,
            &mut r,
            &input,
            dir,
            &format!("{:02}_hollow_floor_{depth}", 8 + i),
        );
    }

    // Hidden behind a wall: the hero shows through as a soft silhouette.
    descend(&mut game, &input, &audio, 3, false);
    {
        let p = play(&mut game);
        p.foes.clear();
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
    snap(&mut game, &mut r, &input, dir, "13b_behind_wall");

    // A guardian floor.
    descend(&mut game, &input, &audio, 10, false);
    {
        let p = play(&mut game);
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
    snap(&mut game, &mut r, &input, dir, "14_guardian");

    // The same floor once the guardian is gone: the waystone haven.
    {
        let p = play(&mut game);
        p.foes.retain(|f| !f.boss);
        p.foes.clear();
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
    snap(&mut game, &mut r, &input, dir, "15_waystone");

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
        p.fade = None;
        p.start_fade(Trans::Sleep { passed_out: false });
    }
    tick(&mut game, &input, &audio, 60);
    snap(&mut game, &mut r, &input, dir, "16_new_day");
    if let State::Play(p) = &game.state {
        println!("day {} gold {}", p.clock.day, p.gold);
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
}
