//! A tiny chiptune band and the sequencer that plays the game's songs (written out in
//! `songs.rs`).
//!
//! A song is sections of chords, a tune and a counter-tune, played in the order of its form.
//! Everything is laid out on one grid of sixteenth notes before the song starts: the tune
//! and counter-tune as written, the chords broken into the song's arpeggio and walked by its
//! bass line, and the drums. The band has five players, one for each of those parts.

use super::songs;
use super::synth::RATE;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Song {
    /// The title screen, and the end of each day.
    Title,
    /// The farm before noon.
    Morning,
    /// The farm from noon until nightfall.
    Afternoon,
    Night,
    /// Bramblewick by day.
    Town,
    /// Inside the shops.
    Shop,
    /// The farmhouse by day, and the Hollow's waystone floors.
    Haven,
    /// The Mossy Burrows and the Fungal Hollow.
    Burrows,
    /// The Crystal Grotto and the Frost Caverns.
    Glimmer,
    /// The Ember Depths and the Sunken Ruins.
    Depths,
    /// A guardian awake.
    Boss,
}

/// Every song, for tools and tests.
pub const SONGS: &[Song] = &[
    Song::Title,
    Song::Morning,
    Song::Afternoon,
    Song::Night,
    Song::Town,
    Song::Shop,
    Song::Haven,
    Song::Burrows,
    Song::Glimmer,
    Song::Depths,
    Song::Boss,
];

impl Song {
    pub fn name(self) -> &'static str {
        match self {
            Song::Title => "title",
            Song::Morning => "morning",
            Song::Afternoon => "afternoon",
            Song::Night => "night",
            Song::Town => "town",
            Song::Shop => "shop",
            Song::Haven => "haven",
            Song::Burrows => "burrows",
            Song::Glimmer => "glimmer",
            Song::Depths => "depths",
            Song::Boss => "boss",
        }
    }

    fn tune(self) -> &'static Tune {
        match self {
            Song::Title => &songs::TITLE,
            Song::Morning => &songs::MORNING,
            Song::Afternoon => &songs::AFTERNOON,
            Song::Night => &songs::NIGHT,
            Song::Town => &songs::TOWN,
            Song::Shop => &songs::SHOP,
            Song::Haven => &songs::HAVEN,
            Song::Burrows => &songs::BURROWS,
            Song::Glimmer => &songs::GLIMMER,
            Song::Depths => &songs::DEPTHS,
            Song::Boss => &songs::BOSS,
        }
    }
}

/// The band's instruments.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Inst {
    /// A square-and-triangle lead with a little vibrato.
    Lead,
    /// A soft chime that fades away.
    Bell,
    /// A gentle triangle that swells in and holds.
    Soft,
    /// A triangle bass.
    Bass,
    /// A thin plucked square that dies away quickly.
    Arp,
    Drums,
}

/// A song for the band.
pub struct Tune {
    pub bpm: f32,
    /// Eighth notes to a bar: 8 for four beats, 6 for a waltz's three.
    pub bar: usize,
    /// The sections' names in the order they're played.
    pub form: &'static str,
    /// Where in the form to start again after the end (so an intro plays only once).
    pub repeat: usize,
    /// Instrument and loudness for the tune, counter-tune, chords, bass and drums.
    pub band: [(Inst, f32); 5],
    /// How each bar's chord is broken up: an index into the chord (0 its lowest note, one
    /// past its top note the bottom again an octave up) for each step, "." resting.
    pub arp: &'static str,
    /// The bass line's shape for each bar: R the root, 3 the third, 5 the fifth, 7 the
    /// seventh, 8 the root an octave up, a a step into the next chord's root.
    pub bass: &'static str,
    pub sections: &'static [Section],
}

pub struct Section {
    pub name: char,
    /// A chord for each bar, "|" between bars; two chords in one bar split it in half.
    pub chords: &'static str,
    /// Eighth notes (or sixteenths), "-" holding the one before and "." resting.
    pub tune: &'static str,
    pub counter: &'static str,
    /// One bar of drums, repeated: k kick, s snare, h hat.
    pub drums: &'static str,
}

const TUNE: usize = 0;
const COUNTER: usize = 1;
const CHORDS: usize = 2;
const BASS: usize = 3;
const DRUMS: usize = 4;
const PARTS: usize = 5;

