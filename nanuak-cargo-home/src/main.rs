use cloud_terrastodon_core_user_input::prelude::Choice;
use cloud_terrastodon_core_user_input::prelude::FzfArgs;
use cloud_terrastodon_core_user_input::prelude::pick_many;
use eyre::Result;
use futures::StreamExt;
use itertools::Itertools;
use std::collections::HashMap;
use std::collections::HashSet;
use std::collections::hash_map::Entry;
use std::env;
use std::ffi::OsString;
use std::path::PathBuf;
use std::time::SystemTime;
use tokio::fs;
use tokio::fs::DirEntry;
use tokio::sync::mpsc::UnboundedReceiver;
use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::mpsc::{self};
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

/// A struct to represent a crate directory along with the modification time of its parent registry directory.
struct CrateEntry {
    entry: DirEntry,
    parent_mod_time: SystemTime,
}

/// Asynchronously scans the given directory and sends each subdirectory as a CrateEntry,
/// attaching the provided parent modification time.
async fn scan_dir_with_parent(
    dir: PathBuf,
    parent_mod_time: SystemTime,
    tx: UnboundedSender<CrateEntry>,
) -> Result<()> {
    let mut read_dir = fs::read_dir(&dir).await?;
    while let Some(entry) = read_dir.next_entry().await? {
        let ft = entry.file_type().await?;
        if ft.is_dir() {
            let crate_entry = CrateEntry {
                entry,
                parent_mod_time,
            };
            if tx.send(crate_entry).is_err() {
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

    // 2. Construct registry/src.
    let registry_src = cargo_home.join("registry").join("src");
    if !registry_src.exists() {
        eyre::bail!("Registry src not found at {:?}", registry_src);
    }
    println!("Registry src: {:?}", registry_src);

    // 3. Create a channel to stream registry directories.
    let (reg_tx, reg_rx): (UnboundedSender<DirEntry>, UnboundedReceiver<DirEntry>) =
        mpsc::unbounded_channel();

    // Spawn a task to scan registry/src for registry directories.
    tokio::spawn(scan_dir(registry_src, reg_tx));

    // Wrap the registry receiver as a stream, and for each registry directory,
    // immediately fetch its modification time.
    let reg_stream = UnboundedReceiverStream::new(reg_rx)
        .then(|entry| async move {
            let meta = entry.metadata().await?;
            let modified = meta.modified()?;
            Ok::<(DirEntry, SystemTime), eyre::Report>((entry, modified))
        })
        .filter_map(|res| async move { res.ok() });

    // 4. Create a channel for crate directories.
    let (crate_tx, crate_rx): (UnboundedSender<CrateEntry>, UnboundedReceiver<CrateEntry>) =
        mpsc::unbounded_channel();

    // Spawn a task that, for each registry directory as soon as it is received,
    // spawns a new scanning task for its children (the crate directories), passing along the parent's modification time.
    let registry_processing = tokio::spawn(async move {
        reg_stream
            .for_each_concurrent(None, |(reg_entry, mod_time)| {
                let path = reg_entry.path();
                let crate_tx = crate_tx.clone();
                async move {
                    if let Err(e) = scan_dir_with_parent(path, mod_time, crate_tx).await {
                        eprintln!("Error scanning child directories: {}", e);
                    }
                }
            })
            .await;
        // When done processing the registry stream, drop the crate sender.
        drop(crate_tx);
    });

    // 5. Wrap the crate receiver as a stream and collect all crate directories into a Vec.
    let crate_processing = async {
        let crate_stream = UnboundedReceiverStream::new(crate_rx);
        // Collect all CrateEntry items.
        crate_stream.collect::<Vec<CrateEntry>>().await
    };

    // Run both tasks concurrently.
    let (registry_result, mut crate_entries) = tokio::join!(registry_processing, crate_processing);
    if let Err(e) = registry_result {
        eprintln!("Error processing registry directories: {}", e);
    }

    // 6. Resolve duplicates: group by the crate directory name and choose the one whose parent modification time is most recent.
    // Build a HashMap where the key is the crate's file name and the value is the CrateEntry
    let mut crate_map: HashMap<OsString, CrateEntry> = HashMap::new();
    for ce in crate_entries {
        let key = ce.entry.file_name();
        match crate_map.entry(key.clone()) {
            Entry::Occupied(mut occ) => {
                if ce.parent_mod_time > occ.get().parent_mod_time {
                    occ.insert(ce);
                }
            }
            Entry::Vacant(vac) => {
                vac.insert(ce);
            }
        }
    }

    // Now collect the unique crate entries.
    let mut unique_crates: Vec<CrateEntry> = crate_map.into_values().collect();

    // Optional: If you need them in a sorted order, you can sort here.
    unique_crates.sort_unstable_by_key(|ce| ce.entry.file_name());

    let seen_names: HashSet<OsString> = unique_crates
        .iter()
        .map(|ce| ce.entry.file_name())
        .collect();
    assert_eq!(seen_names.len(), unique_crates.len());

    println!(
        "Total unique crate directories found: {}",
        unique_crates.len()
    );
    // 7. Use FZF (via pick_many) to let the user select one or more crates.
    let chosen_crates = pick_many(FzfArgs {
        header: Some(format!("Found {} unique crates", unique_crates.len())),
        prompt: Some("Crate to summarize: ".to_string()),
        choices: unique_crates
            .into_iter()
            .map(|ce| Choice {
                key: ce.entry.path().to_string_lossy().to_string(),
                value: ce.entry,
            })
            .collect_vec(),
    })?;

    for crate_entry in chosen_crates {
        println!("Selected crate: {:?}", crate_entry.path());
    }

    Ok(())
}
