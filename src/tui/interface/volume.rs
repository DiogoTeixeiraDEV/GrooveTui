use ratatui::{prelude::*, widgets::*};

use crate::tui::tuner_state::TunerState;

pub fn render(f: &mut Frame, area: Rect, tuner: &TunerState) {
    let level = tuner.current_level().clamp(0.0, 1.0);
    let peak = tuner.peak_level();

    let gauge_color = if peak > 0.9 {
        Color::Red
    } else if peak > 0.7 {
        Color::Yellow
    } else {
        Color::Green
    };

    let label = format!("{:.0}%", level * 100.0);
    let gauge = Gauge::default()
        .block(Block::default().title(" Level ").borders(Borders::ALL))
        .gauge_style(Style::default().fg(gauge_color))
        .ratio(level as f64)
        .label(label);

    f.render_widget(gauge, area);
}
