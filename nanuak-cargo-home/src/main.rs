use eyre::Result;
use futures::StreamExt;
use std::env;
use std::path::PathBuf;
use tokio::fs;
use tokio::fs::DirEntry;
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};
use tokio_stream::wrappers::UnboundedReceiverStream;

/// Returns the Cargo home directory.
/// It checks CARGO_HOME first; if not set, falls back to $HOME/.cargo, else ".cargo".
fn get_cargo_home() -> PathBuf {
    if let Ok(cargo_home) = env::var("CARGO_HOME") {
        PathBuf::from(cargo_home)
    } else if let Ok(home) = env::var("HOME") {
        PathBuf::from(home).join(".cargo")
    } else {
        PathBuf::from(".cargo")
    }
}

/// Asynchronously scans the given directory and sends each sub‑directory over the provided channel.
async fn scan_dir(dir: PathBuf, tx: UnboundedSender<DirEntry>) -> Result<()> {
    let mut read_dir = fs::read_dir(&dir).await?;
    while let Some(entry) = read_dir.next_entry().await? {
        let ft = entry.file_type().await?;
        if ft.is_dir() {
            // Send the entry; if the receiver is dropped, break.
            if tx.send(entry).is_err() {
                break;
            }
        }
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    // 1. Get Cargo home.
    let cargo_home = get_cargo_home();
    println!("Cargo home: {:?}", cargo_home);

    // 2. Build the registry/src path.
    let registry_src = cargo_home.join("registry").join("src");
    if !registry_src.exists() {
        eyre::bail!("Registry src not found at {:?}", registry_src);
    }
    println!("Registry src: {:?}", registry_src);

    // Create an unbounded channel for registry directories.
    let (reg_tx, reg_rx): (UnboundedSender<DirEntry>, UnboundedReceiver<DirEntry>) =
        mpsc::unbounded_channel();
    // Spawn a task to scan registry_src and send each registry directory.
    tokio::spawn(scan_dir(registry_src, reg_tx));

    // Create an unbounded channel for crate directories.
    let (crate_tx, crate_rx): (UnboundedSender<DirEntry>, UnboundedReceiver<DirEntry>) =
        mpsc::unbounded_channel();

    // Convert the registry receiver into a stream.
    let reg_stream = UnboundedReceiverStream::new(reg_rx);

    // Future A: Process the registry stream.
    // For each registry directory that arrives, spawn a new task to scan it for child (crate) directories.
    let registry_processing = async {
        reg_stream
            .for_each_concurrent(None, |reg_entry| {
                let path = reg_entry.path();
                let crate_tx = crate_tx.clone();
                async move {
                    // For each registry directory, scan for its children.
                    if let Err(e) = scan_dir(path, crate_tx) .await {
                        eprintln!("Error scanning child directories: {}", e);
                    }
                }
            })
            .await;
        // When done processing the registry stream, drop the crate sender so the receiver finishes.
        drop(crate_tx);
    };

    // Future B: Process the crate stream concurrently.
    // As crate directories arrive, print them and count them.
    let crate_processing = async {
        let crate_stream = UnboundedReceiverStream::new(crate_rx);
        // Use fold to count items.
        crate_stream
            .fold(0, |acc, crate_entry| async move {
                println!("Found crate directory: {:?}", crate_entry.file_name());
                acc + 1
            })
            .await
    };

    // Run both futures concurrently.
    let ((), total_crates) = tokio::join!(registry_processing, crate_processing);
    println!("Total crate directories found: {}", total_crates);

    Ok(())
}
