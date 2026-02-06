use std::sync::mpsc::Sender;
use std::time::{Duration, Instant};

use crate::audio::AudioCommand;
use crate::music::MusicState;
use crate::tui::state::TunerState;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum AppTab {
    Groove,
    Tuner,
}

pub struct App {
    bpm: u64,
    is_playing: bool,
    music: MusicState,
    start_time: Instant,
    last_tick: Instant,
    audio_tx: Sender<AudioCommand>,
    current_tab: AppTab,
    tuner: TunerState,
}

impl App {
    pub fn new(tx: Sender<AudioCommand>) -> Self {
        Self {
            bpm: 120,
            is_playing: false,
            music: MusicState::new(),
            start_time: Instant::now(),
            last_tick: Instant::now(),
            audio_tx: tx,
            current_tab: AppTab::Groove,
            tuner: TunerState::new(),
        }
    }

    pub fn bpm(&self) -> u64 {
        self.bpm
    }

    pub fn is_playing(&self) -> bool {
        self.is_playing
    }

    pub fn genre(&self) -> &str {
        self.music.genre()
    }

    pub fn root_pitch_label(&self) -> String {
        self.music.root_pitch_label()
    }

    pub fn chord_quality_label(&self) -> &str {
        self.music.chord_quality_label()
    }

    pub fn chord_notes_label(&self) -> String {
        self.music.chord_notes_label()
    }

    pub fn suggested_scales(&self) -> Vec<String> {
        self.music.suggested_scales()
    }

    pub fn current_tab(&self) -> AppTab {
        self.current_tab
    }

    pub fn current_tab_index(&self) -> usize {
        match self.current_tab {
            AppTab::Groove => 0,
            AppTab::Tuner => 1,
        }
    }

    pub fn root_pitch_class(&self) -> u8 {
        self.music.root_pitch_class()
    }

    pub fn first_suggested_scale_label(&self) -> String {
        self.music.first_suggested_scale_label()
    }

    pub fn first_suggested_scale_pitch_classes(&self) -> Vec<u8> {
        self.music.first_suggested_scale_pitch_classes()
    }

    pub fn toggle_play(&mut self) {
        self.is_playing = !self.is_playing;
        if self.is_playing {
            let now = Instant::now();
            self.start_time = now;
            self.last_tick = now;
        }
        let _ = self.audio_tx.send(AudioCommand::Toggle);
    }

    pub fn update(&mut self) {
        
        self.tuner.update();
        
        if !self.is_playing {
            return;
        }

        let period = Duration::from_secs_f32(60.0 / self.bpm as f32);
        while self.last_tick.elapsed() >= period {
            self.last_tick = self.last_tick + period;
        }
    }

    pub fn next_root_pitch(&mut self) {
        self.music.next_root_pitch();
    }

    pub fn next_tab(&mut self) {
        self.current_tab = match self.current_tab {
            AppTab::Groove => AppTab::Tuner,
            AppTab::Tuner => AppTab::Groove,
        };
    }

    pub fn prev_root_pitch(&mut self) {
        self.music.prev_root_pitch();
    }

    pub fn next_chord_quality(&mut self) {
        self.music.next_chord_quality();
    }

    pub fn prev_chord_quality(&mut self) {
        self.music.prev_chord_quality();
    }

    pub fn next_genre(&mut self) {
        self.music.next_genre();
    }

    pub fn prev_genre(&mut self) {
        self.music.prev_genre();
    }

    pub fn increase_bpm(&mut self) {
        self.bpm += 1;
        if self.is_playing {
            let now = Instant::now();
            self.start_time = now;
            self.last_tick = now;
        }
        let _ = self.audio_tx.send(AudioCommand::SetBpm(self.bpm));
    }

    pub fn decrease_bpm(&mut self) {
        if self.bpm > 1 {
            self.bpm -= 1;
            if self.is_playing {
                let now = Instant::now();
                self.start_time = now;
                self.last_tick = now;
            }
            let _ = self.audio_tx.send(AudioCommand::SetBpm(self.bpm));
        }
    }

    pub fn metronome_phase(&self) -> f32 {
        if !self.is_playing {
            return 0.5;
        }

        let period = 60.0 / self.bpm as f32;
        let elapsed = Instant::now().duration_since(self.start_time).as_secs_f32();
        let mut progress = elapsed / (period * 2.0);
        if progress > 1.0 {
            progress = progress.fract();
        }

        let position = if progress < 0.25 {
            0.5 + progress * 2.0
        } else if progress < 0.75 {
            1.0 - (progress - 0.25) * 2.0
        } else {
            (progress - 0.75) * 2.0
        };

        position.clamp(0.0, 1.0)
    }

    pub fn metronome_flash(&self) -> bool {
        self.is_playing && self.metronome_progress() < 0.06
    }

    pub fn metronome_progress(&self) -> f32 {
        if !self.is_playing {
            return 0.0;
        }

        let period = 60.0 / self.bpm as f32;
        let elapsed = self.last_tick.elapsed().as_secs_f32();
        let mut progress = elapsed / period;
        if progress > 1.0 {
            progress = progress.fract();
        }
        progress
    }

    pub fn quit_audio(&self) {
        let _ = self.audio_tx.send(AudioCommand::Quit);
    }

    

    pub fn tuner(&self) -> &TunerState {
        &self.tuner
    }

    pub fn toggle_tuner_capture(&mut self) {
        self.tuner.toggle_capture();
    }

    pub fn next_tuner_device(&mut self) {
        self.tuner.next_device();
    }

    pub fn prev_tuner_device(&mut self) {
        self.tuner.prev_device();
    }
}
