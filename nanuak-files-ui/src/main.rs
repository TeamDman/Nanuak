use crate::db::AppState;
use axum::response::Html;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::Router;
use std::net::SocketAddr;
use tower_http::services::ServeDir;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

// Declare the modules
mod db;
mod routes;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load .env if you like
    dotenvy::dotenv().ok();

    // Create a shared db pool
    let state = AppState::new().await?;

    // Build our router
    let app = Router::new()
        .route("/", get(index_html)) // Serve the index HTML
        .route("/files", get(routes::get_files)) // GET /files -> JSON
        .route("/search", get(routes::search_files)) // GET /search -> JSON
        .route("/images/:file_id", get(routes::get_image))
        // Serve any additional static content from /static if you like
        .nest_service("/static", ServeDir::new("static"))
        // Use state
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
    // In a real app, load from `include_str!` or read from disk
    let html = include_str!("../templates/index.html");
    Html(html.to_owned())
}
