//! The game's songs, written out for the little chiptune band in `music.rs`.
//!
//! Each song is a handful of sections played in the order of its form. A section is some
//! bars of chords ("|" between bars, two chords in a bar split by a space), a tune and a
//! counter-tune in eighth notes ("-" holds the note before, "." rests). The chords are also
//! broken into the song's arpeggio and walked by its bass line, so everything the band plays
//! comes from the same harmony.

use super::music::{Inst, Section, Tune};

const NONE: &str = "";

/// The title screen (and the end of each day): the Hollowbloom theme.
pub const TITLE: Tune = Tune {
    bpm: 88.0,
    bar: 8,
    form: "IABC",
    repeat: 1,
    band: [
        (Inst::Bell, 0.2),
        (Inst::Soft, 0.09),
        (Inst::Arp, 0.09),
        (Inst::Bass, 0.22),
        (Inst::Drums, 0.0),
    ],
    arp: "0 1 2 3 2 1 2 1",
    bass: "R - - - 5 - - -",
    sections: &[
        Section {
            name: 'I',
            chords: "C | Am | F | G",
            tune: NONE,
            counter: NONE,
            drums: NONE,
        },
        Section {
            name: 'A',
            chords: "C | Am | F | C G | C | Am | Dm7 G7 | C",
            tune: "E5 - G5 - C6 - - B5 | A5 - - - G5 - - - | F5 - A5 - G5 - F5 - | E5 - - - D5 - - - | \
                   E5 - G5 - C6 - - B5 | A5 - - - G5 - E5 - | F5 - E5 - D5 - G5 - | C5 - - - - - . .",
            counter: NONE,
            drums: NONE,
        },
        Section {
            name: 'B',
            chords: "F | G | Em | Am | Dm | G | C Am | G7",
            tune: "A4 - C5 - F5 - - E5 | D5 - - - B4 - G4 - | G4 - B4 - E5 - - D5 | C5 - - - A4 - - - | \
                   F5 - A5 - D6 - - C6 | B5 - - - G5 - D5 - | E5 - G5 - C6 - A5 - | G5 - - - F5 - D5 -",
            counter: NONE,
            drums: NONE,
        },
        Section {
            name: 'C',
            chords: "C | Am | F | C G | C | Am | Dm7 G7 | C",
            tune: "E5 - G5 - C6 - - B5 | A5 - - - G5 - - - | F5 - A5 - G5 - F5 - | E5 - - - D5 - - - | \
                   E5 - G5 - C6 - - B5 | A5 - - - G5 - E5 - | F5 - E5 - D5 - G5 - | C5 - - - - - . .",
            counter: "E4 - - - G4 - - - | C5 - - - B4 - - - | A4 - - - C5 - - - | G4 - - - B4 - - - | \
                      C5 - - - E5 - - - | E5 - - - C5 - - - | A4 - - - B4 - - - | C5 - - - - - . .",
            drums: NONE,
        },
    ],
};

