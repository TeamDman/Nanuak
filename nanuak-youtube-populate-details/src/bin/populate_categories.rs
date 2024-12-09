use clap::Parser;
use color_eyre::eyre::Result;
use diesel::prelude::*;
use diesel::r2d2::ConnectionManager;
use diesel::r2d2::Pool;
use reqwest::Client;
use serde::Deserialize;
use serde_json::Value;
use tracing::{debug, error, info, warn};
use tracing::level_filters::LevelFilter;
use tracing_subscriber::EnvFilter;

use nanuak_schema::youtube::video_categories;

#[derive(Debug, Deserialize)]
struct YouTubeCategoryResponse {
    items: Vec<YouTubeCategoryItem>,
}

#[derive(Debug, Deserialize)]
struct YouTubeCategoryItem {
    id: String,
    snippet: CategorySnippet,
}

#[derive(Debug, Deserialize)]
struct CategorySnippet {
    title: String,
    assignable: bool,
    #[serde(rename = "channelId")]
    channel_id: String,
}

/// Insertable struct for `video_categories` table
#[derive(Insertable)]
#[diesel(table_name = video_categories)]
struct NewVideoCategory {
    id: String,
    title: String,
    assignable: bool,
    channel_id: String,
}

/// Command-line arguments for the tool
#[derive(Parser, Debug)]
#[command(version, about = "Populate YouTube Video Categories")]
struct Args {
    /// If set, enable debug logging
    #[arg(long)]
    debug: bool,

    /// Region code (e.g. CA, US)
    #[arg(long, default_value = "CA")]
    region_code: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Adjust logging based on `--debug` flag
    let log_level = if args.debug {
        LevelFilter::DEBUG
    } else {
        LevelFilter::INFO
    };
    let env_filter = EnvFilter::builder()
        .with_default_directive(log_level.into())
        .from_env_lossy();
    tracing_subscriber::fmt().with_env_filter(env_filter).init();

    color_eyre::install()?;
    info!("Starting to populate categories");

    let manager = ConnectionManager::<PgConnection>::new(std::env::var("DATABASE_URL")?);
    let pool = Pool::builder().build(manager)?;
    let mut conn = pool.get()?;
    info!("Established database connection");

    let api_key = std::env::var("YOUTUBE_API_KEY")?;
    let categories = fetch_video_categories(&api_key, &args.region_code).await?;
    if categories.is_empty() {
        warn!("No categories fetched.");
        return Ok(());
    }

    insert_video_categories(&mut conn, &categories)?;
    info!("Inserted {} categories into the database.", categories.len());

    Ok(())
}

async fn fetch_video_categories(api_key: &str, region_code: &str) -> Result<Vec<NewVideoCategory>> {
    let client = Client::new();
    let url = format!(
        "https://www.googleapis.com/youtube/v3/videoCategories?regionCode={}&part=snippet&key={}",
        region_code, api_key
    );

    let response = client.get(&url).send().await?;
    if !response.status().is_success() {
        error!("Failed to fetch video categories: {}", response.status());
        return Err(eyre::eyre!("Request failed"));
    }

    let data: Value = response.json().await?;
    debug!("Response:\n{}", serde_json::to_string_pretty(&data)?);
    let data: YouTubeCategoryResponse = serde_json::from_value(data)?;

    let categories: Vec<NewVideoCategory> = data.items.into_iter().map(|item| {
        NewVideoCategory {
            id: item.id,
            title: item.snippet.title,
            assignable: item.snippet.assignable,
            channel_id: item.snippet.channel_id,
        }
    }).collect();

    Ok(categories)
}

fn insert_video_categories(conn: &mut PgConnection, categories: &[NewVideoCategory]) -> Result<()> {
    use video_categories::dsl as vc;

    diesel::insert_into(vc::video_categories)
        .values(categories)
        .on_conflict_do_nothing()
        .execute(conn)?;
    Ok(())
}
