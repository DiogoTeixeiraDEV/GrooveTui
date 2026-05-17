use std::io::{BufRead, BufReader, Write};
#[cfg(unix)]
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use crate::audio::AudioCommand;
use crate::music::MusicState;
use crate::tui::state::{TunerState, TuningMode};

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum AppTab {
    Groove,
    Tuner,
    Backing,
}

#[derive(Clone, Debug, serde::Deserialize)]
struct YtDlpSearch {
    entries: Option<Vec<YtDlpEntry>>,
}

#[derive(Clone, Debug, serde::Deserialize)]
struct YtDlpEntry {
    id: Option<String>,
    title: Option<String>,
    channel: Option<String>,
    uploader: Option<String>,
    duration: Option<f64>,
    webpage_url: Option<String>,
    url: Option<String>,
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct BackingTrack {
    title: String,
    channel: Option<String>,
    duration_seconds: Option<u64>,
    url: String,
    source_query: Option<String>,
}

impl BackingTrack {
    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn channel(&self) -> Option<&str> {
        self.channel.as_deref()
    }

    pub fn duration_label(&self) -> Option<String> {
        let seconds = self.duration_seconds?;
        let minutes = seconds / 60;
        let seconds = seconds % 60;
        Some(format!("{minutes}:{seconds:02}"))
    }

    fn search_text(&self) -> String {
        match &self.source_query {
            Some(query) if !query.trim().is_empty() => format!("{} {}", self.title, query),
            _ => self.title.clone(),
        }
    }
}

#[derive(Clone, Debug)]
pub enum BackingSearchState {
    Idle,
    Searching,
    Ready,
    Failed(String),
}

#[derive(Clone, Debug)]
pub enum BackingPlayerState {
    Stopped,
    Playing,
    Paused,
    Failed(String),
}

enum BackingSearchMessage {
    Complete(Result<Vec<BackingTrack>, String>),
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum BackingLibraryView {
    SearchResults,
    Favorites,
}

#[derive(Clone, Debug, Default)]
struct BackingTrackContext {
    root: Option<String>,
    quality: Option<String>,
    genre: Option<String>,
}

pub struct BackingTracksState {
    query: String,
    editing_query: bool,
    selected_index: usize,
    results: Vec<BackingTrack>,
    search_state: BackingSearchState,
    search_rx: Option<Receiver<BackingSearchMessage>>,
    player_state: BackingPlayerState,
    player: Option<Child>,
    now_playing: Option<BackingTrack>,
    ipc_socket: Option<PathBuf>,
    progress: PlayerProgress,
    last_progress_poll: Instant,
    favorites: Vec<BackingTrack>,
    library_view: BackingLibraryView,
    message: Option<String>,
}

#[derive(Clone, Debug, Default)]
pub struct PlayerProgress {
    elapsed_seconds: Option<f64>,
    duration_seconds: Option<f64>,
}

impl PlayerProgress {
    pub fn elapsed_label(&self) -> String {
        self.elapsed_seconds
            .map(format_seconds)
            .unwrap_or_else(|| "--:--".to_string())
    }

    pub fn duration_label(&self) -> String {
        self.duration_seconds
            .map(format_seconds)
            .unwrap_or_else(|| "--:--".to_string())
    }

    pub fn fraction(&self) -> f64 {
        let Some(elapsed) = self.elapsed_seconds else {
            return 0.0;
        };
        let Some(duration) = self.duration_seconds else {
            return 0.0;
        };
        if duration <= 0.0 {
            return 0.0;
        }
        (elapsed / duration).clamp(0.0, 1.0)
    }
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
    backing_tracks: BackingTracksState,
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
            backing_tracks: BackingTracksState::new(),
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
            AppTab::Backing => 2,
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
        self.backing_tracks.update();

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
            AppTab::Tuner => AppTab::Backing,
            AppTab::Backing => AppTab::Groove,
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

