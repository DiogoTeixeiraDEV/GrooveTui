# GrooveTui

GrooveTui is a minimalist guitar practice companion for the terminal. Built in Rust, it combines a metronome, harmony suggestions, a fretboard view, and a real-time tuner inside a fast TUI.

## Screenshots

### Groove tab

![Groove tab screenshot](assets/screenshots/groove-tab.svg)

### Tuner tab

![Tuner tab screenshot](assets/screenshots/tuner-tab.svg)

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

## Releases

GitHub Actions builds release archives for Linux, Windows, macOS x86_64, and macOS arm64 whenever you push a tag that starts with `v`.

To publish a release:

```bash
git tag v0.2.1
git push origin v0.2.1
```

When the workflow finishes, GitHub will create a Release page with downloadable files such as:

- `groove-tui-linux-x86_64.tar.gz`
- `groove-tui-windows-x86_64.zip`
- `groove-tui-macos-x86_64.tar.gz`
- `groove-tui-macos-arm64.tar.gz`

People can download the release from the GitHub Releases page in the browser, or with the GitHub CLI:

```bash
gh release download v0.2.1
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


Use `Tab` to switch between Groove and Tuner. In the Tuner tab, select an input device and press `Space` to start capture.

## Tech Stack

- Rust
- ratatui
- crossterm
- rodio
- cpal
- pitch-detection
- rust-music-theory
