//! Audio: a mixer thread that plays procedural sound effects and chiptune music.
//!
//! Output goes through ALSA on Linux (loaded at run time, so nothing is needed to build) and
//! winmm on Windows. Anything else, or a machine without a sound device, runs silently.

mod backend;
pub mod music;
pub mod synth;

use std::sync::mpsc::{Receiver, Sender, TryRecvError, channel};

pub use music::Song;
pub use synth::Sfx;

const FRAMES: usize = 512;

enum Cmd {
    Play(Sfx, f32, f32),
    Music(Option<(Song, i32, f32)>),
    Volume(f32, f32),
}

pub struct Audio {
    tx: Option<Sender<Cmd>>,
    current: std::cell::Cell<Option<(Song, i32)>>,
}

impl Audio {
    /// Starts the audio thread. Never fails: without a device the game is just quiet.
    pub fn start() -> Audio {
        let (tx, rx) = channel();
        let ok = std::thread::Builder::new()
            .name("hollowbloom-audio".into())
            .spawn(move || run(rx))
            .is_ok();
        Audio {
            tx: ok.then_some(tx),
            current: std::cell::Cell::new(None),
        }
    }

    pub fn silent() -> Audio {
        Audio {
            tx: None,
            current: std::cell::Cell::new(None),
        }
    }

    fn send(&self, c: Cmd) {
        if let Some(tx) = &self.tx {
            let _ = tx.send(c);
        }
    }

    pub fn play(&self, s: Sfx) {
        self.send(Cmd::Play(s, 1.0, 1.0));
    }

    pub fn play_at(&self, s: Sfx, vol: f32, pitch: f32) {
        self.send(Cmd::Play(s, vol, pitch));
    }

    /// Switches music (crossfades). Repeating the current song does nothing.
    pub fn music(&self, song: Option<Song>, transpose: i32, tempo: f32) {
        let key = song.map(|s| (s, transpose));
        if self.current.get() == key {
            return;
        }
        self.current.set(key);
        self.send(Cmd::Music(song.map(|s| (s, transpose, tempo))));
    }

    pub fn set_volume(&self, music: f32, sfx: f32) {
        self.send(Cmd::Volume(music, sfx));
    }
}

struct Voice {
    buf: usize,
    pos: f32,
    rate: f32,
    vol: f32,
}

struct Mixer {
    bank: Vec<Vec<f32>>,
    voices: Vec<Voice>,
    music: Option<music::Player>,
    fading: Option<music::Player>,
    music_vol: f32,
    sfx_vol: f32,
}

impl Mixer {
    fn new() -> Mixer {
        Mixer {
            bank: synth::ALL.iter().map(|s| synth::make(*s)).collect(),
            voices: Vec::new(),
            music: None,
            fading: None,
            music_vol: 0.7,
            sfx_vol: 0.8,
        }
    }

    fn apply(&mut self, c: Cmd) {
        match c {
            Cmd::Play(s, vol, pitch) => {
                let buf = synth::ALL.iter().position(|x| *x == s).unwrap_or(0);
                // Avoid piling up identical sounds in the same instant.
                let dupes = self
                    .voices
                    .iter()
                    .filter(|v| v.buf == buf && v.pos < 800.0)
                    .count();
                if dupes < 2 && self.voices.len() < 24 {
                    self.voices.push(Voice {
                        buf,
                        pos: 0.0,
                        rate: pitch,
                        vol,
                    });
                }
            }
            Cmd::Music(next) => {
                if let Some(mut old) = self.music.take() {
                    old.target = 0.0;
                    self.fading = Some(old);
                }
                self.music = next.map(|(s, tr, tempo)| music::Player::new(s, tr, tempo));
            }
            Cmd::Volume(m, s) => {
                self.music_vol = m.clamp(0.0, 1.0);
                self.sfx_vol = s.clamp(0.0, 1.0);
            }
        }
    }

    fn render(&mut self, out: &mut [i16]) {
        let fade_step = 1.0 / (synth::RATE * 0.9);
        for frame in out.chunks_exact_mut(2) {
            let mut m = 0.0;
            for p in [&mut self.music, &mut self.fading].into_iter().flatten() {
                if p.gain < p.target {
                    p.gain = (p.gain + fade_step).min(p.target);
                } else if p.gain > p.target {
                    p.gain = (p.gain - fade_step).max(p.target);
                }
                m += p.next() * p.gain;
            }
            let mut s = 0.0;
            for v in &mut self.voices {
                let buf = &self.bank[v.buf];
                let i = v.pos as usize;
                if i + 1 < buf.len() {
                    let f = v.pos.fract();
                    s += (buf[i] * (1.0 - f) + buf[i + 1] * f) * v.vol;
                }
                v.pos += v.rate;
            }
            let mix = (m * self.music_vol * 0.8 + s * self.sfx_vol * 0.55).clamp(-1.0, 1.0);
            let smp = (mix * 30000.0) as i16;
            frame[0] = smp;
            frame[1] = smp;
        }
        let bank = &self.bank;
        self.voices
            .retain(|v| (v.pos as usize) + 1 < bank[v.buf].len());
        if self.fading.as_ref().is_some_and(|f| f.gain <= 0.0) {
            self.fading = None;
        }
    }
}

fn run(rx: Receiver<Cmd>) {
    let Some(mut sink) = backend::open(synth::RATE as u32, FRAMES) else {
        // No device: keep draining commands so senders never block.
        while rx.recv().is_ok() {}
        return;
    };
    let mut mixer = Mixer::new();
    let mut buf = vec![0i16; FRAMES * 2];
    loop {
        loop {
            match rx.try_recv() {
                Ok(c) => mixer.apply(c),
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => return,
            }
        }
        mixer.render(&mut buf);
        if sink.write(&buf).is_err() {
            return;
        }
    }
}

/// Raises the system timer resolution so frame pacing is smooth (Windows only).
pub fn precise_timers() {
    backend::precise_timers();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mixer_makes_sound() {
        let mut m = Mixer::new();
        m.apply(Cmd::Music(Some((Song::Morning, 0, 1.0))));
        m.apply(Cmd::Play(Sfx::Coin, 1.0, 1.0));
        let mut buf = vec![0i16; FRAMES * 2];
        let mut peak = 0i32;
        for _ in 0..40 {
            m.render(&mut buf);
            peak = peak.max(buf.iter().map(|s| (*s as i32).abs()).max().unwrap_or(0));
        }
        assert!(peak > 1000, "expected audible output, peak {peak}");
        assert!(peak <= 30000);
    }

    #[test]
    fn every_effect_renders() {
        for s in synth::ALL {
            let b = synth::make(*s);
            assert!(!b.is_empty(), "{s:?} is empty");
            assert!(
                b.iter().all(|v| v.is_finite() && v.abs() <= 1.5),
                "{s:?} out of range"
            );
        }
    }
}