/// Grid steps (sixteenths) to an eighth note.
const T8: usize = 2;

/// What a player does at one step of the grid.
#[derive(Clone, Copy, PartialEq, Debug)]
enum Cell {
    Keep,
    Note(f32),
    Off,
    Kick,
    Snare,
    Hat,
}

#[derive(Clone, Copy, PartialEq, Debug)]
struct Chord {
    root: i32,
    /// The note under it (the root, or another for a slash chord).
    bass: i32,
    /// Semitones above the root.
    tones: [i32; 4],
    n: usize,
}

impl Chord {
    fn pcs(&self) -> impl Iterator<Item = i32> + '_ {
        self.tones[..self.n]
            .iter()
            .map(|t| (self.root + t).rem_euclid(12))
    }
}

fn letter(c: char) -> Option<i32> {
    Some(match c {
        'C' => 0,
        'D' => 2,
        'E' => 4,
        'F' => 5,
        'G' => 7,
        'A' => 9,
        'B' => 11,
        _ => return None,
    })
}

/// Splits a sharp or flat off the front of what follows a note's letter.
fn accidental(rest: &str) -> (i32, &str) {
    if let Some(r) = rest.strip_prefix('#') {
        (1, r)
    } else if let Some(r) = rest.strip_prefix('b') {
        (-1, r)
    } else {
        (0, rest)
    }
}

/// A note ("C5", "F#4", "Bb3") as a MIDI note number.
fn parse_note(tok: &str) -> Option<f32> {
    let mut chars = tok.chars();
    let base = letter(chars.next()?)?;
    let (acc, oct) = accidental(chars.as_str());
    let octave: i32 = oct.parse().ok()?;
    Some((12 * (octave + 1) + base + acc) as f32)
}

/// A chord symbol: "C", "Am", "G7", "Fmaj7", "Bbm6", "D/A"...
fn parse_chord(sym: &str) -> Option<Chord> {
    let (main, over) = match sym.split_once('/') {
        Some((m, b)) => (m, Some(b)),
        None => (sym, None),
    };
    let mut chars = main.chars();
    let base = letter(chars.next()?)?;
    let (acc, quality) = accidental(chars.as_str());
    let root = (base + acc).rem_euclid(12);
    let tones: &[i32] = match quality {
        "" => &[0, 4, 7],
        "m" => &[0, 3, 7],
        "7" => &[0, 4, 7, 10],
        "maj7" => &[0, 4, 7, 11],
        "m7" => &[0, 3, 7, 10],
        "6" => &[0, 4, 7, 9],
        "m6" => &[0, 3, 7, 9],
        "dim" => &[0, 3, 6],
        "dim7" => &[0, 3, 6, 9],
        "m7b5" => &[0, 3, 6, 10],
        "aug" => &[0, 4, 8],
        "sus2" => &[0, 2, 7],
        "sus4" => &[0, 5, 7],
        "7sus4" => &[0, 5, 7, 10],
        "mmaj7" => &[0, 3, 7, 11],
        _ => return None,
    };
    let bass = match over {
        Some(b) => {
            let mut chars = b.chars();
            let base = letter(chars.next()?)?;
            let (acc, rest) = accidental(chars.as_str());
            if !rest.is_empty() {
                return None;
            }
            (base + acc).rem_euclid(12)
        }
        None => root,
    };
    let mut t = [0; 4];
    t[..tones.len()].copy_from_slice(tones);
    Some(Chord {
        root,
        bass,
        tones: t,
        n: tones.len(),
    })
}

/// A written line of eighth notes (or sixteenths) spread over `bars` bars of the grid.
fn parse_line(text: &str, bars: usize, bar: usize) -> Result<Vec<Cell>, String> {
    let toks: Vec<&str> = text.split_whitespace().filter(|t| *t != "|").collect();
    let ticks = bars * bar * T8;
    if toks.is_empty() || ticks % toks.len() != 0 {
        return Err(format!(
            "{} notes don't fill {bars} bars of {bar} eighths",
            toks.len()
        ));
    }
    let step = ticks / toks.len();
    if step != 1 && step != 2 {
        return Err(format!(
            "{} notes over {bars} bars are neither eighths nor sixteenths",
            toks.len()
        ));
    }
    let segs: Vec<&str> = text.split('|').collect();
    if segs.len() > 1 {
        if segs.len() != bars {
            return Err(format!(
                "{} bars written where the chords have {bars}",
                segs.len()
            ));
        }
        let per = toks.len() / bars;
        for (i, s) in segs.iter().enumerate() {
            let n = s.split_whitespace().count();
            if n != per {
                return Err(format!("bar {} has {n} steps, not {per}", i + 1));
            }
        }
    }
    let mut out = vec![Cell::Keep; ticks];
    for (i, t) in toks.iter().enumerate() {
        out[i * step] = match *t {
            "-" => Cell::Keep,
            "." => Cell::Off,
            n => Cell::Note(parse_note(n).ok_or_else(|| format!("no such note {n:?}"))?),
        };
    }
    Ok(out)
}

