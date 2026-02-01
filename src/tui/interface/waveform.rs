use ratatui::{
    prelude::*,
    symbols,
    widgets::{canvas::Canvas, Block, Borders, Paragraph},
    widgets::canvas::Line as CanvasLine,
};

use crate::tui::tuner_state::TunerState;

pub fn render(f: &mut Frame, area: Rect, tuner: &TunerState) {
    let samples = tuner.waveform_samples();

    let container = Block::default().title(" Waveform ").borders(Borders::ALL);
    f.render_widget(container.clone(), area);

    let inner = container.inner(area);
    if inner.width < 10 || inner.height < 6 {
        return;
    }

    if samples.len() < 2 {
        let placeholder = Paragraph::new("Waiting for audio...")
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::DarkGray));
        f.render_widget(placeholder, inner);
        return;
    }

    let max_x = (samples.len() - 1) as f64;

    let canvas = Canvas::default()
        .x_bounds([0.0, max_x])
        .y_bounds([-1.0, 1.0])
        .marker(symbols::Marker::Braille)
        .paint(|ctx| {
            let mut prev_x = 0.0;
            let mut prev_y = samples[0] as f64;

            for (i, &sample) in samples.iter().enumerate().skip(1) {
                let x = i as f64;
                let y = sample as f64;
                ctx.draw(&CanvasLine {
                    x1: prev_x,
                    y1: prev_y,
                    x2: x,
                    y2: y,
                    color: Color::Cyan,
                });
                prev_x = x;
                prev_y = y;
            }

            ctx.draw(&CanvasLine {
                x1: 0.0,
                y1: 0.0,
                x2: max_x,
                y2: 0.0,
                color: Color::DarkGray,
            });
        });

    f.render_widget(canvas, inner);
}