    pub fn tuner_mode(&self) -> TuningMode {
        self.tuner.tuning_mode()
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

    pub fn increase_tuner_gain(&mut self) {
        self.tuner.increase_input_gain();
    }

    pub fn decrease_tuner_gain(&mut self) {
        self.tuner.decrease_input_gain();
    }

    pub fn toggle_tuner_mode(&mut self) {
        self.tuner.toggle_tuning_mode();
    }

    pub fn next_tuner_string(&mut self) {
        self.tuner.next_string();
    }

    pub fn prev_tuner_string(&mut self) {
        self.tuner.prev_string();
    }

    pub fn backing_tracks(&self) -> &BackingTracksState {
        &self.backing_tracks
    }

    pub fn backing_begin_search_edit(&mut self) {
        self.backing_tracks.begin_search_edit();
    }

    pub fn backing_push_query_char(&mut self, c: char) {
        self.backing_tracks.push_query_char(c);
    }

    pub fn backing_backspace_query(&mut self) {
        self.backing_tracks.backspace_query();
    }

    pub fn backing_cancel_query_edit(&mut self) {
        self.backing_tracks.cancel_query_edit();
    }

    pub fn backing_submit_search(&mut self) {
        self.backing_tracks.submit_search();
    }

    pub fn backing_next_result(&mut self) {
        self.backing_tracks.next_result();
    }

    pub fn backing_prev_result(&mut self) {
        self.backing_tracks.prev_result();
    }

    pub fn backing_toggle_selected(&mut self) {
        self.backing_tracks.toggle_selected();
        if let Some(context) = self.backing_tracks.now_playing_context() {
            self.music.set_context(
                context.root.as_deref(),
                context.quality.as_deref(),
                context.genre.as_deref(),
            );
        }
    }

    pub fn backing_stop(&mut self) {
        self.backing_tracks.stop();
    }

    pub fn backing_toggle_favorite(&mut self) {
        self.backing_tracks.toggle_favorite();
    }

    pub fn backing_toggle_library_view(&mut self) {
        self.backing_tracks.toggle_library_view();
    }
}

impl BackingTracksState {
    const SEARCH_LIMIT: usize = 10;

    pub fn new() -> Self {
        Self {
            query: String::new(),
            editing_query: false,
            selected_index: 0,
            results: Vec::new(),
            search_state: BackingSearchState::Idle,
            search_rx: None,
            player_state: BackingPlayerState::Stopped,
            player: None,
            now_playing: None,
            ipc_socket: None,
            progress: PlayerProgress::default(),
            last_progress_poll: Instant::now(),
            favorites: load_favorites(),
            library_view: BackingLibraryView::SearchResults,
            message: None,
        }
    }

    pub fn query(&self) -> &str {
        &self.query
    }

    pub fn editing_query(&self) -> bool {
        self.editing_query
    }

    pub fn selected_index(&self) -> usize {
        self.selected_index
    }

    pub fn results(&self) -> &[BackingTrack] {
        match self.library_view {
            BackingLibraryView::SearchResults => &self.results,
            BackingLibraryView::Favorites => &self.favorites,
        }
    }

    pub fn favorites_len(&self) -> usize {
        self.favorites.len()
    }

    pub fn library_view(&self) -> BackingLibraryView {
        self.library_view
    }

    pub fn search_state(&self) -> &BackingSearchState {
        &self.search_state
    }

    pub fn player_state(&self) -> &BackingPlayerState {
        &self.player_state
    }

    pub fn now_playing(&self) -> Option<&BackingTrack> {
        self.now_playing.as_ref()
    }

    pub fn progress(&self) -> &PlayerProgress {
        &self.progress
    }

    pub fn message(&self) -> Option<&str> {
        self.message.as_deref()
    }

    pub fn selected_is_favorite(&self) -> bool {
        self.selected_track()
            .is_some_and(|track| self.is_favorite(track))
    }

    fn begin_search_edit(&mut self) {
        self.editing_query = true;
        self.library_view = BackingLibraryView::SearchResults;
    }

    fn push_query_char(&mut self, c: char) {
        if self.editing_query && !c.is_control() {
            self.query.push(c);
        }
    }

    fn backspace_query(&mut self) {
        if self.editing_query {
            self.query.pop();
        }
    }

    fn cancel_query_edit(&mut self) {
        self.editing_query = false;
    }

