use crate::db::AppState;
use axum::extract::Path;
use axum::extract::Query;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::response::Response;
use axum::Json;
use diesel::prelude::*;
use nanuak_schema::files::captions::dsl as cc;
use nanuak_schema::files::files::dsl as ff;
use serde::Serialize;
use std::io::Read;
use std::path::PathBuf;

// --------------- /files ---------------

#[derive(Debug, Serialize)]
pub struct FileWithCaption {
    pub file_id: i32,
    pub path: String,
    pub caption: Option<String>,
}

pub async fn get_files(
    State(state): State<AppState>,
) -> Result<Json<Vec<FileWithCaption>>, (StatusCode, String)> {
    let mut conn = state.pool.get().map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "DB pool error".to_string(),
        )
    })?;

    // The Diesel approach: left_join, then pick the columns you want
    // We'll rename them in code, or just store them in a tuple first.
    let results = ff::files
        .left_join(cc::captions.on(cc::file_id.eq(ff::id)))
        // columns to select (ff::id, ff::path, cc::caption)
        .select((ff::id, ff::path, cc::caption.nullable()))
        .load::<(i32, String, Option<String>)>(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Convert the tuple into your struct
    let files = results
        .into_iter()
        .map(|(file_id, path, caption)| FileWithCaption {
            file_id,
            path,
            caption,
        })
        .collect();

    Ok(Json(files))
}

// --------------- /search ---------------
#[derive(Debug, serde::Deserialize)]
pub struct SearchParams {
    caption: Option<String>,
    embedding: Option<String>,
}

#[derive(Debug, serde::Serialize)]
pub struct SearchResult {
    pub file_id: i32,
}

pub async fn search_files(
    State(state): State<AppState>,
    Query(params): Query<SearchParams>,
) -> Result<Json<Vec<SearchResult>>, (StatusCode, String)> {
    let mut conn = state.pool.get().map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "DB pool error".to_string(),
        )
    })?;

    let caption_q = params.caption.unwrap_or_default();
    let embedding_q = params.embedding.unwrap_or_default();

    use nanuak_schema::files::captions::dsl as cc;
    use nanuak_schema::files::files::dsl as ff;
    // We'll only do text-based caption searching here:
    if caption_q.is_empty() && embedding_q.is_empty() {
        // Return all file_id’s
        let file_ids = ff::files
            .select(ff::id)
            .load::<i32>(&mut conn)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

        let data = file_ids
            .into_iter()
            .map(|fid| SearchResult { file_id: fid })
            .collect();
        return Ok(Json(data));
    }

    // Build up a Diesel query for text searching
    let mut query = ff::files
        .left_join(cc::captions.on(cc::file_id.eq(ff::id)))
        .select(ff::id)
        .into_boxed(); // into_boxed for conditionally adding filters

    if !caption_q.is_empty() {
        // Diesel’s `.filter(cc::caption.ilike(...))` for case-insensitive partial
        query = query.filter(cc::caption.ilike(format!("%{}%", caption_q)));
    }

    // If you want to do embedding-based filtering, you’d handle that here (but that’s more advanced).
    // For now, we skip it if embedding_q is not empty.

    let results = query
        .load::<i32>(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let data = results
        .into_iter()
        .map(|fid| SearchResult { file_id: fid })
        .collect();
    Ok(Json(data))
}

// --------------- /images/{file_id} ---------------
pub async fn get_image(
    State(state): State<AppState>,
    Path(file_id): Path<i32>,
) -> impl IntoResponse {
    let mut conn = match state.pool.get() {
        Ok(c) => c,
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "DB pool error").into_response(),
    };

    // Query the path
    let path_str = match ff::files
        .filter(ff::id.eq(file_id))
        .select(ff::path)
        .first::<String>(&mut conn)
        .optional()
    {
        Ok(Some(p)) => p,
        Ok(None) => return (StatusCode::NOT_FOUND, "No such file").into_response(),
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    };

    // Attempt to read from disk
    let path = PathBuf::from(path_str);
    let mut file = match std::fs::File::open(&path) {
        Ok(f) => f,
        Err(_) => return (StatusCode::NOT_FOUND, "Could not open file").into_response(),
    };

    let mut buffer = Vec::new();
    if let Err(e) = file.read_to_end(&mut buffer) {
        return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response();
    }

    let mime_type = match path.extension().and_then(|x| x.to_str()) {
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("png") => "image/png",
        Some("gif") => "image/gif",
        _ => "application/octet-stream",
    };

    // Build a Response<Full<_>>. This time, we can return it as an `impl IntoResponse`.
    let response = Response::builder()
        .status(StatusCode::OK)
        .header(axum::http::header::CONTENT_TYPE, mime_type)
        .body(axum::body::Full::from(buffer))
        .unwrap();

    response.into_response()
}
