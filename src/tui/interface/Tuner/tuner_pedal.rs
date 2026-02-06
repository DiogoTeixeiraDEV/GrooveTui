use ratatui::{
    prelude::*,
    widgets::{canvas::Canvas, Block, Borders, Paragraph},
    widgets::canvas::Line as CanvasLine,
};

use crate::tui::state::TunerState;

const TARGETS: [(&str, f32); 6] = [
    ("E2", 82.0),
    ("A2", 110.0),
    ("D3", 146.8),
    ("G3", 196.0),
    ("B3", 246.9),
    ("E4", 329.63),
];

pub fn render(f: &mut Frame, area: Rect, tuner: &TunerState) {
    let block = Block::default().borders(Borders::ALL);
    let inner = block.inner(area);
    f.render_widget(block, area);

    if inner.width < 20 || inner.height < 6 {
        return;
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(3)])
        .split(inner);

    let (note_text, offset_cents) = match tuner.current_frequency() {
        Some(freq) => match nearest_target(freq) {
            Some((label, target, cents)) => {
                let note = format!(
                    "Note: {}   Detected: {:.2} Hz   Target: {:.2} Hz   Offset: {:+.1} cents",
                    label, freq, target, cents
                );
                (note, Some(cents))
            }
            None => (
                format!("Detected: {:.2} Hz", freq),
                None,
            ),
        },
        None => ("Waiting for audio...".to_string(), None),
    };

    let note_widget = Paragraph::new(note_text)
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::NONE));
    f.render_widget(note_widget, chunks[0]);

    let max_cents = 50.0;
    let needle_x = offset_cents
        .unwrap_or(0.0)
        .clamp(-max_cents, max_cents) as f64;

    let canvas = Canvas::default()
        .x_bounds([-max_cents as f64, max_cents as f64])
        .y_bounds([0.0, 1.0])
        .paint(|ctx| {
            ctx.draw(&CanvasLine {
                x1: -max_cents as f64,
                y1: 0.2,
                x2: max_cents as f64,
                y2: 0.2,
                color: Color::DarkGray,
            });

            ctx.draw(&CanvasLine {
                x1: 0.0,
                y1: 0.15,
                x2: 0.0,
                y2: 0.9,
                color: Color::Gray,
            });

            ctx.draw(&CanvasLine {
                x1: needle_x,
                y1: 0.2,
                x2: needle_x,
                y2: 0.9,
                color: Color::Green,
            });
        });

    f.render_widget(canvas, chunks[1]);
}

fn nearest_target(freq: f32) -> Option<(&'static str, f32, f32)> {
    if !freq.is_finite() || freq <= 0.0 {
        return None;
    }

    let mut best: Option<(&'static str, f32, f32)> = None;
    for (label, target) in TARGETS.iter() {
        let cents = 1200.0 * (freq / *target).log2();
        match best {
            Some((_, _, best_cents)) if cents.abs() >= best_cents.abs() => {}
            _ => best = Some((label, *target, cents)),
        }
    }

    best
}
