use std::env;
use std::fs;
use std::path::Path;
use std::path::PathBuf;

use eyre::bail;
use futures::future::try_join_all;
use itertools::Itertools;
use tokio::fs::DirEntry;

enum ActionResult<T,C> {
    Terminate(T),
    Next(C),
}

trait Action<T, C> {
    async fn invoke(self) -> eyre::Result<ActionResult<T,C>>;
}

pub struct DetermineCargoHome;
impl Action<PathBuf> for DetermineCargoHome {
    async fn invoke(self) -> eyre::Result<PathBuf> {
        if let Ok(cargo_home) = env::var("CARGO_HOME") {
            Ok(PathBuf::from(cargo_home))
        } else if let Ok(home) = env::var("HOME") {
            Ok(PathBuf::from(home).join(".cargo"))
        } else {
            // On Windows you might also check "USERPROFILE"
            Ok(PathBuf::from(".cargo"))
        }
    }
}

pub struct GetCratesFromCargoHome {
    cargo_home: PathBuf,
}
impl Action<(),()> for GetCratesFromCargoHome {

}




/// Walk through the cargo registry and extract tuples of
/// (registry name, crate name, version, full crate path).
pub async fn get_crates_from_cargo_home(cargo_home: &Path) -> eyre::Result<Vec<DirEntry>> {
    // get the src dir
    let registry_src = cargo_home.join("registry").join("src");
    if !registry_src.exists() {
        bail!("Registry src not found at {:?}", registry_src);
    }

    // get the children, which will be the registry folders
    let registry_dirs = {
        let mut read_dir = tokio::fs::read_dir(&registry_src).await?;
        let mut entries = Vec::new();
        while let Some(entry) = read_dir.next_entry().await? {
            entries.push(entry);
        }
        let entries = entries
            .into_iter()
            .map(|entry| async move {
                let file_type = entry.file_type().await?;
                if !file_type.is_dir() {
                    return Ok(None);
                }
                let date_modified = entry.metadata().await?.modified()?;
                let entry = (entry, date_modified);
                eyre::Ok(Some(entry))
            })
            .collect_vec();
        let entries = try_join_all(entries).await?.into_iter().filter_map(|x| x).collect_vec();

        eyre::Ok(entries)
    }?;

    let crates_dirs: Vec<(PathBuf, Vec<PathBuf>)> = {
        // read the crates in each registry
        
    }

    // let

    todo!()
}

#[tokio::main]
async fn main() -> eyre::Result<()> {
    let cargo_home = get_cargo_home();
    println!("Using CARGO_HOME: {:?}", cargo_home);

    let crates = get_crates_from_cargo_home(&cargo_home).await?;
    println!("Found {} crates:", crates.len());
    for entry in crates {
        println!("{:?}", entry.file_name());
    }
    Ok(())
}
