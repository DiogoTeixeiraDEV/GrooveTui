use std::collections::HashSet;

use ratatui::{prelude::*, widgets::*};

use crate::app::App;

pub fn render(f: &mut Frame, area: Rect, app: &App) {
    let scale_label = app.first_suggested_scale_label();
    let scale_pitch_classes = app.first_suggested_scale_pitch_classes();
    draw_guitar_neck(
        f,
        area,
        app.root_pitch_class(),
        &scale_pitch_classes,
        &scale_label,
    );
}

fn draw_guitar_neck(
    f: &mut Frame,
    area: Rect,
    root_note: u8,
    scale_notes: &[u8],
    scale_label: &str,
) {
    let tuning: [u8; 6] = [4, 9, 2, 7, 11, 4];
    let fret_count = 12;
    let title = format!(" {} Scale Pattern ", scale_label);
    let container = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .style(Style::default().fg(Color::Gray));
    f.render_widget(container.clone(), area);

    let content_area = container.inner(area);
    let note_width = 8usize;

    let root_style = Style::default().fg(Color::LightYellow).add_modifier(Modifier::BOLD);
    let scale_style = Style::default().fg(Color::LightCyan);
    let empty_style = Style::default().fg(Color::DarkGray);
    let number_style = Style::default().fg(Color::Gray).add_modifier(Modifier::DIM);
    let marker_number_style = Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD);

    let scale_set: HashSet<u8> = scale_notes.iter().copied().collect();

    let mut lines: Vec<Line> = Vec::new();

    for &open_note in tuning.iter().rev() {
        let mut spans: Vec<Span> = Vec::new();
        spans.push(Span::raw(" "));
        spans.push(Span::raw("|"));

        for fret in 0..=fret_count {
            let current_note = (open_note + fret as u8) % 12;

            if scale_set.contains(&current_note) {
                let is_root = current_note == root_note;
                let symbol = if is_root { "R" } else { "●" };
                let style = if is_root { root_style } else { scale_style };
                spans.push(Span::styled(centered_cell(symbol, note_width), style));
            } else {
                spans.push(Span::styled("─".repeat(note_width), empty_style));
            };

            spans.push(Span::raw("|"));
        }

        lines.push(Line::from(spans));
    }

    let mut number_spans: Vec<Span> = Vec::new();
    number_spans.push(Span::raw(" "));
    number_spans.push(Span::raw(" ".repeat(1)));
    for fret in 0..=fret_count {
        let label = centered_cell(&fret.to_string(), note_width);
        let style = match fret {
            3 | 5 | 7 | 9 | 12 => marker_number_style,
            _ => number_style,
        };
        number_spans.push(Span::styled(label, style));
        number_spans.push(Span::raw(" "));
    }
    lines.push(Line::from(number_spans));

    let max_width = lines
        .iter()
        .map(|line| line.width())
        .max()
        .unwrap_or(0) as u16;
    let pad = content_area.width.saturating_sub(max_width) / 2;
    let padded_lines: Vec<Line> = lines
        .into_iter()
        .map(|line| {
            if pad == 0 {
                line
            } else {
                let mut spans = Vec::with_capacity(line.spans.len() + 1);
                spans.push(Span::raw(" ".repeat(pad as usize)));
                spans.extend(line.spans);
                Line::from(spans)
            }
        })
        .collect();

    let top_padding_lines = 3;
    let mut lines_with_padding = Vec::with_capacity(padded_lines.len() + top_padding_lines);
    for _ in 0..top_padding_lines {
        lines_with_padding.push(Line::from(""));
    }
    lines_with_padding.extend(padded_lines);

    let fretboard = Paragraph::new(lines_with_padding).style(Style::default().fg(Color::White));
    f.render_widget(fretboard, content_area);
}

fn centered_cell(text: &str, width: usize) -> String {
    if width == 0 {
        return String::new();
    }
    let text_len = text.chars().count();
    if text_len >= width {
        return text.chars().take(width).collect();
    }
    let padding = width - text_len;
    let left = padding / 2;
    let right = padding - left;
    format!("{}{}{}", " ".repeat(left), text, " ".repeat(right))
}
