use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
};

use crate::app::{App, AppMode, DownloadState};

pub fn render(frame: &mut Frame, app: &App, area: Rect) {
    let (mode_span, hint) = match &app.mode {
        AppMode::Browse => (
            Span::styled(
                " BROWSE ",
                Style::default()
                    .bg(Color::Blue)
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
            "  j/k:move  l:open  h:back  space:select  d:dl  p:preview  b:repo  q:quit",
        ),
        AppMode::Previewing => (
            Span::styled(
                " PREVIEW ",
                Style::default()
                    .bg(Color::Cyan)
                    .fg(Color::Black)
                    .add_modifier(Modifier::BOLD),
            ),
            "  j/k:scroll  h/q:close",
        ),
        AppMode::Input => (
            Span::styled(
                " INPUT ",
                Style::default()
                    .bg(Color::Yellow)
                    .fg(Color::Black)
                    .add_modifier(Modifier::BOLD),
            ),
            "  type owner/repo[@branch]  enter:open  esc:cancel",
        ),
        AppMode::Downloading => (
            Span::styled(
                " DL ",
                Style::default()
                    .bg(Color::Green)
                    .fg(Color::Black)
                    .add_modifier(Modifier::BOLD),
            ),
            "  q:quit",
        ),
        AppMode::Error(_) => (
            Span::styled(
                " ERROR ",
                Style::default()
                    .bg(Color::Red)
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
            "  enter/q:dismiss",
        ),
    };

    let extra = if matches!(app.mode, AppMode::Downloading) {
        let total = app.downloads.len();
        let done = app
            .downloads
            .values()
            .filter(|s| matches!(s, DownloadState::Done))
            .count();
        let errors = app
            .downloads
            .values()
            .filter(|s| matches!(s, DownloadState::Error(_)))
            .count();
        let in_progress = app
            .downloads
            .values()
            .filter(|s| matches!(s, DownloadState::Downloading { .. }))
            .count();

        // aggregate bytes
        let (dl_bytes, total_bytes) = app.downloads.values().fold((0u64, 0u64), |(d, t), s| {
            if let DownloadState::Downloading { downloaded, total } = s {
                (d + downloaded, t + total)
            } else {
                (d, t)
            }
        });

        let progress_str = if total_bytes > 0 {
            format!(
                "  {}/{} files  {} downloading  {} err  ({}/{}KB)",
                done,
                total,
                in_progress,
                errors,
                dl_bytes / 1024,
                total_bytes / 1024
            )
        } else {
            format!(
                "  {}/{} files  {} downloading  {} err",
                done, total, in_progress, errors
            )
        };
        progress_str
    } else {
        String::new()
    };

    let line = Line::from(vec![
        mode_span,
        Span::styled(
            format!("{}{}", hint, extra),
            Style::default().bg(Color::DarkGray).fg(Color::White),
        ),
    ]);

    let widget = Paragraph::new(line).style(Style::default().bg(Color::DarkGray));

    frame.render_widget(widget, area);
}
