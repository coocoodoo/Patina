//! Procedural sound effects, rendered once at start-up into sample buffers.

use crate::util::Rng;

pub const RATE: f32 = 44100.0;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum Sfx {
    Swing,
    Hit,
    EnemyHurt,
    EnemyDie,
    PlayerHurt,
    Pickup,
    Coin,
    Till,
    Water,
    Refill,
    Chop,
    Mine,
    Break,
    Place,
    Plant,
    Harvest,
    UiMove,
    UiSelect,
    UiBack,
    Craft,
    Stairs,
    LevelUp,
    Step,
    Hop,
    Eat,
    Sleep,
    Waystone,
    Dodge,
    Shoot,
    Chest,
    Denied,
    Alert,
    Rare,
    Equip,
    Magic,
    Blast,
    Enchant,
    Block,
    Door,
    BusHorn,
    /// A quest finished.
    Fanfare,
    /// A quest taken on.
    Accept,
    /// One syllable of a villager's chatter.
    Talk,
    /// Friendship growing.
    Heart,
    Bell,
    /// Metal on something hard, with a crackle of sparks.
    Clang,
    /// Drinking a potion.
    Gulp,
    /// A potion bubbling up in the cauldron.
    Brew,
    /// A spell going off.
    Spell,
    /// A spell growing stronger.
    SpellUp,
    /// A line whipping out over the water.
    Cast,
    /// A bobber landing, or a fish nibbling.
    Plop,
    /// Something bites!
    Bite,
    /// Clicking in the reel.
    Reel,
    /// A fish landed.
    Catch,
    /// Something frying on the stove.
    Sizzle,
    /// A little tune on the piano.
    Piano,
}

pub const ALL: &[Sfx] = &[
    Sfx::Swing,
    Sfx::Hit,
    Sfx::EnemyHurt,
    Sfx::EnemyDie,
    Sfx::PlayerHurt,
    Sfx::Pickup,
    Sfx::Coin,
    Sfx::Till,
    Sfx::Water,
    Sfx::Refill,
    Sfx::Chop,
    Sfx::Mine,
    Sfx::Break,
    Sfx::Place,
    Sfx::Plant,
    Sfx::Harvest,
    Sfx::UiMove,
    Sfx::UiSelect,
    Sfx::UiBack,
    Sfx::Craft,
    Sfx::Stairs,
    Sfx::LevelUp,
    Sfx::Step,
    Sfx::Hop,
    Sfx::Eat,
    Sfx::Sleep,
    Sfx::Waystone,
    Sfx::Dodge,
    Sfx::Shoot,
    Sfx::Chest,
    Sfx::Denied,
    Sfx::Alert,
    Sfx::Rare,
    Sfx::Equip,
    Sfx::Magic,
    Sfx::Blast,
    Sfx::Enchant,
    Sfx::Block,
    Sfx::Door,
    Sfx::BusHorn,
    Sfx::Fanfare,
    Sfx::Accept,
    Sfx::Talk,
    Sfx::Heart,
    Sfx::Bell,
    Sfx::Clang,
    Sfx::Gulp,
    Sfx::Brew,
    Sfx::Spell,
    Sfx::SpellUp,
    Sfx::Cast,
    Sfx::Plop,
    Sfx::Bite,
    Sfx::Reel,
    Sfx::Catch,
    Sfx::Sizzle,
    Sfx::Piano,
];

fn square(phase: f32, duty: f32) -> f32 {
    if phase.fract() < duty { 1.0 } else { -1.0 }
}

fn tri(phase: f32) -> f32 {
    let p = phase.fract();
    if p < 0.5 {
        4.0 * p - 1.0
    } else {
        3.0 - 4.0 * p
    }
}

fn sine(phase: f32) -> f32 {
    (phase * std::f32::consts::TAU).sin()
}

/// Renders `secs` of audio with a per-sample generator `f(t, noise)`.
fn render(secs: f32, mut f: impl FnMut(f32, f32) -> f32) -> Vec<f32> {
    let n = (secs * RATE) as usize;
    let mut out = Vec::with_capacity(n);
    let mut rng = Rng::new(n as u64);
    for i in 0..n {
        let t = i as f32 / RATE;
        let noise = rng.f32() * 2.0 - 1.0;
        out.push(f(t, noise));
    }
    // Soften the edges.
    let fade = (RATE * 0.004) as usize;
    for i in 0..fade.min(out.len()) {
        let k = i as f32 / fade as f32;
        out[i] *= k;
        let j = out.len() - 1 - i;
        out[j] *= k;
    }
    out
}

