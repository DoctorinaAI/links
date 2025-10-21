# Database Module

This module provides a unified interface for working with both SQLite and PostgreSQL databases.

## Features

- **Universal Interface**: Single API for both SQLite and PostgreSQL
- **Auto-detection**: Automatically detects database type from connection string
- **Connection Pooling**: Built-in connection pooling via sqlx
- **Migrations**: Support for database migrations
- **Health Checks**: Built-in health check functionality

## Usage

### Basic Connection

```rust
use links::database::{create_database, DatabaseType};

// Auto-detect database type from URL
let db = create_database("sqlite://data/links.db?mode=rwc", None).await?;

// Or explicitly specify the type
let db = create_database(
    "postgres://user:pass@localhost/links",
    Some(DatabaseType::Postgres)
).await?;
```

### Configuration

You can configure the database via environment variables or CLI arguments:

```bash
# SQLite (default)
CONFIG_DATABASE="sqlite://data/links.db?mode=rwc"

# PostgreSQL
CONFIG_DATABASE="postgres://user:password@localhost:5432/links"
```

Or via command line:

```bash
# SQLite
./server --database "sqlite://data/links.db?mode=rwc" --db-type sqlite

# PostgreSQL
./server --database "postgres://user:pass@localhost/links" --db-type postgres
```

### Running Migrations

```rust
// Migrations are automatically run during database initialization
db.migrate().await?;
```

### Health Check

```rust
// Check if database connection is healthy
db.health_check().await?;
```

### Using in Services

```rust
use links::database::SharedDatabase;
use links::services::{ShortLinkService, FingerprintService};

// Create services with database connection
let short_link_service = ShortLinkService::new(db.clone());
let fingerprint_service = FingerprintService::new(db.clone());
```

### Executing Queries

```rust
// Get the underlying pool for custom queries
match db.as_ref() {
    Database::SQLite(pool) => {
        let result = sqlx::query("SELECT * FROM short_links")
            .fetch_all(pool)
            .await?;
    }
    Database::Postgres(pool) => {
        let result = sqlx::query("SELECT * FROM short_links")
            .fetch_all(pool)
            .await?;
    }
}

// Or use the convenience methods
let pool = db.sqlite_pool()?; // Returns error if not SQLite
let pool = db.postgres_pool()?; // Returns error if not PostgreSQL
```

## Migrations

Migrations are stored in the `migrations/` directory. Each migration should be a SQL file with a timestamp prefix:

```
migrations/
  20251021000000_schema.sql
  20251021000001_add_some_feature.sql
```

### Creating a New Migration

1. Create a new SQL file in the `migrations/` directory with a timestamp prefix
2. Write your migration SQL
3. The migration will be automatically applied on next server start

### Example Migration

```sql
-- migrations/20251021000000_schema.sql
CREATE TABLE IF NOT EXISTS short_links (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    short_code TEXT NOT NULL UNIQUE,
    original_url TEXT NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
```

## Database Schema

### Tables

#### `short_links`
- `id`: Primary key (auto-increment)
- `short_code`: Unique short code for the URL
- `original_url`: Original long URL
- `fingerprint`: User fingerprint (optional)
- `created_at`: Creation timestamp
- `updated_at`: Last update timestamp
- `expires_at`: Expiration timestamp (optional)
- `click_count`: Number of clicks

#### `fingerprints`
- `id`: Primary key (auto-increment)
- `fingerprint`: Unique fingerprint hash
- `user_agent`: Browser user agent
- `ip_address`: IP address
- `created_at`: Creation timestamp

#### `clicks`
- `id`: Primary key (auto-increment)
- `short_link_id`: Foreign key to short_links
- `fingerprint`: User fingerprint
- `ip_address`: IP address
- `user_agent`: Browser user agent
- `referer`: HTTP referer
- `clicked_at`: Click timestamp

## Testing

Run database tests with:

```bash
cargo test --test db
```

## Notes

- SQLite is the default database for development
- PostgreSQL is recommended for production
- Connection strings must start with `sqlite:` or `postgres:`
- The database type can be auto-detected from the connection string
