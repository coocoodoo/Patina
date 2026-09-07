//! Tween helpers: exponential easing by default, or spring physics with overshoot.

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Tween {
    pub value: f32,
    pub target: f32,
    /// Spring velocity in units per second (unused by the exponential step).
    pub velocity: f32,
}

/// How a tween approaches its target.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Motion {
    /// Exponential ease-out: quick start, smooth stop, never overshoots.
    Ease,
    /// A damped spring on a unit mass; it may overshoot and settle.
    Spring { stiffness: f32, damping: f32 },
}

impl Motion {
    /// A gentle spring with a small overshoot.
    pub const SPRING: Motion = Motion::Spring {
        stiffness: 200.0,
        damping: 18.0,
    };
    /// A lively spring that visibly bounces before it settles.
    pub const BOUNCY: Motion = Motion::Spring {
        stiffness: 300.0,
        damping: 10.0,
    };

    /// ABI encoding: 0 ease, 1 spring, 2 bouncy.
    pub fn from_style(style: u32) -> Motion {
        match style {
            1 => Motion::SPRING,
            2 => Motion::BOUNCY,
            _ => Motion::Ease,
        }
    }

    pub fn style(self) -> u32 {
        if self == Motion::Ease {
            0
        } else if self == Motion::BOUNCY {
            2
        } else {
            1
        }
    }
}

impl Tween {
    pub const fn at(v: f32) -> Tween {
        Tween {
            value: v,
            target: v,
            velocity: 0.0,
        }
    }

    pub fn set(&mut self, target: f32) {
        self.target = target;
    }

    pub fn snap(&mut self, v: f32) {
        self.value = v;
        self.target = v;
        self.velocity = 0.0;
    }

    pub fn active(&self) -> bool {
        (self.target - self.value).abs() > 0.0005 || self.velocity.abs() > 0.01
    }

    /// Exponential approach; `speed` is roughly 1/seconds to settle. Returns true while moving.
    pub fn step(&mut self, dt: f32, speed: f32) -> bool {
        self.velocity = 0.0;
        if !self.active() {
            self.value = self.target;
            return false;
        }
        let k = 1.0 - (-dt * speed).exp();
        self.value += (self.target - self.value) * k;
        if (self.target - self.value).abs() < 0.002 {
            self.value = self.target;
            return false;
        }
        true
    }

    /// Damped spring integration (semi-implicit Euler with small sub-steps, so it stays
    /// stable at any frame rate). Returns true while moving.
    pub fn step_spring(&mut self, dt: f32, stiffness: f32, damping: f32) -> bool {
        if !self.active() {
            self.value = self.target;
            self.velocity = 0.0;
            return false;
        }
        let steps = ((dt / 0.004).ceil() as usize).clamp(1, 64);
        let h = dt / steps as f32;
        for _ in 0..steps {
            let a = -stiffness * (self.value - self.target) - damping * self.velocity;
            self.velocity += a * h;
            self.value += self.velocity * h;
        }
        if (self.value - self.target).abs() < 0.001 && self.velocity.abs() < 0.02 {
            self.value = self.target;
            self.velocity = 0.0;
            return false;
        }
        true
    }

    /// Advance with the given motion style.
    pub fn advance(&mut self, dt: f32, ease_speed: f32, motion: Motion) -> bool {
        match motion {
            Motion::Ease => self.step(dt, ease_speed),
            Motion::Spring { stiffness, damping } => self.step_spring(dt, stiffness, damping),
        }
    }
}

pub fn ease_out_cubic(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    1.0 - (1.0 - t).powi(3)
}

pub fn ease_in_out(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    if t < 0.5 {
        4.0 * t * t * t
    } else {
        1.0 - (-2.0 * t + 2.0).powi(3) / 2.0
    }
}
