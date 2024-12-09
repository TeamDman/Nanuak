use diesel::pg::data_types::PgInterval;
use clap::Parser;
use color_eyre::eyre::Result;
use diesel::prelude::*;
use diesel::r2d2::ConnectionManager;
use diesel::r2d2::Pool;
use itertools::Itertools;
use reqwest::Client;
use serde::Deserialize;
use serde_json::Value;
use tracing::{debug, error, info, warn};
use tracing::level_filters::LevelFilter;
use tracing_subscriber::EnvFilter;

/// For French practice:
/// "Comment dire 'I want to fetch data from the database' en français ?"
/// "Je veux récupérer des données de la base de données."

#[derive(Debug, Deserialize)]
struct YouTubeResponse {
    items: Vec<YouTubeItem>,
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
struct YouTubeItem {
    etag: String,
    id: String,
    contentDetails: ContentDetails,
    snippet: Snippet,
    statistics: Option<Statistics>,
    status: Option<Status>,
    topicDetails: Option<TopicDetails>,
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
struct ContentDetails {
    duration: String,
    caption: Option<String>, // "true" or "false"
    licensedContent: Option<bool>,
    dimension: Option<String>,
    definition: Option<String>,
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
struct Snippet {
    publishedAt: String,
    channelId: String,
    title: String,
    description: String,
    channelTitle: String,
    categoryId: String,
    tags: Option<Vec<String>>,
    thumbnails: Thumbnails,
}

#[derive(Debug, Deserialize)]
struct Thumbnails {
    #[serde(default)]
    default: Option<Thumbnail>,
    #[serde(default)]
    medium: Option<Thumbnail>,
    #[serde(default)]
    high: Option<Thumbnail>,
    #[serde(default)]
    standard: Option<Thumbnail>,
    #[serde(default)]
    maxres: Option<Thumbnail>,
}

#[derive(Debug, Deserialize)]
struct Thumbnail {
    url: String,
    width: Option<i32>,
    height: Option<i32>,
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
struct Statistics {
    viewCount: Option<String>,
    likeCount: Option<String>,
    commentCount: Option<String>,
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
struct Status {
    privacyStatus: String,
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
struct TopicDetails {
    #[serde(default)]
    topicCategories: Vec<String>,
}

/// Command-line arguments for the tool
#[derive(Parser, Debug)]
#[command(version, about = "Populate YouTube Video Details")]
struct Args {
    /// If set, fetch only this single video ID
    #[arg(short, long)]
    single_video_id: Option<String>,
}

/// Convert an ISO8601 duration (e.g., "PT26S") to a SQL interval.
/// For now we do something naive, or we can parse it properly.
fn parse_iso8601_duration_to_interval(duration: &str) -> Option<PgInterval> {
    // For the sake of demonstration, let's just parse the format:
    // For example: "PT26S" means 26 seconds.
    // A full solution would handle minutes/hours. Let's keep it simple.
    // In French: "C'est juste une démonstration simple."
    let trimmed = duration.trim_start_matches('P').trim_start_matches('T');
    // This is a rough and incomplete parser:
    let mut seconds = 0;
    let mut minutes = 0;
    let mut hours = 0;
    // If we see something like 1H2M30S...
    let mut current = String::new();
    for c in trimmed.chars() {
        if c.is_digit(10) {
            current.push(c);
        } else {
            // c might be S, M, H
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
    // Convert to total number of seconds:
    let total_seconds = hours * 3600 + minutes * 60 + seconds;
    // SQL interval format:
    Some(PgInterval {
        months: 0,
        days: 0,
        microseconds: total_seconds * 1_000_000,
    })
}

/// Fetch up to 50 video IDs that we haven't fetched details for yet.
/// We do this by looking at watch_history and left joining videos,
/// selecting those without an entry in videos.
fn get_next_video_ids_to_fetch(conn: &mut PgConnection, limit: i64) -> eyre::Result<Vec<String>> {
    use nanuak_schema::youtube::watch_history::dsl as w;
    use nanuak_schema::youtube::videos::dsl as v;
    use diesel::dsl::max;
    let ids = w::watch_history
        .left_join(v::videos.on(w::youtube_video_id.eq(v::video_id)))
        .filter(v::video_id.is_null())
        .group_by(w::youtube_video_id)
        .order_by(max(w::time).desc())  // Order by the most recent watch time
        .select(w::youtube_video_id)
        .limit(limit)
        .load::<String>(conn)?;

    Ok(ids)
}

/// Fetch video details from YouTube Data API given a list of video IDs.
async fn fetch_video_details(api_key: &str, video_ids: &[String]) -> Result<Vec<YouTubeItem>> {
    if video_ids.is_empty() {
        return Ok(vec![]);
    }

    let client = Client::new();
    // Join video IDs with commas
    let ids = video_ids.iter().join(",");
    let url = format!(
        "https://www.googleapis.com/youtube/v3/videos?part=contentDetails,id,recordingDetails,snippet,statistics,status,topicDetails&id={}&key={}&hl=en",
        ids, api_key
    );
    let response = client.get(&url).send().await?;
    if !response.status().is_success() {
        error!("Failed to fetch video details: {}", response.status());
        return Err(eyre::eyre!("Request failed"));
    }

    let data: Value = response.json().await?;
    debug!("Response:\n{}", serde_json::to_string_pretty(&data)?);
    let data: YouTubeResponse = serde_json::from_value(data)?;
    Ok(data.items)
}

/// Insert the fetched video details into the database.
fn insert_video_details(conn: &mut PgConnection, items: &[YouTubeItem]) -> Result<()> {
    use nanuak_schema::youtube::videos::dsl as v;
    use nanuak_schema::youtube::video_thumbnails::dsl as vt;
    use nanuak_schema::youtube::video_topics::dsl as tp;

    // We'll do inserts in multiple steps.
    // First insert into `videos`.
    // Then insert thumbnails.
    // Then topics.

    // For French practice: "Insertion des données dans la base de données."
    let new_videos: Vec<_> = items.iter().map(|item| {
        let caption_bool = match item.contentDetails.caption.as_deref() {
            Some("true") => Some(true),
            Some("false") => Some(false),
            _ => None,
        };

        let duration_interval = parse_iso8601_duration_to_interval(&item.contentDetails.duration);

        let view_count = item.statistics.as_ref()
            .and_then(|s| s.viewCount.as_ref())
            .and_then(|vc| vc.parse::<i64>().ok());
        let like_count = item.statistics.as_ref()
            .and_then(|s| s.likeCount.as_ref())
            .and_then(|lc| lc.parse::<i64>().ok());
        let comment_count = item.statistics.as_ref()
            .and_then(|s| s.commentCount.as_ref())
            .and_then(|cc| cc.parse::<i64>().ok());

        // Convert publishedAt
        let published_at = chrono::DateTime::parse_from_rfc3339(&item.snippet.publishedAt)
            .ok()
            .map(|dt| dt.with_timezone(&chrono::Utc));

        // We'll rely on Diesel's `now()` for fetched_on, or we can supply chrono::Utc::now()
        let fetched_on = chrono::Utc::now().naive_utc();

        (
            v::etag.eq(&item.etag),
            v::video_id.eq(&item.id),
            v::fetched_on.eq(fetched_on),
            v::title.eq(&item.snippet.title),
            v::description.eq(&item.snippet.description),
            v::published_at.eq(published_at.map(|d| d.naive_utc())),
            v::channel_id.eq(&item.snippet.channelId),
            v::channel_title.eq(&item.snippet.channelTitle),
            v::category_id.eq(&item.snippet.categoryId),
            v::duration.eq(duration_interval),
            v::caption.eq(caption_bool),
            v::definition.eq(item.contentDetails.definition.as_deref()),
            v::dimension.eq(item.contentDetails.dimension.as_deref()),
            v::licensed_content.eq(item.contentDetails.licensedContent),
            v::privacy_status.eq(item.status.as_ref().map(|s| s.privacyStatus.as_str())),
            v::tags.eq(item.snippet.tags.clone()),
            v::view_count.eq(view_count),
            v::like_count.eq(like_count),
            v::comment_count.eq(comment_count),
        )
    }).collect();

    diesel::insert_into(v::videos)
        .values(&new_videos)
        .on_conflict(v::etag)
        .do_nothing()
        .execute(conn)?;

    // Now we need to insert thumbnails. We'll have to re-fetch the etag we inserted.
    // Actually, we have it in item.etag. Just insert them referencing etag.
    // For each item, insert each available thumbnail variant.
    for item in items {
        let thumbnails = [
            ("default", &item.snippet.thumbnails.default),
            ("medium", &item.snippet.thumbnails.medium),
            ("high", &item.snippet.thumbnails.high),
            ("standard", &item.snippet.thumbnails.standard),
            ("maxres", &item.snippet.thumbnails.maxres),
        ];

        let inserts: Vec<_> = thumbnails.iter().filter_map(|(desc, opt_thumb)| {
            opt_thumb.as_ref().map(|thumb| (
                vt::video_etag.eq(&item.etag),
                vt::size_description.eq(*desc),
                vt::height.eq(thumb.height),
                vt::width.eq(thumb.width),
                vt::url.eq(&thumb.url),
            ))
        }).collect();

        if !inserts.is_empty() {
            diesel::insert_into(vt::video_thumbnails)
                .values(&inserts)
                .on_conflict_do_nothing()
                .execute(conn)?;
        }

        // Insert topic categories
        if let Some(topic_details) = &item.topicDetails {
            let topic_inserts: Vec<_> = topic_details.topicCategories.iter().map(|tcat| (
                tp::video_etag.eq(&item.etag),
                tp::topic_url.eq(tcat),
            )).collect();

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

#[tokio::main]
async fn main() -> Result<()> {
    let env_filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::INFO.into())
        .from_env_lossy()
        .add_directive(
            format!("{}=debug", env!("CARGO_PKG_NAME").replace("-", "_"))
                .parse()
                .unwrap(),
        );
    tracing_subscriber::fmt().with_env_filter(env_filter).init();
    color_eyre::install()?;
    info!("Ahoy!");

    let args = Args::parse();

    let manager = ConnectionManager::<PgConnection>::new(std::env::var("DATABASE_URL")?);
    let pool = Pool::builder().build(manager)?;
    let mut conn = pool.get()?;
    info!("Established database connection");

    let api_key = std::env::var("YOUTUBE_API_KEY")?;

    // If single_video_id is specified, fetch just that one.
    let video_ids = if let Some(vid) = args.single_video_id {
        vec![vid]
    } else {
        // Otherwise, get next 50 videos to fetch
        get_next_video_ids_to_fetch(&mut conn, 50)?
    };

    if video_ids.is_empty() {
        warn!("No new video IDs to fetch.");
        return Ok(());
    }

    info!("Fetching details for {} videos", video_ids.len());
    let items = fetch_video_details(&api_key, &video_ids).await?;
    info!("Fetched {} items from the API", items.len());

    if !items.is_empty() {
        insert_video_details(&mut conn, &items)?;
        info!("Inserted video details into the database.");
    }

    Ok(())
}
