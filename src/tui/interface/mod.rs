use ratatui::{prelude::*, widgets::*};

use crate::app::{App, AppTab};

pub mod fretboard;
pub mod harmony;
pub mod metronome;
pub mod tuner;

pub fn ui(f: &mut Frame, app: &App) {
    let size = f.size();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Min(12),
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

    let tab_titles = vec!["Groove", "Tuner"]
        .into_iter()
        .map(Line::from)
        .collect::<Vec<_>>();
    let tabs = Tabs::new(tab_titles)
        .select(app.current_tab_index())
        .highlight_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .style(Style::default().fg(Color::Gray))
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(tabs, chunks[1]);

    match app.current_tab() {
        AppTab::Groove => {
            let content_layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(5), Constraint::Min(8)])
                .split(chunks[2]);

            let main_layout = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                .split(content_layout[0]);

            metronome::render(f, main_layout[0], app);
            harmony::render(f, main_layout[1], app);
            fretboard::render(f, content_layout[1], app);
        }
        AppTab::Tuner => {
            tuner::render(f, chunks[2], app);
        }
    }

    let help_text = match app.current_tab() {
        AppTab::Groove => {
            " [Tab] Switch | [Space] Play/Pause | [Q] Quit | [↑/↓] BPM | [←/→] Root | [< / >] Major/Minor | [G] Genre "
        }
        AppTab::Tuner => " [Tab] Switch | [Q] Quit ",
    };
    let footer = Paragraph::new(help_text)
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::NONE));
    f.render_widget(footer, chunks[3]);
}
