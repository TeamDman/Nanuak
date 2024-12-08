// @generated automatically by Diesel CLI.

pub mod youtube {
    diesel::table! {
        youtube.search_history (time) {
            time -> Timestamp,
            #[max_length = 256]
            query -> Varchar,
        }
    }

    diesel::table! {
        youtube.watch_history (time) {
            time -> Timestamp,
            #[max_length = 16]
            youtube_video_id -> Varchar,
        }
    }

    diesel::allow_tables_to_appear_in_same_query!(
        search_history,
        watch_history,
    );
}