/// A tone sweep: frequency moves from `f0` to `f1`, amplitude decays exponentially.
fn sweep(secs: f32, f0: f32, f1: f32, decay: f32, wave: impl Fn(f32) -> f32) -> Vec<f32> {
    let mut phase = 0.0;
    render(secs, |t, _| {
        let k = t / secs;
        let f = f0 + (f1 - f0) * k;
        phase += f / RATE;
        wave(phase) * (-t * decay).exp()
    })
}

/// A sequence of notes (frequency, length) with a plucky envelope.
fn arp(notes: &[(f32, f32)], duty: f32, decay: f32) -> Vec<f32> {
    let mut out = Vec::new();
    for &(f, len) in notes {
        let mut phase = 0.0;
        out.extend(render(len, |t, _| {
            phase += f / RATE;
            (square(phase, duty) * 0.6 + sine(phase) * 0.4) * (-t * decay).exp()
        }));
    }
    out
}

/// A simple one-pole low-pass over a buffer.
fn lowpass(buf: &mut [f32], k: f32) {
    let mut y = 0.0;
    for s in buf.iter_mut() {
        y += (*s - y) * k;
        *s = y;
    }
}

fn note(n: i32) -> f32 {
    440.0 * 2f32.powf((n - 69) as f32 / 12.0)
}

