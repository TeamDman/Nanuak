use chrono::Utc;
use clap::Parser;
use color_eyre::eyre::Result;
use diesel::prelude::*;
use diesel::r2d2::ConnectionManager;
use diesel::r2d2::Pool;
use itertools::Itertools;
use nanuak_schema::youtube::video_embeddings_bge_m3;
use nanuak_youtube_embeddings::load_videos_needing_embeddings;
use nanuak_youtube_embeddings::VideoWithLatestWatch;
use ollama_rs::generation::embeddings::request::GenerateEmbeddingsRequest;
use ollama_rs::Ollama;
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

    /// How many batches to process
    #[arg(long, default_value_t = 1)]
    batches: usize,

    /// How many videos per batch to embed
    #[arg(long, default_value_t = 20)]
    batch_size: usize,
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

    // We'll insert rows. `video_etag` and `embedding` should match up.
    // embedding is a vector(1024). `diesel` doesn't have built-in pgvector support by default,
    // so you must have pgvector support via a custom type or raw SQL inserts.
    // If you have pgvector configured and integrated with Diesel (like `diesel-pgvector` crate),
    // you can just insert it as a normal field. If not, you may need a custom SQL for insertion.
    //
    // Let's assume you have `pgvector` properly integrated and can insert a `Vec<f32>` directly.
    // If not, you must handle it as a raw SQL. For the sake of this example, we assume you have a custom SQL type for vector.
    //
    // We'll create a temporary Insertable struct:

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

    for batch_idx in 0..args.batches {
        info!(
            "Fetching next batch of videos (batch {}/{})",
            batch_idx + 1,
            args.batches
        );
        let videos = load_videos_needing_embeddings(&mut conn, args.batch_size as i64)?;
        if videos.is_empty() {
            info!("No more videos to embed.");
            break;
        }

        // Build text representations
        let texts: Vec<String> = videos.iter().map(build_video_string).collect();
        debug!("Built embeddings for {} videos", texts.len());

        // Call Ollama embeddings
        info!("Calling Ollama to embed {} videos...", texts.len());
        let request = GenerateEmbeddingsRequest::new("bge-m3:latest".to_string(), texts.into());
        let response = ollama.generate_embeddings(request).await?;
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
    }

    Ok(())
}
