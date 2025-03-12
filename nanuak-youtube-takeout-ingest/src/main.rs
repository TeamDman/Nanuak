pub mod entry;
pub mod search_entry;
pub mod view_post_entry;
pub mod watch_entry;

use clap::Parser;
use color_eyre::eyre::Result;
use diesel::prelude::*;
use diesel::r2d2::ConnectionManager;
use diesel::r2d2::Pool;
use entry::load_entries;
use entry::Entry;
use nanuak_config::config::NanuakConfig;
use nanuak_config::db_url::DatabasePassword;
use nanuak_schema::youtube::posts::dsl as posts_dsl;
use nanuak_schema::youtube::search_history::dsl as search_dsl;
use nanuak_schema::youtube::watch_history::dsl as watch_dsl;
use std::path::PathBuf;
use tracing::info;

/// Command-line arguments for the ingest tool
#[derive(Parser, Debug)]
#[command(version, about = "Ingest YouTube Takeout JSON Files")]
struct Args {
    /// Path to the directory directly containing JSON files
    #[arg(short, long, value_name = "DIR")]
    ingest_dir: PathBuf,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    color_eyre::install()?;

    // Parse command-line arguments
    let args = Args::parse();

    let database_url = DatabasePassword::format_url(
        &NanuakConfig::acquire()
            .await?
            .get::<DatabasePassword>()
            .await?,
    );

    // Establish a database connection pool
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    let pool = Pool::builder().build(manager)?;
    let mut conn = pool.get()?;
    info!("Established database connection");

    // Read the directory and process JSON files
    let mut entries = tokio::fs::read_dir(&args.ingest_dir).await?;
    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("json") {
            info!("Processing JSON file: {}", path.display());
            let entries = load_entries(&path).await?;

            // Insert rows into the database
            let mut success_count = 0;
            for entry in entries {
                match entry {
                    Entry::Search(search_entry) => {
                        diesel::insert_into(search_dsl::search_history)
                            .values((
                                search_dsl::time.eq(search_entry.time.naive_utc()),
                                search_dsl::query.eq(search_entry.query),
                            ))
                            .on_conflict_do_nothing()
                            .execute(&mut conn)?;
                        // info!("Inserted search entry: {:?}", search_entry);
                    }
                    Entry::Watch(watch_entry) => {
                        diesel::insert_into(watch_dsl::watch_history)
                            .values((
                                watch_dsl::time.eq(watch_entry.time.naive_utc()),
                                watch_dsl::youtube_video_id.eq(watch_entry.youtube_video_id),
                            ))
                            .on_conflict_do_nothing()
                            .execute(&mut conn)?;
                        // info!("Inserted watch entry: {:?}", watch_entry);
                    }
                    Entry::ViewPost(view_post_entry) => {
                        diesel::insert_into(posts_dsl::posts)
                            .values((
                                posts_dsl::time.eq(view_post_entry.time.naive_utc()),
                                posts_dsl::post_title.eq(view_post_entry.post_title),
                                posts_dsl::post_url.eq(view_post_entry.post_url),
                                posts_dsl::channel_url.eq(view_post_entry.channel_url),
                                posts_dsl::channel_name.eq(view_post_entry.channel_name),
                            ))
                            .on_conflict_do_nothing()
                            .execute(&mut conn)?;
                        // info!("Inserted view post entry: {:?}", view_post_entry);
                    }
                }
                success_count += 1;
            }
            info!(
                "Successfully inserted {} entries, some duplicates may have been ignored",
                success_count
            );
        }
    }

    Ok(())
}
