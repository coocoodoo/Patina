//! Recorded music: MP3 tracks built into the game for some of the songs, decoded in the
//! background the first time they're wanted, and looped seamlessly between two points.
//! Songs without a track are played by the chiptune band in `music.rs`.

use std::sync::Arc;
use std::sync::mpsc::{Receiver, TryRecvError, channel};

use symphonia::core::audio::SampleBuffer;
use symphonia::core::codecs::DecoderOptions;
use symphonia::core::errors::Error;
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::{MediaSourceStream, ReadOnlySource};
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;

use super::music::Song;
use super::synth::RATE;

/// A recorded track for one of the songs.
pub struct Track {
    pub song: Song,
    pub mp3: &'static [u8],
    /// The loop, in frames of the decoded track: it plays from the start to `end`, then
    /// carries on from `start`, round and round, the seam blended.
    pub start: usize,
    pub end: usize,
    /// Or, for a track with an ending of its own: frames of quiet after the end before it
    /// starts again from `start` (with no blending).
    pub rest: usize,
    /// A looping track's tempo (four beats to a bar): told to finish, it waits for the next
    /// bar to leave the loop for what follows its end, and plays on to the finish.
    pub bpm: f32,
    /// How much to turn it down to sit with the rest of the game's sound.
    pub gain: f32,
}

/// Frames of quiet between the end of a track and its start again (on top of the second
/// left after its last sound).
const REST: usize = 22_050;

/// The songs' recorded tracks (made with Google's Lyria 3). Each is played all the way
/// through (from its first sound to a second after its last) and then started again, but
/// for the guardians', which loops seamlessly so a fight never stops for an ending.
pub const TRACKS: &[Track] = &[
    // "The First Day of Spring"
    Track {
        song: Song::Title,
        mp3: include_bytes!("../../music/title.mp3"),
        start: 0,
        end: 6_703_200,
        rest: REST,
        bpm: 0.0,
        gain: 0.655,
    },
    // "Seven AM Dew"
    Track {
        song: Song::Morning,
        mp3: include_bytes!("../../music/morning.mp3"),
        start: 0,
        end: 6_174_000,
        rest: REST,
        bpm: 0.0,
        gain: 0.596,
    },
    // "Golden Hour at the Orchard"
    Track {
        song: Song::Afternoon,
        mp3: include_bytes!("../../music/afternoon.mp3"),
        start: 0,
        end: 7_904_925,
        rest: REST,
        bpm: 0.0,
        gain: 0.655,
    },
    // "Moonlit Fence Posts"
    Track {
        song: Song::Night,
        mp3: include_bytes!("../../music/night.mp3"),
        start: 0,
        end: 6_989_850,
        rest: REST,
        bpm: 0.0,
        gain: 0.643,
    },
    // "Cobblestone Promenade"
    Track {
        song: Song::Town,
        mp3: include_bytes!("../../music/town.mp3"),
        start: 70_560,
        end: 7_860_825,
        rest: REST,
        bpm: 0.0,
        gain: 0.673,
    },
    // "Sunday Morning Curio"
    Track {
        song: Song::Shop,
        mp3: include_bytes!("../../music/shop.mp3"),
        start: 103_635,
        end: 7_750_575,
        rest: REST,
        bpm: 0.0,
        gain: 0.629,
    },
    // "The Keeper's Hearth"
    Track {
        song: Song::Haven,
        mp3: include_bytes!("../../music/haven.mp3"),
        start: 0,
        end: 6_857_550,
        rest: REST,
        bpm: 0.0,
        gain: 0.653,
    },
    // "Beneath the Glowing Cap"
    Track {
        song: Song::Burrows,
        mp3: include_bytes!("../../music/burrows.mp3"),
        start: 0,
        end: 6_714_225,
        rest: REST,
        bpm: 0.0,
        gain: 0.668,
    },
    // "Below the Glacial Line"
    Track {
        song: Song::Glimmer,
        mp3: include_bytes!("../../music/glimmer.mp3"),
        start: 0,
        end: 7_089_075,
        rest: REST,
        bpm: 0.0,
        gain: 0.666,
    },
    // "Beneath the Burning Spire"
    Track {
        song: Song::Depths,
        mp3: include_bytes!("../../music/depths.mp3"),
        start: 0,
        end: 7_982_100,
        rest: REST,
        bpm: 0.0,
        gain: 0.641,
    },
    // "The Seventh Gate": loops over its middle, breakdown and all, so a fight never stops
    // for an ending; once the guardian falls it plays on from the loop's end to its own.
    Track {
        song: Song::Boss,
        mp3: include_bytes!("../../music/boss.mp3"),
        start: 2_232_024,
        end: 5_763_072,
        rest: 0,
        bpm: 144.0,
        gain: 0.700,
    },
];

