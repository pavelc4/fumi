use std::sync::Arc;

use reqwest::Client;
use tokio::sync::{Semaphore, mpsc::Sender};
use tokio_util::sync::CancellationToken;

use crate::{
    app::RepoTarget,
    event::{AppCommand, WorkerEvent},
};

pub mod api;
pub mod download;

pub struct WorkerPool {
    semaphore: Arc<Semaphore>,
    client: Client,
    tx: Sender<WorkerEvent>,
    root_token: CancellationToken,
    target: Arc<RepoTarget>,
}

impl WorkerPool {
    pub fn new(
        concurrency: usize,
        client: Client,
        tx: Sender<WorkerEvent>,
        target: RepoTarget,
    ) -> Self {
        Self {
            semaphore: Arc::new(Semaphore::new(concurrency)),
            client,
            tx,
            root_token: CancellationToken::new(),
            target: Arc::new(target),
        }
    }

    pub async fn handle(&self, cmd: AppCommand, out_dir: std::path::PathBuf) {
        match cmd {
            AppCommand::FetchDir(path) => {
                let client = self.client.clone();
                let tx = self.tx.clone();
                let target = Arc::clone(&self.target);
                let token = self.root_token.child_token();

                tokio::spawn(async move {
                    tokio::select! {
                        biased;
                        _ = token.cancelled() => {}
                        _ = api::fetch_dir(&client, &target, &path, tx) => {}
                    }
                });
            }

            AppCommand::Download(entries) => {
                for entry in entries {
                    let client = self.client.clone();
                    let tx = self.tx.clone();
                    let token = self.root_token.child_token();
                    let sem = Arc::clone(&self.semaphore);
                    let out = out_dir.clone();
                    let id = next_id();

                    tokio::spawn(async move {
                        let permit = sem.acquire_owned().await.unwrap();
                        download::download_file(client, entry, id, out, tx, token).await;
                        drop(permit);
                    });
                }
            }

            AppCommand::PreviewFile(entry) => {
                let client = self.client.clone();
                let tx = self.tx.clone();
                let token = self.root_token.child_token();

                tokio::spawn(async move {
                    tokio::select! {
                        biased;
                        _ = token.cancelled() => {}
                        _ = api::fetch_preview(&client, entry, tx) => {}
                    }
                });
            }

            AppCommand::Cancel => {
                self.root_token.cancel();
            }
        }
    }

    pub fn cancel(&self) {
        self.root_token.cancel();
    }
}

fn next_id() -> u64 {
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(1);
    COUNTER.fetch_add(1, Ordering::Relaxed)
}
