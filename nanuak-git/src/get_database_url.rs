use eyre::WrapErr;  // for the .wrap_err() calls if you like to attach context
use tokio::process::Command;
use eyre::{eyre, Context, Result};
use dotenvy::{from_filename, var};
use std::fs::OpenOptions;
use std::io::Write;

pub fn load_env_and_ensure_dburl() -> Result<String> {
    // 1. Load .env into environment
    // This will do nothing if .env doesn't exist, but won't error out.
    // If you'd prefer an error, handle that separately.
    let _ = from_filename(".env");

    // 2. Check if DATABASE_URL is set in environment
    match var("DATABASE_URL") {
        Ok(url) => {
            // Already set, just return it
            Ok(url)
        }
        Err(_) => {
            // If missing, define your default / desired value.
            // For example, constructing a local dev URL:
            let default_url = format!("postgres://postgres:password@localhost/nanuak");

            // 3. Append the key=value line to `.env`
            let line_to_append = format!("\nDATABASE_URL={}", default_url);

            // Open or create the file in append mode
            let mut file = OpenOptions::new()
                .create(true)
                .append(true)
                .open(".env")
                .wrap_err("Failed to open or create .env file")?;

            file.write_all(line_to_append.as_bytes())
                .wrap_err("Failed writing DATABASE_URL to .env")?;

            // Also set it in the current process's environment so that subsequent calls see it
            std::env::set_var("DATABASE_URL", &default_url);

            Ok(default_url)
        }
    }
}

pub async fn get_database_url() -> eyre::Result<String> {
    // Check if DATABASE_URL is already set
    if let Ok(url) = std::env::var("DATABASE_URL") {
        return Ok(url);
    }

    // If not set, get the password from 1Password CLI
    let output = Command::new("op")
        .args(&["item", "get", "PostgreSQL Local", "--vault", "Private", "--field", "password"])
        .output()
        .await
        .wrap_err("Failed to run `op` command")?;

    if !output.status.success() {
        eyre::bail!(
            "Command `op` exited with status: {}",
            output.status.code().map(|x| x.to_string()).unwrap_or("unknown".to_string())
        );
    }

    // Convert the stdout to String
    let password = String::from_utf8(output.stdout)
        .wrap_err("Invalid UTF-8 in `op` command output")?
        .trim()
        .to_owned();

    // Construct the DATABASE_URL
    let url = format!("postgres://postgres:{}@localhost/nanuak", password);
    // Store it in the environment
    std::env::set_var("DATABASE_URL", &url);

    Ok(url)
}
