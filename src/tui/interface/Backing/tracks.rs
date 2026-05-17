use ratatui::{prelude::*, widgets::*};

use crate::app::{App, BackingPlayerState, BackingSearchState};

pub fn render(f: &mut Frame, area: Rect, app: &App) {
    let backing = app.backing_tracks();
    let container = Block::default()
        .title(" Backing Tracks ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded);
    let inner = container.inner(area);
    f.render_widget(container, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(2),
            Constraint::Min(6),
            Constraint::Length(5),
        ])
        .split(inner);

    let query_style = if backing.editing_query() {
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::Gray)
    };
    let query = if backing.query().is_empty() {
        "press / to search backing tracks"
    } else {
        backing.query()
    };
    let search = Paragraph::new(Line::from(vec![
        Span::raw("Search: "),
        Span::styled(query, query_style),
        Span::styled(
            if backing.editing_query() { " _" } else { "" },
            Style::default().fg(Color::Yellow),
        ),
    ]))
    .block(Block::default().borders(Borders::BOTTOM))
    .alignment(Alignment::Left);
    f.render_widget(search, chunks[0]);

    let status = status_line(app);
    f.render_widget(
        Paragraph::new(status).alignment(Alignment::Center),
        chunks[1],
    );

    let items = backing
        .results()
        .iter()
        .enumerate()
        .map(|(index, track)| {
            let marker = if index == backing.selected_index() {
                "> "
            } else {
                "  "
            };
            let duration = track
                .duration_label()
                .unwrap_or_else(|| "--:--".to_string());
            let channel = track.channel().unwrap_or("Unknown channel");
            ListItem::new(Line::from(vec![
                Span::styled(marker, Style::default().fg(Color::Cyan)),
                Span::styled(track.title(), Style::default().fg(Color::White)),
                Span::raw("  "),
                Span::styled(duration, Style::default().fg(Color::DarkGray)),
                Span::raw("  "),
                Span::styled(channel, Style::default().fg(Color::Gray)),
            ]))
        })
        .collect::<Vec<_>>();

    if items.is_empty() {
        let help = match backing.search_state() {
            BackingSearchState::Idle => "Search for things like: funk backing track E minor 90 bpm",
            BackingSearchState::Searching => "Searching YouTube...",
            BackingSearchState::Ready => "No results found. Try a more specific query.",
            BackingSearchState::Failed(_) => "Search failed. Check the status message above.",
        };
        let empty_results = Paragraph::new(help)
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::DarkGray))
            .block(Block::default().borders(Borders::ALL).title(" Results "));
        f.render_widget(empty_results, chunks[2]);
    } else {
        let result_list = List::new(items)
            .block(Block::default().borders(Borders::ALL).title(" Results "))
            .highlight_style(Style::default().bg(Color::DarkGray));
        f.render_widget(result_list, chunks[2]);
    }

    let player_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(2), Constraint::Length(3)])
        .split(chunks[3]);

    let transport_style = match backing.player_state() {
        BackingPlayerState::Playing => Style::default().fg(Color::Green),
        BackingPlayerState::Paused => Style::default().fg(Color::Yellow),
        BackingPlayerState::Stopped => Style::default().fg(Color::Gray),
        BackingPlayerState::Failed(_) => Style::default().fg(Color::Red),
    };
    f.render_widget(
        Paragraph::new(player_status_line(app))
            .style(transport_style)
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::TOP)),
        player_chunks[0],
    );

    let progress = backing.progress();
    let label = format!(
        "{} / {}",
        progress.elapsed_label(),
        progress.duration_label()
    );
    let gauge = Gauge::default()
        .block(Block::default().borders(Borders::NONE))
        .gauge_style(Style::default().fg(Color::Cyan))
        .label(label)
        .ratio(progress.fraction());
    f.render_widget(
        gauge,
        Rect {
            x: player_chunks[1].x,
            y: player_chunks[1].y + 1,
            width: player_chunks[1].width,
            height: 1,
        },
    );
}

fn player_status_line(app: &App) -> String {
    let backing = app.backing_tracks();
    match backing.player_state() {
        BackingPlayerState::Playing => backing
            .now_playing()
            .map(|track| format!("▶ {}", track.title()))
            .unwrap_or_else(|| "▶ Playing".to_string()),
        BackingPlayerState::Paused => backing
            .now_playing()
            .map(|track| format!("⏸ {}", track.title()))
            .unwrap_or_else(|| "⏸ Paused".to_string()),
        BackingPlayerState::Stopped => "▶ Nothing playing".to_string(),
        BackingPlayerState::Failed(message) => message.clone(),
    }
}

fn status_line(app: &App) -> Line<'static> {
    let backing = app.backing_tracks();
    match backing.search_state() {
        BackingSearchState::Idle => {
            Line::from("Type a search, press Enter, pick a track, then press Space.")
        }
        BackingSearchState::Searching => Line::from(vec![
            Span::styled("Searching ", Style::default().fg(Color::Yellow)),
            Span::raw("with yt-dlp"),
        ]),
        BackingSearchState::Ready => Line::from(format!("{} result(s)", backing.results().len())),
        BackingSearchState::Failed(message) => Line::from(Span::styled(
            message.clone(),
            Style::default().fg(Color::Red),
        )),
    }
}
