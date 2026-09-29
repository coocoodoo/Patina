//! Hollow creatures: stats, behaviour and drawing.

use glam::{Mat4, Vec2, Vec3};

use std::f32::consts::PI;

use super::draw::{Outfit, Pose, Swing, draw_humanoid};
use super::dungeon::Foe;
use super::fx::{BREATH_LIFE, Fx, Shot, ShotKind};
use super::world::World;
use crate::assets::{Assets, BIOMES, LOOKS, SEWER_LOOK};
use crate::palette::*;
use crate::render::{DrawOpts, Light, Mode, PointLight, Renderer, Warp};
use crate::util::{Rng, approach};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum St {
    Idle,
    Chase,
    Windup,
    Dash,
    Rest,
    Hop,
    /// Head thrown back, howling (a werewolf; see `Call::Howl`).
    Howl,
}

/// Something a creature has done that reaches past itself, for the floor to answer (see
/// `Play::update_foes`).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Call {
    /// A werewolf's howl: everything in earshot comes running, riled up.
    Howl,
    /// A drakeling breathing out fire or frost (for the roar of it).
    Breath,
    /// A leafling mending whoever's most hurt round it.
    Mend,
}

/// How long a werewolf howls, and how far the howl carries.
pub const HOWL_SECS: f32 = 1.1;
pub const HOWL_REACH: f32 = 9.0;
/// How long a drakeling rears back drawing breath, and how long it breathes for.
pub const BREATH_WINDUP: f32 = 0.6;
pub const BREATH_SECS: f32 = 0.45;
/// How often a leafling mends someone, how far it reaches, and how often it looks again
/// when nobody needs it.
pub const MEND_SECS: f32 = 5.0;
pub const MEND_REACH: f32 = 5.0;
pub const MEND_AGAIN: f32 = 1.0;

pub struct Enemy {
    pub foe: Foe,
    pub boss: bool,
    pub biome: usize,
    pub pos: Vec2,
    pub vel: Vec2,
    pub y: f32,
    pub vy: f32,
    pub hp: i32,
    pub max_hp: i32,
    pub dmg: i32,
    pub speed: f32,
    pub radius: f32,
    pub xp: u32,
    pub st: St,
    pub t: f32,
    pub dir: Vec2,
    pub yaw: f32,
    pub flash: f32,
    pub hurt_cd: f32,
    pub anim: f32,
    pub alert: bool,
    pub hops: u32,
    pub squash: f32,
    pub seed: u32,
    /// Seconds of burning left, and how much each tick hurts.
    pub burn: f32,
    pub burn_dmg: i32,
    pub burn_tick: f32,
    /// Seconds of chill left (chilled foes move at half speed).
    pub chill: f32,
    /// How riled up the moon has them: how far they notice you, how fast they move and how
    /// quickly they strike (1 = a half moon).
    pub fury: f32,
    /// Under a full moon: glowing red, tougher, and carrying better loot.
    pub moonlit: bool,
    /// Seconds its name stays shown over its health bar after you hit it.
    pub named: f32,
    /// Seconds until a guardian next calls for help.
    pub summon: f32,
    /// Seconds a lantern snail has left hiding in its shell.
    pub hide: f32,
    /// A lantern snail's glowing slime: where it has been, and how long ago.
    pub trail: Vec<(Vec2, f32)>,
    /// Living in the old sewers, where everyone is down to their bones (and the slimes are
    /// sludge): the look and the name that go with it.
    pub sewer: bool,
    /// Living in the glowcap caves, where the shroomlings glow like the caps.
    pub glowcave: bool,
    /// What it has just done that the floor has to answer (taken each frame).
    pub call: Option<Call>,
}

/// How long a lantern snail hides in its shell once struck, and how much of a blow the
/// shell lets through meanwhile.
pub const HIDE_SECS: f32 = 1.4;
pub const SHELL_LETS_THROUGH: f32 = 0.35;
/// How long a snail's glowing slime trail lasts.
pub const TRAIL_SECS: f32 = 5.0;

/// The outline tag for creatures maddened by the full moon (outlined in red).
pub const MOONLIT_TAG: u8 = 6;

/// Goblin colours by biome (light, mid, dark): their skin, and the dust their clubs raise.
/// (In the sewers, bone.)
const GOBLIN_DUST: [[u8; 3]; LOOKS] = [
    [LIME, GREEN, TEAL],
    [MINT, AQUA, TEAL],
    [BLUSH, PINK, PLUM],
    [ORANGE, RED, MAROON],
    [WHITE, SKY, BLUE],
    [GOLD, CLAY, RUST],
    [WHITE, SAND, KHAKI],
];

/// The glint of a thrown goblin dagger, by biome (a rusty one, in the sewers).
const DAGGERS: [u8; LOOKS] = [WHITE, AQUA, BLUSH, ORANGE, SKY, GOLD, RUST];

pub struct Base {
    pub hp: i32,
    pub dmg: i32,
    pub speed: f32,
    pub radius: f32,
    pub xp: u32,
}

pub fn base(f: Foe) -> Base {
    let b = |hp, dmg, speed, radius, xp, _name: &str| Base {
        hp,
        dmg,
        speed,
        radius,
        xp,
    };
    match f {
        Foe::Slime => b(14, 6, 2.4, 0.34, 4, "Slime"),
        Foe::Bat => b(9, 5, 3.3, 0.28, 4, "Bat"),
        Foe::Shroom => b(18, 7, 1.7, 0.3, 5, "Shroomling"),
        Foe::Crab => b(22, 9, 1.9, 0.34, 7, "Crystal Crab"),
        Foe::Wisp => b(12, 7, 1.6, 0.26, 7, "Wisp"),
        Foe::Beetle => b(26, 9, 1.9, 0.32, 8, "Beetle"),
        Foe::Imp => b(20, 10, 2.4, 0.3, 9, "Imp"),
        Foe::Skeleton => b(30, 11, 2.2, 0.3, 10, "Skeleton"),
        Foe::Golem => b(60, 16, 1.3, 0.5, 16, "Golem"),
        Foe::Ghost => b(18, 10, 1.8, 0.3, 9, "Ghost"),
        Foe::Frog => b(16, 7, 2.6, 0.3, 6, "Bog Frog"),
        Foe::Jelly => b(14, 8, 1.2, 0.3, 7, "Drift Jelly"),
        Foe::Puffer => b(20, 9, 1.4, 0.32, 8, "Puffer"),
        Foe::Zombie => b(30, 9, 1.25, 0.3, 8, "Zombie"),
        Foe::Brute => b(44, 13, 1.5, 0.4, 12, "Goblin Brute"),
        Foe::Sneak => b(15, 7, 3.0, 0.26, 7, "Goblin Sneak"),
        Foe::Bug => b(11, 6, 3.0, 0.26, 5, "Bug"),
        Foe::Snail => b(34, 9, 0.8, 0.3, 11, "Lantern Snail"),
        Foe::Bookworm => b(22, 8, 1.7, 0.28, 10, "Bibliomancer"),
        Foe::Drake => b(38, 11, 2.0, 0.34, 14, "Drakeling"),
        Foe::Leafling => b(16, 6, 2.7, 0.26, 8, "Leafling"),
        Foe::Werewolf => b(52, 14, 3.0, 0.34, 18, "Werewolf"),
    }
}

/// The Fungal Hollow's bugs are spore moths: they fly.
pub const MOTHS: usize = 2;

/// Creatures that float or fly rather than walk.
pub fn flies(foe: Foe, biome: usize) -> bool {
    matches!(
        foe,
        Foe::Bat | Foe::Wisp | Foe::Ghost | Foe::Jelly | Foe::Puffer | Foe::Leafling
    ) || (foe == Foe::Bug && biome % 6 == MOTHS)
}

/// What a creature is called where it lives: every family has its own kind in each biome.
pub fn kind_name(f: Foe, biome: usize) -> &'static str {
    let b = biome % 6;
    match f {
        Foe::Zombie => [
            "Mossy Zombie",
            "Crystal Zombie",
            "Sporeling Zombie",
            "Charred Zombie",
            "Frozen Zombie",
            "Mummy",
        ][b],
        Foe::Brute => [
            "Mossback Brute",
            "Geode Brute",
            "Toadstool Brute",
            "Cinder Brute",
            "Snowgut Brute",
            "Tomb Brute",
        ][b],
        Foe::Sneak => [
            "Moss Sneak",
            "Glint Sneak",
            "Spore Sneak",
            "Ash Sneak",
            "Frost Sneak",
            "Dune Sneak",
        ][b],
        Foe::Bug => [
            "Moss Spider",
            "Glass Mantis",
            "Spore Moth",
            "Fire Ant",
            "Frost Spider",
            "Scarab",
        ][b],
        Foe::Skeleton => [
            "Mossbones",
            "Crystal Skeleton",
            "Shroomskull",
            "Charred Skeleton",
            "Frostbones",
            "Pharaoh's Guard",
        ][b],
        Foe::Ghost => [
            "Willow Ghost",
            "Glimmer Ghost",
            "Puffball Ghost",
            "Ember Ghost",
            "Snow Ghost",
            "Tomb Ghost",
        ][b],
        Foe::Snail => [
            "Moss Lantern Snail",
            "Crystal Lantern Snail",
            "Spore Lantern Snail",
            "Ember Lantern Snail",
            "Frost Lantern Snail",
            "Sunstone Lantern Snail",
        ][b],
        Foe::Bookworm => [
            "Moss Bibliomancer",
            "Crystal Bibliomancer",
            "Spore Bibliomancer",
            "Cinder Bibliomancer",
            "Frost Bibliomancer",
            "Tomb Bibliomancer",
        ][b],
        Foe::Slime => "Slime",
        Foe::Bat => "Bat",
        Foe::Shroom => "Shroomling",
        Foe::Crab => "Crystal Crab",
        Foe::Wisp => "Wisp",
        Foe::Beetle => "Beetle",
        Foe::Imp => "Imp",
        Foe::Golem => "Golem",
        Foe::Frog => "Bog Frog",
        Foe::Jelly => "Drift Jelly",
        Foe::Puffer => "Puffer",
        Foe::Drake if b == 3 => "Cinder Drakeling",
        Foe::Drake => "Frost Drakeling",
        Foe::Leafling => "Leafling",
        Foe::Werewolf => "Werewolf",
    }
}

/// What a creature is called in the old sewers.
pub fn sewer_name(f: Foe, biome: usize) -> &'static str {
    match f {
        Foe::Slime => "Sludge Slime",
        Foe::Bat => "Bone Bat",
        Foe::Zombie => "Bone Shambler",
        Foe::Brute => "Bone Brute",
        Foe::Sneak => "Bone Sneak",
        Foe::Bug => "Bone Spider",
        Foe::Skeleton => "Sewer Skeleton",
        Foe::Ghost => "Skull Ghost",
        Foe::Frog => "Bone Frog",
        Foe::Puffer => "Bone Puffer",
        Foe::Snail => "Bone Snail",
        Foe::Bookworm => "Bone Bibliomancer",
        _ => kind_name(f, biome),
    }
}

pub fn boss_name(f: Foe) -> &'static str {
    match f {
        Foe::Slime => "King Slime",
        Foe::Crab => "Crystal Matriarch",
        Foe::Shroom => "Old Capwood",
        Foe::Imp => "Ember Lord",
        Foe::Golem => "Frost Colossus",
        Foe::Skeleton => "The Bone Warden",
        _ => "Guardian",
    }
}

/// A guardian's name where it stands: the classic six, and the giants that guard the tenth
/// floors on later trips down.
pub fn guardian_name(f: Foe, biome: usize) -> &'static str {
    match (f, biome % 6) {
        (Foe::Brute, 0) => "Grub the Goblin King",
        (Foe::Bug, 1) => "The Glass Queen",
        (Foe::Zombie, 2) => "The Rotting Gardener",
        (Foe::Sneak, 3) => "Ashfang the Quick",
        (Foe::Ghost, 4) => "The Frost Wraith",
        (Foe::Zombie, 5) => "The Mummy King",
        _ => boss_name(f),
    }
}

