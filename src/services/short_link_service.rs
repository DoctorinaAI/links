use crate::{
    database::{Database, SharedDatabase},
    models::ShortLink,
};
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use sqlx::Row;
use std::collections::HashMap;

#[derive(Clone)]
pub struct ShortLinkService {
    db: SharedDatabase,
}

impl ShortLinkService {
    pub fn new(db: SharedDatabase) -> Self {
        Self { db }
    }

    /// Get a reference to the database
    pub fn database(&self) -> &SharedDatabase {
        &self.db
    }

    /// Create a short link
    pub async fn create_short_link(&self, request: ShortLink) -> Result<ShortLink> {
        let params_json = serde_json::to_string(&request.params)
            .context("Failed to serialize params to JSON")?;
        let created_at = request.created_at.timestamp();
        let updated_at = request.updated_at.timestamp();

        match self.db.as_ref() {
            Database::SQLite(pool) => {
                sqlx::query(
                    r#"
                    INSERT INTO short_links (slug, params, author, created_at, updated_at)
                    VALUES (?, ?, ?, ?, ?)
                    "#
                )
                .bind(&request.slug)
                .bind(&params_json)
                .bind(&request.author)
                .bind(created_at)
                .bind(updated_at)
                .execute(pool)
                .await
                .context("Failed to insert short link into SQLite")?;
            }
            Database::Postgres(pool) => {
                sqlx::query(
                    r#"
                    INSERT INTO short_links (slug, params, author, created_at, updated_at)
                    VALUES ($1, $2, $3, $4, $5)
                    "#
                )
                .bind(&request.slug)
                .bind(&params_json)
                .bind(&request.author)
                .bind(created_at)
                .bind(updated_at)
                .execute(pool)
                .await
                .context("Failed to insert short link into PostgreSQL")?;
            }
        }

        Ok(request)
    }

    /// Resolve a short link by its slug
    pub async fn resolve_short_link(&self, slug: &str) -> Result<Option<ShortLink>> {
        let row = match self.db.as_ref() {
            Database::SQLite(pool) => {
                sqlx::query(
                    r#"
                    SELECT slug, params, author, created_at, updated_at
                    FROM short_links
                    WHERE slug = ?
                    "#
                )
                .bind(slug)
                .fetch_optional(pool)
                .await
                .context("Failed to fetch short link from SQLite")?
                .map(|r| {
                    let slug: String = r.get("slug");
                    let params: String = r.get("params");
                    let author: String = r.get("author");
                    let created_at: i64 = r.get("created_at");
                    let updated_at: i64 = r.get("updated_at");
                    (slug, params, author, created_at, updated_at)
                })
            }
            Database::Postgres(pool) => {
                sqlx::query(
                    r#"
                    SELECT slug, params, author, created_at, updated_at
                    FROM short_links
                    WHERE slug = $1
                    "#
                )
                .bind(slug)
                .fetch_optional(pool)
                .await
                .context("Failed to fetch short link from PostgreSQL")?
                .map(|r| {
                    let slug: String = r.get("slug");
                    let params: String = r.get("params");
                    let author: String = r.get("author");
                    let created_at: i64 = r.get("created_at");
                    let updated_at: i64 = r.get("updated_at");
                    (slug, params, author, created_at, updated_at)
                })
            }
        };

        match row {
            Some((slug, params, author, created_at, updated_at)) => {
                let params: HashMap<String, String> = serde_json::from_str(&params)
                    .context("Failed to deserialize params from JSON")?;
                let created_at = DateTime::from_timestamp(created_at, 0)
                    .context("Invalid created_at timestamp")?;
                let updated_at = DateTime::from_timestamp(updated_at, 0)
                    .context("Invalid updated_at timestamp")?;

                Ok(Some(ShortLink {
                    slug,
                    params,
                    author,
                    created_at,
                    updated_at,
                }))
            }
            None => Ok(None),
        }
    }

