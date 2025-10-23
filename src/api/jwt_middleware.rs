use anyhow::Result;
use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};
use std::sync::Arc;
use tracing::{info, warn, debug};

use crate::api::response::ApiResult;
use crate::services::JwtService;

/// Middleware for validating our internal JWT tokens
///
/// This middleware validates the internal JWT token signed with our secret key.
/// It does NOT validate Google tokens - those are only used in /auth/google endpoint.
pub async fn internal_jwt_auth_middleware(
    State(jwt_service): State<Arc<JwtService>>,
    mut request: Request,
    next: Next,
) -> Result<Response, Response> {
    let auth_header = request
        .headers()
        .get("Authorization")
        .and_then(|header| header.to_str().ok());

    let token = match auth_header {
        Some(header) if header.starts_with("Bearer ") => &header[7..],
        _ => {
            warn!("Missing or invalid Authorization header");
            let error_response = ApiResult::<()>::error_with_status(
                "UNAUTHORIZED",
                "Missing or invalid Authorization header",
                StatusCode::UNAUTHORIZED,
            );
            return Err(error_response.into_response());
        }
    };

    // Validate internal JWT token
    match jwt_service.validate_token(token) {
        Ok(claims) => {
            info!(
                email = %claims.email,
                sub = %claims.sub,
                provider = %claims.provider,
                "User authenticated with internal JWT"
            );

            // Add claims to request extensions for use in handlers
            request.extensions_mut().insert(claims);

            Ok(next.run(request).await)
        }
        Err(err) => {
            warn!(error = %err, "Internal JWT validation failed");
            let error_response = ApiResult::<()>::error_with_status(
                "INVALID_TOKEN",
                "Invalid or expired authentication token",
                StatusCode::UNAUTHORIZED,
            );
            Err(error_response.into_response())
        }
    }
}

/// Middleware for internal JWT authentication with email verification
///
/// Similar to internal_jwt_auth_middleware, but also checks that user's email
/// is in the allowed emails list
#[derive(Clone)]
pub struct AllowedEmails {
    pub emails: Arc<Vec<String>>,
}

impl AllowedEmails {
    pub fn new(emails: Vec<String>) -> Self {
        Self {
            emails: Arc::new(emails),
        }
    }

    pub fn is_allowed(&self, email: &str) -> bool {
        // Empty list = allow all
        if self.emails.is_empty() {
            return true;
        }

        self.emails.iter().any(|allowed| {
            // Support wildcard domains, e.g. "*@example.com"
            if let Some(domain) = allowed.strip_prefix('*') {
                email.ends_with(domain)
            } else {
                email.eq_ignore_ascii_case(allowed)
            }
        })
    }
}

pub async fn internal_jwt_with_email_check_middleware(
    State((jwt_service, allowed_emails)): State<(Arc<JwtService>, AllowedEmails)>,
    mut request: Request,
    next: Next,
) -> Result<Response, Response> {
    let auth_header = request
        .headers()
        .get("Authorization")
        .and_then(|header| header.to_str().ok());

    let token = match auth_header {
        Some(header) if header.starts_with("Bearer ") => &header[7..],
        _ => {
            warn!("Missing or invalid Authorization header");
            let error_response = ApiResult::<()>::error_with_status(
                "UNAUTHORIZED",
                "Missing or invalid Authorization header",
                StatusCode::UNAUTHORIZED,
            );
            return Err(error_response.into_response());
        }
    };

    // Validate internal JWT token
    let claims = match jwt_service.validate_token(token) {
        Ok(claims) => claims,
        Err(err) => {
            warn!(error = %err, "Internal JWT validation failed");
            let error_response = ApiResult::<()>::error_with_status(
                "INVALID_TOKEN",
                "Invalid or expired authentication token",
                StatusCode::UNAUTHORIZED,
            );
            return Err(error_response.into_response());
        }
    };

    // Check if email is allowed
    if !allowed_emails.is_allowed(&claims.email) {
        warn!(
            email = %claims.email,
            "User authenticated but email not in allowed list"
        );
        let error_response = ApiResult::<()>::error_with_status(
            "FORBIDDEN",
            "Access denied for this email address",
            StatusCode::FORBIDDEN,
        );
        return Err(error_response.into_response());
    }

    debug!(
        email = %claims.email,
        sub = %claims.sub,
        provider = %claims.provider,
        "User authenticated with internal JWT and email check passed"
    );

    // Add claims to request extensions
    request.extensions_mut().insert(claims);

    Ok(next.run(request).await)
}
