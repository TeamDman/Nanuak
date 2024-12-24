// @generated automatically by Diesel CLI.

pub mod files {
    diesel::table! {
        use diesel::sql_types::*;
        use pgvector::sql_types::*;
        use diesel_full_text_search::Tsvector;

        files.captions (id) {
            id -> Int4,
            file_id -> Int4,
            model -> Text,
            caption -> Text,
            created_at -> Timestamp,
        }
    }

    diesel::table! {
        use diesel::sql_types::*;
        use pgvector::sql_types::*;
        use diesel_full_text_search::Tsvector;

        files.embeddings_512 (id) {
            id -> Int4,
            file_id -> Int4,
            model -> Text,
            embedding -> Vector,
            created_at -> Timestamp,
        }
    }

    diesel::table! {
        use diesel::sql_types::*;
        use pgvector::sql_types::*;
        use diesel_full_text_search::Tsvector;

        files.files (id) {
            id -> Int4,
            path -> Text,
            file_size -> Int8,
            hash_value -> Text,
            hash_algorithm -> Text,
            seen_at -> Timestamp,
        }
    }

    diesel::table! {
        use diesel::sql_types::*;
        use pgvector::sql_types::*;
        use diesel_full_text_search::Tsvector;

        files.requests (id) {
            id -> Int4,
            file_id -> Int4,
            request_type -> Text,
            requested_at -> Timestamp,
            fulfilled_at -> Nullable<Timestamp>,
            model -> Nullable<Text>,
            error_message -> Nullable<Text>,
        }
    }

    diesel::joinable!(captions -> files (file_id));
    diesel::joinable!(embeddings_512 -> files (file_id));
    diesel::joinable!(requests -> files (file_id));

    diesel::allow_tables_to_appear_in_same_query!(
        captions,
        embeddings_512,
        files,
        requests,
    );
}
