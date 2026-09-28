//! Drawing the playing state: the world, loot on the ground, magic, creatures and the
//! hero in whatever they are wearing.

use glam::{Mat4, Vec2, Vec3};

use super::draw::{self, Outfit, Pose, Swing, draw_humanoid, flames, full_uv};
use super::fish::{self, Hooked, Phase, Water};
use super::fx::shot_colors;
use super::gear::{Rarity, Slot};
use super::home::{self, WINDOWS};
use super::items::{Kind, Placeable, Stack};
use super::play::Play;
use super::player::ActKind;
use super::town::Decor;
use super::travel::area_world_mut;
use super::world::{Area, Wall};
use crate::assets::Assets;
use crate::palette::*;
use crate::render::{DrawOpts, Light, Mesh, Mode, PointLight, Renderer, TexId, UvRect};
use crate::util::hash2;

/// Meshes and texture swaps for what someone is wearing and holding.
pub struct Dressed<'a> {
    pub held: Option<&'a Mesh>,
    pub hat: Option<&'a Mesh>,
    pub boot: Option<&'a Mesh>,
    pub shield: Option<&'a Mesh>,
    pub remap: Vec<(TexId, TexId)>,
}

impl<'a> Dressed<'a> {
    /// The outfit with a different set of texture swaps.
    pub fn outfit_with(&'a self, remap: &'a [(TexId, TexId)]) -> Outfit<'a> {
        Outfit {
            held: self.held,
            hat: self.hat,
            sprout: None,
            boot: self.boot,
            shield: self.shield,
            remap,
        }
    }

    pub fn outfit(&'a self, sprout: Option<&'a Mesh>) -> Outfit<'a> {
        Outfit {
            held: self.held,
            hat: self.hat,
            sprout,
            boot: self.boot,
            shield: self.shield,
            remap: &self.remap,
        }
    }
}

/// A villager's clothes: their gear, with swaps aimed at their own textures.
pub fn villager_dress(a: &Assets, v: super::folk::Villager) -> (Dressed<'_>, Vec<(TexId, TexId)>) {
    let h = &a.folk[v as usize];
    let equip = super::folk::outfit(v);
    let held = v.def().holds.map(|i| Stack::new(i, 1));
    let dressed = dress(a, &equip, held.as_ref());
    let mut remap = Vec::new();
    for (from, to) in &dressed.remap {
        let t = if *from == a.hero.tex.body {
            h.tex.body
        } else if *from == a.hero.tex.arm {
            h.tex.arm
        } else if *from == a.hero.tex.leg {
            h.tex.leg
        } else {
            *from
        };
        remap.push((t, *to));
    }
    (dressed, remap)
}

/// Looks up the hero's clothes and the item in hand.
pub fn dress<'a>(a: &'a Assets, equip: &[Option<Stack>; 5], held: Option<&Stack>) -> Dressed<'a> {
    let icon = |slot: Slot| equip[slot as usize].map(|s| s.item.def().icon);
    let mut remap = Vec::new();
    if let Some((body, arm)) = icon(Slot::Chest).and_then(|i| a.chest_skin(i)) {
        remap.push((a.hero.tex.body, body));
        remap.push((a.hero.tex.arm, arm));
    }
    if let Some(leg) = icon(Slot::Legs).and_then(|i| a.leg_skin(i)) {
        remap.push((a.hero.tex.leg, leg));
    }
    Dressed {
        held: held
            .filter(|s| s.item.class().is_some_and(|c| !c.is_armor()))
            .and_then(|s| a.held_mesh(s.item.def().icon)),
        hat: icon(Slot::Head).and_then(|i| a.hat_mesh(i)),
        boot: icon(Slot::Feet).and_then(|i| a.boot_mesh(i)),
        shield: icon(Slot::Shield).and_then(|i| a.shield_mesh(i)),
        remap,
    }
}