/// One bar of a repeating pattern: where each step falls in the bar, and what it says.
fn parse_pattern(text: &str, bar: usize) -> Result<Vec<(usize, &str)>, String> {
    let toks: Vec<&str> = text.split_whitespace().collect();
    let bar_ticks = bar * T8;
    if toks.is_empty() || bar_ticks % toks.len() != 0 {
        return Err(format!("pattern {text:?} doesn't fit a bar"));
    }
    let step = bar_ticks / toks.len();
    Ok(toks
        .into_iter()
        .enumerate()
        .map(|(i, t)| (i * step, t))
        .collect())
}

/// A chord stacked close together around `center`, in whichever inversion moves least from
/// the chord before (so the arpeggio glides from chord to chord rather than jumping).
fn voicing(c: &Chord, prev: &[i32], center: i32) -> Vec<i32> {
    let pcs: Vec<i32> = c.pcs().collect();
    let n = pcs.len();
    let mut best: Option<(i32, Vec<i32>)> = None;
    for inv in 0..n {
        let mut v: Vec<i32> = Vec::with_capacity(n);
        for k in 0..n {
            let mut m = pcs[(inv + k) % n] + 24;
            if let Some(&last) = v.last() {
                while m <= last {
                    m += 12;
                }
            }
            v.push(m);
        }
        for shift in 0..6 {
            let w: Vec<i32> = v.iter().map(|m| m + 12 * shift).collect();
            let mid = (w[0] + w[n - 1]) / 2;
            let mut cost = 2 * (mid - center).abs();
            if prev.len() == n {
                cost += prev.iter().zip(&w).map(|(a, b)| (a - b).abs()).sum::<i32>();
            }
            if best.as_ref().is_none_or(|(b, _)| cost < *b) {
                best = Some((cost, w));
            }
        }
    }
    best.map(|(_, v)| v).unwrap_or_default()
}

/// Where the arpeggios sit: around D4.
const ARP_CENTER: i32 = 62;

/// A bass note for a pitch class, between Bb1 and A2.
fn bass_pitch(pc: i32) -> i32 {
    let m = 36 + pc.rem_euclid(12);
    if m > 45 { m - 12 } else { m }
}

/// A song laid out on the grid.
struct Score {
    cells: Vec<[Cell; PARTS]>,
    loop_tick: usize,
    /// The chords under every step, where each section starts (and its name) and the
    /// length of a bar: for the tests, which check the tunes against the chords.
    #[cfg_attr(not(test), allow(dead_code))]
    chords: Vec<Chord>,
    #[cfg_attr(not(test), allow(dead_code))]
    starts: Vec<(usize, char)>,
    #[cfg_attr(not(test), allow(dead_code))]
    bar_ticks: usize,
}

