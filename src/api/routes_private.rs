use crate::api::{response::ApiResult, state::ApiState};
use crate::models::ShortLink;
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Private: check autentication handler
#[utoipa::path(
    get,
    path = "/api/v1/admin/check",
    tag = "admin",
    responses(
        (status = 200, description = "Authentication successful")
    )
)]
pub async fn get_check() -> ApiResult<String> {
    let version = env!("CARGO_PKG_VERSION");
    ApiResult::success(format!("Current app version: {}", version))
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateShortLinkRequest {
    /// Unique identifier for the short link (slug)
    #[schema(example = "github")]
    pub slug: String,
    /// Parameters as key-value pairs
    #[schema(example = json!({"url": "https://github.com"}))]
    pub params: std::collections::HashMap<String, String>,
    /// Author of the short link
    #[schema(example = "admin")]
    pub author: String,
    /// Optional redirect URL (must be valid HTTPS URL)
    #[schema(example = "https://example.com")]
    pub redirect: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ShortLinkResponse {
    /// Short link slug
    pub slug: String,
    /// Associated parameters
    pub params: std::collections::HashMap<String, String>,
    /// Author name
    pub author: String,
    /// Optional redirect URL
    pub redirect: Option<String>,
    /// Creation timestamp (RFC3339)
    #[schema(example = "2025-10-22T10:00:00Z")]
    pub created_at: String,
    /// Last update timestamp (RFC3339)
    #[schema(example = "2025-10-22T10:00:00Z")]
    pub updated_at: String,
}

impl From<ShortLink> for ShortLinkResponse {
    fn from(link: ShortLink) -> Self {
        Self {
            slug: link.slug,
            params: link.params,
            author: link.author,
            redirect: link.redirect,
            created_at: link.created_at.to_rfc3339(),
            updated_at: link.updated_at.to_rfc3339(),
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ShortLinksListResponse {
    /// List of all short links
    pub links: Vec<ShortLinkResponse>,
    /// Total count
    pub count: usize,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ClickStatsResponse {
    /// Short link slug
    pub slug: String,
    /// Total number of clicks
    pub total_clicks: i64,
    /// Recent click timestamps (RFC3339)
    pub recent_clicks: Vec<String>,
}

/// Private: Create a new short link
#[utoipa::path(
    post,
    path = "/api/v1/admin/links",
    tag = "admin",
    request_body = CreateShortLinkRequest,
    responses(
        (status = 200, description = "Short link created successfully", body = ShortLinkResponse),
        (status = 500, description = "Failed to create short link")
    )
)]
pub async fn post_create_short_link(
    State(state): State<ApiState>,
    Json(payload): Json<CreateShortLinkRequest>,
) -> ApiResult<ShortLinkResponse> {
    let short_link = ShortLink {
        slug: payload.slug.clone(),
        params: payload.params,
        author: payload.author,
        redirect: payload.redirect,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    match state.short_link_service.create_short_link(short_link).await {
        Ok(link) => ApiResult::success(link.into()),
        Err(e) => ApiResult::error_with_status(
            "CREATE_FAILED",
            format!("Failed to create short link: {}", e),
            StatusCode::INTERNAL_SERVER_ERROR,
        ),
    }
}

/// Private: Get all short links
#[utoipa::path(
    get,
    path = "/api/v1/admin/links",
    tag = "admin",
    responses(
        (status = 200, description = "List of all short links", body = ShortLinksListResponse),
        (status = 500, description = "Failed to list short links")
    )
)]
pub async fn get_list_short_links(
    State(state): State<ApiState>,
) -> ApiResult<ShortLinksListResponse> {
    match state.short_link_service.list_short_links().await {
        Ok(links) => {
            let count = links.len();
            let response = ShortLinksListResponse {
                links: links.into_iter().map(|l| l.into()).collect(),
                count,
            };
            ApiResult::success(response)
        }
        Err(e) => ApiResult::error_with_status(
            "LIST_FAILED",
            format!("Failed to list short links: {}", e),
            StatusCode::INTERNAL_SERVER_ERROR,
        ),
    }
}

/// Private: Get a specific short link by slug
#[utoipa::path(
    get,
    path = "/api/v1/admin/links/{slug}",
    tag = "admin",
    params(
        ("slug" = String, Path, description = "Short link identifier")
    ),
    responses(
        (status = 200, description = "Short link details", body = ShortLinkResponse),
        (status = 404, description = "Short link not found"),
        (status = 500, description = "Failed to fetch short link")
    )
)]
pub async fn get_short_link(
    State(state): State<ApiState>,
    Path(slug): Path<String>,
) -> ApiResult<ShortLinkResponse> {
    match state.short_link_service.resolve_short_link(&slug).await {
        Ok(Some(link)) => ApiResult::success(link.into()),
        Ok(None) => ApiResult::error_with_status(
            "NOT_FOUND",
            format!("Short link '{}' not found", slug),
            StatusCode::NOT_FOUND,
        ),
        Err(e) => ApiResult::error_with_status(
            "FETCH_FAILED",
            format!("Failed to fetch short link: {}", e),
            StatusCode::INTERNAL_SERVER_ERROR,
        ),
    }
}

/// Private: Update a short link
#[utoipa::path(
    put,
    path = "/api/v1/admin/links/{slug}",
    tag = "admin",
    params(
        ("slug" = String, Path, description = "Short link identifier to update")
    ),
    request_body = CreateShortLinkRequest,
    responses(
        (status = 200, description = "Short link updated successfully", body = ShortLinkResponse),
        (status = 404, description = "Short link not found"),
        (status = 500, description = "Failed to update short link")
    )
)]
pub async fn put_update_short_link(
    State(state): State<ApiState>,
    Path(slug): Path<String>,
    Json(payload): Json<CreateShortLinkRequest>,
) -> ApiResult<ShortLinkResponse> {
    let update = ShortLink {
        slug: payload.slug,
        params: payload.params,
        author: payload.author,
        redirect: payload.redirect,
        created_at: chrono::Utc::now(), // Will be ignored in update
        updated_at: chrono::Utc::now(),
    };

    match state
        .short_link_service
        .update_short_link(&slug, update)
        .await
    {
        Ok(link) => ApiResult::success(link.into()),
        Err(e) => {
            if e.to_string().contains("not found") {
                ApiResult::error_with_status(
                    "NOT_FOUND",
                    format!("Short link '{}' not found", slug),
                    StatusCode::NOT_FOUND,
                )
            } else {
                ApiResult::error_with_status(
                    "UPDATE_FAILED",
                    format!("Failed to update short link: {}", e),
                    StatusCode::INTERNAL_SERVER_ERROR,
                )
            }
        }
    }
}

