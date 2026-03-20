use std::path::PathBuf;

use anyhow::Result;
use futures::StreamExt;
use reqwest::Client;
use tokio::{fs, io::AsyncWriteExt, sync::mpsc::Sender};
use tokio_util::sync::CancellationToken;

use crate::{event::WorkerEvent, github::GithubEntry};

pub async fn download_file(
    client: Client,
    entry: GithubEntry,
    id: u64,
    out_dir: PathBuf,
    tx: Sender<WorkerEvent>,
    token: CancellationToken,
) {
    if let Err(e) = run(&client, &entry, id, &out_dir, &tx, &token).await {
        tx.send(WorkerEvent::Error {
            id,
            msg: e.to_string(),
        })
        .await
        .ok();
    }
}

async fn run(
    client: &Client,
    entry: &GithubEntry,
    id: u64,
    out_dir: &PathBuf,
    tx: &Sender<WorkerEvent>,
    token: &CancellationToken,
) -> Result<()> {
    let url = entry
        .download_url
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("no download_url for {}", entry.path))?;

    let final_path = out_dir.join(&entry.path);
    let partial_path = final_path.with_extension(format!(
        "{}.fumi_partial",
        final_path.extension().unwrap_or_default().to_string_lossy()
    ));

    if let Some(parent) = final_path.parent() {
        fs::create_dir_all(parent).await?;
    }

    let resp = client.get(url).header("User-Agent", "fumi").send().await?;
    let total = resp.content_length().unwrap_or(0);
    let mut stream = resp.bytes_stream();

    let mut file = fs::File::create(&partial_path).await?;
    let mut downloaded = 0u64;

    loop {
        tokio::select! {
            biased;

            _ = token.cancelled() => {
                drop(file);
                fs::remove_file(&partial_path).await.ok();
                break;
            }

            chunk = stream.next() => {
                match chunk {
                    Some(Ok(bytes)) => {
                        file.write_all(&bytes).await?;
                        downloaded += bytes.len() as u64;
                        tx.send(WorkerEvent::Progress { id, downloaded, total }).await.ok();
                    }
                    Some(Err(e)) => return Err(e.into()),
                    None => {
                        drop(file);
                        fs::rename(&partial_path, &final_path).await?;
                        tx.send(WorkerEvent::Done {
                            id,
                            path: final_path.display().to_string(),
                        }).await.ok();
                        break;
                    }
                }
            }
        }
    }

    Ok(())
}