fn compile(t: &Tune) -> Result<Score, String> {
    let bar_ticks = t.bar * T8;
    let mut form = Vec::new();
    for name in t.form.chars().filter(|c| !c.is_whitespace()) {
        let s = t
            .sections
            .iter()
            .find(|s| s.name == name)
            .ok_or_else(|| format!("the form names a section {name:?} that isn't written"))?;
        form.push(s);
    }
    // The chords under every step of the song.
    let mut chords = Vec::new();
    let mut starts = Vec::new();
    for s in &form {
        starts.push((chords.len(), s.name));
        for bar in s.chords.split('|') {
            let syms: Vec<&str> = bar.split_whitespace().collect();
            if syms.is_empty() || bar_ticks % syms.len() != 0 {
                return Err(format!(
                    "section {}: can't split a bar into {bar:?}",
                    s.name
                ));
            }
            let each = bar_ticks / syms.len();
            for sym in syms {
                let c = parse_chord(sym)
                    .ok_or_else(|| format!("section {}: no such chord {sym:?}", s.name))?;
                chords.extend(std::iter::repeat_n(c, each));
            }
        }
    }
    let total = chords.len();
    let loop_tick = starts
        .get(t.repeat)
        .map(|s| s.0)
        .ok_or("the form is shorter than where it repeats from")?;
    let mut cells = vec![[Cell::Keep; PARTS]; total];

    // Each section's own tune, counter-tune and drums.
    for (i, s) in form.iter().enumerate() {
        let start = starts[i].0;
        let end = starts.get(i + 1).map_or(total, |n| n.0);
        let bars = (end - start) / bar_ticks;
        for (part, text) in [(TUNE, s.tune), (COUNTER, s.counter)] {
            if text.is_empty() {
                // Nothing to play here: don't let a note from before ring on.
                cells[start][part] = Cell::Off;
                continue;
            }
            let line =
                parse_line(text, bars, t.bar).map_err(|e| format!("section {}: {e}", s.name))?;
            for (k, c) in line.into_iter().enumerate() {
                cells[start + k][part] = c;
            }
        }
        if !s.drums.is_empty() {
            let pat = parse_pattern(s.drums, t.bar)?;
            for b in 0..bars {
                for &(off, tok) in &pat {
                    cells[start + b * bar_ticks + off][DRUMS] = match tok {
                        "k" => Cell::Kick,
                        "s" => Cell::Snare,
                        "h" => Cell::Hat,
                        "." | "-" => Cell::Keep,
                        _ => return Err(format!("no such drum {tok:?}")),
                    };
                }
            }
        }
    }

    // The next chord after a step (round the loop if need be), for a bass line leading into it.
    let next_chord = |tick: usize| -> Chord {
        let now = chords[tick];
        let ahead = (tick + 1..total).chain(loop_tick..tick);
        ahead.map(|k| chords[k]).find(|c| *c != now).unwrap_or(now)
    };

    // The arpeggio and the bass line, from the chords.
    let arp = parse_pattern(t.arp, t.bar)?;
    let bass = parse_pattern(t.bass, t.bar)?;
    let mut shape: Vec<i32> = Vec::new();
    let mut shaped: Option<Chord> = None;
    for bar_start in (0..total).step_by(bar_ticks) {
        for &(off, tok) in &arp {
            let tick = bar_start + off;
            let c = chords[tick];
            if shaped != Some(c) {
                shape = voicing(&c, &shape, ARP_CENTER);
                shaped = Some(c);
            }
            cells[tick][CHORDS] = match tok {
                "-" => Cell::Keep,
                "." => Cell::Off,
                d => {
                    let i: usize = d.parse().map_err(|_| format!("arpeggio step {d:?}"))?;
                    let n = shape.len();
                    Cell::Note((shape[i % n] + 12 * (i / n) as i32) as f32)
                }
            };
        }
        for &(off, tok) in &bass {
            let tick = bar_start + off;
            let c = chords[tick];
            let r = bass_pitch(c.bass);
            let root = bass_pitch(c.root);
            let note = match tok {
                "-" => None,
                "." => {
                    cells[tick][BASS] = Cell::Off;
                    None
                }
                "R" => Some(r),
                "8" => Some(r + 12),
                "3" => Some(root + c.tones[1]),
                "5" => Some(root + 7),
                "7" => Some(root + if c.n > 3 { c.tones[3] } else { 10 }),
                "a" => {
                    let target = bass_pitch(next_chord(tick).bass);
                    Some(if target > r { target - 1 } else { target + 1 })
                }
                _ => return Err(format!("no such bass step {tok:?}")),
            };
            if let Some(m) = note {
                cells[tick][BASS] = Cell::Note(m as f32);
            }
        }
    }
    Ok(Score {
        cells,
        loop_tick,
        chords,
        starts,
        bar_ticks,
    })
}