    fn submit_search(&mut self) {
        if self.query.trim().is_empty() {
            self.search_state =
                BackingSearchState::Failed("Type something to search first.".to_string());
            return;
        }

        let query = self.query.trim().to_string();
        let (tx, rx) = mpsc::channel();
        self.search_rx = Some(rx);
        self.search_state = BackingSearchState::Searching;
        self.editing_query = false;
        self.library_view = BackingLibraryView::SearchResults;
        self.message = None;

        thread::spawn(move || {
            let result = search_youtube(&query, Self::SEARCH_LIMIT);
            let _ = tx.send(BackingSearchMessage::Complete(result));
        });
    }

    fn update(&mut self) {
        if let Some(rx) = self.search_rx.take() {
            match rx.try_recv() {
                Ok(BackingSearchMessage::Complete(Ok(results))) => {
                    self.results = results;
                    self.selected_index = 0;
                    self.search_state = BackingSearchState::Ready;
                    self.message = None;
                }
                Ok(BackingSearchMessage::Complete(Err(message))) => {
                    self.results.clear();
                    self.selected_index = 0;
                    self.search_state = BackingSearchState::Failed(message);
                }
                Err(mpsc::TryRecvError::Empty) => {
                    self.search_rx = Some(rx);
                }
                Err(mpsc::TryRecvError::Disconnected) => {
                    self.search_state = BackingSearchState::Failed(
                        "Search worker stopped unexpectedly.".to_string(),
                    );
                }
            }
        }

        if let Some(child) = self.player.as_mut() {
            match child.try_wait() {
                Ok(Some(status)) => {
                    self.player = None;
                    self.now_playing = None;
                    self.cleanup_ipc_socket();
                    self.progress = PlayerProgress::default();
                    self.player_state = if status.success() {
                        BackingPlayerState::Stopped
                    } else {
                        BackingPlayerState::Failed(format!("Player exited with {status}."))
                    };
                }
                Ok(None) => {}
                Err(err) => {
                    self.player = None;
                    self.now_playing = None;
                    self.player_state = BackingPlayerState::Failed(format!("Player error: {err}"));
                }
            }
        }

        self.poll_player_progress();
    }

    fn next_result(&mut self) {
        if !self.results.is_empty() {
            self.selected_index = (self.selected_index + 1) % self.results.len();
        }
    }

    fn prev_result(&mut self) {
        if self.results.is_empty() {
            return;
        }
        self.selected_index = if self.selected_index == 0 {
            self.results.len() - 1
        } else {
            self.selected_index - 1
        };
    }

    fn toggle_selected(&mut self) {
        let Some(track) = self.selected_track().cloned() else {
            self.player_state =
                BackingPlayerState::Failed("Search and select a track first.".to_string());
            return;
        };

        if self
            .now_playing
            .as_ref()
            .is_some_and(|now_playing| now_playing.url == track.url)
        {
            self.toggle_pause();
            return;
        }

        self.stop();
        self.play_track(track);
    }

    fn play_track(&mut self, track: BackingTrack) {
        let ipc_socket = player_socket_path();
        let _ = std::fs::remove_file(&ipc_socket);

        match Command::new("mpv")
            .arg("--no-video")
            .arg("--really-quiet")
            .arg(format!("--input-ipc-server={}", ipc_socket.display()))
            .arg(&track.url)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
        {
            Ok(child) => {
                self.player = Some(child);
                self.now_playing = Some(track);
                self.ipc_socket = Some(ipc_socket);
                self.progress = PlayerProgress::default();
                self.last_progress_poll = Instant::now();
                self.player_state = BackingPlayerState::Playing;
            }
            Err(err) => {
                let _ = std::fs::remove_file(&ipc_socket);
                self.player_state = BackingPlayerState::Failed(format!(
                    "Could not start mpv: {err}. Install mpv to play audio."
                ));
            }
        }
    }

    fn stop(&mut self) {
        if let Some(mut child) = self.player.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
        self.now_playing = None;
        self.cleanup_ipc_socket();
        self.progress = PlayerProgress::default();
        self.player_state = BackingPlayerState::Stopped;
    }

