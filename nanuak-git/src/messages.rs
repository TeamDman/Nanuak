#[derive(Debug)]
pub enum CrawlMessage {
    FoundRepo {
        path: std::path::PathBuf,
        remotes: String,
    },
    Done,
    Error(eyre::ErrReport),
}