pub fn track(song: Song) -> Option<&'static Track> {
    TRACKS.iter().find(|t| t.song == song)
}

/// Decoded audio: interleaved stereo 16-bit frames at the mixer's rate.
pub struct Pcm {
    pub data: Vec<i16>,
}

impl Pcm {
    pub fn frames(&self) -> usize {
        self.data.len() / 2
    }
}

/// Decodes an MP3 to stereo frames at the mixer's rate.
pub fn decode(mp3: &'static [u8]) -> Result<Pcm, String> {
    let src = ReadOnlySource::new(std::io::Cursor::new(mp3));
    let mss = MediaSourceStream::new(Box::new(src), Default::default());
    let mut hint = Hint::new();
    hint.with_extension("mp3");
    let probed = symphonia::default::get_probe()
        .format(
            &hint,
            mss,
            &FormatOptions::default(),
            &MetadataOptions::default(),
        )
        .map_err(|e| e.to_string())?;
    let mut format = probed.format;
    let track = format.default_track().ok_or("no audio in it")?;
    let id = track.id;
    let rate = track.codec_params.sample_rate.unwrap_or(44_100);
    let mut decoder = symphonia::default::get_codecs()
        .make(&track.codec_params, &DecoderOptions::default())
        .map_err(|e| e.to_string())?;
    let mut data: Vec<i16> = Vec::new();
    let mut buf: Option<SampleBuffer<i16>> = None;
    loop {
        let packet = match format.next_packet() {
            Ok(p) => p,
            Err(Error::IoError(e)) if e.kind() == std::io::ErrorKind::UnexpectedEof => break,
            Err(e) => return Err(e.to_string()),
        };
        if packet.track_id() != id {
            continue;
        }
        let decoded = match decoder.decode(&packet) {
            Ok(d) => d,
            // A damaged frame: skip it and carry on.
            Err(Error::DecodeError(_)) => continue,
            Err(e) => return Err(e.to_string()),
        };
        let spec = *decoded.spec();
        let channels = spec.channels.count().max(1);
        let need = decoded.capacity() * channels;
        if buf.as_ref().is_none_or(|b| b.capacity() < need) {
            buf = Some(SampleBuffer::new(decoded.capacity() as u64, spec));
        }
        let Some(b) = buf.as_mut() else { continue };
        b.copy_interleaved_ref(decoded);
        for f in b.samples().chunks_exact(channels) {
            data.push(f[0]);
            data.push(f[channels.min(2) - 1]);
        }
    }
    if data.is_empty() {
        return Err("no audio decoded".into());
    }
    if rate != RATE as u32 {
        data = resample(&data, rate, RATE as u32);
    }
    Ok(Pcm { data })
}

/// Stereo frames at one rate to another, by straight lines between samples.
fn resample(data: &[i16], from: u32, to: u32) -> Vec<i16> {
    let frames = data.len() / 2;
    let out_frames = (frames as u64 * to as u64 / from as u64) as usize;
    let step = from as f64 / to as f64;
    let mut out = Vec::with_capacity(out_frames * 2);
    for i in 0..out_frames {
        let p = i as f64 * step;
        let j = (p as usize).min(frames - 1);
        let k = (j + 1).min(frames - 1);
        let f = (p - j as f64) as f32;
        for c in 0..2 {
            let (a, b) = (data[j * 2 + c] as f32, data[k * 2 + c] as f32);
            out.push((a + (b - a) * f) as i16);
        }
    }
    out
}

/// How long the seam of a loop is blended over, in frames (40 ms).
const SEAM: usize = 1764;

