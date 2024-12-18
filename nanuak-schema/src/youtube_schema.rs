// @generated automatically by Diesel CLI.

pub mod youtube {
    diesel::table! {
        use diesel::sql_types::*;
        use pgvector::sql_types::*;
        use diesel_full_text_search::Tsvector;

        youtube.channel_embeddings_bge_m3 (channel_id) {
            channel_id -> Text,
            embedded_on -> Timestamp,
            embedding -> Nullable<Vector>,
        }
    }

    diesel::table! {
        use diesel::sql_types::*;
        use pgvector::sql_types::*;
        use diesel_full_text_search::Tsvector;

        youtube.missing_videos (video_id) {
            video_id -> Text,
            fetched_on -> Timestamp,
        }
    }

    diesel::table! {
        use diesel::sql_types::*;
        use pgvector::sql_types::*;
        use diesel_full_text_search::Tsvector;

        youtube.posts (time) {
            time -> Timestamp,
            #[max_length = 8192]
            post_title -> Varchar,
            post_url -> Text,
            channel_url -> Text,
            #[max_length = 128]
            channel_name -> Varchar,
        }
    }

    diesel::table! {
        use diesel::sql_types::*;
        use pgvector::sql_types::*;
        use diesel_full_text_search::Tsvector;

        youtube.search_history (time) {
            time -> Timestamp,
            #[max_length = 256]
            query -> Varchar,
        }
    }

    diesel::table! {
        use diesel::sql_types::*;
        use pgvector::sql_types::*;
        use diesel_full_text_search::Tsvector;

        youtube.video_categories (id) {
            id -> Text,
            title -> Text,
            assignable -> Bool,
            channel_id -> Text,
        }
    }

    diesel::table! {
        use diesel::sql_types::*;
        use pgvector::sql_types::*;
        use diesel_full_text_search::Tsvector;

        youtube.video_embeddings_bge_m3 (video_etag) {
            video_etag -> Text,
            embedded_on -> Timestamp,
            embedding -> Nullable<Vector>,
        }
    }

    diesel::table! {
        use diesel::sql_types::*;
        use pgvector::sql_types::*;
        use diesel_full_text_search::Tsvector;

        youtube.video_thumbnails (id) {
            id -> Int4,
            video_etag -> Nullable<Text>,
            size_description -> Text,
            height -> Nullable<Int4>,
            width -> Nullable<Int4>,
            url -> Text,
        }
    }

    diesel::table! {
        use diesel::sql_types::*;
        use pgvector::sql_types::*;
        use diesel_full_text_search::Tsvector;

        youtube.video_topics (id) {
            id -> Int4,
            video_etag -> Nullable<Text>,
            topic_url -> Text,
        }
    }

    diesel::table! {
        use diesel::sql_types::*;
        use pgvector::sql_types::*;
        use diesel_full_text_search::Tsvector;

        youtube.videos (etag) {
            etag -> Text,
            video_id -> Text,
            fetched_on -> Timestamp,
            title -> Text,
            description -> Text,
            published_at -> Timestamp,
            channel_id -> Text,
            channel_title -> Text,
            category_id -> Text,
            duration -> Interval,
            caption -> Bool,
            definition -> Text,
            dimension -> Text,
            licensed_content -> Bool,
            privacy_status -> Text,
            tags -> Nullable<Array<Nullable<Text>>>,
            view_count -> Nullable<Int8>,
            like_count -> Nullable<Int8>,
            comment_count -> Nullable<Int8>,
            search_document -> Tsvector,
        }
    }

    diesel::table! {
        use diesel::sql_types::*;
        use pgvector::sql_types::*;
        use diesel_full_text_search::Tsvector;

        youtube.watch_history (time) {
            time -> Timestamp,
            #[max_length = 16]
            youtube_video_id -> Varchar,
        }
    }

    diesel::joinable!(video_embeddings_bge_m3 -> videos (video_etag));
    diesel::joinable!(video_thumbnails -> videos (video_etag));
    diesel::joinable!(video_topics -> videos (video_etag));

    diesel::allow_tables_to_appear_in_same_query!(
        channel_embeddings_bge_m3,
        missing_videos,
        posts,
        search_history,
        video_categories,
        video_embeddings_bge_m3,
        video_thumbnails,
        video_topics,
        videos,
        watch_history,
    );
}