    /// Delete a short link by its slug
    pub async fn delete_short_link(&self, slug: &str) -> Result<()> {
        match self.db.as_ref() {
            Database::SQLite(pool) => {
                let result = sqlx::query(
                    r#"
                    DELETE FROM short_links
                    WHERE slug = ?
                    "#
                )
                .bind(slug)
                .execute(pool)
                .await
                .context("Failed to delete short link from SQLite")?;

                if result.rows_affected() == 0 {
                    anyhow::bail!("Short link not found: {}", slug);
                }
            }
            Database::Postgres(pool) => {
                let result = sqlx::query(
                    r#"
                    DELETE FROM short_links
                    WHERE slug = $1
                    "#
                )
                .bind(slug)
                .execute(pool)
                .await
                .context("Failed to delete short link from PostgreSQL")?;

                if result.rows_affected() == 0 {
                    anyhow::bail!("Short link not found: {}", slug);
                }
            }
        }

        Ok(())
    }

    /// List all short links
    pub async fn list_short_links(&self) -> Result<Vec<ShortLink>> {
        let rows = match self.db.as_ref() {
            Database::SQLite(pool) => {
                sqlx::query(
                    r#"
                    SELECT slug, params, author, created_at, updated_at
                    FROM short_links
                    ORDER BY created_at DESC
                    "#
                )
                .fetch_all(pool)
                .await
                .context("Failed to fetch short links from SQLite")?
                .into_iter()
                .map(|r| {
                    let slug: String = r.get("slug");
                    let params: String = r.get("params");
                    let author: String = r.get("author");
                    let created_at: i64 = r.get("created_at");
                    let updated_at: i64 = r.get("updated_at");
                    (slug, params, author, created_at, updated_at)
                })
                .collect::<Vec<_>>()
            }
            Database::Postgres(pool) => {
                sqlx::query(
                    r#"
                    SELECT slug, params, author, created_at, updated_at
                    FROM short_links
                    ORDER BY created_at DESC
                    "#
                )
                .fetch_all(pool)
                .await
                .context("Failed to fetch short links from PostgreSQL")?
                .into_iter()
                .map(|r| {
                    let slug: String = r.get("slug");
                    let params: String = r.get("params");
                    let author: String = r.get("author");
                    let created_at: i64 = r.get("created_at");
                    let updated_at: i64 = r.get("updated_at");
                    (slug, params, author, created_at, updated_at)
                })
                .collect::<Vec<_>>()
            }
        };

        let mut short_links = Vec::new();
        for (slug, params, author, created_at, updated_at) in rows {
            let params: HashMap<String, String> = serde_json::from_str(&params)
                .context("Failed to deserialize params from JSON")?;
            let created_at = DateTime::from_timestamp(created_at, 0)
                .context("Invalid created_at timestamp")?;
            let updated_at = DateTime::from_timestamp(updated_at, 0)
                .context("Invalid updated_at timestamp")?;

            short_links.push(ShortLink {
                slug,
                params,
                author,
                created_at,
                updated_at,
            });
        }

        Ok(short_links)
    }

    /// Update a short link by its slug
    pub async fn update_short_link(&self, slug: &str, update: ShortLink) -> Result<ShortLink> {
        let params_json = serde_json::to_string(&update.params)
            .context("Failed to serialize params to JSON")?;
        let updated_at = Utc::now().timestamp();

        match self.db.as_ref() {
            Database::SQLite(pool) => {
                let result = sqlx::query(
                    r#"
                    UPDATE short_links
                    SET params = ?, author = ?, updated_at = ?
                    WHERE slug = ?
                    "#
                )
                .bind(&params_json)
                .bind(&update.author)
                .bind(updated_at)
                .bind(slug)
                .execute(pool)
                .await
                .context("Failed to update short link in SQLite")?;

                if result.rows_affected() == 0 {
                    anyhow::bail!("Short link not found: {}", slug);
                }
            }
            Database::Postgres(pool) => {
                let result = sqlx::query(
                    r#"
                    UPDATE short_links
                    SET params = $1, author = $2, updated_at = $3
                    WHERE slug = $4
                    "#
                )
                .bind(&params_json)
                .bind(&update.author)
                .bind(updated_at)
                .bind(slug)
                .execute(pool)
                .await
                .context("Failed to update short link in PostgreSQL")?;

                if result.rows_affected() == 0 {
                    anyhow::bail!("Short link not found: {}", slug);
                }
            }
        }

        // Return the updated short link
        self.resolve_short_link(slug)
            .await?
            .context("Failed to fetch updated short link")
    }

