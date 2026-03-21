use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
};

use crate::app::App;

pub fn render(frame: &mut Frame, app: &App, area: Rect) {
    let popup_area = centered_rect(50, 5, area);

    let content = Line::from(vec![
        Span::raw(&app.input_buffer),
        Span::styled("█", Style::default().fg(Color::Cyan)),
    ]);

    let widget = Paragraph::new(content)
        .block(
            Block::default()
                .title(" Open repository  (owner/repo[@branch]) ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan))
                .style(Style::default().add_modifier(Modifier::BOLD)),
        )
        .style(Style::default().fg(Color::White));

    frame.render_widget(Clear, popup_area);
    frame.render_widget(widget, popup_area);
}

fn centered_rect(percent_x: u16, height: u16, area: Rect) -> Rect {
    let width = area.width * percent_x / 100;
    let x = (area.width.saturating_sub(width)) / 2;
    let y = (area.height.saturating_sub(height)) / 2;
    Rect::new(x, y, width, height)
}
