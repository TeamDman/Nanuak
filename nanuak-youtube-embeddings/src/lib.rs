use diesel::prelude::*;
use diesel::sql_query;
use diesel::sql_types::Array;
use diesel::sql_types::BigInt;
use diesel::sql_types::Nullable;
use diesel::sql_types::Text;
use diesel::sql_types::Timestamp;
use serde::Deserialize;

// Define a struct that matches the query result
#[derive(Debug, QueryableByName, Deserialize)]
pub struct VideoWithLatestWatch {
    #[diesel(sql_type = Text)]
    pub etag: String,

    #[diesel(sql_type = Nullable<Text>)]
    pub title: Option<String>,

    #[diesel(sql_type = Nullable<Text>)]
    pub description: Option<String>,

    #[diesel(sql_type = Nullable<Text>)]
    pub channel_title: Option<String>,

    #[diesel(sql_type = Nullable<Text>)]
    pub category_title: Option<String>,

    // Assuming `tags` is TEXT[] in your schema
    #[diesel(sql_type = Nullable<Array<Nullable<Text>>>)]
    pub tags: Option<Vec<Option<String>>>,

    #[diesel(sql_type = Nullable<BigInt>)]
    pub view_count: Option<i64>,

    #[diesel(sql_type = Nullable<BigInt>)]
    pub like_count: Option<i64>,

    #[diesel(sql_type = Nullable<BigInt>)]
    pub comment_count: Option<i64>,

    #[diesel(sql_type = Timestamp)]
    pub latest_watch_time: chrono::NaiveDateTime,
}

// Now we write the function to load these videos:
pub fn load_videos_needing_embeddings(
    conn: &mut PgConnection,
    limit: i64,
) -> QueryResult<Vec<VideoWithLatestWatch>> {
    let sql = format!(
        r#"
        SELECT etag, title, description, channel_title, category_title, tags, view_count, like_count, comment_count, latest_watch_time
        FROM (
            SELECT v.etag,
                   v.title,
                   v.description,
                   v.channel_title,
                   c.title AS category_title,
                   v.tags,
                   v.view_count,
                   v.like_count,
                   v.comment_count,
                   w.time AS latest_watch_time,
                   RANK() OVER (PARTITION BY v.etag ORDER BY w.time DESC) AS rnk
            FROM youtube.videos v
            JOIN youtube.watch_history w ON w.youtube_video_id = v.video_id
            LEFT JOIN youtube.video_categories c ON v.category_id = c.id
            WHERE v.etag NOT IN (
                SELECT video_etag FROM youtube.video_embeddings_bge_m3
            )
        ) AS sub
        WHERE rnk = 1
        ORDER BY latest_watch_time DESC
        LIMIT {}
    "#,
        limit
    );

    sql_query(sql).load::<VideoWithLatestWatch>(conn)
}

/// Count how many videos still need embeddings
pub fn count_videos_needing_embeddings(conn: &mut PgConnection) -> QueryResult<i64> {
    // Similar logic as load_videos_needing_embeddings but just counting
    // We'll reuse the window function approach from before:
    //
    // SELECT count(*) FROM (
    //    SELECT v.etag,
    //           RANK() OVER (PARTITION BY v.etag ORDER BY w.time DESC) AS rnk
    //    FROM youtube.videos v
    //    JOIN youtube.watch_history w ON w.youtube_video_id = v.video_id
    //    WHERE v.etag NOT IN (SELECT video_etag FROM youtube.video_embeddings_bge_m3)
    // ) AS sub
    // WHERE rnk = 1;
    use diesel::sql_types::BigInt;
    let sql = r#"
        SELECT COUNT(*) as count
        FROM (
            SELECT v.etag,
                   RANK() OVER (PARTITION BY v.etag ORDER BY w.time DESC) AS rnk
            FROM youtube.videos v
            JOIN youtube.watch_history w ON w.youtube_video_id = v.video_id
            WHERE v.etag NOT IN (
                SELECT video_etag FROM youtube.video_embeddings_bge_m3
            )
        ) AS sub
        WHERE rnk = 1
    "#;

    #[derive(QueryableByName)]
    struct CountRow {
        #[diesel(sql_type = BigInt)]
        count: i64,
    }

    let row = diesel::sql_query(sql).get_result::<CountRow>(conn)?;
    Ok(row.count)
}
