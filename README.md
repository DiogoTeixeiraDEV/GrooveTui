# 🎸 GrooveTui

**GrooveTui** is a minimalist guitar practice companion for the terminal. Built in **Rust**, it combines the efficiency of TUIs (Terminal User Interfaces) with hands-on music theory learning.

The goal of GrooveTui is to be the ideal companion for your practice sessions, offering a smooth metronome, interactive chord and scale exploration, and a visual guitar fretboard with scale patterns.

## ⚡ Features

- **Smooth Metronome Animation:** Realistic pendulum that swings in sync with your BPM with visual tick feedback
- **Interactive Chord & Scale Explorer:** Switch between genres (Blues, Rock, Jazz, Metal, Funk), root notes (C to B), and chord qualities (Major/Minor)
- **Visual Guitar Fretboard:** Shows the currently selected scale pattern on a 6-string guitar neck with labeled frets
- **Real-time BPM Control:** Adjust tempo on the fly without interrupting playback
- **Multi-threaded Audio:** Metronome sound runs smoothly regardless of UI updates
- **Terminal-Native UI:** Fast, responsive navigation with modern Ratatui interface

## 🛠️ Tech Stack

- **Rust** (Core Language)
- **Ratatui** (Terminal UI Framework)
- **Rodio** (Audio Engine)
- **Crossterm** (Terminal Backend)
- **rust_music_theory** (Music Theory Library)

## ⌨️ Controls

- `Space`: Play/Pause the metronome
- `Arrow Up`: Increase BPM
- `Arrow Down`: Decrease BPM
- `Arrow Left`: Previous root note (C → B)
- `Arrow Right`: Next root note (C → D → ... → B)
- `,` (Comma): Switch to minor chord quality
- `.` (Period): Switch to major chord quality
- `G`: Previous genre
- `H`: Next genre
- `Q`: Quit safely

## 🎵 How to Use

1. **Select a genre** using `G`/`H` keys to explore different musical styles
2. **Choose your root note** with `Arrow Left`/`Right` 
3. **Toggle chord quality** between Major and Minor using `,` and `.`
4. **Watch the fretboard** display the suggested scale patterns
5. **Start the metronome** with `Space` and adjust BPM with `Arrow Up`/`Down`

The metronome shows a smooth pendulum animation that swings in real-time synced to your selected BPM. The harmony panel displays the current chord and suggested scales to practice.

---
*A deep dive into Rust, TUI development, and interactive music theory education.*
