//! Fish art: every fish painted from its description (shape, colours and pattern), plus bait,
//! the odd things that come up on a line, fishy dishes and the fishing stat symbols.

use std::collections::HashMap;

use super::sprites::{BOWL, NO, TART, art};
use crate::game::fish::{FISH, FishDef, Pattern, Shape};
use crate::palette::*;
use crate::render::{TexBank, TexId, Texture};
use crate::util::hash2;

/// What each pixel of a fish is before it is coloured.
#[derive(Clone, Copy, PartialEq)]
enum Px {
    Empty,
    Body,
    Fin,
    Dark,
}

struct Canvas16 {
    px: [[Px; 16]; 16],
}

impl Canvas16 {
    fn new() -> Canvas16 {
        Canvas16 {
            px: [[Px::Empty; 16]; 16],
        }
    }

    fn fill(&mut self, what: Px, f: impl Fn(f32, f32) -> bool) {
        for y in 0..16 {
            for x in 0..16 {
                if self.px[y][x] == Px::Empty && f(x as f32 + 0.5, y as f32 + 0.5) {
                    self.px[y][x] = what;
                }
            }
        }
    }

    fn set(&mut self, x: i32, y: i32, what: Px) {
        if (0..16).contains(&x) && (0..16).contains(&y) {
            self.px[y as usize][x as usize] = what;
        }
    }

    fn get(&self, x: i32, y: i32) -> Px {
        if (0..16).contains(&x) && (0..16).contains(&y) {
            self.px[y as usize][x as usize]
        } else {
            Px::Empty
        }
    }
}

fn ellipse(cx: f32, cy: f32, rx: f32, ry: f32) -> impl Fn(f32, f32) -> bool {
    move |x, y| {
        let dx = (x - cx) / rx;
        let dy = (y - cy) / ry;
        dx * dx + dy * dy <= 1.0
    }
}

