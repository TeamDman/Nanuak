use cloud_terrastodon_core_user_input::prelude::pick;
use cloud_terrastodon_core_user_input::prelude::Choice;
use cloud_terrastodon_core_user_input::prelude::FzfArgs;
use diesel::PgConnection;
use itertools::Itertools;
use nanuak_schema::git_models::ClonedRepo;
use tracing::debug;
use crate::get_repo_list_from_db::get_repo_list_from_db;

pub async fn pick_repo(conn: &mut PgConnection) -> eyre::Result<ClonedRepo> {
    let repos = get_repo_list_from_db(conn).await?;
    let repo = pick(FzfArgs {
        choices: repos
            .into_iter()
            .map(|repo| Choice {
                key: format!("{:120} {}", repo.path, repo.remotes.lines().next().unwrap_or("")),
                value: repo,
            })
            .collect_vec(),
        header: Some("Pick a repo to summarize".to_string()),
        ..Default::default()
    })?;
    debug!("Picked repo: {:?}", repo);
    Ok(repo.value)
}
