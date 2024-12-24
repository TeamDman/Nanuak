-- Your SQL goes here
CREATE TABLE IF NOT EXISTS image_embeddings.embedding_requests (
    id SERIAL PRIMARY KEY,
    image_path TEXT NOT NULL,
    image_hash TEXT NOT NULL,
    image_hash_algo TEXT NOT NULL,
    request_complete_time TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);