/// Plays a track, once it's decoded, round its loop.
pub struct TrackPlayer {
    pub song: Song,
    pcm: Option<Arc<Pcm>>,
    loading: Option<Receiver<Result<Pcm, String>>>,
    /// It couldn't be decoded: the band should play the song instead.
    pub failed: bool,
    pos: usize,
    start: usize,
    end: usize,
    rest: usize,
    seam: usize,
    gain: f32,
    bpm: f32,
    /// Frames to a bar, for leaving the loop in time (none known: straight away).
    bar: f64,
    /// Where it leaves the loop for its ending, once told to finish.
    leave_at: Option<usize>,
    finishing: bool,
    /// It has played its ending to the last frame.
    pub done: bool,
}

impl TrackPlayer {
    /// A player for a track, from an already decoded copy, or decoding it in the background.
    pub fn new(t: &'static Track, pcm: Option<Arc<Pcm>>) -> TrackPlayer {
        let loading = if pcm.is_none() {
            let (tx, rx) = channel();
            let mp3 = t.mp3;
            let spawned = std::thread::Builder::new()
                .name("hollowbloom-decode".into())
                .spawn(move || {
                    let _ = tx.send(decode(mp3));
                });
            spawned.is_ok().then_some(rx)
        } else {
            None
        };
        let mut p = TrackPlayer {
            song: t.song,
            failed: pcm.is_none() && loading.is_none(),
            pcm: None,
            loading,
            // A track with an ending plays from its start every time; a looping one plays
            // its lead-in first.
            pos: if t.rest > 0 { t.start } else { 0 },
            start: t.start,
            end: t.end,
            rest: t.rest,
            seam: SEAM,
            gain: t.gain,
            bpm: t.bpm,
            bar: 0.0,
            leave_at: None,
            finishing: false,
            done: false,
        };
        if let Some(pcm) = pcm {
            p.take(pcm);
        }
        p
    }

    /// Settles the loop to fit the decoded track.
    fn take(&mut self, pcm: Arc<Pcm>) {
        let frames = pcm.frames();
        self.end = self.end.clamp(1, frames);
        self.start = self.start.min(self.end - 1);
        self.seam = if self.rest > 0 {
            0
        } else {
            SEAM.min(self.start).min(self.end - self.start)
        };
        // The loop holds a whole number of bars: take the bar from that, so leaving on a bar
        // lands on the same beat as the loop's own seam.
        self.bar = if self.bpm > 0.0 && self.rest == 0 {
            let len = (self.end - self.start) as f64;
            let guess = 4.0 * 60.0 / self.bpm as f64 * RATE as f64;
            len / (len / guess).round().max(1.0)
        } else {
            0.0
        };
        self.pcm = Some(pcm);
    }

    /// Picks up the decoded track if it has just finished (handing it back, to keep).
    pub fn poll(&mut self) -> Option<Arc<Pcm>> {
        let rx = self.loading.as_ref()?;
        match rx.try_recv() {
            Ok(Ok(pcm)) => {
                self.loading = None;
                let pcm = Arc::new(pcm);
                self.take(pcm.clone());
                Some(pcm)
            }
            Ok(Err(e)) => {
                eprintln!(
                    "hollowbloom: couldn't decode the {:?} track: {e}",
                    self.song
                );
                self.loading = None;
                self.failed = true;
                None
            }
            Err(TryRecvError::Empty) => None,
            Err(TryRecvError::Disconnected) => {
                self.loading = None;
                self.failed = true;
                None
            }
        }
    }

    pub fn ready(&self) -> bool {
        self.pcm.is_some()
    }

    /// Where it's got to, for picking up there again later.
    pub fn pos(&self) -> usize {
        self.pos
    }

    /// Carries on from where it had got to before.
    pub fn resume(&mut self, pos: usize) {
        self.pos = pos.min(self.end + self.rest - 1);
    }

    /// Whether it has an ending to play when told to finish (a looping track does).
    pub fn has_ending(&self) -> bool {
        self.rest == 0 && !self.done
    }

