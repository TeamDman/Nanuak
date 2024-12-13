use clap::Parser;
use color_eyre::eyre::Result;
use diesel::prelude::*;
use diesel::r2d2::ConnectionManager;
use diesel::r2d2::Pool;
use diesel::sql_types::Float8;
use diesel::sql_types::Text;
use ollama_rs::generation::embeddings::request::GenerateEmbeddingsRequest;
use ollama_rs::Ollama;
use pgvector::Vector;
use std::io::Write;
use std::io::{self};
use tracing::error;
use tracing::info;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::EnvFilter;

/// Command-line arguments for the search tool
#[derive(Parser, Debug)]
#[command(version, about = "Search YouTube Video Embeddings")]
struct Args {
    /// If set, enable debug logging
    #[arg(long)]
    debug: bool,

    /// Embedding model to use for queries
    #[arg(long, default_value = "bge-m3:latest")]
    model: String,
}

/// Struct to hold query results
#[derive(QueryableByName, Debug)]
struct SearchResult {
    #[diesel(sql_type = Text)]
    title: String,
    #[diesel(sql_type = Text)]
    video_id: String,
    #[diesel(sql_type = Float8)]
    distance: f64,
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

    loop {
        print!("Enter query (q to quit): ");
        io::stdout().flush()?;
        let mut query = String::new();
        io::stdin().read_line(&mut query)?;
        let query = query.trim();
        if query.is_empty() || query == "q" {
            break;
        }

        // Generate embedding for the query
        info!("Embedding the query...");
        let request = GenerateEmbeddingsRequest::new(args.model.clone(), vec![query].into());
        let response = ollama.generate_embeddings(request).await?;
        if response.embeddings.is_empty() {
            error!("No embeddings returned for query.");
            continue;
        }
        let query_embedding = &response.embeddings[0];

        // Perform vector similarity search:
        // We can use a query like:
        //
        // SELECT v.title, v.video_id, (e.embedding <-> $1) as distance
        // FROM youtube.videos v
        // JOIN youtube.video_embeddings_bge_m3 e ON v.etag = e.video_etag
        // ORDER BY e.embedding <-> $1
        // LIMIT 10;
        //
        // We'll pass the embedding as a parameter.
        //
        // Make sure your pgvector integration allows binding a Vec<f32> as a vector parameter.
        // If not, you might need custom code. We'll assume you have `pgvector` crate integrated.

        let mut conn = pool.get()?;
        let sql = r#"
            SELECT v.title, v.video_id, (e.embedding <-> $1) as distance
            FROM youtube.videos v
            JOIN youtube.video_embeddings_bge_m3 e ON v.etag = e.video_etag
            ORDER BY e.embedding <-> $1
            LIMIT 10
        "#;

        // We'll assume we have implemented From<Vec<f32>> for Vector or can do Vector::from
        let vec_emb = Vector::from(query_embedding.clone());
        let results = diesel::sql_query(sql)
            .bind::<pgvector::sql_types::Vector, _>(vec_emb) // binding the vector parameter
            .load::<SearchResult>(&mut conn)?;

        println!("Top 10 results:");
        for (i, r) in results.iter().enumerate() {
            println!(
                "{}. Title: {}\n   Video ID: https://youtube.com/watch?v={}\n   Distance: {:.4}\n",
                i + 1,
                r.title,
                r.video_id,
                r.distance
            );
        }
    }

    Ok(())
}