impl Play {
    pub fn draw_scene(&mut self, r: &mut Renderer, a: &Assets) {
        r.cam = self.cam.clone();
        r.cam.update(r.width(), r.height());
        let env = self.env();
        let mut lights = Vec::new();
        let ppos = self.player.world_pos();
        match self.area {
            Area::Hollow { .. } => lights.push(PointLight {
                pos: ppos + Vec3::Y * 1.2,
                radius: 6.8,
                power: 0.8,
                warmth: 5.0,
            }),
            Area::Farm | Area::Town if env.night > 0.2 => lights.push(PointLight {
                pos: ppos + Vec3::Y * 1.0,
                radius: 3.8,
                power: 0.4 * env.night,
                warmth: 6.0,
            }),
            _ => {}
        }
        for f in &self.foes {
            if let Some(l) = f.light() {
                lights.push(l);
            }
            if f.burn > 0.0 {
                lights.push(PointLight {
                    pos: f.world_pos() + Vec3::Y * 0.4,
                    radius: 2.0,
                    power: 0.4,
                    warmth: 8.0,
                });
            }
        }
        for s in &self.shots {
            lights.push(PointLight {
                pos: s.world_pos(),
                radius: 2.2,
                power: 0.6,
                warmth: if s.color == ORANGE { 8.0 } else { 2.0 },
            });
        }
        for b in &self.bolts {
            lights.push(PointLight {
                pos: b.world_pos(),
                radius: 2.4,
                power: 0.7,
                warmth: if b.colors[2] == ORANGE { 8.0 } else { 2.5 },
            });
        }
        for f in &self.flashes {
            lights.push(PointLight {
                pos: f.pos,
                radius: 4.5,
                power: 1.2 * (1.0 - f.t / 0.35),
                warmth: f.warmth,
            });
        }
        self.fx.spark_lights(&mut lights);
        self.spell_lights(&mut lights);
        if self.player.ward_t > 0.0 {
            lights.push(PointLight {
                pos: ppos + Vec3::Y * 0.8,
                radius: 2.2,
                power: 0.3,
                warmth: 2.0,
            });
        }
        // Treasure glows a little.
        for d in &self.drops {
            if d.stack.rarity().is_some_and(|r| r >= Rarity::Epic) {
                lights.push(PointLight {
                    pos: d.pos + Vec3::Y * 0.4,
                    radius: 2.2,
                    power: 0.45,
                    warmth: 3.0,
                });
            }
        }
        // Fireflies on warm summer nights.
        let fireflies: Vec<Vec3> = if self.area == Area::Farm && env.night > 0.5 {
            (0..10)
                .map(|i| {
                    let t = self.time * 0.3 + i as f32 * 1.7;
                    let base = self.cam.target;
                    Vec3::new(
                        base.x + (t * 1.3).sin() * 7.0 + (i as f32 * 2.1).cos() * 3.0,
                        0.6 + (t * 2.0).sin() * 0.3,
                        base.z + (t * 0.9).cos() * 4.0,
                    )
                })
                .collect()
        } else {
            Vec::new()
        };
        if let Some(l) = self.bus_light(env.night) {
            lights.push(l);
        }
        if let Some(room) = &self.room {
            for d in &room.decor {
                if let Decor::Sconce { x } = d {
                    lights.push(PointLight {
                        pos: Vec3::new(*x as f32 + 0.5, 1.4, 1.3),
                        radius: 4.0,
                        power: 0.35,
                        warmth: 6.0,
                    });
                }
            }
        }
        let world = area_world_mut(
            self.area,
            &mut self.farm,
            &mut self.town,
            &mut self.house.world,
            &mut self.level,
            &mut self.room,
        );
        draw::draw_world(r, a, world, &env, &lights);
        self.draw_decor(r, a);
        if self.area == Area::Home {
            self.draw_home(r, a);
        }
        self.draw_bus(r, a, self.sky_night());

        // Target cursor.
        if let Some((tx, tz)) = self.target {
            let kind = self.player.held().map(|i| i.def().kind);
            let shows = if self.area == Area::Home {
                self.target_ok
                    || matches!(
                        kind,
                        Some(Kind::Place(_) | Kind::Wallpaper(_) | Kind::Flooring(_))
                    )
            } else {
                match kind {
                    Some(Kind::Gear(b)) => {
                        super::player::ActKind::for_class(b.class).is_some_and(|k| {
                            !matches!(k, ActKind::Slash | ActKind::Bolt | ActKind::Blast)
                        })
                    }
                    Some(Kind::Seed(_) | Kind::Place(_)) => true,
                    _ => false,
                }
            };
            if shows && self.fade.is_none() && self.cooking.is_none() {
                let tex = if self.target_ok {
                    a.cursor
                } else {
                    a.cursor_bad
                };
                let y = if self.world().wall(tx, tz) != Wall::None {
                    1.02
                } else {
                    0.03
                };
                let pulse = (self.time * 6.0).sin() * 0.03;
                // Indoors the cursor covers the whole piece being placed or aimed at.
                let (c, half) = self.cursor_span(tx, tz);
                let mut at = Vec3::new(c.x, y, c.y);
                if y > 1.0 {
                    at.y = 1.02;
                }
                r.decal(
                    a.tex(tex),
                    UvRect::new(0.0, 0.0, 16.0, 16.0),
                    at,
                    half + Vec2::splat(pulse),
                    &DrawOpts {
                        mode: Mode::Unlit,
                        zwrite: false,
                        ..Default::default()
                    },
                );
                if self.area == Area::Home && self.target_ok {
                    self.draw_ghost(r, a, tx, tz);
                }
            }
        }

        self.draw_drops(r, a);

        for f in self.foes.iter().filter(|f| !f.translucent()) {
            f.draw(r, a);
        }
        for s in &self.shots {
            let c = shot_colors(s.color);
            r.halo(s.world_pos(), 0.28, c[1], 0.7);
            r.point(s.world_pos(), 3, c[1]);
            r.point(s.world_pos(), 1, c[0]);
            let tail = s.world_pos() - Vec3::new(s.vel.x, 0.0, s.vel.y) * 0.05;
            r.point(tail, 2, c[2]);
        }
        for b in &self.bolts {
            let p = b.world_pos();
            r.halo(p, 0.32, b.colors[1], 0.85);
            let back = Vec3::new(b.vel.x, 0.0, b.vel.y).normalize_or_zero();
            r.point(p - back * 0.22, 1, b.colors[2]);
            r.point(p - back * 0.12, 2, b.colors[2]);
            r.point(p, 4, b.colors[1]);
            r.point(p, 2, b.colors[0]);
            let twinkle = (self.time * 30.0).sin() > 0.0;
            if twinkle {
                r.point(p + Vec3::Y * 0.12, 1, WHITE);
            }
        }

        // The cat.
        if self.area == Area::Farm {
            let c = &self.cat;
            let base = Vec3::new(c.pos.x, 0.0, c.pos.y);
            r.shadow(a.tex(a.disk), base, 0.22);
            let hop = if c.pet > 0.0 {
                (self.time * 10.0).sin().abs() * 0.05
            } else {
                0.0
            };
            let m = Mat4::from_translation(base + Vec3::Y * hop) * Mat4::from_rotation_y(c.yaw);
            let o = DrawOpts::at(base).with_tag(1);
            r.mesh(&a.bank, &a.critters.cat, &m, &o);
            let wag = (self.time * 5.0).sin() * 0.5;
            let tail = m
                * Mat4::from_translation(Vec3::new(0.0, 0.2, -0.2))
                * Mat4::from_rotation_x(0.8)
                * Mat4::from_rotation_y(wag);
            r.mesh(&a.bank, &a.critters.cat_tail, &tail, &o);
        }

        self.draw_folk(r, a);
        self.draw_player(r, a);
        self.draw_fishing(r, a);
        self.draw_cooking(r);
        // Jelly and ghosts go last, far to near, so whatever is behind them shows through.
        let mut clear: Vec<_> = self.foes.iter().filter(|f| f.translucent()).collect();
        clear.sort_by(|p, q| p.pos.y.total_cmp(&q.pos.y));
        for f in clear {
            f.draw(r, a);
        }
        self.draw_ward(r, a);
        self.draw_spells(r);
        self.fx.draw(r);
        for f in &fireflies {
            let on = ((self.time * 3.0 + f.x).sin() * 0.5 + 0.5) > 0.3;
            if on {
                r.point(*f, 1, CREAM);
            }
        }
        // Rain.
        if self.rain && matches!(self.area, Area::Farm | Area::Town) {
            let t = self.cam.target;
            for i in 0..90 {
                let fx = (hash2(i, 0, 1) % 1000) as f32 / 1000.0;
                let fz = (hash2(i, 1, 1) % 1000) as f32 / 1000.0;
                let fall = (self.time * 1.8 + fx * 7.0).fract();
                let p = Vec3::new(
                    t.x - 11.0 + fx * 22.0,
                    3.5 - fall * 3.5,
                    t.z - 7.0 + fz * 13.0,
                );
                r.point(p, 1, SKY);
                r.point(p + Vec3::Y * 0.12, 1, BLUE);
            }
        }
        r.fb.outline(INK);
    }

