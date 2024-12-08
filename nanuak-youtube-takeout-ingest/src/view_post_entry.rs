use chrono::{DateTime, Local};
use eyre::bail;
use url::Url;

use crate::entry::RawEntry;

#[derive(Debug)]
pub struct ViewPostEntry {
    pub time: DateTime<Local>,
    pub post_title: String,
    pub post_url: String,
    pub channel_url: String,
    pub channel_name: String,
}
impl TryFrom<RawEntry> for ViewPostEntry {
    type Error = eyre::Error;

    fn try_from(entry: RawEntry) -> eyre::Result<Self> {
        let Some(post_url) = entry.title_url else {
            bail!("Missing titleUrl for view post entry: {}", entry.title);
        };
        let Some([_]) = entry.subtitles.as_deref() else {
            bail!("Missing subtitles for view post entry: {}", entry.title);
        };
        let subtitle = entry.subtitles.unwrap().into_iter().next().unwrap();
        let channel_url = subtitle.url;
        let channel_name = subtitle.name;
        Ok(ViewPostEntry {
            time: entry.time,
            post_url,
            post_title: entry.title,
            channel_url,
            channel_name,
        })
    }
}