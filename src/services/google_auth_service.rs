use anyhow::{Context, Result, anyhow};
use jsonwebtoken::{Algorithm, DecodingKey, Validation, decode, decode_header};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{debug, info};

/// Google JWT Claims structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoogleClaims {
    /// Issuer - should be "https://accounts.google.com" or "accounts.google.com"
    pub iss: String,
    /// Subject - unique Google user ID
    pub sub: String,
    /// Audience - your Google Client ID
    pub aud: String,
    /// Issued at time (Unix timestamp)
    pub iat: i64,
    /// Expiration time (Unix timestamp)
    pub exp: i64,
    /// User's email address
    pub email: Option<String>,
    /// Whether email is verified
    pub email_verified: Option<bool>,
    /// User's full name
    pub name: Option<String>,
    /// User's profile picture URL
    pub picture: Option<String>,
    /// User's given name
    pub given_name: Option<String>,
    /// User's family name
    pub family_name: Option<String>,
    /// Hosted domain (for Google Workspace accounts)
    pub hd: Option<String>,
}

/// Google's public key from their JWKS endpoint
#[derive(Debug, Clone, Deserialize)]
struct GooglePublicKey {
    kid: String,
    n: String,
    e: String,
    alg: String,
    kty: String,
}

/// Response from Google's JWKS endpoint
#[derive(Debug, Deserialize)]
struct GoogleJwks {
    keys: Vec<GooglePublicKey>,
}

/// Cached public key with expiration
#[derive(Clone)]
struct CachedKey {
    key: DecodingKey,
    expires_at: Instant,
}

/// Cached token validation result
#[derive(Clone)]
struct CachedToken {
    claims: GoogleClaims,
    expires_at: Instant,
}

/// Google JWT validator service
#[derive(Clone)]
pub struct GoogleAuthService {
    client: Client,
    client_id: String,
    keys_cache: Arc<RwLock<HashMap<String, CachedKey>>>,
    tokens_cache: Arc<RwLock<HashMap<String, CachedToken>>>,
    keys_cache_duration: Duration,
    tokens_cache_duration: Duration,
    jwks_url: String,
}

impl GoogleAuthService {
    /// Create a new Google Auth Service
    pub fn new(client_id: String) -> Self {
        Self {
            client: Client::builder()
                .timeout(Duration::from_secs(10))
                .build()
                .expect("Failed to create HTTP client"),
            client_id,
            keys_cache: Arc::new(RwLock::new(HashMap::new())),
            tokens_cache: Arc::new(RwLock::new(HashMap::new())),
            keys_cache_duration: Duration::from_secs(3600), // Cache keys for 1 hour
            tokens_cache_duration: Duration::from_secs(300), // Cache tokens for 5 minutes
            jwks_url: "https://www.googleapis.com/oauth2/v3/certs".to_string(),
        }
    }

    /// Validate a Google ID Token (JWT) with caching
    pub async fn validate_token(&self, token: &str) -> Result<GoogleClaims> {
        // Check token cache first
        {
            let cache = self.tokens_cache.read().await;
            if let Some(cached) = cache.get(token)
                && cached.expires_at > Instant::now()
            {
                debug!("Using cached token validation result");
                return Ok(cached.claims.clone());
            }
        }

        // Validate token (cache miss)
        // Decode header to get the key ID (kid)
        let header = decode_header(token).context("Failed to decode JWT header")?;

        let kid = header
            .kid
            .ok_or_else(|| anyhow!("JWT header missing 'kid' field"))?;

        debug!(kid = %kid, "Validating Google JWT with key ID");

        // Get the public key for this kid
        let decoding_key = self.get_decoding_key(&kid).await?;

        // Setup validation parameters
        let mut validation = Validation::new(Algorithm::RS256);
        validation.set_audience(&[&self.client_id]);
        validation.set_issuer(&["https://accounts.google.com", "accounts.google.com"]);
        validation.validate_exp = true;

        // Decode and validate the token
        let token_data = decode::<GoogleClaims>(token, &decoding_key, &validation)
            .context("Failed to validate JWT")?;

        let claims = token_data.claims;

        // Additional validation
        if let Some(email_verified) = claims.email_verified
            && !email_verified
        {
            return Err(anyhow!("Email not verified"));
        }

        debug!(
            email = claims.email.as_deref().unwrap_or("unknown"),
            sub = %claims.sub,
            "Successfully validated Google JWT"
        );

        // Cache the validated token
        {
            let mut cache = self.tokens_cache.write().await;

            // Clean up expired entries to prevent memory leak
            cache.retain(|_, v| v.expires_at > Instant::now());

            // Limit cache size to prevent abuse (max 1000 tokens)
            if cache.len() > 1000 {
                // Remove oldest entries
                if let Some(oldest_key) = cache
                    .iter()
                    .min_by_key(|(_, v)| v.expires_at)
                    .map(|(k, _)| k.clone())
                {
                    cache.remove(&oldest_key);
                }
            }

            cache.insert(
                token.to_string(),
                CachedToken {
                    claims: claims.clone(),
                    expires_at: Instant::now() + self.tokens_cache_duration,
                },
            );

            debug!(
                cache_size = cache.len(),
                ttl_secs = self.tokens_cache_duration.as_secs(),
                "Cached token validation result"
            );
        }

        Ok(claims)
    }

