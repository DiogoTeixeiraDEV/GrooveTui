use ratatui::{prelude::*, widgets::*};

use crate::app::App;

pub fn render(f: &mut Frame, area: Rect, app: &App) {
    let tuner = app.tuner();

    // Main container frame
    let container = Block::default()
        .title(" Tuner ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded);
    // Inner layout
    let inner_area = container.inner(area);
    f.render_widget(container, area);
    
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Device Selector
            Constraint::Length(3), // Status info
            Constraint::Min(10),   // Volume Meter and Visualization
        ])
        .margin(1)
        .split(inner_area);

    // --- Device Selector ---
    let device_name = tuner.selected_device().unwrap_or("No devices found");
    // Center the device name with arrows to indicate it's selectable
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

    // --- Status ---
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
    
    // --- Volume Meter ---
    // Split the middle section
    let meter_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(10),
            Constraint::Percentage(80),
            Constraint::Percentage(10),
        ])
        .split(chunks[2]);
        
    let level = tuner.current_level();
    let peak = tuner.peak_level();
    
    let gauge_color = if peak > 0.9 {
        Color::Red
    } else if peak > 0.7 {
        Color::Yellow
    } else {
        Color::Green
    };

    let label = format!("{:.1}%", level * 100.0);
    let gauge = Gauge::default()
        .block(Block::default().title(" Input Level ").borders(Borders::ALL))
        .gauge_style(Style::default().fg(gauge_color))
        .ratio(level.clamp(0.0, 1.0) as f64)
        .label(label);
        
    f.render_widget(gauge, meter_chunks[1]);

    // --- Error Overlay ---
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
