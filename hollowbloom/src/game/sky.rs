//! The sky: where the sun and the moon are through the day, which way their shadows fall,
//! and the moon's phase, which the creatures of the Hollow feel even underground.

use glam::Vec3;

use crate::render::light::KEY;

/// The moon's eight phases, one a day.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MoonPhase {
    New,
    WaxingCrescent,
    FirstQuarter,
    WaxingGibbous,
    Full,
    WaningGibbous,
    LastQuarter,
    WaningCrescent,
}

pub const PHASES: [MoonPhase; 8] = [
    MoonPhase::New,
    MoonPhase::WaxingCrescent,
    MoonPhase::FirstQuarter,
    MoonPhase::WaxingGibbous,
    MoonPhase::Full,
    MoonPhase::WaningGibbous,
    MoonPhase::LastQuarter,
    MoonPhase::WaningCrescent,
];

impl MoonPhase {
    /// Tonight's moon: the cycle starts dark on day 1 and is full every eighth day from day 5.
    pub fn of_day(day: u32) -> MoonPhase {
        PHASES[((day.max(1) - 1) % 8) as usize]
    }

    pub fn name(self) -> &'static str {
        match self {
            MoonPhase::New => "New Moon",
            MoonPhase::WaxingCrescent => "Waxing Crescent",
            MoonPhase::FirstQuarter => "First Quarter",
            MoonPhase::WaxingGibbous => "Waxing Gibbous",
            MoonPhase::Full => "Full Moon",
            MoonPhase::WaningGibbous => "Waning Gibbous",
            MoonPhase::LastQuarter => "Last Quarter",
            MoonPhase::WaningCrescent => "Waning Crescent",
        }
    }

    /// How much of the moon is lit, 0 (new) to 1 (full).
    pub fn light(self) -> f32 {
        [0.0, 0.25, 0.5, 0.75, 1.0, 0.75, 0.5, 0.25][self as usize]
    }

    /// How riled up the creatures of the Hollow are: sleepy under a new moon, fierce under
    /// a full one. Scales how far they notice you, how fast they move and how often they
    /// strike.
    pub fn fury(self) -> f32 {
        [0.8, 0.88, 0.96, 1.1, 1.3, 1.1, 0.96, 0.88][self as usize]
    }

    pub fn full(self) -> bool {
        self == MoonPhase::Full
    }

    /// Waxing moons are lit from the right (and waning ones from the left).
    pub fn waxing(self) -> bool {
        (1..4).contains(&(self as usize))
    }

    /// A word on what it means below, for the morning summary ("New Moon: ...").
    pub fn omen(self) -> &'static str {
        match self {
            MoonPhase::New => "the Hollow sleeps.",
            MoonPhase::WaxingCrescent | MoonPhase::WaningCrescent => "the Hollow is quiet.",
            MoonPhase::FirstQuarter | MoonPhase::LastQuarter => "the Hollow stirs.",
            MoonPhase::WaxingGibbous | MoonPhase::WaningGibbous => "the Hollow grows restless.",
            MoonPhase::Full => "monsters glow red, hit harder and drop more!",
        }
    }
}

/// Sunrise and sunset, in minutes after midnight.
pub const SUNRISE: f32 = 330.0;
pub const SUNSET: f32 = 1190.0;
/// The moon rises at dusk and is up for twelve hours.
pub const MOONRISE: f32 = 1160.0;
const MOON_UP: f32 = 720.0;

/// The light in the sky right now.
#[derive(Clone, Copy, Debug)]
pub struct SkyLight {
    /// The way its light travels (downwards).
    pub dir: Vec3,
    /// How strongly shadows show, 0..1.
    pub shadow: f32,
    /// How much it is the sun (for the key light), 0..1.
    pub sun: f32,
}

impl SkyLight {
    /// Where the key light comes from: the sun swings it round through the day.
    pub fn key(&self) -> Vec3 {
        (KEY + (-self.dir) * (1.2 * self.sun)).normalize()
    }
}

fn smooth(a: f32, b: f32, x: f32) -> f32 {
    let t = ((x - a) / (b - a)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// A body crossing the sky from the east (t = 0) through the south to the west (t = 1),
/// `low` degrees up at either end and `high` at its peak: the way its light travels.
fn arc(t: f32, low: f32, high: f32) -> Vec3 {
    let a = std::f32::consts::PI * t;
    let e = (low + (high - low) * a.sin()).to_radians();
    let to = Vec3::new(a.cos() * e.cos(), e.sin(), a.sin() * e.cos());
    -to
}

/// The sun by day, the moon by night (as bright as its phase), both dimmed by rain.
pub fn sky_light(min: f32, phase: MoonPhase, rain: bool) -> SkyLight {
    let damp = if rain { 0.25 } else { 1.0 };
    let t = (min - SUNRISE) / (SUNSET - SUNRISE);
    let day = if (0.0..=1.0).contains(&t) {
        smooth(0.0, 0.05, t) * (1.0 - smooth(0.93, 1.0, t))
    } else {
        0.0
    };
    let tm = (min - MOONRISE) / MOON_UP;
    let night = if (0.0..=1.0).contains(&tm) {
        phase.light() * 0.7 * smooth(0.0, 0.06, tm)
    } else {
        0.0
    };
    if day >= night {
        SkyLight {
            dir: arc(t.clamp(0.0, 1.0), 12.0, 62.0),
            shadow: day * damp,
            sun: day,
        }
    } else {
        SkyLight {
            dir: arc(tm, 18.0, 58.0),
            shadow: night * damp,
            sun: 0.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_sun_rises_in_the_east_and_sets_in_the_west() {
        let morning = sky_light(420.0, MoonPhase::New, false);
        let noon = sky_light(760.0, MoonPhase::New, false);
        let evening = sky_light(1120.0, MoonPhase::New, false);
        // Light travels west in the morning, east in the evening; shadows point the same way.
        assert!(morning.dir.x < -0.5 && evening.dir.x > 0.5);
        // Noon shadows are short (the sun is high), dawn ones long.
        assert!(-noon.dir.y > -morning.dir.y);
        assert!(noon.shadow > 0.9 && morning.shadow > 0.5);
        // Rain softens them.
        assert!(sky_light(760.0, MoonPhase::New, true).shadow < 0.3);
    }

    #[test]
    fn a_full_moon_casts_shadows_and_a_new_one_does_not() {
        let full = sky_light(1320.0, MoonPhase::Full, false);
        let new = sky_light(1320.0, MoonPhase::New, false);
        assert!(full.shadow > 0.5 && full.sun == 0.0);
        assert_eq!(new.shadow, 0.0);
        assert_eq!(MoonPhase::of_day(5), MoonPhase::Full);
        assert_eq!(MoonPhase::of_day(13), MoonPhase::Full);
        assert_eq!(MoonPhase::of_day(1), MoonPhase::New);
        assert!(MoonPhase::Full.fury() > MoonPhase::New.fury());
    }
}
