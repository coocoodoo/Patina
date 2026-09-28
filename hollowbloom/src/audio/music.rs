//! A tiny chiptune sequencer and the game's songs.

use super::synth::RATE;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Song {
    Title,
    Morning,
    Night,
    Hollow,
    Haven,
    Boss,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Inst {
    Lead,
    Bell,
    Soft,
    Bass,
    Arp,
    Drums,
}

struct SongDef {
    bpm: f32,
    tracks: [(&'static str, Inst, f32); 4],
}

fn def(s: Song) -> SongDef {
    match s {
        Song::Morning | Song::Title => SongDef {
            bpm: if s == Song::Title { 84.0 } else { 104.0 },
            tracks: [
                (
                    "A4 - C5 - F5 - E5 D5 C5 - A4 - - - . . D5 - F5 - Bb5 - A5 G5 A5 - G5 - - - . . \
                     A4 - C5 - F5 - G5 A5 Bb5 - A5 - F5 - D5 - E5 - G5 - C5 - E5 - F5 - - - - - . .",
                    if s == Song::Title {
                        Inst::Bell
                    } else {
                        Inst::Lead
                    },
                    0.2,
                ),
                (
                    "F4 A4 C5 A4 F4 A4 C5 A4 D4 F4 A4 F4 D4 F4 A4 F4 D4 F4 Bb4 F4 D4 F4 Bb4 F4 E4 G4 C5 G4 E4 G4 C5 G4 \
                     F4 A4 C5 A4 F4 A4 C5 A4 D4 F4 Bb4 F4 D4 F4 Bb4 F4 E4 G4 C5 G4 E4 G4 C5 G4 F4 A4 C5 F5 C5 A4 F4 .",
                    Inst::Arp,
                    0.1,
                ),
                (
                    "F2 - - - C3 - - - D2 - - - A2 - - - Bb1 - - - F2 - - - C2 - - - G2 - - - \
                     F2 - - - C3 - - - Bb1 - - - F2 - - - C2 - - - G2 - E2 - F2 - - - C2 - F2 -",
                    Inst::Bass,
                    0.24,
                ),
                (
                    if s == Song::Title {
                        "."
                    } else {
                        "k . h . s . h . k . h . s . h h"
                    },
                    Inst::Drums,
                    0.16,
                ),
            ],
        },
        Song::Night | Song::Haven => SongDef {
            bpm: if s == Song::Haven { 70.0 } else { 76.0 },
            tracks: [
                (
                    "E5 - - - C5 - - - A4 - - - C5 - - - G4 - - - E5 - D5 - D5 - - - - - . . \
                     E5 - - - A5 - - - G5 - F5 - E5 - - - D5 - E5 - C5 - - - B4 - - - - - . .",
                    Inst::Bell,
                    0.2,
                ),
                (
                    "A3 C4 E4 C4 A3 C4 E4 C4 F3 A3 C4 A3 F3 A3 C4 A3 C4 E4 G4 E4 C4 E4 G4 E4 G3 B3 D4 B3 G3 B3 D4 B3",
                    Inst::Soft,
                    0.09,
                ),
                (
                    "A2 - - - - - - - F2 - - - - - - - C3 - - - - - - - G2 - - - - - - -",
                    Inst::Bass,
                    0.2,
                ),
                (".", Inst::Drums, 0.0),
            ],
        },
        Song::Hollow => SongDef {
            bpm: 92.0,
            tracks: [
                (
                    ". . . . A4 - D5 - F5 - E5 - D5 - - - . . . . G4 - C5 - E5 - D5 - C#5 - - - \
                     D5 - - - F5 - A5 - G5 - F5 - D5 - - - E5 - - - G5 - E5 - C#5 - - - - - . .",
                    Inst::Lead,
                    0.17,
                ),
                (
                    "D4 F4 A4 D5 A4 F4 D4 F4 Bb3 D4 F4 Bb4 F4 D4 Bb3 D4 C4 E4 G4 C5 G4 E4 C4 E4 A3 C#4 E4 A4 E4 C#4 A3 C#4",
                    Inst::Arp,
                    0.1,
                ),
                (
                    "D2 - - - D2 - - - Bb1 - - - Bb1 - - - C2 - - - C2 - - - A1 - - - A1 - - -",
                    Inst::Bass,
                    0.24,
                ),
                ("k . . h k . s . k . . h k . s h", Inst::Drums, 0.14),
            ],
        },
        Song::Boss => SongDef {
            bpm: 132.0,
            tracks: [
                (
                    "D5 - D5 E5 F5 - E5 D5 C#5 - A4 - - - . . D5 - D5 E5 F5 - G5 A5 Bb5 - A5 - - - . . \
                     A5 - G5 F5 E5 - F5 G5 F5 - E5 D5 C#5 - D5 E5 D5 - A4 - F4 - A4 - D5 - - - . . . .",
                    Inst::Lead,
                    0.18,
                ),
                (".", Inst::Arp, 0.0),
                (
                    "D2 D2 D3 D2 D2 D2 D3 D2 A1 A1 A2 A1 A1 A1 A2 A1 D2 D2 D3 D2 D2 D2 D3 D2 Bb1 Bb1 Bb2 Bb1 Bb1 Bb1 Bb2 Bb1 \
                     F2 F2 F3 F2 F2 F2 F3 F2 A1 A1 A2 A1 A1 A1 A2 A1 D2 D2 D3 D2 D2 D2 D3 D2 D2 D2 D3 D2 A1 A1 C#2 A1",
                    Inst::Bass,
                    0.24,
                ),
                ("k h s h k h s h", Inst::Drums, 0.18),
            ],
        },
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum Step {
    Note(f32),
    Hold,
    Rest,
    Kick,
    Snare,
    Hat,
}

fn parse_note(tok: &str) -> Option<f32> {
    let mut chars = tok.chars();
    let letter = chars.next()?;
    let base = match letter {
        'C' => 0,
        'D' => 2,
        'E' => 4,
        'F' => 5,
        'G' => 7,
        'A' => 9,
        'B' => 11,
        _ => return None,
    };
    let rest: String = chars.collect();
    let (acc, oct) = if let Some(r) = rest.strip_prefix('#') {
        (1, r)
    } else if let Some(r) = rest.strip_prefix('b') {
        (-1, r)
    } else {
        (0, rest.as_str())
    };
    let octave: i32 = oct.parse().ok()?;
    let midi = 12 * (octave + 1) + base + acc;
    Some(midi as f32)
}

fn parse(track: &str) -> Vec<Step> {
    track
        .split_whitespace()
        .map(|t| match t {
            "-" => Step::Hold,
            "." => Step::Rest,
            "k" => Step::Kick,
            "s" => Step::Snare,
            "h" => Step::Hat,
            n => parse_note(n).map(Step::Note).unwrap_or(Step::Rest),
        })
        .collect()
}

#[derive(Default)]
struct Chan {
    freq: f32,
    phase: f32,
    env: f32,
    gate: bool,
    age: f32,
    lp: f32,
    drum: Option<(Step, f32)>,
    noise: u32,
}

pub struct Player {
    tracks: Vec<(Vec<Step>, Inst, f32)>,
    step: usize,
    sample_in_step: f32,
    samples_per_step: f32,
    chans: [Chan; 4],
    transpose: f32,
    pub gain: f32,
    pub target: f32,
}

fn midi_hz(m: f32) -> f32 {
    440.0 * 2f32.powf((m - 69.0) / 12.0)
}

impl Player {
    pub fn new(song: Song, transpose: i32, tempo: f32) -> Player {
        let d = def(song);
        let tracks = d
            .tracks
            .iter()
            .map(|(t, i, g)| (parse(t), *i, *g))
            .collect();
        Player {
            tracks,
            step: 0,
            sample_in_step: 0.0,
            samples_per_step: RATE * 60.0 / (d.bpm * tempo * 2.0),
            chans: Default::default(),
            transpose: transpose as f32,
            gain: 0.0,
            target: 1.0,
        }
    }

    fn trigger(&mut self) {
        for (ci, (steps, inst, _)) in self.tracks.iter().enumerate() {
            if steps.is_empty() {
                continue;
            }
            let s = steps[self.step % steps.len()];
            let ch = &mut self.chans[ci];
            match s {
                Step::Note(m) => {
                    ch.freq = midi_hz(m + self.transpose);
                    ch.gate = true;
                    ch.age = 0.0;
                    if *inst == Inst::Bell || *inst == Inst::Arp {
                        ch.env = 1.0;
                    }
                }
                Step::Rest => ch.gate = false,
                Step::Hold => {}
                Step::Kick | Step::Snare | Step::Hat => ch.drum = Some((s, 0.0)),
            }
        }
    }

    /// Next mono sample.
    pub fn next(&mut self) -> f32 {
        if self.sample_in_step <= 0.0 {
            self.trigger();
            self.sample_in_step += self.samples_per_step;
            self.step = self.step.wrapping_add(1);
        }
        self.sample_in_step -= 1.0;
        let dt = 1.0 / RATE;
        let mut out = 0.0;
        for (ci, (_, inst, gain)) in self.tracks.iter().enumerate() {
            let ch = &mut self.chans[ci];
            ch.age += dt;
            let s = match inst {
                Inst::Drums => {
                    let mut v = 0.0;
                    if let Some((kind, age)) = ch.drum.as_mut() {
                        *age += dt;
                        ch.noise = ch.noise.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
                        let n = (ch.noise >> 9) as f32 / (1u32 << 23) as f32 * 2.0 - 1.0;
                        v = match kind {
                            Step::Kick => {
                                let f = 45.0 + 110.0 * (-*age * 30.0).exp();
                                ch.phase += f * dt;
                                (ch.phase * std::f32::consts::TAU).sin() * (-*age * 22.0).exp()
                            }
                            Step::Snare => n * 0.6 * (-*age * 20.0).exp(),
                            _ => {
                                let hp = n - ch.lp;
                                ch.lp = n;
                                hp * 0.35 * (-*age * 70.0).exp()
                            }
                        };
                        if *age > 0.4 {
                            ch.drum = None;
                        }
                    }
                    v
                }
                _ => {
                    let target = if ch.gate { 1.0 } else { 0.0 };
                    match inst {
                        Inst::Bell | Inst::Arp => {
                            let decay = if *inst == Inst::Bell { 2.2 } else { 7.0 };
                            ch.env *= 1.0 - decay * dt;
                            if !ch.gate {
                                ch.env *= 1.0 - 12.0 * dt;
                            }
                        }
                        Inst::Soft => ch.env += (target * 0.8 - ch.env) * dt * 12.0,
                        _ => {
                            let rate = if target > ch.env { 180.0 } else { 14.0 };
                            ch.env += (target - ch.env) * (rate * dt).min(1.0);
                        }
                    }
                    let vib = if *inst == Inst::Lead && ch.age > 0.25 {
                        1.0 + (ch.age * 34.0).sin() * 0.006
                    } else {
                        1.0
                    };
                    ch.phase += ch.freq * vib * dt;
                    let p = ch.phase.fract();
                    let wave = match inst {
                        Inst::Lead => {
                            let sq = if p < 0.25 { 1.0 } else { -1.0 };
                            let tr = if p < 0.5 {
                                4.0 * p - 1.0
                            } else {
                                3.0 - 4.0 * p
                            };
                            sq * 0.45 + tr * 0.55
                        }
                        Inst::Bell => {
                            let t = std::f32::consts::TAU;
                            (p * t).sin() * 0.8 + (p * 2.0 * t).sin() * 0.25
                        }
                        Inst::Arp => {
                            if p < 0.125 {
                                1.0
                            } else {
                                -0.15
                            }
                        }
                        _ => {
                            if p < 0.5 {
                                4.0 * p - 1.0
                            } else {
                                3.0 - 4.0 * p
                            }
                        }
                    };
                    // Gentle low-pass keeps the square waves cosy rather than harsh.
                    ch.lp += (wave - ch.lp) * 0.3;
                    ch.lp * ch.env
                }
            };
            out += s * gain;
        }
        out
    }
}
