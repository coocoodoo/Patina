//! Audio: a mixer thread that plays procedural sound effects and the music (recorded tracks
//! where there are some, the chiptune band for the rest).
//!
//! Output goes through ALSA on Linux (loaded at run time, so nothing is needed to build) and
//! winmm on Windows. Anything else, or a machine without a sound device, runs silently.

use std::sync::Arc;

mod backend;
pub mod music;
mod songs;
pub mod synth;
pub mod track;

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

/// Whoever's playing a song: the band, or a recorded track.
enum Source {
    Band(Box<music::Player>),
    Track(track::TrackPlayer),
}

/// A song playing, fading in or out.
struct Playing {
    src: Source,
    gain: f32,
    target: f32,
}

impl Playing {
    /// A track still being decoded isn't ready (and doesn't start fading in until it is).
    fn ready(&self) -> bool {
        match &self.src {
            Source::Band(_) => true,
            Source::Track(t) => t.ready(),
        }
    }

    fn next(&mut self) -> (f32, f32) {
        match &mut self.src {
            Source::Band(p) => {
                let s = p.next();
                (s, s)
            }
            Source::Track(t) => t.next(),
        }
    }
}

/// How many decoded tracks to keep, so going in and out of places doesn't decode them again.
const KEEP: usize = 3;

/// A track that's come back to within this long carries on from where it left off (out of a
/// shop and back into town, say) rather than starting over.
const RESUME: std::time::Duration = std::time::Duration::from_secs(180);

struct Mixer {
    bank: Vec<Vec<f32>>,
    voices: Vec<Voice>,
    music: Option<Playing>,
    fading: Option<Playing>,
    /// Recently decoded tracks, the latest last.
    decoded: Vec<(Song, Arc<track::Pcm>)>,
    /// Where tracks got to when they stopped, and when.
    left_off: Vec<(Song, usize, std::time::Instant)>,
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
            decoded: Vec::new(),
            left_off: Vec::new(),
            music_vol: 0.7,
            sfx_vol: 0.8,
        }
    }

    /// Starts a song: its recorded track if it has one (decoded already, or decoding in the
    /// background), otherwise the band.
    fn start(&mut self, song: Song, transpose: i32, tempo: f32) -> Playing {
        let src = match track::track(song) {
            Some(t) => {
                let pcm = self
                    .decoded
                    .iter()
                    .find(|(s, _)| *s == song)
                    .map(|(_, p)| p.clone());
                let mut p = track::TrackPlayer::new(t, pcm);
                if let Some(&(_, pos, when)) = self.left_off.iter().find(|(s, _, _)| *s == song) {
                    if when.elapsed() < RESUME {
                        p.resume(pos);
                    }
                }
                if p.failed {
                    Source::Band(Box::new(music::Player::new(song, transpose, tempo)))
                } else {
                    Source::Track(p)
                }
            }
            None => Source::Band(Box::new(music::Player::new(song, transpose, tempo))),
        };
        Playing {
            src,
            gain: 0.0,
            target: 1.0,
        }
    }

    /// Notes where a song that's stopped had got to.
    fn retire(&mut self, p: Playing) {
        if let Source::Track(t) = p.src {
            self.left_off.retain(|(s, _, _)| *s != t.song);
            self.left_off
                .push((t.song, t.pos(), std::time::Instant::now()));
        }
    }

    /// Picks up tracks that have finished decoding (keeping them for next time), and hands
    /// any that couldn't be decoded to the band.
    fn poll_tracks(&mut self) {
        for p in [&mut self.music, &mut self.fading].into_iter().flatten() {
            let Source::Track(t) = &mut p.src else {
                continue;
            };
            if let Some(pcm) = t.poll() {
                self.decoded.retain(|(s, _)| *s != t.song);
                self.decoded.push((t.song, pcm));
                if self.decoded.len() > KEEP {
                    self.decoded.remove(0);
                }
            }
            if t.failed {
                p.src = Source::Band(Box::new(music::Player::new(t.song, 0, 1.0)));
            }
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
                    if let Some(gone) = self.fading.replace(old) {
                        self.retire(gone);
                    }
                }
                self.music = next.map(|(s, tr, tempo)| self.start(s, tr, tempo));
            }
            Cmd::Volume(m, s) => {
                self.music_vol = m.clamp(0.0, 1.0);
                self.sfx_vol = s.clamp(0.0, 1.0);
            }
        }
    }

    fn render(&mut self, out: &mut [i16]) {
        self.poll_tracks();
        let fade_step = 1.0 / (synth::RATE * 0.9);
        for frame in out.chunks_exact_mut(2) {
            let (mut ml, mut mr) = (0.0, 0.0);
            for p in [&mut self.music, &mut self.fading].into_iter().flatten() {
                if !p.ready() {
                    continue;
                }
                if p.gain < p.target {
                    p.gain = (p.gain + fade_step).min(p.target);
                } else if p.gain > p.target {
                    p.gain = (p.gain - fade_step).max(p.target);
                }
                let (l, r) = p.next();
                ml += l * p.gain;
                mr += r * p.gain;
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
            let sfx = s * self.sfx_vol * 0.55;
            let music = self.music_vol * 0.8;
            frame[0] = ((ml * music + sfx).clamp(-1.0, 1.0) * 30000.0) as i16;
            frame[1] = ((mr * music + sfx).clamp(-1.0, 1.0) * 30000.0) as i16;
        }
        let bank = &self.bank;
        self.voices
            .retain(|v| (v.pos as usize) + 1 < bank[v.buf].len());
        if self
            .fading
            .as_ref()
            .is_some_and(|f| f.gain <= 0.0 && f.target <= 0.0)
        {
            if let Some(gone) = self.fading.take() {
                self.retire(gone);
            }
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
    fn recorded_tracks_play_once_decoded() {
        let mut m = Mixer::new();
        m.apply(Cmd::Music(Some((Song::Night, 0, 1.0))));
        let mut buf = vec![0i16; FRAMES * 2];
        let mut peak = 0i32;
        let mut wide = false;
        for _ in 0..3000 {
            m.render(&mut buf);
            peak = peak.max(buf.iter().map(|s| (*s as i32).abs()).max().unwrap_or(0));
            wide |= buf.chunks_exact(2).any(|f| f[0] != f[1]);
            if peak > 2000 && wide {
                break;
            }
            // Give the decoder a moment.
            std::thread::sleep(std::time::Duration::from_millis(2));
        }
        assert!(peak > 2000, "expected the night track, peak {peak}");
        assert!(wide, "expected stereo");
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