    /// Leaves the loop for the track's own ending: at the next bar (in step with the loop) it
    /// blends into the loop's end and plays on from there to the finish.
    pub fn finish(&mut self) {
        if !self.has_ending() || self.finishing {
            return;
        }
        self.finishing = true;
        let leave = if self.pos >= self.end {
            self.pos
        } else if self.bar > 0.0 {
            // The next bar with room for the blend before it.
            let from = (self.pos + self.seam) as f64 - self.start as f64;
            let bars = (from / self.bar).ceil();
            (self.start as f64 + bars * self.bar).round() as usize
        } else {
            self.pos + self.seam
        };
        self.leave_at = Some(leave.clamp(self.pos + self.seam, self.end.max(self.pos)));
    }

    /// On the way out of the loop, and then through the ending to the last frame.
    fn play_ending(&mut self, pcm: Arc<Pcm>) -> (f32, f32) {
        let frames = pcm.frames();
        if self.pos >= frames {
            self.done = true;
            return (0.0, 0.0);
        }
        let at = |i: usize| (pcm.data[i * 2] as f32, pcm.data[i * 2 + 1] as f32);
        let (mut l, mut r) = at(self.pos);
        if let Some(j) = self.leave_at {
            if self.seam > 0 && self.pos + self.seam >= j && j < self.end {
                let k = self.pos + self.seam - j;
                let t = k as f32 / self.seam as f32;
                let w = 0.5 - 0.5 * (t * std::f32::consts::PI).cos();
                let (l2, r2) = at(self.end - self.seam + k);
                l += (l2 - l) * w;
                r += (r2 - r) * w;
            }
        }
        self.pos += 1;
        if let Some(j) = self.leave_at {
            if self.pos >= j {
                self.pos = self.pos.max(self.end);
                self.leave_at = None;
            }
        }
        let g = self.gain / 32768.0;
        (l * g, r * g)
    }

