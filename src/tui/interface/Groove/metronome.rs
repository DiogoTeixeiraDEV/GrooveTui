use ratatui::{
    prelude::*,
    widgets::canvas::{Canvas, Circle, Line},
    widgets::*,
};

use crate::app::App;

pub fn render(f: &mut Frame, area: Rect, app: &App) {
    let play_status = if app.is_playing() { "ON" } else { "OFF" };
    let block = Block::default()
        .title(" Metronome ")
        .borders(Borders::ALL)
        .style(Style::default().fg(if app.is_playing() {
            Color::Green
        } else {
            Color::White
        }));
    f.render_widget(block.clone(), area);

    let inner = block.inner(area);
    if inner.width < 12 || inner.height < 8 {
        return;
    }

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(2), Constraint::Min(10)])
        .split(inner);

    let info = Paragraph::new(format!(
        "BPM: {}   Status: {}   Use ↑/↓ to adjust tempo",
        app.bpm(),
        play_status
    ))
    .alignment(Alignment::Center);
    f.render_widget(info, layout[0]);

    let phase = app.metronome_phase().clamp(0.0, 1.0) as f64;
    let angle = (phase - 0.5) * 2.0 * 0.75;

    let canvas_area = layout[1];
    let aspect = canvas_area.width as f64 / canvas_area.height as f64;
    let x_max = 100.0 * aspect;
    let canvas = Canvas::default()
        .x_bounds([0.0, x_max])
        .y_bounds([0.0, 100.0])
        .marker(symbols::Marker::Braille)
        .paint(|ctx| {
            let origin_x = x_max / 2.0;
            let origin_y = 10.0;
            let rod_length = 55.0;

            let bob_x = origin_x + rod_length * angle.sin();
            let bob_y = origin_y + rod_length * angle.cos();

            ctx.draw(&Line {
                x1: origin_x,
                y1: origin_y,
                x2: origin_x,
                y2: origin_y + rod_length,
                color: Color::DarkGray,
            });

            ctx.draw(&Line {
                x1: origin_x,
                y1: origin_y,
                x2: bob_x,
                y2: bob_y,
                color: Color::White,
            });

            ctx.draw(&Circle {
                x: bob_x,
                y: bob_y,
                radius: 2.8,
                color: if app.metronome_flash() {
                    Color::Yellow
                } else {
                    Color::LightRed
                },
            });

            ctx.draw(&Circle {
                x: origin_x,
                y: origin_y,
                radius: 2.0,
                color: if app.is_playing() {
                    Color::Green
                } else {
                    Color::DarkGray
                },
            });

            let pivot_color = if app.is_playing() {
                Color::Green
            } else {
                Color::DarkGray
            };
            ctx.draw(&Line {
                x1: origin_x - 8.0,
                y1: origin_y,
                x2: origin_x + 8.0,
                y2: origin_y,
                color: pivot_color,
            });
        });

    f.render_widget(canvas, layout[1]);
}
