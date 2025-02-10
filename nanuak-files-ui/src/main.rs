use crate::db::AppState;
use axum::response::Html;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::routing::post;
use axum::Router;
use std::net::SocketAddr;
use tower_http::services::ServeDir;

mod db;
mod routes;

use clap::Parser;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::EnvFilter;

#[derive(Parser, Debug)]
#[command(version, about = "Nanuak Files UI")]
struct Args {
    /// If set, enable debug logging
    #[arg(long)]
    debug: bool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    color_eyre::install()?;

    // Parse CLI
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

    // Create a shared db pool
    let state = AppState::new().await?;

    // Build our router
    // Note that for the new POST endpoint, we need `post(routes::get_files_details)`
    let app = Router::new()
        .route("/", get(index_html)) // Serve the index HTML
        .route("/files", get(routes::get_files)) // GET /files -> JSON
        .route("/files/details", post(routes::get_files_details)) // POST -> get file details
        .route("/search", get(routes::search_files)) // GET /search -> JSON
        .route("/embedding_search", get(routes::embedding_search))
        .route("/images/:file_id", get(routes::get_image))
        // Serve any additional static content from /static if you like
        .nest_service("/static", ServeDir::new("static"))
        // Use our shared state
        .with_state(state);

    // Run
    let addr = SocketAddr::from(([127, 0, 0, 1], 3001));
    tracing::info!("Listening on http://{}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}

// Serve the index.html from compiled-in template or from disk.
async fn index_html() -> impl IntoResponse {
    let html = include_str!("../templates/index.html");
    Html(html.to_owned())
}
