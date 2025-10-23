-- Add description column to links table
ALTER TABLE links ADD COLUMN description TEXT;

-- Create index for faster text search on descriptions (optional)
CREATE INDEX IF NOT EXISTS idx_links_description ON links(description);
