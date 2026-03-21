mod app;
mod config;
mod event;
mod github;
mod ui;
mod worker;

use std::{io, path::PathBuf, time::Duration};

use anyhow::Result;
use app::{App, AppMode, DownloadState, NodeState, RepoTarget};
use crossterm::{
    event::{self as ct_event, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use event::{AppCommand, WorkerEvent};
use ratatui::{Terminal, backend::CrosstermBackend};
use tokio::sync::mpsc;
use worker::WorkerPool;

#[tokio::main]
async fn main() -> Result<()> {
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let _ = disable_raw_mode();
        let _ = execute!(io::stderr(), LeaveAlternateScreen);
        default_hook(info);
    }));

    let cfg = config::Config::load().unwrap_or_else(|_| config::Config::default());
    let (cmd_tx, mut cmd_rx) = mpsc::channel::<AppCommand>(64);
    let (event_tx, mut event_rx) = mpsc::channel::<WorkerEvent>(256);

    let out_dir = cfg
        .output
        .as_ref()
        .and_then(|o| o.dir.as_ref())
        .map(|d| PathBuf::from(shellexpand::tilde(d).into_owned()))
        .unwrap_or_else(|| dirs::download_dir().unwrap_or_else(|| PathBuf::from(".")));

    let target = parse_args().unwrap_or_else(|| RepoTarget {
        owner: String::new(),
        repo: String::new(),
        branch: String::from("main"),
    });

    let repo_name = target.repo.clone();
    let out_dir = out_dir.join(&repo_name);

    let concurrency = cfg
        .download
        .as_ref()
        .and_then(|d| d.concurrency)
        .unwrap_or(4);

    let http_client = reqwest::Client::new();
    let pool = WorkerPool::new(concurrency, http_client, event_tx);

    let pool_arc = std::sync::Arc::new(pool);
    let pool_clone = std::sync::Arc::clone(&pool_arc);
    let out_dir_clone = out_dir.clone();

    tokio::spawn(async move {
        while let Some(cmd) = cmd_rx.recv().await {
            pool_clone.handle(cmd, out_dir_clone.clone()).await;
        }
    });

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new(target.clone());

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

    // main event loop
    loop {
        ui::draw(&mut terminal, &app)?;

        tokio::select! {
            // worker events — non-blocking poll
            event = event_rx.recv() => {
                if let Some(ev) = event {
                    handle_worker_event(&mut app, ev);
                }
            }
            _ = tokio::time::sleep(Duration::from_millis(16)) => {
                if ct_event::poll(Duration::ZERO)? {
                    if let Event::Key(key) = ct_event::read()? {
                        if handle_key(&mut app, key, &cmd_tx).await? {
                            break; // quit
                        }
                    }
                }
            }
        }
    }

    // cleanup
    pool_arc.cancel();
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}

