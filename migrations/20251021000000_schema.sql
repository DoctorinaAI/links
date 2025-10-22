-- Create short_links table with slug as primary key
-- DateTime stored as INTEGER (Unix timestamp in seconds) for SQLite compatibility
-- params stored as TEXT (JSON) for SQLite compatibility
CREATE TABLE IF NOT EXISTS short_links (
    slug TEXT PRIMARY KEY NOT NULL,
    params TEXT NOT NULL DEFAULT '{}',
    author TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);

-- Create index for faster lookups by author and timestamps
CREATE INDEX IF NOT EXISTS idx_short_links_author ON short_links(author);
CREATE INDEX IF NOT EXISTS idx_short_links_created_at ON short_links(created_at);
CREATE INDEX IF NOT EXISTS idx_short_links_updated_at ON short_links(updated_at);

-- Create clicks table for analytics
-- Separate table to track clicks on short links
CREATE TABLE IF NOT EXISTS clicks (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    slug TEXT NOT NULL,
    clicked_at INTEGER NOT NULL,
    FOREIGN KEY (slug) REFERENCES short_links(slug) ON DELETE CASCADE
);

-- Create indexes for analytics queries
CREATE INDEX IF NOT EXISTS idx_clicks_slug ON clicks(slug);
CREATE INDEX IF NOT EXISTS idx_clicks_clicked_at ON clicks(clicked_at);
CREATE INDEX IF NOT EXISTS idx_clicks_slug_clicked_at ON clicks(slug, clicked_at);