    fn toggle_favorite(&mut self) {
        let Some(track) = self.selected_track().cloned() else {
            self.message = Some("Select a track before favoriting.".to_string());
            return;
        };

        if let Some(index) = self
            .favorites
            .iter()
            .position(|favorite| favorite.url == track.url)
        {
            let removed = self.favorites.remove(index);
            self.message = Some(format!("Removed favorite: {}", removed.title));
            if self.library_view == BackingLibraryView::Favorites
                && self.selected_index >= self.favorites.len()
            {
                self.selected_index = self.favorites.len().saturating_sub(1);
            }
        } else {
            self.favorites.push(track.clone());
            self.message = Some(format!("Favorited: {}", track.title));
        }

        if let Err(err) = save_favorites(&self.favorites) {
            self.message = Some(format!("Could not save favorites: {err}"));
        }
    }

    fn toggle_library_view(&mut self) {
        self.library_view = match self.library_view {
            BackingLibraryView::SearchResults => BackingLibraryView::Favorites,
            BackingLibraryView::Favorites => BackingLibraryView::SearchResults,
        };
        self.selected_index = 0;
        self.message = None;
    }

    fn selected_track(&self) -> Option<&BackingTrack> {
        self.results().get(self.selected_index)
    }

    pub fn is_favorite(&self, track: &BackingTrack) -> bool {
        self.favorites
            .iter()
            .any(|favorite| favorite.url == track.url)
    }

    fn now_playing_context(&self) -> Option<BackingTrackContext> {
        self.now_playing.as_ref().map(infer_track_context)
    }

    fn toggle_pause(&mut self) {
        let should_pause = matches!(self.player_state, BackingPlayerState::Playing);
        match self.mpv_set_pause(should_pause) {
            Ok(()) => {
                self.player_state = if should_pause {
                    BackingPlayerState::Paused
                } else {
                    BackingPlayerState::Playing
                };
            }
            Err(message) => {
                self.player_state = BackingPlayerState::Failed(message);
            }
        }
    }

    fn poll_player_progress(&mut self) {
        if self.player.is_none() || self.last_progress_poll.elapsed() < Duration::from_millis(250) {
            return;
        }
        self.last_progress_poll = Instant::now();

        if let Ok(Some(elapsed)) = self.mpv_get_number("playback-time") {
            self.progress.elapsed_seconds = Some(elapsed.max(0.0));
        }

        if let Ok(Some(duration)) = self.mpv_get_number("duration") {
            self.progress.duration_seconds = Some(duration.max(0.0));
        } else if let Some(track) = self.now_playing.as_ref() {
            self.progress.duration_seconds = track.duration_seconds.map(|seconds| seconds as f64);
        }

        if let Ok(Some(paused)) = self.mpv_get_bool("pause") {
            self.player_state = if paused {
                BackingPlayerState::Paused
            } else {
                BackingPlayerState::Playing
            };
        }
    }

    fn mpv_get_number(&self, property: &str) -> Result<Option<f64>, String> {
        let command = serde_json::json!({ "command": ["get_property", property] });
        let response = self.mpv_command(command)?;
        Ok(response.get("data").and_then(|value| value.as_f64()))
    }

    fn mpv_get_bool(&self, property: &str) -> Result<Option<bool>, String> {
        let command = serde_json::json!({ "command": ["get_property", property] });
        let response = self.mpv_command(command)?;
        Ok(response.get("data").and_then(|value| value.as_bool()))
    }

    fn mpv_set_pause(&self, paused: bool) -> Result<(), String> {
        let command = serde_json::json!({ "command": ["set_property", "pause", paused] });
        let response = self.mpv_command(command)?;
        if response
            .get("error")
            .and_then(|value| value.as_str())
            .is_some_and(|error| error != "success")
        {
            return Err("Could not update mpv playback state.".to_string());
        }
        Ok(())
    }

