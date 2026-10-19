use std::fmt;

use serde::de::{self, Visitor};
use serde::{Deserialize, Deserializer};

/// Мелодия заставки (§13.1 п. 5, §13.2) из `data/sound/boot.toml`: одноголосый напев пищалки
/// PC-speaker — ноты по очереди, длина ноты в долях, доля длится `beat_ms` миллисекунд.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Melody {
    pub beat_ms: u32,
    pub notes: Vec<Note>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Note {
    pub pitch: Pitch,
    pub beats: u32,
}

/// Высота ноты — научная нотация «E5», «F#4», «Bb3» в октавах 1–8 — или пауза «rest». Частота —
/// в целых герцах, как её и выводит таймер PC-speaker.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pitch {
    Rest,
    Hertz(u32),
}

const REST: &str = "rest";
const CONCERT_A_HERTZ: f64 = 440.0;
/// Номер ля первой октавы в нумерации полутонов MIDI.
const CONCERT_A_NUMBER: i32 = 69;
const SEMITONES_IN_OCTAVE: i32 = 12;
const LOWEST_OCTAVE: i32 = 1;
const HIGHEST_OCTAVE: i32 = 8;

impl Melody {
    /// Ноты как тоны: частота или пауза и длительность в миллисекундах.
    pub fn tones(&self) -> Vec<(Pitch, u32)> {
        self.notes
            .iter()
            .map(|note| (note.pitch, note.beats * self.beat_ms))
            .collect()
    }
}

impl Pitch {
    /// «E5» → 659 Гц: полутон от ля первой октавы, равномерная темперация.
    fn parse(name: &str) -> Option<Self> {
        if name == REST {
            return Some(Self::Rest);
        }
        let mut characters = name.chars();
        let step = match characters.next()? {
            'C' => 0,
            'D' => 2,
            'E' => 4,
            'F' => 5,
            'G' => 7,
            'A' => 9,
            'B' => 11,
            _ => return None,
        };
        let rest: String = characters.collect();
        let (shift, octave) = match rest.chars().next()? {
            '#' => (1, &rest[1..]),
            'b' => (-1, &rest[1..]),
            _ => (0, rest.as_str()),
        };
        let octave: i32 = octave.parse().ok()?;
        if !(LOWEST_OCTAVE..=HIGHEST_OCTAVE).contains(&octave) {
            return None;
        }
        let number = SEMITONES_IN_OCTAVE * (octave + 1) + step + shift;
        let semitones = f64::from(number - CONCERT_A_NUMBER);
        let hertz = CONCERT_A_HERTZ * 2_f64.powf(semitones / f64::from(SEMITONES_IN_OCTAVE));
        Some(Self::Hertz(hertz.round() as u32))
    }
}

impl<'de> Deserialize<'de> for Pitch {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct PitchVisitor;

        impl Visitor<'_> for PitchVisitor {
            type Value = Pitch;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(
                    formatter,
                    "нота научной нотацией в октавах {LOWEST_OCTAVE}–{HIGHEST_OCTAVE} («E5», «F#4», «Bb3») или «{REST}»"
                )
            }

            fn visit_str<E: de::Error>(self, name: &str) -> Result<Pitch, E> {
                Pitch::parse(name).ok_or_else(|| E::invalid_value(de::Unexpected::Str(name), &self))
            }
        }

        deserializer.deserialize_str(PitchVisitor)
    }
}

#[cfg(test)]
mod tests {
    use super::Pitch;

    #[test]
    fn scientific_names_become_equal_tempered_hertz() {
        assert_eq!(Pitch::parse("A4"), Some(Pitch::Hertz(440)));
        assert_eq!(Pitch::parse("E5"), Some(Pitch::Hertz(659)));
        assert_eq!(Pitch::parse("C4"), Some(Pitch::Hertz(262)));
        assert_eq!(Pitch::parse("F#4"), Some(Pitch::Hertz(370)));
        assert_eq!(Pitch::parse("Bb3"), Some(Pitch::Hertz(233)));
        assert_eq!(Pitch::parse("rest"), Some(Pitch::Rest));
    }

    #[test]
    fn unknown_names_and_octaves_outside_the_speaker_are_refused() {
        for name in ["H4", "E", "E9", "E0", "e5", "E#", "пауза", ""] {
            assert_eq!(Pitch::parse(name), None, "{name}");
        }
    }
}
