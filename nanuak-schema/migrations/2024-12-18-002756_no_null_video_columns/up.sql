-- 1. Check expectations of non-null columns
DO $$
DECLARE
    cnt INTEGER;
BEGIN
    SELECT COUNT(*) INTO cnt FROM youtube.videos
    WHERE title IS NULL
       OR video_id IS NULL
       OR etag IS NULL
       OR fetched_on IS NULL
       OR description IS NULL
       OR published_at IS NULL
       OR channel_id IS NULL
       OR channel_title IS NULL
       OR category_id IS NULL
       OR duration IS NULL
       OR caption IS NULL
       OR definition IS NULL
       OR dimension IS NULL
       OR licensed_content IS NULL
       OR privacy_status IS NULL
       OR search_document IS NULL;

    IF cnt > 0 THEN
        RAISE EXCEPTION 'Found unexpected NULL values in non-tags columns before making columns NOT NULL.';
    END IF;
END;
$$;

-- 2. Now that we've asserted no unexpected NULLs, we can safely set the columns to NOT NULL.
--    We do not provide defaults here, since they're already non-null and we want no defaults.
ALTER TABLE youtube.videos
    ALTER COLUMN title SET NOT NULL,
    ALTER COLUMN video_id SET NOT NULL,
    ALTER COLUMN etag SET NOT NULL,
    ALTER COLUMN fetched_on SET NOT NULL,
    ALTER COLUMN description SET NOT NULL,
    ALTER COLUMN published_at SET NOT NULL,
    ALTER COLUMN channel_id SET NOT NULL,
    ALTER COLUMN channel_title SET NOT NULL,
    ALTER COLUMN category_id SET NOT NULL,
    ALTER COLUMN duration SET NOT NULL,
    ALTER COLUMN caption SET NOT NULL,
    ALTER COLUMN definition SET NOT NULL,
    ALTER COLUMN dimension SET NOT NULL,
    ALTER COLUMN licensed_content SET NOT NULL,
    ALTER COLUMN privacy_status SET NOT NULL,
    ALTER COLUMN search_document SET NOT NULL;

-- 4. Add a trigger or function to recompute search_document on INSERT/UPDATE
-- First, create a function that will update the search_document column
CREATE OR REPLACE FUNCTION youtube.update_search_document() RETURNS trigger AS $$
BEGIN
    NEW.search_document := to_tsvector('english',
        coalesce(NEW.title, '') || ' ' ||
        coalesce(NEW.description, '') || ' ' ||
        coalesce(NEW.channel_title, '')
    );
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Now create a trigger that fires before insert or update on title/description/channel_title
DROP TRIGGER IF EXISTS trigger_update_search_doc ON youtube.videos;
CREATE TRIGGER trigger_update_search_doc
BEFORE INSERT OR UPDATE OF title, description, channel_title ON youtube.videos
FOR EACH ROW
EXECUTE PROCEDURE youtube.update_search_document();

-- Reindex the GIN index if necessary (optional step)
-- REINDEX INDEX idx_videos_search_document_gin;