/// The farm in the morning.
pub const MORNING: Tune = Tune {
    bpm: 104.0,
    bar: 8,
    form: "IACBC",
    repeat: 1,
    band: [
        (Inst::Lead, 0.18),
        (Inst::Soft, 0.09),
        (Inst::Arp, 0.09),
        (Inst::Bass, 0.22),
        (Inst::Drums, 0.14),
    ],
    arp: "0 2 1 2 0 2 1 2",
    bass: "R - . 5 R - 5 -",
    sections: &[
        Section {
            name: 'I',
            chords: "G | D7",
            tune: NONE,
            counter: NONE,
            drums: "h . h . h . h .",
        },
        Section {
            name: 'A',
            chords: "G | C | G | D | G | C | D7 | G",
            tune: "B4 - D5 - G5 - F#5 G5 | E5 - - - C5 - - - | D5 - B4 - G4 - A4 B4 | A4 - - - - - . . | \
                   B4 - D5 - G5 - F#5 G5 | E5 - - - G5 - E5 - | C5 - B4 - A4 - F#4 - | G4 - - - - - . .",
            counter: NONE,
            drums: "k . h . s . h h",
        },
        Section {
            name: 'C',
            chords: "G | C | G | D | G | C | D7 | G",
            tune: "B4 - D5 - G5 - F#5 G5 | E5 - - - C5 - - - | D5 - B4 - G4 - A4 B4 | A4 - - - - - . . | \
                   B4 - D5 - G5 - F#5 G5 | E5 - - - G5 - E5 - | C5 - B4 - A4 - F#4 - | G4 - - - - - . .",
            counter: "D4 - - - B3 - - - | E4 - - - G4 - - - | B3 - - - D4 - - - | F#4 - - - A4 - - - | \
                      G4 - - - B4 - - - | C5 - - - E5 - - - | A4 - - - D4 - - - | B3 - - - - - . .",
            drums: "k . h . s . h h",
        },
        Section {
            name: 'B',
            chords: "Em | C | G | D | Em | C | Am7 | D7",
            tune: "G5 - - - F#5 - E5 - | E5 - - - D5 - C5 - | D5 - - - B4 - G4 - | A4 - - - - - . . | \
                   B4 - E5 - G5 - B5 - | A5 - - - G5 - E5 - | C6 - B5 - A5 - G5 - | F#5 - - - D5 - - -",
            counter: NONE,
            drums: "k . h . . . h .",
        },
    ],
};

/// The farm in the afternoon: lazier, with a little syncopation.
pub const AFTERNOON: Tune = Tune {
    bpm: 96.0,
    bar: 8,
    form: "IABC",
    repeat: 1,
    band: [
        (Inst::Lead, 0.17),
        (Inst::Bell, 0.12),
        (Inst::Soft, 0.09),
        (Inst::Bass, 0.22),
        (Inst::Drums, 0.12),
    ],
    arp: "0 - 1 - 2 - 1 -",
    bass: "R - - - 5 - - -",
    sections: &[
        Section {
            name: 'I',
            chords: "F | C7",
            tune: NONE,
            counter: NONE,
            drums: "k . . h s . . h",
        },
        Section {
            name: 'A',
            chords: "F | Bb | Gm7 | C7 | F | Dm | Gm7 C7 | F",
            tune: "C5 - F5 - A5 - G5 F5 | D5 - - - F5 - - - | Bb4 - D5 - G5 - F5 D5 | E5 - - - C5 - - - | \
                   C5 - F5 - A5 - G5 F5 | A5 - - - D6 - - - | C6 - Bb5 - A5 - G5 - | F5 - - - - - . .",
            counter: NONE,
            drums: "k . . h s . . h",
        },
        Section {
            name: 'B',
            chords: "Bb | C | Am | Dm | Gm7 | C7 | F | C7",
            tune: "F5 - - D5 F5 - - D5 | E5 - - - - - . . | E5 - - C5 E5 - - C5 | D5 - - - - - . . | \
                   Bb5 - - G5 Bb5 - - G5 | A5 - - - G5 - E5 - | F5 - A5 - C6 - A5 - | G5 - - - E5 - - -",
            counter: NONE,
            drums: "k . h h s . h .",
        },
        Section {
            name: 'C',
            chords: "F | Bb | Gm7 | C7 | F | Dm | Gm7 C7 | F",
            tune: "C5 - F5 - A5 - G5 F5 | D5 - - - F5 - - - | Bb4 - D5 - G5 - F5 D5 | E5 - - - C5 - - - | \
                   C5 - F5 - A5 - G5 F5 | A5 - - - D6 - - - | C6 - Bb5 - A5 - G5 - | F5 - - - - - . .",
            counter: "A4 - - - C5 - - - | F4 - - - D4 - - - | D4 - - - Bb4 - - - | G4 - - - Bb4 - - - | \
                      A4 - - - C5 - - - | F4 - - - A4 - - - | Bb4 - - - E4 - - - | F4 - - - - - . .",
            drums: "k . . h s . . h",
        },
    ],
};

