//! Recorded music: tracks built into the game (Ogg Vorbis, kept small enough that the whole
//! game stays one modest file), decoded in the background the first time they're wanted, and
//! played through or looped seamlessly between two points. A song without a track is played
//! by the chiptune band in `music.rs`.

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
    /// The track, compressed (Ogg Vorbis, or MP3).
    pub audio: &'static [u8],
    /// The loop, in frames of the decoded track: it plays from the start to `end`, then
    /// carries on from `start`, round and round, the seam blended.
    pub start: usize,
    pub end: usize,
    /// Or, for a track with an ending of its own: frames of quiet after the end before it
    /// starts again from `start` (with no blending).
    pub rest: usize,
    /// A looping track's ending, for once the fight is won.
    pub ending: Option<&'static Ending>,
    /// How much to turn it down to sit with the rest of the game's sound.
    pub gain: f32,
}

/// The ending a looping track leaves its loop for, when told to finish.
pub struct Ending {
    /// The frames it plays, fading out over the last moments.
    pub from: usize,
    pub to: usize,
    /// Frames to a beat, and the points where the loop can be left for the ending in step
    /// with it, passage by passage: from each passage's first frame, at `at + k * beat`.
    /// With none, it leaves straight away.
    pub beat: f64,
    pub exits: &'static [(usize, f64)],
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
        audio: include_bytes!("../../music/ogg/title.ogg"),
        start: 0,
        end: 6_703_200,
        rest: REST,
        ending: None,
        gain: 0.655,
    },
    // "Seven AM Dew"
    Track {
        song: Song::Morning,
        audio: include_bytes!("../../music/ogg/morning.ogg"),
        start: 0,
        end: 6_174_000,
        rest: REST,
        ending: None,
        gain: 0.596,
    },
    // "Golden Hour at the Orchard"
    Track {
        song: Song::Afternoon,
        audio: include_bytes!("../../music/ogg/afternoon.ogg"),
        start: 0,
        end: 7_904_925,
        rest: REST,
        ending: None,
        gain: 0.655,
    },
    // "Moonlit Fence Posts"
    Track {
        song: Song::Night,
        audio: include_bytes!("../../music/ogg/night.ogg"),
        start: 0,
        end: 6_989_850,
        rest: REST,
        ending: None,
        gain: 0.643,
    },
    // "Cobblestone Promenade"
    Track {
        song: Song::Town,
        audio: include_bytes!("../../music/ogg/town.ogg"),
        start: 70_560,
        end: 7_860_825,
        rest: REST,
        ending: None,
        gain: 0.673,
    },
    // "Sunday Morning Curio"
    Track {
        song: Song::Shop,
        audio: include_bytes!("../../music/ogg/shop.ogg"),
        start: 103_635,
        end: 7_750_575,
        rest: REST,
        ending: None,
        gain: 0.629,
    },
    // "The Keeper's Hearth"
    Track {
        song: Song::Haven,
        audio: include_bytes!("../../music/ogg/haven.ogg"),
        start: 0,
        end: 6_857_550,
        rest: REST,
        ending: None,
        gain: 0.653,
    },
    // "Beneath the Glowing Cap"
    Track {
        song: Song::Burrows,
        audio: include_bytes!("../../music/ogg/burrows.ogg"),
        start: 0,
        end: 6_714_225,
        rest: REST,
        ending: None,
        gain: 0.668,
    },
    // "Below the Glacial Line"
    Track {
        song: Song::Glimmer,
        audio: include_bytes!("../../music/ogg/glimmer.ogg"),
        start: 0,
        end: 7_089_075,
        rest: REST,
        ending: None,
        gain: 0.666,
    },
    // "Beneath the Burning Spire"
    Track {
        song: Song::Depths,
        audio: include_bytes!("../../music/ogg/depths.ogg"),
        start: 0,
        end: 7_982_100,
        rest: REST,
        ending: None,
        gain: 0.641,
    },
    // "The Seventh Gate": loops over its middle, breakdown and all, so a fight never stops
    // for an ending; once the guardian falls it leaves the loop for the track's last bars.
    Track {
        song: Song::Boss,
        audio: include_bytes!("../../music/ogg/boss.ogg"),
        start: 2_232_024,
        end: 5_763_072,
        rest: 0,
        ending: Some(&SEVENTH_GATE_ENDING),
        gain: 0.700,
    },
];

