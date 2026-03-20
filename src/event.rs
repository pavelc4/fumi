use crate::github::GithubEntry;

#[derive(Debug)]
pub enum AppCommand {
    FetchDir(String),
    Download(Vec<GithubEntry>),
    PreviewFile(GithubEntry),
    Cancel,
}

#[derive(Debug)]
pub enum WorkerEvent {
    DirLoaded {
        path: String,
        entries: Vec<GithubEntry>,
    },
    Progress {
        id: u64,
        downloaded: u64,
        total: u64,
    },
    Done {
        id: u64,
        path: String,
    },
    PreviewReady {
        content: String,
    },
    Error {
        id: u64,
        msg: String,
    },
}
