use diesel::prelude::*;
use nanuak_youtube_embeddings::load_videos_needing_embeddings;

// Example usage in your main code:
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let database_url = std::env::var("DATABASE_URL")?;
    let mut conn = PgConnection::establish(&database_url)?;

    // Get 50 videos
    let videos = load_videos_needing_embeddings(&mut conn, 50)?;
    for video in videos {
        println!("{:?}", video);
    }

    Ok(())
}
