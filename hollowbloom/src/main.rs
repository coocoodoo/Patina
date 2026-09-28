//! Hollowbloom: a cozy top-down action RPG. Farm on the surface, delve an endless dungeon
//! below it. Rendered in low-poly 3D by a software renderer that can only produce the 32
//! colours of the Resurrect 32 palette.

#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

mod app;
mod assets;
mod audio;
mod game;
mod headless;
mod input;
mod pad;
mod palette;
mod render;
mod shot;
mod ui;
mod util;

const HELP: &str = "\
Hollowbloom - a cozy farm above an endless Hollow

USAGE:
    hollowbloom [OPTIONS]

OPTIONS:
    --mute          start without sound
    --shots DIR     render a tour of the game to PNG files in DIR (no window needed)
    --wardrobe FILE render the hero in every piece of gear to one PNG
    --magic-shots DIR  render jelly slimes, sparks, grass, potions and spells
    --home-shots DIR   render fishing, the farmhouse, cooking and the furniture shop
    --light-shots DIR  render sun and moon shadows and ambient occlusion
    --monster-shots DIR  render every monster family in every biome's look
    --feature-shots DIR  render recipe cards, bombs, secret rooms and store hours
    --bench         measure rendering speed
    -h, --help      show this help

CONTROLS:
    WASD / arrows        move            Space / Shift   dodge roll
    J / Z / left click   use tool        E / right click interact, eat, drink, place
    hold J, let go       cast a rod      J on a bite, then hold J to reel the fish in
    T (at home)          turn furniture  J (at home)     pick furniture back up
    1-0 / wheel / [ ]    hotbar          Q / R           cast your two spells
    Tab / I              bag             C               crafting (and brewing)
    L                    quest journal   E near someone  talk, give gifts, take quests
    M                    minimap         Esc             pause    F11 fullscreen   F12 screenshot

STEAM DECK / CONTROLLER:
    left stick, D-pad    walk            right stick     aim
    A                    interact, OK    X               use tool (hold), fish
    B                    roll, back      Y               bag
    L1 / R1              hotbar, tabs    L2 / R2         cast your two spells
    View                 crafting        Menu            pause (Controls shows the layout)
    L3 / R3              map / quests    B (at home)     turn furniture
";

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.iter().any(|a| a == "-h" || a == "--help") {
        print!("{HELP}");
        return;
    }
    if let Some(i) = args.iter().position(|a| a == "--shots") {
        let dir = args.get(i + 1).map(String::as_str).unwrap_or("shots");
        headless::shots(dir);
        return;
    }
    if let Some(i) = args.iter().position(|a| a == "--palette-chart") {
        headless::palette_chart(args.get(i + 1).map(String::as_str).unwrap_or("palette.png"));
        return;
    }
    if let Some(i) = args.iter().position(|a| a == "--wardrobe") {
        headless::wardrobe(
            args.get(i + 1)
                .map(String::as_str)
                .unwrap_or("wardrobe.png"),
        );
        return;
    }
    if let Some(i) = args.iter().position(|a| a == "--folk-shots") {
        headless::folk_shots(args.get(i + 1).map(String::as_str).unwrap_or("folk"));
        return;
    }
    if let Some(i) = args.iter().position(|a| a == "--magic-shots") {
        headless::magic_shots(args.get(i + 1).map(String::as_str).unwrap_or("magic"));
        return;
    }
    if let Some(i) = args.iter().position(|a| a == "--feature-shots") {
        headless::feature_shots(args.get(i + 1).map(String::as_str).unwrap_or("features"));
        return;
    }
    if let Some(i) = args.iter().position(|a| a == "--monster-shots") {
        headless::monster_shots(args.get(i + 1).map(String::as_str).unwrap_or("monsters"));
        return;
    }
    if let Some(i) = args.iter().position(|a| a == "--light-shots") {
        headless::light_shots(args.get(i + 1).map(String::as_str).unwrap_or("light"));
        return;
    }
    if let Some(i) = args.iter().position(|a| a == "--home-shots") {
        headless::home_shots(args.get(i + 1).map(String::as_str).unwrap_or("home"));
        return;
    }
    if let Some(i) = args.iter().position(|a| a == "--town-shots") {
        headless::town_shots(args.get(i + 1).map(String::as_str).unwrap_or("town"));
        return;
    }
    if args.iter().any(|a| a == "--bench") {
        headless::bench();
        return;
    }
    let game = game::Game::new();
    let audio = if args.iter().any(|a| a == "--mute") {
        audio::Audio::silent()
    } else {
        audio::Audio::start()
    };
    if let Err(e) = app::run(game, audio) {
        eprintln!("hollowbloom: {e}");
        std::process::exit(1);
    }
}
