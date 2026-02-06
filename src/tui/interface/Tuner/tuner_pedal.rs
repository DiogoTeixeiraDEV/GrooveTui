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
        .constraints([Constraint::Length(2), Constraint::Min(3)])
        .split(inner);

    let (note_text, details_text, offset_cents) = match tuner.current_frequency() {
        Some(freq) => match nearest_target(freq) {
            Some((label, target, cents)) => (
                note_display(label),
                format!(
                    "Detected: {:.2} Hz   Target: {:.2} Hz   Offset: {:+.1} cents",
                    freq, target, cents
                ),
                Some(cents),
            ),
            None => ("".to_string(), format!("Detected: {:.2} Hz", freq), None),
        },
        None => ("".to_string(), "Waiting for audio...".to_string(), None),
    };

    let note_color = tuning_color(offset_cents);
    let note_style = Style::default().fg(note_color).add_modifier(Modifier::BOLD);
    let details_style = Style::default().fg(Color::White);

    let note_widget = Paragraph::new(vec![
        Line::from(Span::styled(details_text, details_style)),
        Line::from(Span::styled(note_text, note_style)),
    ])
    .alignment(Alignment::Center)
    .block(Block::default().borders(Borders::NONE));
    f.render_widget(note_widget, chunks[0]);

    let max_cents = 100.0;
    let needle_x = offset_cents
        .unwrap_or(0.0)
        .clamp(-max_cents, max_cents) as f64;

    let needle_color = tuning_color(offset_cents);
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
                color: needle_color,
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

fn tuning_color(offset_cents: Option<f32>) -> Color {
    match offset_cents {
        Some(cents) => {
            let distance = cents.abs();
            if distance <= 10.0 {
                Color::Green
            } else if distance <= 25.0 {
                Color::Yellow
            } else {
                Color::Red
            }
        }
        None => Color::Gray,
    }
}

fn note_display(label: &str) -> String {
    let trimmed: String = label.chars().take_while(|c| !c.is_ascii_digit()).collect();
    if trimmed.is_empty() {
        label.to_string()
    } else {
        trimmed
    }
}
