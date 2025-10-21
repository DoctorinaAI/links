use anyhow::{Context, Result};
use sqlx::migrate::MigrateDatabase;
use sqlx::{Pool, Postgres, Sqlite, postgres::PgPoolOptions, sqlite::SqlitePoolOptions};
use std::str::FromStr;
use std::sync::Arc;
use std::{path::Path, time::Duration};
use tracing::debug;

/// Database type enum
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DatabaseType {
    SQLite,
    Postgres,
}

impl FromStr for DatabaseType {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "sqlite" | "sqlite3" => Ok(DatabaseType::SQLite),
            "postgres" | "postgresql" | "pg" => Ok(DatabaseType::Postgres),
            _ => anyhow::bail!("Unknown database type: {}", s),
        }
    }
}

impl DatabaseType {
    /// Detect database type from connection string
    pub fn from_connection_string(url: &str) -> Result<Self> {
        if url.starts_with("sqlite:") {
            Ok(DatabaseType::SQLite)
        } else if url.starts_with("postgres:") || url.starts_with("postgresql:") {
            Ok(DatabaseType::Postgres)
        } else {
            anyhow::bail!("Cannot detect database type from URL: {}", url)
        }
    }
}

/// Database connection wrapper that supports both SQLite and PostgreSQL
#[derive(Clone)]
pub enum Database {
    SQLite(Pool<Sqlite>),
    Postgres(Pool<Postgres>),
}

impl Database {
    /// Create a new database connection based on the connection string
    pub async fn new(url: &str, db_type: Option<DatabaseType>) -> Result<Self> {
        // Determine database type
        let db_type = match db_type {
            Some(t) => t,
            None => DatabaseType::from_connection_string(url)?,
        };

        match db_type {
            DatabaseType::SQLite => {
                debug!("Connecting to SQLite database: {}", url);
                Self::new_sqlite(url).await
            }
            DatabaseType::Postgres => {
                debug!("Connecting to PostgreSQL database");
                Self::new_postgres(url).await
            }
        }
    }

    /// Create a new SQLite database connection
    async fn new_sqlite(url: &str) -> Result<Self> {
        // Ensure directory exists for database file
        fn ensure_database_directory(database_url: &str) -> Result<()> {
            if let Some(path_part) = database_url.strip_prefix("sqlite://")
                && let Some(db_path) = path_part.split('?').next()
                && let Some(parent) = Path::new(db_path).parent()
            {
                // Create the parent directory if it does not exist
                debug!("ensuring database directory exists: {}", parent.display());
                std::fs::create_dir_all(parent)?;
            }
            Ok(())
        }

        // Ensure the database directory exists
        ensure_database_directory(url)?;

        // Create database if it doesn't exist
        if !Sqlite::database_exists(url)
            .await
            .context("Failed to check if SQLite database exists")?
        {
            debug!("Creating SQLite database: {}", url);
            Sqlite::create_database(url)
                .await
                .context("Failed to create SQLite database")?;
        }

        // Connect to database
        let pool = SqlitePoolOptions::new()
            // SQLite with WAL mode can handle multiple readers + 1 writer
            // For web servers, 2-5 connections is usually optimal for SQLite
            .max_connections(5)
            .min_connections(1)
            .acquire_timeout(Duration::from_secs(5))
            .idle_timeout(Duration::from_secs(600)) // 10 minutes
            .max_lifetime(Duration::from_secs(1800)) // 30 minutes
            // Enable WAL mode and other optimizations after connecting
            .after_connect(|conn, _meta| {
                Box::pin(async move {
                    // Enable WAL mode for better concurrency
                    sqlx::query("PRAGMA journal_mode = WAL;")
                        .execute(&mut *conn)
                        .await?;

                    // Set synchronous to NORMAL for better performance
                    // FULL is safer but much slower
                    sqlx::query("PRAGMA synchronous = NORMAL;")
                        .execute(&mut *conn)
                        .await?;

                    // Set a reasonable busy timeout (5 seconds)
                    sqlx::query("PRAGMA busy_timeout = 5000;")
                        .execute(&mut *conn)
                        .await?;

                    // Enable foreign keys
                    sqlx::query("PRAGMA foreign_keys = ON;")
                        .execute(&mut *conn)
                        .await?;

                    // Optimize cache size (in KB)
                    sqlx::query("PRAGMA cache_size = -64000;") // 64MB
                        .execute(&mut *conn)
                        .await?;

                    Ok(())
                })
            })
            .connect(url)
            .await
            .context("Failed to connect to SQLite database")?;

        debug!("Successfully connected to SQLite database");
        Ok(Database::SQLite(pool))
    }

