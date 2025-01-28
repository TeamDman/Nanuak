-- Create the github schema if it doesn't exist
CREATE SCHEMA IF NOT EXISTS github;

-- 1) Repositories table: stores information about GitHub repositories
CREATE TABLE github.repos (
    id              SERIAL PRIMARY KEY,
    full_name       TEXT NOT NULL UNIQUE,   -- e.g., "octocat/Hello-World"
    html_url        TEXT NOT NULL,          -- URL to the repository on GitHub
    stars_count     INTEGER NOT NULL DEFAULT 0,
    pushed_at      TIMESTAMP,
    created_at      TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    description     TEXT
);

-- 2) Stars table: tracks when a user starred a repository
CREATE TABLE github.stars (
    repo_id        INT NOT NULL REFERENCES github.repos(id) ON DELETE CASCADE,
    user_login     TEXT NOT NULL,          -- GitHub username of the person who starred the repo
    timestamp      TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (repo_id, user_login)
);

-- 3) Linking table to connect repositories to local file paths
CREATE TABLE github.repo_to_path (
    id             SERIAL PRIMARY KEY,
    repo_id        INT NOT NULL REFERENCES github.repos(id) ON DELETE CASCADE,
    file_id        INT NOT NULL REFERENCES files.files(id) ON DELETE CASCADE,
    created_at     TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Create indexes for faster lookups
CREATE INDEX idx_repo_to_path_repo_id ON github.repo_to_path(repo_id);
CREATE INDEX idx_repo_to_path_file_id ON github.repo_to_path(file_id);