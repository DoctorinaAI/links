use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Platform information for fingerprinting
/// Used to create unique fingerprints based on user's platform characteristics
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq, Eq)]
pub struct Platform {
    /// User's locale/language (e.g., "en", "ru", "en-US")
    #[schema(example = "en")]
    pub locale: String,

    /// Operating system (e.g., "android", "ios", "windows", "macos", "linux")
    #[schema(example = "android")]
    pub os: String,

    /// Timezone offset in hours from UTC (e.g., 3, -5, 0)
    #[schema(example = 3)]
    pub timezone: i32,
}

impl Platform {
    /// Validate platform data
    /// Returns error message if validation fails
    pub fn validate(&self) -> Result<(), String> {
        // Validate locale (should be 2-5 characters, alphanumeric with hyphens)
        if self.locale.is_empty() || self.locale.len() > 10 {
            return Err("locale must be between 1 and 10 characters".to_string());
        }
        if !self
            .locale
            .chars()
            .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
        {
            return Err(
                "locale must contain only alphanumeric characters, hyphens, and underscores"
                    .to_string(),
            );
        }

        // Validate OS (should be reasonable length and alphanumeric)
        if self.os.is_empty() || self.os.len() > 20 {
            return Err("os must be between 1 and 20 characters".to_string());
        }
        if !self.os.chars().all(|c| c.is_alphanumeric() || c == '-') {
            return Err("os must contain only alphanumeric characters and hyphens".to_string());
        }

        // Validate timezone (should be between -12 and +14)
        if !(-12..=14).contains(&self.timezone) {
            return Err("timezone must be between -12 and +14".to_string());
        }

        Ok(())
    }

    /// Normalize platform data for consistent fingerprinting
    /// Converts to lowercase and trims whitespace
    pub fn normalize(&mut self) {
        self.locale = self.locale.trim().to_lowercase();
        self.os = self.os.trim().to_lowercase();
    }

    /// Create a canonical string representation for hashing
    /// Format: "locale|os|timezone"
    /// The values are sorted alphabetically by key to ensure consistent hashing
    pub fn to_canonical_string(&self) -> String {
        // Sort fields alphabetically by key name for consistent ordering
        format!(
            "locale:{}|os:{}|timezone:{}",
            self.locale, self.os, self.timezone
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_platform_validation_valid() {
        let platform = Platform {
            locale: "en".to_string(),
            os: "android".to_string(),
            timezone: 3,
        };
        assert!(platform.validate().is_ok());
    }

    #[test]
    fn test_platform_validation_locale_too_long() {
        let platform = Platform {
            locale: "a".repeat(11),
            os: "android".to_string(),
            timezone: 3,
        };
        assert!(platform.validate().is_err());
    }

    #[test]
    fn test_platform_validation_locale_empty() {
        let platform = Platform {
            locale: "".to_string(),
            os: "android".to_string(),
            timezone: 3,
        };
        assert!(platform.validate().is_err());
    }

    #[test]
    fn test_platform_validation_os_too_long() {
        let platform = Platform {
            locale: "en".to_string(),
            os: "a".repeat(21),
            timezone: 3,
        };
        assert!(platform.validate().is_err());
    }

    #[test]
    fn test_platform_validation_timezone_out_of_range() {
        let mut platform = Platform {
            locale: "en".to_string(),
            os: "android".to_string(),
            timezone: -13,
        };
        assert!(platform.validate().is_err());

        platform.timezone = 15;
        assert!(platform.validate().is_err());
    }

    #[test]
    fn test_platform_normalize() {
        let mut platform = Platform {
            locale: " EN-US ".to_string(),
            os: " Android ".to_string(),
            timezone: 3,
        };
        platform.normalize();
        assert_eq!(platform.locale, "en-us");
        assert_eq!(platform.os, "android");
    }

    #[test]
    fn test_platform_to_canonical_string() {
        let platform = Platform {
            locale: "en".to_string(),
            os: "android".to_string(),
            timezone: 3,
        };
        assert_eq!(
            platform.to_canonical_string(),
            "locale:en|os:android|timezone:3"
        );
    }

    #[test]
    fn test_platform_canonical_string_consistency() {
        let platform1 = Platform {
            locale: "en".to_string(),
            os: "android".to_string(),
            timezone: 3,
        };
        let platform2 = Platform {
            locale: "en".to_string(),
            os: "android".to_string(),
            timezone: 3,
        };
        assert_eq!(
            platform1.to_canonical_string(),
            platform2.to_canonical_string()
        );
    }
}
