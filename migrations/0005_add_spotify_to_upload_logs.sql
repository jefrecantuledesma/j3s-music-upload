-- Add 'spotify' to upload_type CHECK constraint
-- SQLite doesn't support ALTER COLUMN, so we need to recreate the table

-- Create new table with updated constraint
CREATE TABLE IF NOT EXISTS upload_logs_new (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id TEXT NOT NULL,
    upload_type TEXT NOT NULL CHECK(upload_type IN ('file', 'youtube', 'spotify')),
    source TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'pending' CHECK(status IN ('pending', 'processing', 'completed', 'failed')),
    file_count INTEGER NOT NULL DEFAULT 0,
    error_message TEXT,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    completed_at TEXT,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

-- Copy existing data
INSERT INTO upload_logs_new (id, user_id, upload_type, source, status, file_count, error_message, created_at, completed_at)
SELECT id, user_id, upload_type, source, status, file_count, error_message, created_at, completed_at
FROM upload_logs;

-- Drop old table
DROP TABLE upload_logs;

-- Rename new table
ALTER TABLE upload_logs_new RENAME TO upload_logs;

-- Recreate indexes
CREATE INDEX IF NOT EXISTS idx_upload_logs_user_id ON upload_logs(user_id);
CREATE INDEX IF NOT EXISTS idx_upload_logs_created_at ON upload_logs(created_at);