/// The silhouette of a shape, head to the right.
fn silhouette(shape: Shape) -> Canvas16 {
    let mut c = Canvas16::new();
    match shape {
        Shape::Slim => {
            c.fill(Px::Body, ellipse(9.0, 8.0, 5.0, 2.3));
            c.fill(Px::Fin, |x, y| {
                (2.0..5.0).contains(&x) && (y - 8.0).abs() <= (5.0 - x) * 0.9
            });
            c.set(8, 5, Px::Fin);
            c.set(9, 5, Px::Fin);
        }
        Shape::Round => {
            c.fill(Px::Body, ellipse(8.8, 8.2, 4.6, 4.0));
            for x in 6..=10 {
                c.set(x, 4, Px::Fin);
                if x % 2 == 0 {
                    c.set(x, 3, Px::Fin);
                }
            }
            c.fill(Px::Fin, |x, y| {
                (1.0..4.2).contains(&x) && (y - 8.2).abs() <= (4.6 - x) + 0.8
            });
            c.set(8, 12, Px::Fin);
        }
        Shape::Long => {
            c.fill(Px::Body, |x, y| {
                let half = if (4.0..13.0).contains(&x) { 1.4 } else { 0.9 };
                (2.0..15.0).contains(&x) && (y - 8.0).abs() <= half
            });
            for x in 4..=12 {
                c.set(x, 6, Px::Fin);
            }
            c.fill(Px::Fin, |x, y| {
                (0.0..2.0).contains(&x) && (y - 8.0).abs() <= 1.6
            });
        }
        Shape::Flat => {
            c.fill(Px::Body, |x, y| {
                (x - 9.5).abs() / 5.5 + (y - 8.0).abs() / 5.0 <= 1.0
            });
            c.fill(Px::Fin, |x, y| x < 4.0 && (y - 8.0).abs() < 0.6);
            c.set(0, 7, Px::Fin);
            c.set(0, 9, Px::Fin);
        }
        Shape::Puffer => {
            c.fill(Px::Body, ellipse(8.6, 8.2, 4.3, 4.3));
            for k in 0..12 {
                let a = k as f32 / 12.0 * std::f32::consts::TAU;
                let (x, y) = (8.6 + a.cos() * 5.4, 8.2 + a.sin() * 5.4);
                if x > 3.5 {
                    c.set(x as i32, y as i32, Px::Fin);
                }
            }
            c.fill(Px::Fin, |x, y| {
                (2.0..4.4).contains(&x) && (y - 8.2).abs() <= 1.6
            });
        }
        Shape::Fancy => {
            c.fill(Px::Body, ellipse(9.4, 8.0, 4.2, 3.1));
            for y in 2..=4 {
                let lo = 5 + (y - 2);
                for x in lo..=lo + 4 {
                    c.set(x, y, Px::Fin);
                }
            }
            c.fill(Px::Fin, |x, y| {
                x < 5.3 && (y - 8.0).abs() <= (5.4 - x) * 0.85 + 0.6 && (2.0..14.0).contains(&y)
            });
            c.set(9, 11, Px::Fin);
            c.set(9, 12, Px::Fin);
            c.set(8, 12, Px::Fin);
        }
        Shape::Whiskers => {
            c.fill(Px::Body, |x, y| {
                ellipse(8.0, 8.5, 6.0, 3.3)(x, y) && y <= 11.6
            });
            c.fill(Px::Body, ellipse(11.8, 8.3, 3.0, 3.4));
            for (x, y) in [(14, 10), (15, 11), (13, 11), (13, 12), (15, 9)] {
                if c.get(x, y) == Px::Empty {
                    c.set(x, y, Px::Dark);
                }
            }
            c.fill(Px::Fin, |x, y| {
                x < 2.6 && (y - 8.5).abs() <= (3.0 - x) + 0.9
            });
            for x in 7..=9 {
                c.set(x, 4, Px::Fin);
            }
        }
        Shape::Trout => {
            c.fill(Px::Body, ellipse(8.8, 8.0, 5.6, 2.7));
            c.fill(Px::Fin, |x, y| {
                let dy = (y - 8.0).abs();
                (1.0..3.6).contains(&x) && dy <= (4.4 - x) && !(x < 2.0 && dy < 1.0)
            });
            c.set(8, 5, Px::Fin);
            c.set(9, 5, Px::Fin);
            c.set(9, 4, Px::Fin);
            c.set(7, 11, Px::Fin);
        }
        Shape::Jelly => {
            c.fill(Px::Body, |x, y| {
                ellipse(8.0, 8.2, 5.4, 4.8)(x, y) && y <= 8.5
            });
            for (i, x) in [4, 6, 8, 10, 12].into_iter().enumerate() {
                for y in 9..=14 {
                    let wig = if (y / 2 + i as i32) % 2 == 0 { 0 } else { 1 };
                    c.set(x + wig, y, Px::Fin);
                }
            }
        }
        Shape::Cray => {
            c.fill(Px::Body, ellipse(7.4, 9.0, 4.6, 2.0));
            c.fill(Px::Body, ellipse(12.0, 9.0, 1.8, 1.6));
            c.fill(Px::Body, ellipse(13.4, 5.6, 1.9, 1.3));
            c.fill(Px::Body, ellipse(13.4, 12.4, 1.9, 1.3));
            c.set(12, 7, Px::Body);
            c.set(12, 11, Px::Body);
            c.fill(Px::Fin, |x, y| {
                (1.0..3.0).contains(&x) && (y - 9.0).abs() <= 2.2
            });
            for x in [6, 8, 10] {
                c.set(x, 11, Px::Dark);
                c.set(x - 1, 12, Px::Dark);
            }
            c.set(14, 8, Px::Dark);
            c.set(15, 7, Px::Dark);
            c.set(15, 10, Px::Dark);
        }
    }
    c
}

