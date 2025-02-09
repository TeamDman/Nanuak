use std::path::PathBuf;

use directories::ProjectDirs;
use eyre::Context;
use eyre::Result;
use eyre::eyre;

pub fn get_project_dirs() -> Result<ProjectDirs> {
    ProjectDirs::from("ca", "teamdman", "nanuak")
        .ok_or_else(|| eyre!("Could not determine platform-specific project directory"))
}

pub async fn get_config_path() -> Result<PathBuf> {
    let proj_dirs = get_project_dirs()?;
    let config_dir = proj_dirs.config_dir();
    tokio::fs::create_dir_all(config_dir)
        .await
        .wrap_err_with(|| {
            format!(
                "Failed to create config directory: {}",
                config_dir.display()
            )
        })?;
    Ok(config_dir.join("config.toml"))
}
