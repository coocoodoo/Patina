//! The farm: layout generation and the overnight update (growth, sprinklers, weeds).

use super::items::{AUTUMN, Crop, SPRING, WINTER};
use super::world::{Area, FERTILE, Floor, Obj, WATERED, Wall, World};
use crate::util::{Rng, hash2};

pub const FARM_W: i32 = 64;
pub const FARM_H: i32 = 48;

/// Important places on the farm.
pub struct Landmarks {
    pub house: (i32, i32),
    pub door: (i32, i32),
    pub bin: (i32, i32),
    pub hollow: (i32, i32),
    pub stall: (i32, i32),
    pub spawn: (i32, i32),
    pub enchant: (i32, i32),
    /// The bus shelter (2x1, anchored at its west end).
    pub stop: (i32, i32),
}

/// The country road along the east edge, two tiles wide.
pub const ROAD_X: i32 = 60;

pub const MARKS: Landmarks = Landmarks {
    house: (28, 9),
    door: (29, 10),
    bin: (32, 10),
    hollow: (52, 17),
    stall: (44, 12),
    spawn: (29, 11),
    enchant: (34, 9),
    stop: (57, 18),
};

/// Can the hoe turn this tile? Any open ground on the farm will do: lawn, bare soil, the
/// paths and the sand by the pond. Not water, walls, laid floors or the road to town.
pub fn tillable(w: &World, x: i32, z: i32) -> bool {
    matches!(
        w.floor(x, z),
        Floor::Grass | Floor::Soil | Floor::Path | Floor::Sand
    ) && w.wall(x, z) == Wall::None
        && w.obj(x, z)
            .is_none_or(|o| matches!(o, Obj::Weed { .. } | Obj::Flower { .. }))
}

fn reserved(x: i32, z: i32) -> bool {
    // Keep structures, paths and the starter field clear of clutter.
    let near = |cx: i32, cz: i32, rx: i32, rz: i32| (x - cx).abs() <= rx && (z - cz).abs() <= rz;
    near(30, 8, 4, 4)
        || near(52, 17, 3, 3)
        || near(45, 12, 3, 2)
        || near(28, 15, 5, 3)
        || (x == 29 && (10..=21).contains(&z))
        || (z == 20 && (29..=52).contains(&x))
        || (x == 52 && (18..=20).contains(&z))
        || (x >= 53 && (17..=21).contains(&z))
        || x >= ROAD_X - 1
}

