use anyhow::{Context, Result};
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};

/// Our internal JWT claims (non-expiring)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InternalClaims {
    /// Subject - user ID from provider
    pub sub: String,
    /// Provider (e.g., "google")
    pub provider: String,
    /// User's email
    pub email: String,
    /// Email verified status
    pub email_verified: bool,
    /// User's full name
    pub name: String,
    /// User's given name
    pub given_name: Option<String>,
    /// User's family name
    pub family_name: Option<String>,
    /// User's profile picture URL
    pub picture: Option<String>,
    /// Issued at timestamp
    pub iat: i64,
    // Note: No exp field - token never expires
}

/// JWT service for creating and validating internal tokens
pub struct JwtService {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
    secret: String,
}

impl JwtService {
    /// Create a new JWT service with a secret key
    pub fn new(secret: String) -> Self {
        Self {
            encoding_key: EncodingKey::from_secret(secret.as_bytes()),
            decoding_key: DecodingKey::from_secret(secret.as_bytes()),
            secret,
        }
    }

    /// Create a non-expiring JWT token from claims
    pub fn create_token(&self, claims: InternalClaims) -> Result<String> {
        let token = encode(&Header::default(), &claims, &self.encoding_key)
            .context("Failed to encode JWT")?;
        Ok(token)
    }

    /// Validate and decode internal JWT token
    pub fn validate_token(&self, token: &str) -> Result<InternalClaims> {
        let mut validation = Validation::new(Algorithm::HS256);
        // Don't validate expiration since our tokens are non-expiring
        validation.validate_exp = false;
        // Remove 'exp' from required claims since we don't use it
        validation.required_spec_claims.clear();

        let token_data = decode::<InternalClaims>(token, &self.decoding_key, &validation)
            .context("Failed to decode and validate JWT")?;

        Ok(token_data.claims)
    }

    /// Create internal claims from Google claims
    pub fn from_google_claims(google_claims: &crate::services::GoogleClaims) -> InternalClaims {
        InternalClaims {
            sub: google_claims.sub.clone(),
            provider: "google".to_string(),
            email: google_claims.email.clone().unwrap_or_default(),
            email_verified: google_claims.email_verified.unwrap_or(false),
            name: google_claims.name.clone().unwrap_or_default(),
            given_name: google_claims.given_name.clone(),
            family_name: google_claims.family_name.clone(),
            picture: google_claims.picture.clone(),
            iat: chrono::Utc::now().timestamp(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jwt_service_creation() {
        let service = JwtService::new("test-secret-key-at-least-32-chars-long".to_string());
        // Service should be created successfully
        assert!(std::ptr::addr_of!(service.encoding_key) as usize != 0);
    }

    #[test]
    fn test_create_and_validate_token() {
        let service = JwtService::new("test-secret-key-at-least-32-chars-long".to_string());
        let claims = InternalClaims {
            sub: "123456".to_string(),
            provider: "google".to_string(),
            email: "test@example.com".to_string(),
            email_verified: true,
            name: "Test User".to_string(),
            given_name: Some("Test".to_string()),
            family_name: Some("User".to_string()),
            picture: Some("https://example.com/avatar.jpg".to_string()),
            iat: 1234567890,
        };

        // Create token
        let token = service.create_token(claims.clone());
        assert!(token.is_ok());
        let token_str = token.unwrap();
        assert!(!token_str.is_empty());

        // Validate token
        let decoded_claims = service.validate_token(&token_str);
        assert!(decoded_claims.is_ok());

        let decoded = decoded_claims.unwrap();
        assert_eq!(decoded.sub, claims.sub);
        assert_eq!(decoded.email, claims.email);
        assert_eq!(decoded.provider, claims.provider);
        assert_eq!(decoded.name, claims.name);
    }

    #[test]
    fn test_invalid_token() {
        let service = JwtService::new("test-secret-key-at-least-32-chars-long".to_string());

        // Try to validate an invalid token
        let result = service.validate_token("invalid.token.here");
        assert!(result.is_err());
    }

    #[test]
    fn test_token_with_different_secret() {
        let service1 = JwtService::new("secret1-at-least-32-chars-long-key".to_string());
        let service2 = JwtService::new("secret2-at-least-32-chars-long-key".to_string());

        let claims = InternalClaims {
            sub: "123456".to_string(),
            provider: "google".to_string(),
            email: "test@example.com".to_string(),
            email_verified: true,
            name: "Test User".to_string(),
            given_name: None,
            family_name: None,
            picture: None,
            iat: chrono::Utc::now().timestamp(),
        };

        // Create token with service1
        let token = service1.create_token(claims).unwrap();

        // Try to validate with service2 (different secret)
        let result = service2.validate_token(&token);
        assert!(result.is_err());
    }
}
