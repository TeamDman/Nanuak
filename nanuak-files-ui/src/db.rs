use diesel::pg::PgConnection;
use diesel::r2d2::ConnectionManager;
use diesel::r2d2::Pool;
use std::env;
use thiserror::Error;

#[derive(Clone)]
pub struct AppState {
    pub pool: Pool<ConnectionManager<PgConnection>>,
}

impl AppState {
    pub async fn new() -> Result<Self, DbError> {
        // Load the DATABASE_URL from environment variables
        let database_url = env::var("DATABASE_URL")
            .map_err(|_| DbError::MissingEnv("DATABASE_URL".to_string()))?;

        let manager = ConnectionManager::<PgConnection>::new(database_url);
        let pool = Pool::builder()
            .build(manager)
            .map_err(|e| DbError::PoolError(format!("{:?}", e)))?;

        Ok(Self { pool })
    }
}

#[derive(Debug, Error)]
pub enum DbError {
    #[error("Missing environment var: {0}")]
    MissingEnv(String),
    #[error("Error building pool: {0}")]
    PoolError(String),
}
