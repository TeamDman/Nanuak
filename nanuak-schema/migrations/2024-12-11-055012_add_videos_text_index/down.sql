-- Remove the index and column
DROP INDEX IF EXISTS idx_videos_search_document_gin;

ALTER TABLE youtube.videos
DROP COLUMN IF EXISTS search_document;