    /// Get decoding key for a specific key ID (with caching)
    async fn get_decoding_key(&self, kid: &str) -> Result<DecodingKey> {
        // Check cache first
        {
            let cache = self.keys_cache.read().await;
            if let Some(cached) = cache.get(kid) {
                if cached.expires_at > Instant::now() {
                    debug!(kid = %kid, "Using cached public key");
                    return Ok(cached.key.clone());
                } else {
                    debug!(kid = %kid, "Cached public key expired");
                }
            }
        }

        // Fetch fresh keys from Google
        debug!("Fetching fresh public keys from Google");
        let jwks = self.fetch_google_jwks().await?;

        // Find the key we need
        let google_key = jwks
            .keys
            .iter()
            .find(|k| k.kid == kid)
            .ok_or_else(|| anyhow!("Key ID '{}' not found in Google JWKS", kid))?;

        // Convert to DecodingKey
        let decoding_key = DecodingKey::from_rsa_components(&google_key.n, &google_key.e)
            .context("Failed to create decoding key from RSA components")?;

        // Cache all keys
        let mut cache = self.keys_cache.write().await;
        let expires_at = Instant::now() + self.keys_cache_duration;

        for key in &jwks.keys {
            let key_for_cache = DecodingKey::from_rsa_components(&key.n, &key.e)?;
            cache.insert(
                key.kid.clone(),
                CachedKey {
                    key: key_for_cache,
                    expires_at,
                },
            );
        }

        info!(
            keys_cached = jwks.keys.len(),
            cache_duration_secs = self.keys_cache_duration.as_secs(),
            "Cached Google public keys"
        );

        Ok(decoding_key)
    }

    /// Fetch Google's JWKS (JSON Web Key Set)
    async fn fetch_google_jwks(&self) -> Result<GoogleJwks> {
        let response = self
            .client
            .get(&self.jwks_url)
            .send()
            .await
            .context("Failed to fetch Google JWKS")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(anyhow!(
                "Google JWKS request failed with status {}: {}",
                status,
                body
            ));
        }

        let jwks: GoogleJwks = response
            .json()
            .await
            .context("Failed to parse Google JWKS response")?;

        Ok(jwks)
    }

    /// Clear the keys cache (useful for testing or forced refresh)
    pub async fn clear_keys_cache(&self) {
        let mut cache = self.keys_cache.write().await;
        cache.clear();
        info!("Cleared Google public keys cache");
    }

    /// Clear the tokens cache (useful for testing or forced refresh)
    pub async fn clear_tokens_cache(&self) {
        let mut cache = self.tokens_cache.write().await;
        cache.clear();
        info!("Cleared Google tokens cache");
    }

    /// Clear both keys and tokens caches
    pub async fn clear_all_caches(&self) {
        self.clear_keys_cache().await;
        self.clear_tokens_cache().await;
        info!("Cleared all Google auth caches");
    }

    /// Get cache statistics
    pub async fn cache_stats(&self) -> (usize, usize) {
        let keys_count = self.keys_cache.read().await.len();
        let tokens_count = self.tokens_cache.read().await.len();
        (keys_count, tokens_count)
    }

    /// Get the configured Google Client ID
    pub fn client_id(&self) -> &str {
        &self.client_id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_service_creation() {
        let service =
            GoogleAuthService::new("test-client-id.apps.googleusercontent.com".to_string());
        assert_eq!(
            service.client_id(),
            "test-client-id.apps.googleusercontent.com"
        );
        assert_eq!(
            service.jwks_url,
            "https://www.googleapis.com/oauth2/v3/certs"
        );
        assert_eq!(service.keys_cache_duration, Duration::from_secs(3600));
        assert_eq!(service.tokens_cache_duration, Duration::from_secs(300));
    }

    #[tokio::test]
    async fn test_cache_stats() {
        let service =
            GoogleAuthService::new("test-client-id.apps.googleusercontent.com".to_string());
        let (keys, tokens) = service.cache_stats().await;
        assert_eq!(keys, 0);
        assert_eq!(tokens, 0);
    }

    // Note: Integration tests would require a real Google JWT token
    // For testing, you can generate a test token from:
    // https://developers.google.com/oauthplayground/
}