/// Night on the farm, in town and at home: a lullaby.
pub const NIGHT: Tune = Tune {
    bpm: 72.0,
    bar: 8,
    form: "ACBC",
    repeat: 0,
    band: [
        (Inst::Bell, 0.2),
        (Inst::Soft, 0.08),
        (Inst::Soft, 0.09),
        (Inst::Bass, 0.2),
        (Inst::Drums, 0.0),
    ],
    arp: "0 1 2 1 0 1 2 1",
    bass: "R - - - - - - -",
    sections: &[
        Section {
            name: 'A',
            chords: "Am | F | C | G | Am | F | Dm | E7",
            tune: "E5 - - - C5 - D5 E5 | F5 - - - A5 - - - | G5 - - - E5 - C5 - | D5 - - - - - . . | \
                   E5 - - - C5 - D5 E5 | F5 - - - A5 - C6 - | A5 - - - F5 - D5 - | E5 - - - D5 - B4 -",
            counter: NONE,
            drums: NONE,
        },
        Section {
            name: 'C',
            chords: "Am | F | C | G | Am | F | Dm E7 | Am",
            tune: "E5 - - - C5 - D5 E5 | F5 - - - A5 - - - | G5 - - - E5 - C5 - | D5 - - - - - . . | \
                   E5 - - - C5 - D5 E5 | F5 - - - A5 - C6 - | A5 - - - G#5 - - - | A5 - - - - - . .",
            counter: "C5 - - - - - - - | C5 - - - - - - - | C5 - - - G4 - - - | B4 - - - - - - - | \
                      C5 - - - - - - - | A4 - - - - - - - | F4 - - - E4 - - - | E4 - - - - - . .",
            drums: NONE,
        },
        Section {
            name: 'B',
            chords: "F | G | C | Am | F | G | Esus4 E7 | E7",
            tune: "C6 - - - A5 - - - | B5 - - - G5 - - - | E5 - G5 - C6 - - - | A5 - - - - - . . | \
                   A5 - - - F5 - A5 - | G5 - - - D5 - B4 - | A4 - - - G#4 - - - | B4 - - - D5 - - -",
            counter: NONE,
            drums: NONE,
        },
    ],
};

