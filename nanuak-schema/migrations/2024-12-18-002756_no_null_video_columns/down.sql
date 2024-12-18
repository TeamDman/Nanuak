-- The down migration will revert the NOT NULL constraints and remove the trigger and function.
-- Be careful with down migrations that remove NOT NULL - if any nulls appear, it could fail. 
-- We'll just restore the schema to a previous nullable state as best as possible.

-- Drop the trigger and function
DROP TRIGGER IF EXISTS trigger_update_search_doc ON youtube.videos;
DROP FUNCTION IF EXISTS youtube.update_search_document();

-- Remove the NOT NULL constraints and the default on tags
ALTER TABLE youtube.videos
    ALTER COLUMN title DROP NOT NULL,
    ALTER COLUMN video_id DROP NOT NULL,
    ALTER COLUMN etag DROP NOT NULL,
    ALTER COLUMN fetched_on DROP NOT NULL,
    ALTER COLUMN description DROP NOT NULL,
    ALTER COLUMN published_at DROP NOT NULL,
    ALTER COLUMN channel_id DROP NOT NULL,
    ALTER COLUMN channel_title DROP NOT NULL,
    ALTER COLUMN category_id DROP NOT NULL,
    ALTER COLUMN duration DROP NOT NULL,
    ALTER COLUMN caption DROP NOT NULL,
    ALTER COLUMN definition DROP NOT NULL,
    ALTER COLUMN dimension DROP NOT NULL,
    ALTER COLUMN licensed_content DROP NOT NULL,
    ALTER COLUMN privacy_status DROP NOT NULL,
    ALTER COLUMN search_document DROP NOT NULL;
