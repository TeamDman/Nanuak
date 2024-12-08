use reqwest::Client;
use serde::Deserialize;
use tracing::error;
use tracing::info;
use tracing::warn;

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
    tracing_subscriber::fmt::init();
    color_eyre::install()?;

    // Load API key from environment variable
    let api_key = std::env::var("YOUTUBE_API_KEY")?;

    // Define video ID
    let video_id = "nmcuoaqdJ9w";

    // Build API URL
    let url = format!(
        "https://www.googleapis.com/youtube/v3/videos?part=contentDetails&id={}&key={}",
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

    // Deserialize JSON response
    let data: YouTubeResponse = response.json().await?;

    // Print the duration
    if let Some(item) = data.items.first() {
        info!("Video url: https://www.youtube.com/watch?v={}", video_id);
        info!("Video duration: {}", item.contentDetails.duration);
    } else {
        warn!("No items found in response");
    }

    Ok(())
}