/// Bramblewick by day: a village waltz.
pub const TOWN: Tune = Tune {
    bpm: 138.0,
    bar: 6,
    form: "IABC",
    repeat: 1,
    band: [
        (Inst::Lead, 0.17),
        (Inst::Bell, 0.1),
        (Inst::Arp, 0.09),
        (Inst::Bass, 0.22),
        (Inst::Drums, 0.12),
    ],
    arp: ". . 1 . 2 .",
    bass: "R - - - - -",
    sections: &[
        Section {
            name: 'I',
            chords: "D | D | A7 | A7",
            tune: NONE,
            counter: NONE,
            drums: "k . h . h .",
        },
        Section {
            name: 'A',
            chords: "D | D | G | G | A7 | A7 | D | A7 | D | D | G | Gm | D/A | A7 | A7 | D",
            tune: "A4 - D5 - F#5 - | A5 - - - F#5 - | G5 - - - B5 - | D6 - - - - - | \
                   C#6 - B5 - A5 - | G5 - - - E5 - | F#5 - E5 - D5 - | E5 - - - - - | \
                   A4 - D5 - F#5 - | A5 - - - F#5 - | G5 - - - B5 - | D6 - - - Bb5 - | \
                   A5 - - - F#5 - | G5 - - - E5 - | E5 - - - C#5 - | D5 - - - - -",
            counter: NONE,
            drums: "k . h . h .",
        },
        Section {
            name: 'B',
            chords: "G | G | C | C | D7 | D7 | G | G | Em | Em | A7 | A7 | D | Bm | Em7 | A7",
            tune: "B4 - D5 - G5 - | B5 - - - A5 - | G5 - - - E5 - | C5 - - - - - | \
                   D5 - F#5 - A5 - | C6 - - - B5 - | B5 - A5 - G5 - | D5 - - - - - | \
                   B4 - E5 - G5 - | B5 - - - A5 - | G5 - - - E5 - | C#5 - - - - - | \
                   D5 - F#5 - A5 - | B5 - - - F#5 - | G5 - - - E5 - | C#5 - E5 - A5 -",
            counter: NONE,
            drums: "k . h . h .",
        },
        Section {
            name: 'C',
            chords: "D | D | G | G | A7 | A7 | D | A7 | D | D | G | Gm | D/A | A7 | A7 | D",
            tune: "A4 - D5 - F#5 - | A5 - - - F#5 - | G5 - - - B5 - | D6 - - - - - | \
                   C#6 - B5 - A5 - | G5 - - - E5 - | F#5 - E5 - D5 - | E5 - - - - - | \
                   A4 - D5 - F#5 - | A5 - - - F#5 - | G5 - - - B5 - | D6 - - - Bb5 - | \
                   A5 - - - F#5 - | G5 - - - E5 - | E5 - - - C#5 - | D5 - - - - -",
            counter: "D5 - - - - - | C#5 - - - - - | B4 - - - - - | B4 - - - - - | \
                      A4 - - - - - | G4 - - - - - | A4 - - - - - | A4 - - - - - | \
                      D5 - - - - - | C#5 - - - - - | B4 - - - - - | Bb4 - - - - - | \
                      A4 - - - - - | G4 - - - - - | G4 - - - - - | F#4 - - - - -",
            drums: "k . h . h .",
        },
    ],
};

/// Inside the shops.
pub const SHOP: Tune = Tune {
    bpm: 92.0,
    bar: 8,
    form: "ACBC",
    repeat: 0,
    band: [
        (Inst::Bell, 0.18),
        (Inst::Soft, 0.08),
        (Inst::Soft, 0.08),
        (Inst::Bass, 0.2),
        (Inst::Drums, 0.1),
    ],
    arp: "0 - 2 - 1 - 3 -",
    bass: "R - 3 - 5 - a -",
    sections: &[
        Section {
            name: 'A',
            chords: "Fmaj7 | Dm7 | Gm7 | C7 | Am7 | Dm7 | Gm7 | C7",
            tune: "A4 - C5 - E5 - - D5 | C5 - - - A4 - - - | Bb4 - D5 - F5 - - E5 | E5 - - - G5 - - - | \
                   C6 - - - A5 - - - | F5 - - - D5 - - - | Bb5 - A5 - G5 - F5 - | E5 - - - - - . .",
            counter: NONE,
            drums: "k . . h s . . h",
        },
        Section {
            name: 'C',
            chords: "Fmaj7 | Dm7 | Gm7 | C7 | Am7 | Dm7 | Gm7 C7 | F6",
            tune: "A4 - C5 - E5 - - D5 | C5 - - - A4 - - - | Bb4 - D5 - F5 - - E5 | E5 - - - G5 - - - | \
                   C6 - - - A5 - - - | F5 - - - D5 - - - | Bb5 - A5 - G5 - E5 - | F5 - - - - - . .",
            counter: "F4 - - - E4 - - - | F4 - - - - - - - | F4 - - - - - - - | E4 - - - - - - - | \
                      E4 - - - - - - - | F4 - - - - - - - | F4 - - - E4 - - - | F4 - - - - - . .",
            drums: "k . . h s . . h",
        },
        Section {
            name: 'B',
            chords: "Bbmaj7 | Bbm6 | Am7 | D7 | Gm7 | C7 | Fmaj7 | Gm7 C7",
            tune: "D5 - F5 - A5 - - F5 | Db5 - - - Bb4 - - - | C5 - E5 - G5 - - E5 | F#5 - - - D5 - - - | \
                   G5 - Bb5 - D6 - - C6 | Bb5 - - - G5 - - - | A5 - - - F5 - - - | G5 - - - E5 - - -",
            counter: NONE,
            drums: "k . h h s . h h",
        },
    ],
};

