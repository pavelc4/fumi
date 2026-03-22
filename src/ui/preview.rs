use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Paragraph, Wrap},
};

use crate::app::{ActivePanel, App};

pub fn render(frame: &mut Frame, app: &App, area: Rect) {
    let is_active = matches!(app.active_panel, ActivePanel::Preview);

    let (content, is_placeholder) = match app.preview.as_deref() {
        Some(c) => (c, false),
        None => (
            "  No preview\n\n  Navigate to a file and press p or l to preview.",
            true,
        ),
    };

    let border_style = if is_active {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let title_style = if is_active {
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let widget = Paragraph::new(content)
        .block(
            Block::default()
                .title(" Preview ")
                .title_style(title_style)
                .borders(Borders::ALL)
                .border_style(border_style),
        )
        .style(if is_placeholder {
            Style::default().fg(Color::DarkGray)
        } else {
            Style::default().fg(Color::White)
        })
        .scroll((app.preview_scroll, 0))
        .wrap(Wrap { trim: false });

    frame.render_widget(widget, area);
}
