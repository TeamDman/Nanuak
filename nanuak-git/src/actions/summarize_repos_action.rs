use diesel::r2d2::ConnectionManager;
use diesel::PgConnection;
use r2d2::Pool;
use tracing::info;

use crate::get_github_details::get_github_details;
use crate::pick_remote::pick_remote;
use crate::pick_repo::pick_repo;
use crate::repo_manifest::RepoManifest;

pub async fn summarize_repos_action(
    pool: Pool<ConnectionManager<PgConnection>>,
) -> eyre::Result<()> {
    let mut conn = pool.get()?;

    let repo = pick_repo(&mut conn).await?;

    info!("Summarizing repo: {:?}", repo);

    let remote = pick_remote(&repo.remotes)?;
    let github_details = get_github_details(remote.1).await?;

    let manifest = RepoManifest {
        github_details,
        notable_file_contents: vec![],
    };

    info!("Manifest: {:?}", manifest);
    Ok(())
}
