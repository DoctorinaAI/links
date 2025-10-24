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
use tower_http::cors::CorsLayer;
use tracing::{error, info, warn};

use crate::api::response::ApiResult;
use crate::services::GoogleAuthService;

/// Create a CORS layer with configurable allowed origins
///
/// If origins is empty, creates a permissive CORS layer allowing all origins.
/// Otherwise, restricts CORS to the specified origins.
/// All methods and headers are always allowed.
///
/// Example:
/// ```ignore
/// // Allow all origins
/// let cors = create_cors_layer(vec![]);
///
/// // Restrict to specific origins
/// let cors = create_cors_layer(vec!["https://example.com".to_string(), "https://app.example.com".to_string()]);
/// ```
pub fn create_cors_layer(origins: Vec<String>) -> CorsLayer {
    if origins.is_empty() {
        // Permissive mode: allow all origins
        CorsLayer::permissive()
    } else {
        // Restricted mode: allow only specified origins
        use tower_http::cors::AllowOrigin;

        let allowed_origins: Vec<axum::http::HeaderValue> = origins
            .into_iter()
            .filter_map(|origin| origin.parse().ok())
            .collect();

        CorsLayer::new()
            .allow_origin(AllowOrigin::list(allowed_origins))
            .allow_methods(tower_http::cors::Any)
            .allow_headers(tower_http::cors::Any)
            .allow_credentials(true)
    }
}

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

/// Check for admin authentication via a static secret token
///
/// # Deprecated
/// This middleware is deprecated. Use JWT-based authentication instead.
/// See `jwt_middleware::internal_jwt_with_email_check_middleware` for the recommended approach.
#[deprecated(
    since = "0.0.1",
    note = "Use JWT-based authentication (jwt_middleware) instead of static secret"
)]
#[allow(dead_code)]
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
///
/// # Deprecated
/// This middleware is deprecated. Use `server_timing_middleware` for more detailed timing metrics.
/// The `server_timing_middleware` provides better breakdown of request processing time and follows
/// the W3C Server-Timing specification.
#[deprecated(
    since = "0.0.1",
    note = "Use server_timing_middleware for more detailed performance metrics"
)]
#[allow(dead_code)]
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

    // Add Server-Timing header for detailed performance metrics
    // Format: Server-Timing: metric_name;dur=duration_in_ms;desc="description"
    let server_timing = format!(
        "total;dur={};desc=\"Total request time\"",
        duration.as_millis()
    );
    if let Ok(timing_value) = HeaderValue::from_str(&server_timing) {
        headers.insert("Server-Timing", timing_value);
    }

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

/// Middleware for adding detailed Server-Timing headers with component breakdown
/// This provides more granular timing information for debugging and optimization
///
/// Uses RequestMetrics from request extensions to collect timing data from handlers
pub async fn server_timing_middleware(mut request: Request, next: Next) -> Response {
    use crate::api::timing::RequestMetrics;

    // Initialize metrics tracker and inject into request extensions
    let metrics = RequestMetrics::new();
    request.extensions_mut().insert(metrics.clone());

    // Measure total request time
    let _total_guard = metrics.measure("total");

    // Measure handler execution time
    let handler_guard = metrics.measure("handler");
    let mut response = next.run(request).await;
    drop(handler_guard);

    // Build and attach Server-Timing header from collected metrics
    if let Some(timing_header) = metrics.build_header() {
        response
            .headers_mut()
            .insert("Server-Timing", timing_header);
    }

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

/// Middleware for Google JWT authentication with token caching
///
/// Validates JWT token from Authorization header via Google
/// Token validation is automatically cached in GoogleAuthService (5 min TTL)
/// Adds user information to request extensions for use in handlers
///
/// # Deprecated
/// This middleware is deprecated. Use the internal JWT approach instead.
/// See `jwt_middleware::internal_jwt_with_email_check_middleware` which validates internal
/// non-expiring JWT tokens (signed by the server after Google authentication).
#[deprecated(
    since = "0.0.1",
    note = "Use internal JWT middleware (jwt_middleware::internal_jwt_with_email_check_middleware) instead"
)]
#[allow(dead_code)]
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

    // Validate Google JWT token (with internal caching)
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
                StatusCode::UNAUTHORIZED, // 401
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

