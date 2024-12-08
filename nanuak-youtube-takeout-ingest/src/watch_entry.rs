use chrono::DateTime;
use chrono::Local;
use chrono::NaiveDateTime;
use eyre::bail;
use url::Url;

use crate::entry::RawEntry;

#[derive(Debug)]
pub struct WatchEntry {
    pub time: NaiveDateTime,
    pub youtube_video_id: String,
}
impl TryFrom<RawEntry> for WatchEntry {
    type Error = eyre::Error;

    fn try_from(entry: RawEntry) -> eyre::Result<Self> {
        let Some(url) = entry.title_url else {
            bail!("Missing titleUrl for watch entry: {}", entry.title);
        };
        let url = Url::parse(&url)?;
        if url.host_str() != Some("www.youtube.com") && url.host_str() != Some("music.youtube.com")
        {
            bail!("Invalid host for watch entry: {}", url);
        }
        if url.path() != "/watch" {
            bail!("Invalid path for watch entry: {}", url);
        }
        let youtube_video_id = url
            .query_pairs()
            .find(|(key, _)| key == "v")
            .map(|(_, value)| value.to_string())
            .ok_or_else(|| eyre::eyre!("Missing 'v' query parameter in URL: {}", url))?;
        Ok(WatchEntry {
            time: entry.time,
            youtube_video_id,
        })
    }
}