pub fn generate(seed: u64) -> World {
    let mut w = World::new(FARM_W, FARM_H, Area::Farm, 0);
    let mut r = Rng::new(seed ^ 0xFA_4A);
    for z in 0..FARM_H {
        for x in 0..FARM_W {
            w.set_floor(x, z, Floor::Grass);
        }
    }
    // Cliffs along the north edge and the sides, trees along the south.
    for x in 0..FARM_W {
        let depth = 3 + (hash2(x / 3, 0, 5) % 2) as i32;
        for z in 0..depth {
            w.set_wall(x, z, Wall::Cliff);
        }
    }
    for z in 0..FARM_H {
        let d = 2 + (hash2(0, z / 4, 6) % 2) as i32;
        for x in 0..d {
            w.set_wall(x, z, Wall::Cliff);
            w.set_wall(FARM_W - 1 - x, z, Wall::Cliff);
        }
    }
    for x in 0..FARM_W {
        for z in FARM_H - 3..FARM_H {
            if w.wall(x, z) == Wall::None {
                let o = if (x + z) % 2 == 0 {
                    Obj::Pine {
                        var: (x % 2) as u8,
                        hp: 8,
                    }
                } else {
                    Obj::Tree {
                        var: (hash2(x, z, 1) % 2) as u8 * 3,
                        hp: 6,
                    }
                };
                w.set_obj(x, z, Some(o));
            }
        }
    }

    // Pond with a sandy rim.
    let (px, pz) = (13.0f32, 31.0f32);
    for z in 22..42 {
        for x in 4..24 {
            let dx = (x as f32 + 0.5 - px) / 6.5;
            let dz = (z as f32 + 0.5 - pz) / 4.8;
            let wob = (hash2(x, z, 9) % 100) as f32 / 400.0;
            let d = dx * dx + dz * dz + wob;
            if d < 1.0 {
                w.set_floor(x, z, Floor::Water);
            } else if d < 1.45 {
                w.set_floor(x, z, Floor::Sand);
            }
        }
    }

    // Paths.
    for z in 10..=20 {
        w.set_floor(29, z, Floor::Path);
    }
    for x in 29..=52 {
        w.set_floor(x, 20, Floor::Path);
    }
    for z in 18..=20 {
        w.set_floor(52, z, Floor::Path);
    }
    for x in 20..29 {
        w.set_floor(x, 20, Floor::Path);
    }
    for z in 20..27 {
        w.set_floor(20, z, Floor::Path);
    }
    for x in 44..=46 {
        w.set_floor(x, 13, Floor::Path);
    }
    w.set_floor(45, 14, Floor::Path);
    for x in 29..=45 {
        if x % 5 != 0 {
            w.set_floor(x, 14, Floor::Path);
        }
    }

    // Farmhouse, 4x3, anchored at its front-left tile.
    let (hx, hz) = MARKS.house;
    for dz in -2..=0 {
        for dx in 0..4 {
            let (x, z) = (hx + dx, hz + dz);
            if dx == 0 && dz == 0 {
                w.set_obj(x, z, Some(Obj::House));
            } else {
                w.set_obj(
                    x,
                    z,
                    Some(Obj::Part {
                        ax: hx as i16,
                        az: hz as i16,
                    }),
                );
            }
        }
    }
    w.set_obj(MARKS.bin.0, MARKS.bin.1, Some(Obj::Bin));
    w.set_obj(27, 10, Some(Obj::FlowerPot { var: 0 }));
    w.set_obj(33, 11, Some(Obj::Lamp));
    w.set_obj(24, 12, Some(Obj::Bench));
    w.set_obj(MARKS.enchant.0, MARKS.enchant.1, Some(Obj::EnchantTable));

    // The Hollow entrance, 3x2, anchored at the opening.
    let (ex, ez) = MARKS.hollow;
    for dz in -1..=0 {
        for dx in -1..=1 {
            let (x, z) = (ex + dx, ez + dz);
            if dx == 0 && dz == 0 {
                w.set_obj(x, z, Some(Obj::Hollow));
            } else {
                w.set_obj(
                    x,
                    z,
                    Some(Obj::Part {
                        ax: ex as i16,
                        az: ez as i16,
                    }),
                );
            }
        }
    }
    w.set_obj(ex + 2, ez + 2, Some(Obj::Sign { text: 0 }));
    w.set_obj(ex - 2, ez + 1, Some(Obj::Torch));
    w.set_obj(ex + 2, ez + 1, Some(Obj::Torch));

    // Merchant stall, 2x1.
    let (sx, sz) = MARKS.stall;
    w.set_obj(sx, sz, Some(Obj::Stall));
    w.set_obj(
        sx + 1,
        sz,
        Some(Obj::Part {
            ax: sx as i16,
            az: sz as i16,
        }),
    );
    w.set_obj(sx - 1, sz + 1, Some(Obj::Sign { text: 1 }));

    lay_road(&mut w);

    // Starter field: tilled soil, a few turnips already sprouting.
    for z in 13..16 {
        for x in 24..30 {
            w.set_floor(x, z, Floor::Tilled);
        }
    }
    for x in 24..27 {
        w.set_obj(
            x,
            13,
            Some(Obj::Crop {
                crop: Crop::Turnip,
                days: 2,
                harvested: false,
            }),
        );
        w.set_flag(x, 13, WATERED, true);
    }
    w.set_obj(
        27,
        13,
        Some(Obj::Crop {
            crop: Crop::Turnip,
            days: 4,
            harvested: false,
        }),
    );

    // Scatter nature over what is left.
    let place = |w: &mut World, r: &mut Rng, n: usize, f: &dyn Fn(&mut Rng) -> Obj| {
        let mut tries = 0;
        let mut placed = 0;
        while placed < n && tries < n * 30 {
            tries += 1;
            let x = r.range(3, FARM_W - 3);
            let z = r.range(4, FARM_H - 3);
            if reserved(x, z) || w.wall(x, z) != Wall::None || w.obj(x, z).is_some() {
                continue;
            }
            if w.floor(x, z) != Floor::Grass {
                continue;
            }
            w.set_obj(x, z, Some(f(r)));
            placed += 1;
        }
    };
    place(&mut w, &mut r, 36, &|r| {
        if r.chance(0.3) {
            Obj::Pine {
                var: r.below(2) as u8,
                hp: 8,
            }
        } else {
            Obj::Tree {
                var: r.below(4) as u8,
                hp: 6,
            }
        }
    });
    place(&mut w, &mut r, 22, &|r| Obj::Rock {
        var: r.below(3) as u8,
        hp: 4,
    });
    place(&mut w, &mut r, 4, &|_| Obj::Boulder { hp: 14 });
    place(&mut w, &mut r, 5, &|_| Obj::Stump { hp: 8 });
    place(&mut w, &mut r, 3, &|_| Obj::Log { hp: 12 });
    place(&mut w, &mut r, 110, &|r| Obj::Weed {
        var: r.below(2) as u8,
    });
    place(&mut w, &mut r, 40, &|r| Obj::Flower {
        var: r.below(4) as u8,
    });
    place(&mut w, &mut r, 14, &|r| Obj::Shrub {
        var: [0, 0, 1, 2][r.below(4)],
        hp: 4,
    });
    w
}

