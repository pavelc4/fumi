use reqwest::Client;
use serde::Deserialize;
use tokio::sync::mpsc::Sender;

use crate::{
    app::RepoTarget,
    event::WorkerEvent,
    github::{EntryType, GithubEntry},
};

#[derive(Debug, Deserialize)]
struct ApiEntry {
    name: String,
    path: String,
    sha: String,
    size: Option<u64>,
    #[serde(rename = "type")]
    entry_type: String,
    download_url: Option<String>,
}

pub async fn fetch_dir(client: &Client, target: &RepoTarget, path: &str, tx: Sender<WorkerEvent>) {
    let url = format!(
        "https://api.github.com/repos/{}/{}/contents/{}?ref={}",
        target.owner, target.repo, path, target.branch
    );

    let result = client
        .get(&url)
        .header("User-Agent", "fumi")
        .header("Accept", "application/vnd.github+json")
        .send()
        .await;

    match result {
        Err(e) => {
            tx.send(WorkerEvent::Error {
                id: 0,
                msg: e.to_string(),
            })
            .await
            .ok();
        }
        Ok(resp) => match resp.json::<Vec<ApiEntry>>().await {
            Err(e) => {
                tx.send(WorkerEvent::Error {
                    id: 0,
                    msg: e.to_string(),
                })
                .await
                .ok();
            }
            Ok(raw) => {
                let entries = raw
                    .into_iter()
                    .map(|e| GithubEntry {
                        name: e.name,
                        path: e.path.clone(),
                        sha: e.sha,
                        size: e.size.unwrap_or(0),
                        entry_type: if e.entry_type == "dir" {
                            EntryType::Dir
                        } else {
                            EntryType::File
                        },
                        Download_Url: e.download_url,
                    })
                    .collect();

                tx.send(WorkerEvent::DirLoaded {
                    path: path.to_string(),
                    entries,
                })
                .await
                .ok();
            }
        },
    }
}

pub async fn fetch_preview(client: &Client, entry: GithubEntry, tx: Sender<WorkerEvent>) {
    let url = match &entry.Download_Url {
        Some(u) => u.clone(),
        None => {
            tx.send(WorkerEvent::Error {
                id: 0,
                msg: "no download_url for preview".into(),
            })
            .await
            .ok();
            return;
        }
    };

    match client.get(&url).header("User-Agent", "fumi").send().await {
        Err(e) => {
            tx.send(WorkerEvent::Error {
                id: 0,
                msg: e.to_string(),
            })
            .await
            .ok();
        }
        Ok(resp) => match resp.text().await {
            Err(e) => {
                tx.send(WorkerEvent::Error {
                    id: 0,
                    msg: e.to_string(),
                })
                .await
                .ok();
            }
            Ok(content) => {
                tx.send(WorkerEvent::PreviewReady { content }).await.ok();
            }
        },
    }
}
