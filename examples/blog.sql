-- Blog Database Example
-- This script creates a simple blog database with users, posts, and comments.

-- Create tables
CREATE TABLE users (
    id INTEGER,
    name TEXT,
    email TEXT,
    created_at TEXT
);

CREATE TABLE posts (
    id INTEGER,
    title TEXT,
    content TEXT,
    author_id INTEGER,
    created_at TEXT,
    published BOOLEAN
);

CREATE TABLE comments (
    id INTEGER,
    post_id INTEGER,
    author_id INTEGER,
    content TEXT,
    created_at TEXT
);

-- Insert users
INSERT INTO users (id, name, email, created_at) VALUES (1, 'Alice', 'alice@example.com', '2026-01-01');
INSERT INTO users (id, name, email, created_at) VALUES (2, 'Bob', 'bob@example.com', '2026-01-02');
INSERT INTO users (id, name, email, created_at) VALUES (3, 'Charlie', 'charlie@example.com', '2026-01-03');

-- Insert posts
INSERT INTO posts (id, title, content, author_id, created_at, published)
VALUES (1, 'Getting Started with Tywindb', 'Tywindb is a modern, fast database...', 1, '2026-01-10', true);

INSERT INTO posts (id, title, content, author_id, created_at, published)
VALUES (2, 'Advanced SQL Queries', 'Learn how to write complex queries...', 1, '2026-01-12', true);

INSERT INTO posts (id, title, content, author_id, created_at, published)
VALUES (3, 'Draft Post', 'This is a draft...', 2, '2026-01-15', false);

-- Insert comments
INSERT INTO comments (id, post_id, author_id, content, created_at)
VALUES (1, 1, 2, 'Great introduction!', '2026-01-11');

INSERT INTO comments (id, post_id, author_id, content, created_at)
VALUES (2, 1, 3, 'Thanks for sharing!', '2026-01-11');

INSERT INTO comments (id, post_id, author_id, content, created_at)
VALUES (3, 2, 2, 'Very helpful!', '2026-01-13');

-- Query examples

-- Get all published posts
SELECT id, title, author_id FROM posts WHERE published = true;

-- Get posts by a specific author
SELECT * FROM posts WHERE author_id = 1;

-- Get comments for a specific post
SELECT * FROM comments WHERE post_id = 1;

-- Count posts per user
SELECT author_id, COUNT(*) as post_count FROM posts GROUP BY author_id;

-- Get user with most posts
SELECT author_id, COUNT(*) as post_count
FROM posts
GROUP BY author_id
ORDER BY post_count DESC
LIMIT 1;