    /// The townsfolk, dressed up, with a bubble over anyone who wants you.
    fn draw_folk(&self, r: &mut Renderer, a: &Assets) {
        for (k, n) in self.folk.iter().enumerate() {
            let v = n.who;
            let d = v.def();
            let h = &a.folk[v as usize];
            let base = n.world_pos();
            let (dressed, remap) = villager_dress(a, v);
            let fit = dressed.outfit_with(&remap);
            let pose = Pose {
                walk: n.walk,
                stride: n.stride,
                bob: if n.stride < 0.1 {
                    (self.time * 2.2 + k as f32).sin().abs() * 0.015
                } else {
                    0.0
                },
                ..Default::default()
            };
            r.shadow(a.tex(a.disk), base, 0.28 * d.look.scale);
            let o = DrawOpts {
                light: Light::At(base),
                tag: 1,
                ..Default::default()
            };
            draw_humanoid(r, a, h, base, n.yaw, &pose, &o, &fit);
            // Quest bubbles.
            if let Some(done) = self.marker(v) {
                let icon = if done { "bubble_done" } else { "bubble_new" };
                let id = a.icon(icon);
                let bob = (self.time * 3.0 + k as f32).sin() * 0.05;
                let top = base + Vec3::Y * (1.35 * d.look.scale + 0.2 + bob);
                r.billboard(
                    a.tex(id),
                    full_uv(a, id),
                    top,
                    Vec2::splat(0.42),
                    &DrawOpts {
                        mode: Mode::Unlit,
                        tag: 3,
                        ..Default::default()
                    },
                );
            } else if n.talking {
                let id = a.icon("bubble_talk");
                let top = base + Vec3::Y * (1.35 * d.look.scale + 0.2);
                r.billboard(
                    a.tex(id),
                    full_uv(a, id),
                    top,
                    Vec2::splat(0.38),
                    &DrawOpts {
                        mode: Mode::Unlit,
                        tag: 3,
                        ..Default::default()
                    },
                );
            }
        }
    }

