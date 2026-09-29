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
    /// How much to turn it down to sit with the rest of the game's sound.
    pub gain: f32,
}

/// Frames of quiet between the end of a track and its start again (on top of the second
/// left after its last sound).
const REST: usize = 22_050;

/// The songs that have a recorded track (made with Google's Lyria 3). Most are played all
/// the way through (from their first sound to a second after their last) and then started
/// again; one with a long stretch that repeats cleanly loops over it instead.
pub const TRACKS: &[Track] = &[
    // "The First Day of Spring"
    Track {
        song: Song::Title,
        mp3: include_bytes!("../../music/title.mp3"),
        start: 0,
        end: 6_703_200,
        rest: REST,
        gain: 0.655,
    },
    // "Seven AM Dew"
    Track {
        song: Song::Morning,
        mp3: include_bytes!("../../music/morning.mp3"),
        start: 0,
        end: 6_174_000,
        rest: REST,
        gain: 0.596,
    },
    // "Golden Hour at the Orchard"
    Track {
        song: Song::Afternoon,
        mp3: include_bytes!("../../music/afternoon.mp3"),
        start: 0,
        end: 7_904_925,
        rest: REST,
        gain: 0.655,
    },
    // "Moonlit Fence Posts"
    Track {
        song: Song::Night,
        mp3: include_bytes!("../../music/night.mp3"),
        start: 0,
        end: 6_989_850,
        rest: REST,
        gain: 0.643,
    },
    // "Cobblestone Promenade"
    Track {
        song: Song::Town,
        mp3: include_bytes!("../../music/town.mp3"),
        start: 70_560,
        end: 7_860_825,
        rest: REST,
        gain: 0.673,
    },
    // "Sunday Morning Curio"
    Track {
        song: Song::Shop,
        mp3: include_bytes!("../../music/shop.mp3"),
        start: 103_635,
        end: 7_750_575,
        rest: REST,
        gain: 0.629,
    },
    // "The Keeper's Hearth"
    Track {
        song: Song::Haven,
        mp3: include_bytes!("../../music/haven.mp3"),
        start: 0,
        end: 6_857_550,
        rest: REST,
        gain: 0.653,
    },
    // "Beneath the Glowing Cap"
    Track {
        song: Song::Burrows,
        mp3: include_bytes!("../../music/burrows.mp3"),
        start: 0,
        end: 6_714_225,
        rest: REST,
        gain: 0.668,
    },
    // "Below the Glacial Line": loops from the end of its quiet middle back to the end of
    // its quiet opening, where the full band comes in.
    Track {
        song: Song::Glimmer,
        mp3: include_bytes!("../../music/glimmer.mp3"),
        start: 775_988,
        end: 5_828_608,
        rest: 0,
        gain: 0.631,
    },
    // "Beneath the Burning Spire"
    Track {
        song: Song::Depths,
        mp3: include_bytes!("../../music/depths.mp3"),
        start: 0,
        end: 7_982_100,
        rest: REST,
        gain: 0.641,
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

    /// Next stereo frame (silence until it's decoded).
    pub fn next(&mut self) -> (f32, f32) {
        let Some(pcm) = &self.pcm else {
            return (0.0, 0.0);
        };
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
