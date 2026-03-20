use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem},
    Frame,
};

use crate::app::{App, NodeState};

pub fn render(frame: &mut Frame, app: &App, area: Rect) {
    let entries = match app.tree.get(&app.current_path) {
        Some(NodeState::Loaded(entries)) => entries
            .iter()
            .enumerate()
            .map(|(i, e)| {
                let prefix = if app.selected.contains(&e.path) { "● " } else { "  " };
                let label = format!("{}{}", prefix, e.name);
                let style = if i == app.cursor {
                    Style::default().fg(Color::Black).bg(Color::Cyan).add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };
                ListItem::new(label).style(style)
            })
            .collect::<Vec<_>>(),
        Some(NodeState::Loading) => vec![ListItem::new("  Loading...")],
        _ => vec![ListItem::new("  (empty)")],
    };

    let block = Block::default()
        .title(format!(" /{} ", app.current_path))
        .borders(Borders::ALL);

    let list = List::new(entries).block(block);
    frame.render_widget(list, area);
}