    /// Record a click on a short link
    pub async fn click_short_link(&self, slug: &str) -> Result<()> {
        let clicked_at = Utc::now().timestamp();

        match self.db.as_ref() {
            Database::SQLite(pool) => {
                sqlx::query(
                    r#"
                    INSERT INTO clicks (slug, clicked_at)
                    VALUES (?, ?)
                    "#
                )
                .bind(slug)
                .bind(clicked_at)
                .execute(pool)
                .await
                .context("Failed to record click in SQLite")?;
            }
            Database::Postgres(pool) => {
                sqlx::query(
                    r#"
                    INSERT INTO clicks (slug, clicked_at)
                    VALUES ($1, $2)
                    "#
                )
                .bind(slug)
                .bind(clicked_at)
                .execute(pool)
                .await
                .context("Failed to record click in PostgreSQL")?;
            }
        }

        Ok(())
    }

    /// Get click count for a short link
    pub async fn get_click_count(&self, slug: &str) -> Result<i64> {
        let count = match self.db.as_ref() {
            Database::SQLite(pool) => {
                let row = sqlx::query(
                    r#"
                    SELECT COUNT(*) as count
                    FROM clicks
                    WHERE slug = ?
                    "#
                )
                .bind(slug)
                .fetch_one(pool)
                .await
                .context("Failed to get click count from SQLite")?;
                
                row.get::<i32, _>("count") as i64
            }
            Database::Postgres(pool) => {
                let row = sqlx::query(
                    r#"
                    SELECT COUNT(*) as count
                    FROM clicks
                    WHERE slug = $1
                    "#
                )
                .bind(slug)
                .fetch_one(pool)
                .await
                .context("Failed to get click count from PostgreSQL")?;

                row.get::<i64, _>("count")
            }
        };

        Ok(count)
    }

    /// Get click statistics for a short link
    pub async fn get_click_stats(&self, slug: &str, limit: Option<i64>) -> Result<Vec<DateTime<Utc>>> {
        let limit = limit.unwrap_or(100);
        
        let timestamps = match self.db.as_ref() {
            Database::SQLite(pool) => {
                sqlx::query(
                    r#"
                    SELECT clicked_at
                    FROM clicks
                    WHERE slug = ?
                    ORDER BY clicked_at DESC
                    LIMIT ?
                    "#
                )
                .bind(slug)
                .bind(limit)
                .fetch_all(pool)
                .await
                .context("Failed to fetch click stats from SQLite")?
                .into_iter()
                .filter_map(|r| {
                    let timestamp: i64 = r.get("clicked_at");
                    DateTime::from_timestamp(timestamp, 0)
                })
                .collect()
            }
            Database::Postgres(pool) => {
                sqlx::query(
                    r#"
                    SELECT clicked_at
                    FROM clicks
                    WHERE slug = $1
                    ORDER BY clicked_at DESC
                    LIMIT $2
                    "#
                )
                .bind(slug)
                .bind(limit)
                .fetch_all(pool)
                .await
                .context("Failed to fetch click stats from PostgreSQL")?
                .into_iter()
                .filter_map(|r| {
                    let timestamp: i64 = r.get("clicked_at");
                    DateTime::from_timestamp(timestamp, 0)
                })
                .collect()
            }
        };

        Ok(timestamps)
    }
}