    /// The Ward spell: a soap-bubble of light around the hero, thinning as it wears out.
    fn draw_ward(&self, r: &mut Renderer, a: &Assets) {
        let p = &self.player;
        if p.ward_t <= 0.0 {
            return;
        }
        let fading = p.ward_t < 1.5 && (self.time * 14.0).sin() > 0.0;
        let wobble = 1.0 + (self.time * 5.0).sin() * 0.03;
        let m = Mat4::from_translation(p.world_pos())
            * Mat4::from_rotation_y(self.time * 0.6)
            * Mat4::from_scale(Vec3::new(wobble, 1.0 / wobble, wobble));
        let o = DrawOpts {
            light: Light::Fixed(1.1, 2.5),
            zwrite: false,
            tag: 0,
            ..DrawOpts::default()
        };
        let alpha = if fading { 0.02 } else { 0.08 };
        r.mesh(&a.bank, &a.props.bubble, &m, &o.glass(alpha));
        // A shimmer of added light on the rim.
        let glow = DrawOpts {
            mode: Mode::Glow,
            alpha: 0.05,
            ..o
        };
        r.mesh(&a.bank, &a.props.bubble, &m, &glow);
    }

    /// How dark it is outside (rooms still know whether it's night).
    pub fn sky_night(&self) -> f32 {
        ((self.clock.min - 1150.0) / 110.0).clamp(0.0, 1.0)
    }

    /// Rugs, windows, pictures and lamps in the room you're in.
    fn draw_decor(&self, r: &mut Renderer, a: &Assets) {
        let Some(room) = &self.room else { return };
        let night = self.sky_night() > 0.3;
        let t = &a.town;
        // The doormat on the way out.
        let (ex, ez) = room.exit;
        let mat = t.rugs[(room.place as usize + 3) % t.rugs.len()];
        r.decal(
            a.tex(mat),
            UvRect::new(0.0, 0.0, 16.0, 16.0),
            Vec3::new(ex as f32 + 0.5, 0.01, ez as f32 + 0.35),
            Vec2::new(0.42, 0.3),
            &DrawOpts {
                zwrite: false,
                ..Default::default()
            },
        );
        for d in &room.decor {
            match *d {
                Decor::Rug { x0, z0, x1, z1 } => {
                    let tex = t.rugs[room.place as usize % t.rugs.len()];
                    let c = Vec3::new((x0 + x1 + 1) as f32 * 0.5, 0.01, (z0 + z1 + 1) as f32 * 0.5);
                    let half = Vec2::new(
                        (x1 - x0 + 1) as f32 * 0.5 - 0.05,
                        (z1 - z0 + 1) as f32 * 0.5 - 0.05,
                    );
                    r.decal(
                        a.tex(tex),
                        UvRect::new(0.0, 0.0, 16.0, 16.0),
                        c,
                        half,
                        &DrawOpts {
                            zwrite: false,
                            ..Default::default()
                        },
                    );
                }
                Decor::Window { x } => {
                    let m = Mat4::from_translation(Vec3::new(x as f32 + 0.5, 0.0, 1.01));
                    r.mesh(&a.bank, &t.window, &m, &DrawOpts::default());
                    let o = if night {
                        DrawOpts::default()
                    } else {
                        DrawOpts::default().with_mode(Mode::Unlit)
                    };
                    r.mesh(&a.bank, &t.window_glass, &m, &o);
                }
                Decor::Painting { x } => {
                    let m = Mat4::from_translation(Vec3::new(x as f32 + 0.5, 0.0, 1.01));
                    let p = &t.paintings[(x as usize + room.place as usize) % t.paintings.len()];
                    r.mesh(&a.bank, p, &m, &DrawOpts::default());
                }
                Decor::Sconce { x } => {
                    let m = Mat4::from_translation(Vec3::new(x as f32 + 0.5, 0.0, 1.01));
                    r.mesh(&a.bank, &t.sconce, &m, &DrawOpts::default());
                    flames(
                        r,
                        a,
                        Vec3::new(x as f32 + 0.5, 1.44, 1.1),
                        0.22,
                        self.time,
                        x,
                    );
                }
            }
        }
    }

