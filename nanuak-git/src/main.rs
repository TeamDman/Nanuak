use clap::{Parser, Subcommand};
use color_eyre::eyre::Result;
use tracing::{info, warn};
use tracing_subscriber::EnvFilter;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(version, about = "Nanuak Git CLI")]
struct Cli {
    /// If set, enable debug logging
    #[arg(long)]
    debug: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Crawl local directory for Git repos
    Crawl {
        /// The starting directory to search
        #[arg(long, default_value = ".")]
        dir: PathBuf,
    },
    // Future subcommand placeholders
    // e.g. "sync", "fetch-remote", etc.
}

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    // Parse CLI
    let cli = Cli::parse();

    // Setup logging
    let log_level = if cli.debug { "DEBUG" } else { "INFO" };
    let env_filter = EnvFilter::builder()
        .with_default_directive(log_level.parse().unwrap_or_else(|_| "INFO".parse().unwrap()))
        .from_env_lossy();
    tracing_subscriber::fmt().with_env_filter(env_filter).init();

    match cli.command {
        Commands::Crawl { dir } => {
            crawl_git_repos(dir).await?;
        }
    }

    Ok(())
}

async fn crawl_git_repos(start_dir: PathBuf) -> Result<()> {
    info!("Crawling for Git repos in {:?}", start_dir);
    // 1) Validate the path
    if !start_dir.exists() {
        return Err(color_eyre::eyre::eyre!(
            "Path does not exist: {:?}",
            start_dir
        ));
    }

    // 2) Use `ignore` crate or simple recursion to find .git folders
    //    For instance:
    //    WalkBuilder::new(&start_dir).build() ...

    // For demonstration, let’s show a quick approach:
    use ignore::WalkBuilder;
    for result in WalkBuilder::new(&start_dir).build() {
        let entry = match result {
            Ok(e) => e,
            Err(e) => {
                warn!("Error reading entry: {:?}", e);
                continue;
            }
        };
        if entry.file_type().map_or(false, |ft| ft.is_dir()) {
            let path = entry.path();
            // If path/.git exists or path is a .git folder
            if path.join(".git").is_dir() {
                // Found a Git repo. Now let's gather info
                info!("Found git repo at {:?}", path);
                // You could call `git remote -v` or parse .git/config
                // e.g. using tokio::process::Command
                // or using the `git2` crate

                // For example, let's do a quick Command approach:
                // gather_repo_info(path).await?;
            }
        }
    }

    Ok(())
}

// async fn gather_repo_info(repo_path: &Path) -> Result<RepoInfo> {
//     // Some pseudo-code:
//     // let output = tokio::process::Command::new("git")
//     //     .arg("-C")
//     //     .arg(repo_path)
//     //     .arg("remote")
//     //     .arg("-v")
//     //     .output()
//     //     .await?;
//     // parse the lines to find the remote name + url
//     // ...
//     Ok(RepoInfo {
//         remote_urls: vec![],
//         // ...
//     })
// }

// struct RepoInfo {
//     remote_urls: Vec<String>,
//     // ...
// }
