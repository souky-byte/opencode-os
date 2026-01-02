-- Review comments for diff annotations
CREATE TABLE IF NOT EXISTS review_comments (
    id TEXT PRIMARY KEY,
    task_id TEXT NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
    file_path TEXT NOT NULL,
    line_start INTEGER NOT NULL,
    line_end INTEGER NOT NULL,
    side TEXT NOT NULL DEFAULT 'new',
    content TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'pending',
    created_at INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_review_comments_task_id ON review_comments(task_id);
CREATE INDEX IF NOT EXISTS idx_review_comments_file ON review_comments(task_id, file_path);
