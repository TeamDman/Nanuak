use eyre::Result;
use futures::future::{BoxFuture, FutureExt};
use std::env;
use std::path::PathBuf;
use tokio::fs;
use tokio::fs::DirEntry;
use tokio::sync::mpsc::{self, Receiver, Sender};
use tower::Service;
use tower::ServiceExt; // provided by the "util" feature

/// Get the Cargo home directory.
/// First checks the CARGO_HOME env var, then falls back to $HOME/.cargo, else ".cargo".
fn get_cargo_home() -> PathBuf {
    if let Ok(cargo_home) = env::var("CARGO_HOME") {
        PathBuf::from(cargo_home)
    } else if let Ok(home) = env::var("HOME") {
        PathBuf::from(home).join(".cargo")
    } else {
        PathBuf::from(".cargo")
    }
}

/// A Tower service that, given a directory path, spawns a background task to scan
/// that directory for subdirectories. It sends each matching DirEntry to an mpsc channel
/// and returns the receiver immediately.
#[derive(Clone, Debug)]
struct StreamDirs;

impl Service<PathBuf> for StreamDirs {
    type Response = Receiver<DirEntry>;
    type Error = eyre::Report;
    type Future = BoxFuture<'static, Result<Self::Response>>;

    fn poll_ready(
        &mut self,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<()>> {
        // Always ready.
        std::task::Poll::Ready(Ok(()))
    }

    fn call(&mut self, dir: PathBuf) -> Self::Future {
        async move {
            // Create an mpsc channel with capacity 100.
            let (tx, rx) = mpsc::channel::<DirEntry>(100);
            // Spawn a background task to scan the directory.
            tokio::spawn(async move {
                if let Err(e) = scan_dir(dir, tx).await {
                    eprintln!("Error scanning directory: {}", e);
                }
            });
            Ok(rx)
        }
        .boxed()
    }
}

/// Asynchronously scans the given directory and sends each sub‑directory via the provided channel.
async fn scan_dir(dir: PathBuf, mut tx: Sender<DirEntry>) -> Result<()> {
    let mut read_dir = fs::read_dir(&dir).await?;
    while let Some(entry) = read_dir.next_entry().await? {
        let ft = entry.file_type().await?;
        if ft.is_dir() {
            // Send the entry; if the receiver is dropped, break.
            if tx.send(entry).await.is_err() {
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

    // 2. Form registry/src.
    let registry_src = cargo_home.join("registry").join("src");
    if !registry_src.exists() {
        eyre::bail!("Registry src not found at {:?}", registry_src);
    }
    println!("Registry src: {:?}", registry_src);

    // 3. Get the registry directories.
    let mut registry_rx = StreamDirs.oneshot(registry_src).await?;
    let mut registry_dirs = Vec::new();
    while let Some(entry) = registry_rx.recv().await {
        registry_dirs.push(entry);
    }
    println!("Found {} registry directories:", registry_dirs.len());
    for entry in &registry_dirs {
        println!(" - {:?}", entry.file_name());
    }

    // 4. For each registry directory, get its children (the crate directories).
    let mut all_crates = Vec::new();
    for registry_entry in registry_dirs {
        let reg_path = registry_entry.path();
        let mut crate_rx = StreamDirs.oneshot(reg_path.clone()).await?;
        let mut crate_dirs = Vec::new();
        while let Some(entry) = crate_rx.recv().await {
            crate_dirs.push(entry);
        }
        println!(
            "Registry directory {:?} has {} crate directories:",
            reg_path.file_name().unwrap_or_default(),
            crate_dirs.len()
        );
        for crate_entry in &crate_dirs {
            println!("    * {:?}", crate_entry.file_name());
        }
        all_crates.extend(crate_dirs);
    }

    println!("Total crates found: {}", all_crates.len());
    Ok(())
}
