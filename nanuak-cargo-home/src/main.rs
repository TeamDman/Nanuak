use cloud_terrastodon_core_user_input::prelude::{pick_many, Choice, FzfArgs};
use eyre::Result;
use futures::StreamExt;
use itertools::Itertools;
use std::env;
use std::path::PathBuf;
use tokio::fs;
use tokio::fs::DirEntry;
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};
use tokio_stream::wrappers::UnboundedReceiverStream;

/// Returns the Cargo home directory.
/// It first checks CARGO_HOME; if not set, falls back to $HOME/.cargo, otherwise ".cargo".
fn get_cargo_home() -> PathBuf {
    if let Ok(cargo_home) = env::var("CARGO_HOME") {
        PathBuf::from(cargo_home)
    } else if let Ok(home) = env::var("HOME") {
        PathBuf::from(home).join(".cargo")
    } else {
        PathBuf::from(".cargo")
    }
}

/// Asynchronously scans the given directory and sends each subdirectory (DirEntry)
/// through the provided unbounded sender.
async fn scan_dir(dir: PathBuf, tx: UnboundedSender<DirEntry>) -> Result<()> {
    let mut read_dir = fs::read_dir(&dir).await?;
    while let Some(entry) = read_dir.next_entry().await? {
        let ft = entry.file_type().await?;
        if ft.is_dir() {
            // If sending fails (receiver dropped), break.
            if tx.send(entry).is_err() {
                break;
            }
        }
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    // Step 1: Get Cargo home.
    let cargo_home = get_cargo_home();
    println!("Cargo home: {:?}", cargo_home);

    // Step 2: Construct registry/src.
    let registry_src = cargo_home.join("registry").join("src");
    if !registry_src.exists() {
        eyre::bail!("Registry src not found at {:?}", registry_src);
    }
    println!("Registry src: {:?}", registry_src);

    // Step 3: Create a channel to stream registry directories.
    let (reg_tx, reg_rx): (UnboundedSender<DirEntry>, UnboundedReceiver<DirEntry>) =
        mpsc::unbounded_channel();

    // Spawn a task to scan registry/src for registry directories.
    tokio::spawn(scan_dir(registry_src, reg_tx));

    // Step 4: Create a channel for crate (child) directories.
    let (crate_tx, crate_rx): (UnboundedSender<DirEntry>, UnboundedReceiver<DirEntry>) =
        mpsc::unbounded_channel();

    // Wrap the registry receiver as a stream.
    let reg_stream = UnboundedReceiverStream::new(reg_rx);

    // Spawn a task that, for each registry directory as soon as it is received,
    // spawns a new scanning task for its children.
    let registry_processing = tokio::spawn(async move {
        reg_stream
            .for_each_concurrent(None, |reg_entry| {
                let path = reg_entry.path();
                let crate_tx = crate_tx.clone();
                async move {
                    // Scan this registry directory for its child directories.
                    if let Err(e) = scan_dir(path, crate_tx).await {
                        eprintln!("Error scanning child directories: {}", e);
                    }
                }
            })
            .await;
        // When done processing the registry stream, drop the sender.
        drop(crate_tx);
    });

    // Step 5: Wrap the crate receiver as a stream and collect all crate directories into a Vec.
    let crate_processing = async {
        let crate_stream = UnboundedReceiverStream::new(crate_rx);
        let crate_entries: Vec<DirEntry> = crate_stream.collect().await;
        crate_entries
    };

    // Run both tasks concurrently.
    let (registry_result, crate_entries) =
        tokio::join!(registry_processing, crate_processing);

    if let Err(e) = registry_result {
        eprintln!("Error processing registry directories: {}", e);
    }

    let crates_to_summarize = pick_many(FzfArgs {
        header: Some(format!("Found {} crates", crate_entries.len())),
        choices: crate_entries.into_iter().map(|c| Choice {
            key: c.file_name().to_string_lossy().to_string(),
            value: c,
        }).collect_vec(),
        prompt: Some("Crate to summarize: ".to_string()),
    })?;

    for crate_entry in crates_to_summarize {
        println!("Crate: {:?}", crate_entry.path());
    }

    Ok(())
}
