use crate::api::response::ApiResult;

/// Private: check autentication handler
pub async fn get_check() -> ApiResult<String> {
    let version = env!("CARGO_PKG_VERSION");
    ApiResult::success(format!("Current app version: {}", version))
}
