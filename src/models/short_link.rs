use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShortLink {
    pub slug: String,
    pub params: HashMap<String, String>,
    pub author: String,
    /// Optional redirect URL - if set, user will be forcefully redirected to this URL
    /// Must be a valid HTTPS URL, otherwise stored as None
    pub redirect: Option<String>,
    pub updated_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

impl ShortLink {
    /// Validate and normalize redirect URL
    /// Returns Some(url) if valid HTTPS URL, None otherwise
    pub fn validate_redirect(redirect: Option<String>) -> Option<String> {
        redirect.and_then(|url| {
            let url = url.trim();
            if url.is_empty() {
                return None;
            }

            // Check if URL starts with https://
            if url.starts_with("https://") {
                // Basic validation - check if it's parseable as URL
                if url::Url::parse(url).is_ok() {
                    Some(url.to_string())
                } else {
                    None
                }
            } else {
                None
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_redirect_valid_https() {
        let result = ShortLink::validate_redirect(Some("https://example.com".to_string()));
        assert_eq!(result, Some("https://example.com".to_string()));
    }

    #[test]
    fn test_validate_redirect_http_rejected() {
        let result = ShortLink::validate_redirect(Some("http://example.com".to_string()));
        assert_eq!(result, None);
    }

    #[test]
    fn test_validate_redirect_invalid_url() {
        let result = ShortLink::validate_redirect(Some("not-a-url".to_string()));
        assert_eq!(result, None);
    }

    #[test]
    fn test_validate_redirect_empty_string() {
        let result = ShortLink::validate_redirect(Some("".to_string()));
        assert_eq!(result, None);
    }

    #[test]
    fn test_validate_redirect_none() {
        let result = ShortLink::validate_redirect(None);
        assert_eq!(result, None);
    }

    #[test]
    fn test_validate_redirect_with_path() {
        let result =
            ShortLink::validate_redirect(Some("https://example.com/path?query=1".to_string()));
        assert_eq!(result, Some("https://example.com/path?query=1".to_string()));
    }
}
