use eyre::Result;
use futures::future::{BoxFuture, FutureExt};
use futures::stream::{self, StreamExt};
use std::env;
use std::path::PathBuf;
use tokio::fs;
use tokio::fs::DirEntry;
use tower::Service;
use tower::ServiceExt; // Requires the "util" feature in tower

/// Stage 1: Determine the Cargo home directory.
/// Checks CARGO_HOME first, then falls back to $HOME/.cargo.
#[derive(Clone, Debug)]
struct DetermineCargoHome;

impl Service<()> for DetermineCargoHome {
    type Response = PathBuf;
    type Error = eyre::Report;
    type Future = BoxFuture<'static, Result<Self::Response>>;

    fn poll_ready(
        &mut self,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<()>> {
        std::task::Poll::Ready(Ok(()))
    }

    fn call(&mut self, _req: ()) -> Self::Future {
        async move {
            if let Ok(cargo_home) = env::var("CARGO_HOME") {
                Ok(PathBuf::from(cargo_home))
            } else if let Ok(home) = env::var("HOME") {
                Ok(PathBuf::from(home).join(".cargo"))
            } else {
                Ok(PathBuf::from(".cargo"))
            }
        }
        .boxed()
    }
}

/// Stage 2: Given a Cargo home, return the registry/src folder.
#[derive(Clone, Debug)]
struct GetRegistrySrc;

impl Service<PathBuf> for GetRegistrySrc {
    type Response = PathBuf;
    type Error = eyre::Report;
    type Future = BoxFuture<'static, Result<Self::Response>>;

    fn poll_ready(
        &mut self,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<()>> {
        std::task::Poll::Ready(Ok(()))
    }
    fn call(&mut self, cargo_home: PathBuf) -> Self::Future {
        async move {
            let registry_src = cargo_home.join("registry").join("src");
            if registry_src.exists() {
                Ok(registry_src)
            } else {
                Err(eyre::eyre!("Registry src not found at {:?}", registry_src))
            }
        }
        .boxed()
    }
}

/// Stage 3: List the children of a given directory that are directories.
/// This stage uses a stream to concurrently call file_type() on each entry.
#[derive(Clone, Debug)]
struct ListDirs;

impl Service<PathBuf> for ListDirs {
    type Response = Vec<DirEntry>;
    type Error = eyre::Report;
    type Future = BoxFuture<'static, Result<Self::Response>>;

    fn poll_ready(
        &mut self,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<()>> {
        std::task::Poll::Ready(Ok(()))
    }
    
    fn call(&mut self, dir: PathBuf) -> Self::Future {
        async move {
            let mut read_dir = fs::read_dir(&dir).await?;
            let mut entries = Vec::new();
            while let Some(entry) = read_dir.next_entry().await? {
                entries.push(entry);
            }
            // For each entry, concurrently check if it is a directory.
            let dirs: Vec<DirEntry> = stream::iter(entries)
                // Use map (not then) so each closure returns a future.
                .map(|entry| async move {
                    let ft = entry.file_type().await?;
                    if ft.is_dir() {
                        Ok(Some(entry))
                    } else {
                        Ok(None)
                    }
                })
                .buffer_unordered(100)
                .filter_map(|result: eyre::Result<_>| async move { result.ok().flatten() })
                .collect()
                .await;
            Ok(dirs)
        }
        .boxed()
    }
}

/// Variant B: A streaming pipeline that directly returns a Vec<DirEntry> from a given directory.
/// Here we build an unfold stream and then use .map (not then) to produce futures that are run concurrently.
async fn stream_list_dirs(dir: PathBuf) -> Result<Vec<DirEntry>> {
    let read_dir = fs::read_dir(&dir).await?;
    let stream = futures::stream::unfold(read_dir, |mut rd| async {
        match rd.next_entry().await {
            Ok(Some(entry)) => Some((entry, rd)),
            Ok(None) => None,
            Err(_) => None, // For simplicity, errors are ignored here.
        }
    });
    let dirs: Vec<DirEntry> = stream
        .map(|entry| async move {
            if let Ok(ft) = entry.file_type().await {
                if ft.is_dir() {
                    Some(entry)
                } else {
                    None
                }
            } else {
                None
            }
        })
        .buffer_unordered(100)
        .filter_map(|x| async move { x })
        .collect()
        .await;
    Ok(dirs)
}

#[tokio::main]
async fn main() -> Result<()> {
    // --- Variant A: Using Tower Services with oneshot chaining ---
    println!("--- Tower Service Pipeline ---");
    let cargo_home = DetermineCargoHome.oneshot(()).await?;
    println!("Cargo home: {:?}", cargo_home);

    let registry_src = GetRegistrySrc.oneshot(cargo_home.clone()).await?;
    println!("Registry src: {:?}", registry_src);

    let registry_dirs = ListDirs.oneshot(registry_src.clone()).await?;
    println!("Found {} registry directories (Tower service).", registry_dirs.len());
    for d in &registry_dirs {
        println!(" - {:?}", d.file_name());
    }

    // --- Variant B: Using a Streaming Pipeline Directly ---
    println!("\n--- Streaming Pipeline ---");
    let stream_dirs = stream_list_dirs(registry_src).await?;
    println!("Found {} registry directories (Streaming pipeline).", stream_dirs.len());
    for d in &stream_dirs {
        println!(" - {:?}", d.file_name());
    }

    Ok(())
}
