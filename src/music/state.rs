use rust_music_theory::chord::{Chord, Number as ChordNumber, Quality as ChordQuality};
use rust_music_theory::note::{Notes, Pitch, PitchSymbol};
use rust_music_theory::scale::{Direction, Mode, Scale, ScaleType};

const ROOT_PITCHES: &[PitchSymbol] = &[
    PitchSymbol::C,
    PitchSymbol::Cs,
    PitchSymbol::D,
    PitchSymbol::Ds,
    PitchSymbol::E,
    PitchSymbol::F,
    PitchSymbol::Fs,
    PitchSymbol::G,
    PitchSymbol::Gs,
    PitchSymbol::A,
    PitchSymbol::As,
    PitchSymbol::B,
];

const GENRES: &[&str] = &["Blues", "Rock", "Jazz", "Metal", "Funk"];

const CHORD_QUALITIES: &[(ChordQuality, &str)] = &[
    (ChordQuality::Major, "Major"),
    (ChordQuality::Minor, "Minor"),
];

pub struct MusicState {
    genre_index: usize,
    root_pitch_index: usize,
    chord_quality_index: usize,
}

impl MusicState {
    pub fn new() -> Self {
        Self {
            genre_index: GENRES
                .iter()
                .position(|genre| *genre == "Blues")
                .unwrap_or(0),
            root_pitch_index: ROOT_PITCHES
                .iter()
                .position(|pitch| *pitch == PitchSymbol::E)
                .unwrap_or(0),
            chord_quality_index: CHORD_QUALITIES
                .iter()
                .position(|(quality, _)| *quality == ChordQuality::Major)
                .unwrap_or(0),
        }
    }

    pub fn genre(&self) -> &str {
        GENRES[self.genre_index]
    }

    pub fn root_pitch_label(&self) -> String {
        format!("{}", ROOT_PITCHES[self.root_pitch_index])
    }

    pub fn chord_quality_label(&self) -> &str {
        CHORD_QUALITIES[self.chord_quality_index].1
    }

    pub fn chord_notes_label(&self) -> String {
        let chord = self.current_chord();
        chord
            .notes()
            .iter()
            .map(|note| note.to_string())
            .collect::<Vec<_>>()
            .join(" ")
    }

    pub fn suggested_scales(&self) -> Vec<String> {
        let root_pitch = self.current_pitch();
        self.scale_suggestions_for_genre()
            .into_iter()
            .filter_map(|suggestion| {
                let scale = Scale::new(
                    suggestion.scale_type,
                    root_pitch,
                    4,
                    suggestion.mode,
                    Direction::Ascending,
                )
                .ok()?;

                let notes = scale
                    .notes()
                    .iter()
                    .map(|note| note.to_string())
                    .collect::<Vec<_>>()
                    .join(" ");

                Some(format!("{}: {}", suggestion.label, notes))
            })
            .collect()
    }

    pub fn root_pitch_class(&self) -> u8 {
        self.current_pitch().into_u8()
    }

    pub fn first_suggested_scale_label(&self) -> String {
        self.scale_suggestions_for_genre()
            .first()
            .map(|suggestion| suggestion.label.to_string())
            .unwrap_or_else(|| "(No scale)".to_string())
    }

    pub fn first_suggested_scale_pitch_classes(&self) -> Vec<u8> {
        let root_pitch = self.current_pitch();
        let suggestions = self.scale_suggestions_for_genre();
        let Some(suggestion) = suggestions.first() else {
            return Vec::new();
        };

        let scale = match Scale::new(
            suggestion.scale_type,
            root_pitch,
            4,
            suggestion.mode,
            Direction::Ascending,
        ) {
            Ok(scale) => scale,
            Err(_) => return Vec::new(),
        };

        scale
            .notes()
            .iter()
            .map(|note| note.pitch.into_u8())
            .collect()
    }

    pub fn next_root_pitch(&mut self) {
        self.root_pitch_index = (self.root_pitch_index + 1) % ROOT_PITCHES.len();
    }

