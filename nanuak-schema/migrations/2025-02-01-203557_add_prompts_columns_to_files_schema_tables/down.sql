-- Remove 'prompt' column from files.captions
ALTER TABLE files.captions DROP COLUMN prompt;

-- Remove 'prompt' column from files.requests
ALTER TABLE files.requests DROP COLUMN prompt;
