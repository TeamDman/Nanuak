DROP SCHEMA IF EXISTS image_embeddings CASCADE;
DROP SCHEMA IF EXISTS captions CASCADE;


CREATE SCHEMA IF NOT EXISTS files;

-- 1) The main "files" table: each row represents one unique file that Nanuak is aware of.
--    We store the hash, path, etc., so we don't re-ingest duplicates.
CREATE TABLE files.files (
    id              SERIAL PRIMARY KEY,
    path            TEXT NOT NULL UNIQUE,  -- local file system path
    file_size       BIGINT NOT NULL,
    hash_value      TEXT NOT NULL,
    hash_algorithm  TEXT NOT NULL,
    seen_at         TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- 2) Embeddings table: each row stores a vector embedding for a given file+model.
--    The "model" column can store "clip-vit-base-patch32" or "Salesforce/blip-image-captioning-large" or "bge-m3" etc.
CREATE TABLE files.embeddings_512 (
    id             SERIAL PRIMARY KEY,
    file_id        INT NOT NULL REFERENCES files.files(id) ON DELETE CASCADE,
    model          TEXT NOT NULL,
    embedding      VECTOR(512) NOT NULL,
    created_at     TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- 3) Captions table: each row stores a generated caption for a given file+model.
CREATE TABLE files.captions (
    id             SERIAL PRIMARY KEY,
    file_id        INT NOT NULL REFERENCES files.files(id) ON DELETE CASCADE,
    model          TEXT NOT NULL,
    caption        TEXT NOT NULL,
    created_at     TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- 4) Requests table: a generic request for either "caption" or "embedding" on a particular file.
--    This is how your server(s) know which tasks to fulfill.
CREATE TABLE files.requests (
    id               SERIAL PRIMARY KEY,
    file_id          INT NOT NULL REFERENCES files.files(id) ON DELETE CASCADE,
    request_type     TEXT NOT NULL CHECK (request_type IN ('caption','embed')),
    requested_at     TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    fulfilled_at     TIMESTAMP,
    model            TEXT,   -- optional: which model to use, if you want.
    error_message    TEXT     -- optional: store any error that occurred
);