/// The farmhouse by day, and the Hollow's waystone floors: somewhere safe.
pub const HAVEN: Tune = Tune {
    bpm: 76.0,
    bar: 8,
    form: "IABC",
    repeat: 1,
    band: [
        (Inst::Bell, 0.19),
        (Inst::Soft, 0.08),
        (Inst::Soft, 0.09),
        (Inst::Bass, 0.2),
        (Inst::Drums, 0.0),
    ],
    arp: "0 1 2 1 0 1 2 1",
    bass: "R - - - 5 - - -",
    sections: &[
        Section {
            name: 'I',
            chords: "Eb | Bb7",
            tune: NONE,
            counter: NONE,
            drums: NONE,
        },
        Section {
            name: 'A',
            chords: "Eb | Cm7 | Ab | Bb7 | Eb | Gm7 | Ab Bb7 | Eb",
            tune: "G5 - - - Bb5 - - - | G5 - - - Eb5 - - - | C5 - Eb5 - Ab5 - G5 - | F5 - - - D5 - - - | \
                   G5 - - - Bb5 - Eb6 - | D6 - - - Bb5 - - - | C6 - Ab5 - Bb5 - F5 - | Eb5 - - - - - . .",
            counter: NONE,
            drums: NONE,
        },
        Section {
            name: 'B',
            chords: "Ab | Bb | Gm7 | Cm7 | Fm7 | Bb7 | Eb Ab | Fm7 Bb7",
            tune: "Eb5 - C5 - Eb5 - Ab5 - | F5 - - - D5 - - - | D5 - Bb4 - D5 - G5 - | Eb5 - - - - - . . | \
                   Ab5 - - - C6 - - - | Bb5 - - - Ab5 - F5 - | G5 - - - C6 - - - | Bb5 - Ab5 - F5 - D5 -",
            counter: NONE,
            drums: NONE,
        },
        Section {
            name: 'C',
            chords: "Eb | Cm7 | Ab | Bb7 | Eb | Gm7 | Ab Bb7 | Eb",
            tune: "G5 - - - Bb5 - - - | G5 - - - Eb5 - - - | C5 - Eb5 - Ab5 - G5 - | F5 - - - D5 - - - | \
                   G5 - - - Bb5 - Eb6 - | D6 - - - Bb5 - - - | C6 - Ab5 - Bb5 - F5 - | Eb5 - - - - - . .",
            counter: "Eb5 - - - D5 - - - | C5 - - - Bb4 - - - | Ab4 - - - C5 - - - | D5 - - - Bb4 - - - | \
                      Bb4 - - - G4 - - - | Bb4 - - - D5 - - - | Eb5 - - - D5 - - - | Bb4 - - - - - . .",
            drums: NONE,
        },
    ],
};

