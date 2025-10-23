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
    /// Optional description for the short link (max 500 characters)
    pub description: Option<String>,
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

    /// Validate and normalize description
    /// Returns Some(description) if valid (max 500 chars), None if empty or too long
    pub fn validate_description(description: Option<String>) -> Option<String> {
        description.and_then(|desc| {
            let desc = desc.trim();
            if desc.is_empty() {
                None
            } else if desc.len() > 500 {
                // Truncate to 500 characters
                Some(desc.chars().take(500).collect())
            } else {
                Some(desc.to_string())
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

    #[test]
    fn test_validate_description_valid() {
        let result = ShortLink::validate_description(Some("Valid description".to_string()));
        assert_eq!(result, Some("Valid description".to_string()));
    }

    #[test]
    fn test_validate_description_empty_string() {
        let result = ShortLink::validate_description(Some("".to_string()));
        assert_eq!(result, None);
    }

    #[test]
    fn test_validate_description_whitespace_only() {
        let result = ShortLink::validate_description(Some("   ".to_string()));
        assert_eq!(result, None);
    }

    #[test]
    fn test_validate_description_none() {
        let result = ShortLink::validate_description(None);
        assert_eq!(result, None);
    }

    #[test]
    fn test_validate_description_max_length() {
        let long_desc = "a".repeat(500);
        let result = ShortLink::validate_description(Some(long_desc.clone()));
        assert_eq!(result, Some(long_desc));
    }

    #[test]
    fn test_validate_description_truncate_too_long() {
        let too_long = "a".repeat(600);
        let result = ShortLink::validate_description(Some(too_long));
        assert_eq!(result.unwrap().len(), 500);
    }
}
