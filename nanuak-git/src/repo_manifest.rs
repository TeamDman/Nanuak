use std::path::PathBuf;

#[derive(Debug)]
pub struct RepoManifest {
    pub github_details: String,
    pub notable_file_contents: Vec<(PathBuf, String)>,
}
