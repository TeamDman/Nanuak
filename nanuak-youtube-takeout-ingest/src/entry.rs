use chrono::DateTime;
use chrono::Local;
use color_eyre::eyre::Result;
use eyre::bail;
use eyre::Context;
use serde::Deserialize;
use serde_json::Value;
use std::path::PathBuf;

use crate::search_entry::SearchEntry;
use crate::view_post_entry::ViewPostEntry;
use crate::watch_entry::WatchEntry;

#[derive(Debug, Deserialize)]
pub struct RawEntry {
    pub time: DateTime<Local>,
    pub title: String,
    #[serde(rename = "titleUrl")]
    pub title_url: Option<String>,
    pub subtitles: Option<Vec<ActivitySubtitle>>,
}
#[derive(Debug, Deserialize)]
pub struct ActivitySubtitle {
    pub name: String,
    pub url: String,
}

pub async fn load_entries(file: &PathBuf) -> Result<Vec<Entry>> {
    let contents = tokio::fs::read_to_string(file).await?;
    let entries: Value = serde_json::from_str(&contents)?;
    let entries: Vec<RawEntry> = serde_json::from_value(entries)?;
    let entries: Vec<Entry> = entries
        .into_iter()
        .enumerate()
        .filter(|(_, entry)| entry.title != "Used YouTube")
        .filter(|(_, entry)| entry.title != "Viewed a post that is no longer available")
        .map(|(index, entry)| {
            Entry::try_from(entry).map_err(|e| eyre::eyre!("Error in entry {}: {}", index, e))
        })
        .collect::<Result<Vec<_>>>()
        .context(format!("Parsing {}", file.display()))?;
    Ok(entries)
}

#[derive(Debug)]
pub enum Entry {
    Search(SearchEntry),
    Watch(WatchEntry),
    ViewPost(ViewPostEntry),
}

impl TryFrom<RawEntry> for Entry {
    type Error = eyre::Error;

    fn try_from(entry: RawEntry) -> Result<Self> {
        let Some((keyword, _)) = entry.title.split_once(' ') else {
            bail!("Invalid title format: {}", entry.title);
        };
        match keyword {
            "Viewed" => {
                let view_post_entry = ViewPostEntry::try_from(entry)?;
                Ok(Entry::ViewPost(view_post_entry))
            }
            "Searched" => {
                let search_entry = SearchEntry::try_from(entry)?;
                Ok(Entry::Search(search_entry))
            }
            "Watched" => {
                let watch_entry = WatchEntry::try_from(entry)?;
                Ok(Entry::Watch(watch_entry))
            }
            unknown => bail!("Unknown entry type: {}", unknown),
        }
    }
}
