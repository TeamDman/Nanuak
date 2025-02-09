use nanuak_1password::pick_secret::pick_secret;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    color_eyre::install()?;
    tracing_subscriber::fmt().init();
    let field = pick_secret().await?;
    println!("field: {:?}", field);
    Ok(())
}
