use crate::api::{response::ApiResult, state::ApiState};
use crate::config::{AUTHOR, DESCRIPTION, HOMEPAGE, LICENSE, NAME, REPOSITORY, VERSION};
use crate::services::{GoogleAuthService, JwtService};
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use utoipa::ToSchema;

/// Public: Health-check handler
/// Checks the health of the application and the database connection
#[utoipa::path(
    get,
    path = "/api/health",
    tag = "public",
    responses(
        (status = 200, description = "Service is healthy"),
        (status = 500, description = "Service is unhealthy")
    )
)]
pub async fn get_health(State(_state): State<ApiState>) -> ApiResult<()> {
    // TODO: Uncomment when database is available
    // match state.db.health_check().await {
    //     Ok(true) => ApiResult::success(HealthStatus {
    //         status: "OK".to_string(),
    //         database: "Connected".to_string(),
    //     }),
    //     Ok(false) => ApiResult::error_with_status(
    //         "DATABASE_ERROR",
    //         "Database health check failed",
    //         StatusCode::INTERNAL_SERVER_ERROR,
    //     ),
    //     Err(e) => {
    //         error!("database connection failed: {}", e);
    //         ApiResult::error_with_status(
    //             "DATABASE_ERROR",
    //             "Database connection failed",
    //             StatusCode::INTERNAL_SERVER_ERROR,
    //         )
    //     }
    // }

    // Temporary response without database
    ApiResult::success(())
}

#[derive(Deserialize, ToSchema)]
pub struct GoogleAuthRequest {
    /// Google JWT token from client
    #[schema(example = "eyJhbGciOiJSUzI1NiIsImtpZCI6...")]
    pub token: String,
}

#[derive(Serialize, ToSchema)]
pub struct GoogleAuthResponse {
    /// Our internal JWT token (non-expiring)
    #[schema(example = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...")]
    pub token: String,
    /// User information
    pub user: UserInfo,
}

#[derive(Serialize, ToSchema)]
pub struct UserInfo {
    /// User ID from provider
    #[schema(example = "123456789")]
    pub id: String,
    /// Provider name
    #[schema(example = "google")]
    pub provider: String,
    /// User's email
    #[schema(example = "user@example.com")]
    pub email: String,
    /// User's full name
    #[schema(example = "John Doe")]
    pub name: String,
    /// User's profile picture URL
    #[schema(example = "https://lh3.googleusercontent.com/a/...")]
    pub picture: Option<String>,
}

/// Public: Exchange Google JWT for internal token
/// Validates Google JWT and returns our internal non-expiring token
#[utoipa::path(
    post,
    path = "/api/v1/auth/google",
    tag = "public",
    request_body = GoogleAuthRequest,
    responses(
        (status = 200, description = "Authentication successful", body = GoogleAuthResponse),
        (status = 401, description = "Invalid or expired Google token"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn post_auth_google(
    State((google_auth, jwt_service)): State<(Arc<GoogleAuthService>, Arc<JwtService>)>,
    Json(payload): Json<GoogleAuthRequest>,
) -> ApiResult<GoogleAuthResponse> {
    // Validate Google JWT
    let google_claims = match google_auth.validate_token(&payload.token).await {
        Ok(claims) => claims,
        Err(e) => {
            return ApiResult::error_with_status(
                "INVALID_TOKEN",
                format!("Invalid Google token: {}", e),
                StatusCode::UNAUTHORIZED,
            );
        }
    };

    // Create internal claims from Google claims
    let internal_claims = JwtService::from_google_claims(&google_claims);

    // Generate our internal JWT (non-expiring)
    let token = match jwt_service.create_token(internal_claims.clone()) {
        Ok(t) => t,
        Err(e) => {
            return ApiResult::error_with_status(
                "TOKEN_GENERATION_FAILED",
                format!("Failed to generate token: {}", e),
                StatusCode::INTERNAL_SERVER_ERROR,
            );
        }
    };

    // Build user info response
    let user = UserInfo {
        id: internal_claims.sub,
        provider: internal_claims.provider,
        email: internal_claims.email,
        name: internal_claims.name,
        picture: internal_claims.picture,
    };

    ApiResult::success(GoogleAuthResponse { token, user })
}

#[derive(Serialize, ToSchema)]
pub struct AboutInfo {
    /// Application name
    name: &'static str,
    /// Application version
    version: &'static str,
    /// Application author
    author: &'static str,
    /// Application description
    description: &'static str,
    /// Repository URL
    repository: &'static str,
    /// License type
    license: &'static str,
    /// Homepage URL
    homepage: &'static str,
}

/// Public: About handler
/// Returns information about the application, such as version
#[utoipa::path(
    get,
    path = "/api/about",
    tag = "public",
    responses(
        (status = 200, description = "Application information", body = AboutInfo)
    )
)]
pub async fn get_about() -> ApiResult<AboutInfo> {
    ApiResult::success(AboutInfo {
        name: NAME,
        version: VERSION,
        author: AUTHOR,
        description: DESCRIPTION,
        repository: REPOSITORY,
        license: LICENSE,
        homepage: HOMEPAGE,
    })
}

