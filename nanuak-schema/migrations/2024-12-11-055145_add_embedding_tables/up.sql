-- Ensure the vector extension is enabled (if not already)
-- CREATE EXTENSION IF NOT EXISTS vector;
CREATE EXTENSION vector;

-- Create a table for video embeddings
-- We assume a dimension of 1024 for the embedding vector; adjust as needed.
CREATE TABLE youtube.video_embeddings_bge_m3 (
    video_etag TEXT PRIMARY KEY REFERENCES youtube.videos(etag) ON DELETE CASCADE,
    embedded_on TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    embedding vector(1024)
);

-- Create a table for channel embeddings
-- We assume channel_id is just text for now. If you have a channels table, reference it similarly.
CREATE TABLE youtube.channel_embeddings_bge_m3 (
    channel_id TEXT PRIMARY KEY,
    embedded_on TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    embedding vector(1024)
);
