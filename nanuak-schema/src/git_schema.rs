// @generated automatically by Diesel CLI.

pub mod git {
    diesel::table! {
        use diesel::sql_types::*;
        use pgvector::sql_types::*;
        use diesel_full_text_search::Tsvector;

        git.cloned_repos (path) {
            path -> Text,
            remotes -> Text,
            seen -> Timestamp,
        }
    }
}
