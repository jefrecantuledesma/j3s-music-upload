-- Create config table for runtime-editable settings
CREATE TABLE IF NOT EXISTS config (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Trigger to auto-update updated_at
CREATE TRIGGER IF NOT EXISTS update_config_updated_at
AFTER UPDATE ON config
FOR EACH ROW
BEGIN
    UPDATE config SET updated_at = CURRENT_TIMESTAMP WHERE key = NEW.key;
END;