/// How much bigger than life a guardian is.
pub const BOSS_SCALE: f32 = 2.8;

impl Enemy {
    pub fn new(foe: Foe, x: f32, z: f32, depth: u32, biome: usize, boss: bool, seed: u32) -> Enemy {
        let mut b = base(foe);
        if foe == Foe::Bug {
            // Each biome's bug fights its own way: glass mantises cut deep, fire ants are
            // quick, frost spiders and scarabs are hard to crack.
            let (h, d, sp) = [
                (1.0, 1.0, 1.0),
                (0.9, 1.35, 0.9),
                (0.8, 0.9, 0.75),
                (1.0, 1.1, 1.25),
                (1.5, 1.0, 0.85),
                (1.4, 1.1, 0.95),
            ][biome % 6];
            b.hp = (b.hp as f32 * h) as i32;
            b.dmg = (b.dmg as f32 * d) as i32;
            b.speed *= sp;
        }
        let d = depth.max(1) as f32 - 1.0;
        let mut hp = (b.hp as f32 * (1.0 + 0.14 * d)) as i32;
        let mut dmg = (b.dmg as f32 * (1.0 + 0.08 * d)) as i32;
        let mut xp = (b.xp as f32 * (1.0 + 0.1 * d)) as u32;
        let mut radius = b.radius;
        if boss {
            // Oversized, and strong with it.
            hp *= 14;
            dmg = dmg * 17 / 10;
            xp *= 15;
            radius *= 2.4;
        }
        Enemy {
            foe,
            boss,
            biome,
            pos: Vec2::new(x, z),
            vel: Vec2::ZERO,
            y: if flies(foe, biome) { 0.55 } else { 0.0 },
            vy: 0.0,
            hp,
            max_hp: hp,
            dmg,
            speed: b.speed,
            radius,
            xp,
            st: St::Idle,
            t: 0.5 + (seed % 100) as f32 / 100.0,
            dir: Vec2::new(0.0, 1.0),
            yaw: 0.0,
            flash: 0.0,
            hurt_cd: 0.0,
            anim: (seed % 628) as f32 / 100.0,
            alert: false,
            hops: 0,
            squash: 0.0,
            seed,
            burn: 0.0,
            burn_dmg: 0,
            burn_tick: 0.5,
            chill: 0.0,
            fury: 1.0,
            moonlit: false,
            named: 0.0,
            summon: 6.0,
            hide: 0.0,
            trail: Vec::new(),
            sewer: false,
            glowcave: false,
            call: None,
        }
    }

    /// Dressed for the old sewers (see `sewer`). Their spiders walk, whatever the biome's
    /// bugs do.
    pub fn in_the_sewers(mut self) -> Enemy {
        self.sewer = true;
        if !self.flying() {
            self.y = 0.0;
        }
        self
    }

    /// Living in the glowcap caves (see `glowcave`).
    pub fn in_the_glowcaves(mut self) -> Enemy {
        self.glowcave = true;
        self
    }

    /// A glowing shroomling's colour, as the glowcaps' (see `glowcave_art::GLOW`).
    pub fn glow_colour(&self) -> usize {
        self.seed as usize % 4
    }

    /// One of its own kind, called up or split off: dressed as it is.
    fn kin(&self, e: Enemy) -> Enemy {
        let e = if self.sewer { e.in_the_sewers() } else { e };
        if self.glowcave {
            e.in_the_glowcaves()
        } else {
            e
        }
    }

    /// Which of its family's looks it wears: its biome's, or the sewers'.
    pub fn look(&self) -> usize {
        if self.sewer {
            SEWER_LOOK
        } else {
            self.biome % BIOMES
        }
    }

