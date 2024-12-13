// @generated automatically by Diesel CLI.

pub mod youtube {
    pub mod sql_types {
        #[derive(diesel::query_builder::QueryId, Clone, diesel::sql_types::SqlType)]
        #[diesel(postgres_type(name = "tsvector", schema = "pg_catalog"))]
        pub struct Tsvector;

        #[derive(diesel::query_builder::QueryId, Clone, diesel::sql_types::SqlType)]
        #[diesel(postgres_type(name = "vector"))]
        pub struct Vector;
    }

    diesel::table! {
        use diesel::sql_types::*;
        use super::sql_types::Vector;

        youtube.channel_embeddings_bge_m3 (channel_id) {
            channel_id -> Text,
            embedded_on -> Timestamp,
            embedding -> Nullable<Vector>,
        }
    }

    diesel::table! {
        youtube.missing_videos (video_id) {
            video_id -> Text,
            fetched_on -> Timestamp,
        }
    }

    diesel::table! {
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
        youtube.search_history (time) {
            time -> Timestamp,
            #[max_length = 256]
            query -> Varchar,
        }
    }

    diesel::table! {
        youtube.video_categories (id) {
            id -> Text,
            title -> Text,
            assignable -> Bool,
            channel_id -> Text,
        }
    }

    diesel::table! {
        use diesel::sql_types::*;
        use super::sql_types::Vector;

        youtube.video_embeddings_bge_m3 (video_etag) {
            video_etag -> Text,
            embedded_on -> Timestamp,
            embedding -> Nullable<Vector>,
        }
    }

    diesel::table! {
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
        youtube.video_topics (id) {
            id -> Int4,
            video_etag -> Nullable<Text>,
            topic_url -> Text,
        }
    }

    diesel::table! {
        use diesel::sql_types::*;
        use super::sql_types::Tsvector;

        youtube.videos (etag) {
            etag -> Text,
            video_id -> Text,
            fetched_on -> Timestamp,
            title -> Text,
            description -> Nullable<Text>,
            published_at -> Nullable<Timestamp>,
            channel_id -> Text,
            channel_title -> Nullable<Text>,
            category_id -> Nullable<Text>,
            duration -> Nullable<Interval>,
            caption -> Nullable<Bool>,
            definition -> Nullable<Text>,
            dimension -> Nullable<Text>,
            licensed_content -> Nullable<Bool>,
            privacy_status -> Nullable<Text>,
            tags -> Nullable<Array<Nullable<Text>>>,
            view_count -> Nullable<Int8>,
            like_count -> Nullable<Int8>,
            comment_count -> Nullable<Int8>,
            search_document -> Nullable<Tsvector>,
        }
    }

    diesel::table! {
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
