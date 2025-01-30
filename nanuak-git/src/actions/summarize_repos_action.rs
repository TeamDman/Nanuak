use diesel::r2d2::ConnectionManager;
use diesel::PgConnection;
use r2d2::Pool;
use tracing::info;

use crate::pick_repo::pick_repo;

pub async fn summarize_repos_action(
    pool: Pool<ConnectionManager<PgConnection>>,
) -> eyre::Result<()> {
    let mut conn = pool.get()?;

    let repo = pick_repo(&mut conn).await?;

    info!("Summarizing repo: {:?}", repo);

    Ok(())
}
