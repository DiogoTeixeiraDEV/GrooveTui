use ratatui::{prelude::*, widgets::*};

use crate::app::App;
use super::{tuner_pedal, volume};

pub fn render(f: &mut Frame, area: Rect, app: &App) {
    let tuner = app.tuner();

    
    let container = Block::default()
        .title(" Tuner ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded);
    
    let inner_area = container.inner(area);
    f.render_widget(container, area);
    
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), 
            Constraint::Length(3), 
            Constraint::Min(8),    
            Constraint::Length(3), 
        ])
        .margin(1)
        .split(inner_area);

    
    let device_name = tuner.selected_device().unwrap_or("No devices found");
    
    let device_text = Line::from(vec![
        Span::raw("Input Device: "),
        Span::styled(" < ", Style::default().fg(Color::DarkGray)),
        Span::styled(device_name, Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        Span::styled(" > ", Style::default().fg(Color::DarkGray)),
    ]);
    
    let device_widget = Paragraph::new(device_text)
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::BOTTOM).border_style(Style::default().fg(Color::DarkGray)));
    f.render_widget(device_widget, chunks[0]);

    
    let (status_text, status_style) = if tuner.is_capturing() {
        ("CAPTURING", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))
    } else {
        ("STOPPED", Style::default().fg(Color::Red))
    };

    let status_widget = Paragraph::new(Line::from(vec![
        Span::raw("Status: "),
        Span::styled(status_text, status_style),
        Span::raw("  (Press [Space] to toggle)"),
    ]))
    .alignment(Alignment::Center);
    f.render_widget(status_widget, chunks[1]);

    
    tuner_pedal::render(f, chunks[2], tuner);
    
    
    let meter_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(20),
            Constraint::Percentage(60),
            Constraint::Percentage(20),
        ])
        .split(chunks[3]);

    volume::render(f, meter_chunks[1], tuner);

    
    if let Some(err) = tuner.error_message() {
        let error_area = Rect::new(area.x + 2, area.y + area.height - 4, area.width - 4, 3);
        let error_widget = Paragraph::new(err)
            .style(Style::default().fg(Color::White).bg(Color::Red).add_modifier(Modifier::BOLD))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(Color::White)));
        
        f.render_widget(Clear, error_area);
        f.render_widget(error_widget, error_area);
    }
}
