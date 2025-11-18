-- Add ferric_enabled to config table
-- This allows toggling Ferric processing from the admin panel

INSERT OR IGNORE INTO config (key, value) VALUES ('ferric_enabled', 'false');