/// The Hollow's earthy floors (the Mossy Burrows and Fungal Hollow): off on an adventure.
pub const BURROWS: Tune = Tune {
    bpm: 100.0,
    bar: 8,
    form: "IABAC",
    repeat: 1,
    band: [
        (Inst::Lead, 0.17),
        (Inst::Bell, 0.1),
        (Inst::Arp, 0.1),
        (Inst::Bass, 0.23),
        (Inst::Drums, 0.13),
    ],
    arp: "0 2 1 2 0 2 1 2",
    bass: "R - R - R - 5 -",
    sections: &[
        Section {
            name: 'I',
            chords: "Dm | A7",
            tune: NONE,
            counter: NONE,
            drums: "k . . h k . s .",
        },
        Section {
            name: 'A',
            chords: "Dm | Bb | C | A7 | Dm | Bb | Gm | A7",
            tune: "D5 - - - A4 - D5 E5 | F5 - - - D5 - Bb4 - | C5 - - - G4 - C5 D5 | E5 - - - C#5 - A4 - | \
                   D5 - - - A4 - D5 E5 | F5 - - - G5 - A5 - | Bb5 - A5 - G5 - F5 - | E5 - - - - - . .",
            counter: NONE,
            drums: "k . . h k . s .",
        },
        Section {
            name: 'B',
            chords: "F | C | Dm | A | Bb | C | Gm A7 | Dm",
            tune: "A5 - - - C6 - A5 - | G5 - - - E5 - C5 - | F5 - - - A5 - F5 - | E5 - - - - - . . | \
                   D5 - F5 - Bb5 - A5 - | G5 - - - E5 - G5 - | Bb5 - G5 - A5 - C#5 - | D5 - - - - - . .",
            counter: NONE,
            drums: "k . h h k . s h",
        },
        Section {
            name: 'C',
            chords: "Dm | Dm | Bb | A7 | Dm | Dm | Bb | A7",
            tune: NONE,
            counter: "D6 - - - A5 - - - | F5 - - - A5 - - - | D6 - - - F5 - - - | C#6 - - - A5 - - - | \
                      D6 - - - A5 - - - | F5 - - - A5 - - - | F5 - - - D5 - - - | E5 - - - C#5 - - -",
            drums: "k . . h k . s .",
        },
    ],
};

/// The Hollow's glittering floors (the Crystal Grotto and Frost Caverns): slow and spacious.
pub const GLIMMER: Tune = Tune {
    bpm: 84.0,
    bar: 8,
    form: "IABC",
    repeat: 1,
    band: [
        (Inst::Bell, 0.2),
        (Inst::Soft, 0.08),
        (Inst::Soft, 0.1),
        (Inst::Bass, 0.2),
        (Inst::Drums, 0.06),
    ],
    arp: "0 1 2 3 2 1 2 1",
    bass: "R - - - - - 5 -",
    sections: &[
        Section {
            name: 'I',
            chords: "Em | Cmaj7",
            tune: NONE,
            counter: NONE,
            drums: ". . h . . . h .",
        },
        Section {
            name: 'A',
            chords: "Em | Cmaj7 | G | D | Em | Cmaj7 | Am | B7",
            tune: "B5 - - - G5 - - - | E5 - - - - - . . | D5 - - - G5 - - - | F#5 - - - - - . . | \
                   B5 - - - E6 - - - | D6 - - - B5 - G5 - | C6 - - - A5 - E5 - | D#5 - - - F#5 - - -",
            counter: NONE,
            drums: ". . h . . . h .",
        },
        Section {
            name: 'B',
            chords: "Cmaj7 | D | Bm7 | Em | Cmaj7 | D | Em | Em",
            tune: "G5 - E5 - G5 - B5 - | A5 - - - F#5 - - - | F#5 - D5 - F#5 - A5 - | G5 - - - - - . . | \
                   E5 - G5 - C6 - B5 - | A5 - - - D6 - - - | B5 - - - G5 - - - | E5 - - - - - . .",
            counter: NONE,
            drums: ". . h . . . h .",
        },
        Section {
            name: 'C',
            chords: "Em | Cmaj7 | G | D | Em | Cmaj7 | Am | B7",
            tune: "B5 - - - G5 - - - | E5 - - - - - . . | D5 - - - G5 - - - | F#5 - - - - - . . | \
                   B5 - - - E6 - - - | D6 - - - B5 - G5 - | C6 - - - A5 - E5 - | D#5 - - - F#5 - - -",
            counter: "E4 - - - - - - - | E4 - - - - - - - | D4 - - - - - - - | D4 - - - - - - - | \
                      E4 - - - G4 - - - | G4 - - - E4 - - - | E4 - - - C4 - - - | B3 - - - D#4 - - -",
            drums: ". . h . . . h .",
        },
    ],
};

