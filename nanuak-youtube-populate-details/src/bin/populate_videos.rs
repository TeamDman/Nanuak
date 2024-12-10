use std::collections::HashSet;

use clap::Parser;
use color_eyre::eyre::Result;
use diesel::pg::data_types::PgInterval;
use diesel::prelude::*;
use diesel::r2d2::ConnectionManager;
use diesel::r2d2::Pool;
use futures::stream;
use futures::StreamExt;
use futures::TryStreamExt;
use itertools::Itertools;
use nanuak_schema::youtube;
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
    #[serde(rename = "etag")]
    etag: String,
    #[serde(rename = "id")]
    id: String,
    #[serde(rename = "contentDetails")]
    content_details: ContentDetails,
    #[serde(rename = "snippet")]
    snippet: Snippet,
    #[serde(rename = "statistics")]
    statistics: Option<Statistics>,
    #[serde(rename = "status")]
    status: Option<Status>,
    #[serde(rename = "topicDetails")]
    topic_details: Option<TopicDetails>,
}

#[derive(Debug, Deserialize)]
struct ContentDetails {
    #[serde(rename = "duration")]
    duration: String,
    #[serde(rename = "caption")]
    caption: Option<String>, // "true" or "false"
    #[serde(rename = "licensedContent")]
    licensed_content: Option<bool>,
    #[serde(rename = "dimension")]
    dimension: Option<String>,
    #[serde(rename = "definition")]
    definition: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Snippet {
    #[serde(rename = "publishedAt")]
    published_at: String,
    #[serde(rename = "channelId")]
    channel_id: String,
    #[serde(rename = "title")]
    title: String,
    #[serde(rename = "description")]
    description: String,
    #[serde(rename = "channelTitle")]
    channel_title: String,
    #[serde(rename = "categoryId")]
    category_id: String,
    #[serde(rename = "tags")]
    tags: Option<Vec<String>>,
    #[serde(rename = "thumbnails")]
    thumbnails: Thumbnails,
}

#[derive(Debug, Deserialize)]
struct Thumbnails {
    #[serde(rename = "default", default)]
    default: Option<Thumbnail>,
    #[serde(rename = "medium", default)]
    medium: Option<Thumbnail>,
    #[serde(rename = "high", default)]
    high: Option<Thumbnail>,
    #[serde(rename = "standard", default)]
    standard: Option<Thumbnail>,
    #[serde(rename = "maxres", default)]
    maxres: Option<Thumbnail>,
}

#[derive(Debug, Deserialize)]
struct Thumbnail {
    #[serde(rename = "url")]
    url: String,
    #[serde(rename = "width")]
    width: Option<i32>,
    #[serde(rename = "height")]
    height: Option<i32>,
}

#[derive(Debug, Deserialize)]
struct Statistics {
    #[serde(rename = "viewCount")]
    view_count: Option<String>,
    #[serde(rename = "likeCount")]
    like_count: Option<String>,
    #[serde(rename = "commentCount")]
    comment_count: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Status {
    #[serde(rename = "privacyStatus")]
    privacy_status: String,
}

#[derive(Debug, Deserialize)]
struct TopicDetails {
    #[serde(rename = "topicCategories", default)]
    topic_categories: Vec<String>,
}

/// Command-line arguments for the tool
#[derive(Parser, Debug)]
#[command(version, about = "Populate YouTube Video Details")]
struct Args {
    /// If set, fetch only this single video ID
    #[arg(short, long)]
    single_video_id: Option<String>,

    /// If set, enable debug logging
    #[arg(long)]
    debug: bool,

    /// How many pages of videos to fetch (each page is `--page-size`)
    #[arg(long, default_value_t = 1)]
    pages: u32,

    /// How many videos per page to fetch
    #[arg(long, default_value_t = 50)]
    page_size: usize,

