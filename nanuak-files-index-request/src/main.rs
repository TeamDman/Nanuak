use clap::Parser;
use diesel::dsl::now;
use diesel::insert_into;
use diesel::prelude::*;
use diesel::r2d2::ConnectionManager;
use diesel::r2d2::Pool;
use diesel::upsert::excluded;
use nanuak_schema::files_models::NewFile;
use nanuak_schema::files_models::NewRequest;
use sha2::Digest;
use sha2::Sha256;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    version,
    about = "Index a file and optionally request embedding/caption"
)]
struct Args {
    /// Path to the file to index
    file_path: PathBuf,

    /// Whether to request an embedding
    #[arg(long, default_value = "true")]
    embed: bool,

    /// Whether to request a caption
    #[arg(long, default_value = "true")]
    caption: bool,

    /// The model to request for embedding (if relevant)
    #[arg(long, default_value = "openai/clip-vit-base-patch32")]
    embedding_model: String,

    /// The model to request for caption (if relevant)
    #[arg(long, default_value = "Salesforce/blip-image-captioning-large")]
    captioning_model: String,


}

fn main() -> eyre::Result<()> {
    color_eyre::install()?;
    let args = Args::parse();

    // Setup DB connection
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let manager = ConnectionManager::<PgConnection>::new(&database_url);
    let pool = Pool::builder().build(manager)?;
    let mut conn = pool.get()?;

    // 1) Gather file info: size, sha256
    let metadata = std::fs::metadata(&args.file_path)?;
    let file_size = metadata.len() as i64;

    let mut file = File::open(&args.file_path)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 8192];
    loop {
        let bytes_read = file.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }
    let hash_value = format!("{:x}", hasher.finalize());
    let hash_algo = "sha256";

    // 2) Insert or find existing record in files.files
    let file_path_str = args.file_path.to_string_lossy().to_string();

    // Perform the INSERT ... RETURNING id query
    let new_file = NewFile {
        path: &file_path_str,
        file_size,
        hash_value: &hash_value,
        hash_algorithm: &hash_algo,
    };

    use nanuak_schema::files::files::dsl as files_dsl;

    let inserted_file: i32 = insert_into(files_dsl::files)
        .values(&new_file)
        .on_conflict(files_dsl::path)
        .do_update()
        .set((
            files_dsl::file_size.eq(excluded(files_dsl::file_size)),
            files_dsl::hash_value.eq(excluded(files_dsl::hash_value)),
            files_dsl::hash_algorithm.eq(excluded(files_dsl::hash_algorithm)),
            files_dsl::seen_at.eq(now),
        ))
        .returning(files_dsl::id)
        .get_result(&mut conn)?;

    let file_id = inserted_file;
    println!("File record ID = {}", file_id);

    // 3) Optionally create requests
    if args.embed {
        insert_request_if_needed(&mut conn, file_id, "embed", &args.embedding_model)?;
    }
    if args.caption {
        insert_request_if_needed(&mut conn, file_id, "caption", &args.captioning_model)?;
    }

    println!("Requests created (if needed).");
    Ok(())
}

fn insert_request_if_needed(
    conn: &mut PgConnection,
    file_id_val: i32,
    request_type_val: &str,
    model_val: &str,
) -> eyre::Result<()> {
    use nanuak_schema::files::captions::dsl as captions_dsl;
    use nanuak_schema::files::embeddings_512::dsl as embeddings_dsl;
    use nanuak_schema::files::requests::dsl::*;

    // Check if the embedding or caption already exists
    let exists = if request_type_val == "caption" {
        diesel::select(diesel::dsl::exists(
            captions_dsl::captions
                .filter(captions_dsl::file_id.eq(file_id_val))
                .filter(captions_dsl::model.eq(model_val)),
        ))
        .get_result::<bool>(conn)?
    } else {
        diesel::select(diesel::dsl::exists(
            embeddings_dsl::embeddings_512
                .filter(embeddings_dsl::file_id.eq(file_id_val))
                .filter(embeddings_dsl::model.eq(model_val)),
        ))
        .get_result::<bool>(conn)?
    };

    if exists {
        println!(
            "Already have {} for (file_id={}, model='{}'), skipping request.",
            request_type_val, file_id_val, model_val
        );
        return Ok(());
    }

    // Insert into requests
    let new_request = NewRequest {
        file_id: file_id_val,
        request_type: request_type_val,
        model: model_val,
    };

    insert_into(requests).values(&new_request).execute(conn)?;

    println!(
        "Created request: (file_id={}, type='{}', model='{}')",
        file_id_val, request_type_val, model_val
    );
    Ok(())
}
