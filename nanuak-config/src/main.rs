use nanuak_config::{config::NanuakConfig, db_url::DatabaseUrl};
use tracing::info;

#[tokio::main]
pub async fn main() -> eyre::Result<()> {
    color_eyre::install()?;
    tracing_subscriber::fmt().init();
    let mut config = NanuakConfig::acquire().await?;
    let db_url = config.get::<DatabaseUrl>().await?;
    info!("Database URL: {:?}", db_url);
    Ok(())
}