async fn handle_key(
    app: &mut App,
    key: ct_event::KeyEvent,
    cmd_tx: &mpsc::Sender<AppCommand>,
) -> Result<bool> {
    use AppMode::*;

    match &app.mode {
        Browse | Downloading => match key.code {
            KeyCode::Char('q') | KeyCode::Esc => return Ok(true),
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => return Ok(true),
            KeyCode::Char('b') => {
                app.mode = AppMode::Input;
            }

            KeyCode::Char('j') | KeyCode::Down => {
                let max = current_entries_len(app).saturating_sub(1);
                if app.cursor < max {
                    app.cursor += 1;
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                if app.cursor > 0 {
                    app.cursor -= 1;
                }
            }

            KeyCode::Char('l') | KeyCode::Enter => {
                if let Some(entry) = current_entry(app) {
                    match entry.entry_type {
                        github::EntryType::Dir => {
                            let path = entry.path.clone();
                            if !matches!(app.tree.get(&path), Some(NodeState::Loaded(_))) {
                                app.tree.insert(path.clone(), NodeState::Loading);
                                cmd_tx
                                    .send(AppCommand::FetchDir {
                                        path: path.clone(),
                                        target: app.target.clone(),
                                    })
                                    .await
                                    .ok();
                            }
                            app.current_path = path;
                            app.cursor = 0;
                            app.scroll = 0;
                        }
                        github::EntryType::File => {
                            let entry = entry.clone();
                            app.mode = AppMode::Previewing;
                            cmd_tx.send(AppCommand::PreviewFile(entry)).await.ok();
                        }
                    }
                }
            }
            KeyCode::Char('h') | KeyCode::Backspace => {
                if !app.current_path.is_empty() {
                    let parent = app
                        .current_path
                        .rfind('/')
                        .map(|i| app.current_path[..i].to_string())
                        .unwrap_or_default();
                    app.current_path = parent;
                    app.cursor = 0;
                    app.scroll = 0;
                }
            }

            KeyCode::Char(' ') => {
                if let Some(entry) = current_entry(app) {
                    let path = entry.path.clone();
                    if app.selected.contains(&path) {
                        app.selected.remove(&path);
                    } else {
                        app.selected.insert(path);
                    }
                }
            }

            KeyCode::Char('d') => {
                if !app.selected.is_empty() {
                    // collect selected GithubEntry from tree
                    let to_dl: Vec<_> = app
                        .tree
                        .values()
                        .filter_map(|n| match n {
                            NodeState::Loaded(entries) => Some(entries),
                            _ => None,
                        })
                        .flatten()
                        .filter(|e| app.selected.contains(&e.path))
                        .cloned()
                        .collect();

                    if !to_dl.is_empty() {
                        app.mode = AppMode::Downloading;
                        cmd_tx.send(AppCommand::Download(to_dl)).await.ok();
                    }
                }
            }

            KeyCode::Char('p') => {
                if let Some(entry) = current_entry(app) {
                    if entry.entry_type == github::EntryType::File {
                        let entry = entry.clone();
                        app.mode = AppMode::Previewing;
                        cmd_tx.send(AppCommand::PreviewFile(entry)).await.ok();
                    }
                }
            }

            KeyCode::Char('r') => {
                let path = app.current_path.clone();
                app.tree.insert(path.clone(), NodeState::Loading);
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
            KeyCode::Esc | KeyCode::Char('q') => {
                app.mode = AppMode::Browse;
                app.preview = None;
            }
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

            KeyCode::Char(c) => {
                app.input_buffer.push(c);
            }

            KeyCode::Backspace => {
                app.input_buffer.pop();
            }

            KeyCode::Enter => {
                if let Some(target) = parse_input(&app.input_buffer) {
                    app.target = target;
                    app.input_buffer = String::new();
                    app.tree.clear();
                    app.selected.clear();
                    app.downloads.clear();
                    app.current_path = String::new();
                    app.cursor = 0;
                    app.mode = AppMode::Browse;

                    app.tree.insert(String::new(), NodeState::Loading);
                    cmd_tx
                        .send(AppCommand::FetchDir {
                            path: String::new(),
                            target: app.target.clone(),
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

fn handle_worker_event(app: &mut App, ev: WorkerEvent) {
    match ev {
        WorkerEvent::DirLoaded { path, entries } => {
            app.tree.insert(path, NodeState::Loaded(entries));
        }

        WorkerEvent::Progress {
            id,
            downloaded,
            total,
        } => {
            app.downloads
                .insert(id, DownloadState::Downloading { downloaded, total });
        }

        WorkerEvent::Done { id, path: _ } => {
            app.downloads.insert(id, DownloadState::Done);
            let all_done = app
                .downloads
                .values()
                .all(|s| matches!(s, DownloadState::Done | DownloadState::Error(_)));
            if all_done {
                app.mode = AppMode::Browse;
            }
        }

        WorkerEvent::PreviewReady { content } => {
            app.preview = Some(content);
        }

        WorkerEvent::Error { id: _, msg } => {
            app.mode = AppMode::Error(msg);
        }
    }
}

fn current_entries_len(app: &App) -> usize {
    match app.tree.get(&app.current_path) {
        Some(NodeState::Loaded(e)) => e.len(),
        _ => 0,
    }
}

fn current_entry(app: &App) -> Option<&github::GithubEntry> {
    match app.tree.get(&app.current_path) {
        Some(NodeState::Loaded(entries)) => entries.get(app.cursor),
        _ => None,
    }
}

fn parse_args() -> Option<RepoTarget> {
    let arg = std::env::args().nth(1)?;
    let (repo_part, branch) = if let Some((r, b)) = arg.split_once('@') {
        (r, b.to_string())
    } else {
        (arg.as_str(), String::from("main"))
    };
    let (owner, repo) = repo_part.split_once('/')?;
    Some(RepoTarget {
        owner: owner.to_string(),
        repo: repo.to_string(),
        branch,
    })
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
