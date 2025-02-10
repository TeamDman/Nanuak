use nanuak_config::config::NanuakConfig;
use nanuak_config::db_url::DatabasePassword;

pub async fn get_database_url() -> eyre::Result<String> {
    let mut config = NanuakConfig::acquire().await?;
    let password = config.get::<DatabasePassword>().await?;
    Ok(DatabasePassword::format_url(&password))
}
