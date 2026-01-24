use ratatui::{prelude::*, widgets::*};

use crate::app::App;

pub fn ui(f: &mut Frame, app: &App) {
    let size = f.size();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(5),
            Constraint::Length(3),
        ])
        .split(size);

    let header = Paragraph::new(" GUITUI: O Companheiro do Guitarrista Rustáceo ")
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

    let play_status = if app.is_playing() { "ON" } else { "OFF" };
    let metronome_text = format!(
        "\n  BPM: {}\n  Status: {}\n\n  Setas UP/DOWN ajustam tempo",
        app.bpm(),
        play_status
    );
    let metronome_block = Paragraph::new(metronome_text).block(
        Block::default()
            .title(" Metrônomo ")
            .borders(Borders::ALL)
            .style(Style::default().fg(if app.is_playing() {
                Color::Green
            } else {
                Color::White
            })),
    );
    f.render_widget(metronome_block, main_layout[0]);

    let harmony_text = format!(
        "\n  Nota Dominante: {}\n  Gênero: {}\n\n  (Lógica de acordes virá aqui)",
        app.root_note(),
        app.genre()
    );
    let harmony_block = Paragraph::new(harmony_text).block(
        Block::default()
            .title(" Sugestão de Harmonia ")
            .borders(Borders::ALL),
    );
    f.render_widget(harmony_block, main_layout[1]);

    let help_text = " [Space] Play/Pause | [Q] Sair | [↑/↓] BPM ";
    let footer = Paragraph::new(help_text)
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::NONE));
    f.render_widget(footer, chunks[2]);
}
