use nanuak_config::{config::NanuakConfig, db_url::DatabasePassword};

pub async fn get_database_url() -> eyre::Result<String> {
    let mut config = NanuakConfig::acquire().await?;
    let password = config.get::<DatabasePassword>().await?;
    let url = format!("postgres://postgres:{}@localhost/nanuak", password);
    Ok(url)
}