    /// Next stereo frame (silence until it's decoded).
    pub fn next(&mut self) -> (f32, f32) {
        let Some(pcm) = &self.pcm else {
            return (0.0, 0.0);
        };
        if self.done {
            return (0.0, 0.0);
        }
        if self.finishing {
            return self.play_ending(pcm.clone());
        }
        if self.pos >= self.end {
            // A rest after the track's ending, before it starts again.
            self.pos += 1;
            if self.pos >= self.end + self.rest {
                self.pos = self.start;
            }
            return (0.0, 0.0);
        }
        let at = |i: usize| (pcm.data[i * 2] as f32, pcm.data[i * 2 + 1] as f32);
        let (mut l, mut r) = at(self.pos);
        // Over the last moments before the loop's end, blend into what leads up to its start,
        // so the jump back is seamless.
        if self.seam > 0 && self.pos + self.seam >= self.end {
            let k = self.pos + self.seam - self.end;
            let t = k as f32 / self.seam as f32;
            let w = 0.5 - 0.5 * (t * std::f32::consts::PI).cos();
            let (l2, r2) = at(self.start - self.seam + k);
            l += (l2 - l) * w;
            r += (r2 - r) * w;
        }
        self.pos += 1;
        if self.pos >= self.end && self.rest == 0 {
            self.pos = self.start;
        }
        let g = self.gain / 32768.0;
        (l * g, r * g)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_track_decodes_and_its_loop_fits() {
        for t in TRACKS {
            let pcm = decode(t.mp3).unwrap_or_else(|e| panic!("{:?}: {e}", t.song));
            assert!(
                t.start < t.end && t.end <= pcm.frames(),
                "{:?}: loop {}..{} in {} frames",
                t.song,
                t.start,
                t.end,
                pcm.frames()
            );
            assert!(
                (t.end - t.start) as f32 / RATE >= 20.0,
                "{:?}: loop too short",
                t.song
            );
            assert!(
                t.gain > 0.0 && t.gain <= 1.0,
                "{:?}: gain {}",
                t.song,
                t.gain
            );
        }
    }

    #[test]
    fn a_loop_goes_round_and_blends_its_seam() {
        // A ramp from 0 up: the loop jumps from frame 900 back to 300, blending over the seam.
        let data: Vec<i16> = (0..1000).flat_map(|i| [i as i16, i as i16]).collect();
        let t = Box::leak(Box::new(Track {
            song: Song::Title,
            mp3: &[],
            start: 300,
            end: 900,
            rest: 0,
            bpm: 0.0,
            gain: 1.0,
        }));
        let mut p = TrackPlayer::new(t, Some(Arc::new(Pcm { data })));
        let mut out = Vec::new();
        for _ in 0..1500 {
            out.push((p.next().0 * 32768.0).round() as i32);
        }
        assert_eq!(out[0], 0);
        assert_eq!(out[299], 299);
        // By the end of the seam it's playing what leads into the loop's start...
        assert!((out[899] - 299).abs() <= 1, "{}", out[899]);
        // ...and then carries on from the start.
        assert_eq!(out[900], 300);
        assert!((out[1499] - 299).abs() <= 1, "{}", out[1499]);
    }

    #[test]
    fn a_finished_loop_leaves_on_the_next_bar_and_plays_its_ending() {
        // Frames count up from 1. The loop is 2000..18000, four bars of 4000 frames; the
        // ending runs on to 20000.
        let data: Vec<i16> = (0..20000)
            .flat_map(|i| [i as i16 + 1, i as i16 + 1])
            .collect();
        let t = Box::leak(Box::new(Track {
            song: Song::Boss,
            mp3: &[],
            start: 2000,
            end: 18000,
            rest: 0,
            bpm: 4.0 * 60.0 * RATE / 4000.0,
            gain: 1.0,
        }));
        let mut p = TrackPlayer::new(t, Some(Arc::new(Pcm { data })));
        let mut out = Vec::new();
        for _ in 0..3000 {
            out.push((p.next().0 * 32768.0).round() as i32);
        }
        // Told to finish a quarter of the way into the loop's first bar...
        p.finish();
        for _ in 0..5100 {
            out.push((p.next().0 * 32768.0).round() as i32);
        }
        assert_eq!(out[2999], 3000);
        // ...it blends into what leads up to the loop's end over the moments before the next
        // bar (at 6000), carries on from the loop's end there, plays the ending to its last
        // frame and falls silent.
        assert!((out[5999] - 18000).abs() <= 1, "{}", out[5999]);
        assert_eq!(out[6000], 18001);
        assert_eq!(out[7999], 20000);
        assert!(p.done && out[8099] == 0);
    }

    #[test]
    fn leaving_right_before_a_bar_waits_for_the_one_after() {
        let data: Vec<i16> = (0..20000)
            .flat_map(|i| [i as i16 + 1, i as i16 + 1])
            .collect();
        let t = Box::leak(Box::new(Track {
            song: Song::Boss,
            mp3: &[],
            start: 2000,
            end: 18000,
            rest: 0,
            bpm: 4.0 * 60.0 * RATE / 4000.0,
            gain: 1.0,
        }));
        let mut p = TrackPlayer::new(t, Some(Arc::new(Pcm { data })));
        for _ in 0..5500 {
            p.next();
        }
        // Too close to the bar at 6000 to blend: it leaves at 10000 instead.
        p.finish();
        let mut out = Vec::new();
        for _ in 0..4600 {
            out.push((p.next().0 * 32768.0).round() as i32);
        }
        assert_eq!(out[1000 - 1000], 5501);
        assert!((out[4499] - 18000).abs() <= 1, "{}", out[4499]);
        assert_eq!(out[4500], 18001);
    }

    #[test]
    fn a_track_with_an_ending_rests_then_starts_over() {
        let data: Vec<i16> = (0..100)
            .flat_map(|i| [i as i16 + 1, i as i16 + 1])
            .collect();
        let t = Box::leak(Box::new(Track {
            song: Song::Title,
            mp3: &[],
            start: 0,
            end: 100,
            rest: 50,
            bpm: 0.0,
            gain: 1.0,
        }));
        let mut p = TrackPlayer::new(t, Some(Arc::new(Pcm { data })));
        let out: Vec<i32> = (0..300)
            .map(|_| (p.next().0 * 32768.0).round() as i32)
            .collect();
        assert_eq!((out[0], out[99]), (1, 100));
        assert!(out[100..150].iter().all(|&v| v == 0));
        assert_eq!((out[150], out[249]), (1, 100));
    }
}
