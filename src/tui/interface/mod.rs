use ratatui::{prelude::*, widgets::*};

use crate::app::App;

pub mod fretboard;
pub mod harmony;
pub mod metronome;

pub fn ui(f: &mut Frame, app: &App) {
    let size = f.size();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(5),
            Constraint::Min(8),
            Constraint::Length(3),
        ])
        .split(size);

    let header = Paragraph::new("GrooveTui")
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .style(Style::default().fg(Color::Cyan)),
        );
    f.render_widget(header, chunks[0]);

    let main_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(chunks[1]);

    metronome::render(f, main_layout[0], app);
    harmony::render(f, main_layout[1], app);
    fretboard::render(f, chunks[2], app);

    let help_text =
        " [Space] Play/Pause | [Q] Quit | [↑/↓] BPM | [←/→] Root | [< / >] Major/Minor | [G] Genre ";
    let footer = Paragraph::new(help_text)
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::NONE));
    f.render_widget(footer, chunks[3]);
}
