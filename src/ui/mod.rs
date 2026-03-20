use anyhow::Result;
use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout},
    Terminal,
};

use crate::app::App;

pub mod filetree;
pub mod preview;
pub mod statusbar;

pub fn draw<B: Backend>(terminal: &mut Terminal<B>, app: &App) -> Result<()>
where
    <B as Backend>::Error: Send + Sync + 'static,
{
    terminal.draw(|frame| {
        let area = frame.area();

        let rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(0), Constraint::Length(1)])
            .split(area);

        let cols = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
            .split(rows[0]);

        filetree::render(frame, app, cols[0]);
        preview::render(frame, app, cols[1]);
        statusbar::render(frame, app, rows[1]);
    })?;
    Ok(())
}
