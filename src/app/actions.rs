use crate::github::{EntryType, GithubEntry, RepoTarget};

use super::state::{ActivePanel, App, AppMode, NodeState};

#[derive(Debug)]
pub enum NavAction {
    FetchDir(String),
    PreviewFile(GithubEntry),
    Download(Vec<GithubEntry>),
    Quit,
    None,
}

impl App {
    pub fn move_down(&mut self) {
        let max = self.current_entries_len().saturating_sub(1);
        if self.cursor < max {
            self.cursor += 1;
        }
    }

    pub fn move_up(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
        }
    }

    pub fn enter_current(&mut self) -> NavAction {
        if !self.current_path.is_empty() && self.cursor == 0 {
            self.go_back();
            return NavAction::None;
        }

        let Some(entry) = self.current_entry().cloned() else {
            return NavAction::None;
        };

        match entry.entry_type {
            EntryType::Dir => {
                let path = entry.path.clone();
                if !matches!(self.tree.get(&path), Some(NodeState::Loaded(_))) {
                    self.tree.insert(path.clone(), NodeState::Loading);
                }
                self.current_path = path.clone();
                self.cursor = 0;
                self.scroll = 0;
                NavAction::FetchDir(path)
            }
            EntryType::File => {
                self.mode = AppMode::Previewing;
                self.active_panel = ActivePanel::Preview;
                NavAction::PreviewFile(entry)
            }
        }
    }

    pub fn go_back(&mut self) {
        if !self.current_path.is_empty() {
            let parent = self
                .current_path
                .rfind('/')
                .map(|i| self.current_path[..i].to_string())
                .unwrap_or_default();
            self.current_path = parent;
            self.cursor = 0;
            self.scroll = 0;
        }
    }

    pub fn toggle_select(&mut self) {
        if let Some(entry) = self.current_entry() {
            let path = entry.path.clone();
            if self.selected.contains(&path) {
                self.selected.remove(&path);
            } else {
                self.selected.insert(path);
            }
        }
    }

    pub fn start_download(&mut self) -> NavAction {
        let to_dl: Vec<GithubEntry> = self
            .tree
            .values()
            .filter_map(|n| match n {
                NodeState::Loaded(entries) => Some(entries),
                _ => None,
            })
            .flatten()
            .filter(|e| self.selected.contains(&e.path))
            .cloned()
            .collect();

        if to_dl.is_empty() {
            return NavAction::None;
        }
        self.mode = AppMode::Downloading;
        NavAction::Download(to_dl)
    }

    pub fn reset_for_target(&mut self, target: RepoTarget) {
        self.target = target;
        self.input_buffer.clear();
        self.tree.clear();
        self.selected.clear();
        self.downloads.clear();
        self.current_path.clear();
        self.cursor = 0;
        self.mode = AppMode::Browse;
    }

    pub fn scroll_preview_down(&mut self) {
        self.preview_scroll = self.preview_scroll.saturating_add(1);
    }

    pub fn scroll_preview_up(&mut self) {
        self.preview_scroll = self.preview_scroll.saturating_sub(1);
    }

    pub fn reset_preview(&mut self) {
        self.preview = None;
        self.preview_scroll = 0;
        self.active_panel = ActivePanel::FileTree;
        self.mode = AppMode::Browse;
    }

    pub fn current_entries_len(&self) -> usize {
        let base = match self.tree.get(&self.current_path) {
            Some(NodeState::Loaded(e)) => e.len(),
            _ => 0,
        };
        if self.current_path.is_empty() {
            base
        } else {
            base + 1
        }
    }

    pub fn current_entry(&self) -> Option<&GithubEntry> {
        match self.tree.get(&self.current_path) {
            Some(NodeState::Loaded(entries)) => {
                let idx = if self.current_path.is_empty() {
                    self.cursor
                } else {
                    self.cursor.checked_sub(1)?
                };
                entries.get(idx)
            }
            _ => None,
        }
    }
}
