-- Create the "youtube" schema
CREATE SCHEMA youtube;

-- Create the "search_history" table with a primary key
CREATE TABLE youtube.search_history (
    time TIMESTAMP NOT NULL PRIMARY KEY,
    query VARCHAR(256) NOT NULL
);

-- Create the "watch_history" table with a primary key
CREATE TABLE youtube.watch_history (
    time TIMESTAMP NOT NULL PRIMARY KEY,
    youtube_video_id VARCHAR(16) NOT NULL
);

-- Create the "posts" table with a primary key
CREATE TABLE youtube.posts (
    time TIMESTAMP NOT NULL PRIMARY KEY,
    post_title VARCHAR(256) NOT NULL,
    post_url TEXT NOT NULL,
    channel_url TEXT NOT NULL,
    channel_name VARCHAR(128) NOT NULL
);
