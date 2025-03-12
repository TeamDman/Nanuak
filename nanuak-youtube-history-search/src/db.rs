use color_eyre::Result;
use diesel::prelude::*;
use diesel::PgConnection;
use nanuak_schema::youtube::videos;

#[derive(Debug, Queryable)]
pub struct SearchResult {
    pub title: String,
    pub video_id: String,
}

pub async fn get_all_results(conn: &mut PgConnection) -> Result<Vec<SearchResult>> {
    let result = videos::table
        .select((videos::title, videos::video_id))
        .load::<SearchResult>(conn)?;

    Ok(result)
}