/// Every note a part plays: (grid step, how many steps it lasts, MIDI note).
fn notes(score: &Score, part: usize) -> Vec<(usize, usize, f32)> {
    let mut out: Vec<(usize, usize, f32)> = Vec::new();
    let mut open: Option<(usize, f32)> = None;
    for (tick, row) in score.cells.iter().enumerate() {
        match row[part] {
            Cell::Note(m) => {
                if let Some((s, n)) = open.take() {
                    out.push((s, tick - s, n));
                }
                open = Some((tick, m));
            }
            Cell::Off => {
                if let Some((s, n)) = open.take() {
                    out.push((s, tick - s, n));
                }
            }
            _ => {}
        }
    }
    if let Some((s, n)) = open {
        out.push((s, score.cells.len() - s, n));
    }
    out
}

#[derive(Default)]
struct Chan {
    freq: f32,
    phase: f32,
    env: f32,
    gate: bool,
    age: f32,
    lp: f32,
    drum: Option<(Cell, f32)>,
    noise: u32,
    /// Samples left in the breath taken before a note repeats, and the note to play then.
    dip: u32,
    pending: f32,
    /// A chime or pluck rising to its peak.
    attack: bool,
}

/// Samples of breath between a note and the same note again, and how fast it falls silent.
const DIP: u32 = 180;
const DIP_FALL: f32 = 0.97;

impl Chan {
    fn play(&mut self, freq: f32, inst: Inst) {
        let repeat = (freq - self.freq).abs() < 0.01;
        if repeat && self.env > 0.02 && matches!(inst, Inst::Lead | Inst::Soft | Inst::Bass) {
            // The same note again: part it from the one before so it's heard.
            self.dip = DIP;
            self.pending = freq;
        } else {
            self.start(freq, inst);
        }
    }

    fn start(&mut self, freq: f32, inst: Inst) {
        self.freq = freq;
        self.gate = true;
        self.age = 0.0;
        self.attack = matches!(inst, Inst::Bell | Inst::Arp);
    }
}

pub struct Player {
    score: Score,
    band: [(Inst, f32); PARTS],
    tick: usize,
    sample_in_tick: f32,
    samples_per_tick: f32,
    chans: [Chan; PARTS],
    transpose: f32,
}

fn midi_hz(m: f32) -> f32 {
    440.0 * 2f32.powf((m - 69.0) / 12.0)
}

impl Player {
    pub fn new(song: Song, transpose: i32, tempo: f32) -> Player {
        let t = song.tune();
        let score = compile(t).unwrap_or_else(|e| {
            // The tests compile every song, so this can't happen in a release.
            eprintln!("hollowbloom: song {:?} doesn't compile: {e}", song);
            Score {
                cells: vec![[Cell::Keep; PARTS]],
                loop_tick: 0,
                chords: Vec::new(),
                starts: Vec::new(),
                bar_ticks: 1,
            }
        });
        Player {
            score,
            band: t.band,
            tick: 0,
            sample_in_tick: 0.0,
            samples_per_tick: RATE * 60.0 / (t.bpm * tempo * 4.0),
            chans: Default::default(),
            transpose: transpose as f32,
        }
    }

    /// Every note the song plays, once through: (part, start and length in seconds, MIDI
    /// note), the parts being the tune, counter-tune, chords and bass.
    pub fn written(&self) -> Vec<(usize, f32, f32, f32)> {
        let secs = |ticks: usize| ticks as f32 * self.samples_per_tick / RATE;
        let mut out = Vec::new();
        for part in [TUNE, COUNTER, CHORDS, BASS] {
            for (s, len, m) in notes(&self.score, part) {
                out.push((part, secs(s), secs(len), m + self.transpose));
            }
        }
        out
    }

    /// Seconds from the start to the end, and of each time round after that.
    pub fn length(&self) -> (f32, f32) {
        let secs = |ticks: usize| ticks as f32 * self.samples_per_tick / RATE;
        let total = self.score.cells.len();
        (secs(total), secs(total - self.score.loop_tick))
    }

    fn trigger(&mut self) {
        let row = self.score.cells[self.tick];
        for (part, cell) in row.iter().enumerate() {
            let inst = self.band[part].0;
            let ch = &mut self.chans[part];
            match *cell {
                Cell::Keep => {}
                Cell::Note(m) => ch.play(midi_hz(m + self.transpose), inst),
                Cell::Off => ch.gate = false,
                Cell::Kick | Cell::Snare | Cell::Hat => {
                    ch.drum = Some((*cell, 0.0));
                    ch.phase = 0.0;
                }
            }
        }
    }

