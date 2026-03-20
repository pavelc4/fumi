use ratatui::{
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

use crate::app::App;

pub fn render(frame: &mut Frame, app: &App, area: Rect) {
    let content = app
        .preview
        .as_deref()
        .unwrap_or("No preview — press p on a file");

    let widget = Paragraph::new(content)
        .block(Block::default().title(" Preview ").borders(Borders::ALL))
        .style(Style::default().fg(Color::White))
        .wrap(Wrap { trim: false });

    frame.render_widget(widget, area);
}
