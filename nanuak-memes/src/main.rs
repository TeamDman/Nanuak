// src/main.rs

#[macro_use]
extern crate diesel;

use base64::engine::general_purpose;
use base64::Engine as _;
use clap::Parser;
use diesel::prelude::*;
use diesel::r2d2::ConnectionManager;
use diesel::r2d2::Pool;
use dotenvy::dotenv;
use reqwest::Client;
use serde::Deserialize;
use serde::Serialize;
use std::env;
use std::fs;
use std::io::Write;
use std::io::{self};
use std::time::Duration;
use walkdir::WalkDir;

use std::error::Error;

// --- Diesel Schema ---
// (You can also use `diesel print-schema` if you have the migration in place.)
mod schema {
    table! {
        memes (id) {
            id -> Integer,
            file_path -> Text,
            description -> Text,
        }
    }
}

// --- Models ---
mod models {
    use super::schema::memes;
    #[derive(Queryable, Debug)]
    #[allow(dead_code)]
    pub struct Meme {
        pub id: i32,
        pub file_path: String,
        pub description: String,
    }

    #[derive(Insertable, Debug)]
    #[diesel(table_name = memes)]
    pub struct NewMeme<'a> {
        pub file_path: &'a str,
        pub description: &'a str,
    }
}

use models::Meme;
use models::NewMeme;
use schema::memes::dsl::*;

/// A simple interactive CLI for indexing and querying memes.
///
/// There are two modes:
/// 1. **Index a folder:** Walk a folder and for each image file, call Ollama’s vision endpoint
///    (for example using the model `x/llama3.2-vision:latest`) with the prompt:
///
///    > "Describe this meme for a visually impaired person and explain why it is funny."
///
///    The returned description is stored (with the file path) in a database.
///
/// 2. **Query memes:** Enter a search term, and the program lists the top 5 matches (by doing a
///    case‑insensitive search against the stored descriptions). Then you can choose one to open
///    (the file is opened with your default image viewer on Windows).
///
/// **Example meme description (sample):**
///
/// > The meme shows two people embracing. On the man’s chest (labeled “me”) and his right arm
/// > (labeled “movie I heard of three minutes ago”) versus the woman’s chest (labeled “movies on my
/// > watchlist for months”). The humor comes from the relatable contrast between someone who is
/// > spontaneous and watches what’s new versus someone who has long delayed watching the movies
/// > they’ve been saving.
#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    // (For now we don’t add any CLI arguments because the program is interactive.)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Load environment variables from .env (including DATABASE_URL)
    dotenv().ok();
    // Set up database connection pool (Postgres is assumed)
    let database_url = env::var("DATABASE_URL")?;
    let manager = ConnectionManager::<diesel::pg::PgConnection>::new(database_url);
    let pool = Pool::builder().build(manager)?;

    loop {
        println!();
        println!("Select mode: [1] Index folder, [2] Query memes, [q] Quit");
        print!("Enter choice: ");
        io::stdout().flush()?;
        let mut choice = String::new();
        io::stdin().read_line(&mut choice)?;
        let choice = choice.trim();
        match choice {
            "1" => {
                println!("Enter the directory path to index:");
                let mut dir = String::new();
                io::stdin().read_line(&mut dir)?;
                let dir = dir.trim();
                if dir.is_empty() {
                    println!("Directory path cannot be empty.");
                    continue;
                }
                index_folder(dir, &pool).await?;
            }
            "2" => {
                query_memes(&pool)?;
            }
            "q" | "Q" => break,
            _ => println!("Invalid choice."),
        }
    }
    Ok(())
}

