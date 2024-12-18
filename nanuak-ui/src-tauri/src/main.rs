// Prevents additional console window on Windows in release
#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]
use diesel::data_types::PgInterval;
use diesel::pg::PgConnection;
use diesel::prelude::*;
use diesel::r2d2::ConnectionManager;
use diesel::r2d2::Pool;
use diesel::sql_types::Float4;
use diesel::sql_types::Float8;
use diesel::sql_types::Int8;
use diesel::sql_types::Interval;
use diesel::sql_types::Nullable;
use diesel::sql_types::Text;
use ollama_rs::generation::embeddings::request::GenerateEmbeddingsRequest;
use ollama_rs::Ollama;
use pgvector::Vector;
use serde::Deserialize;
use serde::Serialize;
use specta::Type;
use specta_typescript::Typescript;
use std::sync::LazyLock;
use tauri_specta::collect_commands;
use tauri_specta::collect_events;

static DB_POOL: LazyLock<Pool<ConnectionManager<PgConnection>>> = LazyLock::new(|| {
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    Pool::builder()
        .build(manager)
        .expect("Failed to create pool")
});

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
#[specta::specta]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[derive(Serialize, Deserialize, Type)]
pub struct Video {
    id: String,
    title: String,
    thumbnail: String,
    duration: u32,
    views: u32,
}

#[derive(QueryableByName)]
struct ResultRow {
    #[diesel(sql_type = Text)]
    video_id: String,
    #[diesel(sql_type = Text)]
    title: String,
    #[diesel(sql_type = Interval)]
    duration: PgInterval,
    #[diesel(sql_type = Nullable<Int8>)]
    view_count: Option<i64>,
    #[diesel(sql_type = Nullable<Text>)]
    url: Option<String>,
}

