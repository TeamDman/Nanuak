-- Your SQL goes here
-- Create a new schema or reuse an existing one, e.g. "image_embeddings"
CREATE SCHEMA IF NOT EXISTS captions;

-- Table to store caption requests
-- e.g., user or system requests a caption for a particular image
CREATE TABLE captions.caption_requests (
    id SERIAL PRIMARY KEY,
    image_path TEXT NOT NULL,
    request_time TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Table to store caption responses
-- e.g., the system that actually generates the captions
CREATE TABLE captions.caption_responses (
    id SERIAL PRIMARY KEY,
    caption_request_id INT NOT NULL REFERENCES captions.caption_requests(id) ON DELETE CASCADE,
    caption_text TEXT NOT NULL,
    response_time TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Alternatively, if you prefer to store final captions in a single table (like 'captions.captions'), do that.
-- For the sake of demonstration, we'll store them in 'caption_responses', which references the request.

-- Create channels (this is optional, Postgres doesn't store channels in migrations,
-- but it's helpful as documentation.)
-- We'll rely on NOTIFY "caption_request" and NOTIFY "caption_response"
-- (These do not persist; they're ephemeral. Just a helpful comment.)