/// "The Seventh Gate" keeps a steady 145 beats a minute.
const SEVENTH_GATE_BEAT: f64 = 60.0 * RATE as f64 / 145.0;

/// The guardians' ending: the last three and a half seconds of the track, from the hit on the
/// offbeat at 156.30 s to 159.77 s. The loop is left for it on an offbeat too, so the ending's
/// beats carry on the loop's.
const SEVENTH_GATE_ENDING: Ending = Ending {
    from: 6_892_990,
    to: 7_045_945,
    beat: SEVENTH_GATE_BEAT,
    exits: &[
        // Offbeats: the accents fall on 4266 + k * beat...
        (0, 4_266.0 + SEVENTH_GATE_BEAT / 2.0),
        // ...then half a beat later from 41.5 s...
        (1_830_150, 4_266.0),
        // ...and back from 86.5 s.
        (3_813_768, 4_266.0 + SEVENTH_GATE_BEAT / 2.0),
    ],
};

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

/// Decodes a track (Ogg Vorbis or MP3, told apart by their contents) to stereo frames at the
/// mixer's rate.
pub fn decode(audio: &'static [u8]) -> Result<Pcm, String> {
    let src = ReadOnlySource::new(std::io::Cursor::new(audio));
    let mss = MediaSourceStream::new(Box::new(src), Default::default());
    let probed = symphonia::default::get_probe()
        .format(
            &Hint::new(),
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

/// How long the seam of a loop is blended over, in frames (40 ms): the way into an ending
/// too, and its fade at the last.
const SEAM: usize = 1764;

/// Rises from 0 to 1 as `t` goes from 0 to 1, smoothly at both ends.
fn ease(t: f32) -> f32 {
    0.5 - 0.5 * (t * std::f32::consts::PI).cos()
}

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
    ending: Option<&'static Ending>,
    /// The ending's frames, as they fit the decoded track.
    ending_at: (usize, usize),
    /// Where it leaves the loop for its ending, once told to finish.
    leave_at: Option<usize>,
    /// It has left the loop and is playing its ending.
    in_ending: bool,
    /// It has played its ending to the last frame.
    pub done: bool,
}

impl TrackPlayer {
    /// A player for a track, from an already decoded copy, or decoding it in the background.
    pub fn new(t: &'static Track, pcm: Option<Arc<Pcm>>) -> TrackPlayer {
        let loading = if pcm.is_none() {
            let (tx, rx) = channel();
            let audio = t.audio;
            let spawned = std::thread::Builder::new()
                .name("hollowbloom-decode".into())
                .spawn(move || {
                    let _ = tx.send(decode(audio));
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
            ending: if t.rest > 0 { None } else { t.ending },
            ending_at: t.ending.map_or((0, 0), |e| (e.from, e.to)),
            leave_at: None,
            in_ending: false,
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
        let (from, to) = self.ending_at;
        let to = to.min(frames);
        self.ending_at = (from.clamp(self.seam, to.max(self.seam)), to);
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

    /// Whether it has an ending to play when told to finish (a looping track can).
    pub fn has_ending(&self) -> bool {
        self.ending.is_some() && self.ending_at.0 < self.ending_at.1 && !self.done
    }

    /// Leaves the loop for the track's ending: at the next point in step with it, blending
    /// into what leads up to the ending over the moments before.
    pub fn finish(&mut self) {
        if !self.has_ending() || self.leave_at.is_some() || self.in_ending {
            return;
        }
        let leave = self
            .exit_after(self.pos + self.seam)
            // None left before the loop's end: the first once round again.
            .or_else(|| self.exit_after(self.start + self.seam))
            .unwrap_or(self.end);
        self.leave_at = Some(leave);
    }

    /// The first point at or after `from`, and no later than the loop's end, where the loop
    /// can be left for its ending in step.
    fn exit_after(&self, from: usize) -> Option<usize> {
        let e = self.ending?;
        let exit = if e.exits.is_empty() || e.beat <= 0.0 {
            Some(from)
        } else {
            e.exits.iter().enumerate().find_map(|(i, &(first, at))| {
                let until = e.exits.get(i + 1).map_or(usize::MAX, |n| n.0);
                let from = from.max(first);
                let k = ((from as f64 - at) / e.beat).ceil();
                let exit = (at + k * e.beat).round() as usize;
                (exit < until).then_some(exit)
            })
        };
        exit.filter(|&x| x <= self.end)
    }

    /// Next stereo frame (silence until it's decoded).
    pub fn next(&mut self) -> (f32, f32) {
        let Some(pcm) = &self.pcm else {
            return (0.0, 0.0);
        };
        if self.done {
            return (0.0, 0.0);
        }
        let at = |i: usize| (pcm.data[i * 2] as f32, pcm.data[i * 2 + 1] as f32);
        let g = self.gain / 32768.0;
        if self.in_ending {
            // The ending, faded out over its last moments.
            let (l, r) = at(self.pos);
            let left = self.ending_at.1 - self.pos;
            let w = if left < SEAM {
                ease(left as f32 / SEAM as f32)
            } else {
                1.0
            };
            self.pos += 1;
            self.done = self.pos >= self.ending_at.1;
            return (l * g * w, r * g * w);
        }
        if self.pos >= self.end {
            // A rest after the track's ending, before it starts again.
            self.pos += 1;
            if self.pos >= self.end + self.rest {
                self.pos = self.start;
            }
            return (0.0, 0.0);
        }
        let (mut l, mut r) = at(self.pos);
        // Over the moments before a way out of the loop (or its end), blend into what leads
        // up to the ending (or the loop's start), so the jump is seamless.
        let blend = match self.leave_at {
            Some(j) if self.pos < j && self.pos + self.seam >= j => {
                Some((self.pos + self.seam - j, self.ending_at.0))
            }
            _ if self.seam > 0 && self.pos + self.seam >= self.end => {
                Some((self.pos + self.seam - self.end, self.start))
            }
            _ => None,
        };
        if let Some((k, to)) = blend {
            let w = ease(k as f32 / self.seam as f32);
            let (l2, r2) = at(to - self.seam + k);
            l += (l2 - l) * w;
            r += (r2 - r) * w;
        }
        self.pos += 1;
        if self.leave_at == Some(self.pos) {
            self.pos = self.ending_at.0;
            self.leave_at = None;
            self.in_ending = true;
            self.done = self.pos >= self.ending_at.1;
        } else if self.pos >= self.end && self.rest == 0 {
            self.pos = self.start;
        }
        (l * g, r * g)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_track_decodes_and_its_loop_fits() {
        for t in TRACKS {
            let pcm = decode(t.audio).unwrap_or_else(|e| panic!("{:?}: {e}", t.song));
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
            if let Some(e) = t.ending {
                assert_eq!(t.rest, 0, "{:?}: only a loop leaves for an ending", t.song);
                assert!(
                    SEAM <= e.from && e.from < e.to && e.to <= pcm.frames(),
                    "{:?}: ending {}..{} in {} frames",
                    t.song,
                    e.from,
                    e.to,
                    pcm.frames()
                );
                assert!(
                    e.beat > 0.0 && e.exits.windows(2).all(|w| w[0].0 < w[1].0),
                    "{:?}: exits",
                    t.song
                );
            }
        }
    }

    /// A player over a ramp counting up from `first`, for a made-up track.
    fn ramp(first: i16, frames: usize, t: Track) -> TrackPlayer {
        let data: Vec<i16> = (0..frames)
            .flat_map(|i| [i as i16 + first, i as i16 + first])
            .collect();
        TrackPlayer::new(Box::leak(Box::new(t)), Some(Arc::new(Pcm { data })))
    }

    fn take(p: &mut TrackPlayer, n: usize) -> Vec<i32> {
        (0..n)
            .map(|_| (p.next().0 * 32768.0).round() as i32)
            .collect()
    }

    #[test]
    fn a_loop_goes_round_and_blends_its_seam() {
        // A ramp from 0 up: the loop jumps from frame 900 back to 300, blending over the seam.
        let mut p = ramp(
            0,
            1000,
            Track {
                song: Song::Title,
                audio: &[],
                start: 300,
                end: 900,
                rest: 0,
                ending: None,
                gain: 1.0,
            },
        );
        let out = take(&mut p, 1500);
        assert_eq!(out[0], 0);
        assert_eq!(out[299], 299);
        // By the end of the seam it's playing what leads into the loop's start...
        assert!((out[899] - 299).abs() <= 1, "{}", out[899]);
        // ...and then carries on from the start.
        assert_eq!(out[900], 300);
        assert!((out[1499] - 299).abs() <= 1, "{}", out[1499]);
    }

    /// Frames count up from 1. The loop is 2000..18000 and the ending 22000..26000; the loop
    /// can be left at 1000 + 4000k until 10000, and at 3000 + 4000k from there on.
    fn guardian() -> TrackPlayer {
        static ENDING: Ending = Ending {
            from: 22000,
            to: 26000,
            beat: 4000.0,
            exits: &[(0, 1000.0), (10000, 3000.0)],
        };
        ramp(
            1,
            30000,
            Track {
                song: Song::Boss,
                audio: &[],
                start: 2000,
                end: 18000,
                rest: 0,
                ending: Some(&ENDING),
                gain: 1.0,
            },
        )
    }

    #[test]
    fn a_finished_loop_leaves_in_step_and_plays_its_ending() {
        let mut p = guardian();
        let mut out = take(&mut p, 3000);
        // Told to finish a little way into the loop...
        p.finish();
        out.extend(take(&mut p, 6100));
        assert_eq!(out[2999], 3000);
        // ...it plays on to the moments before the next way out (at 5000), blends into what
        // leads up to the ending over them, and carries on into the ending there...
        assert_eq!(out[3235], 3236);
        assert!((out[4999] - 22000).abs() <= 1, "{}", out[4999]);
        assert_eq!(out[5000], 22001);
        // ...which plays through, fades out at the last and falls silent.
        assert_eq!(out[7000], 24001);
        assert!(out[8999].abs() <= 1, "{}", out[8999]);
        assert!(p.done && out[9000..].iter().all(|&v| v == 0));
    }

    #[test]
    fn finishing_right_before_a_way_out_waits_for_the_next() {
        let mut p = guardian();
        take(&mut p, 4500);
        // Too close to 5000 to blend: it leaves at 9000 instead.
        p.finish();
        let out = take(&mut p, 4600);
        assert_eq!(out[0], 4501);
        assert!((out[4499] - 22000).abs() <= 1, "{}", out[4499]);
        assert_eq!(out[4500], 22001);
    }

    #[test]
    fn the_ways_out_move_with_the_passage() {
        let mut p = guardian();
        take(&mut p, 9500);
        // 13000 would be in step with the passage before 10000, but not with this one: 15000.
        p.finish();
        let out = take(&mut p, 5600);
        assert_eq!(out[3499], 13000);
        assert_eq!(out[5500], 22001);
    }

    #[test]
    fn finishing_near_the_loops_end_leaves_once_round_again() {
        let mut p = guardian();
        take(&mut p, 16000);
        p.finish();
        let out = take(&mut p, 5100);
        // Round the loop (seam and all) and on to the first way out after its start.
        assert_eq!(out[235], 16236);
        assert!((out[1999] - 2000).abs() <= 1, "{}", out[1999]);
        assert_eq!(out[2000], 2001);
        assert!((out[4999] - 22000).abs() <= 1, "{}", out[4999]);
        assert_eq!(out[5000], 22001);
    }

    #[test]
    fn a_track_played_through_rests_then_starts_over() {
        let mut p = ramp(
            1,
            100,
            Track {
                song: Song::Title,
                audio: &[],
                start: 0,
                end: 100,
                rest: 50,
                ending: None,
                gain: 1.0,
            },
        );
        // It has no ending to leave for.
        assert!(!p.has_ending());
        p.finish();
        let out = take(&mut p, 300);
        assert_eq!((out[0], out[99]), (1, 100));
        assert!(out[100..150].iter().all(|&v| v == 0));
        assert_eq!((out[150], out[249]), (1, 100));
    }
}
