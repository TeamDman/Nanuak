use clap::Parser;
use tracing::{debug, info};
use tracing::level_filters::LevelFilter;
use tracing_subscriber::EnvFilter;
/// Command-line arguments for the tool
#[derive(Parser, Debug)]
#[command(version, about = "Ahoy!")]
struct Args {
    /// If set, enable debug logging
    #[arg(long)]
    debug: bool,
}

fn main() {
    let args = Args::parse();

    // Adjust logging based on `--debug` flag
    let log_level = if args.debug {
        LevelFilter::DEBUG
    } else {
        LevelFilter::INFO
    };

    let env_filter = EnvFilter::builder()
        .with_default_directive(log_level.into())
        .from_env_lossy();
    tracing_subscriber::fmt().with_env_filter(env_filter).init();
    info!("Ahoy!");
    debug!("Debug logging enabled");
    println!("Goodbye!");
}
