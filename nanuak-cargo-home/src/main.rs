use cloud_terrastodon_core_user_input::prelude::Choice;
use cloud_terrastodon_core_user_input::prelude::FzfArgs;
use cloud_terrastodon_core_user_input::prelude::pick_many;
use eyre::Result;
use futures::stream::StreamExt;
use futures::stream::{self};
use itertools::Itertools;
use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::env;
use std::ffi::OsString;
use std::path::PathBuf;
use std::time::SystemTime;
use tokio::fs;
use tokio::sync::mpsc;
use tokio::sync::mpsc::UnboundedSender;
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

/// Reads all entries in `dir` and, in parallel, checks if each one is a directory.
/// If so, sends its path to `tx`.
async fn scan_subdirs(dir: PathBuf, tx: UnboundedSender<PathBuf>) -> Result<()> {
    let mut read_dir = fs::read_dir(dir).await?;
    let mut entries = Vec::new();

    // Collect all entries first.
    while let Some(entry) = read_dir.next_entry().await? {
        entries.push(entry);
    }

    // Now, concurrently check file types.
    stream::iter(entries)
        .for_each_concurrent(None, |entry| {
            let tx = tx.clone();
            async move {
                if let Ok(ft) = entry.file_type().await {
                    if ft.is_dir() {
                        let _ = tx.send(entry.path());
                    }
                }
            }
        })
        .await;

    Ok(())
}

/// A struct to represent a crate directory along with its own modification time.
struct CrateEntry {
    path: PathBuf,
    crate_mod_time: SystemTime,
}

/// For a given crate directory path, fetch metadata and produce a CrateEntry.
async fn load_crate_entry(path: PathBuf) -> Option<CrateEntry> {
    match fs::metadata(&path).await {
        Ok(md) => match md.modified() {
            Ok(mtime) => Some(CrateEntry {
                path,
                crate_mod_time: mtime,
            }),
            Err(_) => None,
        },
        Err(_) => None,
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // 1. Get Cargo home.
    let cargo_home = get_cargo_home();
    println!("Cargo home: {:?}", cargo_home);

    // 2. Construct registry/src.
    let registry_src = cargo_home.join("registry").join("src");
    if !registry_src.exists() {
        eyre::bail!("Registry src not found at {:?}", registry_src);
    }
    println!("Registry src: {:?}", registry_src);

    // =============================
    // Stage 1: discover registry subdirs
    // =============================
    let (reg_tx, reg_rx) = mpsc::unbounded_channel();

    // We'll spawn a task that scans the top-level registry_src directory.
    // Each subdir is sent to reg_tx.
    let registry_scan_task = tokio::spawn(async move {
        if let Err(e) = scan_subdirs(registry_src, reg_tx).await {
            eprintln!("Error scanning registry src: {}", e);
        }
    });

    // =============================
    // Stage 2: discover crate directories
    // =============================
    let (crate_tx, crate_rx) = mpsc::unbounded_channel();

    // For each registry directory that appears in reg_rx, we scan for crate subdirs.

    let reg_stream = {
        let crate_tx = crate_tx.clone();
        UnboundedReceiverStream::new(reg_rx).for_each_concurrent(None, move |reg_path| {
            let crate_tx = crate_tx.clone();
            async move {
                if let Err(e) = scan_subdirs(reg_path, crate_tx).await {
                    eprintln!("Error scanning crate directories: {}", e);
                }
            }
        })
    };

    // We'll spawn that so it runs concurrently
    let registry_stream_task = tokio::spawn(async move {
        reg_stream.await;
        // After we've consumed all registry dirs, close crate_tx.
        drop(crate_tx);
    });

    // =============================
    // Stage 3: load crate entries
    // =============================
    let (entry_tx, entry_rx) = mpsc::unbounded_channel();

    // For each path from crate_rx, load metadata in parallel.
    let crate_stream = {
        let entry_tx = entry_tx.clone();
        UnboundedReceiverStream::new(crate_rx).for_each_concurrent(None, move |crate_path| {
            let entry_tx = entry_tx.clone();
            async move {
                if let Some(ce) = load_crate_entry(crate_path).await {
                    let _ = entry_tx.send(ce);
                }
            }
        })
    };

    let crate_stream_task = tokio::spawn(async move {
        crate_stream.await;
        drop(entry_tx);
    });

    // =============================
    // Stage 4: collect final CrateEntry items
    // =============================
    // We'll spawn a task to collect all crate entries from entry_rx.
    let collector_task = tokio::spawn(async move {
        // As soon as entry_tx is dropped, the stream terminates.
        let crate_entries: Vec<CrateEntry> = UnboundedReceiverStream::new(entry_rx).collect().await;
        crate_entries
    });

    // Wait for all tasks to finish.
    let (reg_scan_res, reg_stream_res, crate_stream_res, collector_res) = tokio::join!(
        registry_scan_task,
        registry_stream_task,
        crate_stream_task,
        collector_task
    );

    // Log any errors if the tasks themselves panicked or returned Err.
    if let Err(e) = reg_scan_res {
        eprintln!("Registry scan task error: {}", e);
    }
    if let Err(e) = reg_stream_res {
        eprintln!("Registry stream task error: {}", e);
    }
    if let Err(e) = crate_stream_res {
        eprintln!("Crate stream task error: {}", e);
    }

    // The collector task returns our final Vec<CrateEntry>.
    let crate_entries = match collector_res {
        Ok(entries) => entries,
        Err(e) => {
            eprintln!("Collector task error: {}", e);
            vec![]
        }
    };

    // 6. Resolve duplicates by crate directory name, picking the latest mod time.
    let mut crate_map: HashMap<OsString, CrateEntry> = HashMap::new();
    for ce in crate_entries {
        // Convert to an owned OsString so the sort later won't borrow from 'ce'.
        let key = ce.path.file_name().unwrap_or_default().to_os_string();

        match crate_map.entry(key) {
            Entry::Occupied(mut occ) => {
                if ce.crate_mod_time > occ.get().crate_mod_time {
                    occ.insert(ce);
                }
            }
            Entry::Vacant(vac) => {
                vac.insert(ce);
            }
        }
    }

    let mut unique_crates: Vec<CrateEntry> = crate_map.into_values().collect();

    // Convert file_name to an owned type in the sort closure to avoid lifetime errors
    unique_crates.sort_unstable_by_key(|ce| ce.path.file_name().unwrap_or_default().to_os_string());

    println!(
        "Total unique crate directories found: {}",
        unique_crates.len()
    );

    // 7. Let the user select crates with fzf.
    let chosen_crates = pick_many(FzfArgs {
        header: Some(format!("Found {} unique crates", unique_crates.len())),
        prompt: Some("Crate to summarize: ".to_string()),
        choices: unique_crates
            .into_iter()
            .map(|ce| Choice {
                key: ce.path.to_string_lossy().to_string(),
                value: ce.path.clone(),
            })
            .collect_vec(),
    })?;

    for crate_path in chosen_crates {
        println!("Selected crate: {:?}", crate_path);
    }

    Ok(())
}
