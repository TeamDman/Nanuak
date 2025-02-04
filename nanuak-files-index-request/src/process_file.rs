use diesel::dsl::now;
use diesel::insert_into;
use diesel::prelude::*;
use diesel::upsert::excluded;
use nanuak_schema::files_models::NewFile;
use sha2::Digest;
use sha2::Sha256;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use tracing::info;

use crate::args::Args;
use crate::insert_request_if_needed::insert_request_if_needed;
use crate::request_kind::RequestKind;

pub fn process_file(conn: &mut PgConnection, file_path: &PathBuf, args: &Args) -> eyre::Result<()> {
    // 1) Gather file info: size, sha256
    let metadata = std::fs::metadata(file_path)?;
    let file_size = metadata.len() as i64;

    let mut file = File::open(file_path)?;
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
    let file_path_str = file_path.to_string_lossy().to_string();

    // Perform the INSERT ... RETURNING id query
    let new_file = NewFile {
        path: &file_path_str,
        file_size,
        hash_value: &hash_value,
        hash_algorithm: hash_algo,
    };

    use nanuak_schema::files::files::dsl as files_dsl;

    let file_id: i32 = insert_into(files_dsl::files)
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
        .get_result(conn)?;
    info!("File record ID = {}", file_id);

    // 3) Optionally create requests
    if args.embed {
        insert_request_if_needed(
            conn,
            file_id,
            RequestKind::Embed,
            &args.embedding_model,
            args.prompt.as_deref(),
        )?;
    }
    if args.caption {
        insert_request_if_needed(
            conn,
            file_id,
            RequestKind::Caption,
            &args.captioning_model,
            args.prompt.as_deref(),
        )?;
    }

    Ok(())
}
