CREATE SCHEMA IF NOT EXISTS git;

CREATE TABLE git.cloned_repos (
    path TEXT PRIMARY KEY,  -- local path, unique
    remotes TEXT NOT NULL,   -- result of `git remote --verbose`
    seen TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);