    /// Next mono sample.
    pub fn next(&mut self) -> f32 {
        if self.sample_in_tick <= 0.0 {
            self.trigger();
            self.sample_in_tick += self.samples_per_tick;
            self.tick += 1;
            if self.tick >= self.score.cells.len() {
                self.tick = self.score.loop_tick;
            }
        }
        self.sample_in_tick -= 1.0;
        let dt = 1.0 / RATE;
        let mut out = 0.0;
        for (part, &(inst, gain)) in self.band.iter().enumerate() {
            if gain <= 0.0 {
                continue;
            }
            let ch = &mut self.chans[part];
            ch.age += dt;
            let s = match inst {
                Inst::Drums => {
                    let mut v = 0.0;
                    if let Some((kind, age)) = ch.drum.as_mut() {
                        *age += dt;
                        ch.noise = ch.noise.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
                        let n = (ch.noise >> 9) as f32 / (1u32 << 23) as f32 * 2.0 - 1.0;
                        v = match kind {
                            Cell::Kick => {
                                let f = 45.0 + 110.0 * (-*age * 30.0).exp();
                                ch.phase += f * dt;
                                (ch.phase * std::f32::consts::TAU).sin() * (-*age * 22.0).exp()
                            }
                            Cell::Snare => n * 0.6 * (-*age * 20.0).exp(),
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
                    if ch.dip > 0 {
                        ch.env *= DIP_FALL;
                        ch.dip -= 1;
                        if ch.dip == 0 {
                            let f = ch.pending;
                            ch.start(f, inst);
                        }
                    } else {
                        let target = if ch.gate { 1.0 } else { 0.0 };
                        match inst {
                            Inst::Bell | Inst::Arp => {
                                if ch.attack {
                                    ch.env += (1.0 - ch.env) * 0.08;
                                    if ch.env > 0.97 {
                                        ch.attack = false;
                                    }
                                } else {
                                    let decay = if inst == Inst::Bell { 2.2 } else { 7.0 };
                                    ch.env *= 1.0 - decay * dt;
                                    if !ch.gate {
                                        ch.env *= 1.0 - 12.0 * dt;
                                    }
                                }
                            }
                            Inst::Soft => ch.env += (target * 0.8 - ch.env) * dt * 12.0,
                            _ => {
                                let rate = if target > ch.env { 180.0 } else { 14.0 };
                                ch.env += (target - ch.env) * (rate * dt).min(1.0);
                            }
                        }
                    }
                    let vib = if inst == Inst::Lead && ch.age > 0.25 {
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

#[cfg(test)]
mod tests {
    use super::*;

    /// Where a grid step is, for reading a failure: section, bar and beat.
    fn place(score: &Score, tick: usize) -> String {
        let (start, name) = score
            .starts
            .iter()
            .rev()
            .find(|s| s.0 <= tick)
            .copied()
            .unwrap_or((0, '?'));
        let bar = (tick - start) / score.bar_ticks + 1;
        let beat = (tick - start) % score.bar_ticks / 4 + 1;
        format!("section {name} bar {bar} beat {beat}")
    }

    /// A note that rubs against a chord: a semitone above one of its notes (a fourth over
    /// a major third, a flat ninth over the root, a flat thirteenth over a minor chord's
    /// fifth), or a sharp eleventh over a major chord.
    fn clashes(c: &Chord, midi: f32) -> bool {
        let pc = (midi as i32).rem_euclid(12);
        let tones: Vec<i32> = c.pcs().collect();
        if tones.contains(&pc) {
            return false;
        }
        let major = tones.contains(&((c.root + 4) % 12));
        let sharp11 = (c.root + 6).rem_euclid(12);
        tones.iter().any(|t| (pc - t).rem_euclid(12) == 1)
            || (major && pc == sharp11 && !tones.contains(&sharp11))
    }

    #[test]
    fn every_song_is_written_to_fit_its_bars() {
        for &song in SONGS {
            if let Err(e) = compile(song.tune()) {
                panic!("{song:?}: {e}");
            }
        }
    }

    #[test]
    fn tunes_sit_on_their_chords() {
        // Every note on a beat, and every note held a beat or more, belongs with the chord
        // under it (and with the next chord, if it's held on into it).
        let mut bad = Vec::new();
        for &song in SONGS {
            let score = compile(song.tune()).unwrap();
            for part in [TUNE, COUNTER] {
                for (start, len, m) in notes(&score, part) {
                    let mut check = vec![start];
                    for k in start + 1..start + len {
                        if score.chords[k] != score.chords[k - 1] && start + len - k >= 4 {
                            check.push(k);
                        }
                    }
                    for (i, &k) in check.iter().enumerate() {
                        let strong = i > 0 || k % 4 == 0 || len >= 4;
                        if strong && clashes(&score.chords[k], m) {
                            bad.push(format!(
                                "{song:?} {} ({}): note {m} against {:?}",
                                if part == TUNE { "tune" } else { "counter" },
                                place(&score, k),
                                score.chords[k]
                            ));
                        }
                    }
                }
            }
        }
        assert!(bad.is_empty(), "{}", bad.join("\n"));
    }

    #[test]
    fn counter_tunes_never_rub_against_the_tune() {
        let mut bad = Vec::new();
        for &song in SONGS {
            let score = compile(song.tune()).unwrap();
            let sounding = |part: usize| {
                let mut at = vec![None; score.cells.len()];
                for (s, len, m) in notes(&score, part) {
                    for slot in at.iter_mut().skip(s).take(len) {
                        *slot = Some(m);
                    }
                }
                at
            };
            let (tune, counter) = (sounding(TUNE), sounding(COUNTER));
            for tick in (0..score.cells.len()).step_by(4) {
                if let (Some(a), Some(b)) = (tune[tick], counter[tick]) {
                    let gap = ((a - b).abs() as i32) % 12;
                    if gap == 1 || gap == 11 {
                        bad.push(format!("{song:?} ({}): {a} and {b}", place(&score, tick)));
                    }
                }
            }
        }
        assert!(bad.is_empty(), "{}", bad.join("\n"));
    }

    #[test]
    fn parts_stay_in_their_ranges() {
        for &song in SONGS {
            let score = compile(song.tune()).unwrap();
            for (part, lo, hi) in [
                (TUNE, 55.0, 91.0),
                (COUNTER, 48.0, 88.0),
                (CHORDS, 45.0, 80.0),
                (BASS, 30.0, 58.0),
            ] {
                for (s, _, m) in notes(&score, part) {
                    assert!(
                        (lo..=hi).contains(&m),
                        "{song:?} part {part} ({}): note {m} out of range",
                        place(&score, s)
                    );
                }
            }
        }
    }

    #[test]
    fn songs_play_a_good_while_before_repeating() {
        for &song in SONGS {
            let p = Player::new(song, 0, 1.0);
            let (_, round) = p.length();
            assert!(round >= 50.0, "{song:?} repeats after {round:.0}s");
        }
    }

    #[test]
    fn every_song_plays() {
        for &song in SONGS {
            let mut p = Player::new(song, 0, 1.0);
            let mut peak = 0.0f32;
            for _ in 0..(RATE as usize * 4) {
                let s = p.next();
                assert!(s.is_finite(), "{song:?}");
                peak = peak.max(s.abs());
            }
            assert!(peak > 0.05 && peak < 1.0, "{song:?} peaks at {peak}");
        }
    }

    #[test]
    fn repeated_notes_are_heard_again() {
        // The boss's bass hammers the same note in eighths: each one dips and comes back
        // (five times in its first two seconds).
        let mut p = Player::new(Song::Boss, 0, 1.0);
        let mut env = Vec::new();
        for _ in 0..(RATE as usize * 2) {
            p.next();
            env.push(p.chans[BASS].env);
        }
        let dips = env
            .windows(2)
            .filter(|w| w[0] >= 0.05 && w[1] < 0.05)
            .count();
        assert_eq!(dips, 5, "repeats heard in two seconds");
    }

    #[test]
    fn chords_read_as_written() {
        let c = parse_chord("Bbm6").unwrap();
        assert_eq!((c.root, c.n), (10, 4));
        assert_eq!(c.pcs().collect::<Vec<_>>(), vec![10, 1, 5, 7]);
        let c = parse_chord("D/A").unwrap();
        assert_eq!((c.root, c.bass), (2, 9));
        assert!(parse_chord("H7").is_none() && parse_chord("Cwhatever").is_none());
        assert_eq!(parse_note("C#5"), Some(73.0));
        assert_eq!(parse_note("Db5"), Some(73.0));
    }
}
