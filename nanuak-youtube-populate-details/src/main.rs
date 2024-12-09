use itertools::Itertools;
use reqwest::Client;
use serde::Deserialize;
use serde_json::Value;
use tracing::debug;
use tracing::error;
use tracing::info;
use tracing::level_filters::LevelFilter;
use tracing::warn;
use tracing_subscriber::EnvFilter;

#[derive(Debug, Deserialize)]
struct YouTubeResponse {
    items: Vec<YouTubeItem>,
}

#[derive(Debug, Deserialize)]
struct YouTubeItem {
    contentDetails: ContentDetails,
}

#[derive(Debug, Deserialize)]
struct ContentDetails {
    duration: String,
}

#[tokio::main]
async fn main() -> eyre::Result<()> {
    let env_filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::INFO.into())
        .from_env_lossy()
        .add_directive(
            format!(
                "
                {}=debug
                ",
                env!("CARGO_PKG_NAME").replace("-", "_")
            )
            .lines()
            .map(|line| line.trim())
            .filter(|line| !line.starts_with("//"))
            .filter(|line| !line.is_empty())
            .join(",")
            .trim()
            .parse()
            .unwrap(),
        );
    tracing_subscriber::fmt().with_env_filter(env_filter).init();

    color_eyre::install()?;

    // Load API key from environment variable
    let api_key = std::env::var("YOUTUBE_API_KEY")?;

    // Define video ID
    let video_id = "nmcuoaqdJ9w";

    // Build API URL
    let url = format!(
        // "https://www.googleapis.com/youtube/v3/videos?part=contentDetails&id={}&key={}",
        "https://www.googleapis.com/youtube/v3/videos?part=contentDetails,id,localizations,recordingDetails,snippet,statistics,status,topicDetails&id={}&key={}&hl=en",
        video_id, api_key
    );

    // Create an HTTP client
    let client = Client::new();

    // Send GET request
    let response = client.get(&url).send().await?;

    // Ensure the response is successful
    if !response.status().is_success() {
        error!("Failed to fetch video details: {}", response.status());
        return Err(eyre::eyre!("Request failed"));
    }

    // Receive JSON response
    let data: Value = response.json().await?;
    // Display it pretty
    debug!("Response:\n{}", serde_json::to_string_pretty(&data)?);
    // Convert to expected format
    let data: YouTubeResponse = serde_json::from_value(data)?;

    // Print the duration
    if let Some(item) = data.items.first() {
        info!("Video url: https://www.youtube.com/watch?v={}", video_id);
        info!("Video duration: {}", item.contentDetails.duration);
    } else {
        warn!("No items found in response");
    }

    Ok(())
}
