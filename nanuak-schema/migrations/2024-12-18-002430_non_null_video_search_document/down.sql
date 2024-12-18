UPDATE youtube.videos
SET search_document = to_tsvector('english', coalesce(title,'') || ' ' || coalesce(description,'') || ' ' || coalesce(channel_title,''))
WHERE search_document IS NULL;
