use clap::Parser;
use color_eyre::eyre::Result;
use color_eyre::eyre::WrapErr;
use ignore::WalkBuilder;
use itertools::Itertools;
use std::collections::HashSet;
use std::fs::OpenOptions;
use std::fs::{self};
use std::io::BufRead;
use std::io::BufReader;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::process::Stdio;
use tracing::debug;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::EnvFilter;

mod fzf;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// If set, enable debug logging
    #[arg(long)]
    debug: bool,

    /// Path to the file containing the list of paths we want to feed to the LLM
    #[arg(long, default_value = "files.txt")]
    files_txt: PathBuf,

    /// The path in which we'll search for unignored files
    #[arg(long, default_value = ".")]
    path: PathBuf,

    /// The markdown file to write the LLM prompt into
    #[arg(long, default_value = "prompt.md")]
    output_file: PathBuf,

    /// Editor to open the LLM prompt file with
    #[arg(long, default_value = "code")]
    editor: String,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Setup logging
    let log_level = if cli.debug {
        LevelFilter::DEBUG
    } else {
        LevelFilter::INFO
    };
    let env_filter = EnvFilter::builder()
        .with_default_directive(log_level.into())
        .from_env_lossy();
    tracing_subscriber::fmt().with_env_filter(env_filter).init();
    color_eyre::install()?;

    // Read existing files from files.txt into a Set<PathBuf>
    let mut tracked_files = read_tracked_files(&cli.files_txt)?;

    // The interactive menu actions
    let add = "Add";
    let remove = "Remove";
    let show = "Show";
    let write_prompt = "WritePrompt";
    let quit = "Quit";
    let actions = vec![add, remove, show, write_prompt, quit];

    loop {
        let action = fzf::pick(fzf::FzfArgs {
            choices: actions.clone(),
            prompt: Some("Choose an action".to_string()),
            header: None,
        })?;
        debug!("Chose action: {}", action);

        match action {
            "Add" => {
                // Let user pick multiple unignored files not already in our set
                let new_files = pick_files_to_add(&cli.path, &tracked_files)?;
                for f in new_files {
                    tracked_files.insert(f);
                }
                write_tracked_files(&cli.files_txt, &tracked_files)?;
            }
            "Remove" => {
                // Let user pick from the currently tracked set
                let removed_files = pick_files_to_remove(&tracked_files)?;
                for f in removed_files {
                    tracked_files.remove(&f);
                }
                write_tracked_files(&cli.files_txt, &tracked_files)?;
            }
            "Show" => {
                show_tracked_files(&tracked_files)?;
            }
            "WritePrompt" => {
                write_prompt_file(&cli.output_file, &tracked_files)?;
                open_in_editor(&cli.editor, &cli.output_file)?;
            }
            "AddCrate" => {
                let cargo_home = std::env::args().get("CARGO_HOME");
            }
            "Quit" => break,
            _ => unreachable!(),
        }
    }

    Ok(())
}

/// Reads the lines from `files.txt` into a `HashSet<PathBuf>`
/// Ignores empty lines.
fn read_tracked_files(path: &Path) -> Result<HashSet<PathBuf>> {
    let mut set = HashSet::new();
    if path.exists() {
        let file = fs::File::open(path)
            .wrap_err_with(|| format!("Could not open {:?}", path.display()))?;
        let reader = BufReader::new(file);

        for line_result in reader.lines() {
            let line = line_result?;
            let line = line.trim();
            if !line.is_empty() {
                set.insert(PathBuf::from(line));
            }
        }
    }
    Ok(set)
}

/// Writes out the `tracked_files` to `files.txt`.
fn write_tracked_files(path: &Path, tracked_files: &HashSet<PathBuf>) -> Result<()> {
    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(path)
        .wrap_err_with(|| format!("Could not open file for writing: {:?}", path.display()))?;

    // We can choose any ordering, but let's sort by display to keep them consistent
    let mut entries: Vec<_> = tracked_files.iter().collect();
    entries.sort_by_key(|p| p.display().to_string());

    for entry in entries {
        writeln!(file, "{}", entry.display())?;
    }

    Ok(())
}

