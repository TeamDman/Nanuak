// @generated automatically by Diesel CLI.

pub mod youtube {
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
        }
    }

    diesel::table! {
        youtube.watch_history (time) {
            time -> Timestamp,
            #[max_length = 16]
            youtube_video_id -> Varchar,
        }
    }

    diesel::joinable!(video_thumbnails -> videos (video_etag));
    diesel::joinable!(video_topics -> videos (video_etag));

    diesel::allow_tables_to_appear_in_same_query!(
        posts,
        search_history,
        video_categories,
        video_thumbnails,
        video_topics,
        videos,
        watch_history,
    );
}