/// Middleware for Google JWT authentication with email verification
///
/// Similar to google_auth_middleware, but also checks that user's email
/// is in the allowed emails list
///
/// # Deprecated
/// This middleware is deprecated. Use the internal JWT approach instead.
/// See `jwt_middleware::internal_jwt_with_email_check_middleware` which validates internal
/// non-expiring JWT tokens with email checking (signed by the server after Google authentication).
#[deprecated(
    since = "0.0.1",
    note = "Use internal JWT middleware (jwt_middleware::internal_jwt_with_email_check_middleware) instead"
)]
#[allow(dead_code)]
pub async fn google_auth_with_email_check_middleware(
    State((google_auth, allowed_emails)): State<(Arc<GoogleAuthService>, AllowedEmails)>,
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

    // Validate Google JWT token (with internal caching)
    let claims = match google_auth.validate_token(token).await {
        Ok(claims) => claims,
        Err(err) => {
            warn!(error = %err, "Google JWT validation failed");
            let error_response = ApiResult::<()>::error_with_status(
                "INVALID_TOKEN",
                "Invalid or expired authentication token",
                StatusCode::UNAUTHORIZED,
            );
            return Err(error_response.into_response());
        }
    };

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

/// Middleware for adding unique request ID to each request
/// Useful for request tracing and debugging across distributed systems
pub async fn request_id_middleware(mut request: Request, next: Next) -> Response {
    // Simple request ID generation without uuid crate (using timestamp + random)
    let request_id = request
        .headers()
        .get("X-Request-ID")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
        .unwrap_or_else(|| {
            use std::time::{SystemTime, UNIX_EPOCH};
            let timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_micros();
            format!("req-{:x}", timestamp)
        });

    // Store request ID in extensions for use in handlers
    request.extensions_mut().insert(request_id.clone());

    let mut response = next.run(request).await;

    // Add request ID to response headers
    if let Ok(header_value) = HeaderValue::from_str(&request_id) {
        response.headers_mut().insert("X-Request-ID", header_value);
    }

    response
}

/// Middleware for request/response body size tracking
/// Adds headers with body sizes for monitoring and optimization
pub async fn body_size_middleware(request: Request, next: Next) -> Response {
    // Get request body size if available from Content-Length header (clone before moving)
    let request_size = request
        .headers()
        .get("Content-Length")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.parse::<usize>().ok())
        .map(|size| size.to_string());

    let mut response = next.run(request).await;

    // Add request size to response headers for monitoring
    if let Some(size_str) = request_size
        && let Ok(size_value) = HeaderValue::from_str(&size_str)
    {
        response.headers_mut().insert("X-Request-Size", size_value);
    }

    // Note: Response body size would require buffering the entire response
    // which is expensive. Better to rely on Content-Length from the handler.

    response
}

/// Middleware for adding cache control headers
/// Configures caching behavior for different types of responses
pub async fn cache_control_middleware(request: Request, next: Next) -> Response {
    let path = request.uri().path().to_string();
    let mut response = next.run(request).await;

    // Don't add cache headers if already present
    if response.headers().contains_key("Cache-Control") {
        return response;
    }

    let cache_header = if path.starts_with("/api/") {
        // API responses: no cache by default
        "no-cache, no-store, must-revalidate"
    } else if path.starts_with("/static/") || path.ends_with(".js") || path.ends_with(".css") {
        // Static assets: cache for 1 year
        "public, max-age=31536000, immutable"
    } else if path.starts_with("/api-docs/") || path == "/scalar" {
        // API docs: cache for 5 minutes
        "public, max-age=300"
    } else {
        // Everything else: no cache
        "no-cache"
    };

    response
        .headers_mut()
        .insert("Cache-Control", HeaderValue::from_static(cache_header));

    response
}

/// Middleware for request timeout handling
/// Returns 408 Request Timeout if handler takes too long
pub async fn timeout_middleware(request: Request, next: Next) -> Result<Response, Response> {
    use tokio::time::{Duration, timeout};

    // 30 second timeout for all requests
    let timeout_duration = Duration::from_secs(30);

    match timeout(timeout_duration, next.run(request)).await {
        Ok(response) => Ok(response),
        Err(_) => {
            warn!("Request timed out after {:?}", timeout_duration);
            let error_response = ApiResult::<()>::error_with_status(
                "REQUEST_TIMEOUT",
                "Request processing timed out",
                StatusCode::REQUEST_TIMEOUT,
            );
            Err(error_response.into_response())
        }
    }
}
