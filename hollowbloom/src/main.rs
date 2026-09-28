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
    --bench         measure rendering speed
    -h, --help      show this help

CONTROLS:
    WASD / arrows        move            Space / Shift   dodge roll
    J / Z / left click   use tool        E / right click interact, eat, place
    1-0 / wheel / Q R    hotbar          Tab / I         bag      C  crafting
    M                    minimap         Esc             pause    F11 fullscreen   F12 screenshot
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