/// Let user pick multiple new files to add using FZF, skipping those already in the set.
fn pick_files_to_add(base_path: &Path, tracked_files: &HashSet<PathBuf>) -> Result<Vec<PathBuf>> {
    debug!("Picking files to add from {:?}", base_path);
    let all_unignored = get_unignored_files(base_path)?;
    let filtered = all_unignored
        .into_iter()
        .filter(|path| !tracked_files.contains(path))
        .collect::<Vec<_>>();

    if filtered.is_empty() {
        // No new files to pick
        println!("No untracked files found.");
        return Ok(vec![]);
    }

    let chosen = fzf::pick_many(fzf::FzfArgs {
        choices: filtered
            .into_iter()
            .map(|path| fzf::Choice {
                key: path.display().to_string(),
                value: path,
            })
            .collect(),
        prompt: Some("Select files to add".to_string()),
        header: None,
    })?
    .into_iter()
    .map(|x| x.value)
    .collect();
    Ok(chosen)
}

/// Let user pick multiple from the currently tracked set to remove.
fn pick_files_to_remove(tracked_files: &HashSet<PathBuf>) -> Result<Vec<PathBuf>> {
    if tracked_files.is_empty() {
        println!("No files to remove.");
        return Ok(vec![]);
    }

    // Convert to a Vec to hand over to FZF
    let choices = tracked_files.iter().cloned().collect::<Vec<_>>();
    let chosen = fzf::pick_many(fzf::FzfArgs {
        choices: choices
            .into_iter()
            .map(|path| fzf::Choice {
                key: path.display().to_string(),
                value: path,
            })
            .collect(),
        prompt: Some("Select files to remove".to_string()),
        header: None,
    })?
    .into_iter()
    .map(|x| x.value)
    .collect();
    Ok(chosen)
}

/// Show tracked files in the terminal
fn show_tracked_files(tracked_files: &HashSet<PathBuf>) -> eyre::Result<()> {
    if tracked_files.is_empty() {
        println!("No tracked files.");
        return Ok(());
    }
    let mut entries: Vec<_> = tracked_files.iter().collect();
    entries.sort_by_key(|p| p.display().to_string());

    let _ = fzf::pick(fzf::FzfArgs {
        choices: entries
            .into_iter()
            .map(|p| fzf::Choice {
                key: p.display().to_string(),
                value: p,
            })
            .collect(),
        prompt: Some("Press Enter to continue".to_string()),
        header: Some("Tracked files".to_string()),
    })?;
    Ok(())
}

/// Gathers unignored files by walking the directory using `ignore::WalkBuilder`.
fn get_unignored_files(base_path: &Path) -> Result<Vec<PathBuf>> {
    let mut choices = Vec::new();
    for result in WalkBuilder::new(base_path).build() {
        if let Ok(entry) = result {
            if entry.file_type().map_or(false, |ft| ft.is_file()) {
                choices.push(entry.path().to_path_buf());
            }
        }
    }
    Ok(choices)
}

/// Writes out the prompt file content, mimicking the idea from the powershell script.
fn write_prompt_file(output_file: &Path, tracked_files: &HashSet<PathBuf>) -> Result<()> {
    let mut prompt_content = String::from("# File summary\n\n");

    // We'll sort them just for consistent ordering
    let mut sorted_files: Vec<_> = tracked_files.iter().collect();
    sorted_files.sort_by_key(|p| p.display().to_string());

    for path in sorted_files {
        let extension = path
            .extension()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        // Attempt to read the file contents
        let contents =
            fs::read_to_string(path).unwrap_or_else(|_| "<Could not read file>".to_string());

        prompt_content.push_str(&format!("## {}\n", path.display()));
        prompt_content.push_str("````");
        prompt_content.push_str(&extension);
        prompt_content.push_str("\n");
        prompt_content.push_str(&contents);
        prompt_content.push_str("\n`````````\n\n");
    }

    fs::write(output_file, prompt_content)
        .wrap_err_with(|| format!("Failed to write to {:?}", output_file.display()))?;

    println!("Wrote prompt to: {}", output_file.display());
    Ok(())
}

/// Open the prompt file in the user's chosen editor.
fn open_in_editor(editor: &str, file: &Path) -> Result<()> {
    Command::new(editor)
        .arg(file)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .wrap_err_with(|| format!("Failed to open editor: {} {:?}", editor, file.display()))?;
    Ok(())
}