    /// Create a new PostgreSQL database connection
    async fn new_postgres(url: &str) -> Result<Self> {
        // Connect to database with optimized pool settings
        // PostgreSQL can handle many more concurrent connections than SQLite
        // Typical production settings for medium-high load web servers
        let pool = PgPoolOptions::new()
            // PostgreSQL can efficiently handle 20-100+ connections
            // For high load scenarios, start with more connections
            .max_connections(50)
            .min_connections(5)
            // Connection timeout - how long to wait for an available connection
            .acquire_timeout(Duration::from_secs(10))
            // Idle connection timeout - close idle connections after 10 minutes
            .idle_timeout(Duration::from_secs(600))
            // Maximum connection lifetime - recycle connections after 30 minutes
            // This helps prevent issues with stale connections
            .max_lifetime(Duration::from_secs(1800))
            // Test connections before using them
            .test_before_acquire(true)
            // Configure connection-level settings
            .after_connect(|conn, _meta| {
                Box::pin(async move {
                    // Set statement timeout to prevent long-running queries
                    sqlx::query("SET statement_timeout = '30s';")
                        .execute(&mut *conn)
                        .await?;

                    // Set search path if needed
                    sqlx::query("SET search_path TO public;")
                        .execute(&mut *conn)
                        .await?;

                    // Set timezone to UTC for consistency
                    sqlx::query("SET timezone = 'UTC';")
                        .execute(&mut *conn)
                        .await?;

                    // Enable connection pooling optimizations
                    sqlx::query("SET lock_timeout = '10s';")
                        .execute(&mut *conn)
                        .await?;

                    Ok(())
                })
            })
            .connect(url)
            .await
            .context("Failed to connect to PostgreSQL database")?;

        debug!("Successfully connected to PostgreSQL database");
        Ok(Database::Postgres(pool))
    }

    /// Run migrations
    pub async fn migrate(&self) -> Result<()> {
        match self {
            Database::SQLite(pool) => {
                debug!("Running SQLite migrations");
                sqlx::migrate!("./migrations")
                    .run(pool)
                    .await
                    .context("Failed to run SQLite migrations")?;

                // VACUUM the database to optimize it
                debug!("Running VACUUM on the database");
                sqlx::query("VACUUM")
                    .execute(pool)
                    .await
                    .context("Failed to VACUUM SQLite database")?;
            }
            Database::Postgres(pool) => {
                debug!("Running PostgreSQL migrations");
                sqlx::migrate!("./migrations")
                    .run(pool)
                    .await
                    .context("Failed to run PostgreSQL migrations")?;
            }
        }
        debug!("Migrations completed successfully");
        Ok(())
    }

    /// Get the database type
    pub fn db_type(&self) -> DatabaseType {
        match self {
            Database::SQLite(_) => DatabaseType::SQLite,
            Database::Postgres(_) => DatabaseType::Postgres,
        }
    }

    /// Execute a query that returns no results (INSERT, UPDATE, DELETE)
    pub async fn execute(&self, query: &str) -> Result<u64> {
        let rows_affected = match self {
            Database::SQLite(pool) => sqlx::query(query)
                .execute(pool)
                .await
                .context("Failed to execute SQLite query")?
                .rows_affected(),
            Database::Postgres(pool) => sqlx::query(query)
                .execute(pool)
                .await
                .context("Failed to execute PostgreSQL query")?
                .rows_affected(),
        };
        Ok(rows_affected)
    }

    /// Get a reference to the SQLite pool
    pub fn sqlite_pool(&self) -> Result<&Pool<Sqlite>> {
        match self {
            Database::SQLite(pool) => Ok(pool),
            _ => anyhow::bail!("Not a SQLite database"),
        }
    }

    /// Get a reference to the PostgreSQL pool
    pub fn postgres_pool(&self) -> Result<&Pool<Postgres>> {
        match self {
            Database::Postgres(pool) => Ok(pool),
            _ => anyhow::bail!("Not a PostgreSQL database"),
        }
    }

    /// Close the database connection
    pub async fn close(&self) {
        match self {
            Database::SQLite(pool) => {
                pool.close().await;
                debug!("SQLite connection closed");
            }
            Database::Postgres(pool) => {
                pool.close().await;
                debug!("PostgreSQL connection closed");
            }
        }
    }

    /// Check if the database connection is healthy
    pub async fn health_check(&self) -> Result<()> {
        match self {
            Database::SQLite(pool) => {
                sqlx::query("SELECT 1")
                    .execute(pool)
                    .await
                    .context("SQLite health check failed")?;
            }
            Database::Postgres(pool) => {
                sqlx::query("SELECT 1")
                    .execute(pool)
                    .await
                    .context("PostgreSQL health check failed")?;
            }
        }
        Ok(())
    }
}

impl std::fmt::Debug for Database {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Database::SQLite(_) => write!(f, "Database::SQLite"),
            Database::Postgres(_) => write!(f, "Database::Postgres"),
        }
    }
}

/// Shared database connection
pub type SharedDatabase = Arc<Database>;

/// Create a shared database connection
pub async fn create_database(url: &str, db_type: Option<DatabaseType>) -> Result<SharedDatabase> {
    let db = Database::new(url, db_type).await?;
    Ok(Arc::new(db))
}
