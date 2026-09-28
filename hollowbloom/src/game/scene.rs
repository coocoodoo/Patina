//! Drawing the playing state: the world, loot on the ground, magic, creatures and the
//! hero in whatever they are wearing.

use glam::{Mat4, Vec2, Vec3};

use super::draw::{self, Outfit, Pose, Swing, draw_humanoid, full_uv};
use super::fx::shot_colors;
use super::gear::{Rarity, Slot};
use super::items::{Kind, Stack};
use super::play::Play;
use super::player::ActKind;
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
            Area::Farm if env.night > 0.2 => lights.push(PointLight {
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
        let world = match &mut self.level {
            Some(l) if matches!(self.area, Area::Hollow { .. }) => &mut l.world,
            _ => &mut self.farm,
        };
        draw::draw_world(r, a, world, &env, &lights);

        // Target cursor.
        if let (Some((tx, tz)), Some(item)) = (self.target, self.player.held()) {
            let shows = match item.def().kind {
                Kind::Gear(b) => super::player::ActKind::for_class(b.class)
                    .is_some_and(|k| !matches!(k, ActKind::Slash | ActKind::Bolt | ActKind::Blast)),
                Kind::Seed(_) | Kind::Place(_) => true,
                _ => false,
            };
            if shows && self.fade.is_none() {
                let tex = if self.target_ok {
                    a.cursor
                } else {
                    a.cursor_bad
                };
                let y = if world.wall(tx, tz) != Wall::None {
                    1.02
                } else {
                    0.03
                };
                let pulse = 0.46 + (self.time * 6.0).sin() * 0.03;
                r.decal(
                    a.tex(tex),
                    UvRect::new(0.0, 0.0, 16.0, 16.0),
                    Vec3::new(tx as f32 + 0.5, y, tz as f32 + 0.5),
                    Vec2::splat(pulse),
                    &DrawOpts {
                        mode: Mode::Unlit,
                        zwrite: false,
                        ..Default::default()
                    },
                );
            }
        }

        self.draw_drops(r, a);

        for f in &self.foes {
            f.draw(r, a);
        }
        for s in &self.shots {
            let c = shot_colors(s.color);
            r.point(s.world_pos(), 3, c[1]);
            r.point(s.world_pos(), 1, c[0]);
            let tail = s.world_pos() - Vec3::new(s.vel.x, 0.0, s.vel.y) * 0.05;
            r.point(tail, 2, c[2]);
        }
        for b in &self.bolts {
            let p = b.world_pos();
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

        self.draw_player(r, a);
        self.fx.draw(r);
        for f in &fireflies {
            let on = ((self.time * 3.0 + f.x).sin() * 0.5 + 0.5) > 0.3;
            if on {
                r.point(*f, 1, CREAM);
            }
        }
        // Rain.
        if self.rain && self.area == Area::Farm {
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
            let Some(rarity) = d.stack.rarity() else {
                continue;
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
}
