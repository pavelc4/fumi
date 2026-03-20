use std::collections::{HashMap, HashSet};

use crate::github::GithubEntry;

#[derive(Debug)]
pub enum AppMode {
    Browse,
    Input,
    Downloading,
    Previewing,
    Error(String),
}

#[derive(Debug)]
pub enum NodeState {
    Unloaded,
    Loading,
    Loaded(Vec<GithubEntry>),
}

#[derive(Debug)]
pub enum DownloadState {
    Queued,
    Downloading { downloaded: u64, total: u64 },
    Done,
    Error(String),
}

#[derive(Debug, Copy, Clone)]
pub enum TreeStrategy {
    Lazy,
    FullTree,
}

#[derive(Debug, Clone)]
pub struct RepoTarget {
    pub owner: String,
    pub repo: String,
    pub branch: String,
}

#[derive(Debug)]
pub struct App {
    pub target: RepoTarget,
    pub tree: HashMap<String, NodeState>,
    pub cursor: usize,
    pub scroll: usize,
    pub selected: HashSet<String>,
    pub downloads: HashMap<u64, DownloadState>,
    pub preview: Option<String>,
    pub mode: AppMode,
    pub strategy: TreeStrategy,
    pub current_path: String,
    pub input_buffer: String,
}

impl App {
    pub fn new(target: RepoTarget) -> Self {
        Self {
            target,
            tree: HashMap::new(),
            cursor: 0,
            scroll: 0,
            selected: HashSet::new(),
            downloads: HashMap::new(),
            preview: None,
            mode: AppMode::Browse,
            strategy: TreeStrategy::Lazy,
            current_path: String::from(""),
            input_buffer: String::new(),
        }
    }
}