/// Paints a fish: dark back, lighter belly, the pattern, fins, an eye and an ink outline.
pub fn fish_icon(d: &FishDef) -> Texture {
    let c = silhouette(d.shape);
    let mut t = Texture::clear(16, 16);
    let seed = d.item as u32;
    for x in 0..16i32 {
        // How far down the body each pixel is, for shading back to belly.
        let rows: Vec<i32> = (0..16).filter(|&y| c.get(x, y) == Px::Body).collect();
        let (top, bot) = match (rows.first(), rows.last()) {
            (Some(a), Some(b)) => (*a, *b),
            _ => (0, 0),
        };
        for y in 0..16i32 {
            let col = match c.get(x, y) {
                Px::Empty => continue,
                Px::Fin => d.fin,
                Px::Dark => d.colors[2],
                Px::Body => {
                    let rel = if bot > top {
                        (y - top) as f32 / (bot - top) as f32
                    } else {
                        0.5
                    };
                    let mut col = if d.shape == Shape::Jelly {
                        if rel < 0.5 { d.colors[0] } else { d.colors[1] }
                    } else if rel < 0.3 {
                        d.colors[2]
                    } else if rel < 0.68 {
                        d.colors[1]
                    } else {
                        d.colors[0]
                    };
                    let h = hash2(x, y, seed);
                    let on = match d.pattern {
                        Pattern::Plain => false,
                        Pattern::Stripes => x % 3 == 1 && rel < 0.8,
                        Pattern::Band => (0.4..0.64).contains(&rel),
                        Pattern::Lines => {
                            ((0.28..0.42).contains(&rel) || (0.58..0.72).contains(&rel))
                                && x % 2 == 0
                        }
                        Pattern::Spots => h % 5 == 0 && rel < 0.85,
                        Pattern::Belly => rel >= 0.62,
                        Pattern::Speckle => h % 3 == 0 && rel < 0.6,
                        Pattern::Glow => h % 6 == 0,
                        Pattern::Bones => {
                            (x % 2 == 0 && (0.2..0.8).contains(&rel)) || (0.45..0.55).contains(&rel)
                        }
                    };
                    if on {
                        col = d.accent;
                    }
                    col
                }
            };
            t.set(x, y, col);
        }
    }
    // The eye, near the front of the head.
    if d.shape != Shape::Jelly {
        let ey = match d.shape {
            Shape::Cray => 8,
            Shape::Whiskers => 7,
            _ => 7,
        };
        let front = (0..16)
            .rev()
            .find(|&x| c.get(x, ey) == Px::Body)
            .unwrap_or(12);
        let big = matches!(
            d.shape,
            Shape::Round | Shape::Fancy | Shape::Whiskers | Shape::Puffer
        );
        if d.shape == Shape::Flat {
            t.set(front - 2, 6, INK);
            t.set(front - 2, 9, INK);
        } else if big {
            t.set(front - 2, ey, WHITE);
            t.set(front - 1, ey, INK);
        } else {
            t.set(front - 1, ey, INK);
        }
        if d.shape == Shape::Puffer {
            t.set(front, ey + 2, INK);
        }
    } else {
        t.set(6, 6, INK);
        t.set(9, 6, INK);
        t.set(7, 4, WHITE);
    }
    outline(&mut t);
    t
}

/// An ink line around everything drawn.
fn outline(t: &mut Texture) {
    let src = t.clone();
    for y in 0..16 {
        for x in 0..16 {
            if src.get(x, y) != CLEAR {
                continue;
            }
            let near = [(1, 0), (-1, 0), (0, 1), (0, -1)].iter().any(|(dx, dy)| {
                let (nx, ny) = (x + dx, y + dy);
                (0..16).contains(&nx) && (0..16).contains(&ny) && src.get(nx, ny) != CLEAR
            });
            if near {
                t.set(x, y, INK);
            }
        }
    }
}

const BAIT: &[&str] = &[
    "................",
    "................",
    "......PP........",
    ".....P..P..P....",
    "....KKKPKKKPKK..",
    "...KhhhhhhhhhhK.",
    "...KnnnnnnnnnnK.",
    "...KRPsPPsPsRRK.",
    "...KRRuRRuRRuRK.",
    "...KRRRRRRRRRRK.",
    "...KDDDDDDDDDDK.",
    "...KkkkkkkkkkkK.",
    "....KKKKKKKKKK..",
    "................",
];

