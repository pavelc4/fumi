mod app;
mod config;
mod event;
mod github;
mod handler;
mod run;
mod ui;
mod worker;

use std::{io, path::PathBuf};

use anyhow::Result;
use app::App;
use crossterm::{
    execute,
    terminal::{EnterAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use event::{AppCommand, WorkerEvent};
use github::RepoTarget;
use ratatui::{Terminal, backend::CrosstermBackend};
use tokio::sync::mpsc;

#[tokio::main]
async fn main() -> Result<()> {
    // Restore terminal on panic
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let _ = disable_raw_mode();
        let _ = execute!(io::stderr(), crossterm::terminal::LeaveAlternateScreen);
        default_hook(info);
    }));

    // Load config (fall back to defaults if missing)
    let cfg = config::Config::load().unwrap_or_default();

    // Channels
    let (cmd_tx, mut cmd_rx) = mpsc::channel::<AppCommand>(64);
    let (event_tx, event_rx) = mpsc::channel::<WorkerEvent>(256);

    // Resolve output directory
    let out_dir = cfg
        .output
        .as_ref()
        .and_then(|o| o.dir.as_ref())
        .map(|d| PathBuf::from(shellexpand::tilde(d).into_owned()))
        .unwrap_or_else(|| dirs::download_dir().unwrap_or_else(|| PathBuf::from(".")));

    // Parse CLI arg  →  owner/repo[@branch]
    let target = parse_args().unwrap_or_else(|| RepoTarget {
        owner: String::new(),
        repo: String::new(),
        branch: String::from("main"),
    });

    let out_dir = out_dir.join(&target.repo);

    let concurrency = cfg
        .download
        .as_ref()
        .and_then(|d| d.concurrency)
        .unwrap_or(4);

    // Worker pool
    let pool = std::sync::Arc::new(worker::WorkerPool::new(
        concurrency,
        reqwest::Client::new(),
        event_tx,
    ));

    // Dispatch AppCommands → WorkerPool
    let pool_cmd = std::sync::Arc::clone(&pool);
    tokio::spawn(async move {
        while let Some(cmd) = cmd_rx.recv().await {
            pool_cmd.handle(cmd, out_dir.clone()).await;
        }
    });

    // Terminal setup
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let terminal = Terminal::new(CrosstermBackend::new(stdout))?;

    // Run the TUI event loop
    run::run(App::new(target), terminal, cmd_tx, event_rx).await?;

    // Cancel any in-flight workers
    pool.cancel();

    Ok(())
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
