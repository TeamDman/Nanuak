use tracing::info;

#[tokio::main]
pub async fn main() -> eyre::Result<()> {
    color_eyre::install()?;
    tracing_subscriber::fmt().init();
    info!("Hello, world!");
    Ok(())
}