/// The road to town along the east edge, the path out to it and the bus shelter. Also run
/// on farms saved before the bus came, so it clears whatever grew in the way.
pub fn lay_road(w: &mut World) {
    for z in 0..FARM_H {
        for x in ROAD_X..ROAD_X + 2 {
            w.set_wall(x, z, Wall::None);
            w.set_obj(x, z, None);
            w.set_floor(x, z, Floor::Street);
        }
        // A grassy verge between the road and the cliff.
        if w.wall(ROAD_X + 2, z) == Wall::Cliff && w.wall(ROAD_X + 3, z) == Wall::Cliff {
            w.set_wall(ROAD_X + 2, z, Wall::None);
            w.set_floor(ROAD_X + 2, z, Floor::Grass);
        }
        if w.wall(ROAD_X - 1, z) != Wall::None {
            w.set_wall(ROAD_X - 1, z, Wall::None);
            w.set_floor(ROAD_X - 1, z, Floor::Grass);
        }
    }
    let (sx, sz) = MARKS.stop;
    for x in 53..ROAD_X {
        if !matches!(w.obj(x, 20), Some(Obj::Chest { .. })) {
            w.set_obj(x, 20, None);
        }
        w.set_wall(x, 20, Wall::None);
        w.set_floor(x, 20, Floor::Path);
    }
    for x in sx..sx + 2 {
        for z in sz..=sz + 1 {
            if !matches!(w.obj(x, z), Some(Obj::Chest { .. })) {
                w.set_obj(x, z, None);
            }
            w.set_wall(x, z, Wall::None);
        }
        w.set_floor(x, sz + 1, Floor::Path);
    }
    w.set_obj(sx, sz, Some(Obj::BusStop));
    w.set_obj(
        sx + 1,
        sz,
        Some(Obj::Part {
            ax: sx as i16,
            az: sz as i16,
        }),
    );
    w.set_obj(sx - 1, sz + 1, Some(Obj::Sign { text: 2 }));
}

/// What happened overnight, for the morning summary.
#[derive(Default)]
pub struct Night {
    pub grown: u32,
    pub ready: u32,
    /// Weeds, flowers and bushes that sprang up.
    pub sprouted: u32,
    /// Crops lost to the change of season.
    pub withered: u32,
}

