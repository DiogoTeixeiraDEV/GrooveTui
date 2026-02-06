use ratatui::{prelude::*, widgets::*};

use crate::app::App;

pub fn render(f: &mut Frame, area: Rect, app: &App) {
    let suggested_scales = app.suggested_scales();
    let suggested_scales_text = if suggested_scales.is_empty() {
        "  (No scale suggestions)".to_string()
    } else {
        suggested_scales
            .iter()
            .map(|scale| format!("  - {}", scale))
            .collect::<Vec<_>>()
            .join("\n")
    };

    let harmony_text = format!(
        "\n  Root Chord: {} {}\n  Chord Notes: {}\n  Genre: {}\n\n  Suggested Scales:\n{}",
        app.root_pitch_label(),
        app.chord_quality_label(),
        app.chord_notes_label(),
        app.genre(),
        suggested_scales_text
    );
    let harmony_block = Paragraph::new(harmony_text).block(
        Block::default()
            .title(" Harmony Suggestions ")
            .borders(Borders::ALL),
    );
    f.render_widget(harmony_block, area);
}
