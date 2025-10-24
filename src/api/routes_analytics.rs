use crate::api::{response::ApiResult, state::ApiState, timing::RequestMetrics};
use crate::models::Platform;
use axum::{Extension, Json, extract::State, http::StatusCode};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use utoipa::ToSchema;

/// Client IP address extracted from connection or proxy headers
/// This is injected by the IP extraction middleware
#[derive(Clone, Debug)]
pub struct ClientIp(pub String);

#[derive(Deserialize, ToSchema)]
pub struct StoreAnalyticsRequest {
    /// Short link slug that the user clicked
    #[schema(example = "some-link")]
    pub slug: String,

    /// Platform information from the client
    pub platform: Platform,
}

#[derive(Serialize, ToSchema)]
pub struct StoreAnalyticsResponse {
    /// Success status
    #[schema(example = true)]
    pub success: bool,

    /// Fingerprint hash (for debugging)
    #[schema(example = "a1b2c3d4e5f6...")]
    pub fingerprint: String,

    /// Expiration timestamp (Unix timestamp)
    #[schema(example = 1729890000)]
    pub expires_at: i64,
}

/// Public: Store analytics fingerprint
/// Records user's click with platform information for later attribution
#[utoipa::path(
    put,
    path = "/api/v1/analytics",
    tag = "public",
    request_body = StoreAnalyticsRequest,
    responses(
        (status = 200, description = "Fingerprint stored successfully", body = StoreAnalyticsResponse),
        (status = 400, description = "Invalid platform data"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn put_store_analytics(
    State(state): State<ApiState>,
    Extension(metrics): Extension<RequestMetrics>,
    Extension(client_ip): Extension<ClientIp>,
    Json(payload): Json<StoreAnalyticsRequest>,
) -> ApiResult<StoreAnalyticsResponse> {
    // Validate platform data
    let mut platform = payload.platform;
    if let Err(e) = platform.validate() {
        return ApiResult::error_with_status(
            "INVALID_PLATFORM",
            format!("Invalid platform data: {}", e),
            StatusCode::BAD_REQUEST,
        );
    }

    // Normalize platform data
    platform.normalize();

    // Verify the slug exists
    let link = metrics
        .measure_async("resolve_link", || {
            state.short_link_service.resolve_short_link(&payload.slug)
        })
        .await;

    let link = match link {
        Ok(Some(link)) => link,
        Ok(None) => {
            return ApiResult::error_with_status(
                "SLUG_NOT_FOUND",
                format!("Short link '{}' not found", payload.slug),
                StatusCode::NOT_FOUND,
            );
        }
        Err(e) => {
            return ApiResult::error_with_status(
                "DATABASE_ERROR",
                format!("Failed to verify slug: {}", e),
                StatusCode::INTERNAL_SERVER_ERROR,
            );
        }
    };

    // Store fingerprint with 24-hour TTL
    let result = metrics
        .measure_async("store_fingerprint", || {
            state.fingerprint_service.store_fingerprint(
                &payload.slug,
                &link.params,
                &client_ip.0,
                &platform,
                Some(24), // 24 hours TTL
            )
        })
        .await;

    match result {
        Ok(fingerprint) => ApiResult::success(StoreAnalyticsResponse {
            success: true,
            fingerprint: fingerprint.fingerprint,
            expires_at: fingerprint.expires_at.timestamp(),
        }),
        Err(e) => ApiResult::error_with_status(
            "STORE_FAILED",
            format!("Failed to store fingerprint: {}", e),
            StatusCode::INTERNAL_SERVER_ERROR,
        ),
    }
}

#[derive(Deserialize, ToSchema)]
pub struct RetrieveAnalyticsRequest {
    /// Platform information from the client
    pub platform: Platform,
}

#[derive(Serialize, ToSchema)]
pub struct RetrieveAnalyticsResponse {
    /// Short link slug
    #[schema(example = "some-link")]
    pub slug: String,

    /// Parameters associated with this short link
    #[schema(example = json!({"url": "https://example.com", "campaign": "spring2024"}))]
    pub params: HashMap<String, String>,

    /// Fingerprint hash (for debugging)
    #[schema(example = "a1b2c3d4e5f6...")]
    pub fingerprint: String,
}

/// Public: Retrieve analytics attribution data
/// Returns the short link slug and params based on user's IP and platform
#[utoipa::path(
    post,
    path = "/api/v1/analytics",
    tag = "public",
    request_body = RetrieveAnalyticsRequest,
    responses(
        (status = 200, description = "Attribution data found", body = RetrieveAnalyticsResponse),
        (status = 404, description = "No attribution data found for this device"),
        (status = 400, description = "Invalid platform data"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn post_retrieve_analytics(
    State(state): State<ApiState>,
    Extension(metrics): Extension<RequestMetrics>,
    Extension(client_ip): Extension<ClientIp>,
    Json(payload): Json<RetrieveAnalyticsRequest>,
) -> ApiResult<RetrieveAnalyticsResponse> {
    // Validate platform data
    let mut platform = payload.platform;
    if let Err(e) = platform.validate() {
        return ApiResult::error_with_status(
            "INVALID_PLATFORM",
            format!("Invalid platform data: {}", e),
            StatusCode::BAD_REQUEST,
        );
    }

    // Normalize platform data
    platform.normalize();

    // Retrieve fingerprint
    let result = metrics
        .measure_async("get_fingerprint", || {
            state
                .fingerprint_service
                .get_fingerprint(&client_ip.0, &platform)
        })
        .await;

    match result {
        Ok(Some(fingerprint)) => ApiResult::success(RetrieveAnalyticsResponse {
            slug: fingerprint.slug,
            params: fingerprint.params,
            fingerprint: fingerprint.fingerprint,
        }),
        Ok(None) => ApiResult::error_with_status(
            "NOT_FOUND",
            "No attribution data found for this device",
            StatusCode::NOT_FOUND,
        ),
        Err(e) => ApiResult::error_with_status(
            "RETRIEVE_FAILED",
            format!("Failed to retrieve fingerprint: {}", e),
            StatusCode::INTERNAL_SERVER_ERROR,
        ),
    }
}
