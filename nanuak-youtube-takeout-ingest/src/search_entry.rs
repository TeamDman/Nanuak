use chrono::NaiveDateTime;
use eyre::bail;
use url::Url;

use crate::entry::RawEntry;

#[derive(Debug)]
pub struct SearchEntry {
    pub time: NaiveDateTime,
    pub query: String,
}
impl TryFrom<RawEntry> for SearchEntry {
    type Error = eyre::Error;

    fn try_from(entry: RawEntry) -> eyre::Result<Self> {
        // Ensure the title starts with "Searched for "
        let Some(query) = entry.title.strip_prefix("Searched for ") else {
            bail!("Invalid search entry: {}", entry.title);
        };

        // Validate the query parameter from titleUrl
        if let Some(title_url) = entry.title_url {
            // Parse the URL and extract the search query parameter
            let url = Url::parse(&title_url)?;
            let query_param = url
                .query_pairs()
                .find(|(key, _)| key == "search_query")
                .map(|(_, value)| value.to_string());

            match query_param {
                Some(decoded_query) if decoded_query == query => {
                    // The title matches the search query
                    Ok(SearchEntry {
                        time: entry.time,
                        query: query.to_string(),
                    })
                }
                Some(decoded_query) => {
                    bail!(
                        "Title query mismatch: title query '{}' != URL query '{}'",
                        query,
                        decoded_query
                    );
                }
                None => {
                    bail!("No search_query parameter found in titleUrl: {}", title_url);
                }
            }
        } else {
            bail!("Missing titleUrl for search entry: {}", entry.title);
        }
    }
}
