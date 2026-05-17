use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use std::time::Duration;

use crate::app::{App, AppTab};
use crate::tui::state::TuningMode;

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
                    KeyCode::Tab => app.next_tab(),
                    _ => match app.current_tab() {
                        AppTab::Groove => match key.code {
                            KeyCode::Char(' ') => app.toggle_play(),
                            KeyCode::Up => app.increase_bpm(),
                            KeyCode::Down => app.decrease_bpm(),
                            KeyCode::Left => app.prev_root_pitch(),
                            KeyCode::Right => app.next_root_pitch(),
                            KeyCode::Char('.') => app.prev_chord_quality(),
                            KeyCode::Char(',') => app.next_chord_quality(),
                            KeyCode::Char('g') | KeyCode::Char('G') => app.prev_genre(),
                            KeyCode::Char('h') | KeyCode::Char('H') => app.next_genre(),
                            _ => {}
                        },
                        AppTab::Tuner => match key.code {
                            KeyCode::Char(' ') => app.toggle_tuner_capture(),
                            KeyCode::Left => app.prev_tuner_device(),
                            KeyCode::Right => app.next_tuner_device(),
                            KeyCode::Up => app.increase_tuner_gain(),
                            KeyCode::Down => app.decrease_tuner_gain(),
                            KeyCode::Char('m') | KeyCode::Char('M') => {
                                app.toggle_tuner_mode();
                            }
                            KeyCode::Char('a') => {
                                if app.tuner_mode() == TuningMode::Manual {
                                    app.prev_tuner_string();
                                }
                            }
                            KeyCode::Char('d') => {
                                if app.tuner_mode() == TuningMode::Manual {
                                    app.next_tuner_string();
                                }
                            }
                            _ => {}
                        },
                        AppTab::Backing => {
                            if app.backing_tracks().editing_query() {
                                match key.code {
                                    KeyCode::Enter => app.backing_submit_search(),
                                    KeyCode::Esc => app.backing_cancel_query_edit(),
                                    KeyCode::Backspace => app.backing_backspace_query(),
                                    KeyCode::Char(c) => app.backing_push_query_char(c),
                                    _ => {}
                                }
                            } else {
                                match key.code {
                                    KeyCode::Char('/') => app.backing_begin_search_edit(),
                                    KeyCode::Enter => app.backing_submit_search(),
                                    KeyCode::Up => app.backing_prev_result(),
                                    KeyCode::Down => app.backing_next_result(),
                                    KeyCode::Char(' ') => app.backing_toggle_selected(),
                                    KeyCode::Char('s') | KeyCode::Char('S') => app.backing_stop(),
                                    _ => {}
                                }
                            }
                        }
                    },
                }
            }
        }
    }

    Ok(InputAction::Continue)
}
