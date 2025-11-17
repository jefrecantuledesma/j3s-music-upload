-- Add library_path column to users table
-- This allows each user to have their own music library directory

ALTER TABLE users ADD COLUMN library_path TEXT;

-- Create index for faster lookups
CREATE INDEX idx_users_library_path ON users(library_path);