/// Walk the directory, process image files, call the Ollama API for a description,
/// and save/update the entry in the `memes` table.
async fn index_folder(
    dir: &str,
    pool: &Pool<ConnectionManager<diesel::pg::PgConnection>>,
) -> Result<(), Box<dyn Error>> {
    println!("Indexing folder: {}", dir);
    let client = Client::new();
    let ollama_url = "http://localhost:11434/api/generate";

    // Walk the directory recursively
    for entry in WalkDir::new(dir).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.is_file() {
            // Check file extension
            if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                let ext = ext.to_lowercase();
                if ext == "jpg" || ext == "jpeg" || ext == "png" || ext == "gif" {
                    let file_path_str = path.to_string_lossy();
                    println!("Processing file: {}", file_path_str);

                    // Read file and encode in base64
                    let bytes = fs::read(path)?;
                    let b64 = general_purpose::STANDARD.encode(&bytes);
                    // Determine MIME type based on extension
                    let mime = match ext.as_str() {
                        "jpg" | "jpeg" => "image/jpeg",
                        "png" => "image/png",
                        "gif" => "image/gif",
                        _ => "application/octet-stream",
                    };
                    let data_uri = format!("data:{};base64,{}", mime, b64);

                    // Prepare request payload.
                    #[derive(Serialize)]
                    struct GenerateRequest<'a> {
                        model: &'a str,
                        prompt: &'a str,
                        images: Vec<&'a str>,
                        stream: bool,
                    }
                    let prompt_text = "Describe this meme for a visually impaired person and explain why it is funny.";
                    let payload = GenerateRequest {
                        model: "x/llama3.2-vision:latest",
                        prompt: prompt_text,
                        images: vec![&data_uri],
                        stream: false,
                    };

                    // Send POST request to Ollama API
                    let resp = client.post(ollama_url).json(&payload).send().await?;
                    if !resp.status().is_success() {
                        println!("Failed to process image: {}", file_path_str);
                        continue;
                    }

                    // Parse the response JSON.
                    #[derive(Deserialize)]
                    struct GenerateResponse {
                        response: String,
                    }
                    let gen_resp: GenerateResponse = resp.json().await?;
                    println!("Description: {}", gen_resp.response);

                    // Insert (or update) into the database.
                    {
                        let conn = &mut pool.get()?;
                        // If an entry for this file already exists, update it; otherwise, insert a new one.
                        let existing: Option<Meme> = memes
                            .filter(file_path.eq(&file_path_str))
                            .first(conn)
                            .optional()?;
                        if existing.is_some() {
                            diesel::update(memes.filter(file_path.eq(&file_path_str)))
                                .set(description.eq(&gen_resp.response))
                                .execute(conn)?;
                        } else {
                            let new_meme = NewMeme {
                                file_path: &file_path_str,
                                description: &gen_resp.response,
                            };
                            diesel::insert_into(memes).values(&new_meme).execute(conn)?;
                        }
                    }
                    println!("Indexed: {}", file_path_str);
                    // Pause a bit between API calls.
                    tokio::time::sleep(Duration::from_millis(500)).await;
                }
            }
        }
    }
    println!("Indexing complete.");
    Ok(())
}

/// Enter query mode: ask the user for a search term, list the top 5 matches (based on description),
/// and let the user choose one to open using the default image viewer.
fn query_memes(
    pool: &Pool<ConnectionManager<diesel::pg::PgConnection>>,
) -> Result<(), Box<dyn Error>> {
    println!("Entering query mode. Type 'q' to quit.");
    loop {
        print!("Enter search term: ");
        io::stdout().flush()?;
        let mut term = String::new();
        io::stdin().read_line(&mut term)?;
        let term = term.trim();
        if term.eq_ignore_ascii_case("q") {
            break;
        }
        let conn = &mut pool.get()?;
        let pattern = format!("%{}%", term);
        let results = memes
            .filter(description.ilike(&pattern))
            .limit(5)
            .load::<Meme>(conn)?;
        if results.is_empty() {
            println!("No memes found matching '{}'", term);
        } else {
            println!("Top {} results:", results.len());
            for (i, m) in results.iter().enumerate() {
                println!("{}. {} - {}", i + 1, m.file_path, m.description);
            }
            print!("Enter the number of the meme to open (or press Enter to search again): ");
            io::stdout().flush()?;
            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            let input = input.trim();
            if !input.is_empty() {
                if let Ok(num) = input.parse::<usize>() {
                    if num >= 1 && num <= results.len() {
                        let file_to_open = &results[num - 1].file_path;
                        println!("Opening file: {}", file_to_open);
                        // Use the 'open' crate to launch the default viewer.
                        open::that(file_to_open)?;
                    } else {
                        println!("Invalid number.");
                    }
                } else {
                    println!("Invalid input.");
                }
            }
        }
    }
    Ok(())
}
