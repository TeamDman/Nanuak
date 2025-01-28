pub mod db;
pub mod crawl_repos;
pub mod messages;

use clap::{Parser, Subcommand};
use color_eyre::eyre::Result;
use db::upsert_cloned_repos;
use diesel::{r2d2::ConnectionManager, PgConnection};
use messages::CrawlMessage;
use r2d2::Pool;
use tracing::{info, warn};
use tracing_subscriber::EnvFilter;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(version, about = "Nanuak Git CLI")]
struct Cli {
    /// If set, enable debug logging
    #[arg(long)]
    debug: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Crawl local directory for Git repos
    Crawl {
        /// The starting directory to search
        #[arg(long, default_value = ".")]
        dir: PathBuf,
    },
    // Future subcommand placeholders
    // e.g. "sync", "fetch-remote", etc.
}

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    // Parse CLI
    let cli = Cli::parse();

    // Setup logging
    let log_level = if cli.debug { "DEBUG" } else { "INFO" };
    let env_filter = EnvFilter::builder()
        .with_default_directive(log_level.parse().unwrap_or_else(|_| "INFO".parse().unwrap()))
        .from_env_lossy();
    tracing_subscriber::fmt().with_env_filter(env_filter).init();

    //
    // 1) Setup a DB pool
    //
    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set in env or .env");
    let manager = ConnectionManager::<PgConnection>::new(&database_url);
    let pool = Pool::builder()
        .build(manager)
        .expect("Failed to create r2d2 pool for PgConnection");


    match cli.command {
        Commands::Crawl { dir } => {
            // Start the crawler
            let (mut rx, handle) = crawl_repos::crawl_repos(dir);

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
        }
    }

    Ok(())
}
