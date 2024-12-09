-- Main table to store YouTube video metadata
CREATE TABLE youtube.videos (
    etag TEXT PRIMARY KEY, -- ETag for this specific fetch
    video_id TEXT NOT NULL, -- YouTube video ID
    fetched_on TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP, -- When this data was fetched
    title TEXT NOT NULL, -- Video title
    description TEXT, -- Video description
    published_at TIMESTAMP, -- Video publication date
    channel_id TEXT NOT NULL, -- YouTube channel ID
    channel_title TEXT, -- YouTube channel name
    category_id TEXT, -- Video category ID
    duration INTERVAL, -- Video duration in a standard SQL format
    caption BOOLEAN, -- Whether captions are available
    definition TEXT, -- Video resolution (e.g., HD, SD)
    dimension TEXT, -- Video dimension (e.g., 2D, 3D)
    licensed_content BOOLEAN, -- Whether the content is licensed
    privacy_status TEXT, -- Privacy status (e.g., public, private)
    tags TEXT[], -- Array of tags
    view_count BIGINT, -- Number of views
    like_count BIGINT, -- Number of likes
    comment_count BIGINT -- Number of comments
);

-- Table to store video thumbnails
CREATE TABLE youtube.video_thumbnails (
    id SERIAL PRIMARY KEY, -- Unique identifier for this row
    video_etag TEXT REFERENCES youtube.videos(etag) ON DELETE CASCADE, -- Foreign key to youtube_videos
    size_description TEXT NOT NULL, -- Thumbnail size description (e.g., default, high, medium)
    height INTEGER, -- Thumbnail height
    width INTEGER, -- Thumbnail width
    url TEXT NOT NULL -- Thumbnail URL
);

-- Table to store topic categories
CREATE TABLE youtube.video_topics (
    id SERIAL PRIMARY KEY, -- Unique identifier for this row
    video_etag TEXT REFERENCES youtube.videos(etag) ON DELETE CASCADE, -- Foreign key to youtube_videos
    topic_url TEXT NOT NULL -- Topic URL (e.g., Wikipedia category URL)
);
