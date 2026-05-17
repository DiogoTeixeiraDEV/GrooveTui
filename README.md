# GrooveTui

GrooveTui is a minimalist guitar practice companion for the terminal. Built in Rust, it combines a metronome, harmony suggestions, a fretboard view, YouTube backing-track search, and a real-time tuner inside a fast TUI.

## Screenshots

### Groove tab

![Groove tab screenshot](assets/screenshots/groove.png)

### Tuner tab

![Tuner tab screenshot](assets/screenshots/tuner.png)

## Usage

Install it with:

```bash
cargo install groove-tui
```

```bash
groove-tui
```

```bash
cargo run --release
```

Backing-track search and playback use external command-line tools:

```bash
brew install yt-dlp mpv
```

## Controls

Global
- `Tab`: switch tabs
- `Q`: quit

Groove tab
- `Space`: play/pause metronome
- `Arrow Up`: increase BPM
- `Arrow Down`: decrease BPM
- `Arrow Left`: previous root note
- `Arrow Right`: next root note
- `,`: minor chord quality
- `.`: major chord quality
- `G`: previous genre
- `H`: next genre

Tuner tab
- `Space`: toggle capture
- `Arrow Left`: previous input device
- `Arrow Right`: next input device
- `Arrow Up`: increase input gain
- `Arrow Down`: decrease input gain
- `M`: toggle auto/manual mode
- `A`: previous target string (manual mode)
- `D`: next target string (manual mode)

Backing tab
- `/`: edit search query
- `Enter`: run search
- `Esc`: cancel query editing
- `Arrow Up`: previous result
- `Arrow Down`: next result
- `Space`: play/pause selected track
- `F`: favorite/unfavorite selected track
- `V`: switch between search results and favorites
- `S`: stop playback

Use `Tab` to switch between Groove, Tuner, and Backing. In the Tuner tab, select an input device and press `Space` to start capture.

When a backing track starts, GrooveTui tries to infer the key, major/minor quality, and genre from the track title/search text, then updates the Groove tab's scale suggestions and fretboard overlay.

## Tech Stack

- Rust
- ratatui
- crossterm
- rodio
- cpal
- pitch-detection
- rust-music-theory
- yt-dlp
- mpv
