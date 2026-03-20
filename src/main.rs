mod app;
mod config;
mod event;
mod github;
mod ui;
mod worker;
use std::io;

use anyhow::Result;
use app::{App, RepoTarget};
use crossterm::{
    event::{self as ct_event, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};

#[tokio::main]
async fn main() -> Result<()> {
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let _ = disable_raw_mode();
        let _ = execute!(io::stderr(), LeaveAlternateScreen);
        default_hook(info);
    }));

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;

    let backend  = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let app = App::new(RepoTarget {
        owner:  String::from("torvalds"),
        repo:   String::from("linux"),
        branch: String::from("master"),
    });

    loop {
        ui::draw(&mut terminal, &app)?;

        if ct_event::poll(std::time::Duration::from_millis(250))? {
            if let Event::Key(key) = ct_event::read()? {
                if key.code == KeyCode::Char('q') {
                    break;
                }
            }
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}
