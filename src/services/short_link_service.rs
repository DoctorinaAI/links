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

    /// Helper function to convert row tuple to ShortLink
    /// Reduces code duplication in parsing logic
    fn tuple_to_short_link(
        slug: String,
        params_json: String,
        author: String,
        redirect: Option<String>,
        created_at: i64,
        updated_at: i64,
    ) -> Result<ShortLink> {
        let params: HashMap<String, String> =
            serde_json::from_str(&params_json).context("Failed to deserialize params from JSON")?;
        let created_at =
            DateTime::from_timestamp(created_at, 0).context("Invalid created_at timestamp")?;
        let updated_at =
            DateTime::from_timestamp(updated_at, 0).context("Invalid updated_at timestamp")?;

        Ok(ShortLink {
            slug,
            params,
            author,
            redirect,
            created_at,
            updated_at,
        })
    }

    /// Create a short link
    pub async fn create_short_link(&self, request: ShortLink) -> Result<ShortLink> {
        let params_json =
            serde_json::to_string(&request.params).context("Failed to serialize params to JSON")?;
        let created_at = request.created_at.timestamp();
        let updated_at = request.updated_at.timestamp();

        // Validate redirect URL
        let redirect = ShortLink::validate_redirect(request.redirect.clone());

        match self.db.as_ref() {
            Database::SQLite(pool) => {
                sqlx::query("INSERT INTO links (slug, params, author, redirect, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?)")
                    .bind(&request.slug)
                    .bind(&params_json)
                    .bind(&request.author)
                    .bind(&redirect)
                    .bind(created_at)
                    .bind(updated_at)
                    .execute(pool)
                    .await
                    .context("Failed to insert short link")?;
            }
            Database::Postgres(pool) => {
                sqlx::query("INSERT INTO links (slug, params, author, redirect, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6)")
                    .bind(&request.slug)
                    .bind(&params_json)
                    .bind(&request.author)
                    .bind(&redirect)
                    .bind(created_at)
                    .bind(updated_at)
                    .execute(pool)
                    .await
                    .context("Failed to insert short link")?;
            }
        }

        Ok(ShortLink {
            redirect,
            ..request
        })
    }

    /// Resolve a short link by its slug
    pub async fn resolve_short_link(&self, slug: &str) -> Result<Option<ShortLink>> {
        // Note: SQL is duplicated for SQLite (?) and PostgreSQL ($1) parameter binding.
        // This is intentional to avoid string manipulation and SQL injection risks.
        let row = match self.db.as_ref() {
            Database::SQLite(pool) => sqlx::query(
                "SELECT slug, params, author, redirect, created_at, updated_at FROM links WHERE slug = ?",
            )
            .bind(slug)
            .fetch_optional(pool)
            .await
            .context("Failed to fetch short link")?
            .map(|r| {
                (
                    r.get("slug"),
                    r.get("params"),
                    r.get("author"),
                    r.get("redirect"),
                    r.get("created_at"),
                    r.get("updated_at"),
                )
            }),
            Database::Postgres(pool) => sqlx::query(
                "SELECT slug, params, author, redirect, created_at, updated_at FROM links WHERE slug = $1",
            )
            .bind(slug)
            .fetch_optional(pool)
            .await
            .context("Failed to fetch short link")?
            .map(|r| {
                (
                    r.get("slug"),
                    r.get("params"),
                    r.get("author"),
                    r.get("redirect"),
                    r.get("created_at"),
                    r.get("updated_at"),
                )
            }),
        };

        match row {
            Some((slug, params, author, redirect, created_at, updated_at)) => {
                Self::tuple_to_short_link(slug, params, author, redirect, created_at, updated_at)
                    .map(Some)
            }
            None => Ok(None),
        }
    }

    /// Delete a short link by its slug
    pub async fn delete_short_link(&self, slug: &str) -> Result<()> {
        let rows_affected = match self.db.as_ref() {
            Database::SQLite(pool) => sqlx::query("DELETE FROM links WHERE slug = ?")
                .bind(slug)
                .execute(pool)
                .await
                .context("Failed to delete short link")?
                .rows_affected(),
            Database::Postgres(pool) => sqlx::query("DELETE FROM links WHERE slug = $1")
                .bind(slug)
                .execute(pool)
                .await
                .context("Failed to delete short link")?
                .rows_affected(),
        };

        if rows_affected == 0 {
            anyhow::bail!("Short link not found: {}", slug);
        }

        Ok(())
    }

    /// List all short links
    pub async fn list_short_links(&self) -> Result<Vec<ShortLink>> {
        let tuples: Vec<(String, String, String, Option<String>, i64, i64)> = match self.db.as_ref() {
            Database::SQLite(pool) => {
                sqlx::query("SELECT slug, params, author, redirect, created_at, updated_at FROM links ORDER BY created_at DESC")
                    .fetch_all(pool)
                    .await
                    .context("Failed to fetch short links")?
                    .into_iter()
                    .map(|r| (r.get("slug"), r.get("params"), r.get("author"), r.get("redirect"), r.get("created_at"), r.get("updated_at")))
                    .collect()
            }
            Database::Postgres(pool) => {
                sqlx::query("SELECT slug, params, author, redirect, created_at, updated_at FROM links ORDER BY created_at DESC")
                    .fetch_all(pool)
                    .await
                    .context("Failed to fetch short links")?
                    .into_iter()
                    .map(|r| (r.get("slug"), r.get("params"), r.get("author"), r.get("redirect"), r.get("created_at"), r.get("updated_at")))
                    .collect()
            }
        };

        tuples
            .into_iter()
            .map(|(slug, params, author, redirect, created_at, updated_at)| {
                Self::tuple_to_short_link(slug, params, author, redirect, created_at, updated_at)
            })
            .collect()
    }

    /// Update a short link by its slug
    pub async fn update_short_link(&self, slug: &str, update: ShortLink) -> Result<ShortLink> {
        let params_json =
            serde_json::to_string(&update.params).context("Failed to serialize params to JSON")?;
        let updated_at = Utc::now().timestamp();

        // Validate redirect URL
        let redirect = ShortLink::validate_redirect(update.redirect);

        let rows_affected = match self.db.as_ref() {
            Database::SQLite(pool) => sqlx::query(
                "UPDATE links SET params = ?, author = ?, redirect = ?, updated_at = ? WHERE slug = ?",
            )
            .bind(&params_json)
            .bind(&update.author)
            .bind(&redirect)
            .bind(updated_at)
            .bind(slug)
            .execute(pool)
            .await
            .context("Failed to update short link")?
            .rows_affected(),
            Database::Postgres(pool) => sqlx::query(
                "UPDATE links SET params = $1, author = $2, redirect = $3, updated_at = $4 WHERE slug = $5",
            )
            .bind(&params_json)
            .bind(&update.author)
            .bind(&redirect)
            .bind(updated_at)
            .bind(slug)
            .execute(pool)
            .await
            .context("Failed to update short link")?
            .rows_affected(),
        };

        if rows_affected == 0 {
            anyhow::bail!("Short link not found: {}", slug);
        }

        self.resolve_short_link(slug)
            .await?
            .context("Failed to fetch updated short link")
    }

    /// Record a click on a short link
    pub async fn click_short_link(&self, slug: &str) -> Result<()> {
        let clicked_at = Utc::now().timestamp();

        match self.db.as_ref() {
            Database::SQLite(pool) => {
                sqlx::query("INSERT INTO clicks (slug, clicked_at) VALUES (?, ?)")
                    .bind(slug)
                    .bind(clicked_at)
                    .execute(pool)
                    .await
                    .context("Failed to record click")?;
            }
            Database::Postgres(pool) => {
                sqlx::query("INSERT INTO clicks (slug, clicked_at) VALUES ($1, $2)")
                    .bind(slug)
                    .bind(clicked_at)
                    .execute(pool)
                    .await
                    .context("Failed to record click")?;
            }
        }

        Ok(())
    }

    /// Get click count for a short link
    pub async fn get_click_count(&self, slug: &str) -> Result<i64> {
        let count = match self.db.as_ref() {
            Database::SQLite(pool) => {
                let row = sqlx::query("SELECT COUNT(*) as count FROM clicks WHERE slug = ?")
                    .bind(slug)
                    .fetch_one(pool)
                    .await
                    .context("Failed to get click count")?;
                row.get::<i32, _>("count") as i64
            }
            Database::Postgres(pool) => {
                let row = sqlx::query("SELECT COUNT(*) as count FROM clicks WHERE slug = $1")
                    .bind(slug)
                    .fetch_one(pool)
                    .await
                    .context("Failed to get click count")?;
                row.get::<i64, _>("count")
            }
        };

        Ok(count)
    }

    /// Get click statistics for a short link
    pub async fn get_click_stats(
        &self,
        slug: &str,
        limit: Option<i64>,
    ) -> Result<Vec<DateTime<Utc>>> {
        let limit = limit.unwrap_or(100);

        let timestamps = match self.db.as_ref() {
            Database::SQLite(pool) => sqlx::query(
                "SELECT clicked_at FROM clicks WHERE slug = ? ORDER BY clicked_at DESC LIMIT ?",
            )
            .bind(slug)
            .bind(limit)
            .fetch_all(pool)
            .await
            .context("Failed to fetch click stats")?
            .into_iter()
            .filter_map(|r| DateTime::from_timestamp(r.get("clicked_at"), 0))
            .collect(),
            Database::Postgres(pool) => sqlx::query(
                "SELECT clicked_at FROM clicks WHERE slug = $1 ORDER BY clicked_at DESC LIMIT $2",
            )
            .bind(slug)
            .bind(limit)
            .fetch_all(pool)
            .await
            .context("Failed to fetch click stats")?
            .into_iter()
            .filter_map(|r| DateTime::from_timestamp(r.get("clicked_at"), 0))
            .collect(),
        };

        Ok(timestamps)
    }
}
