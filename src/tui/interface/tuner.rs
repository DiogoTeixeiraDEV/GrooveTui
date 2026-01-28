use ratatui::{prelude::*, widgets::*};

use crate::app::App;

pub fn render(f: &mut Frame, area: Rect, _app: &App) {
    let container = Block::default().title(" Tuner ").borders(Borders::ALL);
    f.render_widget(container, area);
}
