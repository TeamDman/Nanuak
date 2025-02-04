use clap::{Parser, ArgGroup};
use eyre::bail;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    version,
    about = "Index a file and optionally request embedding/caption",
    group = ArgGroup::new("input")
        .required(true)
        .args(&["file_path", "file_path_txt"])
)]
pub struct Args {
    /// Path to the file to index
    #[arg(long)]
    pub file_path: Option<PathBuf>,

    /// Path to a text file containing list of file paths to index
    #[arg(long)]
    pub file_path_txt: Option<PathBuf>,

    /// The prompt to use for captioning
    #[arg(long)]
    pub prompt: Option<String>,

    /// Path to a text file containing the prompt to use for captioning
    #[arg(long)]
    pub prompt_txt: Option<PathBuf>,

    /// Whether to request an embedding
    #[arg(long, default_value = "true")]
    pub embed: bool,

    /// Whether to request a caption
    #[arg(long, default_value = "true")]
    pub caption: bool,

    /// The model to request for embedding (if relevant)
    #[arg(long, default_value = "openai/clip-vit-base-patch32")]
    pub embedding_model: String,

    /// The model to request for caption (if relevant)
    #[arg(long, default_value = "Salesforce/blip-image-captioning-large")]
    pub captioning_model: String,

    /// Enable debug logging
    #[arg(long, action = clap::ArgAction::SetTrue)]
    pub debug: bool,
}
impl Args {
    pub fn get_file_paths(&self) -> eyre::Result<Vec<PathBuf>> {
        let file_paths = if let Some(txt_path) = &self.file_path_txt {
            let content = std::fs::read_to_string(txt_path)?;
            content
                .lines()
                .filter_map(|line| {
                    let trimmed = line.trim();
                    if trimmed.is_empty() {
                        None
                    } else {
                        // if line starts and ends with quotes, strip them.
                        let trimmed = if trimmed.starts_with('"') && trimmed.ends_with('"') {
                            &trimmed[1..trimmed.len() - 1]
                        } else {
                            trimmed
                        };
                        Some(PathBuf::from(trimmed))
                    }
                })
                .collect::<Vec<PathBuf>>()
        } else if let Some(single_path) = &self.file_path {
            vec![single_path.to_owned()]
        } else {
            bail!("No file paths provided");
        };
        Ok(file_paths)
    }
    pub fn get_prompt(&self) -> eyre::Result<String> {
        if let Some(prompt) = &self.prompt {
            Ok(prompt.clone())
        } else if let Some(prompt_txt) = &self.prompt_txt {
            let content = std::fs::read_to_string(prompt_txt)?;
            Ok(content.trim().to_string())
        } else {
            bail!("No prompt provided");
        }
    }
}