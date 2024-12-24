use clap::Parser;
use diesel::prelude::*;
use diesel::r2d2::ConnectionManager;
use diesel::r2d2::Pool;
use diesel::sql_query;
use diesel::sql_types::BigInt;
use diesel::sql_types::Int4;
use diesel::sql_types::Text;
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
    #[arg(long)]
    request_embedding: bool,

    /// Whether to request a caption
    #[arg(long)]
    request_caption: bool,

    /// The model to request for embedding/caption (if relevant)
    #[arg(long, default_value = "clip-vit-base-patch32")]
    model: String,
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
    let inserted_file: models::InsertedFile = sql_query(
        r#"
        INSERT INTO files.files (path, file_size, hash_value, hash_algorithm)
        VALUES ($1, $2, $3, $4)
        ON CONFLICT (path) DO UPDATE
            SET file_size = EXCLUDED.file_size,
                hash_value = EXCLUDED.hash_value,
                hash_algorithm = EXCLUDED.hash_algorithm,
                seen_at = CURRENT_TIMESTAMP
        RETURNING id
        "#,
    )
    .bind::<Text, _>(&file_path_str)
    .bind::<BigInt, _>(file_size)
    .bind::<Text, _>(&hash_value)
    .bind::<Text, _>(hash_algo)
    .get_result(&mut conn)?;

    let file_id = inserted_file.id;
    println!("File record ID = {}", file_id);

    // 3) Optionally create requests
    if args.request_embedding {
        insert_request_if_needed(&mut conn, file_id, "embed", &args.model)?;
    }
    if args.request_caption {
        insert_request_if_needed(&mut conn, file_id, "caption", &args.model)?;
    }

    println!("Requests created (if needed).");
    Ok(())
}

fn insert_request_if_needed(
    conn: &mut PgConnection,
    file_id: i32,
    request_type: &str,
    model: &str,
) -> eyre::Result<()> {
    use diesel::sql_query;
    use models::ExistsResult;

    // Determine the table to check based on request_type
    let table = if request_type == "caption" {
        "captions"
    } else {
        "embeddings"
    };

    // Check if an embedding or caption already exists for this file and model
    let query = format!(
        r#"
        SELECT EXISTS (
            SELECT 1 FROM files.{} 
            WHERE file_id = $1 AND model = $2
        ) AS exists
        "#,
        table
    );

    let exists_result: ExistsResult = sql_query(query)
        .bind::<Int4, _>(file_id)
        .bind::<Text, _>(model)
        .get_result(conn)?;

    if exists_result.exists {
        println!(
            "Already have {} for (file_id={}, model='{}'), skipping request.",
            request_type, file_id, model
        );
        return Ok(());
    }

    // Insert into requests
    sql_query(
        r#"
        INSERT INTO files.requests(file_id, request_type, model)
        VALUES ($1, $2, $3)
        "#,
    )
    .bind::<Int4, _>(file_id)
    .bind::<Text, _>(request_type)
    .bind::<Text, _>(model)
    .execute(conn)?;

    println!(
        "Created request: (file_id={}, type='{}', model='{}')",
        file_id, request_type, model
    );
    Ok(())
}

mod models {
    use diesel::sql_types::Bool;
    use diesel::sql_types::Int4;
    use diesel::QueryableByName;

    #[derive(Debug, QueryableByName)]
    pub struct InsertedFile {
        #[diesel(sql_type = Int4)]
        pub id: i32,
    }

    #[derive(Debug, QueryableByName)]
    pub struct ExistsResult {
        #[diesel(sql_type = Bool)]
        pub exists: bool,
    }
}
