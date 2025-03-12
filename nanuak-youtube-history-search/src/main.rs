use color_eyre::Result;
use diesel::prelude::*;
use diesel::r2d2::ConnectionManager;
use diesel::r2d2::Pool;
use nanuak_config::config::NanuakConfig;
use nanuak_config::db_url::DatabasePassword;
use r2d2::PooledConnection;
use ratatui::DefaultTerminal;
use tracing::info;

// Bring in our modules:
mod app;
mod db;
mod ui;

use crate::app::App;

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    tracing_subscriber::fmt::init();

    let database_url = DatabasePassword::format_url(
        &NanuakConfig::acquire()
            .await?
            .get::<DatabasePassword>()
            .await?,
    );

    // Set up a database connection pool
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    let pool = Pool::builder().build(manager)?;
    let mut conn = pool.get()?;
    info!("Established database connection");

    // Initialize the terminal
    let mut terminal = ratatui::init();

    // Create the app, then run it
    let app = App::new(pool.clone(), &mut conn).await?;
    let result = app.run(&mut terminal, &mut conn).await;

    // Restore the terminal state
    ratatui::restore();

    result
}
