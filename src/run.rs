use std::time::Duration;

use anyhow::Result;
use crossterm::{
    event::{self as ct_event, Event},
    execute,
    terminal::{LeaveAlternateScreen, disable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};
use tokio::sync::mpsc::{Receiver, Sender};

use crate::app::{App, AppMode, NodeState};
use crate::event::{AppCommand, WorkerEvent};
use crate::handler::{key::handle_key, worker::handle_worker_event};

pub async fn run(
    mut app: App,
    mut terminal: Terminal<CrosstermBackend<std::io::Stdout>>,
    cmd_tx: Sender<AppCommand>,
    mut event_rx: Receiver<WorkerEvent>,
) -> Result<()> {
    // If a repo was given as arg, kick off the initial fetch
    if !app.target.owner.is_empty() {
        app.tree.insert(String::new(), NodeState::Loading);
        cmd_tx
            .send(AppCommand::FetchDir {
                path: String::new(),
                target: app.target.clone(),
            })
            .await
            .ok();
    } else {
        app.mode = AppMode::Input;
    }

    loop {
        crate::ui::draw(&mut terminal, &app)?;

        tokio::select! {
            ev = event_rx.recv() => {
                if let Some(ev) = ev {
                    handle_worker_event(&mut app, ev);
                }
            }
            _ = tokio::time::sleep(Duration::from_millis(16)) => {
                if ct_event::poll(Duration::ZERO)? {
                    if let Event::Key(key) = ct_event::read()? {
                        if handle_key(&mut app, key, &cmd_tx).await? {
                            break;
                        }
                    }
                }
            }
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}
