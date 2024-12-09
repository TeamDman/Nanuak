-- Create the video_categories table
CREATE TABLE youtube.video_categories (
    id TEXT PRIMARY KEY,               -- Unique category ID
    title TEXT NOT NULL,               -- Category title (e.g., "Film & Animation")
    assignable BOOLEAN NOT NULL,       -- Whether this category is assignable
    channel_id TEXT NOT NULL           -- Associated channel ID
);
