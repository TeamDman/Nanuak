use crate::crawl_message::CrawlMessage;
use crate::get_remotes::get_remotes;
use color_eyre::eyre::Result;
use eyre::eyre;
use ignore::WalkBuilder;
use std::path::PathBuf;
use tokio::sync::mpsc::Receiver;
use tokio::sync::mpsc::{self};
use tracing::info;

pub fn crawl_repos(
    start_dir: PathBuf,
) -> (Receiver<CrawlMessage>, tokio::task::JoinHandle<Result<()>>) {
    // Create an MPSC channel
    let (tx, rx) = mpsc::channel::<CrawlMessage>(100);

    // Spawn a new task that performs the crawling
    let handle = tokio::spawn(async move {
        // We'll do the scanning in here
        if !start_dir.exists() {
            let _ = tx
                .send(CrawlMessage::Error(eyre!(
                    "Start dir does not exist: {:?}",
                    start_dir
                )))
                .await;
            // Then we can bail
            return Ok(());
        }

        info!("Crawling for Git repos in {:?}", start_dir);

        // We'll do the scanning using ignore::WalkBuilder
        for result in WalkBuilder::new(&start_dir).build() {
            let entry = match result {
                Ok(e) => e,
                Err(e) => {
                    let _ = tx
                        .send(CrawlMessage::Error(eyre!("Reading entry error: {:?}", e)))
                        .await;
                    continue;
                }
            };

            if entry.file_type().is_some_and(|ft| ft.is_dir()) {
                let path = entry.path();
                // If path/.git exists, we consider it a Git repo
                if path.join(".git").is_dir() {
                    let remotes = get_remotes(path).await?;
                    tx.send(CrawlMessage::FoundRepo {
                        path: path.to_path_buf(),
                        remotes,
                    })
                    .await?;
                }
            }
        }

        // Once done scanning
        let _ = tx.send(CrawlMessage::Done).await;
        Ok(())
    });

    // Return the receiver + handle
    (rx, handle)
}
