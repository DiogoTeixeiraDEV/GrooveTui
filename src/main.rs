mod app;
mod audio;
mod entities;
mod tui;

use anyhow::Result;
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::prelude::*;
use std::{io, sync::mpsc, thread};

use app::App;
use tui::{
    input::{handle_input, InputAction},
    interface::ui,
};

fn main() -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let (tx, rx) = mpsc::channel();
    let audio_thread = thread::spawn(move || audio::run_audio_thread(rx));

    let mut app = App::new(tx);

    loop {
        terminal.draw(|f| ui(f, &app))?;

        if matches!(handle_input(&mut app)?, InputAction::Quit) {
            break;
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    let _ = audio_thread.join();
    Ok(())
}
