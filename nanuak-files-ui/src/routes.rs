use crate::db::AppState;
use axum::extract::Path;
use axum::extract::Query;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::response::Response;
use axum::Json;
use diesel::prelude::*;
use diesel::sql_query;
use diesel::sql_types::Array;
use diesel::sql_types::Int4;
use diesel::RunQueryDsl;
use nanuak_schema::files::captions::dsl as cc;
use nanuak_schema::files::files::dsl as ff;
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;
use std::io::Read;
use std::path::PathBuf;

// --------------- /files ---------------

#[derive(Debug, Serialize)]
pub struct FileWithCaption {
    pub file_id: i32,
    pub path: String,
    pub caption: Option<String>,
}

// We expand the GET /files endpoint to handle offset + limit
#[derive(Debug, Deserialize)]
pub struct FilesQuery {
    pub offset: Option<i64>,
    pub limit: Option<i64>,
}

pub async fn get_files(
    State(state): State<AppState>,
    Query(params): Query<FilesQuery>,
) -> Result<Json<Vec<FileWithCaption>>, (StatusCode, String)> {
    let mut conn = state.pool.get().map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "DB pool error".to_string(),
        )
    })?;

    // We parse offset + limit
    let offset_val = params.offset.unwrap_or(0);
    let limit_val = params.limit.unwrap_or(100); // default limit e.g. 100 if none provided

    // left_join for captions
    let query = ff::files
        .left_join(cc::captions.on(cc::file_id.eq(ff::id)))
        .select((ff::id, ff::path, cc::caption.nullable()))
        .offset(offset_val)
        .limit(limit_val);

    let results = query
        .load::<(i32, String, Option<String>)>(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Convert
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

// --------------- /files/details ---------------
// So we can request data for a set of file IDs

#[derive(Debug, Deserialize)]
pub struct FileDetailsRequest {
    pub file_ids: Vec<i32>,
}

#[derive(Debug, Serialize)]
pub struct FileDetailsResponse {
    pub file_id: i32,
    pub path: String,
    pub caption: Option<String>,
}

pub async fn get_files_details(
    State(state): State<AppState>,
    Json(body): Json<FileDetailsRequest>,
) -> Result<Json<Vec<FileDetailsResponse>>, (StatusCode, String)> {
    let mut conn = state.pool.get().map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "DB pool error".to_string(),
        )
    })?;

    if body.file_ids.is_empty() {
        return Ok(Json(vec![]));
    }

    // We'll do a left_join with captions too
    // We can do something like "WHERE id = ANY($1)" with Diesel
    // but let's keep it simpler with standard DSL:
    let results = ff::files
        .filter(ff::id.eq_any(&body.file_ids))
        .left_join(cc::captions.on(cc::file_id.eq(ff::id)))
        .select((ff::id, ff::path, cc::caption.nullable()))
        .load::<(i32, String, Option<String>)>(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let data = results
        .into_iter()
        .map(|(file_id, path, caption)| FileDetailsResponse {
            file_id,
            path,
            caption,
        })
        .collect();

    Ok(Json(data))
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

    let caption_q = params.caption.clone().unwrap_or_default();
    let embedding_q = params.embedding.clone().unwrap_or_default();

    // If both empty => return all
    if caption_q.is_empty() && embedding_q.is_empty() {
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

    // Only do text-based searching for caption
    let mut query = ff::files
        .left_join(cc::captions.on(cc::file_id.eq(ff::id)))
        .select(ff::id)
        .into_boxed();

    if !caption_q.is_empty() {
        query = query.filter(cc::caption.ilike(format!("%{}%", caption_q)));
    }

    // embedding_q is not handled here, so if you pass embedding it’s ignored in this text-based route
    // If you want to unify them, you can. Right now, we skip it.

    let results = query
        .load::<i32>(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let data = results
        .into_iter()
        .map(|fid| SearchResult { file_id: fid })
        .collect();
    Ok(Json(data))
}

// --------------- /embedding_search ---------------
#[derive(Debug, Deserialize)]
struct EmbeddingSearchResponse {
    file_id: i32,
    distance: f64,
}

#[derive(Debug, Serialize)]
pub struct EmbeddedSearchResult {
    file_id: i32,
    path: String,
    distance: f64,
}

pub async fn embedding_search(
    State(state): State<AppState>,
    Query(params): Query<SearchParams>,
) -> Result<Json<Vec<EmbeddedSearchResult>>, (StatusCode, String)> {
    // Our code uses "params.caption" for the actual embedding text
    let query_str = params.caption.clone().unwrap_or_default();
    if query_str.is_empty() {
        return Ok(Json(vec![]));
    }

    // 1) call the external Python service
    let fastapi_url = format!(
        "http://127.0.0.1:8000/search_embedding?q={}",
        urlencoding::encode(&query_str)
    );

    let client = reqwest::Client::new();
    let response = client
        .get(&fastapi_url)
        .send()
        .await
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;

    if !response.status().is_success() {
        return Err((StatusCode::BAD_GATEWAY, "FastAPI call failed".to_string()));
    }

    let data = response
        .json::<Vec<EmbeddingSearchResponse>>()
        .await
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;

    // data is an array of { file_id, distance }
    let file_ids: Vec<i32> = data.iter().map(|x| x.file_id).collect();
    if file_ids.is_empty() {
        return Ok(Json(vec![]));
    }

    let mut conn = state.pool.get().map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "DB pool error".to_string(),
        )
    })?;

    // We'll get the path from the DB
    // Diesel trick for "WHERE id = ANY($1)"
    #[derive(Debug, QueryableByName)]
    struct IdPath {
        #[diesel(sql_type = diesel::sql_types::Int4)]
        id: i32,

        #[diesel(sql_type = diesel::sql_types::Text)]
        path: String,
    }

    let pairs = sql_query("SELECT id, path FROM files.files WHERE id = ANY($1)")
        .bind::<Array<Int4>, _>(file_ids.clone())
        .load::<IdPath>(&mut conn)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let map_path: HashMap<i32, String> = pairs
        .into_iter()
        .map(|record| (record.id, record.path))
        .collect();

    // build final
    let mut results: Vec<EmbeddedSearchResult> = data
        .iter()
        .map(|r| {
            let path = map_path.get(&r.file_id).cloned().unwrap_or_default();
            EmbeddedSearchResult {
                file_id: r.file_id,
                path,
                distance: r.distance,
            }
        })
        .collect();

    // (Optionally) sort by distance ascending if needed
    results.sort_by(|a, b| {
        a.distance
            .partial_cmp(&b.distance)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    Ok(Json(results))
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

    let response = Response::builder()
        .status(StatusCode::OK)
        .header(axum::http::header::CONTENT_TYPE, mime_type)
        .body(axum::body::Full::from(buffer))
        .unwrap();

    response.into_response()
}
