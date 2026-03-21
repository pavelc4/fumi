use crate::app::{App, AppMode, DownloadState, NodeState};
use crate::event::WorkerEvent;

/// Update App state based on events from the worker pool.
pub fn handle_worker_event(app: &mut App, ev: WorkerEvent) {
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
            let all_done = app.downloads.values().all(|s| {
                matches!(s, DownloadState::Done | DownloadState::Error(_))
            });
            if all_done && matches!(app.mode, AppMode::Downloading) {
                app.mode = AppMode::Browse;
            }
        }

        WorkerEvent::PreviewReady { content } => {
            app.preview = Some(content);
        }

        WorkerEvent::Error { id, msg } => {
            if id == 0 {
                // id=0 used for non-download errors (FetchDir, Preview)
                app.mode = AppMode::Error(msg);
            } else {
                app.downloads.insert(id, DownloadState::Error(msg));
            }
        }
    }
}
