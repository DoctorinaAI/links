use axum::{
    extract::{ConnectInfo, Request, State},
    http::{HeaderMap, HeaderValue, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use futures::FutureExt;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;
use tracing::{error, info, warn};

use crate::api::response::ApiResult;
use crate::services::GoogleAuthService;

#[derive(Clone)]
pub struct AdminScope {
    pub secret: String,
}

/// Rate limiting state
#[derive(Clone)]
pub struct RateLimitState {
    pub requests: Arc<RwLock<HashMap<String, (u32, Instant)>>>,
    pub max_requests: u32,
    pub window_duration: std::time::Duration,
}

impl RateLimitState {
    pub fn new(max_requests: u32, window_seconds: u64) -> Self {
        Self {
            requests: Arc::new(RwLock::new(HashMap::new())),
            max_requests,
            window_duration: std::time::Duration::from_secs(window_seconds),
        }
    }
}

pub async fn auth_middleware(
    State(private): State<AdminScope>,
    headers: HeaderMap,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let auth_header = headers
        .get("Authorization")
        .and_then(|header| header.to_str().ok());

    match auth_header {
        Some(token) if token.starts_with("Bearer ") => {
            let token = &token[7..]; // Remove "Bearer " prefix
            if token == private.secret {
                Ok(next.run(request).await)
            } else {
                Err(StatusCode::UNAUTHORIZED)
            }
        }
        _ => Err(StatusCode::UNAUTHORIZED),
    }
}

/// Middleware for logging requests with execution time and IP information
pub async fn logging_middleware(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    request: Request,
    next: Next,
) -> Response {
    let start = Instant::now();
    let method = request.method().clone();
    let uri = request.uri().clone();
    let user_agent = request
        .headers()
        .get("user-agent")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown")
        .to_string();

    // Get real IP from headers (if behind proxy)
    let real_ip = get_real_ip(&request, addr);

    let response = next.run(request).await;
    let duration = start.elapsed();
    let status = response.status();

    // Log the request
    if status.is_server_error() {
        warn!(
            ip = %real_ip,
            method = %method,
            uri = %uri,
            status = %status,
            duration_ms = duration.as_millis(),
            user_agent = user_agent,
            "Request completed with server error"
        );
    } else if status.is_client_error() {
        warn!(
            ip = %real_ip,
            method = %method,
            uri = %uri,
            status = %status,
            duration_ms = duration.as_millis(),
            user_agent = user_agent,
            "Request completed with client error"
        );
    } else {
        info!(
            ip = %real_ip,
            method = %method,
            uri = %uri,
            status = %status,
            duration_ms = duration.as_millis(),
            user_agent = user_agent,
            "Request completed successfully"
        );
    }

    response
}

/// Extracts real IP address from headers or uses connection address as fallback
fn get_real_ip(request: &Request, fallback_addr: SocketAddr) -> String {
    // Check headers in priority order
    let headers = request.headers();

    // X-Forwarded-For (most common)
    if let Some(forwarded_for) = headers.get("x-forwarded-for")
        && let Ok(value) = forwarded_for.to_str()
    {
        // Take first IP from the list
        if let Some(first_ip) = value.split(',').next() {
            return first_ip.trim().to_string();
        }
    }

    // X-Real-IP (used by Nginx)
    if let Some(real_ip) = headers.get("x-real-ip")
        && let Ok(value) = real_ip.to_str()
    {
        return value.to_string();
    }

    // CF-Connecting-IP (Cloudflare)
    if let Some(cf_ip) = headers.get("cf-connecting-ip")
        && let Ok(value) = cf_ip.to_str()
    {
        return value.to_string();
    }

    // X-Forwarded (less common)
    if let Some(forwarded) = headers.get("x-forwarded")
        && let Ok(value) = forwarded.to_str()
        && let Some(for_part) = value
            .split(';')
            .find(|part| part.trim().starts_with("for="))
        && let Some(ip) = for_part.split('=').nth(1)
    {
        return ip.trim_matches('"').to_string();
    }

    // If nothing found, use connection address
    fallback_addr.ip().to_string()
}

/// Middleware for adding security headers
pub async fn security_middleware(request: Request, next: Next) -> Response {
    let mut response = next.run(request).await;

    let headers = response.headers_mut();

    // Add security headers
    headers.insert(
        "X-Content-Type-Options",
        HeaderValue::from_static("nosniff"),
    );
    headers.insert("X-Frame-Options", HeaderValue::from_static("DENY"));
    headers.insert(
        "X-XSS-Protection",
        HeaderValue::from_static("1; mode=block"),
    );
    headers.insert(
        "Referrer-Policy",
        HeaderValue::from_static("strict-origin-when-cross-origin"),
    );
    headers.insert(
        "Content-Security-Policy",
        HeaderValue::from_static(
            "default-src 'self'; img-src 'self' data:; style-src 'self' 'unsafe-inline'",
        ),
    );

    response
}

/// Middleware for handling panics and converting them to ApiErrorResponse
pub async fn panic_recovery_middleware(request: Request, next: Next) -> Response {
    let result = std::panic::AssertUnwindSafe(next.run(request))
        .catch_unwind()
        .await;

    match result {
        Ok(response) => response,
        Err(panic_info) => {
            let panic_message = if let Some(s) = panic_info.downcast_ref::<String>() {
                s.clone()
            } else if let Some(s) = panic_info.downcast_ref::<&str>() {
                s.to_string()
            } else {
                "Unknown panic occurred".to_string()
            };

            error!("panic occurred in request handler: {}", panic_message);

            // Create standard error response
            let error_response = ApiResult::<()>::error_with_status(
                "INTERNAL_SERVER_ERROR",
                "An internal server error occurred",
                StatusCode::INTERNAL_SERVER_ERROR,
            );

            error_response.into_response()
        }
    }
}

/// Middleware for adding performance metrics to response headers
pub async fn metrics_middleware(request: Request, next: Next) -> Response {
    let start = Instant::now();
    let method = request.method().clone();
    let uri = request.uri().clone();

    // Run the handler
    let mut response = next.run(request).await;

    let duration = start.elapsed();
    let headers = response.headers_mut();

    // Add metrics to headers
    if let Ok(duration_ms) = HeaderValue::from_str(&duration.as_millis().to_string()) {
        headers.insert("X-Response-Time-Ms", duration_ms);
    }

    if let Ok(timestamp) = HeaderValue::from_str(&chrono::Utc::now().timestamp().to_string()) {
        headers.insert("X-Timestamp", timestamp);
    }

    headers.insert("X-Server", HeaderValue::from_static("vixen-rs"));
    headers.insert(
        "X-Version",
        HeaderValue::from_static(env!("CARGO_PKG_VERSION")),
    );

    // Log metrics for analysis
    info!(
        method = %method,
        uri = %uri,
        duration_ms = duration.as_millis(),
        status = %response.status(),
        "Request metrics"
    );

    response
}

/// Rate limiting middleware for protection against abuse
pub async fn rate_limit_middleware(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State(rate_limit): State<RateLimitState>,
    request: Request,
    next: Next,
) -> Result<Response, Response> {
    let client_ip = addr.ip().to_string();
    let now = Instant::now();

    {
        let mut requests = rate_limit.requests.write().await;

        // Clean up old entries
        requests.retain(|_, (_, timestamp)| {
            now.duration_since(*timestamp) < rate_limit.window_duration
        });

        // Check limit for current IP
        let (count, first_request) = requests.entry(client_ip.clone()).or_insert((0, now));

        if now.duration_since(*first_request) > rate_limit.window_duration {
            // Window expired, reset counter
            *count = 1;
            *first_request = now;
        } else {
            *count += 1;
            if *count > rate_limit.max_requests {
                warn!(
                    ip = %client_ip,
                    requests = *count,
                    limit = rate_limit.max_requests,
                    "Rate limit exceeded"
                );

                let error_response = ApiResult::<()>::error_with_status(
                    "RATE_LIMIT_EXCEEDED",
                    "Too many requests. Please try again later.",
                    StatusCode::TOO_MANY_REQUESTS,
                );

                return Err(error_response.into_response());
            }
        }
    }

    Ok(next.run(request).await)
}

/// Middleware for validating request size and other parameters
pub async fn request_validation_middleware(
    request: Request,
    next: Next,
) -> Result<Response, Response> {
    let method = request.method();
    let uri = request.uri();

    // Check maximum URI length
    if uri.path().len() > 2048 {
        warn!(uri = %uri, "Request URI too long");
        let error_response = ApiResult::<()>::error_with_status(
            "URI_TOO_LONG",
            "Request URI is too long",
            StatusCode::URI_TOO_LONG,
        );
        return Err(error_response.into_response());
    }

    // Check request method
    if !matches!(
        method,
        &axum::http::Method::GET
            | &axum::http::Method::POST
            | &axum::http::Method::PUT
            | &axum::http::Method::DELETE
            | &axum::http::Method::OPTIONS
            | &axum::http::Method::HEAD
    ) {
        warn!(method = %method, "Unsupported HTTP method");
        let error_response = ApiResult::<()>::error_with_status(
            "METHOD_NOT_ALLOWED",
            "HTTP method not allowed",
            StatusCode::METHOD_NOT_ALLOWED,
        );
        return Err(error_response.into_response());
    }

    Ok(next.run(request).await)
}

/// Middleware for Google JWT authentication
///
/// Validates JWT token from Authorization header via Google
/// Adds user information to request extensions for use in handlers
pub async fn google_auth_middleware(
    State(google_auth): State<GoogleAuthService>,
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

    // Validate Google JWT token
    match google_auth.validate_token(token).await {
        Ok(claims) => {
            info!(
                email = claims.email.as_deref().unwrap_or("unknown"),
                sub = %claims.sub,
                "User authenticated via Google"
            );

            // Add claims to request extensions for use in handlers
            request.extensions_mut().insert(claims);

            Ok(next.run(request).await)
        }
        Err(err) => {
            warn!(error = %err, "Google JWT validation failed");
            let error_response = ApiResult::<()>::error_with_status(
                "INVALID_TOKEN",
                "Invalid or expired authentication token",
                StatusCode::UNAUTHORIZED,
            );
            Err(error_response.into_response())
        }
    }
}

/// Middleware for Google JWT authentication with email verification
///
/// Similar to google_auth_middleware, but also checks that user's email
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

pub async fn google_auth_with_email_check_middleware(
    State((google_auth, allowed_emails)): State<(GoogleAuthService, AllowedEmails)>,
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

    // Validate Google JWT token
    match google_auth.validate_token(token).await {
        Ok(claims) => {
            // Check email
            let email = claims.email.as_deref().unwrap_or("");

            if !allowed_emails.is_allowed(email) {
                warn!(
                    email = email,
                    "User authenticated but email not in allowed list"
                );
                let error_response = ApiResult::<()>::error_with_status(
                    "FORBIDDEN",
                    "Access denied for this email address",
                    StatusCode::FORBIDDEN,
                );
                return Err(error_response.into_response());
            }

            info!(
                email = email,
                sub = %claims.sub,
                "User authenticated via Google with email check"
            );

            // Add claims to request extensions
            request.extensions_mut().insert(claims);

            Ok(next.run(request).await)
        }
        Err(err) => {
            warn!(error = %err, "Google JWT validation failed");
            let error_response = ApiResult::<()>::error_with_status(
                "INVALID_TOKEN",
                "Invalid or expired authentication token",
                StatusCode::UNAUTHORIZED,
            );
            Err(error_response.into_response())
        }
    }
}
