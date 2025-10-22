-- Add redirect column to links table
-- Stores optional redirect URL (must be valid HTTPS URL)
ALTER TABLE links ADD COLUMN redirect TEXT DEFAULT NULL;

-- Create index for faster lookups by redirect
--CREATE INDEX IF NOT EXISTS idx_links_redirect ON links(redirect);
