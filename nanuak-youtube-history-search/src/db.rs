use chrono::Duration;
use chrono::NaiveDateTime;
use color_eyre::Result;
use diesel::prelude::*;
use diesel::sql_query;
use diesel::sql_types::Interval;
use diesel::sql_types::Nullable;
use diesel::sql_types::Text;
use diesel::sql_types::Timestamp;

/// Tells Diesel "this struct can be loaded from a raw SQL query"
#[derive(Debug, QueryableByName)]
pub struct SearchResult {
    #[diesel(sql_type = Text)]
    pub title: String,

    #[diesel(sql_type = Text)]
    pub video_id: String,

    #[diesel(sql_type = Interval)]
    pub duration: Duration,

    #[diesel(sql_type = Nullable<Timestamp>)]
    pub last_watch: Option<NaiveDateTime>,
}

pub async fn get_filtered_results(
    conn: &mut PgConnection,
    search_pattern: String,
    min_secs: Option<i64>,
    max_secs: Option<i64>,
    ago_secs: Option<i64>,
) -> Result<Vec<SearchResult>> {
    // Filter in SQL if 'ago_secs' is provided, but do not do the difference math here.
    // We'll just retrieve the raw "last_watch" value and let the Rust side handle "X days ago."
    // The row is included if last_watch >= now() - interval(...) or last_watch is NULL (if no filter).
    let query_sql = r#"
        SELECT
            v.title,
            v.video_id,
            v.duration,
            lw.last_watch AS last_watch
        FROM youtube.videos v
        LEFT JOIN (
            SELECT
                youtube_video_id,
                MAX(time) AS last_watch
            FROM youtube.watch_history
            GROUP BY youtube_video_id
        ) lw
            ON lw.youtube_video_id = v.video_id
        WHERE
            v.title ILIKE $1
            AND ($2 IS NULL OR v.duration >= (($2::text || ' seconds')::interval))
            AND ($3 IS NULL OR v.duration <= (($3::text || ' seconds')::interval))
            AND (
                $4 IS NULL
                OR (
                    lw.last_watch IS NOT NULL
                    AND lw.last_watch >= (now() - (($4::text || ' seconds')::interval))
                )
            )
        ORDER BY v.title ASC
    "#;

    fn opt_i64_to_string(secs: Option<i64>) -> Option<String> {
        secs.map(|x| x.to_string())
    }

    let results = sql_query(query_sql)
        .bind::<Text, _>(search_pattern) // $1
        .bind::<diesel::sql_types::Nullable<Text>, _>(opt_i64_to_string(min_secs)) // $2
        .bind::<diesel::sql_types::Nullable<Text>, _>(opt_i64_to_string(max_secs)) // $3
        .bind::<diesel::sql_types::Nullable<Text>, _>(opt_i64_to_string(ago_secs)) // $4
        .load::<SearchResult>(conn)?;

    Ok(results)
}
