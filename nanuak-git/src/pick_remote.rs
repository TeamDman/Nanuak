use crate::remotes::RepoRemote;
use crate::remotes::RepoRemotes;
use cloud_terrastodon_core_user_input::prelude::pick;
use cloud_terrastodon_core_user_input::prelude::FzfArgs;

pub fn pick_remote(remotes: &str) -> eyre::Result<RepoRemote> {
    let remotes = remotes.parse::<RepoRemotes>()?;
    let chosen_remote = pick(FzfArgs {
        choices: remotes.into(),
        header: Some("Pick a remote to summarize".to_string()),
        ..Default::default()
    })?;
    Ok(chosen_remote)
}