    pub fn prev_root_pitch(&mut self) {
        if self.root_pitch_index == 0 {
            self.root_pitch_index = ROOT_PITCHES.len() - 1;
        } else {
            self.root_pitch_index -= 1;
        }
    }

    pub fn next_chord_quality(&mut self) {
        self.chord_quality_index = (self.chord_quality_index + 1) % CHORD_QUALITIES.len();
    }

    pub fn prev_chord_quality(&mut self) {
        if self.chord_quality_index == 0 {
            self.chord_quality_index = CHORD_QUALITIES.len() - 1;
        } else {
            self.chord_quality_index -= 1;
        }
    }

    pub fn next_genre(&mut self) {
        self.genre_index = (self.genre_index + 1) % GENRES.len();
    }

    pub fn set_context(&mut self, root: Option<&str>, quality: Option<&str>, genre: Option<&str>) {
        if let Some(root) = root.and_then(find_root_pitch_index) {
            self.root_pitch_index = root;
        }

        if let Some(quality) = quality.and_then(find_chord_quality_index) {
            self.chord_quality_index = quality;
        }

        if let Some(genre) = genre.and_then(find_genre_index) {
            self.genre_index = genre;
        }
    }

    pub fn prev_genre(&mut self) {
        if self.genre_index == 0 {
            self.genre_index = GENRES.len() - 1;
        } else {
            self.genre_index -= 1;
        }
    }

    fn current_pitch(&self) -> Pitch {
        Pitch::from(ROOT_PITCHES[self.root_pitch_index])
    }

    fn current_chord(&self) -> Chord {
        let quality = CHORD_QUALITIES[self.chord_quality_index].0;
        Chord::new(self.current_pitch(), quality, ChordNumber::Triad)
    }