const SEAWEED: &[&str] = &[
    "................",
    "......K..K......",
    ".....KgKKlK.....",
    "....KgK.KlK.K...",
    "....KgK.KgKKlK..",
    ".....KgKKgKKlK..",
    ".....KgK.KgKlK..",
    "....KgK.KgK.KgK.",
    "....KgKKgK.KgK..",
    ".....KgKgK.KgK..",
    ".....KtKgKKgK...",
    "....KtK.KtKgK...",
    "....KtK.KtKtK...",
    ".....KKKKKKKK...",
];

const BOOT: &[&str] = &[
    "................",
    ".......S........",
    "....KKKKKK......",
    "...KuCCCCuK.S...",
    "...KuCCCCuK.....",
    "...KuCCCCuK..S..",
    "...KuCCCCuK.....",
    "...KuCCCCuK.....",
    "...KuCCCCuKKKK..",
    "...KuCCCCCCCCuK.",
    "...KuCCCCCCCCCuK",
    "...KmmmmmmmmmmmK",
    "....KKKKKKKKKKK.",
    ".......S...S....",
];

const TIN: &[&str] = &[
    "................",
    "................",
    ".....KKKKKK.....",
    "....KwhhhhwK....",
    "....KKKKKKKK....",
    "....KhnhhhDK....",
    "....KrrrrrrK....",
    "....KryyyyrK....",
    "....KrrrrrrK....",
    "....KhnhhhDK....",
    "....KhnhhhDK....",
    "....KKKKKKKK....",
    "................",
];

/// A fish on a plate: 0-2 the fish, 3 what's alongside.
const FISH_PLATE: &[&str] = &[
    "................",
    "................",
    "................",
    "................",
    "......KKKKK.....",
    "....KK01110KK...",
    "..KK01112111KK..",
    ".K3K1112221K1K..",
    ".K33K22222KK1K..",
    "..K33KKKKK3KK...",
    ".KwSK33333KSwK..",
    "KwwSSSSSSSSSSwwK",
    ".KwwwwwwwwwwwwK.",
    "..KKKKKKKKKKKK..",
];

const SUSHI: &[&str] = &[
    "................",
    "................",
    "................",
    "..KKKK....KKKK..",
    ".KTwwTK..KTwwTK.",
    ".KTwsTK..KTwsTK.",
    ".KTwwTK..KTwwTK.",
    "..KKKK.KK.KKKK..",
    "......KTTK......",
    ".....KwsswK.....",
    ".....KwwwwK.....",
    ".....KTTTTK.....",
    "..KKKKKKKKKKKK..",
    "..KuCCCCCCCCuK..",
    "...KKKKKKKKKK...",
];

const TACO: &[&str] = &[
    "................",
    "................",
    "................",
    "................",
    "......KKKKK.....",
    "....KKgrgsgKK...",
    "...KgrgYsrgrgK..",
    "..KYYsYYsYYsYYK.",
    ".KYYYYYYYYYYYYYK",
    ".KYCYYYCYYYYCYK.",
    "..KYYYYYYCYYYK..",
    "...KKYYYYYYKK...",
    ".....KKKKKK.....",
];

/// A fish kebab: 0-2 the pieces, 3 the veg between them.
const FISH_SKEWER: &[&str] = &[
    "............K...",
    "...........KnK..",
    "........KKKKK...",
    ".......K0012K...",
    ".......K1122K...",
    "......KKKKKK....",
    ".....K333K......",
    ".....K333K......",
    "....KKKKK.......",
    "...K001K........",
    "...K122K........",
    "..KKKKK.........",
    ".KnK............",
    "KnK.............",
    "KK..............",
];