    #[cfg(unix)]
    fn mpv_command(&self, command: serde_json::Value) -> Result<serde_json::Value, String> {
        let socket = self
            .ipc_socket
            .as_ref()
            .ok_or_else(|| "Player control socket is not ready yet.".to_string())?;
        let mut stream = UnixStream::connect(socket)
            .map_err(|err| format!("Could not control mpv yet: {err}"))?;
        stream
            .set_read_timeout(Some(Duration::from_millis(120)))
            .map_err(|err| format!("Could not set mpv timeout: {err}"))?;
        let mut payload = serde_json::to_vec(&command)
            .map_err(|err| format!("Could not encode mpv command: {err}"))?;
        payload.push(b'\n');
        stream
            .write_all(&payload)
            .map_err(|err| format!("Could not send command to mpv: {err}"))?;

        let mut line = String::new();
        let mut reader = BufReader::new(stream);
        reader
            .read_line(&mut line)
            .map_err(|err| format!("Could not read mpv response: {err}"))?;
        serde_json::from_str(&line).map_err(|err| format!("Could not parse mpv response: {err}"))
    }

    #[cfg(not(unix))]
    fn mpv_command(&self, _command: serde_json::Value) -> Result<serde_json::Value, String> {
        Err("mpv progress controls require Unix socket support.".to_string())
    }

    fn cleanup_ipc_socket(&mut self) {
        if let Some(socket) = self.ipc_socket.take() {
            let _ = std::fs::remove_file(socket);
        }
    }
}

impl Drop for BackingTracksState {
    fn drop(&mut self) {
        self.stop();
    }
}

fn search_youtube(query: &str, limit: usize) -> Result<Vec<BackingTrack>, String> {
    let search = format!("ytsearch{limit}:{query}");
    let output = Command::new("yt-dlp")
        .arg("--dump-single-json")
        .arg("--flat-playlist")
        .arg("--no-warnings")
        .arg(search)
        .output()
        .map_err(|err| format!("Could not run yt-dlp: {err}. Install yt-dlp to search YouTube."))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let message = stderr.trim();
        return Err(if message.is_empty() {
            format!("yt-dlp exited with {}.", output.status)
        } else {
            message.to_string()
        });
    }

    let parsed: YtDlpSearch = serde_json::from_slice(&output.stdout)
        .map_err(|err| format!("Could not parse yt-dlp output: {err}"))?;

    let tracks = parsed
        .entries
        .unwrap_or_default()
        .into_iter()
        .filter_map(|entry| {
            let title = entry.title?;
            let url = entry.webpage_url.or(entry.url).or(entry.id)?;
            Some(BackingTrack {
                title,
                channel: entry.channel.or(entry.uploader),
                duration_seconds: entry.duration.map(|seconds| seconds.round() as u64),
                url: youtube_watch_url(&url),
                source_query: Some(query.to_string()),
            })
        })
        .collect::<Vec<_>>();

