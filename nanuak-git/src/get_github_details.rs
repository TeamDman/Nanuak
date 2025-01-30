use eyre::eyre;
use tokio::process::Command;

pub async fn get_github_details(github_repo: &str) -> eyre::Result<String> {
    let output = Command::new("gh")
        .args(["repo", "view", github_repo, "--json", "description,createdAt,languages,updatedAt,stargazerCount,watchers,licenseInfo,homepageUrl,primaryLanguage,projects,issues,pullRequests,milestones,forkCount"])
        .output()
        .await?;
    if !output.status.success() {
        // If the command fails, return an error
        return Err(eyre!(
            "stdout={}\nstderr={}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        )
        .wrap_err(color_eyre::eyre::eyre!(
            "gh repo view command failed for {:?}",
            github_repo
        )));
    }
    let stdout = String::from_utf8(output.stdout)?.trim().to_string();
    Ok(stdout)
}

#[cfg(test)]
mod tests {
    use super::*;
    use color_eyre::eyre::Result;

    #[tokio::test]
    async fn test_get_github_details() -> Result<()> {
        let github_repo = "rust-lang/rust";
        let output = get_github_details(github_repo).await?;
        println!("{}", output);
        assert!(!output.is_empty());
        Ok(())
    }
}
