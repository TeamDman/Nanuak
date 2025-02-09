use nanuak_config::config::NanuakConfig;
use nanuak_config::db_url::DatabaseUrl;
use nanuak_config::dirs::get_config_path;

#[tokio::test]
pub async fn load_and_save_config() -> eyre::Result<()> {
    let mut config = NanuakConfig::acquire().await?;
    println!("Config is {:?}", config);
    let db_url = config.get::<DatabaseUrl>().await?;
    println!("DB url is {:?}", db_url);
    config.save().await?;
    Ok(())
}
#[tokio::test]
pub async fn print_path() -> eyre::Result<()> {
    let config_path = get_config_path().await?;
    println!("{:?}", config_path);
    Ok(())
}

#[tokio::test]
#[ignore]
pub async fn open_path() -> eyre::Result<()> {
    let mut config_dir = get_config_path().await?;
    config_dir.pop();
    println!("{:?}", config_dir);
    match tokio::fs::try_exists(&config_dir).await {
        Ok(true) => open::that(config_dir)?,
        Ok(false) => return Err(eyre::eyre!("Config file does not exist")),
        Err(e) => return Err(e.into()),
    }
    Ok(())
}
