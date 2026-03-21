use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem},
};

use crate::app::{App, NodeState};
use crate::github::EntryType;

pub fn render(frame: &mut Frame, app: &App, area: Rect) {
    let has_parent = !app.current_path.is_empty();

    let entries = match app.tree.get(&app.current_path) {
        Some(NodeState::Loaded(entries)) => {
            let mut items: Vec<ListItem> = Vec::new();

            if has_parent {
                let style = if app.cursor == 0 {
                    Style::default()
                        .fg(Color::Black)
                        .bg(Color::Cyan)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::DarkGray)
                };
                items.push(ListItem::new("  ↑ ..").style(style));
            }

            for (i, e) in entries.iter().enumerate() {
                let cursor_idx = if has_parent { i + 1 } else { i };
                let prefix = if app.selected.contains(&e.path) {
                    "◆ "
                } else {
                    "  "
                };
                let icon = match e.entry_type {
                    EntryType::Dir => "▸ ",
                    EntryType::File => "  ",
                };
                let label = format!("{}{}{}", prefix, icon, e.name);
                let style = if cursor_idx == app.cursor {
                    Style::default()
                        .fg(Color::Black)
                        .bg(Color::Cyan)
                        .add_modifier(Modifier::BOLD)
                } else if matches!(e.entry_type, EntryType::Dir) {
                    Style::default()
                        .fg(Color::Blue)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };
                items.push(ListItem::new(label).style(style));
            }

            items
        }
        Some(NodeState::Loading) => {
            vec![ListItem::new("  Loading...").style(Style::default().fg(Color::DarkGray))]
        }
        _ => vec![ListItem::new("  (empty)").style(Style::default().fg(Color::DarkGray))],
    };

    let title = if app.current_path.is_empty() {
        format!(" ▸ {} ", app.target.repo)
    } else {
        format!(" ▸ {} ", app.current_path)
    };

    let block = Block::default().title(title).borders(Borders::ALL);
    let list = List::new(entries).block(block);
    frame.render_widget(list, area);
}
