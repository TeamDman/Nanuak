// Prevents additional console window on Windows in release
#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use serde::Deserialize;
use serde::Serialize;
use specta::Type;
use specta_typescript::Typescript;
use tauri_specta::collect_commands;
use tauri_specta::collect_events;

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

#[tauri::command]
#[specta::specta]
fn fetch_videos() -> Vec<Video> {
    vec![
        Video {
            id: "1".to_owned(),
            title: "Building a Modern Web Application".to_owned(),
            thumbnail: "https://picsum.photos/seed/1/640/360".to_owned(),
            duration: 1845,
            views: 15420,
        },
        Video {
            id: "2".to_owned(),
            title: "Advanced Database Concepts".to_owned(),
            thumbnail: "https://picsum.photos/seed/2/640/360".to_owned(),
            duration: 2250,
            views: 8750,
        },
        Video {
            id: "3".to_owned(),
            title: "Understanding Kubernetes".to_owned(),
            thumbnail: "https://picsum.photos/seed/3/640/360".to_owned(),
            duration: 3600,
            views: 12300,
        },
    ]
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