    /// Loot on the ground bobs and spins; coins pile up; rare things shine.
    fn draw_drops(&self, r: &mut Renderer, a: &Assets) {
        for (k, d) in self.drops.iter().enumerate() {
            let bob = (self.time * 4.0 + d.age).sin() * 0.04;
            let base = d.pos + Vec3::Y * bob;
            let ground = Vec3::new(d.pos.x, 0.0, d.pos.z);
            let id = a.icon(d.stack.item.def().icon);
            if d.is_coin() {
                r.shadow(a.tex(a.disk), ground, 0.1);
                let n = (d.stack.n as usize).clamp(1, 3);
                for c in 0..n {
                    let off = Vec3::new(
                        (c as f32 - (n - 1) as f32 * 0.5) * 0.09,
                        c as f32 * 0.03,
                        c as f32 * 0.01,
                    );
                    let at = base + off;
                    r.billboard(
                        a.tex(id),
                        full_uv(a, id),
                        at - Vec3::Y * 0.05,
                        Vec2::splat(0.26),
                        &DrawOpts::at(at).with_tag(3).with_glow(0.9),
                    );
                }
                let shine = ((self.time * 5.0 + k as f32 * 1.3).sin()) > 0.85;
                if shine {
                    r.point(base + Vec3::new(0.06, 0.2, 0.0), 1, WHITE);
                }
                continue;
            }
            r.shadow(a.tex(a.disk), ground, 0.14);
            r.billboard(
                a.tex(id),
                full_uv(a, id),
                base - Vec3::Y * 0.05,
                Vec2::splat(0.5),
                &DrawOpts::at(base).with_tag(3).with_glow(0.8),
            );
            let rarity = match d.stack.rarity() {
                Some(r) => r,
                // Keepsakes shine like treasure.
                None if d.stack.item.def().kind == Kind::Keepsake => Rarity::Legendary,
                None => continue,
            };
            if rarity == Rarity::Common {
                continue;
            }
            let col = rarity.color();
            // Sparkles circling it.
            let spin = self.time * 2.5 + k as f32;
            for i in 0..(rarity as usize).min(3) {
                let ang = spin + i as f32 * 2.1;
                let q = base
                    + Vec3::new(
                        ang.cos() * 0.28,
                        0.2 + (spin * 1.7 + i as f32).sin() * 0.1,
                        ang.sin() * 0.28,
                    );
                r.point(q, 1, if i % 2 == 0 { col } else { WHITE });
            }
            // A shimmering beam of light for rare finds, so they are easy to spot: sparks
            // drift up a column in the rarity's colour and fade out towards the top.
            if rarity >= Rarity::Rare && d.age > 0.3 {
                let height = match rarity {
                    Rarity::Rare => 1.2,
                    Rarity::Epic => 1.7,
                    _ => 2.3,
                };
                let sparks = 7 + rarity as usize * 2;
                for i in 0..sparks {
                    let phase =
                        (self.time * 0.55 + i as f32 / sparks as f32 + k as f32 * 0.37).fract();
                    let y = 0.3 + phase * height;
                    let wob = (phase * 17.0 + i as f32).sin() * 0.04;
                    let fade = phase < 0.85 || (self.time * 20.0 + i as f32).sin() > 0.0;
                    if fade {
                        let c = if i % 3 == 0 { WHITE } else { col };
                        r.point(ground + Vec3::new(wob, y, 0.0), 1, c);
                    }
                }
                if (self.time * 4.0 + k as f32).sin() > 0.2 {
                    r.point(base + Vec3::Y * 0.32, 1, WHITE);
                }
            }
        }
    }

