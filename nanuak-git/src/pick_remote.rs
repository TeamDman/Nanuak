use cloud_terrastodon_core_user_input::prelude::pick;
use cloud_terrastodon_core_user_input::prelude::Choice;
use cloud_terrastodon_core_user_input::prelude::FzfArgs;
use eyre::OptionExt;
use itertools::Itertools;

pub fn pick_remote(remotes: &str) -> eyre::Result<(&str, &str, &str)> {
    let remotes = remotes
        .lines()
        .map(|line| {
            // line.split('\t')
            //     .into_iter()
            //     .collect_tuple::<(_, _, _)>()
            //     .ok_or_eyre(format!("Invalid remote line: {}", line))
            try {
                let (name, without_name) = line.split_once('\t').ok_or_eyre(format!("Invalid remote line, failed to extract name: {}", line))?;
                let (url, usage) = without_name.rsplit_once(" (").ok_or_eyre(format!("Invalid remote line, failed to extract url and usage: {} (using chunk {})", line, without_name))?;
                (name, url, &usage[..usage.len() - 1])
            }
        })
        .collect::<eyre::Result<Vec<_>>>()?;

    let chosen_remote = pick(FzfArgs {
        choices: remotes
            .into_iter()
            .map(|(name, url, usage)| Choice {
                key: format!("{:40} {} ({})", name, url, usage),
                value: (name, url, usage),
            })
            .collect_vec(),
        header: Some("Pick a remote to summarize".to_string()),
        ..Default::default()
    })?;
    Ok(chosen_remote.value)
}
