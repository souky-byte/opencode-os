-- Track which files have been viewed in diff review
CREATE TABLE IF NOT EXISTS diff_viewed_files (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    task_id TEXT NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
    file_path TEXT NOT NULL,
    viewed_at INTEGER NOT NULL,
    UNIQUE(task_id, file_path)
);

CREATE INDEX IF NOT EXISTS idx_diff_viewed_task_id ON diff_viewed_files(task_id);
