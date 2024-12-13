use chrono::Utc;
use clap::Parser;
use color_eyre::eyre::Result;
use diesel::prelude::*;
use diesel::r2d2::ConnectionManager;
use diesel::r2d2::Pool;
use itertools::Itertools;
use nanuak_schema::youtube::video_embeddings_bge_m3;
use nanuak_youtube_embeddings::{load_videos_needing_embeddings, VideoWithLatestWatch};
use ollama_rs::generation::embeddings::request::GenerateEmbeddingsRequest;
use ollama_rs::Ollama;
use std::time::Instant;
use tracing::{debug, error, info};
use tracing::level_filters::LevelFilter;
use tracing_subscriber::EnvFilter;

#[derive(Parser, Debug)]
#[command(version, about = "Populate YouTube Video Embeddings")]
struct Args {
    /// If set, enable debug logging
    #[arg(long)]
    debug: bool,

    /// How many batches to process
    #[arg(long, default_value_t = 1)]
    batches: usize,

    /// How many videos per batch to embed
    #[arg(long, default_value_t = 20)]
    batch_size: usize,

    /// If set, try to optimize batch size for best throughput
    #[arg(long)]
    optimize: bool,
}

/// Build a string representation of the video for embedding
fn build_video_string(video: &VideoWithLatestWatch) -> String {
    // Safely unwrap optionals or provide placeholders
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

    let mut current_batch_size = args.batch_size;
    let mut best_batch_size = current_batch_size;
    let mut best_throughput = 0.0; // items/second
    let mut last_improved = true; // whether we improved last time
    let mut increment = 50; // how much we increase or decrease batch size by

    for batch_idx in 0..args.batches {
        info!(
            "Fetching next batch of videos (batch {}/{}) with batch_size={}",
            batch_idx + 1,
            args.batches,
            current_batch_size
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
            continue;
        }

        // Insert embeddings
        let etags: Vec<String> = videos.iter().map(|v| v.etag.clone()).collect();
        insert_embeddings(&mut conn, &etags, &response.embeddings)?;
        info!("Inserted embeddings for {} videos.", etags.len());

        // Throughput calculation
        let seconds = elapsed.as_secs_f64();
        let throughput = videos.len() as f64 / seconds;
        info!(
            "Batch {}: Processed {} videos in {:.2}s => {:.2} vids/s",
            batch_idx + 1,
            videos.len(),
            seconds,
            throughput
        );

        // If optimize is set, attempt to adjust batch size based on throughput
        if args.optimize && batch_idx + 1 < args.batches {
            if throughput > best_throughput {
                best_throughput = throughput;
                best_batch_size = current_batch_size;
                // Try going bigger, maybe it helps
                current_batch_size = (current_batch_size + increment).min(1000);
                last_improved = true;
            } else {
                // we got worse, revert to best or try smaller
                if last_improved {
                    // We just improved last time, now we got worse
                    // revert to best and try smaller increments
                    current_batch_size = (current_batch_size - increment).max(10);
                    last_improved = false;
                } else {
                    // we got worse again, try reducing increment
                    increment = (increment / 2).max(10);
                    current_batch_size = best_batch_size;
                }
            }
        }
    }

    info!(
        "Finished. Best throughput was {:.2} vids/s at batch size {}",
        best_throughput, best_batch_size
    );

    Ok(())
}
