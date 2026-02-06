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
- `Tab`: Switch Tabs

## 🎵 How to Use

1. **Select a genre** using `G`/`H` keys to explore different musical styles
2. **Choose your root note** with `Arrow Left`/`Right` 
3. **Toggle chord quality** between Major and Minor using `,` and `.`
4. **Watch the fretboard** display the suggested scale patterns
5. **Start the metronome** with `Space` and adjust BPM with `Arrow Up`/`Down`

The metronome shows a smooth pendulum animation that swings in real-time synced to your selected BPM. The harmony panel displays the current chord and suggested scales to practice.

---

## Future Funcionalities

🚀 Development Roadmap - **Tuner**
1. Audio Infrastructure (Core)

    [x] Device Selection: List and select input devices across platforms using cpal.

    [x] Capture Stream: Configure audio callbacks to capture raw samples in f32 format.

    [x] Channel Handling: Implement logic to isolate the guitar channel (Mono/Left) from multi-channel interfaces.

    [x] Circular Buffer: Implement a thread-safe ringbuf (Producer/Consumer) to stream audio data to the logic thread without blocking the UI.

2. Digital Signal Processing (DSP)

    [x] Pitch Detection: Integrate the YIN algorithm (via pitch-detection crate) for fundamental frequency extraction.

    [x] Noise Gate: Implement a threshold to ignore background noise and silence.

    [x] Signal Smoothing: Apply a Moving Average filter to stabilize the needle and prevent jitter.

    [x] Musical Mapping: Create a conversion engine: Frequency (Hz) → MIDI Note → Cents (pitch deviation).

3. User Interface (TUI/Ratatui)

    [x] Main Dashboard: Layout featuring the detected note name, octave, and real-time frequency.

    [x] Precision Meter: Implement a Canvas widget for a fluid analog needle using Braille patterns for high-resolution graphics.

    [x] Dynamic Feedback: Visual cues using colors (e.g., Green for "In Tune", Red/Yellow for "Sharp/Flat").

    [x] Settings Menu: Interactive UI to switch audio devices and adjust the reference pitch (e.g., A=440Hz vs. A=432Hz).

4. Performance & Optimization

    [ ] Multi-threading: Decouple audio processing from the Ratatui render loop to ensure 60+ FPS UI performance.

    [ ] Latency Tuning: Optimize buffer sizes to balance instantaneous response with low-frequency (E2) accuracy.
