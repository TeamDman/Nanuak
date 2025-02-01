-- Add 'prompt' column to files.captions (defaulting to NULL)
ALTER TABLE files.captions ADD COLUMN prompt TEXT;

-- Add 'prompt' column to files.requests (defaulting to NULL)
ALTER TABLE files.requests ADD COLUMN prompt TEXT;