    /// How many pages to fetch concurrently
    #[arg(long, default_value_t = 1)]
    concurrency_limit: usize,
}

/// Convert an ISO8601 duration (e.g., "PT26S") to a SQL interval.
fn parse_iso8601_duration_to_interval(duration: &str) -> Option<PgInterval> {
    let trimmed = duration.trim_start_matches('P').trim_start_matches('T');
    let mut seconds = 0;
    let mut minutes = 0;
    let mut hours = 0;
    let mut current = String::new();
    for c in trimmed.chars() {
        if c.is_ascii_digit() {
            current.push(c);
        } else {
            let val: i64 = current.parse().unwrap_or(0);
            current.clear();
            match c {
                'H' => hours = val,
                'M' => minutes = val,
                'S' => seconds = val,
                _ => {}
            }
        }
    }
    let total_seconds = hours * 3600 + minutes * 60 + seconds;
    Some(PgInterval {
        months: 0,
        days: 0,
        microseconds: total_seconds * 1_000_000,
    })
}

/// Fetch up to `count` video IDs that we haven't fetched details for yet.
fn get_next_video_ids_to_fetch(
    conn: &mut PgConnection,
    count: i64,
) -> color_eyre::Result<Vec<String>> {
    use diesel::dsl::max;
    use diesel::prelude::*;
    use nanuak_schema::youtube::missing_videos as mv;
    use nanuak_schema::youtube::videos as v;
    use nanuak_schema::youtube::watch_history as w;
    use nanuak_schema::youtube::{self};

    let ids = w::table
        .left_join(v::table.on(w::youtube_video_id.eq(v::video_id)))
        // Exclude missing videos
        .filter(v::video_id.is_null())
        .filter(w::youtube_video_id.ne_all(mv::table.select(mv::video_id)))
        .group_by(w::youtube_video_id)
        .order_by(max(w::time).desc())
        .select(w::youtube_video_id)
        .limit(count)
        .load::<String>(conn)?;

    Ok(ids)
}

/// Fetch video details from YouTube Data API given a list of video IDs.
async fn fetch_video_details(api_key: &str, video_ids: &[String]) -> Result<Vec<YouTubeItem>> {
    if video_ids.is_empty() {
        return Ok(vec![]);
    }

    let client = Client::new();
    let ids = video_ids.iter().join(",");
    debug!("Fetching videos with IDs: {}", ids);
    let url = format!(
        "https://www.googleapis.com/youtube/v3/videos?part=contentDetails,id,recordingDetails,snippet,statistics,status,topicDetails&id={}&key={}&hl=en",
        ids, api_key
    );
    let response = client.get(&url).send().await?;
    if !response.status().is_success() {
        error!("Failed to fetch video details: {}", response.status());
        return Err(eyre::eyre!("Request failed"));
    }

    debug!("Response headers: {:#?}", response.headers());
    let data: Value = response.json().await?;
    debug!("Response:\n{}", serde_json::to_string_pretty(&data)?);
    let data: YouTubeResponse = serde_json::from_value(data)?;
    Ok(data.items)
}

/// Insert the fetched video details into the database.
fn insert_video_details(conn: &mut PgConnection, items: &[YouTubeItem]) -> Result<()> {
    use youtube::video_thumbnails::dsl as vt;
    use youtube::video_topics::dsl as tp;
    use youtube::videos::dsl as v;

    let new_videos: Vec<_> = items
        .iter()
        .map(|item| {
            let caption_bool = match item.content_details.caption.as_deref() {
                Some("true") => Some(true),
                Some("false") => Some(false),
                _ => None,
            };

            let duration_interval =
                parse_iso8601_duration_to_interval(&item.content_details.duration);

            let view_count = item
                .statistics
                .as_ref()
                .and_then(|s| s.view_count.as_ref())
                .and_then(|vc| vc.parse::<i64>().ok());
            let like_count = item
                .statistics
                .as_ref()
                .and_then(|s| s.like_count.as_ref())
                .and_then(|lc| lc.parse::<i64>().ok());
            let comment_count = item
                .statistics
                .as_ref()
                .and_then(|s| s.comment_count.as_ref())
                .and_then(|cc| cc.parse::<i64>().ok());

            let published_at = chrono::DateTime::parse_from_rfc3339(&item.snippet.published_at)
                .ok()
                .map(|dt| dt.with_timezone(&chrono::Utc));
            let fetched_on = chrono::Utc::now().naive_utc();

            (
                v::etag.eq(&item.etag),
                v::video_id.eq(&item.id),
                v::fetched_on.eq(fetched_on),
                v::title.eq(&item.snippet.title),
                v::description.eq(&item.snippet.description),
                v::published_at.eq(published_at.map(|d| d.naive_utc())),
                v::channel_id.eq(&item.snippet.channel_id),
                v::channel_title.eq(&item.snippet.channel_title),
                v::category_id.eq(&item.snippet.category_id),
                v::duration.eq(duration_interval),
                v::caption.eq(caption_bool),
                v::definition.eq(item.content_details.definition.as_deref()),
                v::dimension.eq(item.content_details.dimension.as_deref()),
                v::licensed_content.eq(item.content_details.licensed_content),
                v::privacy_status.eq(item.status.as_ref().map(|s| s.privacy_status.as_str())),
                v::tags.eq(item.snippet.tags.clone()),
                v::view_count.eq(view_count),
                v::like_count.eq(like_count),
                v::comment_count.eq(comment_count),
            )
        })
        .collect();

    diesel::insert_into(v::videos)
        .values(&new_videos)
        .on_conflict(v::etag)
        .do_nothing()
        .execute(conn)?;

    for item in items {
        let thumbnails = [
            ("default", &item.snippet.thumbnails.default),
            ("medium", &item.snippet.thumbnails.medium),
            ("high", &item.snippet.thumbnails.high),
            ("standard", &item.snippet.thumbnails.standard),
            ("maxres", &item.snippet.thumbnails.maxres),
        ];

        let inserts: Vec<_> = thumbnails
            .iter()
            .filter_map(|(desc, opt_thumb)| {
                opt_thumb.as_ref().map(|thumb| {
                    (
                        vt::video_etag.eq(&item.etag),
                        vt::size_description.eq(*desc),
                        vt::height.eq(thumb.height),
                        vt::width.eq(thumb.width),
                        vt::url.eq(&thumb.url),
                    )
                })
            })
            .collect();

        if !inserts.is_empty() {
            diesel::insert_into(vt::video_thumbnails)
                .values(&inserts)
                .on_conflict_do_nothing()
                .execute(conn)?;
        }

        if let Some(topic_details) = &item.topic_details {
            let topic_inserts: Vec<_> = topic_details
                .topic_categories
                .iter()
                .map(|tcat| (tp::video_etag.eq(&item.etag), tp::topic_url.eq(tcat)))
                .collect();

            if !topic_inserts.is_empty() {
                diesel::insert_into(tp::video_topics)
                    .values(&topic_inserts)
                    .on_conflict_do_nothing()
                    .execute(conn)?;
            }
        }
    }

    Ok(())
}

fn insert_missing_videos(conn: &mut PgConnection, missing_ids: &[String]) -> Result<()> {
    use diesel::prelude::*;
    use nanuak_schema::youtube::missing_videos as mv;
    let fetched_on = chrono::Utc::now().naive_utc();

    let inserts: Vec<_> = missing_ids
        .iter()
        .map(|video_id| (mv::video_id.eq(video_id), mv::fetched_on.eq(fetched_on)))
        .collect();

    diesel::insert_into(mv::table)
        .values(&inserts)
        .on_conflict_do_nothing()
        .execute(conn)?;

    Ok(())
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
    info!("Ahoy!");

    let manager = ConnectionManager::<PgConnection>::new(std::env::var("DATABASE_URL")?);
    let pool = Pool::builder().build(manager)?;
    let mut conn = pool.get()?;
    info!("Established database connection");

    let api_key = std::env::var("YOUTUBE_API_KEY")?;

    // If single_video_id is specified, fetch just that one.
    if let Some(vid) = args.single_video_id {
        info!("Fetching single video ID: {}", vid);
        let items = fetch_video_details(&api_key, &[vid]).await?;
        insert_video_details(&mut conn, &items)?;
        info!("Inserted single video detail.");
        return Ok(());
    }

    // Calculate total number of videos we want
    let total_videos = args.pages as i64 * args.page_size as i64;
    info!(
        "Fetching up to {} videos ({} pages of {} each)",
        total_videos, args.pages, args.page_size
    );

    // Fetch all video IDs at once
    let video_ids = get_next_video_ids_to_fetch(&mut conn, total_videos)?;
    if video_ids.is_empty() {
        warn!("No new video IDs to fetch.");
        return Ok(());
    }

    // Chunk into pages
    let pages: Vec<Vec<String>> = video_ids
        .chunks(args.page_size)
        .map(|chunk| chunk.to_vec())
        .collect();

    info!("Got {} pages of video IDs", pages.len());

    // Process pages in parallel with concurrency limit
    // We'll use futures::stream::iter and try_for_each_concurrent
    // Each page fetches details and inserts them
    let pool = pool.clone(); // clone for move into async tasks
    stream::iter(pages.into_iter().enumerate())
        .map(Ok)
        .try_for_each_concurrent(args.concurrency_limit, |(idx, page_ids)| {
            let api_key = api_key.clone();
            let pool = pool.clone();
            async move {
                info!("Processing page {} with {} videos", idx + 1, page_ids.len());
                let items = fetch_video_details(&api_key, &page_ids).await?;
                info!(
                    "Fetched {} items from the API for page {}",
                    items.len(),
                    idx + 1
                );

                // Determine missing videos
                let returned_ids: HashSet<_> = items.iter().map(|i| i.id.clone()).collect();
                let missing_ids: Vec<_> = page_ids
                    .iter()
                    .filter(|id| !returned_ids.contains(*id))
                    .cloned()
                    .collect();

                if !missing_ids.is_empty() {
                    warn!(
                        "Detected {} unavailable videos on page {}: {:?}",
                        missing_ids.len(),
                        idx + 1,
                        missing_ids
                    );
                    let mut conn = pool.get()?;
                    insert_missing_videos(&mut conn, &missing_ids)?;
                }

                if !items.is_empty() {
                    let mut conn = pool.get()?;
                    insert_video_details(&mut conn, &items)?;
                    info!(
                        "Inserted video details into the database for page {}",
                        idx + 1
                    );
                }
                Ok::<(), eyre::Error>(())
            }
        })
        .await?;

    Ok(())
}