    Ok(tracks)
}

fn infer_track_context(track: &BackingTrack) -> BackingTrackContext {
    let text = track.search_text();
    BackingTrackContext {
        root: infer_root(&text),
        quality: infer_quality(&text),
        genre: infer_genre(&text),
    }
}

fn infer_root(text: &str) -> Option<String> {
    let normalized = normalize_inference_text(text);
    let tokens = normalized.split_whitespace().collect::<Vec<_>>();
    for (index, token) in tokens.iter().enumerate() {
        if matches!(*token, "in" | "key" | "of") {
            if let Some(root) = tokens
                .get(index + 1)
                .and_then(|token| root_from_token(token))
            {
                return Some(root);
            }
        }
    }

    tokens.iter().find_map(|token| root_from_token(token))
}

fn infer_quality(text: &str) -> Option<String> {
    let normalized = normalize_inference_text(text);
    let has_minor_key_token = normalized.split_whitespace().any(|token| {
        matches!(
            token,
            "cm" | "c#m"
                | "dbm"
                | "dm"
                | "d#m"
                | "ebm"
                | "em"
                | "fm"
                | "f#m"
                | "gbm"
                | "gm"
                | "g#m"
                | "abm"
                | "am"
                | "a#m"
                | "bbm"
                | "bm"
        )
    });
    if normalized.contains(" minor ")
        || normalized.contains(" min ")
        || normalized.contains(" dorian ")
        || normalized.contains(" aeolian ")
        || has_minor_key_token
    {
        Some("Minor".to_string())
    } else if normalized.contains(" major ") || normalized.contains(" maj ") {
        Some("Major".to_string())
    } else {
        None
    }
}

fn infer_genre(text: &str) -> Option<String> {
    let normalized = normalize_inference_text(text);
    [
        ("blues", "Blues"),
        ("rock", "Rock"),
        ("jazz", "Jazz"),
        ("metal", "Metal"),
        ("funk", "Funk"),
    ]
    .iter()
    .find_map(|(needle, genre)| normalized.contains(needle).then(|| (*genre).to_string()))
}

fn root_from_token(token: &str) -> Option<String> {
    match token.trim_matches(|c: char| !c.is_ascii_alphanumeric() && c != '#') {
        "c" => Some("C".to_string()),
        "cm" | "cmin" => Some("C".to_string()),
        "c#" | "cs" | "db" => Some("Cs".to_string()),
        "c#m" | "c#min" | "csm" | "csmin" | "dbm" | "dbmin" => Some("Cs".to_string()),
        "d" => Some("D".to_string()),
        "dm" | "dmin" => Some("D".to_string()),
        "d#" | "ds" | "eb" => Some("Ds".to_string()),
        "d#m" | "d#min" | "dsm" | "dsmin" | "ebm" | "ebmin" => Some("Ds".to_string()),
        "e" => Some("E".to_string()),
        "em" | "emin" => Some("E".to_string()),
        "f" => Some("F".to_string()),
        "fm" | "fmin" => Some("F".to_string()),
        "f#" | "fs" | "gb" => Some("Fs".to_string()),
        "f#m" | "f#min" | "fsm" | "fsmin" | "gbm" | "gbmin" => Some("Fs".to_string()),
        "g" => Some("G".to_string()),
        "gm" | "gmin" => Some("G".to_string()),
        "g#" | "gs" | "ab" => Some("Gs".to_string()),
        "g#m" | "g#min" | "gsm" | "gsmin" | "abm" | "abmin" => Some("Gs".to_string()),
        "a" => Some("A".to_string()),
        "am" | "amin" => Some("A".to_string()),
        "a#" | "as" | "bb" => Some("As".to_string()),
        "a#m" | "a#min" | "asm" | "asmin" | "bbm" | "bbmin" => Some("As".to_string()),
        "b" => Some("B".to_string()),
        "bm" | "bmin" => Some("B".to_string()),
        _ => None,
    }
}

fn normalize_inference_text(text: &str) -> String {
    format!(
        " {} ",
        text.replace('♯', "#")
            .replace('♭', "b")
            .replace('-', " ")
            .replace('_', " ")
            .to_ascii_lowercase()
    )
}

fn load_favorites() -> Vec<BackingTrack> {
    let path = favorites_path();
    let Ok(bytes) = std::fs::read(path) else {
        return Vec::new();
    };
    serde_json::from_slice(&bytes).unwrap_or_default()
}

fn save_favorites(favorites: &[BackingTrack]) -> Result<(), String> {
    let path = favorites_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|err| format!("failed to create {}: {err}", parent.display()))?;
    }
    let bytes = serde_json::to_vec_pretty(favorites)
        .map_err(|err| format!("failed to encode favorites: {err}"))?;
    std::fs::write(&path, bytes).map_err(|err| format!("failed to write {}: {err}", path.display()))
}

fn favorites_path() -> PathBuf {
    data_dir().join("favorites.json")
}

fn data_dir() -> PathBuf {
    if let Some(home) = std::env::var_os("HOME") {
        return Path::new(&home)
            .join(".local")
            .join("share")
            .join("groove-tui");
    }
    std::env::temp_dir().join("groove-tui")
}

fn youtube_watch_url(value: &str) -> String {
    if value.starts_with("http://") || value.starts_with("https://") {
        value.to_string()
    } else {
        format!("https://www.youtube.com/watch?v={value}")
    }
}

fn format_seconds(seconds: f64) -> String {
    let total_seconds = seconds.round().max(0.0) as u64;
    let hours = total_seconds / 3600;
    let minutes = (total_seconds % 3600) / 60;
    let seconds = total_seconds % 60;

    if hours > 0 {
        format!("{hours}:{minutes:02}:{seconds:02}")
    } else {
        format!("{minutes}:{seconds:02}")
    }
}

fn player_socket_path() -> PathBuf {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default();
    std::env::temp_dir().join(format!(
        "groove-tui-mpv-{}-{timestamp}.sock",
        std::process::id()
    ))
}
