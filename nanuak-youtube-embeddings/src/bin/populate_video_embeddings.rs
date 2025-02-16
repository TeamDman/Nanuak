use chrono::Utc;
use clap::Parser;
use color_eyre::eyre::Result;
use diesel::prelude::*;
use diesel::r2d2::ConnectionManager;
use diesel::r2d2::Pool;
use itertools::Itertools;
use nanuak_schema::youtube::video_embeddings_bge_m3;
use nanuak_youtube_embeddings::count_videos_needing_embeddings;
use nanuak_youtube_embeddings::load_videos_needing_embeddings;
use nanuak_youtube_embeddings::VideoWithLatestWatch;
use ollama_rs::generation::embeddings::request::GenerateEmbeddingsRequest;
use ollama_rs::Ollama;
use std::time::Instant;
use tracing::debug;
use tracing::error;
use tracing::info;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::EnvFilter;

#[derive(Parser, Debug)]
#[command(version, about = "Populate YouTube Video Embeddings")]
struct Args {
    /// If set, enable debug logging
    #[arg(long)]
    debug: bool,

    /// How many batches to process. If not provided, run until no more videos remain.
    #[arg(long)]
    batches: Option<usize>,

    /// How many videos per batch to embed
    #[arg(long, default_value_t = 20)]
    batch_size: usize,

    /// If set, try to optimize batch size for best throughput
    #[arg(long)]
    optimize: bool,
}

/// Build a string representation of the video for embedding
fn build_video_string(video: &VideoWithLatestWatch) -> String {
    let title = video.title.as_deref().unwrap_or("Unknown Title");
    let channel = video.channel_title.as_deref().unwrap_or("Unknown Channel");
    let category = video
        .category_title
        .as_deref()
        .unwrap_or("Unknown Category");
    let description = video.description.as_deref().unwrap_or("No Description");
    let tags = video
        .tags
        .as_ref()
        .map(|ts| ts.iter().flatten().join(", "))
        .unwrap_or("No Tags".to_string());
    let view_count = video
        .view_count
        .map(|v| v.to_string())
        .unwrap_or("Unknown".to_string());
    let like_count = video
        .like_count
        .map(|v| v.to_string())
        .unwrap_or("Unknown".to_string());
    let comment_count = video
        .comment_count
        .map(|v| v.to_string())
        .unwrap_or("Unknown".to_string());

    format!("Title: {title}\nChannel: {channel}\nCategory: {category}\nDescription: {description}\nTags: {tags}\nView Count: {view_count}\nLike Count: {like_count}\nComment Count: {comment_count}\n")
}

/// Insert embeddings into the database
fn insert_embeddings(
    conn: &mut PgConnection,
    etags: &[String],
    embeddings: &[Vec<f32>],
) -> Result<()> {
    use video_embeddings_bge_m3::dsl as vemb;
    let now = Utc::now().naive_utc();

    #[derive(Insertable)]
    #[diesel(table_name = video_embeddings_bge_m3)]
    struct NewEmbedding<'a> {
        video_etag: &'a str,
        embedded_on: chrono::NaiveDateTime,
        embedding: Option<pgvector::Vector>,
    }

    let insert_data: Vec<_> = etags
        .iter()
        .zip(embeddings)
        .map(|(etag, emb)| NewEmbedding {
            video_etag: etag,
            embedded_on: now,
            embedding: Some(pgvector::Vector::from(emb.clone())),
        })
        .collect();

    diesel::insert_into(vemb::video_embeddings_bge_m3)
        .values(&insert_data)
        .on_conflict_do_nothing()
        .execute(conn)?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Setup logging
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

    info!("Connecting to database...");
    let manager = ConnectionManager::<PgConnection>::new(std::env::var("DATABASE_URL")?);
    let pool = Pool::builder().build(manager)?;

    let ollama = Ollama::default();
    let mut conn = pool.get()?;

    // Count how many videos are remaining at the start
    let mut remaining = count_videos_needing_embeddings(&mut conn)?;
    if remaining == 0 {
        info!("No videos need embeddings. Exiting.");
        return Ok(());
    }

    info!("{} videos remain to be embedded.", remaining);

    let mut current_batch_size = args.batch_size;
    let mut best_batch_size = current_batch_size;
    let mut best_throughput = 0.0;
    let mut last_improved = true;
    let mut increment = 50;

    // Determine how many batches we run:
    // If batches is None, run until no more videos remain.
    let total_batches = args.batches.unwrap_or(usize::MAX);
    let total_batches_str = if total_batches == usize::MAX {
        "-∞-".to_string()
    } else {
        total_batches.to_string()
    };

    let mut batch_count = 0;

    while batch_count < total_batches {
        if remaining == 0 {
            info!("No more videos to embed.");
            break;
        }

        info!(
            "Fetching next batch of videos (batch {}/{}) with batch_size={} ({} remain)",
            batch_count + 1,
            total_batches_str,
            current_batch_size,
            remaining
        );

        let videos = load_videos_needing_embeddings(&mut conn, current_batch_size as i64)?;
        if videos.is_empty() {
            info!("No more videos to embed.");
            break;
        }

        // Build text representations
        let texts: Vec<String> = videos.iter().map(build_video_string).collect();
        debug!("Built embeddings for {} videos", texts.len());

        // Call Ollama embeddings
        info!("Calling Ollama to embed {} videos...", texts.len());
        let start = Instant::now();
        let request = GenerateEmbeddingsRequest::new("bge-m3:latest".to_string(), texts.into());
        let response = ollama.generate_embeddings(request).await?;
        let elapsed = start.elapsed();
        if response.embeddings.len() != videos.len() {
            error!(
                "Mismatch in embeddings count. Expected {} got {}",
                videos.len(),
                response.embeddings.len()
            );
            // We'll continue, but this batch might be messed up
        }

        // Insert embeddings
        let etags: Vec<String> = videos.iter().map(|v| v.etag.clone()).collect();
        insert_embeddings(&mut conn, &etags, &response.embeddings)?;
        info!("Inserted embeddings for {} videos.", etags.len());

        // Update counts
        let processed = etags.len();
        let seconds = elapsed.as_secs_f64();
        let throughput = processed as f64 / seconds;
        batch_count += 1;

        // Recount how many remain
        remaining = count_videos_needing_embeddings(&mut conn)?;
        info!(
            "Batch {}: Processed {} videos in {:.2}s => {:.2} vids/s. {} remain.",
            batch_count, processed, seconds, throughput, remaining
        );

        if throughput > 0.0 && remaining > 0 {
            let eta_sec = remaining as f64 / throughput;
            let eta_min = eta_sec / 60.0;
            info!(
                "ETA: {:.2}s (~{:.1}m) at current throughput.",
                eta_sec, eta_min
            );
        }

        // If optimize is set, attempt to adjust batch size based on throughput
        if args.optimize && batch_count < total_batches && processed > 0 {
            if throughput > best_throughput {
                best_throughput = throughput;
                best_batch_size = current_batch_size;
                current_batch_size = (current_batch_size + increment).min(1000);
                last_improved = true;
            } else if last_improved {
                current_batch_size = (current_batch_size - increment).max(10);
                last_improved = false;
            } else {
                increment = (increment / 2).max(10);
                current_batch_size = best_batch_size;
            }
        }

        if remaining == 0 {
            info!("All videos embedded!");
            break;
        }
    }

    info!(
        "Finished. Best throughput was {:.2} vids/s at batch size {}",
        best_throughput, best_batch_size
    );

    Ok(())
}
