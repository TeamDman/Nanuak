use nanuak_config::config::NanuakConfig;
use nanuak_config::db_url::DatabasePassword;
use tracing::info;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::EnvFilter;

#[tokio::main]
pub async fn main() -> eyre::Result<()> {
    color_eyre::install()?;
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::DEBUG.into())
                .from_env()?,
        )
        .init();
    let mut config = NanuakConfig::acquire().await?;
    let db_url = config.get::<DatabasePassword>().await?;
    info!("Database URL: {:?}", db_url.len());
    Ok(())
}
