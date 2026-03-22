use anyhow::Result;
use crossterm::event::KeyEvent;
use tokio::sync::mpsc::Sender;

use crate::app::{App, AppMode, NavAction};
use crate::event::AppCommand;
use crate::github::RepoTarget;

pub async fn handle_key(app: &mut App, key: KeyEvent, cmd_tx: &Sender<AppCommand>) -> Result<bool> {
    use AppMode::*;
    use crossterm::event::{KeyCode, KeyModifiers};

    match &app.mode {
        Browse | Downloading => match key.code {
            KeyCode::Char('q') | KeyCode::Esc => return Ok(true),
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => return Ok(true),

            KeyCode::Char('b') => app.mode = AppMode::Input,

            KeyCode::Char('j') | KeyCode::Down => app.move_down(),
            KeyCode::Char('k') | KeyCode::Up => app.move_up(),

            KeyCode::Char('h') | KeyCode::Backspace => app.go_back(),

            KeyCode::Char('l') | KeyCode::Enter => {
                dispatch_nav(app, cmd_tx).await?;
            }

            KeyCode::Char('p') => {
                if let Some(entry) = app.current_entry().cloned() {
                    if entry.entry_type == crate::github::EntryType::File {
                        app.mode = AppMode::Previewing;
                        cmd_tx.send(AppCommand::PreviewFile(entry)).await.ok();
                    }
                }
            }

            KeyCode::Char(' ') => app.toggle_select(),

            KeyCode::Char('d') => {
                let action = app.start_download();
                dispatch_action(action, app, cmd_tx).await?;
            }

            KeyCode::Char('r') => {
                let path = app.current_path.clone();
                app.tree.insert(path, crate::app::NodeState::Loading);
                cmd_tx
                    .send(AppCommand::FetchDir {
                        path: String::new(),
                        target: app.target.clone(),
                    })
                    .await
                    .ok();
            }

            _ => {}
        },

        Previewing => match key.code {
            KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('h') => {
                app.reset_preview();
            }
            KeyCode::Char('j') | KeyCode::Down => app.scroll_preview_down(),
            KeyCode::Char('k') | KeyCode::Up => app.scroll_preview_up(),
            _ => {}
        },

        Error(_) => match key.code {
            KeyCode::Esc | KeyCode::Char('q') | KeyCode::Enter => {
                app.mode = AppMode::Browse;
            }
            _ => {}
        },

        Input => match key.code {
            KeyCode::Esc => {
                if app.target.owner.is_empty() {
                    return Ok(true);
                }
                app.input_buffer.clear();
                app.mode = AppMode::Browse;
            }

            KeyCode::Char(c) => app.input_buffer.push(c),

            KeyCode::Backspace => {
                app.input_buffer.pop();
            }

            KeyCode::Enter => {
                if let Some(target) = parse_input(&app.input_buffer) {
                    app.reset_for_target(target.clone());
                    app.tree
                        .insert(String::new(), crate::app::NodeState::Loading);
                    cmd_tx
                        .send(AppCommand::FetchDir {
                            path: String::new(),
                            target,
                        })
                        .await
                        .ok();
                }
            }

            _ => {}
        },
    }

    Ok(false)
}

async fn dispatch_nav(app: &mut App, cmd_tx: &Sender<AppCommand>) -> Result<()> {
    let action = app.enter_current();
    dispatch_action(action, app, cmd_tx).await
}

async fn dispatch_action(action: NavAction, app: &App, cmd_tx: &Sender<AppCommand>) -> Result<()> {
    match action {
        NavAction::FetchDir(path) => {
            if !matches!(app.tree.get(&path), Some(crate::app::NodeState::Loaded(_))) {
                cmd_tx
                    .send(AppCommand::FetchDir {
                        path,
                        target: app.target.clone(),
                    })
                    .await
                    .ok();
            }
        }
        NavAction::PreviewFile(entry) => {
            cmd_tx.send(AppCommand::PreviewFile(entry)).await.ok();
        }
        NavAction::Download(entries) => {
            cmd_tx
                .send(AppCommand::Download {
                    entries,
                    repo: app.target.repo.clone(),
                })
                .await
                .ok();
        }
        NavAction::Quit | NavAction::None => {}
    }
    Ok(())
}

fn parse_input(s: &str) -> Option<RepoTarget> {
    let s = s.trim();
    let s = s
        .trim_start_matches("https://github.com/")
        .trim_start_matches("http://github.com/")
        .trim_end_matches(".git");

    let (repo_part, branch) = if let Some((r, b)) = s.split_once('@') {
        (r, b.to_string())
    } else {
        (s, String::from("main"))
    };

    let (owner, repo) = repo_part.split_once('/')?;
    if owner.is_empty() || repo.is_empty() {
        return None;
    }

    Some(RepoTarget {
        owner: owner.to_string(),
        repo: repo.to_string(),
        branch,
    })
}