    fn draw_player(&self, r: &mut Renderer, a: &Assets) {
        let p = &self.player;
        let mut pos = p.world_pos();
        let mut pose = Pose {
            walk: p.walk,
            stride: p.stride,
            ..Default::default()
        };
        if p.dodge > 0.0 {
            pos.y += (p.dodge / 0.26 * std::f32::consts::PI).sin() * 0.25;
            pose.squash = 0.5;
        }
        if let Some(act) = &p.act {
            pose.swing = Some((act.progress(), act.kind.swing()));
        } else if p.eat_t > 0.0 {
            pose.swing = Some((1.0 - p.eat_t / 0.6, Swing::Use));
        } else if let Some(f) = &self.fishing {
            pose.swing = Some((f.lift(self.time), Swing::Fish));
        } else if self.cooking.is_some() {
            // Stirring the pot.
            pose.swing = Some(((self.time * 1.6).fract() * 0.5, Swing::Use));
        }
        let dressed = dress(a, &p.equip, p.held_stack());
        let fit = dressed.outfit(Some(&a.sprout));
        r.shadow(a.tex(a.disk), p.world_pos(), 0.3);
        let blink = p.hurt > 0.0 && (self.time * 20.0).sin() > 0.3;
        let mut o = DrawOpts {
            light: Light::At(p.world_pos()),
            tag: 1,
            ..Default::default()
        };
        if p.flash > 0.0 {
            o.mode = Mode::Solid(WHITE);
        }
        if !blink {
            draw_humanoid(r, a, &a.hero, pos, p.yaw, &pose, &o, &fit);
            // A soft silhouette where walls hide the hero.
            let ghost = DrawOpts {
                mode: Mode::Hidden(LAVENDER),
                zwrite: false,
                tag: 1,
                ..o
            };
            draw_humanoid(r, a, &a.hero, pos, p.yaw, &pose, &ghost, &fit);
        }
        if let Some(act) = &p.act {
            match act.kind {
                ActKind::Slash | ActKind::Reap => {
                    let col = match p.held_stack().and_then(|s| s.rarity()) {
                        Some(r) if r > Rarity::Common => r.color(),
                        _ => KHAKI,
                    };
                    let yaw = act.dir.x.atan2(act.dir.y);
                    let reach = if act.kind == ActKind::Reap { 1.1 } else { 1.25 };
                    draw::slash_arc(r, p.world_pos(), yaw, act.progress(), reach, col);
                }
                ActKind::Cast => {
                    // Starlight gathering in the hand.
                    let cols = self
                        .casting
                        .map_or([WHITE, LAVENDER, BLUSH], |s| s.def().colors);
                    let t = self.time * 10.0;
                    let hand = p.world_pos() + Vec3::new(act.dir.x * 0.45, 0.85, act.dir.y * 0.45);
                    r.halo(hand, 0.3, cols[1], 0.8);
                    for i in 0..5 {
                        let ang = t + i as f32 * 1.26;
                        let q = hand
                            + Vec3::new(
                                ang.cos() * 0.22,
                                (ang * 1.3).sin() * 0.12,
                                ang.sin() * 0.22,
                            );
                        r.point(q, 1, cols[i % 3]);
                    }
                }
                ActKind::Blast if act.progress() < 0.55 => {
                    // Gathering sparkles above the staff.
                    let t = self.time * 8.0;
                    for i in 0..4 {
                        let ang = t + i as f32 * 1.57;
                        let q = p.world_pos()
                            + Vec3::new(ang.cos() * 0.3, 1.2 + act.progress(), ang.sin() * 0.3);
                        r.point(q, 1, [WHITE, MINT, LAVENDER, BLUSH][i]);
                    }
                }
                _ => {}
            }
        }
    }

    /// Where the cursor sits and how big it is: one tile, or everything a piece covers.
    fn cursor_span(&self, tx: i32, tz: i32) -> (Vec2, Vec2) {
        let one = (
            Vec2::new(tx as f32 + 0.5, tz as f32 + 0.5),
            Vec2::splat(0.46),
        );
        if self.area != Area::Home {
            return one;
        }
        let span = |x: f32, z: f32, w: f32, d: f32| {
            (
                Vec2::new(x + w * 0.5, z + d * 0.5),
                Vec2::new(w * 0.5 - 0.04, d * 0.5 - 0.04),
            )
        };
        match self.player.held().map(|i| i.def().kind) {
            Some(Kind::Place(Placeable::Furniture(f))) => {
                let (rot, ax, az) = self.placement(f, tx, tz);
                let (w, d) = home::turned(f, rot);
                span(ax as f32, az as f32, w as f32, d as f32)
            }
            Some(Kind::Place(Placeable::Rug(_))) => {
                let (rx, rz) = self.rug_anchor(tx, tz);
                span(rx as f32, rz as f32, 2.0, 2.0)
            }
            Some(Kind::Place(_)) => one,
            _ => {
                let w = &self.house.world;
                let (ax, az) = w.anchor(tx, tz);
                if let Some(super::world::Obj::Furniture { f, rot, .. }) = w.obj(ax, az) {
                    let (fw, fd) = home::turned(*f, *rot);
                    return span(ax as f32, az as f32, fw as f32, fd as f32);
                }
                if w.obj(ax, az).is_none() {
                    if let Some(rug) = self.house.rugs.iter().find(|r| r.covers(tx, tz)) {
                        return span(rug.x as f32, rug.z as f32, 2.0, 2.0);
                    }
                }
                one
            }
        }
    }

