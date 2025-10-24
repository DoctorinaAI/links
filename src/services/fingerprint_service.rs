use crate::database::{Database, SharedDatabase};
use crate::models::Platform;
use anyhow::{Context, Result};
use chrono::{DateTime, Duration, Utc};
use serde_json;
use sha2::{Digest, Sha256};
use sqlx::Row;
use std::collections::HashMap;

/// Fingerprint data stored in database
#[derive(Debug, Clone)]
pub struct Fingerprint {
    pub fingerprint: String,
    pub slug: String,
    pub params: HashMap<String, String>,
    pub ip_address: String,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone)]
pub struct FingerprintService {
    db: SharedDatabase,
}

impl FingerprintService {
    pub fn new(db: SharedDatabase) -> Self {
        Self { db }
    }

    /// Get a reference to the database
    pub fn database(&self) -> &SharedDatabase {
        &self.db
    }

    /// Create a fingerprint hash from IP address and platform data
    /// The fingerprint is deterministic: same IP + platform = same fingerprint
    pub fn create_fingerprint_hash(ip: &str, platform: &Platform) -> String {
        let mut hasher = Sha256::new();

        // Combine IP and canonical platform string
        let data = format!("{}|{}", ip, platform.to_canonical_string());
        hasher.update(data.as_bytes());

        // Return hex-encoded hash
        format!("{:x}", hasher.finalize())
    }

    /// Store a fingerprint in the database with TTL
    /// If fingerprint already exists, it will be updated with new data
    pub async fn store_fingerprint(
        &self,
        slug: &str,
        params: &HashMap<String, String>,
        ip: &str,
        platform: &Platform,
        ttl_hours: Option<i64>,
    ) -> Result<Fingerprint> {
        let fingerprint = Self::create_fingerprint_hash(ip, platform);
        let ttl = ttl_hours.unwrap_or(24); // Default 24 hours
        let now = Utc::now();
        let expires_at = now + Duration::hours(ttl);

        let params_json =
            serde_json::to_string(params).context("Failed to serialize params to JSON")?;

        let created_at_ts = now.timestamp();
        let expires_at_ts = expires_at.timestamp();

        // Use UPSERT to update if exists or insert if new
        match self.db.as_ref() {
            Database::SQLite(pool) => {
                sqlx::query(
                    "INSERT INTO fingerprints (fingerprint, slug, params, ip_address, expires_at, created_at)
                     VALUES (?, ?, ?, ?, ?, ?)
                     ON CONFLICT(fingerprint) DO UPDATE SET
                         slug = excluded.slug,
                         params = excluded.params,
                         ip_address = excluded.ip_address,
                         expires_at = excluded.expires_at,
                         created_at = excluded.created_at"
                )
                .bind(&fingerprint)
                .bind(slug)
                .bind(&params_json)
                .bind(ip)
                .bind(expires_at_ts)
                .bind(created_at_ts)
                .execute(pool)
                .await
                .context("Failed to store fingerprint")?;
            }
            Database::Postgres(pool) => {
                sqlx::query(
                    "INSERT INTO fingerprints (fingerprint, slug, params, ip_address, expires_at, created_at)
                     VALUES ($1, $2, $3, $4, $5, $6)
                     ON CONFLICT(fingerprint) DO UPDATE SET
                         slug = EXCLUDED.slug,
                         params = EXCLUDED.params,
                         ip_address = EXCLUDED.ip_address,
                         expires_at = EXCLUDED.expires_at,
                         created_at = EXCLUDED.created_at"
                )
                .bind(&fingerprint)
                .bind(slug)
                .bind(&params_json)
                .bind(ip)
                .bind(expires_at_ts)
                .bind(created_at_ts)
                .execute(pool)
                .await
                .context("Failed to store fingerprint")?;
            }
        }

        Ok(Fingerprint {
            fingerprint,
            slug: slug.to_string(),
            params: params.clone(),
            ip_address: ip.to_string(),
            expires_at,
            created_at: now,
        })
    }

    /// Retrieve fingerprint data by IP and platform
    /// Returns None if fingerprint not found or expired
    pub async fn get_fingerprint(
        &self,
        ip: &str,
        platform: &Platform,
    ) -> Result<Option<Fingerprint>> {
        let fingerprint = Self::create_fingerprint_hash(ip, platform);
        let now = Utc::now().timestamp();

        match self.db.as_ref() {
            Database::SQLite(pool) => {
                let row = sqlx::query(
                    "SELECT fingerprint, slug, params, ip_address, expires_at, created_at
                     FROM fingerprints
                     WHERE fingerprint = ? AND expires_at > ?",
                )
                .bind(&fingerprint)
                .bind(now)
                .fetch_optional(pool)
                .await
                .context("Failed to fetch fingerprint")?;

                match row {
                    Some(row) => {
                        let params_json: String = row.get("params");
                        let params: HashMap<String, String> = serde_json::from_str(&params_json)
                            .context("Failed to deserialize params from JSON")?;

                        let expires_at_ts: i64 = row.get("expires_at");
                        let created_at_ts: i64 = row.get("created_at");

                        let expires_at = DateTime::from_timestamp(expires_at_ts, 0)
                            .context("Invalid expires_at timestamp")?;
                        let created_at = DateTime::from_timestamp(created_at_ts, 0)
                            .context("Invalid created_at timestamp")?;

                        Ok(Some(Fingerprint {
                            fingerprint: row.get("fingerprint"),
                            slug: row.get("slug"),
                            params,
                            ip_address: row.get("ip_address"),
                            expires_at,
                            created_at,
                        }))
                    }
                    None => Ok(None),
                }
            }
            Database::Postgres(pool) => {
                let row = sqlx::query(
                    "SELECT fingerprint, slug, params, ip_address, expires_at, created_at
                     FROM fingerprints
                     WHERE fingerprint = $1 AND expires_at > $2",
                )
                .bind(&fingerprint)
                .bind(now)
                .fetch_optional(pool)
                .await
                .context("Failed to fetch fingerprint")?;

                match row {
                    Some(row) => {
                        let params_json: String = row.get("params");
                        let params: HashMap<String, String> = serde_json::from_str(&params_json)
                            .context("Failed to deserialize params from JSON")?;

                        let expires_at_ts: i64 = row.get("expires_at");
                        let created_at_ts: i64 = row.get("created_at");

                        let expires_at = DateTime::from_timestamp(expires_at_ts, 0)
                            .context("Invalid expires_at timestamp")?;
                        let created_at = DateTime::from_timestamp(created_at_ts, 0)
                            .context("Invalid created_at timestamp")?;

                        Ok(Some(Fingerprint {
                            fingerprint: row.get("fingerprint"),
                            slug: row.get("slug"),
                            params,
                            ip_address: row.get("ip_address"),
                            expires_at,
                            created_at,
                        }))
                    }
                    None => Ok(None),
                }
            }
        }
    }

