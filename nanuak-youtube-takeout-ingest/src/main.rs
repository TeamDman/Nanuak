pub mod entry;
pub mod search_entry;
pub mod watch_entry;
pub mod view_post_entry;
use clap::Parser;
use color_eyre::eyre::Result;
use entry::load_entries;
use std::path::PathBuf;
use tracing::info;

/// Command-line arguments for the ingest tool
#[derive(Parser, Debug)]
#[command(version, about = "Ingest YouTube Takeout JSON Files")]
struct Args {
    /// Path to the directory containing JSON files
    #[arg(short, long, value_name = "DIR")]
    ingest_dir: PathBuf,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    // Initialize color-eyre for better error reporting
    color_eyre::install()?;

    // Parse command-line arguments
    let args = Args::parse();

    // Read the directory and list .json files
    let mut entries = tokio::fs::read_dir(&args.ingest_dir).await?;
    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("json") {
            info!("Found JSON file: {}", path.display());
            let entries = load_entries(&path).await?;
            info!("Loaded {} entries", entries.len());
        }
    }

    Ok(())
}