    /// A see-through preview of the piece in hand, where it would go.
    fn draw_ghost(&self, r: &mut Renderer, a: &Assets, tx: i32, tz: i32) {
        let Some(item) = self.player.held() else {
            return;
        };
        let Kind::Place(pl) = item.def().kind else {
            return;
        };
        let bob = (self.time * 3.0).sin() * 0.02 + 0.04;
        match pl {
            Placeable::Furniture(f) => {
                let (rot, ax, az) = self.placement(f, tx, tz);
                let m = Mat4::from_translation(home::center(f, rot, ax, az) + Vec3::Y * bob)
                    * Mat4::from_rotation_y(rot as f32 * std::f32::consts::FRAC_PI_2);
                r.mesh(
                    &a.bank,
                    &a.home.furn[f as usize],
                    &m,
                    &DrawOpts::default().glass(0.4).with_glow(1.0),
                );
            }
            Placeable::Rug(k) => {
                let tex = a.home.rugs[k as usize % a.home.rugs.len()];
                let (rx, rz) = self.rug_anchor(tx, tz);
                r.decal(
                    a.tex(tex),
                    UvRect::new(0.0, 0.0, 16.0, 16.0),
                    Vec3::new(rx as f32 + 1.0, 0.02, rz as f32 + 1.0),
                    Vec2::splat(0.95),
                    &DrawOpts {
                        zwrite: false,
                        ..DrawOpts::default().glass(0.55)
                    },
                );
            }
            Placeable::WallArt(k) => {
                if let Some(col) = self.art_spot(tx, tz) {
                    let m = Mat4::from_translation(Vec3::new(col as f32 + 0.5, 0.0, 1.02));
                    r.mesh(
                        &a.bank,
                        &a.home.art[k as usize % a.home.art.len()],
                        &m,
                        &DrawOpts::default().glass(0.55).with_glow(1.0),
                    );
                }
            }
            _ => {}
        }
    }

    /// Windows, pictures and rugs in the farmhouse.
    fn draw_home(&self, r: &mut Renderer, a: &Assets) {
        let night = self.sky_night() > 0.3;
        let t = &a.town;
        let flat = DrawOpts {
            zwrite: false,
            ..Default::default()
        };
        // The doormat.
        let (dx, dz) = home::HOUSE_DOOR;
        r.decal(
            a.tex(t.rugs[2 % t.rugs.len()]),
            UvRect::new(0.0, 0.0, 16.0, 16.0),
            Vec3::new(dx as f32 + 0.5, 0.01, dz as f32 - 0.65),
            Vec2::new(0.42, 0.3),
            &flat,
        );
        for rug in &self.house.rugs {
            let tex = a.home.rugs[rug.kind as usize % a.home.rugs.len()];
            r.decal(
                a.tex(tex),
                UvRect::new(0.0, 0.0, 16.0, 16.0),
                Vec3::new(rug.x as f32 + 1.0, 0.012, rug.z as f32 + 1.0),
                Vec2::splat(0.96),
                &flat,
            );
        }
        for x in WINDOWS {
            let m = Mat4::from_translation(Vec3::new(x as f32 + 0.5, 0.0, 1.01));
            r.mesh(&a.bank, &t.window, &m, &DrawOpts::default());
            let o = if night {
                DrawOpts::default()
            } else {
                DrawOpts::default().with_mode(Mode::Unlit)
            };
            r.mesh(&a.bank, &t.window_glass, &m, &o);
        }
        for (col, kind) in &self.house.art {
            let m = Mat4::from_translation(Vec3::new(*col as f32 + 0.5, 0.0, 1.01));
            r.mesh(
                &a.bank,
                &a.home.art[*kind as usize % a.home.art.len()],
                &m,
                &DrawOpts::at(Vec3::new(*col as f32 + 0.5, 1.2, 1.5)),
            );
        }
    }

    /// Steam curling up off the pot, and the burner glowing under it.
    fn draw_cooking(&self, r: &mut Renderer) {
        let Some(c) = &self.cooking else { return };
        let pot = c.at + Vec3::Y * 0.9;
        r.halo(c.at + Vec3::Y * 0.7, 0.35, ORANGE, 0.5);
        for i in 0..10 {
            let k = (self.time * 0.9 + i as f32 * 0.1).fract();
            let sway = (k * 7.0 + i as f32 * 1.7).sin() * 0.12 * k;
            let q = pot + Vec3::new(sway + (i % 3) as f32 * 0.05 - 0.05, k * 0.9, 0.0);
            if k < 0.85 || (self.time * 20.0 + i as f32).sin() > 0.0 {
                r.point(
                    q,
                    if k < 0.4 { 2 } else { 1 },
                    if i % 3 == 0 { CREAM } else { WHITE },
                );
            }
        }
        // Bubbles popping in the pan.
        if (self.time * 9.0).sin() > 0.3 {
            r.point(pot + Vec3::new(0.06, -0.02, 0.05), 1, GOLD);
        }
    }