    fn scale_suggestions_for_genre(&self) -> Vec<ScaleSuggestion> {
        let quality = CHORD_QUALITIES[self.chord_quality_index].0;
        let is_minor = matches!(quality, ChordQuality::Minor);

        match self.genre() {
            "Blues" => {
                if is_minor {
                    vec![
                        ScaleSuggestion::new("Blues", ScaleType::Blues, Some(Mode::Blues)),
                        ScaleSuggestion::new(
                            "Minor Pentatonic",
                            ScaleType::PentatonicMinor,
                            Some(Mode::PentatonicMinor),
                        ),
                        ScaleSuggestion::new("Dorian", ScaleType::Diatonic, Some(Mode::Dorian)),
                    ]
                } else {
                    vec![
                        ScaleSuggestion::new("Blues", ScaleType::Blues, Some(Mode::Blues)),
                        ScaleSuggestion::new(
                            "Major Pentatonic",
                            ScaleType::PentatonicMajor,
                            Some(Mode::PentatonicMajor),
                        ),
                        ScaleSuggestion::new("Mixolydian", ScaleType::Diatonic, Some(Mode::Mixolydian)),
                    ]
                }
            }
            "Rock" => {
                if is_minor {
                    vec![
                        ScaleSuggestion::new(
                            "Minor Pentatonic",
                            ScaleType::PentatonicMinor,
                            Some(Mode::PentatonicMinor),
                        ),
                        ScaleSuggestion::new("Aeolian", ScaleType::Diatonic, Some(Mode::Aeolian)),
                        ScaleSuggestion::new("Dorian", ScaleType::Diatonic, Some(Mode::Dorian)),
                    ]
                } else {
                    vec![
                        ScaleSuggestion::new(
                            "Major Pentatonic",
                            ScaleType::PentatonicMajor,
                            Some(Mode::PentatonicMajor),
                        ),
                        ScaleSuggestion::new("Ionian", ScaleType::Diatonic, Some(Mode::Ionian)),
                        ScaleSuggestion::new("Mixolydian", ScaleType::Diatonic, Some(Mode::Mixolydian)),
                    ]
                }
            }
            "Jazz" => {
                if is_minor {
                    vec![
                        ScaleSuggestion::new("Dorian", ScaleType::Diatonic, Some(Mode::Dorian)),
                        ScaleSuggestion::new(
                            "Melodic Minor",
                            ScaleType::MelodicMinor,
                            Some(Mode::MelodicMinor),
                        ),
                        ScaleSuggestion::new(
                            "Harmonic Minor",
                            ScaleType::HarmonicMinor,
                            Some(Mode::HarmonicMinor),
                        ),
                    ]
                } else {
                    vec![
                        ScaleSuggestion::new("Ionian", ScaleType::Diatonic, Some(Mode::Ionian)),
                        ScaleSuggestion::new("Lydian", ScaleType::Diatonic, Some(Mode::Lydian)),
                        ScaleSuggestion::new("Mixolydian", ScaleType::Diatonic, Some(Mode::Mixolydian)),
                    ]
                }
            }
            "Metal" => {
                if is_minor {
                    vec![
                        ScaleSuggestion::new("Phrygian", ScaleType::Diatonic, Some(Mode::Phrygian)),
                        ScaleSuggestion::new("Aeolian", ScaleType::Diatonic, Some(Mode::Aeolian)),
                        ScaleSuggestion::new(
                            "Harmonic Minor",
                            ScaleType::HarmonicMinor,
                            Some(Mode::HarmonicMinor),
                        ),
                    ]
                } else {
                    vec![
                        ScaleSuggestion::new("Phrygian", ScaleType::Diatonic, Some(Mode::Phrygian)),
                        ScaleSuggestion::new("Mixolydian", ScaleType::Diatonic, Some(Mode::Mixolydian)),
                        ScaleSuggestion::new("Whole Tone", ScaleType::WholeTone, Some(Mode::WholeTone)),
                    ]
                }
            }
            "Funk" => {
                if is_minor {
                    vec![
                        ScaleSuggestion::new("Dorian", ScaleType::Diatonic, Some(Mode::Dorian)),
                        ScaleSuggestion::new(
                            "Minor Pentatonic",
                            ScaleType::PentatonicMinor,
                            Some(Mode::PentatonicMinor),
                        ),
                        ScaleSuggestion::new("Mixolydian", ScaleType::Diatonic, Some(Mode::Mixolydian)),
                    ]
                } else {
                    vec![
                        ScaleSuggestion::new("Mixolydian", ScaleType::Diatonic, Some(Mode::Mixolydian)),
                        ScaleSuggestion::new(
                            "Major Pentatonic",
                            ScaleType::PentatonicMajor,
                            Some(Mode::PentatonicMajor),
                        ),
                        ScaleSuggestion::new("Ionian", ScaleType::Diatonic, Some(Mode::Ionian)),
                    ]
                }
            }
            _ => vec![ScaleSuggestion::new(
                "Ionian",
                ScaleType::Diatonic,
                Some(Mode::Ionian),
            )],
        }
    }

}

fn find_root_pitch_index(label: &str) -> Option<usize> {
    let normalized = normalize_music_token(label);
    ROOT_PITCHES.iter().position(|pitch| {
        normalize_music_token(&format!("{pitch}")) == normalized
    })
}

fn find_chord_quality_index(label: &str) -> Option<usize> {
    let normalized = label.trim().to_ascii_lowercase();
    CHORD_QUALITIES
        .iter()
        .position(|(_, quality)| quality.to_ascii_lowercase() == normalized)
}

fn find_genre_index(label: &str) -> Option<usize> {
    let normalized = label.trim().to_ascii_lowercase();
    GENRES
        .iter()
        .position(|genre| genre.to_ascii_lowercase() == normalized)
}

fn normalize_music_token(label: &str) -> String {
    label
        .trim()
        .replace('#', "s")
        .replace('♯', "s")
        .replace('b', "b")
        .to_ascii_lowercase()
}

struct ScaleSuggestion {
    label: &'static str,
    scale_type: ScaleType,
    mode: Option<Mode>,
}

impl ScaleSuggestion {
    fn new(label: &'static str, scale_type: ScaleType, mode: Option<Mode>) -> Self {
        Self {
            label,
            scale_type,
            mode,
        }
    }
}
