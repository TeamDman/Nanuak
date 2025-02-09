use eyre::Result;
use futures::future::{BoxFuture, FutureExt};
use futures::stream::{self, StreamExt};
use std::env;
use std::path::PathBuf;
use tokio::fs;
use tokio::fs::DirEntry;
use tower::Service;
use tower::ServiceExt; // requires the "util" feature

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
/// We use a stream to concurrently call file_type() on each DirEntry.
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
                .map(|entry| async move {
                    let ft = entry.file_type().await?;
                    if ft.is_dir() {
                        eyre::Ok(Some(entry))
                    } else {
                        eyre::Ok(None)
                    }
                })
                .buffer_unordered(100)
                .filter_map(|result| async move { result.ok().flatten() })
                .collect()
                .await;
            Ok(dirs)
        }
        .boxed()
    }
}

/// Variant B: A streaming pipeline that directly returns a Vec<DirEntry> from a given directory.
/// We use an unfold stream to yield each entry and then concurrently filter for directories.
async fn stream_list_dirs(dir: PathBuf) -> Result<Vec<DirEntry>> {
    let read_dir = fs::read_dir(&dir).await?;
    let stream = futures::stream::unfold(read_dir, |mut rd| async {
        match rd.next_entry().await {
            Ok(Some(entry)) => Some((entry, rd)),
            Ok(None) => None,
            Err(_) => None, // In a real app, you’d handle errors appropriately.
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
    // --- Variant A: Tower services chained together ---
    println!("--- Tower Service Pipeline ---");
    let cargo_home = DetermineCargoHome.oneshot(()).await?;
    println!("Cargo home: {:?}", cargo_home);

    let registry_src = GetRegistrySrc.oneshot(cargo_home.clone()).await?;
    println!("Registry src: {:?}", registry_src);

    // List the registry directories (first level)
    let registry_dirs = ListDirs.oneshot(registry_src.clone()).await?;
    println!("Found {} registry directories:", registry_dirs.len());
    for d in &registry_dirs {
        println!(" - {:?}", d.file_name());
    }

    // Now, for each registry directory, list its child directories.
    let child_dirs_futures = registry_dirs.into_iter().map(|entry| {
        // entry.path() gives the full path of the registry directory.
        ListDirs.oneshot(entry.path())
    });
    let child_dirs_results = futures::future::join_all(child_dirs_futures).await;
    let mut all_child_dirs = Vec::new();
    for res in child_dirs_results {
        match res {
            Ok(child_dirs) => all_child_dirs.extend(child_dirs),
            Err(err) => eprintln!("Error processing child dirs: {}", err),
        }
    }
    println!("Found {} child directories (crates directories):", all_child_dirs.len());
    for d in &all_child_dirs {
        println!("   * {:?}", d.file_name());
    }

    // --- Variant B: Streaming pipeline approach ---
    println!("\n--- Streaming Pipeline ---");
    // First, use the streaming pipeline to list registry directories.
    let stream_registry_dirs = stream_list_dirs(registry_src).await?;
    println!("(Streaming) Found {} registry directories:", stream_registry_dirs.len());
    for d in &stream_registry_dirs {
        println!(" - {:?}", d.file_name());
    }
    // Then, for each registry directory, concurrently list its children.
    let child_dirs: Vec<DirEntry> = stream::iter(stream_registry_dirs)
        .map(|entry| {
            let path = entry.path();
            async move { stream_list_dirs(path).await }
        })
        .buffer_unordered(10)  // adjust concurrency as needed
        .filter_map(|res: Result<Vec<DirEntry>>| async move { res.ok() })
        .collect::<Vec<Vec<DirEntry>>>()
        .await
        .into_iter()
        .flatten()
        .collect();
    println!("(Streaming) Found {} child directories:", child_dirs.len());
    for d in &child_dirs {
        println!("   * {:?}", d.file_name());
    }

    Ok(())
}