    /// The line, the bobber, rings on the water, and the catch dangling from the rod.
    fn draw_fishing(&self, r: &mut Renderer, a: &Assets) {
        let Some(f) = &self.fishing else { return };
        let p = &self.player;
        let tip = fish::rod_tip(p.world_pos(), p.yaw, f.lift(self.time));
        let col = fish::line_color(p.held_stack());
        let line = |r: &mut Renderer, from: Vec3, to: Vec3, sag: f32| {
            let n = ((from.distance(to) * 40.0) as usize).clamp(6, 260);
            for q in fish::line_points(from, to, sag, n) {
                r.point(q, 1, col);
            }
        };
        match f.phase {
            Phase::Charge => {
                // The bobber dangles from the tip, swinging as you wind up.
                let sway = (self.time * 7.0).sin() * 0.06 * (0.3 + f.power);
                let bob = tip + Vec3::new(sway, -0.26, 0.0);
                line(r, tip, bob, 0.0);
                draw_bobber(r, bob);
            }
            Phase::Caught => {
                let Some(Hooked::Fish(item, _)) = f.hooked else {
                    return;
                };
                // Held up high on a short line, flapping.
                let flap = (self.time * 14.0).sin();
                let at = tip - Vec3::Y * 0.62;
                line(r, tip, at + Vec3::Y * 0.4, 0.0);
                let id = a.icon(item.def().icon);
                let uv = if flap > 0.0 {
                    full_uv(a, id)
                } else {
                    UvRect::new(15.96, 0.0, 0.04, 16.0)
                };
                r.billboard(
                    a.tex(id),
                    uv,
                    at,
                    Vec2::splat(0.5),
                    &DrawOpts::at(at).with_glow(0.9).with_tag(3),
                );
                let rare = fish::fish_def(item).map_or(0, |d| d.rarity);
                if rare > 0 {
                    let c = fish::rarity_color(rare);
                    for i in 0..(2 + rare as usize * 2) {
                        let ang = self.time * 3.0 + i as f32 * 1.3;
                        let q = at
                            + Vec3::new(
                                ang.cos() * 0.34,
                                0.25 + (ang * 1.7).sin() * 0.2,
                                ang.sin() * 0.2,
                            );
                        r.point(q, 1, if i % 2 == 0 { c } else { WHITE });
                    }
                }
                // Drips falling off it.
                for i in 0..3 {
                    let k = (self.time * 1.4 + i as f32 * 0.33).fract();
                    let q = at + Vec3::new((i as f32 - 1.0) * 0.1, 0.1 - k * 1.1, 0.0);
                    if q.y > 0.05 {
                        r.point(q, 1, SKY);
                    }
                }
            }
            _ => {
                let bob = f.bobber(self.time);
                let sag = match f.phase {
                    Phase::Wait => 0.2,
                    Phase::Bite => 0.06,
                    Phase::Fly => 0.04,
                    _ => 0.01,
                };
                line(r, tip, bob, sag);
                draw_bobber(r, bob);
                if !matches!(f.phase, Phase::Wait | Phase::Bite | Phase::Reel) {
                    return;
                }
                let lava = f.water == Some(Water::Lava);
                let rip = if lava { [GOLD, ORANGE] } else { [WHITE, SKY] };
                // Rings spreading out on the water, quicker when something's on the line.
                let speed = if f.phase == Phase::Wait { 0.6 } else { 2.0 };
                let y = f.to.y - 0.02;
                for ring in 0..2 {
                    let k = (self.time * speed + ring as f32 * 0.5).fract();
                    let rad = 0.08 + k * 0.42;
                    for i in 0..14 {
                        if k > 0.65 && (i + ring) % 2 == 1 {
                            continue;
                        }
                        let ang = i as f32 / 14.0 * std::f32::consts::TAU;
                        let q = Vec3::new(
                            f.bob.x + ang.cos() * rad,
                            y,
                            f.bob.z + ang.sin() * rad * 0.8,
                        );
                        r.point(q, 1, rip[(i + ring) % 2]);
                    }
                }
                if f.phase != Phase::Wait {
                    // Thrashing: droplets leaping up round the bobber.
                    for i in 0..6 {
                        let k = (self.time * 2.6 + i as f32 * 0.17).fract();
                        let ang = i as f32 * 1.05 + (self.time * 0.7).floor();
                        let q = Vec3::new(
                            f.bob.x + ang.cos() * (0.1 + k * 0.25),
                            y + (k * std::f32::consts::PI).sin() * 0.3,
                            f.bob.z + ang.sin() * (0.1 + k * 0.2),
                        );
                        r.point(q, 1, rip[i % 2]);
                    }
                }
            }
        }
    }
}

/// A little red and white float.
fn draw_bobber(r: &mut Renderer, at: Vec3) {
    r.point(at + Vec3::Y * 0.03, 2, RED);
    r.point(at + Vec3::Y * 0.08, 2, WHITE);
    r.point(at + Vec3::Y * 0.12, 1, RED);
}
