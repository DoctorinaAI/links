-- Create short_links table
CREATE TABLE IF NOT EXISTS short_links (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    short_code TEXT NOT NULL UNIQUE,
    original_url TEXT NOT NULL,
    fingerprint TEXT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    expires_at TIMESTAMP,
    click_count INTEGER DEFAULT 0
);

-- Create index for faster lookups
CREATE INDEX IF NOT EXISTS idx_short_code ON short_links(short_code);
CREATE INDEX IF NOT EXISTS idx_fingerprint ON short_links(fingerprint);

-- Create fingerprints table
CREATE TABLE IF NOT EXISTS fingerprints (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    fingerprint TEXT NOT NULL UNIQUE,
    user_agent TEXT,
    ip_address TEXT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Create clicks table for analytics
CREATE TABLE IF NOT EXISTS clicks (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    short_link_id INTEGER NOT NULL,
    fingerprint TEXT,
    ip_address TEXT,
    user_agent TEXT,
    referer TEXT,
    clicked_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (short_link_id) REFERENCES short_links(id) ON DELETE CASCADE
);

-- Create index for analytics queries
CREATE INDEX IF NOT EXISTS idx_clicks_short_link_id ON clicks(short_link_id);
CREATE INDEX IF NOT EXISTS idx_clicks_fingerprint ON clicks(fingerprint);
CREATE INDEX IF NOT EXISTS idx_clicks_clicked_at ON clicks(clicked_at);
