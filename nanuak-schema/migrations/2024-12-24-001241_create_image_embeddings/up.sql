-- Your SQL goes here
CREATE SCHEMA IF NOT EXISTS image_embeddings;

CREATE TABLE image_embeddings.embeddings (
    hash_value TEXT PRIMARY KEY,
    hash_algorithm TEXT NOT NULL,
    embedding VECTOR(768) NOT NULL
);