/// Advances the farm by one day, into the season with bit `season` (`turned` on its first
/// morning, when anything that doesn't grow in it withers).
pub fn new_day(w: &mut World, day: u32, rain: bool, season: u8, turned: bool) -> Night {
    let mut night = Night::default();
    let mut r = Rng::new(day as u64 * 7919);
    // Sprinklers water first so today's growth counts them.
    let mut sprinklers = Vec::new();
    for z in 0..w.h {
        for x in 0..w.w {
            if let Some(Obj::Sprinkler { tier }) = w.obj(x, z) {
                sprinklers.push((x, z, *tier));
            }
        }
    }
    if turned {
        for z in 0..w.h {
            for x in 0..w.w {
                if let Some(Obj::Crop { crop, .. }) = w.obj(x, z) {
                    if !crop.grows_in(season) {
                        // Out of season: it wilts to a dry, straggly weed.
                        w.set_obj(x, z, Some(Obj::Weed { var: 1 }));
                        night.withered += 1;
                    }
                }
            }
        }
    }
    for z in 0..w.h {
        for x in 0..w.w {
            let watered = rain || w.flag(x, z, WATERED);
            let fertile = w.flag(x, z, FERTILE);
            if let Some(Obj::Crop {
                crop,
                days,
                harvested,
            }) = w.obj_mut(x, z)
            {
                let total = crop.def().days;
                if watered && *days < total {
                    *days += 1;
                    // Growth from an enchanted can: an extra day now and then.
                    if fertile && *days < total {
                        *days += 1;
                    }
                    night.grown += 1;
                    if *days >= total {
                        night.ready += 1;
                    }
                }
                let _ = harvested;
            }
            w.set_flag(x, z, WATERED, false);
            w.set_flag(x, z, FERTILE, false);
            // Bare tilled soil sometimes settles back.
            if w.floor(x, z) == Floor::Tilled && w.obj(x, z).is_none() && r.chance(0.1) {
                w.set_floor(x, z, Floor::Soil);
            }
        }
    }
    for (x, z, tier) in sprinklers {
        let reach: &[(i32, i32)] = match tier {
            0 => &[(1, 0), (-1, 0), (0, 1), (0, -1)],
            1 => &[
                (1, 0),
                (-1, 0),
                (0, 1),
                (0, -1),
                (1, 1),
                (-1, 1),
                (1, -1),
                (-1, -1),
            ],
            _ => &[],
        };
        let mut apply = |tx: i32, tz: i32| {
            if w.floor(tx, tz) == Floor::Tilled {
                w.set_flag(tx, tz, WATERED, true);
            }
        };
        if tier >= 2 {
            for dz in -2..=2 {
                for dx in -2..=2 {
                    if dx != 0 || dz != 0 {
                        apply(x + dx, z + dz);
                    }
                }
            }
        } else {
            for (dx, dz) in reach {
                apply(x + dx, z + dz);
            }
        }
    }
    if rain {
        for z in 0..w.h {
            for x in 0..w.w {
                if w.floor(x, z) == Floor::Tilled {
                    w.set_flag(x, z, WATERED, true);
                }
            }
        }
    }
    night.sprouted = grow_wild(w, &mut r, season);
    night
}

/// Overnight the wild creeps back in: weeds, wildflowers and the odd bush spring up on empty
/// ground. Never on paths, dug soil or right by the house, and it eases off once the farm is
/// already overgrown. Returns how many things sprouted.
pub fn grow_wild(w: &mut World, r: &mut Rng, season: u8) -> u32 {
    // Snow lies over everything in winter: nothing springs up.
    if season == WINTER {
        return 0;
    }
    let wild = w
        .objs
        .iter()
        .flatten()
        .filter(|o| matches!(o, Obj::Weed { .. } | Obj::Shrub { .. }))
        .count();
    let (weeds, bushes) = if wild > 260 {
        (2, 0)
    } else if wild > 180 {
        (5, 1)
    } else {
        (9 + r.below(6), 2 + r.below(3))
    };
    let mut sprouted = 0;
    let mut place = |w: &mut World, r: &mut Rng, n: usize, bush: bool| {
        let mut left = n;
        for _ in 0..n * 25 {
            if left == 0 {
                break;
            }
            let x = r.range(3, w.w - 3);
            let z = r.range(4, w.h - 3);
            let open = matches!(w.floor(x, z), Floor::Grass | Floor::Soil)
                && w.obj(x, z).is_none()
                && w.wall(x, z) == Wall::None
                && !reserved(x, z);
            // Bushes keep a step back from paths so they never block the way.
            let roomy = !bush
                || [(1, 0), (-1, 0), (0, 1), (0, -1)].iter().all(|(dx, dz)| {
                    !matches!(
                        w.floor(x + dx, z + dz),
                        Floor::Path | Floor::Planks | Floor::Cobble | Floor::Street
                    )
                });
            if !open || !roomy {
                continue;
            }
            // Spring is full of wildflowers; autumn has few.
            let weedy = match season {
                SPRING => 0.55,
                AUTUMN => 0.9,
                _ => 0.82,
            };
            let o = if bush {
                let var = [0, 0, 0, 1, 1, 2][r.below(6)];
                Obj::Shrub { var, hp: 4 }
            } else if r.chance(weedy) {
                Obj::Weed {
                    var: r.below(2) as u8,
                }
            } else {
                Obj::Flower {
                    var: r.below(4) as u8,
                }
            };
            w.set_obj(x, z, Some(o));
            left -= 1;
            sprouted += 1;
        }
    };
    place(w, r, weeds, false);
    place(w, r, bushes, true);
    sprouted
}