#[derive(Serialize, ToSchema)]
pub struct ResolveResponse {
    /// Short link slug identifier
    #[schema(example = "github")]
    pub slug: String,
    /// Parameters associated with this short link
    #[schema(example = json!({"url": "https://github.com"}))]
    pub params: std::collections::HashMap<String, String>,
}

/// Public: Resolve a short link and return its parameters
/// This is the main endpoint for redirecting short links
#[utoipa::path(
    get,
    path = "/api/link/{slug}",
    tag = "public",
    params(
        ("slug" = String, Path, description = "Short link identifier")
    ),
    responses(
        (status = 200, description = "Short link resolved successfully", body = ResolveResponse),
        (status = 404, description = "Short link not found"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn get_resolve_short_link(
    State(state): State<ApiState>,
    Path(slug): Path<String>,
) -> ApiResult<ResolveResponse> {
    match state.short_link_service.resolve_short_link(&slug).await {
        Ok(Some(link)) => ApiResult::success(ResolveResponse {
            slug: link.slug,
            params: link.params,
        }),
        Ok(None) => ApiResult::error_with_status(
            "NOT_FOUND",
            format!("Short link '{}' not found", slug),
            StatusCode::NOT_FOUND,
        ),
        Err(e) => ApiResult::error_with_status(
            "RESOLVE_FAILED",
            format!("Failed to resolve short link: {}", e),
            StatusCode::INTERNAL_SERVER_ERROR,
        ),
    }
}

/// Public: Record a click on a short link
/// This endpoint should be called when a short link is accessed
#[utoipa::path(
    post,
    path = "/api/click/{slug}",
    tag = "public",
    params(
        ("slug" = String, Path, description = "Short link identifier")
    ),
    responses(
        (status = 200, description = "Click recorded successfully"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn post_click_short_link(
    State(state): State<ApiState>,
    Path(slug): Path<String>,
) -> ApiResult<()> {
    match state.short_link_service.click_short_link(&slug).await {
        Ok(_) => ApiResult::success(()),
        Err(e) => ApiResult::error_with_status(
            "CLICK_FAILED",
            format!("Failed to record click: {}", e),
            StatusCode::INTERNAL_SERVER_ERROR,
        ),
    }
}

/// Public: Fallback handler for 404 Not Found
#[utoipa::path(
    get,
    path = "/api/404",
    tag = "public",
    responses(
        (status = 404, description = "Resource not found")
    )
)]
pub async fn not_found() -> ApiResult<()> {
    ApiResult::error_with_status(
        "NOT_FOUND",
        "The requested resource was not found",
        StatusCode::NOT_FOUND,
    )
}
