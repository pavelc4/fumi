use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style},
    text::Span,
    widgets::Paragraph,
};

use crate::app::{App, AppMode};

pub fn render(frame: &mut Frame, app: &App, area: Rect) {
    let mode_str = match &app.mode {
        AppMode::Browse => " BROWSE  ".to_string(),
        AppMode::Input => " INPUT   ".to_string(),
        AppMode::Downloading => " DL      ".to_string(),
        AppMode::Previewing => " PREVIEW ".to_string(),
        AppMode::Error(msg) => format!(" ERROR: {} ", msg),
    };

    let hint = "  j/k:move  l:open  h:back  space:select  d:dl  p:preview  q:quit";
    let text = format!("{}{}", mode_str, hint);

    let widget = Paragraph::new(Span::raw(text))
        .style(Style::default().bg(Color::DarkGray).fg(Color::White));

    frame.render_widget(widget, area);
}
