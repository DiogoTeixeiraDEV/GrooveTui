use ratatui::{prelude::*, widgets::*};

use crate::tui::tuner_state::TunerState;

pub fn render(f: &mut Frame, area: Rect, tuner: &TunerState) {
    let freq_text = tuner
        .current_frequency()
        .map(|f| format!("{:.2} Hz", f))
        .unwrap_or_else(|| "-- Hz".to_string());

    let clarity_text = tuner
        .current_clarity()
        .map(|c| format!("{:.0}%", c * 100.0))
        .unwrap_or_else(|| "--%".to_string());

    let note_text = tuner
        .current_note_label()
        .unwrap_or_else(|| "--".to_string());

    let text = Line::from(vec![
        Span::raw("Frequency: "),
        Span::styled(freq_text, Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
        Span::raw("   Note: "),
        Span::styled(note_text, Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        Span::raw("   Clarity: "),
        Span::styled(clarity_text, Style::default().fg(Color::Gray)),
    ]);

    let widget = Paragraph::new(text)
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL).title(" Pitch "));

    f.render_widget(widget, area);
}