/// The Hollow's hot and ancient floors (the Ember Depths and Sunken Ruins): driving.
pub const DEPTHS: Tune = Tune {
    bpm: 112.0,
    bar: 8,
    form: "IABAC",
    repeat: 1,
    band: [
        (Inst::Lead, 0.17),
        (Inst::Bell, 0.1),
        (Inst::Arp, 0.1),
        (Inst::Bass, 0.24),
        (Inst::Drums, 0.15),
    ],
    arp: "0 2 1 2 0 2 1 2",
    bass: "R R 8 R R R 8 R",
    sections: &[
        Section {
            name: 'I',
            chords: "Am | E",
            tune: NONE,
            counter: NONE,
            drums: "k h s h k k s h",
        },
        Section {
            name: 'A',
            chords: "Am | G | F | E | Am | G | Dm E | Am",
            tune: "E5 - E5 - A5 - G5 E5 | D5 - - - B4 - - - | C5 - C5 - F5 - E5 C5 | B4 - - - G#4 - - - | \
                   E5 - E5 - A5 - G5 E5 | D5 - - - G5 - B5 - | D5 - F5 - E5 - G#5 - | A5 - - - - - . .",
            counter: NONE,
            drums: "k h s h k k s h",
        },
        Section {
            name: 'B',
            chords: "F | G | Em | Am | F | G | E | E7",
            tune: "A5 - - - C6 - A5 - | B5 - - - G5 - D5 - | E5 - - - G5 - B5 - | C6 - - - A5 - - - | \
                   F5 - A5 - C6 - A5 - | D6 - - - B5 - G5 - | G#5 - - - B5 - - - | D6 - - - B5 - G#5 -",
            counter: NONE,
            drums: "k h s h k k s h",
        },
        Section {
            name: 'C',
            chords: "Am | Am | F | E | Am | Am | F | E",
            tune: NONE,
            counter: "A5 - - - E5 - - - | C6 - - - A5 - - - | A5 - - - F5 - - - | G#5 - - - B5 - - - | \
                      A5 - - - E5 - - - | C6 - - - A5 - - - | C6 - - - A5 - - - | B5 - - - G#5 - - -",
            drums: "k . h . k . h .",
        },
    ],
};

/// A guardian awakes.
pub const BOSS: Tune = Tune {
    bpm: 144.0,
    bar: 8,
    form: "ABAC",
    repeat: 0,
    band: [
        (Inst::Lead, 0.18),
        (Inst::Bell, 0.0),
        (Inst::Arp, 0.09),
        (Inst::Bass, 0.24),
        (Inst::Drums, 0.18),
    ],
    arp: "0 1 2 1 0 1 2 1",
    bass: "R R 8 R R R 8 R",
    sections: &[
        Section {
            name: 'A',
            chords: "Dm | Dm | Bb | C | Dm | Dm | Gm | A",
            tune: "D5 - - A4 D5 - F5 - | A5 - - - F5 - - - | D5 - - Bb4 D5 - F5 - | G5 - - - E5 - - - | \
                   D5 - - A4 D5 - F5 - | A5 - - - D6 - - - | Bb5 - A5 - G5 - D5 - | E5 - - - C#5 - A4 -",
            counter: NONE,
            drums: "k h s h k k s h",
        },
        Section {
            name: 'B',
            chords: "Bb | C | Dm | Dm | Bb | C | A | A7",
            tune: "F5 - F5 - G5 - F5 D5 | E5 - - - C5 - - - | F5 - F5 - A5 - F5 D5 | D5 - - - - - . . | \
                   D6 - D6 - C6 - Bb5 - | C6 - - - G5 - E5 - | E5 - E5 - C#5 - E5 - | G5 - - - E5 - C#5 -",
            counter: NONE,
            drums: "k h s h k k s h",
        },
        Section {
            name: 'C',
            chords: "Dm | Bb | Gm | A | Dm | Bb | Gm | A7",
            tune: NONE,
            counter: NONE,
            drums: "k k s h k k s s",
        },
    ],
};
