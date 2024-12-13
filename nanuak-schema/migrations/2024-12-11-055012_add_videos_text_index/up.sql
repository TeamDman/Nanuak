-- Add a tsvector column to store combined text from title, description, and channel_title
ALTER TABLE youtube.videos
ADD COLUMN search_document tsvector;

-- Populate the search_document column with the existing data
UPDATE youtube.videos
SET search_document = to_tsvector('english',
    coalesce(title, '') || ' ' ||
    coalesce(description, '') || ' ' ||
    coalesce(channel_title, '')
);

-- Create a GIN index for fast full-text search queries
CREATE INDEX idx_videos_search_document_gin
ON youtube.videos
USING GIN (search_document);
