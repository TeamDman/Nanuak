-- Drop the repo_to_path table first to avoid foreign key constraints issues
DROP TABLE IF EXISTS github.repo_to_path;
DROP TABLE IF EXISTS github.stars;
DROP TABLE IF EXISTS github.repos;

-- Drop the github schema if it exists and is empty
DROP SCHEMA IF EXISTS github CASCADE;