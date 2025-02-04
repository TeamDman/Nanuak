use clap::Parser;
use eyre::{eyre, Context};
use std::fs;
use std::path::PathBuf;
use std::str::FromStr;
use vtt::prelude::*;

/// Simple program to output the plaintext from a WebVTT file.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to the WebVTT file to process
    #[arg(short, long, value_name = "FILE")]
    file: PathBuf,
}

fn main() -> eyre::Result<()> {
    color_eyre::install()?;
    // Parse the command-line arguments
    let args = Args::parse();

    // Read the entire contents of the provided VTT file
    let content = fs::read_to_string(&args.file)?;

    // Parse the content into a WebVtt instance
    let vtt = WebVtt::from_str(&content).wrap_err(eyre!("Failed to parse VTT file"))?;

    println!("{}", vtt.deduplicated_text());
    Ok(())
}