    /// Its name, as shown over its health bar and in the guardian's banner.
    pub fn name(&self) -> &'static str {
        if self.boss {
            guardian_name(self.foe, self.biome)
        } else if self.sewer {
            sewer_name(self.foe, self.biome)
        } else {
            kind_name(self.foe, self.biome)
        }
    }

    /// Floats or flies rather than walks.
    pub fn flying(&self) -> bool {
        flies(self.foe, self.biome) && !(self.sewer && self.foe == Foe::Bug)
    }

    /// Tonight's moon stirs them up (or calms them down). A full moon makes them glow red,
    /// hit harder and take more beating, and they drop better things for it.
    pub fn feel_the_moon(&mut self, phase: super::sky::MoonPhase) {
        self.fury = phase.fury();
        self.speed *= self.fury.sqrt();
        if phase.full() {
            self.moonlit = true;
            self.max_hp = (self.max_hp as f32 * 1.6).ceil() as i32;
            self.hp = self.max_hp;
            self.dmg = (self.dmg as f32 * 1.25).ceil() as i32;
            self.xp = self.xp * 3 / 2 + 1;
        }
    }

    pub fn scale(&self) -> f32 {
        if self.boss { BOSS_SCALE } else { 1.0 }
    }

    /// How far its attacks reach compared with one of normal size.
    fn reach(&self) -> f32 {
        self.scale().sqrt()
    }

    pub fn world_pos(&self) -> Vec3 {
        Vec3::new(self.pos.x, self.y, self.pos.y)
    }

    /// Jelly and ghosts are see-through, so they are drawn after everything else. (Sludge
    /// isn't.)
    pub fn translucent(&self) -> bool {
        matches!(self.foe, Foe::Ghost | Foe::Jelly) || (self.foe == Foe::Slime && !self.sewer)
    }

    /// Hard shells and bones strike sparks (a lantern snail's, while it hides in it).
    pub fn armored(&self) -> bool {
        matches!(
            self.foe,
            Foe::Crab | Foe::Beetle | Foe::Golem | Foe::Skeleton | Foe::Drake
        ) || (self.foe == Foe::Bug && (self.sewer || self.biome % 6 != MOTHS))
            || self.shelled()
    }

    /// A lantern snail tucked away in its shell.
    pub fn shelled(&self) -> bool {
        self.foe == Foe::Snail && self.hide > 0.0
    }

    /// Too heavy to be knocked about much.
    pub fn heavy(&self) -> bool {
        self.boss || matches!(self.foe, Foe::Golem | Foe::Brute)
    }

    /// Does this enemy touch the ground (and so can bump the player)?
    pub fn grounded(&self) -> bool {
        match self.foe {
            Foe::Slime | Foe::Frog => self.y < 0.35 * self.scale(),
            Foe::Snail => !self.shelled(),
            _ => true,
        }
    }

    fn passes_walls(&self) -> bool {
        self.foe == Foe::Ghost
    }

    fn step(&mut self, world: &World, delta: Vec2) {
        if self.passes_walls() {
            let n = self.pos + delta;
            if world.inside(n.x as i32, n.y as i32) {
                self.pos = n;
            }
        } else {
            self.pos = world.move_circle(self.pos, delta, self.radius.min(0.45));
        }
    }

    /// Advances the AI. Returns true if the enemy wants to deal contact damage this frame.
    #[allow(clippy::too_many_arguments)]
    pub fn update(
        &mut self,
        dt: f32,
        world: &World,
        player: Vec2,
        shots: &mut Vec<Shot>,
        spawns: &mut Vec<Enemy>,
        fx: &mut Fx,
        rng: &mut Rng,
        depth: u32,
    ) {
        self.anim += dt;
        self.named = (self.named - dt).max(0.0);
        self.hide = (self.hide - dt).max(0.0);
        if self.foe == Foe::Snail {
            self.lay_trail(dt);
        }
        self.flash = (self.flash - dt).max(0.0);
        self.hurt_cd = (self.hurt_cd - dt).max(0.0);
        self.squash = approach(self.squash, 0.0, dt * 3.0);
        // Knockback decays.
        if self.vel.length_squared() > 0.001 {
            let v = self.vel * dt;
            self.step(world, v);
            self.vel *= (1.0 - dt * 8.0).max(0.0);
        }
        let to = player - self.pos;
        let dist = to.length();
        let dirp = to.normalize_or_zero();
        if !self.alert {
            let sees = dist < 7.0 * self.fury
                && (self.passes_walls() || world.clear_line(self.pos, player));
            if sees || (self.boss && dist < 9.0) {
                self.alert = true;
                fx.popup(self.world_pos() + Vec3::Y * (0.8 * self.scale()), "!", GOLD);
                if self.foe == Foe::Werewolf && self.hops == 0 {
                    // The first scent of you: its head goes back for a howl that brings the
                    // whole floor running.
                    self.hops = 1;
                    self.st = St::Howl;
                    self.t = HOWL_SECS;
                    self.call = Some(Call::Howl);
                }
            } else {
                self.idle_wander(dt, world, rng);
                return;
            }
        }
        if dist > 16.0 * self.fury.max(1.0) && !self.boss {
            self.alert = false;
            return;
        }
        // A bigger moon, a shorter fuse.
        self.t -= dt * self.fury;
        if self.boss && matches!(self.foe, Foe::Brute | Foe::Sneak | Foe::Bug) {
            // Goblin kings whistle up their sneaks; bug queens call their brood.
            self.summon -= dt;
            if self.summon <= 0.0 {
                self.summon = 7.0;
                let help = if self.foe == Foe::Bug {
                    Foe::Bug
                } else {
                    Foe::Sneak
                };
                for k in 0..2u32 {
                    let a = k as f32 * std::f32::consts::PI + self.anim;
                    let p = self.pos + Vec2::new(a.cos(), a.sin()) * 1.6;
                    if !world.blocked(p.x as i32, p.y as i32) {
                        let mut e = self.kin(Enemy::new(
                            help,
                            p.x,
                            p.y,
                            depth,
                            self.biome,
                            false,
                            self.seed + k,
                        ));
                        e.alert = true;
                        spawns.push(e);
                    }
                }
                fx.burst(
                    self.world_pos(),
                    12,
                    &[WHITE, CREAM, self.color()],
                    3.0,
                    2.0,
                );
            }
        }
        match self.foe {
            Foe::Slime => self.slime(dt, world, dirp, dist, spawns, rng, depth),
            Foe::Bat => {
                self.y = 0.55 + (self.anim * 5.0).sin() * 0.1;
                let side =
                    Vec2::new(-dirp.y, dirp.x) * (self.anim * 3.0 + self.seed as f32).sin() * 0.8;
                let dir = if self.st == St::Rest {
                    -dirp
                } else {
                    (dirp + side).normalize_or_zero()
                };
                if self.st == St::Rest && self.t <= 0.0 {
                    self.st = St::Chase;
                }
                self.dir = dir;
                self.step(world, dir * self.speed * dt);
            }
            Foe::Shroom | Foe::Skeleton | Foe::Ghost | Foe::Zombie => {
                if self.foe == Foe::Ghost {
                    self.y = 0.5 + (self.anim * 2.0).sin() * 0.12;
                }
                match self.st {
                    St::Windup => {
                        if self.t <= 0.0 {
                            self.st = St::Dash;
                            self.t = 0.25;
                        }
                    }
                    St::Dash => {
                        let d = self.dir * self.speed * 3.2 * dt;
                        self.step(world, d);
                        if self.t <= 0.0 {
                            self.st = St::Rest;
                            self.t = 0.6;
                        }
                    }
                    St::Rest => {
                        if self.t <= 0.0 {
                            self.st = St::Chase;
                        }
                    }
                    _ => {
                        self.dir = dirp;
                        if self.foe == Foe::Skeleton && dist < 1.6 * self.reach() {
                            self.st = St::Windup;
                            self.t = 0.35;
                        } else if self.foe == Foe::Zombie && dist < 1.5 * self.reach() {
                            // Arms up, a groan, and a grab.
                            self.st = St::Windup;
                            self.t = 0.55;
                        } else if self.foe == Foe::Zombie {
                            // A lurching shamble: a drag of the foot, then a lunge of a step.
                            let lurch = 0.45 + 0.9 * (self.anim * 4.0).sin().abs();
                            self.step(world, dirp * self.speed * lurch * dt);
                        } else {
                            self.step(world, dirp * self.speed * dt);
                        }
                        if self.boss && self.t <= 0.0 {
                            // Summon helpers now and then.
                            self.t = 4.0;
                            for k in 0..2 {
                                let a = k as f32 * 3.0 + self.anim;
                                let p = self.pos + Vec2::new(a.cos(), a.sin()) * 1.2;
                                spawns.push(self.kin(Enemy::new(
                                    self.foe,
                                    p.x,
                                    p.y,
                                    depth,
                                    self.biome,
                                    false,
                                    self.seed + k,
                                )));
                            }
                        }
                    }
                }
            }
            Foe::Crab | Foe::Beetle | Foe::Golem => {
                self.charger(dt, world, dirp, dist, fx, shots, depth)
            }
            Foe::Wisp | Foe::Imp => self.shooter(dt, world, dirp, dist, shots, rng),
            Foe::Frog => self.frog(dt, world, dirp, dist, shots, rng),
            Foe::Jelly | Foe::Puffer => self.floater(dt, world, dirp, dist, shots, fx, rng),
            Foe::Brute => self.brute(dt, world, dirp, dist, shots, fx),
            Foe::Sneak => self.sneak(dt, world, dirp, dist, shots, rng),
            Foe::Bug => self.bug(dt, world, dirp, dist, shots, rng),
            Foe::Snail => self.snail(dt, world, dirp, dist),
            Foe::Bookworm => self.bookworm(dt, world, dirp, dist, shots, rng),
            Foe::Drake => self.drake(dt, world, dirp, dist, shots, rng),
            Foe::Leafling => self.leafling(dt, world, dirp, dist, shots, rng),
            Foe::Werewolf => self.werewolf(dt, world, dirp, dist, rng),
        }
        if dist > 0.01 && self.st != St::Dash {
            self.yaw = dirp.x.atan2(dirp.y);
        } else if self.st == St::Dash {
            self.yaw = self.dir.x.atan2(self.dir.y);
        }
    }

    /// Glowing slime where a lantern snail has crawled, fading as it dries.
    fn lay_trail(&mut self, dt: f32) {
        for p in &mut self.trail {
            p.1 += dt;
        }
        self.trail.retain(|p| p.1 < TRAIL_SECS);
        let moved = self
            .trail
            .last()
            .is_none_or(|p| (p.0 - self.pos).length() > 0.16);
        if moved && !self.shelled() {
            self.trail.push((self.pos, 0.0));
        }
    }

    /// A lantern snail glides after you, slow and steady, and once it's close draws back
    /// and lunges. Struck, it hides in its shell a moment (see `hide`).
    fn snail(&mut self, dt: f32, world: &World, dirp: Vec2, dist: f32) {
        if self.shelled() {
            return;
        }
        match self.st {
            St::Windup => {
                if self.t <= 0.0 {
                    self.st = St::Dash;
                    self.t = 0.2;
                }
            }
            St::Dash => {
                let d = self.dir * self.speed * 6.5 * dt;
                self.step(world, d);
                if self.t <= 0.0 {
                    self.st = St::Rest;
                    self.t = 1.1;
                }
            }
            St::Rest => {
                if self.t <= 0.0 {
                    self.st = St::Chase;
                }
            }
            _ => {
                self.dir = dirp;
                // The foot ripples it along in waves.
                let wave = 0.55 + 0.45 * (self.anim * 3.2).sin().max(0.0);
                self.step(world, dirp * self.speed * wave * dt);
                if dist < 1.2 * self.reach() {
                    self.st = St::Windup;
                    self.t = 0.55;
                }
            }
        }
    }

    /// A book-worm bibliomancer keeps a reading distance, rears up over its book and spits
    /// a gob of glowing ink, which dries into a rune pointing the way on.
    fn bookworm(
        &mut self,
        dt: f32,
        world: &World,
        dirp: Vec2,
        dist: f32,
        shots: &mut Vec<Shot>,
        rng: &mut Rng,
    ) {
        match self.st {
            St::Windup => {
                if self.t <= 0.0 {
                    let color = crate::assets::deep_art::INK_COLORS[self.look()][1];
                    let spread: &[f32] = if self.boss {
                        &[-0.4, -0.2, 0.0, 0.2, 0.4]
                    } else {
                        &[0.0]
                    };
                    for a in spread {
                        let d = Vec2::from_angle(*a).rotate(dirp);
                        shots.push(Shot {
                            pos: self.pos + d * 0.35,
                            vel: d * 3.9,
                            dmg: self.dmg,
                            life: 2.4,
                            color,
                            radius: 0.17,
                            kind: ShotKind::Ink,
                        });
                    }
                    self.st = St::Rest;
                    self.t = rng.range_f(1.9, 2.8);
                }
            }
            _ => {
                let strafe =
                    Vec2::new(-dirp.y, dirp.x) * (self.anim * 0.6 + self.seed as f32).sin();
                let want = if dist < 3.2 {
                    -dirp
                } else if dist > 6.5 {
                    dirp
                } else {
                    strafe * 0.5
                };
                // Caterpillars crawl in ripples.
                let ripple = 0.4 + 0.6 * (self.anim * 5.0).sin().abs();
                self.step(world, want * self.speed * ripple * dt);
                if self.t <= 0.0 && dist < 8.5 && world.clear_line(self.pos, self.pos + dirp * dist)
                {
                    self.st = St::Windup;
                    self.t = 0.65;
                }
            }
        }
    }

    /// A drakeling keeps its distance, circling; then it rears back drawing breath (its
    /// mouth glowing) and breathes a gout of fire or frost, sweeping it after you.
    fn drake(
        &mut self,
        dt: f32,
        world: &World,
        dirp: Vec2,
        dist: f32,
        shots: &mut Vec<Shot>,
        rng: &mut Rng,
    ) {
        match self.st {
            St::Windup => {
                if self.t <= 0.0 {
                    self.st = St::Dash;
                    self.t = BREATH_SECS;
                    self.dir = dirp;
                    self.call = Some(Call::Breath);
                }
            }
            St::Dash => {
                // It follows you with its breath, but slowly.
                let turn = (dt * 1.6).min(1.0);
                self.dir = (self.dir + (dirp - self.dir) * turn).normalize_or(self.dir);
                // A puff every so often, spread through a cone.
                let every = 0.05;
                let before = self.t + dt * self.fury;
                if (before / every).floor() != (self.t / every).floor() {
                    let spread = if self.boss { 0.6 } else { 0.32 };
                    let d = Vec2::from_angle(rng.range_f(-spread, spread)).rotate(self.dir);
                    shots.push(Shot {
                        pos: self.pos + self.dir * 0.4 * self.scale(),
                        vel: d * rng.range_f(5.0, 6.2),
                        dmg: self.dmg,
                        life: BREATH_LIFE,
                        color: if self.biome % 6 == 3 { ORANGE } else { SKY },
                        radius: 0.2,
                        kind: ShotKind::Breath,
                    });
                }
                if self.t <= 0.0 {
                    self.st = St::Rest;
                    self.t = 0.9;
                }
            }
            St::Rest => {
                if self.t <= 0.0 {
                    self.st = St::Chase;
                    self.t = rng.range_f(1.2, 2.2);
                }
            }
            _ => {
                // Near enough to scorch you, never near enough to be hit.
                let turn = if self.seed % 2 == 0 { 1.0 } else { -1.0 };
                let circle = Vec2::new(-dirp.y, dirp.x) * turn;
                let want = if dist > 3.8 {
                    dirp
                } else if dist < 2.3 {
                    (circle * 0.5 - dirp).normalize_or_zero()
                } else {
                    circle * 0.6
                };
                self.dir = dirp;
                self.step(world, want * self.speed * dt);
                let sees = world.clear_line(self.pos, self.pos + dirp * dist);
                if self.t <= 0.0 && dist < 4.0 * self.reach() && sees {
                    self.st = St::Windup;
                    self.t = BREATH_WINDUP;
                }
            }
        }
    }

    /// A leafling flits round you in fits and starts, throws a fan of leaves now and then,
    /// and every so often mends whoever's most hurt round it (see `Call::Mend`).
    fn leafling(
        &mut self,
        dt: f32,
        world: &World,
        dirp: Vec2,
        dist: f32,
        shots: &mut Vec<Shot>,
        rng: &mut Rng,
    ) {
        self.y = 0.55 + (self.anim * 4.0).sin() * 0.1;
        self.summon -= dt;
        if self.summon <= 0.0 {
            self.summon = MEND_SECS;
            self.call = Some(Call::Mend);
        }
        match self.st {
            St::Windup => {
                if self.t <= 0.0 {
                    let n = if self.boss { 7 } else { 3 };
                    for k in 0..n {
                        let a = (k as f32 - (n - 1) as f32 * 0.5) * 0.28;
                        let d = Vec2::from_angle(a).rotate(dirp);
                        shots.push(Shot {
                            pos: self.pos + d * 0.3,
                            vel: d * 4.4,
                            dmg: self.dmg,
                            life: 1.8,
                            color: GREEN,
                            radius: 0.14,
                            kind: ShotKind::Leaf,
                        });
                    }
                    self.st = St::Chase;
                    self.t = rng.range_f(1.8, 2.8);
                }
            }
            _ => {
                // Darting this way and that, never still for long.
                let side = Vec2::new(-dirp.y, dirp.x);
                let flit = side * (self.anim * 1.9 + self.seed as f32).sin() * 1.3
                    + Vec2::new((self.anim * 4.3).sin(), (self.anim * 3.1).cos()) * 0.5;
                let want = if dist > 5.0 {
                    dirp
                } else if dist < 2.8 {
                    -dirp
                } else {
                    Vec2::ZERO
                };
                let d = (want + flit).normalize_or_zero();
                let dart = 0.6 + 0.8 * (self.anim * 5.0 + self.seed as f32).sin().max(0.0);
                self.dir = d;
                self.step(world, d * self.speed * dart * dt);
                let sees = world.clear_line(self.pos, self.pos + dirp * dist);
                if self.t <= 0.0 && dist < 7.0 && sees {
                    self.st = St::Windup;
                    self.t = 0.35;
                }
            }
        }
    }

    /// A werewolf, after its howl (see `Call::Howl`), runs you down in long lopes, drops
    /// into a crouch and pounces, claws first.
    fn werewolf(&mut self, dt: f32, world: &World, dirp: Vec2, dist: f32, rng: &mut Rng) {
        match self.st {
            St::Howl => {
                if self.t <= 0.0 {
                    self.st = St::Chase;
                    self.t = 0.3;
                }
            }
            St::Windup => {
                if self.t <= 0.0 {
                    self.st = St::Dash;
                    self.t = 0.32;
                }
            }
            St::Dash => {
                let before = self.pos;
                let d = self.dir * self.speed * 3.2 * dt;
                self.step(world, d);
                if self.t <= 0.0 || (self.pos - before).length() < d.length() * 0.3 {
                    self.st = St::Rest;
                    self.t = 0.6;
                }
            }
            St::Rest => {
                if self.t <= 0.0 {
                    self.st = St::Chase;
                    self.t = rng.range_f(0.5, 1.1);
                }
            }
            _ => {
                self.dir = dirp;
                let sees = world.clear_line(self.pos, self.pos + dirp * dist);
                if self.t <= 0.0 && dist < 3.4 * self.reach() && sees {
                    self.st = St::Windup;
                    self.t = 0.3;
                } else {
                    let lope = 0.75 + 0.45 * (self.anim * 7.0).sin().abs();
                    self.step(world, dirp * self.speed * lope * dt);
                }
            }
        }
    }

    fn idle_wander(&mut self, dt: f32, world: &World, rng: &mut Rng) {
        self.t -= dt;
        if self.t <= 0.0 {
            self.t = rng.range_f(1.0, 2.5);
            let a = rng.f32() * std::f32::consts::TAU;
            self.dir = if rng.chance(0.5) {
                Vec2::new(a.cos(), a.sin())
            } else {
                Vec2::ZERO
            };
        }
        if self.flying() && self.foe != Foe::Ghost {
            self.y = 0.55 + (self.anim * 4.0).sin() * 0.1;
        }
        if !matches!(self.foe, Foe::Slime | Foe::Frog) {
            let d = self.dir * self.speed * 0.35 * dt;
            self.step(world, d);
            if self.dir.length_squared() > 0.0 {
                self.yaw = self.dir.x.atan2(self.dir.y);
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn slime(
        &mut self,
        dt: f32,
        world: &World,
        dirp: Vec2,
        dist: f32,
        spawns: &mut Vec<Enemy>,
        rng: &mut Rng,
        depth: u32,
    ) {
        let s = self.scale();
        match self.st {
            St::Hop => {
                self.vy -= 16.0 * dt;
                self.y += self.vy * dt;
                let d = self.dir * self.speed * 1.5 * dt;
                self.step(world, d);
                if self.y <= 0.0 {
                    self.y = 0.0;
                    self.vy = 0.0;
                    self.st = St::Idle;
                    self.t = rng.range_f(0.35, 0.9);
                    self.squash = 1.0;
                    self.hops += 1;
                    if self.boss && self.hops % 3 == 0 {
                        for k in 0..2 {
                            let a = rng.f32() * std::f32::consts::TAU;
                            let p = self.pos + Vec2::new(a.cos(), a.sin()) * 1.4;
                            if !world.blocked(p.x as i32, p.y as i32) {
                                spawns.push(self.kin(Enemy::new(
                                    Foe::Slime,
                                    p.x,
                                    p.y,
                                    depth,
                                    self.biome,
                                    false,
                                    self.seed + k,
                                )));
                            }
                        }
                    }
                }
            }
            _ => {
                if self.t <= 0.0 {
                    self.st = St::Hop;
                    self.vy = 4.6 * s.sqrt();
                    self.dir = if dist < 9.0 { dirp } else { Vec2::ZERO };
                    self.squash = 0.6;
                } else if self.t < 0.2 {
                    self.squash = 0.8;
                }
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn charger(
        &mut self,
        dt: f32,
        world: &World,
        dirp: Vec2,
        dist: f32,
        fx: &mut Fx,
        shots: &mut Vec<Shot>,
        _depth: u32,
    ) {
        let golem = self.foe == Foe::Golem;
        match self.st {
            St::Windup => {
                if self.t <= 0.0 {
                    if golem {
                        // Ground slam: a ring of shards.
                        fx.burst(self.world_pos(), 18, &[SAND, KHAKI, WHITE], 4.0, 2.0);
                        let n = if self.boss { 12 } else { 6 };
                        for k in 0..n {
                            let a = k as f32 / n as f32 * std::f32::consts::TAU + self.anim;
                            shots.push(Shot {
                                pos: self.pos,
                                vel: Vec2::new(a.cos(), a.sin()) * 4.0,
                                dmg: self.dmg * 2 / 3,
                                life: 1.2,
                                color: SKY,
                                radius: 0.18,
                                kind: ShotKind::Spark,
                            });
                        }
                        self.st = St::Rest;
                        self.t = 1.0;
                    } else {
                        self.st = St::Dash;
                        self.t = 0.45;
                    }
                }
            }
            St::Dash => {
                let before = self.pos;
                let d = self.dir * 7.5 * dt;
                self.step(world, d);
                if self.t <= 0.0 || (self.pos - before).length() < d.length() * 0.3 {
                    self.st = St::Rest;
                    self.t = 0.8;
                }
            }
            St::Rest => {
                if self.t <= 0.0 {
                    self.st = St::Chase;
                }
            }
            _ => {
                self.dir = dirp;
                let trigger = if golem { 2.0 * self.reach() } else { 4.2 };
                if dist < trigger && world.clear_line(self.pos, self.pos + dirp * dist) {
                    self.st = St::Windup;
                    self.t = if golem { 0.7 } else { 0.5 };
                } else {
                    self.step(world, dirp * self.speed * dt);
                }
            }
        }
    }

    fn shooter(
        &mut self,
        dt: f32,
        world: &World,
        dirp: Vec2,
        dist: f32,
        shots: &mut Vec<Shot>,
        rng: &mut Rng,
    ) {
        if self.foe == Foe::Wisp {
            self.y = 0.55 + (self.anim * 3.0).sin() * 0.1;
        }
        match self.st {
            St::Windup => {
                if self.t <= 0.0 {
                    let color = if self.foe == Foe::Imp { ORANGE } else { AQUA };
                    let spread: &[f32] = if self.boss {
                        &[-0.5, -0.25, 0.0, 0.25, 0.5]
                    } else {
                        &[0.0]
                    };
                    for a in spread {
                        let d = Vec2::from_angle(*a).rotate(dirp);
                        shots.push(Shot {
                            pos: self.pos + d * 0.3,
                            vel: d * 4.6,
                            dmg: self.dmg,
                            life: 2.5,
                            color,
                            radius: 0.16,
                            kind: ShotKind::Spark,
                        });
                    }
                    self.st = St::Chase;
                    self.t = rng.range_f(1.6, 2.4);
                }
            }
            _ => {
                let strafe =
                    Vec2::new(-dirp.y, dirp.x) * (self.anim * 0.7 + self.seed as f32).sin();
                let want = if dist < 3.0 {
                    -dirp
                } else if dist > 6.0 {
                    dirp
                } else {
                    strafe
                };
                self.step(world, want * self.speed * dt);
                if self.t <= 0.0 && dist < 9.0 && world.clear_line(self.pos, self.pos + dirp * dist)
                {
                    self.st = St::Windup;
                    self.t = 0.45;
                }
            }
        }
    }

    /// A bog frog: long hops towards you, and now and then a spat bubble.
    fn frog(
        &mut self,
        dt: f32,
        world: &World,
        dirp: Vec2,
        dist: f32,
        shots: &mut Vec<Shot>,
        rng: &mut Rng,
    ) {
        match self.st {
            St::Hop => {
                self.vy -= 18.0 * dt;
                self.y += self.vy * dt;
                let d = self.dir * self.speed * 1.8 * dt;
                self.step(world, d);
                if self.y <= 0.0 {
                    self.y = 0.0;
                    self.vy = 0.0;
                    self.squash = 1.0;
                    self.hops += 1;
                    if dist < 6.5 && rng.chance(0.35) {
                        self.st = St::Windup;
                        self.t = 0.4;
                    } else {
                        self.st = St::Idle;
                        self.t = rng.range_f(0.3, 0.8);
                    }
                }
            }
            St::Windup => {
                self.squash = 0.5;
                if self.t <= 0.0 {
                    shots.push(Shot {
                        pos: self.pos + dirp * 0.3,
                        vel: dirp * 4.2,
                        dmg: self.dmg,
                        life: 2.0,
                        color: SKY,
                        radius: 0.17,
                        kind: ShotKind::Spark,
                    });
                    self.st = St::Idle;
                    self.t = rng.range_f(0.5, 1.0);
                }
            }
            _ => {
                if self.t <= 0.0 {
                    self.st = St::Hop;
                    self.vy = 5.4;
                    self.dir = if dist < 10.0 { dirp } else { Vec2::ZERO };
                    self.squash = 0.6;
                }
            }
        }
    }

    /// Drift jellies and puffers bob towards you, then let fly: a ring of sparks, or a
    /// burst of spines once the puffer has blown itself up.
    #[allow(clippy::too_many_arguments)]
    fn floater(
        &mut self,
        dt: f32,
        world: &World,
        dirp: Vec2,
        dist: f32,
        shots: &mut Vec<Shot>,
        fx: &mut Fx,
        rng: &mut Rng,
    ) {
        let jelly = self.foe == Foe::Jelly;
        self.y = 0.55 + (self.anim * if jelly { 2.2 } else { 3.0 }).sin() * 0.08;
        match self.st {
            St::Windup => {
                if !jelly {
                    self.squash = (0.6 - self.t).clamp(0.0, 0.6) / 0.6;
                }
                if self.t <= 0.0 {
                    let n = if self.boss { 14 } else { 8 };
                    let (color, speed, life) = if jelly {
                        (CREAM, 5.0, 0.35)
                    } else {
                        (SAND, 4.6, 0.9)
                    };
                    for k in 0..n {
                        let a = k as f32 / n as f32 * std::f32::consts::TAU + self.anim;
                        shots.push(Shot {
                            pos: self.pos,
                            vel: Vec2::new(a.cos(), a.sin()) * speed,
                            dmg: self.dmg * 2 / 3,
                            life,
                            color,
                            radius: 0.15,
                            kind: ShotKind::Spark,
                        });
                    }
                    if jelly {
                        fx.burst(self.world_pos(), 12, &[WHITE, CREAM, AQUA], 3.0, 1.0);
                    }
                    self.st = St::Rest;
                    self.t = if jelly { 1.3 } else { 1.7 };
                }
            }
            St::Rest => {
                self.squash = (self.squash - dt * 2.0).max(0.0);
                if self.t <= 0.0 {
                    self.st = St::Chase;
                }
            }
            _ => {
                let near = if jelly { 1.9 } else { 3.2 };
                let keep = if jelly { 0.8 } else { 2.2 };
                if dist < near && self.t <= 0.0 {
                    self.st = St::Windup;
                    self.t = if jelly { 0.55 } else { 0.6 };
                } else {
                    let want = if dist > keep {
                        dirp
                    } else {
                        Vec2::new(-dirp.y, dirp.x)
                    };
                    let wobble = Vec2::new((self.anim * 1.7).sin(), (self.anim * 1.3).cos()) * 0.3;
                    let d = (want + wobble).normalize_or_zero();
                    self.dir = d;
                    self.step(world, d * self.speed * dt);
                    if self.t <= 0.0 {
                        self.t = rng.range_f(0.2, 0.6);
                    }
                }
            }
        }
    }

    /// A fat goblin: plods up, raises its club and slams the ground, sending a shockwave
    /// out in front (all round, for a guardian).
    #[allow(clippy::too_many_arguments)]
    fn brute(
        &mut self,
        dt: f32,
        world: &World,
        dirp: Vec2,
        dist: f32,
        shots: &mut Vec<Shot>,
        fx: &mut Fx,
    ) {
        match self.st {
            St::Windup => {
                if self.t <= 0.0 {
                    let hit = self.pos + self.dir * 0.7 * self.scale();
                    let skin = GOBLIN_DUST[self.look()];
                    fx.burst(Vec3::new(hit.x, 0.05, hit.y), 16, &skin, 3.5, 2.0);
                    let fan: Vec<f32> = if self.boss {
                        (0..14)
                            .map(|k| k as f32 / 14.0 * std::f32::consts::TAU)
                            .collect()
                    } else {
                        vec![-0.7, -0.35, 0.0, 0.35, 0.7]
                    };
                    for a in fan {
                        let d = Vec2::from_angle(a).rotate(self.dir);
                        shots.push(Shot {
                            pos: hit,
                            vel: d * 3.8,
                            dmg: self.dmg * 2 / 3,
                            life: if self.boss { 0.9 } else { 0.4 },
                            color: skin[0],
                            radius: 0.2,
                            kind: ShotKind::Spark,
                        });
                    }
                    self.st = St::Rest;
                    self.t = 0.9;
                }
            }
            St::Rest => {
                if self.t <= 0.0 {
                    self.st = St::Chase;
                }
            }
            _ => {
                self.dir = dirp;
                let reach = 1.9 * self.reach();
                if dist < reach && world.clear_line(self.pos, self.pos + dirp * dist) {
                    self.st = St::Windup;
                    self.t = 0.65;
                } else {
                    self.step(world, dirp * self.speed * dt);
                }
            }
        }
    }

    /// A skinny goblin: circles just out of reach, darts in to stab and away again, and
    /// throws daggers from further off.
    fn sneak(
        &mut self,
        dt: f32,
        world: &World,
        dirp: Vec2,
        dist: f32,
        shots: &mut Vec<Shot>,
        rng: &mut Rng,
    ) {
        match self.st {
            St::Windup => {
                if self.t <= 0.0 {
                    self.st = St::Dash;
                    self.t = 0.28;
                }
            }
            St::Dash => {
                let d = self.dir * self.speed * 2.4 * dt;
                self.step(world, d);
                if self.t <= 0.0 {
                    self.st = St::Rest;
                    self.t = 0.7;
                }
            }
            St::Rest => {
                // Skip back out of reach.
                self.step(world, -dirp * self.speed * 0.8 * dt);
                if self.t <= 0.0 {
                    self.st = St::Chase;
                    self.t = rng.range_f(0.4, 1.2);
                }
            }
            _ => {
                let turn = if self.seed % 2 == 0 { 1.0 } else { -1.0 };
                let circle = Vec2::new(-dirp.y, dirp.x) * turn;
                let want = if dist > 3.2 {
                    dirp
                } else if dist < 2.0 {
                    (circle - dirp).normalize_or_zero()
                } else {
                    circle
                };
                self.dir = dirp;
                self.step(world, want * self.speed * 0.8 * dt);
                let sees = world.clear_line(self.pos, self.pos + dirp * dist);
                if self.t <= 0.0 && sees {
                    if dist < 3.4 * self.reach() {
                        self.st = St::Windup;
                        self.t = 0.22;
                    } else if dist < 8.0 {
                        // A thrown dagger.
                        let spread: &[f32] = if self.boss { &[-0.2, 0.0, 0.2] } else { &[0.0] };
                        for a in spread {
                            let d = Vec2::from_angle(*a).rotate(dirp);
                            shots.push(Shot {
                                pos: self.pos + d * 0.3,
                                vel: d * 6.0,
                                dmg: (self.dmg * 4 / 5).max(1),
                                life: 1.6,
                                color: DAGGERS[self.look()],
                                radius: 0.12,
                                kind: ShotKind::Spark,
                            });
                        }
                        self.t = rng.range_f(1.8, 2.8);
                    }
                }
            }
        }
    }

    /// Bugs skitter in stop-and-go zigzags and pounce; spore moths flutter about instead
    /// and puff spores at you.
    fn bug(
        &mut self,
        dt: f32,
        world: &World,
        dirp: Vec2,
        dist: f32,
        shots: &mut Vec<Shot>,
        rng: &mut Rng,
    ) {
        let side = Vec2::new(-dirp.y, dirp.x);
        if self.flying() {
            self.y = 0.55 + (self.anim * 5.0).sin() * 0.12;
            let flutter = side * (self.anim * 2.3 + self.seed as f32).sin() * 1.2
                + Vec2::new((self.anim * 3.7).sin(), (self.anim * 2.9).cos()) * 0.4;
            let want = if dist > 2.2 { dirp } else { -dirp * 0.3 };
            let d = (want + flutter).normalize_or_zero();
            self.dir = d;
            self.step(world, d * self.speed * dt);
            if self.t <= 0.0 {
                if dist < 6.0 && world.clear_line(self.pos, self.pos + dirp * dist) {
                    let n = if self.boss { 6 } else { 1 };
                    for k in 0..n {
                        let a = (k as f32 - (n - 1) as f32 * 0.5) * 0.35;
                        let v = Vec2::from_angle(a).rotate(dirp);
                        shots.push(Shot {
                            pos: self.pos + v * 0.2,
                            vel: v * 2.4,
                            dmg: self.dmg,
                            life: 2.4,
                            color: PINK,
                            radius: 0.2,
                            kind: ShotKind::Spark,
                        });
                    }
                }
                self.t = rng.range_f(2.0, 3.2);
            }
            return;
        }
        match self.st {
            St::Windup => {
                if self.t <= 0.0 {
                    self.st = St::Dash;
                    self.t = 0.22;
                }
            }
            St::Dash => {
                let d = self.dir * 7.0 * dt;
                self.step(world, d);
                if self.t <= 0.0 {
                    self.st = St::Rest;
                    self.t = 0.6;
                }
            }
            St::Rest => {
                if self.t <= 0.0 {
                    self.st = St::Chase;
                }
            }
            _ => {
                if dist < 1.8 * self.reach() && self.t <= 0.0 {
                    self.dir = dirp;
                    self.st = St::Windup;
                    self.t = if self.biome % 6 == 1 { 0.4 } else { 0.3 };
                    return;
                }
                let zig = side * (self.anim * 6.0 + self.seed as f32).sin() * 0.9;
                let d = (dirp + zig).normalize_or_zero();
                // Stop, start, stop: the way bugs go.
                let go = (0.3 + 1.1 * (self.anim * 3.1 + self.seed as f32).sin()).max(0.0);
                self.dir = d;
                self.step(world, d * self.speed * go * dt);
                if self.t <= 0.0 {
                    self.t = rng.range_f(0.2, 0.5);
                }
            }
        }
    }

    pub fn color(&self) -> u8 {
        if self.sewer {
            // Bone, mostly.
            return match self.foe {
                Foe::Slime => CLAY,
                Foe::Ghost => LIME,
                Foe::Snail => crate::assets::deep_art::GLASS[SEWER_LOOK][1],
                Foe::Bookworm => crate::assets::deep_art::INK_COLORS[SEWER_LOOK][1],
                _ => SAND,
            };
        }
        match (self.foe, self.biome) {
            (Foe::Slime, 0) => GREEN,
            (Foe::Slime, 1) => BLUE,
            (Foe::Slime, 2) => PINK,
            (Foe::Slime, 3) => ORANGE,
            (Foe::Slime, 4) => WHITE,
            (Foe::Slime, _) => PURPLE,
            (Foe::Bat, _) => PURPLE,
            (Foe::Shroom, _) => RED,
            (Foe::Crab, _) => AQUA,
            (Foe::Wisp, _) => MINT,
            (Foe::Beetle, _) => LAVENDER,
            (Foe::Imp, _) => RED,
            (Foe::Skeleton, b) => [KHAKI, SKY, SAND, SHADOW, SKY, GOLD][b % 6],
            (Foe::Golem, _) => KHAKI,
            (Foe::Ghost, b) => [LIME, SKY, PINK, GOLD, WHITE, BLUSH][b % 6],
            (Foe::Frog, 2) => PURPLE,
            (Foe::Frog, _) => GREEN,
            (Foe::Jelly, _) => LAVENDER,
            (Foe::Puffer, _) => GOLD,
            (Foe::Zombie, b) => [GREEN, BLUE, PURPLE, SHADOW, SKY, SAND][b % 6],
            (Foe::Brute | Foe::Sneak, b) => GOBLIN_DUST[b % 6][1],
            (Foe::Bug, b) => [GREEN, AQUA, LAVENDER, RED, SKY, GOLD][b % 6],
            (Foe::Snail, b) => crate::assets::deep_art::GLASS[b % 6][1],
            (Foe::Bookworm, b) => crate::assets::deep_art::INK_COLORS[b % 6][1],
            (Foe::Drake, b) => {
                let look =
                    &crate::assets::beast_art::DRAKES[crate::assets::beast_art::drake_look(b)];
                look.scales[1]
            }
            (Foe::Leafling, _) => GREEN,
            (Foe::Werewolf, _) => KHAKI,
        }
    }

    pub fn light(&self) -> Option<PointLight> {
        if self.moonlit {
            return Some(PointLight {
                pos: self.world_pos() + Vec3::Y * 0.4,
                radius: 2.4 * self.scale(),
                power: 0.4,
                warmth: 8.0,
            });
        }
        match self.foe {
            // A glowcap caves' shroomling lights its way like the caps round it.
            Foe::Shroom if self.glowcave => Some(PointLight {
                pos: self.world_pos() + Vec3::Y * 0.45 * self.scale(),
                radius: 2.6 * self.scale(),
                power: 0.45,
                warmth: crate::assets::glowcave_art::WARMTH[self.glow_colour()],
            }),
            Foe::Wisp => Some(PointLight {
                pos: self.world_pos() + Vec3::Y * 0.3,
                radius: 3.5,
                power: 0.6,
                warmth: 2.0,
            }),
            Foe::Imp => Some(PointLight {
                pos: self.world_pos() + Vec3::Y * 0.5,
                radius: 2.5,
                power: 0.35,
                warmth: 8.0,
            }),
            Foe::Jelly => Some(PointLight {
                pos: self.world_pos() + Vec3::Y * 0.2,
                radius: 3.0,
                power: 0.45,
                warmth: 2.2,
            }),
            // Fire ants' tails glow like coals.
            Foe::Bug if self.biome % 6 == 3 && !self.sewer => Some(PointLight {
                pos: self.world_pos() + Vec3::Y * 0.2,
                radius: 1.8 * self.scale(),
                power: 0.3,
                warmth: 7.0,
            }),
            // A lantern snail lights up the dark all round it, brightest tucked in its shell.
            Foe::Snail => Some(PointLight {
                pos: self.world_pos() + Vec3::Y * 0.35,
                radius: 3.8 * self.scale(),
                power: if self.shelled() { 0.75 } else { 0.55 },
                warmth: [3.5, 2.0, 4.5, 7.5, 1.0, 6.5, 2.5][self.look()],
            }),
            // A drakeling's mouth glows as it draws breath, and its breath lights the way
            // ahead of it.
            Foe::Drake if matches!(self.st, St::Windup | St::Dash) => {
                let breathing = self.st == St::Dash;
                let ahead = if breathing { 1.3 } else { 0.35 } * self.scale();
                let at = self.pos + self.dir * ahead;
                let fire = self.biome % 6 == 3;
                Some(PointLight {
                    pos: Vec3::new(at.x, 0.5 * self.scale(), at.y),
                    radius: if breathing { 3.4 } else { 1.6 },
                    power: match (breathing, fire) {
                        (false, _) => 0.4,
                        (true, true) => 0.75,
                        (true, false) => 0.45,
                    },
                    warmth: if fire { 8.0 } else { 1.2 },
                })
            }
            // A book-worm's pages glow as it gathers its ink.
            Foe::Bookworm if self.st == St::Windup => Some(PointLight {
                pos: self.world_pos() + Vec3::Y * 0.4,
                radius: 1.8,
                power: 0.35,
                warmth: 3.0,
            }),
            _ => None,
        }
    }

    /// The full moon's red aura, drawn over everything once the scene is lit.
    pub fn draw_moonglow(&self, r: &mut Renderer) {
        if !self.moonlit {
            return;
        }
        let s = self.scale();
        let pulse = (self.anim * 3.0).sin() * 0.5 + 0.5;
        let at = self.world_pos() + Vec3::Y * (0.35 * s);
        r.halo(at, 0.55 * s + pulse * 0.1, RED, 0.35 + pulse * 0.2);
        // Motes of red light drifting up off them.
        for k in 0..3 {
            let t = (self.anim * 0.8 + k as f32 * 0.33).fract();
            let a = self.anim * 1.3 + k as f32 * 2.1;
            let q = at + Vec3::new(a.cos() * 0.3 * s, t * 0.8 * s, a.sin() * 0.3 * s);
            r.point(q, 1, if k == 0 { SALMON } else { RED });
        }
    }

    pub fn draw(&self, r: &mut Renderer, a: &Assets) {
        let s = self.scale();
        let base = Vec3::new(self.pos.x, 0.0, self.pos.y);
        r.shadow(a.tex(a.disk), base, self.radius * 0.95);
        let mut o = DrawOpts {
            light: Light::At(base),
            tag: if self.moonlit { MOONLIT_TAG } else { 2 },
            ..Default::default()
        };
        if self.flash > 0.0 {
            o.mode = Mode::Solid(WHITE);
        } else if self.st == St::Windup && (self.anim * 20.0).sin() > 0.0 {
            o.glow = 1.3;
        } else if self.burn > 0.0 {
            o.glow = 0.9 + (self.anim * 12.0).sin() * 0.15;
        }
        let b = self.look();
        let rot = Mat4::from_rotation_y(self.yaw);
        let at = |y: f32| Mat4::from_translation(Vec3::new(self.pos.x, y, self.pos.y));
        let sc = |x: f32, y: f32| Mat4::from_scale(Vec3::new(x * s, y * s, x * s));
        match self.foe {
            Foe::Slime if self.sewer => {
                // A swirl of sludge: it squashes and stretches as it hops, its curl wags, and
                // flies buzz round it.
                let breathe = (self.anim * 4.0).sin() * 0.03;
                let (sx, sy) = if self.st == St::Hop {
                    (0.9, 1.15)
                } else {
                    (
                        1.0 + self.squash * 0.25 + breathe,
                        1.0 - self.squash * 0.3 - breathe,
                    )
                };
                let wag = Vec2::new((self.anim * 3.0).sin(), (self.anim * 2.3).cos()) * 0.04;
                let warp = Warp::Bend {
                    base: self.y,
                    h: 0.5 * s * sy,
                    lean: (-self.vel * 0.03 + wag) * s,
                };
                let body = at(self.y) * rot * sc(sx, sy * 1.2);
                r.mesh(&a.bank, &a.sewer.swirl, &body, &o.with_warp(warp));
                // Eyes glowing red in the dark.
                let eyes = if self.flash > 0.0 {
                    o
                } else {
                    o.with_mode(Mode::Unlit)
                };
                r.mesh(&a.bank, &a.sewer.swirl_eyes, &body, &eyes.with_warp(warp));
                for sx in [-1.0f32, 1.0] {
                    let eye = warp.apply(body.transform_point3(Vec3::new(sx * 0.074, 0.24, 0.26)));
                    r.halo(eye, 0.1 * s, RED, 0.3);
                }
                // Flies: dark specks with a flicker of wing, so they show on a dark floor.
                for i in 0..3 {
                    let t = self.anim * (4.0 + i as f32 * 1.3) + i as f32 * 2.1 + self.seed as f32;
                    let rad = (0.32 + 0.07 * (t * 0.7).sin()) * s;
                    let p = Vec3::new(
                        self.pos.x + t.cos() * rad,
                        self.y + (0.66 + 0.08 * (t * 1.9).sin()) * s * sy,
                        self.pos.y + t.sin() * rad,
                    );
                    r.point(p, 2, INK);
                    if (t * 5.0).sin() > 0.0 {
                        r.point(p + Vec3::Y * 0.03, 1, WHITE);
                    }
                }
            }
            Foe::Slime => {
                let breathe = (self.anim * 4.0).sin() * 0.04;
                let (sx, sy) = if self.st == St::Hop {
                    (0.9, 1.15)
                } else {
                    (
                        1.0 + self.squash * 0.25 + breathe,
                        1.0 - self.squash * 0.3 - breathe,
                    )
                };
                // Jelly wobbles: the top lags behind a shove and quivers after a landing
                // or a hit.
                let side = if self.dir.length_squared() > 0.01 {
                    self.dir.normalize()
                } else {
                    Vec2::new(0.8, 0.6)
                };
                let quiver = (self.anim * 17.0).sin()
                    * (0.012 + self.squash * 0.07 + (self.flash * 3.0).min(0.3));
                let lean = (-self.vel * 0.035 + side * quiver) * s;
                let warp = Warp::Bend {
                    base: self.y,
                    h: 0.45 * s * sy,
                    lean,
                };
                let body = at(self.y) * rot * sc(sx, sy);
                let cols = a.foes.slime_cols[b];
                // The nucleus drifts about inside, and bubbles rise through the jelly.
                let drift = Vec3::new(
                    (self.anim * 1.3).sin() * 0.05,
                    0.2 + (self.anim * 2.1).sin() * 0.025,
                    (self.anim * 1.7).cos() * 0.04,
                );
                let core = at(self.y)
                    * sc(sx, sy)
                    * Mat4::from_translation(drift)
                    * Mat4::from_rotation_y(self.anim * 0.8);
                r.mesh(&a.bank, &a.foes.slime_core[b], &core, &o.with_warp(warp));
                for i in 0..3 {
                    let k = (self.anim * 0.45 + i as f32 * 0.33 + self.seed as f32 * 0.07).fract();
                    let ang = i as f32 * 2.1 + self.seed as f32;
                    let p = Vec3::new(
                        self.pos.x + ang.cos() * 0.17 * s * sx,
                        self.y + (0.06 + k * 0.3) * s * sy,
                        self.pos.y + ang.sin() * 0.17 * s * sx,
                    );
                    r.point(warp.apply(p), 1, if i == 0 { WHITE } else { cols[0] });
                }
                r.mesh(&a.bank, &a.critters.slime_face, &body, &o.with_warp(warp));
                let shell = if self.flash > 0.0 {
                    o
                } else {
                    o.glass(0.36).with_glow(o.glow.max(0.55))
                };
                r.mesh(&a.bank, &a.foes.slime[b], &body, &shell.with_warp(warp));
                // A crisp glint where the dome mirrors the key light.
                let base = Vec3::new(self.pos.x, self.y, self.pos.y);
                let view = (r.cam.eye - base).normalize_or_zero();
                let hv = (crate::render::light::KEY + view).normalize_or_zero();
                let (ra, rb) = (0.42 * s * sx, 0.45 * s * sy);
                let (a2, b2) = (ra * ra, rb * rb);
                let norm = (a2 * hv.x * hv.x + b2 * hv.y * hv.y + a2 * hv.z * hv.z).sqrt();
                let spot =
                    base + Vec3::new(a2 * hv.x, b2 * hv.y, a2 * hv.z) / norm.max(1e-4) + hv * 0.04;
                let spot = warp.apply(spot);
                r.point(spot, if s > 1.0 { 3 } else { 2 }, WHITE);
                r.point(spot + Vec3::new(0.09, -0.07, 0.02) * s, 1, cols[0]);
            }
            Foe::Bat => {
                let m = at(self.y) * rot * sc(1.0, 1.0);
                r.mesh(&a.bank, &a.foes.bat_body[b], &m, &o);
                let flap = (self.anim * 18.0).sin() * 0.7;
                let wo = o.two_sided();
                r.mesh(
                    &a.bank,
                    &a.foes.bat_wing[b],
                    &(m * Mat4::from_translation(Vec3::new(0.1, 0.0, 0.0))
                        * Mat4::from_rotation_z(flap)),
                    &wo,
                );
                r.mesh(
                    &a.bank,
                    &a.foes.bat_wing[b],
                    &(m * Mat4::from_translation(Vec3::new(-0.1, 0.0, 0.0))
                        * Mat4::from_scale(Vec3::new(-1.0, 1.0, 1.0))
                        * Mat4::from_rotation_z(flap)),
                    &wo,
                );
            }
            Foe::Shroom => {
                let waddle = (self.anim * 8.0).sin() * 0.15;
                let hop = (self.anim * 8.0).sin().abs() * 0.08;
                let m = at(hop) * rot * Mat4::from_rotation_z(waddle) * sc(1.0, 1.0);
                if self.glowcave {
                    // Its cap glowing like the glowcaps', a glow round it and spores
                    // puffing off it.
                    use crate::assets::glowcave_art::{GLOW, HALO};
                    let c = self.glow_colour();
                    // It glows by itself: its own colours, whatever light falls on it.
                    let own = DrawOpts {
                        light: Light::Fixed(1.0, crate::palette::NEUTRAL),
                        ..o
                    };
                    r.mesh(&a.bank, &a.foes.glow_shroom[c], &m, &own);
                    let cap = m.transform_point3(Vec3::Y * 0.44);
                    let pulse = (self.anim * 2.0).sin() * 0.5 + 0.5;
                    r.halo(cap, 0.5 * s, HALO[c], 0.26 + pulse * 0.1);
                    let t = (self.anim * 0.4).fract();
                    let q = cap + Vec3::new((self.anim * 1.3).sin() * 0.15, 0.1 + t * 0.7, 0.0) * s;
                    r.point(q, 1, if t < 0.5 { GLOW[c][0] } else { GLOW[c][1] });
                } else {
                    r.mesh(&a.bank, &a.foes.shroom[b], &m, &o);
                }
            }
            Foe::Crab => {
                let shuffle = (self.anim * 10.0).sin() * 0.05;
                let shake = if self.st == St::Windup {
                    (self.anim * 40.0).sin() * 0.04
                } else {
                    0.0
                };
                r.mesh(
                    &a.bank,
                    &a.foes.crab[b],
                    &(at(0.0)
                        * Mat4::from_translation(Vec3::new(shake, shuffle.abs(), 0.0))
                        * rot
                        * sc(1.0, 1.0)),
                    &o,
                );
            }
            Foe::Wisp => {
                let spin = Mat4::from_rotation_y(self.anim * 2.0);
                let wo = DrawOpts {
                    mode: if self.flash > 0.0 {
                        Mode::Solid(WHITE)
                    } else {
                        Mode::Unlit
                    },
                    ..o
                };
                r.mesh(
                    &a.bank,
                    &a.foes.wisp[b],
                    &(at(self.y) * rot * spin * sc(1.0, 1.0)),
                    &wo,
                );
            }
            Foe::Beetle => {
                let shake = if self.st == St::Windup {
                    (self.anim * 40.0).sin() * 0.04
                } else {
                    0.0
                };
                r.mesh(
                    &a.bank,
                    &a.foes.beetle[b],
                    &(at(0.0)
                        * Mat4::from_translation(Vec3::new(shake, 0.0, 0.0))
                        * rot
                        * sc(1.0, 1.0)),
                    &o,
                );
            }
            Foe::Golem => {
                let sway = (self.anim * 2.0).sin() * 0.05;
                let lift = if self.st == St::Windup { 0.2 } else { 0.0 };
                r.mesh(
                    &a.bank,
                    &a.foes.golem[b],
                    &(at(lift) * rot * Mat4::from_rotation_z(sway) * sc(1.0, 1.0)),
                    &o,
                );
            }
            Foe::Ghost => {
                // A wisp of sheet trailing behind as it drifts.
                let m = at(self.y - 0.3) * rot * sc(1.0, 1.0);
                let tail = Vec2::new((self.anim * 3.0).sin(), (self.anim * 2.3).cos()) * 0.05
                    - self.dir * 0.08;
                let warp = Warp::Bend {
                    base: self.y - 0.3 + 0.4 * s,
                    h: -0.4 * s,
                    lean: tail * s,
                };
                let looks = &a.monsters;
                // (A skull ghost's face is its skull.)
                if !self.sewer {
                    r.mesh(&a.bank, &looks.ghost_face, &m, &o);
                }
                r.mesh(&a.bank, &looks.ghost_hat[b], &m, &o);
                let sheet = if self.flash > 0.0 {
                    o
                } else {
                    o.glass(0.28).with_glow(o.glow.max(0.8))
                };
                r.mesh(&a.bank, &looks.ghost[b], &m, &sheet.with_warp(warp));
            }
            Foe::Frog => {
                let (sx, sy) = if self.st == St::Hop {
                    (0.85, 1.2)
                } else {
                    (1.0 + self.squash * 0.25, 1.0 - self.squash * 0.3)
                };
                let m = at(self.y) * rot * sc(sx, sy);
                r.mesh(&a.bank, &a.foes.frog[b], &m, &o);
                // Legs trail out behind on a jump.
                let kick = if self.st == St::Hop { 0.5 } else { 0.0 };
                for side in [-1.0f32, 1.0] {
                    let leg = m
                        * Mat4::from_translation(Vec3::new(side * 0.2, 0.08, -0.12))
                        * Mat4::from_rotation_x(kick);
                    r.mesh(&a.bank, &a.foes.frog_leg[b], &leg, &o);
                }
            }
            Foe::Jelly => {
                // A glowing bell, and threads that stream behind it.
                let pulse = 1.0 + (self.anim * 3.0).sin() * 0.06;
                let m = at(self.y) * rot * sc(pulse, 1.0 / pulse);
                let trail = -self.dir * 0.12
                    + Vec2::new((self.anim * 2.3).sin(), (self.anim * 1.9).cos()) * 0.05;
                let warp = Warp::Bend {
                    base: self.y,
                    h: -0.45 * s,
                    lean: trail * s,
                };
                let lit = if self.flash > 0.0 {
                    o
                } else {
                    o.with_glow(o.glow.max(0.9))
                };
                r.mesh(&a.bank, &a.critters.jelly_core, &m, &lit.with_warp(warp));
                r.mesh(
                    &a.bank,
                    &a.critters.jelly_threads,
                    &m,
                    &lit.two_sided().with_warp(warp),
                );
                let bell = if self.flash > 0.0 {
                    o
                } else {
                    o.glass(0.3).with_glow(o.glow.max(0.8))
                };
                r.mesh(&a.bank, &a.critters.jelly_bell, &m, &bell);
                if self.st == St::Windup && (self.anim * 30.0).sin() > 0.0 {
                    r.halo(self.world_pos() + Vec3::Y * 0.1, 0.5 * s, CREAM, 0.7);
                }
            }
            Foe::Puffer if self.sewer => {
                // A cage of ribs round a glowing heart, bristling with bony spines, eyes
                // burning in their sockets.
                let puff = 1.0 + self.squash * 0.55;
                let m = at(self.y) * rot * sc(puff, puff);
                r.mesh(&a.bank, &a.foes.puffer[b], &m, &o.two_sided());
                r.mesh(&a.bank, &a.critters.puffer_spikes, &m, &o);
                let heart = m.transform_point3(Vec3::new(0.0, 0.0, 0.0));
                let beat = 0.3 + (self.anim * 5.0).sin().max(0.0) * 0.15;
                r.halo(heart, 0.22 * s * puff, LIME, beat);
                for sx in [-1.0f32, 1.0] {
                    let eye = m.transform_point3(Vec3::new(sx * 0.085, 0.07, 0.27));
                    r.halo(eye, 0.08 * s, RED, 0.35);
                }
            }
            Foe::Puffer => {
                // Blows itself up into a spiky ball before letting fly.
                let puff = 1.0 + self.squash * 0.55;
                let m = at(self.y) * rot * sc(puff, puff);
                r.mesh(&a.bank, &a.foes.puffer[b], &m, &o);
                if self.squash > 0.2 {
                    r.mesh(&a.bank, &a.critters.puffer_spikes, &m, &o);
                }
                let flap = (self.anim * 14.0).sin() * 0.5;
                for side in [-1.0f32, 1.0] {
                    let fin = m
                        * Mat4::from_translation(Vec3::new(side * 0.22, 0.0, 0.0))
                        * Mat4::from_rotation_y(side * (0.4 + flap));
                    r.mesh(&a.bank, &a.critters.puffer_fin, &fin, &o.two_sided());
                }
            }
            Foe::Bug => self.draw_bug(r, a, &o, b),
            Foe::Snail => self.draw_snail(r, a, &o, b),
            Foe::Bookworm => self.draw_worm(r, a, &o, b),
            Foe::Drake => self.draw_drake(r, a, &o),
            Foe::Leafling => self.draw_leafling(r, a, &o),
            Foe::Werewolf => self.draw_werewolf(r, a, &o),
            Foe::Imp | Foe::Skeleton | Foe::Zombie | Foe::Brute | Foe::Sneak => {
                let looks = &a.monsters;
                let h = match self.foe {
                    Foe::Imp => &a.critters.imp,
                    Foe::Skeleton => &looks.skeleton[b],
                    Foe::Zombie => &looks.zombie[b],
                    Foe::Brute => &looks.brute[b],
                    _ => &looks.sneak[b],
                };
                let held = match self.foe {
                    Foe::Brute => Some(&looks.club[b]),
                    Foe::Sneak => Some(&looks.dagger[b]),
                    _ => None,
                };
                let moving = if self.alert {
                    !(self.st == St::Windup || (self.foe == Foe::Brute && self.st == St::Rest))
                } else {
                    self.dir.length_squared() > 0.0
                };
                let pace = match self.foe {
                    Foe::Zombie => 6.0,
                    Foe::Brute => 7.5,
                    Foe::Sneak => 13.0,
                    _ => 10.0,
                };
                let swing = match (self.foe, self.st) {
                    (Foe::Zombie, _) => None,
                    // The club goes up, and comes down hard.
                    (Foe::Brute, St::Windup) => Some((0.0, super::draw::Swing::Chop)),
                    (Foe::Brute, St::Rest) if self.t > 0.3 => Some((
                        ((0.9 - self.t) / 0.15).clamp(0.0, 1.0),
                        super::draw::Swing::Chop,
                    )),
                    (Foe::Brute, _) => None,
                    (Foe::Sneak, St::Windup) => Some((0.0, super::draw::Swing::Slash)),
                    (Foe::Sneak, St::Dash) => Some((
                        1.0 - self.t.clamp(0.0, 0.28) / 0.28,
                        super::draw::Swing::Slash,
                    )),
                    (_, St::Windup | St::Dash) => Some((
                        1.0 - self.t.clamp(0.0, 0.4) / 0.4,
                        super::draw::Swing::Slash,
                    )),
                    _ => None,
                };
                let reach = match (self.foe, self.st) {
                    (Foe::Zombie, St::Windup) => 1.2 + (self.anim * 30.0).sin() * 0.06,
                    (Foe::Zombie, _) if self.alert => 1.0,
                    (Foe::Zombie, _) => 0.85,
                    _ => 0.0,
                };
                let stride = if moving { 0.8 } else { 0.0 };
                let pose = Pose {
                    walk: self.anim * pace,
                    stride,
                    swing,
                    bob: if self.foe == Foe::Brute {
                        (self.anim * pace).sin().abs() * 0.03 * stride
                    } else {
                        0.0
                    },
                    squash: self.flash * 2.0,
                    reach,
                    grow: s - 1.0,
                    look_up: 0.0,
                };
                let mut ho = o;
                if self.boss {
                    ho.glow = ho.glow.max(0.2);
                }
                // The dead lurch from side to side as they go.
                let lurch = if self.foe == Foe::Zombie {
                    (self.anim * pace * 0.5).sin() * 0.14
                } else {
                    0.0
                };
                let root = Vec3::new(self.pos.x, 0.0, self.pos.y);
                let fit = Outfit {
                    held,
                    ..Default::default()
                };
                draw_humanoid(r, a, h, root, self.yaw + lurch, &pose, &ho, &fit);
            }
        }
    }

    /// A drakeling, sat up on its haunches with its wings half open and its tail swishing.
    /// Drawing breath it rears back, wings up and mouth aglow; breathing, it leans into it
    /// with its wings beating.
    fn draw_drake(&self, r: &mut Renderer, a: &Assets, o: &DrawOpts) {
        use crate::assets::beast_art::{DRAKES, SNOUT_AT, TAIL_AT, WING_AT, drake_look};
        let s = self.scale();
        let look = drake_look(self.biome);
        let art = &a.beasts.drakes[look];
        let rearing = 1.0 - (self.t / BREATH_WINDUP).clamp(0.0, 1.0);
        let (lean, raise, beat) = match self.st {
            St::Windup => (-0.3 * rearing, 0.9 * rearing, 0.0),
            St::Dash => (0.18, 0.5, 1.0),
            _ => (0.0, 0.25, 0.0),
        };
        let walking = self.alert && self.st == St::Chase || !self.alert && self.dir != Vec2::ZERO;
        let hop = if walking {
            (self.anim * 9.0).sin().abs() * 0.04
        } else {
            0.0
        };
        let body = Mat4::from_translation(Vec3::new(self.pos.x, hop, self.pos.y))
            * Mat4::from_rotation_y(self.yaw)
            * Mat4::from_scale(Vec3::splat(s))
            * Mat4::from_rotation_x(lean);
        r.mesh(&a.bank, &art.body, &body, o);
        // Its wings, half open at rest and beating hard as it breathes.
        let wave = (self.anim * if beat > 0.0 { 22.0 } else { 6.0 }).sin();
        let open = raise + wave * (0.15 + beat * 0.3);
        let wo = o.two_sided();
        for side in [1.0f32, -1.0] {
            let m = body
                * Mat4::from_translation(Vec3::new(WING_AT.x * side, WING_AT.y, WING_AT.z))
                * Mat4::from_scale(Vec3::new(side, 1.0, 1.0))
                * Mat4::from_rotation_y(0.2)
                * Mat4::from_rotation_z(open);
            r.mesh(&a.bank, &art.wing, &m, &wo);
        }
        let swish = (self.anim * 2.5 + self.seed as f32).sin() * 0.35;
        let tail = body * Mat4::from_translation(TAIL_AT) * Mat4::from_rotation_y(swish);
        r.mesh(&a.bank, &art.tail, &tail, o);
        // Its mouth glowing brighter as it draws breath; the breath itself; and at rest,
        // a wisp of smoke (or frost) from its nostrils now and then.
        let snout = body.transform_point3(SNOUT_AT);
        let breath = DRAKES[look].breath;
        match self.st {
            St::Windup => {
                r.halo(
                    snout,
                    (0.12 + rearing * 0.2) * s,
                    breath[1],
                    0.3 + rearing * 0.5,
                );
                r.point(snout, 2, breath[0]);
            }
            St::Dash => {
                r.halo(snout, 0.3 * s, breath[1], 0.8);
                r.point(snout, 3, breath[0]);
            }
            _ => {
                let t = (self.anim * 0.7 + self.seed as f32 * 0.1).fract();
                if t < 0.5 {
                    r.point(snout + Vec3::Y * (t * 0.5 * s), 1, breath[2]);
                }
            }
        }
    }

    /// A leafling bobbing in the air, its leaf wings a blur, leaning back to throw.
    fn draw_leafling(&self, r: &mut Renderer, a: &Assets, o: &DrawOpts) {
        use crate::assets::beast_art::LEAF_WING_AT;
        let s = self.scale();
        let art = &a.beasts.leafling;
        let lean = if self.st == St::Windup { -0.35 } else { 0.12 };
        let m = Mat4::from_translation(Vec3::new(self.pos.x, self.y - 0.22 * s, self.pos.y))
            * Mat4::from_rotation_y(self.yaw)
            * Mat4::from_scale(Vec3::splat(s))
            * Mat4::from_rotation_x(lean);
        r.mesh(&a.bank, &art.body, &m, o);
        let flap = (self.anim * 30.0 + self.seed as f32).sin() * 0.6;
        let wo = o.two_sided();
        for side in [1.0f32, -1.0] {
            let w =
                m * Mat4::from_translation(Vec3::new(
                    LEAF_WING_AT.x * side,
                    LEAF_WING_AT.y,
                    LEAF_WING_AT.z,
                )) * Mat4::from_scale(Vec3::new(side, 1.0, 1.0))
                    * Mat4::from_rotation_y(0.5 + flap);
            r.mesh(&a.bank, &art.wing, &w, &wo);
        }
    }

    /// A werewolf loping along on its long legs, its tail streaming behind. Howling, it
    /// throws its head back; about to pounce it crouches, and its claws come round.
    fn draw_werewolf(&self, r: &mut Renderer, a: &Assets, o: &DrawOpts) {
        let s = self.scale();
        let h = &a.monsters.werewolf;
        let moving = if self.alert {
            matches!(self.st, St::Chase | St::Idle | St::Dash)
        } else {
            self.dir.length_squared() > 0.0
        };
        let pace = if self.st == St::Dash { 16.0 } else { 12.0 };
        let swing = match self.st {
            St::Windup => Some((0.0, Swing::Slash)),
            St::Dash => Some((1.0 - self.t.clamp(0.0, 0.32) / 0.32, Swing::Slash)),
            _ => None,
        };
        // Head back for the howl, held, and down again.
        let howl = if self.st == St::Howl {
            let k = 1.0 - (self.t / HOWL_SECS).clamp(0.0, 1.0);
            ((k * PI).sin() / 0.8).min(1.0)
        } else {
            0.0
        };
        let crouch = if self.st == St::Windup { 0.35 } else { 0.0 };
        let stride = if moving { 1.0 } else { 0.0 };
        let pose = Pose {
            walk: self.anim * pace,
            stride,
            swing,
            bob: (self.anim * pace).sin().abs() * 0.05 * stride,
            squash: self.flash * 2.0 + crouch,
            // Claws out in front, ready.
            reach: if self.st == St::Howl { 0.0 } else { 0.25 },
            grow: s - 1.0,
            look_up: howl * 0.9,
        };
        let root = Vec3::new(self.pos.x, 0.0, self.pos.y);
        draw_humanoid(r, a, h, root, self.yaw, &pose, o, &Outfit::default());
        // The tail, from the small of its back: streaming out behind as it runs, up for
        // the howl, and wagging.
        let wag = (self.anim * if moving { 9.0 } else { 3.0 }).sin() * 0.3;
        let (squash, widen) = (1.0 - pose.squash * 0.25, 1.0 + pose.squash * 0.2);
        let m = Mat4::from_translation(root)
            * Mat4::from_rotation_y(self.yaw)
            * Mat4::from_scale(Vec3::splat(1.0 + pose.grow))
            * Mat4::from_scale(Vec3::new(widen, squash, widen))
            * Mat4::from_translation(Vec3::new(0.0, h.hip + 0.06, -0.15))
            * Mat4::from_rotation_y(wag)
            * Mat4::from_rotation_x(stride * 0.5 + howl * 0.9);
        r.mesh(&a.bank, &a.monsters.wolf_tail, &m, o);
    }

    /// A bug: its body, jointed legs on both sides scuttling in step, and wings or claws
    /// for those that have them.
    /// A lantern snail: soft body, waggling eye stalks and a shell of stained glass that
    /// glows like a lantern, over a trail of shining slime. Struck, it pulls itself in.
    fn draw_snail(&self, r: &mut Renderer, a: &Assets, o: &DrawOpts, b: usize) {
        use crate::assets::deep_art::{SHELL_AT, STALK_AT};
        let s = self.scale();
        let art = &a.deep.snails[b];
        // Its slime, sparkling where it's fresh.
        for (k, &(p, age)) in self.trail.iter().enumerate() {
            let fade = 1.0 - age / TRAIL_SECS;
            let c = art.glass[k % 3];
            let at = Vec3::new(p.x, 0.02, p.y);
            if fade > 0.35 || (k % 2 == 0 && fade > 0.1) {
                r.point(at, 1, c);
            }
            if k % 3 == 0 && fade > 0.3 {
                r.halo(at, 0.16 * s, c, 0.22 * fade);
            }
        }
        // How far into its shell it has pulled (a quick tuck, a slow peek out).
        let tuck = (self.hide / 0.3).min(1.0);
        let lunge = match self.st {
            St::Windup => -0.07 * (1.0 - self.t / 0.55).clamp(0.0, 1.0),
            St::Dash => 0.09,
            _ => 0.0,
        };
        let ripple = if self.alert || self.dir.length_squared() > 0.0 {
            (self.anim * 3.2).sin() * 0.04
        } else {
            0.0
        };
        let m = Mat4::from_translation(Vec3::new(self.pos.x, 0.0, self.pos.y))
            * Mat4::from_rotation_y(self.yaw)
            * Mat4::from_scale(Vec3::splat(s));
        if tuck < 0.98 {
            let out = 1.0 - tuck;
            let body = m
                * Mat4::from_translation(Vec3::new(0.0, 0.0, lunge * out + SHELL_AT.z * tuck))
                * Mat4::from_scale(Vec3::new(
                    0.5 + 0.5 * out,
                    0.4 + 0.6 * out,
                    (1.0 + ripple) * (0.3 + 0.7 * out),
                ));
            r.mesh(&a.bank, &art.body, &body, o);
            for side in [-1.0f32, 1.0] {
                let wag = (self.anim * 2.3 + side * 1.3).sin() * 0.22;
                let perk = if self.st == St::Windup { 0.35 } else { 0.0 };
                let stalk = body
                    * Mat4::from_translation(Vec3::new(side * STALK_AT.x, STALK_AT.y, STALK_AT.z))
                    * Mat4::from_rotation_z(-side * (0.3 + wag * 0.5))
                    * Mat4::from_rotation_x(0.25 + wag * 0.3 - perk)
                    * Mat4::from_scale(Vec3::new(1.0, out, 1.0));
                r.mesh(&a.bank, &art.stalk, &stalk, o);
            }
        }
        // The shell, lit from within; it rattles as the snail hides.
        let rattle = if self.shelled() {
            (self.anim * 34.0).sin() * 0.06 * (self.hide / HIDE_SECS)
        } else {
            0.0
        };
        let shell = m
            * Mat4::from_translation(SHELL_AT - Vec3::Y * 0.035 * tuck)
            * Mat4::from_rotation_z(rattle);
        let glass = if self.flash > 0.0 {
            *o
        } else {
            o.with_mode(Mode::Unlit)
        };
        r.mesh(&a.bank, &art.shell, &shell, &glass);
        let pulse = (self.anim * 2.0).sin() * 0.5 + 0.5;
        let c = shell.transform_point3(Vec3::ZERO);
        r.halo(
            c,
            (0.4 + pulse * 0.06 + tuck * 0.12) * s,
            art.glass[0],
            0.26 + pulse * 0.1 + tuck * 0.2,
        );
    }

    /// A book-worm bibliomancer: a caterpillar in spectacles rippling along behind its head,
    /// its little book floating open beside it. It rears back to spit.
    fn draw_worm(&self, r: &mut Renderer, a: &Assets, o: &DrawOpts, b: usize) {
        use crate::assets::deep_art::{HEAD_AT, SEG_GAP, SEG_R, SEGMENTS};
        let s = self.scale();
        let art = &a.deep.worms[b];
        let m = Mat4::from_translation(Vec3::new(self.pos.x, 0.0, self.pos.y))
            * Mat4::from_rotation_y(self.yaw)
            * Mat4::from_scale(Vec3::splat(s));
        let moving = self.alert || self.dir.length_squared() > 0.0;
        let rear = if self.st == St::Windup {
            (1.0 - self.t / 0.65).clamp(0.0, 1.0)
        } else {
            0.0
        };
        // Humps travel down the body from head to tail.
        for i in 1..=SEGMENTS {
            let k = i as f32;
            let hump = if moving {
                (self.anim * 9.0 - k * 1.2).sin().max(0.0) * 0.05
            } else {
                ((self.anim * 1.6 - k).sin() * 0.5 + 0.5) * 0.012
            };
            let lift = if i == 1 { rear * 0.06 } else { 0.0 };
            let size = 1.0 - k * 0.05;
            let at =
                m * Mat4::from_translation(Vec3::new(
                    0.0,
                    SEG_R * size + hump + lift,
                    -k * SEG_GAP + 0.02,
                )) * Mat4::from_scale(Vec3::splat(size));
            r.mesh(&a.bank, &art.segs[i % 2], &at, o);
        }
        let nod = if moving {
            (self.anim * 9.0).sin() * 0.06
        } else {
            (self.anim * 1.4).sin() * 0.08
        };
        let head = m
            * Mat4::from_translation(HEAD_AT + Vec3::new(0.0, rear * 0.14, 0.04 - rear * 0.05))
            * Mat4::from_rotation_x(-rear * 0.55 + nod);
        r.mesh(&a.bank, &art.head, &head, o);
        // The book, open, bobbing at its side; its pages shine as the ink gathers.
        let bob = (self.anim * 2.2).sin() * 0.03;
        let book = m
            * Mat4::from_translation(Vec3::new(0.24, 0.36 + bob + rear * 0.08, -0.02))
            * Mat4::from_rotation_y(-0.6)
            * Mat4::from_rotation_x(0.45);
        r.mesh(&a.bank, &art.book, &book, o);
        let pages = if rear > 0.0 && self.flash <= 0.0 {
            o.with_mode(Mode::Unlit)
        } else {
            *o
        };
        r.mesh(&a.bank, &art.pages, &book, &pages);
        if rear > 0.0 {
            let c = book.transform_point3(Vec3::Y * 0.03);
            r.halo(c, (0.2 + rear * 0.15) * s, art.ink[1], 0.35 + rear * 0.3);
            for k in 0..3 {
                let t = (self.anim * 1.5 + k as f32 * 0.33).fract();
                let ang = self.anim * 2.0 + k as f32 * 2.1;
                let q = c + Vec3::new(ang.cos() * 0.08, t * 0.25, ang.sin() * 0.08);
                r.point(q, 1, if k == 0 { art.ink[0] } else { art.ink[1] });
            }
            // A bead of ink swelling at its lips.
            let lips = head.transform_point3(Vec3::new(0.0, -0.05, 0.13));
            r.point(lips, 1 + (rear * 2.0) as i32, art.ink[1]);
        }
    }

    fn draw_bug(&self, r: &mut Renderer, a: &Assets, o: &DrawOpts, b: usize) {
        let s = self.scale();
        let bug = &a.monsters.bug[b];
        let moving = self.alert || self.dir.length_squared() > 0.0;
        let pace = if self.st == St::Dash { 30.0 } else { 18.0 };
        let bob = if bug.flies {
            0.0
        } else {
            (self.anim * pace).sin().abs() * 0.012 * s
        };
        let shake = if self.st == St::Windup {
            (self.anim * 40.0).sin() * 0.03
        } else {
            0.0
        };
        let m = Mat4::from_translation(Vec3::new(self.pos.x, self.y + bob, self.pos.y))
            * Mat4::from_rotation_y(self.yaw)
            * Mat4::from_translation(Vec3::new(shake, 0.0, 0.0))
            * Mat4::from_scale(Vec3::splat(s));
        r.mesh(&a.bank, &bug.body, &m, o);
        let limbs = o.two_sided();
        // Each leg is a thigh rising from the hip to a knee and a shin reaching from there
        // down to the floor (or dangling, for fliers). They step in two alternating sets,
        // a lifted foot swinging forwards while the planted ones push back.
        for (k, &(z, splay)) in bug.legs.iter().enumerate() {
            for side in [-1.0f32, 1.0] {
                let set = (k + usize::from(side > 0.0)) % 2;
                let ph = self.anim * pace + set as f32 * PI;
                let (swing, lift) = if bug.flies {
                    (0.0, (self.anim * 3.0 + k as f32).sin() * 0.1)
                } else if moving {
                    (ph.sin() * 0.3, ph.cos().max(0.0) * 0.3)
                } else {
                    (0.0, 0.0)
                };
                let hip = m
                    * Mat4::from_translation(Vec3::new(side * bug.hip_x, bug.hip_y, z))
                    * Mat4::from_scale(Vec3::new(side, 1.0, 1.0))
                    * Mat4::from_rotation_y(-(splay + swing));
                let up = bug.knee + lift;
                r.mesh(
                    &a.bank,
                    &bug.thigh,
                    &(hip * Mat4::from_rotation_z(up)),
                    &limbs,
                );
                let knee = Vec3::new(bug.thigh_len * up.cos(), bug.thigh_len * up.sin(), 0.0);
                // Steep enough that the foot lands on the floor (a lifted one just above it).
                let down = if bug.flies {
                    1.3
                } else {
                    ((bug.hip_y + knee.y - lift * 0.15) / bug.shin_len)
                        .clamp(0.0, 1.0)
                        .asin()
                };
                let shin = hip * Mat4::from_translation(knee) * Mat4::from_rotation_z(-down);
                r.mesh(&a.bank, &bug.shin, &shin, &limbs);
            }
        }
        if let Some(w) = &bug.wing {
            let flap = (self.anim * 22.0).sin() * 0.9;
            for side in [-1.0f32, 1.0] {
                let wm = m
                    * Mat4::from_translation(Vec3::new(side * 0.05, 0.12, 0.0))
                    * Mat4::from_scale(Vec3::new(side, 1.0, 1.0))
                    * Mat4::from_rotation_z(0.25 + flap);
                r.mesh(&a.bank, w, &wm, &limbs);
            }
        }
        if let Some(c) = &bug.claw {
            // Folded in prayer, raised to strike, then brought down.
            let strike = match self.st {
                St::Windup => -0.8,
                St::Dash => 1.1,
                _ => 0.5 + (self.anim * 2.0).sin() * 0.1,
            };
            for side in [-1.0f32, 1.0] {
                let cm = m
                    * Mat4::from_translation(Vec3::new(side * 0.07, 0.34, 0.12))
                    * Mat4::from_rotation_x(strike);
                r.mesh(&a.bank, c, &cm, o);
            }
        }
    }
}
