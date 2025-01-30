use diesel::r2d2::ConnectionManager;
use diesel::PgConnection;
use r2d2::Pool;
use std::path::PathBuf;
use tracing::info;
use tracing::warn;

use crate::crawl_message::CrawlMessage;
use crate::crawl_repos::crawl_repos;
use crate::upsert_cloned_repos::upsert_cloned_repos;

pub async fn crawl_repos_action(
    dir: PathBuf,
    pool: Pool<ConnectionManager<PgConnection>>,
) -> eyre::Result<()> {
    // Start the crawler
    let (mut rx, handle) = crawl_repos(dir);

    // We'll get a single connection now. If you prefer to open/close
    // a new one for each FoundRepo, that's also possible.
    let mut conn = pool.get()?;

    // In a loop, read from rx
    while let Some(msg) = rx.recv().await {
        match msg {
            CrawlMessage::FoundRepo { path, remotes } => {
                println!("Found repo: {:?}, remotes={}", path, remotes);
                // e.g. store in DB, etc.
                upsert_cloned_repos(&mut conn, &[(path, remotes)])?;
            }
            CrawlMessage::Error(err) => {
                warn!("Crawler error: {:?}", err);
            }
            CrawlMessage::Done => {
                info!("Crawling done!");
                break;
            }
        }
    }

    // Optionally wait for the crawl task to exit
    let res = handle.await?;
    if let Err(e) = res {
        // handle the error from inside the spawned task
        warn!("Crawler task error: {}", e);
    }
    Ok(())
}
