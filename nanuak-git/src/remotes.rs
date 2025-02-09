use std::ops::Deref;
use std::ops::DerefMut;
use std::str::FromStr;

use eyre::eyre;
use eyre::Context;
use eyre::OptionExt;
use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct RepoRemotes(Vec<RepoRemote>);
impl Deref for RepoRemotes {
    type Target = Vec<RepoRemote>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl DerefMut for RepoRemotes {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
impl FromStr for RepoRemotes {
    type Err = eyre::ErrReport;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let remotes = s
            .lines()
            .map(|line| try {
                let (name, without_name) = line.split_once('\t').ok_or_eyre(format!(
                    "Invalid remote line, failed to extract name: {}",
                    line
                ))?;
                let (url, usage) = without_name.rsplit_once(" (").ok_or_eyre(format!(
                    "Invalid remote line, failed to extract url and usage: {} (using chunk {})",
                    line, without_name
                ))?;
                let usage: RemoteUsage = {
                    let usage = usage.strip_suffix(")").ok_or_eyre(eyre!(
                        "Expected remote usage to be wrapped in parens, got {usage:?}"
                    ))?;
                    serde_json::from_str(usage)
                        .wrap_err(format!("Invalid remote usage, got {usage:?}"))?
                };
                RepoRemote {
                    name: name.to_string(),
                    url: url.to_string(),
                    usage,
                }
            })
            .collect::<eyre::Result<Vec<_>>>()?;
        Ok(RepoRemotes(remotes))
    }
}
impl From<RepoRemotes> for Vec<RepoRemote> {
    fn from(value: RepoRemotes) -> Self {
        value.0
    }
}
impl From<Vec<RepoRemote>> for RepoRemotes {
    fn from(value: Vec<RepoRemote>) -> Self {
        RepoRemotes(value)
    }
}
impl RepoRemotes {
    pub fn new(remotes: Vec<RepoRemote>) -> Self {
        RepoRemotes(remotes)
    }
}
impl std::fmt::Display for RepoRemotes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for remote in &self.0 {
            writeln!(f, "{}", remote)?;
        }
        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq)]
pub enum RemoteUsage {
    #[serde(rename = "fetch")]
    Fetch,
    #[serde(rename = "push")]
    Push,
}
impl std::fmt::Display for RemoteUsage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RemoteUsage::Fetch => write!(f, "fetch"),
            RemoteUsage::Push => write!(f, "push"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct RepoRemote {
    pub name: String,
    pub url: String,
    pub usage: RemoteUsage,
}
impl std::fmt::Display for RepoRemote {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}\t{} ({})", self.name, self.url, self.usage)
    }
}
