use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use std::time::Duration;

use crate::app::App;

pub enum InputAction {
    Continue,
    Quit,
}

pub fn handle_input(app: &mut App) -> Result<InputAction> {
    if event::poll(Duration::from_millis(16))? {
        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                match key.code {
                    KeyCode::Char('q') => {
                        app.quit_audio();
                        return Ok(InputAction::Quit);
                    }
                    KeyCode::Char(' ') => app.toggle_play(),
                    KeyCode::Up => app.increase_bpm(),
                    KeyCode::Down => app.decrease_bpm(),
                    KeyCode::Left => app.prev_root_pitch(),
                    KeyCode::Right => app.next_root_pitch(),
                    KeyCode::Char('[') => app.prev_chord_quality(),
                    KeyCode::Char(']') => app.next_chord_quality(),
                    KeyCode::Char('g') | KeyCode::Char('G') => app.prev_genre(),
                    KeyCode::Char('h') | KeyCode::Char('H') => app.next_genre(),
                    _ => {}
                }
            }
        }
    }

    Ok(InputAction::Continue)
}
