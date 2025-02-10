use diesel::pg::PgConnection;
use diesel::r2d2::ConnectionManager;
use diesel::r2d2::Pool;
use eyre::Context;
use nanuak_config::config::NanuakConfig;
use nanuak_config::db_url::DatabasePassword;

#[derive(Clone)]
pub struct AppState {
    pub pool: Pool<ConnectionManager<PgConnection>>,
}

impl AppState {
    pub async fn new() -> eyre::Result<Self> {
        let mut config = NanuakConfig::acquire().await?;
        let database_url = DatabasePassword::format_url(&config.get::<DatabasePassword>().await?);

        let manager = ConnectionManager::<PgConnection>::new(database_url);
        let pool = Pool::builder()
            .build(manager)
            .wrap_err("Failed building database pool")?;

        Ok(Self { pool })
    }
}
