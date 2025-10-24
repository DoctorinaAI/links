-- Create fingerprints table for analytics tracking
-- This table stores temporary fingerprints based on IP + platform info
-- to enable attribution tracking for marketing links
CREATE TABLE IF NOT EXISTS fingerprints (
    fingerprint TEXT PRIMARY KEY NOT NULL,
    slug TEXT NOT NULL,
    ip_address TEXT NOT NULL,
    params TEXT NOT NULL DEFAULT '{}',
    created_at INTEGER NOT NULL,
    expires_at INTEGER NOT NULL,
    FOREIGN KEY (slug) REFERENCES links(slug) ON DELETE CASCADE
);

-- Create index for cleanup of expired fingerprints
CREATE INDEX IF NOT EXISTS idx_fingerprints_expires_at ON fingerprints(expires_at);

-- Create index for slug lookups
CREATE INDEX IF NOT EXISTS idx_fingerprints_slug ON fingerprints(slug);
