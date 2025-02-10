#![feature(try_blocks)]
pub mod actions;
pub mod crawl_message;
pub mod crawl_repos;
pub mod fetch_github_repo_details;
pub mod get_database_url;
pub mod get_remotes;
pub mod get_repo_list_from_db;
pub mod pick_remote;
pub mod pick_repo;
pub mod remotes;
pub mod repo_manifest;
pub mod upsert_cloned_repos;

use clap::Parser;
use clap::Subcommand;
use color_eyre::eyre::Result;
use diesel::r2d2::ConnectionManager;
use diesel::PgConnection;
use get_database_url::get_database_url;
use r2d2::Pool;
use std::path::PathBuf;
use tracing_subscriber::EnvFilter;

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
    /// Summarize a repository in the DB
    Summarize,
}

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    // Parse CLI
    let cli = Cli::parse();

    // Setup logging
    let log_level = if cli.debug { "DEBUG" } else { "INFO" };
    let env_filter = EnvFilter::builder()
        .with_default_directive(
            log_level
                .parse()
                .unwrap_or_else(|_| "INFO".parse().unwrap()),
        )
        .from_env_lossy();
    tracing_subscriber::fmt().with_env_filter(env_filter).init();

    //
    // 1) Setup a DB pool
    //
    let database_url = get_database_url().await?;
    let manager = ConnectionManager::<PgConnection>::new(&database_url);
    let pool = Pool::builder()
        .build(manager)
        .expect("Failed to create r2d2 pool for PgConnection");

    match cli.command {
        Commands::Crawl { dir } => {
            actions::crawl_repos_action::crawl_repos_action(dir, pool).await?;
        }
        Commands::Summarize => {
            actions::summarize_repos_action::summarize_repos_action(pool).await?;
        }
    }

    Ok(())
}
