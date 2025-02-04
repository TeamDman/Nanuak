pub mod args;
pub mod process_file;
pub mod insert_request_if_needed;
pub mod request_kind;

use args::Args;
use clap::Parser;
use diesel::prelude::*;
use diesel::r2d2::ConnectionManager;
use diesel::r2d2::Pool;
use process_file::process_file;
use tracing::{debug, info};

fn init() -> eyre::Result<Args> {
    color_eyre::install()?;

    let args = Args::parse();

    // Initialize tracing subscriber
    if args.debug {
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::DEBUG)
            .init();
    } else {
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::INFO)
            .init();
    }

    debug!("Arguments: {:?}", args);
    Ok(args)
}

fn main() -> eyre::Result<()> {
    let args = init()?;

    // Setup DB connection
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let manager = ConnectionManager::<PgConnection>::new(&database_url);
    let pool = Pool::builder().build(manager)?;
    let mut conn = pool.get()?;

    // Gather file paths
    let file_paths = args.get_file_paths()?;
    for file_path in file_paths {
        debug!("Processing file: {:?}", file_path);
        process_file(&mut conn, &file_path, &args)?;
    }

    info!("Requests created (if needed).");
    Ok(())
}