const ST_LURE: &[&str] = &[
    "...KK...", "...KMK..", "...KMK..", "...KMK..", "K..KMK..", "KMKKMK..", ".KMMK...", "..KK....",
];
const ST_LINE: &[&str] = &[
    "..KKKK..", ".KSSSSK.", "KSKKKKSK", "KSKwwKSK", "KSKwwKSK", "KSKKKKSK", ".KSSSSK.", "..KKKK..",
];
const ST_TREASURE: &[&str] = &[
    "........", ".KKKKKK.", "KYyYYyYK", "KKKKKKKK", "KYYKKYYK", "KYYyYYYK", "KKKKKKKK", "........",
];
const ST_ANGLER: &[&str] = &[
    "........", "........", "K..KKK..", "KKKaaaK.", "KaaaawaK", "KKKaaaK.", "K..KKK..", "........",
];

pub fn build(bank: &mut TexBank, icons: &mut HashMap<&'static str, TexId>) {
    let mut add = |name: &'static str, t: Texture| {
        icons.insert(name, bank.add(t));
    };
    for d in FISH.iter() {
        add(d.item.def().icon, fish_icon(d));
    }
    add("bait", art(BAIT, NO));
    add("seaweed", art(SEAWEED, NO));
    add("soggy_boot", art(BOOT, NO));
    add("tin_can", art(TIN, NO));
    // Fishy dishes.
    add("fish_and_chips", art(FISH_PLATE, [CREAM, GOLD, CLAY, GOLD]));
    add(
        "grilled_trout",
        art(FISH_PLATE, [PEACH, SALMON, RUST, LIME]),
    );
    add(
        "minnow_fritters",
        art(FISH_PLATE, [CREAM, GOLD, ORANGE, GREEN]),
    );
    add("sushi", art(SUSHI, NO));
    add("fish_stew", art(BOWL, [SKY, CREAM, ORANGE, BLUE]));
    add("carp_curry", art(BOWL, [GOLD, ORANGE, CREAM, CLAY]));
    add(
        "salmon_steak",
        art(FISH_PLATE, [BLUSH, SALMON, CRIMSON, LIME]),
    );
    add("crayfish_boil", art(BOWL, [RED, SALMON, GOLD, MAROON]));
    add(
        "cave_skewer",
        art(FISH_SKEWER, [BLUSH, PEACH, SALMON, MINT]),
    );
    add("ceviche", art(FISH_PLATE, [WHITE, MINT, AQUA, GOLD]));
    add("smelt_pie", art(TART, [CLEAR, WHITE, SKY, CLEAR]));
    add("eel_kebab", art(FISH_SKEWER, [GOLD, ORANGE, RED, GREEN]));
    add("fish_tacos", art(TACO, NO));
    add(
        "emperor_platter",
        art(FISH_PLATE, [CREAM, GOLD, CLAY, SHADOW]),
    );
    // Stat symbols.
    add("st_lure", art(ST_LURE, NO));
    add("st_line", art(ST_LINE, NO));
    add("st_treasure", art(ST_TREASURE, NO));
    add("st_angler", art(ST_ANGLER, NO));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fish_art_is_tidy() {
        for rows in [
            BAIT,
            SEAWEED,
            BOOT,
            TIN,
            FISH_PLATE,
            SUSHI,
            TACO,
            FISH_SKEWER,
        ] {
            assert!(rows.len() <= 16);
            assert!(rows.iter().all(|r| r.chars().count() <= 16), "{rows:?}");
        }
        for rows in [ST_LURE, ST_LINE, ST_TREASURE, ST_ANGLER] {
            assert!(rows.iter().all(|r| r.chars().count() <= 8), "{rows:?}");
        }
        // Every fish looks different from every other.
        let icons: Vec<Texture> = FISH.iter().map(fish_icon).collect();
        for (i, a) in icons.iter().enumerate() {
            let filled = a.data.iter().filter(|c| **c != CLEAR).count();
            assert!(filled > 30, "{:?} is nearly empty", FISH[i].item);
            for b in icons.iter().skip(i + 1) {
                assert_ne!(a.data, b.data);
            }
        }
    }
}
