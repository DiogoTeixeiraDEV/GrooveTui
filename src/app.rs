use std::sync::mpsc::Sender;

use crate::audio::AudioCommand;
use crate::entities::{genres, notes};

pub struct App {
    bpm: u64,
    is_playing: bool,
    genre: String,
    root_note: String,
    audio_tx: Sender<AudioCommand>,
}

impl App {
    pub fn new(tx: Sender<AudioCommand>) -> Self {
        Self {
            bpm: 120,
            is_playing: false,
            genre: genres::DEFAULT.to_string(),
            root_note: notes::DEFAULT_ROOT.to_string(),
            audio_tx: tx,
        }
    }

    pub fn bpm(&self) -> u64 {
        self.bpm
    }

    pub fn is_playing(&self) -> bool {
        self.is_playing
    }

    pub fn genre(&self) -> &str {
        &self.genre
    }

    pub fn root_note(&self) -> &str {
        &self.root_note
    }

    pub fn toggle_play(&mut self) {
        self.is_playing = !self.is_playing;
        let _ = self.audio_tx.send(AudioCommand::Toggle);
    }

    pub fn increase_bpm(&mut self) {
        self.bpm += 1;
        let _ = self.audio_tx.send(AudioCommand::SetBpm(self.bpm));
    }

    pub fn decrease_bpm(&mut self) {
        if self.bpm > 1 {
            self.bpm -= 1;
            let _ = self.audio_tx.send(AudioCommand::SetBpm(self.bpm));
        }
    }

    pub fn quit_audio(&self) {
        let _ = self.audio_tx.send(AudioCommand::Quit);
    }
}