    /// Delete expired fingerprints from database
    /// Returns the number of deleted records
    pub async fn cleanup_expired(&self) -> Result<u64> {
        let now = Utc::now().timestamp();

        let rows_deleted = match self.db.as_ref() {
            Database::SQLite(pool) => sqlx::query("DELETE FROM fingerprints WHERE expires_at <= ?")
                .bind(now)
                .execute(pool)
                .await
                .context("Failed to cleanup expired fingerprints")?
                .rows_affected(),
            Database::Postgres(pool) => {
                sqlx::query("DELETE FROM fingerprints WHERE expires_at <= $1")
                    .bind(now)
                    .execute(pool)
                    .await
                    .context("Failed to cleanup expired fingerprints")?
                    .rows_affected()
            }
        };

        Ok(rows_deleted)
    }

    /// Delete a specific fingerprint
    pub async fn delete_fingerprint(&self, fingerprint: &str) -> Result<bool> {
        let rows_deleted = match self.db.as_ref() {
            Database::SQLite(pool) => sqlx::query("DELETE FROM fingerprints WHERE fingerprint = ?")
                .bind(fingerprint)
                .execute(pool)
                .await
                .context("Failed to delete fingerprint")?
                .rows_affected(),
            Database::Postgres(pool) => {
                sqlx::query("DELETE FROM fingerprints WHERE fingerprint = $1")
                    .bind(fingerprint)
                    .execute(pool)
                    .await
                    .context("Failed to delete fingerprint")?
                    .rows_affected()
            }
        };

        Ok(rows_deleted > 0)
    }

    /// Get statistics about stored fingerprints
    pub async fn get_stats(&self) -> Result<FingerprintStats> {
        let now = Utc::now().timestamp();

        let (total, active, expired) = match self.db.as_ref() {
            Database::SQLite(pool) => {
                let total_row = sqlx::query("SELECT COUNT(*) as count FROM fingerprints")
                    .fetch_one(pool)
                    .await
                    .context("Failed to get total count")?;
                let total: i64 = total_row.get::<i32, _>("count") as i64;

                let active_row =
                    sqlx::query("SELECT COUNT(*) as count FROM fingerprints WHERE expires_at > ?")
                        .bind(now)
                        .fetch_one(pool)
                        .await
                        .context("Failed to get active count")?;
                let active: i64 = active_row.get::<i32, _>("count") as i64;

                let expired = total - active;
                (total, active, expired)
            }
            Database::Postgres(pool) => {
                let total_row = sqlx::query("SELECT COUNT(*) as count FROM fingerprints")
                    .fetch_one(pool)
                    .await
                    .context("Failed to get total count")?;
                let total: i64 = total_row.get("count");

                let active_row =
                    sqlx::query("SELECT COUNT(*) as count FROM fingerprints WHERE expires_at > $1")
                        .bind(now)
                        .fetch_one(pool)
                        .await
                        .context("Failed to get active count")?;
                let active: i64 = active_row.get("count");

                let expired = total - active;
                (total, active, expired)
            }
        };

        Ok(FingerprintStats {
            total,
            active,
            expired,
        })
    }
}

/// Statistics about stored fingerprints
#[derive(Debug, Clone)]
pub struct FingerprintStats {
    pub total: i64,
    pub active: i64,
    pub expired: i64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_fingerprint_hash_deterministic() {
        let platform = Platform {
            locale: "en".to_string(),
            os: "android".to_string(),
            timezone: 3,
        };

        let hash1 = FingerprintService::create_fingerprint_hash("192.168.1.1", &platform);
        let hash2 = FingerprintService::create_fingerprint_hash("192.168.1.1", &platform);

        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_create_fingerprint_hash_different_ips() {
        let platform = Platform {
            locale: "en".to_string(),
            os: "android".to_string(),
            timezone: 3,
        };

        let hash1 = FingerprintService::create_fingerprint_hash("192.168.1.1", &platform);
        let hash2 = FingerprintService::create_fingerprint_hash("192.168.1.2", &platform);

        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_create_fingerprint_hash_different_platforms() {
        let platform1 = Platform {
            locale: "en".to_string(),
            os: "android".to_string(),
            timezone: 3,
        };
        let platform2 = Platform {
            locale: "ru".to_string(),
            os: "ios".to_string(),
            timezone: 3,
        };

        let hash1 = FingerprintService::create_fingerprint_hash("192.168.1.1", &platform1);
        let hash2 = FingerprintService::create_fingerprint_hash("192.168.1.1", &platform2);

        assert_ne!(hash1, hash2);
    }
}