/// Private: Delete a short link
#[utoipa::path(
    delete,
    path = "/api/v1/admin/links/{slug}",
    tag = "admin",
    params(
        ("slug" = String, Path, description = "Short link identifier to delete")
    ),
    responses(
        (status = 200, description = "Short link deleted successfully"),
        (status = 404, description = "Short link not found"),
        (status = 500, description = "Failed to delete short link")
    )
)]
pub async fn delete_short_link(
    State(state): State<ApiState>,
    Path(slug): Path<String>,
) -> ApiResult<()> {
    match state.short_link_service.delete_short_link(&slug).await {
        Ok(_) => ApiResult::success(()),
        Err(e) => {
            if e.to_string().contains("not found") {
                ApiResult::error_with_status(
                    "NOT_FOUND",
                    format!("Short link '{}' not found", slug),
                    StatusCode::NOT_FOUND,
                )
            } else {
                ApiResult::error_with_status(
                    "DELETE_FAILED",
                    format!("Failed to delete short link: {}", e),
                    StatusCode::INTERNAL_SERVER_ERROR,
                )
            }
        }
    }
}

/// Private: Get click statistics for a short link
#[utoipa::path(
    get,
    path = "/api/v1/admin/links/{slug}/stats",
    tag = "admin",
    params(
        ("slug" = String, Path, description = "Short link identifier")
    ),
    responses(
        (status = 200, description = "Click statistics", body = ClickStatsResponse),
        (status = 500, description = "Failed to get statistics")
    )
)]
pub async fn get_click_stats(
    State(state): State<ApiState>,
    Path(slug): Path<String>,
) -> ApiResult<ClickStatsResponse> {
    let total_clicks = match state.short_link_service.get_click_count(&slug).await {
        Ok(count) => count,
        Err(e) => {
            return ApiResult::error_with_status(
                "STATS_FAILED",
                format!("Failed to get click count: {}", e),
                StatusCode::INTERNAL_SERVER_ERROR,
            );
        }
    };

    let recent_clicks = match state
        .short_link_service
        .get_click_stats(&slug, Some(10))
        .await
    {
        Ok(clicks) => clicks.into_iter().map(|c| c.to_rfc3339()).collect(),
        Err(e) => {
            return ApiResult::error_with_status(
                "STATS_FAILED",
                format!("Failed to get click stats: {}", e),
                StatusCode::INTERNAL_SERVER_ERROR,
            );
        }
    };

    ApiResult::success(ClickStatsResponse {
        slug,
        total_clicks,
        recent_clicks,
    })
}