pub fn make(s: Sfx) -> Vec<f32> {
    match s {
        Sfx::Swing | Sfx::Dodge => {
            let len = if s == Sfx::Swing { 0.14 } else { 0.2 };
            let mut lp = 0.0;
            render(len, |t, n| {
                let k = 0.05 + 0.5 * (1.0 - t / len);
                lp += (n - lp) * k;
                lp * (1.0 - t / len) * 0.9
            })
        }
        Sfx::Hit => {
            let mut phase = 0.0;
            render(0.12, |t, n| {
                phase += (160.0 - t * 700.0).max(50.0) / RATE;
                (square(phase, 0.5) * 0.5 + n * 0.5) * (-t * 28.0).exp()
            })
        }
        Sfx::EnemyHurt => sweep(0.12, 700.0, 280.0, 18.0, |p| square(p, 0.25) * 0.7),
        Sfx::EnemyDie => {
            let mut b = arp(
                &[
                    (note(76), 0.05),
                    (note(72), 0.05),
                    (note(67), 0.05),
                    (note(60), 0.12),
                ],
                0.25,
                14.0,
            );
            let puff = render(0.25, |t, n| n * 0.4 * (-t * 12.0).exp());
            for (i, p) in puff.iter().enumerate() {
                if i < b.len() {
                    b[i] += p;
                }
            }
            b
        }
        Sfx::PlayerHurt => {
            let mut phase = 0.0;
            render(0.28, |t, _| {
                let f = 330.0 - t * 500.0 + (t * 60.0).sin() * 20.0;
                phase += f / RATE;
                square(phase, 0.5) * 0.6 * (-t * 7.0).exp()
            })
        }
        Sfx::Pickup => arp(&[(note(79), 0.05), (note(86), 0.09)], 0.5, 18.0),
        Sfx::Coin => arp(&[(note(83), 0.06), (note(88), 0.22)], 0.5, 9.0),
        Sfx::Till | Sfx::Place => {
            let mut phase = 0.0;
            let mut b = render(0.12, |t, n| {
                phase += (120.0 - t * 400.0).max(40.0) / RATE;
                (sine(phase) * 0.8 + n * 0.25) * (-t * 26.0).exp()
            });
            lowpass(&mut b, 0.3);
            b
        }
        Sfx::Water | Sfx::Refill => {
            let len = if s == Sfx::Water { 0.35 } else { 0.5 };
            let mut phase = 0.0;
            let mut b = render(len, |t, n| {
                let bubble = ((t * 30.0).sin() * 0.5 + 0.5) * 900.0 + 400.0;
                phase += bubble / RATE;
                (n * 0.5 + sine(phase) * 0.25) * (1.0 - t / len)
            });
            lowpass(&mut b, 0.25);
            b
        }
        Sfx::Chop => {
            let mut phase = 0.0;
            render(0.12, |t, n| {
                phase += 230.0 / RATE;
                (sine(phase) * 0.7 + n * 0.3) * (-t * 30.0).exp()
            })
        }
        Sfx::Mine => {
            let (mut a, mut b) = (0.0, 0.0);
            render(0.16, |t, n| {
                a += 1650.0 / RATE;
                b += 2470.0 / RATE;
                (sine(a) * 0.5 + sine(b) * 0.3 + n * 0.4 * (-t * 80.0).exp()) * (-t * 22.0).exp()
            })
        }
        Sfx::Break => {
            let mut b = render(0.3, |t, n| n * (-t * 10.0).exp() * 0.8);
            lowpass(&mut b, 0.35);
            b
        }
        Sfx::Plant => sweep(0.08, 500.0, 900.0, 20.0, sine),
        Sfx::Harvest => {
            let mut b = sweep(0.08, 400.0, 900.0, 10.0, sine);
            b.extend(arp(&[(note(84), 0.06), (note(91), 0.14)], 0.5, 12.0));
            b
        }
        Sfx::UiMove => sweep(0.03, 1200.0, 1200.0, 60.0, |p| square(p, 0.5) * 0.4),
        Sfx::UiSelect => sweep(0.08, 660.0, 990.0, 18.0, |p| square(p, 0.5) * 0.5),
        Sfx::UiBack => sweep(0.08, 660.0, 440.0, 18.0, |p| square(p, 0.5) * 0.5),
        Sfx::Craft => arp(
            &[
                (note(72), 0.06),
                (note(76), 0.06),
                (note(79), 0.06),
                (note(84), 0.2),
            ],
            0.25,
            8.0,
        ),
        Sfx::Stairs => arp(
            &[
                (note(79), 0.09),
                (note(76), 0.09),
                (note(72), 0.09),
                (note(67), 0.25),
            ],
            0.5,
            6.0,
        ),
        Sfx::LevelUp => arp(
            &[
                (note(72), 0.08),
                (note(76), 0.08),
                (note(79), 0.08),
                (note(84), 0.08),
                (note(88), 0.45),
            ],
            0.25,
            4.0,
        ),
        Sfx::Step => {
            let mut b = render(0.03, |t, n| n * 0.25 * (-t * 90.0).exp());
            lowpass(&mut b, 0.2);
            b
        }
        Sfx::Hop => sweep(0.1, 220.0, 440.0, 14.0, |p| sine(p) * 0.8),
        Sfx::Eat => {
            let mut b = Vec::new();
            for _ in 0..3 {
                let mut c = render(0.07, |t, n| n * 0.5 * (-t * 40.0).exp());
                lowpass(&mut c, 0.4);
                b.extend(c);
                b.extend(std::iter::repeat_n(0.0, (RATE * 0.04) as usize));
            }
            b
        }
        Sfx::Sleep => {
            let notes = [72, 76, 79, 83, 79, 76];
            let mut b = Vec::new();
            for n in notes {
                let f = note(n);
                let mut phase = 0.0;
                b.extend(render(0.2, |t, _| {
                    phase += f / RATE;
                    tri(phase) * 0.5 * (-t * 3.0).exp()
                }));
            }
            b
        }
        Sfx::Waystone => {
            let (mut a, mut b2) = (0.0, 0.0);
            render(1.0, |t, _| {
                a += (440.0 + t * 440.0) / RATE;
                b2 += (443.0 + t * 450.0) / RATE;
                (sine(a) + sine(b2)) * 0.3 * (1.0 - t) * (t * 6.0).min(1.0)
            })
        }
        Sfx::Shoot => sweep(0.15, 900.0, 300.0, 12.0, |p| square(p, 0.25) * 0.5),
        Sfx::Chest => {
            let mut b = render(0.12, |t, n| n * 0.2 * (-t * 20.0).exp());
            b.extend(arp(
                &[(note(76), 0.07), (note(81), 0.07), (note(88), 0.25)],
                0.5,
                7.0,
            ));
            b
        }
        Sfx::Denied => sweep(0.15, 150.0, 120.0, 8.0, |p| square(p, 0.5) * 0.4),
        Sfx::Alert => arp(&[(note(88), 0.04), (note(93), 0.08)], 0.25, 20.0),
        Sfx::Rare => {
            // A bright little fanfare with a shimmer on top.
            let mut b = arp(
                &[
                    (note(84), 0.05),
                    (note(88), 0.05),
                    (note(91), 0.05),
                    (note(96), 0.32),
                ],
                0.25,
                5.0,
            );
            let (mut a, mut c) = (0.0, 0.0);
            let shimmer = render(0.45, |t, _| {
                a += note(103) / RATE;
                c += (note(103) + 7.0) / RATE;
                (sine(a) + sine(c)) * 0.12 * (-t * 5.0).exp() * (t * 40.0).min(1.0)
            });
            for (i, v) in shimmer.iter().enumerate() {
                if let Some(x) = b.get_mut(i + (RATE * 0.12) as usize) {
                    *x += v;
                }
            }
            b
        }
        Sfx::Equip => {
            let mut b = render(0.09, |t, n| n * 0.45 * (-t * 30.0).exp());
            lowpass(&mut b, 0.45);
            b.extend(sweep(0.07, 900.0, 1400.0, 25.0, |p| square(p, 0.5) * 0.3));
            b
        }
        Sfx::Magic => {
            let (mut a, mut c) = (0.0, 0.0);
            render(0.2, |t, _| {
                let f = 1500.0 - t * 4000.0;
                a += f.max(500.0) / RATE;
                c += (f * 1.5).max(700.0) / RATE;
                (sine(a) * 0.5 + square(c, 0.25) * 0.15) * (-t * 12.0).exp()
            })
        }
        Sfx::Blast => {
            let mut phase = 0.0;
            let mut b = render(0.42, |t, n| {
                phase += (220.0 - t * 400.0).max(45.0) / RATE;
                (sine(phase) * 0.7 + n * 0.5 * (-t * 9.0).exp()) * (-t * 6.0).exp()
            });
            lowpass(&mut b, 0.3);
            b
        }
        Sfx::Enchant => {
            // Rising sparkles over a soft, slightly detuned chord.
            let mut b = Vec::new();
            for n in [72, 76, 79, 84, 88, 91, 96] {
                let f = note(n);
                let mut phase = 0.0;
                b.extend(render(0.06, |t, _| {
                    phase += f / RATE;
                    (square(phase, 0.25) * 0.3 + sine(phase) * 0.3) * (-t * 20.0).exp()
                }));
            }
            let (mut a, mut c, mut d) = (0.0, 0.0, 0.0);
            let pad = render(1.0, |t, _| {
                a += note(72) / RATE;
                c += note(76) * 1.003 / RATE;
                d += note(79) * 0.997 / RATE;
                (sine(a) + sine(c) + sine(d)) * 0.12 * (1.0 - t) * (t * 8.0).min(1.0)
            });
            let start = b.len() / 3;
            b.resize(start + pad.len(), 0.0);
            for (i, v) in pad.iter().enumerate() {
                b[start + i] += v;
            }
            b
        }
        Sfx::Block => {
            let (mut a, mut c) = (0.0, 0.0);
            render(0.3, |t, n| {
                a += 1320.0 / RATE;
                c += 1830.0 / RATE;
                (sine(a) * 0.4 + sine(c) * 0.3) * (-t * 14.0).exp() + n * 0.4 * (-t * 60.0).exp()
            })
        }
        Sfx::Door => {
            // A latch, then a soft wooden thump.
            let mut b = render(0.05, |t, n| n * 0.35 * (-t * 60.0).exp());
            lowpass(&mut b, 0.5);
            let mut phase = 0.0;
            let mut thump = render(0.16, |t, n| {
                phase += (140.0 - t * 300.0).max(60.0) / RATE;
                (sine(phase) * 0.6 + n * 0.15) * (-t * 18.0).exp()
            });
            lowpass(&mut thump, 0.35);
            b.extend(thump);
            b
        }
        Sfx::BusHorn => {
            // Beep beep! Two friendly honks.
            let mut b = Vec::new();
            for _ in 0..2 {
                let (mut a, mut c) = (0.0, 0.0);
                b.extend(render(0.13, |t, _| {
                    a += note(69) / RATE;
                    c += note(73) / RATE;
                    (square(a, 0.5) * 0.25 + square(c, 0.5) * 0.2) * (1.0 - t * 2.0).max(0.2)
                }));
                b.extend(std::iter::repeat_n(0.0, (RATE * 0.06) as usize));
            }
            lowpass(&mut b, 0.35);
            b
        }
        Sfx::Fanfare => {
            // Ta-da! A rising run, a held chord and a sparkle on top.
            let mut b = arp(
                &[
                    (note(72), 0.09),
                    (note(76), 0.09),
                    (note(79), 0.09),
                    (note(84), 0.14),
                    (note(79), 0.09),
                    (note(84), 0.5),
                ],
                0.25,
                3.5,
            );
            let (mut a, mut c, mut d) = (0.0, 0.0, 0.0);
            let chord = render(0.9, |t, _| {
                a += note(72) / RATE;
                c += note(76) / RATE;
                d += note(79) / RATE;
                (tri(a) + tri(c) + tri(d)) * 0.1 * (1.0 - t / 0.9) * (t * 20.0).min(1.0)
            });
            let start = (RATE * 0.5) as usize;
            b.resize(b.len().max(start + chord.len()), 0.0);
            for (i, v) in chord.iter().enumerate() {
                b[start + i] += v;
            }
            let (mut e, mut f) = (0.0, 0.0);
            let shimmer = render(0.6, |t, _| {
                e += note(108) / RATE;
                f += (note(108) + 9.0) / RATE;
                (sine(e) + sine(f)) * 0.08 * (-t * 4.0).exp()
            });
            let start = (RATE * 0.55) as usize;
            b.resize(b.len().max(start + shimmer.len()), 0.0);
            for (i, v) in shimmer.iter().enumerate() {
                b[start + i] += v;
            }
            b
        }
        Sfx::Accept => arp(
            &[(note(79), 0.06), (note(84), 0.06), (note(91), 0.18)],
            0.25,
            9.0,
        ),
        Sfx::Talk => sweep(0.045, 620.0, 560.0, 30.0, |p| {
            square(p, 0.5) * 0.35 + sine(p) * 0.2
        }),
        Sfx::Heart => {
            let (mut a, mut c) = (0.0, 0.0);
            render(0.45, |t, _| {
                a += note(88) / RATE;
                c += note(93) / RATE;
                (sine(a) * 0.35 + sine(c) * 0.25 * (t * 10.0).min(1.0)) * (-t * 6.0).exp()
            })
        }
        Sfx::Bell => {
            let (mut a, mut c, mut d) = (0.0, 0.0, 0.0);
            render(1.6, |t, _| {
                a += 392.0 / RATE;
                c += 392.0 * 2.76 / RATE;
                d += 392.0 * 5.4 / RATE;
                (sine(a) * 0.4
                    + sine(c) * 0.2 * (-t * 3.0).exp()
                    + sine(d) * 0.1 * (-t * 6.0).exp())
                    * (-t * 2.2).exp()
            })
        }
        Sfx::Clang => {
            // Inharmonic ringing partials over a sharp strike, then a spit of crackles.
            let (mut a, mut c, mut d) = (0.0, 0.0, 0.0);
            let mut rng = Rng::new(77);
            render(0.32, |t, n| {
                a += 2093.0 / RATE;
                c += 2093.0 * 1.47 / RATE;
                d += 2093.0 * 2.09 / RATE;
                let ring = (sine(a) * 0.35 + sine(c) * 0.22 + sine(d) * 0.12) * (-t * 16.0).exp();
                let strike = n * 0.5 * (-t * 90.0).exp();
                let crackle = if t > 0.02 && rng.chance(0.004) {
                    n.signum() * 0.35 * (-t * 8.0).exp()
                } else {
                    0.0
                };
                ring + strike + crackle
            })
        }
        Sfx::Gulp => {
            let mut b = Vec::new();
            for k in 0..2 {
                let mut phase = 0.0;
                let base = 260.0 + k as f32 * 60.0;
                let mut c = render(0.1, |t, n| {
                    phase += (base + t * 900.0) / RATE;
                    (sine(phase) * 0.6 + n * 0.1) * (-t * 22.0).exp()
                });
                lowpass(&mut c, 0.4);
                b.extend(c);
                b.extend(std::iter::repeat_n(0.0, (RATE * 0.05) as usize));
            }
            b.extend(arp(&[(note(84), 0.05), (note(91), 0.12)], 0.5, 14.0));
            b
        }
        Sfx::Brew => {
            // Bubbles popping at random pitches, then a bright chime.
            let mut rng = Rng::new(5);
            let mut phase = 0.0;
            let mut f = 500.0;
            let mut left = 0.0;
            let mut b = render(0.55, |t, _| {
                left -= 1.0 / RATE;
                if left <= 0.0 {
                    left = 0.04 + rng.f32() * 0.05;
                    f = 350.0 + rng.f32() * 700.0;
                }
                phase += (f + (0.09 - left) * 3000.0) / RATE;
                sine(phase) * 0.35 * (left * 25.0).min(1.0) * (1.0 - t / 0.55)
            });
            lowpass(&mut b, 0.5);
            b.extend(arp(
                &[(note(79), 0.06), (note(86), 0.06), (note(91), 0.2)],
                0.25,
                7.0,
            ));
            b
        }
        Sfx::Spell => {
            let (mut a, mut c) = (0.0, 0.0);
            let mut lp = 0.0;
            render(0.38, |t, n| {
                a += (500.0 + t * 1800.0) / RATE;
                c += (757.0 + t * 2600.0) / RATE;
                lp += (n - lp) * 0.2;
                (sine(a) * 0.3 + square(c, 0.25) * 0.1 + lp * 0.35) * (-t * 7.0).exp()
            })
        }
        Sfx::SpellUp => arp(
            &[
                (note(76), 0.06),
                (note(83), 0.06),
                (note(88), 0.06),
                (note(95), 0.3),
            ],
            0.25,
            5.0,
        ),
        Sfx::Cast => {
            // A whip of air, then the reel spinning out.
            let mut lp = 0.0;
            let mut b = render(0.18, |t, n| {
                let k = 0.08 + 0.6 * (t / 0.18);
                lp += (n - lp) * k;
                lp * (1.0 - t / 0.18) * 0.8
            });
            let mut rng = Rng::new(3);
            b.extend(render(0.25, |t, n| {
                let click = if rng.chance(0.012) {
                    n.signum() * 0.3
                } else {
                    0.0
                };
                click * (1.0 - t / 0.25)
            }));
            b
        }
        Sfx::Plop => {
            // A water drop: a quick upward chirp over a soft splash.
            let mut phase = 0.0;
            let mut b = render(0.16, |t, n| {
                phase += (380.0 + t * 2600.0) / RATE;
                sine(phase) * 0.5 * (-t * 26.0).exp() + n * 0.12 * (-t * 40.0).exp()
            });
            lowpass(&mut b, 0.5);
            b
        }
        Sfx::Bite => {
            // Plunk! and a bright little alarm.
            let mut phase = 0.0;
            let mut b = render(0.12, |t, n| {
                phase += (220.0 + t * 1500.0) / RATE;
                sine(phase) * 0.6 * (-t * 20.0).exp() + n * 0.25 * (-t * 30.0).exp()
            });
            lowpass(&mut b, 0.45);
            b.extend(arp(&[(note(88), 0.05), (note(93), 0.1)], 0.25, 14.0));
            b
        }
        Sfx::Reel => {
            let mut b = Vec::new();
            for k in 0..6 {
                let mut c = render(0.025, |t, n| n * 0.4 * (-t * 160.0).exp());
                lowpass(&mut c, 0.6);
                b.extend(c);
                b.extend(std::iter::repeat_n(
                    0.0,
                    (RATE * (0.03 + k as f32 * 0.004)) as usize,
                ));
            }
            b
        }
        Sfx::Catch => {
            // A splash out of the water and a happy jingle.
            let mut b = render(0.14, |t, n| n * 0.35 * (-t * 18.0).exp());
            lowpass(&mut b, 0.35);
            b.extend(arp(
                &[
                    (note(79), 0.07),
                    (note(83), 0.07),
                    (note(86), 0.07),
                    (note(91), 0.26),
                ],
                0.25,
                6.0,
            ));
            b
        }
        Sfx::Sizzle => {
            // Frying: crackling hiss with little pops.
            let mut rng = Rng::new(11);
            let mut lp = 0.0;
            let mut b = render(0.5, |t, n| {
                lp += (n - lp) * 0.6;
                let hiss = (n - lp) * 0.22;
                let pop = if rng.chance(0.002) {
                    n.signum() * 0.4
                } else {
                    0.0
                };
                (hiss + pop) * (t * 20.0).min(1.0) * (1.0 - t / 0.5)
            });
            lowpass(&mut b, 0.8);
            b
        }
        Sfx::Piano => {
            // A little tune on a slightly out-of-tune upright.
            let tune = [72, 76, 79, 77, 76, 74, 72, 79, 84];
            let mut b = Vec::new();
            for (i, n) in tune.into_iter().enumerate() {
                let f = note(n);
                let len = if i == tune.len() - 1 { 0.6 } else { 0.16 };
                let (mut a, mut c) = (0.0, 0.0);
                b.extend(render(len, |t, _| {
                    a += f / RATE;
                    c += f * 2.003 / RATE;
                    (tri(a) * 0.4 + sine(c) * 0.15) * (-t * 5.0).exp() * (t * 200.0).min(1.0)
                }));
            }
            b
        }
    }
}