#[tauri::command]
#[specta::specta]
async fn fetch_videos(search: Option<String>) -> Result<Vec<Video>, String> {
    async fn fetch_videos_inner(search: Option<String>) -> eyre::Result<Vec<Video>> {
        log::info!("fetching videos with search: {:?}", search);

        let mut conn = DB_POOL.get()?;
        let q = search.unwrap_or_default().trim().to_string();

        if q.is_empty() {
            let query = r#"
                SELECT v.video_id, v.title, v.duration, v.view_count, vt.url
                FROM youtube.videos v
                LEFT JOIN LATERAL (
                    SELECT th.url, th.width, th.height
                    FROM youtube.video_thumbnails th
                    WHERE th.video_etag = v.etag
                    ORDER BY (th.width * th.height) DESC NULLS LAST
                    LIMIT 1
                ) vt ON true
                ORDER BY v.fetched_on DESC
                LIMIT 50
            "#;

            let results: Vec<ResultRow> = diesel::sql_query(query).load::<ResultRow>(&mut conn)?;

            let videos = results
                .into_iter()
                .map(|r| Video {
                    id: r.video_id,
                    title: r.title,
                    thumbnail: r
                        .url
                        .unwrap_or_else(|| "https://picsum.photos/seed/default/640/360".to_owned()),
                    duration: (r.duration.microseconds / 1_000_000) as u32,
                    views: r.view_count.unwrap_or(0) as u32,
                })
                .collect();
            return Ok(videos);
        }

        // 1. Text-based search with ranking
        // We'll return rank using ts_rank
        #[derive(QueryableByName)]
        struct TextSearchRow {
            #[diesel(sql_type = Text)]
            video_id: String,
            #[diesel(sql_type = Text)]
            title: String,
            #[diesel(sql_type = Interval)]
            duration: PgInterval,
            #[diesel(sql_type = Nullable<Int8>)]
            view_count: Option<i64>,
            #[diesel(sql_type = Nullable<Text>)]
            thumbnail: Option<String>,
            #[diesel(sql_type = Float4)]
            rank: f32,
        }

        let text_query = r#"
        SELECT v.video_id, v.title, v.duration, v.view_count, vt.url AS thumbnail,
               ts_rank(v.search_document, plainto_tsquery('english', $1)) AS rank
        FROM youtube.videos v
        LEFT JOIN LATERAL (
            SELECT vt.url, vt.width, vt.height
            FROM youtube.video_thumbnails vt
            WHERE vt.video_etag = v.etag
            ORDER BY (vt.width * vt.height) DESC
            LIMIT 1
        ) vt ON true
        WHERE v.search_document @@ plainto_tsquery('english', $1)
        ORDER BY rank DESC
        LIMIT 50
    "#;

        let text_results: Vec<TextSearchRow> = diesel::sql_query(text_query)
            .bind::<Text, _>(&q)
            .load(&mut conn)?;

        // 2. Embedding-based search
        // Generate embedding for the query
        // Make sure this code runs inside an async function and ollama is async.
        let ollama = Ollama::default();
        let embed_req =
            GenerateEmbeddingsRequest::new("bge-m3:latest".to_string(), q.clone().into());
        let embed_res = ollama.generate_embeddings(embed_req).await?;
        if embed_res.embeddings.is_empty() {
            // If no embeddings returned, fallback to just text results.
            let videos = text_results
                .into_iter()
                .map(|row| Video {
                    id: row.video_id,
                    title: row.title,
                    thumbnail: row
                        .thumbnail
                        .unwrap_or_else(|| "https://picsum.photos/seed/default/640/360".to_owned()),
                    duration: (row.duration.microseconds / 1_000_000_i64) as u32,
                    views: row.view_count.unwrap_or(0) as u32,
                })
                .collect();
            return Ok(videos);
        }

        let query_embedding = Vector::from(embed_res.embeddings[0].clone());

        #[derive(QueryableByName)]
        struct EmbedSearchRow {
            #[diesel(sql_type = Text)]
            video_id: String,
            #[diesel(sql_type = Text)]
            title: String,
            #[diesel(sql_type = Interval)]
            duration: PgInterval,
            #[diesel(sql_type = Nullable<Int8>)]
            view_count: Option<i64>,
            #[diesel(sql_type = Nullable<Text>)]
            thumbnail: Option<String>,
            #[diesel(sql_type = Float8)]
            distance: f64,
        }

        let embed_query = r#"
        SELECT v.video_id, v.title, v.duration, v.view_count, vt.url AS thumbnail,
               (e.embedding <-> $1) AS distance
        FROM youtube.videos v
        JOIN youtube.video_embeddings_bge_m3 e ON v.etag = e.video_etag
        LEFT JOIN LATERAL (
            SELECT vt.url, vt.width, vt.height
            FROM youtube.video_thumbnails vt
            WHERE vt.video_etag = v.etag
            ORDER BY (vt.width * vt.height) DESC
            LIMIT 1
        ) vt ON true
        ORDER BY e.embedding <-> $1
        LIMIT 50
    "#;

        let embed_results: Vec<EmbedSearchRow> = diesel::sql_query(embed_query)
            .bind::<pgvector::sql_types::Vector, _>(query_embedding)
            .load(&mut conn)?;

        // 3. Weave results: text_results (high rank first), embed_results (low distance first)
        // Convert both to a common Video form
        let text_videos: Vec<Video> = text_results
            .into_iter()
            .map(|row| Video {
                id: row.video_id,
                title: row.title,
                thumbnail: row
                    .thumbnail
                    .unwrap_or_else(|| "https://picsum.photos/seed/default/640/360".to_owned()),
                duration: (row.duration.microseconds / 1_000_000_i64) as u32,
                views: row.view_count.unwrap_or(0) as u32,
            })
            .collect();

        let embed_videos: Vec<Video> = embed_results
            .into_iter()
            .map(|row| Video {
                id: row.video_id,
                title: row.title,
                thumbnail: row
                    .thumbnail
                    .unwrap_or_else(|| "https://picsum.photos/seed/default/640/360".to_owned()),
                duration: (row.duration.microseconds / 1_000_000_i64) as u32,
                views: row.view_count.unwrap_or(0) as u32,
            })
            .collect();

        // Weave them: 1 from text, 1 from embed, etc.
        let mut combined = Vec::new();
        let mut ti = text_videos.into_iter();
        let mut ei = embed_videos.into_iter();

        loop {
            if let Some(tv) = ti.next() {
                // combined.push(tv);
                // lets just do embedding for now
            } else {
                break;
            }
            if let Some(ev) = ei.next() {
                combined.push(ev);
            } else {
                break;
            }
        }

        // If one list is exhausted, we could append the rest of the other:
        // combined.extend(ti);
        // combined.extend(ei);
        //
        // But for a perfect weave, we'll just stop as soon as one is exhausted.
        // If desired, uncomment the above lines.

        Ok(combined)
    }
    match fetch_videos_inner(search).await {
        Ok(videos) => Ok(videos),
        Err(e) => {
            log::error!("Error fetching videos: {:?}", e);
            Err(e.to_string())
        }
    }
}

fn main() -> eyre::Result<()> {
    color_eyre::install()?;

    let specta_builder = tauri_specta::Builder::<tauri::Wry>::new()
        .commands(collect_commands![greet, fetch_videos])
        .events(collect_events![]);

    #[cfg(debug_assertions)]
    specta_builder
        .export(Typescript::default(), "../src/lib/bindings.ts")
        .expect("Failed to export typescript bindings");

    // tauri_specta::Builder::<tauri::Wry>::new()
    tauri::Builder::default()
        .plugin(tauri_plugin_log::Builder::new().build())
        .invoke_handler(specta_builder.invoke_handler())
        .setup(move |app| {
            specta_builder.mount_events(app);
            Ok(())
        })
        .plugin(tauri_plugin_app::init())
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_shell::init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
    Ok(())
}
