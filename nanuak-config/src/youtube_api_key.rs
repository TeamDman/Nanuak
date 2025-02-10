use crate::config_entry::ConfigField;

pub struct YouTubeApiKey;
impl ConfigField for YouTubeApiKey {
    type Value = String;
    fn key() -> &'static str {
        "YOUTUBE_API_KEY"
    }
}