// Prevents additional console window on Windows in release
#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use diesel::pg::PgConnection;
use diesel::prelude::*;
use diesel::r2d2::ConnectionManager;
use diesel::r2d2::Pool;
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

// Struct to receive query results
#[derive(QueryableByName)]
#[allow(dead_code)]
struct DbVideo {
    #[diesel(sql_type = diesel::sql_types::Text)]
    video_id: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    title: String,
    #[diesel(sql_type = diesel::sql_types::Int4)]
    dur_seconds: i32,
    #[diesel(sql_type = diesel::sql_types::Int8)]
    view_count: i64,
    #[diesel(sql_type = diesel::sql_types::Text)]
    thumbnail: String,
}

#[tauri::command]
#[specta::specta]
fn fetch_videos(search: Option<String>) -> Result<Vec<Video>, String> {
    log::info!("Fetching using query {search:?}");
    use diesel::prelude::*;
    use diesel_full_text_search::plainto_tsquery;
    use diesel_full_text_search::TsVectorExtensions;
    use nanuak_schema::youtube::video_thumbnails;
    use nanuak_schema::youtube::videos;

    let mut conn = DB_POOL.get().map_err(|e| e.to_string())?;
    // Start building the query
    let mut query = videos::table
        .left_join(
            video_thumbnails::table.on(video_thumbnails::video_etag.eq(videos::etag.nullable()).and(video_thumbnails::size_description.eq("maxres"))),
        )
        .select((
            videos::video_id,
            videos::title,
            videos::duration,
            videos::view_count,
            video_thumbnails::url.nullable(),
        ))
        .into_boxed();

    // If the user provided a search string, apply full-text search
    if let Some(ref q) = search {
        if !q.is_empty() {
            query = query.filter(
                // videos::search_document.matches(sql("'english'::regconfig'"), plainto_tsquery(q)),
                videos::search_document.matches(plainto_tsquery(q)),
            );
        }
    }

    // Limit the number of results
    let results = query
        .limit(50)
        .load::<(
            String,
            String,
            chrono::Duration,
            Option<i64>,
            Option<String>,
        )>(&mut conn)
        .map_err(|e| e.to_string())?;

    // Convert DB rows into the frontend `Video` struct
    let videos = results
        .into_iter()
        .map(|(vid, title, dur, views, thumb)| Video {
            id: vid,
            title,
            thumbnail: thumb
                .unwrap_or_else(|| "https://picsum.photos/seed/default/640/360".to_owned()),
            duration: dur.num_seconds() as u32,
            views: views.unwrap_or(0) as u32,
        })
        .collect();

    Ok(videos)
}

fn main() {
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
}
