use crate::api::{response::ApiResult, state::ApiState};
use crate::config::{AUTHOR, DESCRIPTION, HOMEPAGE, LICENSE, NAME, REPOSITORY, VERSION};
use axum::{
    extract::{Path, State},
    http::StatusCode,
};
use serde::Serialize;
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
