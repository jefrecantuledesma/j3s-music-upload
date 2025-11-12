-- Create upload logs table to track uploads
CREATE TABLE IF NOT EXISTS upload_logs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id TEXT NOT NULL,
    upload_type TEXT NOT NULL CHECK(upload_type IN ('file', 'youtube')),
    source TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'pending' CHECK(status IN ('pending', 'processing', 'completed', 'failed')),
    file_count INTEGER NOT NULL DEFAULT 0,
    error_message TEXT,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    completed_at TEXT,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

-- Create index for user lookups
CREATE INDEX IF NOT EXISTS idx_upload_logs_user_id ON upload_logs(user_id);
CREATE INDEX IF NOT EXISTS idx_upload_logs_created_at ON upload_logs(created_at);
