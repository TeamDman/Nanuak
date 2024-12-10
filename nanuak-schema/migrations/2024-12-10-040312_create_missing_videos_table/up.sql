-- Create a table for missing videos
CREATE TABLE youtube.missing_videos (
    video_id TEXT PRIMARY KEY,
    fetched_on TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);
