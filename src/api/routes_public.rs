use crate::api::{response::ApiResult, state::ApiState};
use crate::config::{AUTHOR, DESCRIPTION, HOMEPAGE, LICENSE, NAME, REPOSITORY, VERSION};
use axum::{extract::State, http::StatusCode};
use serde::Serialize;

/// Public: Health-check handler
/// Checks the health of the application and the database connection
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

#[derive(Serialize)]
pub struct AboutInfo {
    name: &'static str,
    version: &'static str,
    author: &'static str,
    description: &'static str,
    repository: &'static str,
    license: &'static str,
    homepage: &'static str,
}

/// Public: About handler
/// Returns information about the application, such as version
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

/// Public: Fallback handler for 404 Not Found
pub async fn not_found() -> ApiResult<()> {
    ApiResult::error_with_status(
        "NOT_FOUND",
        "The requested resource was not found",
        StatusCode::NOT_FOUND,
    )
}
