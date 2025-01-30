use color_eyre::eyre::Result;
use eyre::eyre;
use std::path::Path;

pub async fn gather_repo_info(repo_path: &Path) -> Result<String> {
    // For demonstration, we fetch "origin" via `git remote get-url origin`
    use std::str;
    use tokio::process::Command;

    let output = Command::new("git")
        .arg("-C")
        .arg(repo_path)
        .arg("remote")
        .arg("--verbose")
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
            "git command failed for {:?}",
            repo_path
        )));
    }

    let stdout = str::from_utf8(&output.stdout)?.trim().to_string();
    Ok(stdout)
}
