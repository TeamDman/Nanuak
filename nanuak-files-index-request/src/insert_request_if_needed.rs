use diesel::insert_into;
use diesel::prelude::*;
use nanuak_schema::files_models::NewRequest;
use tracing::info;

use crate::request_kind::RequestKind;

pub fn insert_request_if_needed(
    conn: &mut PgConnection,
    file_id_val: i32,
    request_kind: RequestKind,
    model_val: &str,
    prompt_val: Option<&str>,
) -> eyre::Result<()> {
    use nanuak_schema::files::captions::dsl as captions_dsl;
    use nanuak_schema::files::embeddings_512::dsl as embeddings_dsl;
    use nanuak_schema::files::requests::dsl::*;

    // Check if the embedding or caption already exists
    let exists = match request_kind {
        RequestKind::Caption => diesel::select(diesel::dsl::exists(
            captions_dsl::captions
                .filter(captions_dsl::file_id.eq(file_id_val))
                .filter(captions_dsl::model.eq(model_val))
                .filter(captions_dsl::prompt.eq(prompt_val)),
        ))
        .get_result::<bool>(conn)?,
        RequestKind::Embed => diesel::select(diesel::dsl::exists(
            embeddings_dsl::embeddings_512
                .filter(embeddings_dsl::file_id.eq(file_id_val))
                .filter(embeddings_dsl::model.eq(model_val)),
        ))
        .get_result::<bool>(conn)?,
    };

    if exists {
        info!(
            "Already have {} for (file_id={}, model='{}'), skipping request.",
            request_kind, file_id_val, model_val
        );
        return Ok(());
    }

    // Insert into requests
    let new_request = NewRequest {
        file_id: file_id_val,
        request_type: &request_kind.to_string(),
        model: model_val,
    };

    insert_into(requests).values(&new_request).execute(conn)?;

    info!(
        "Created request: (file_id={}, kind='{}', model='{}')",
        file_id_val, request_kind, model_val
    );
    Ok(())
}
