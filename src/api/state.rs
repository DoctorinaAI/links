use crate::{services::FingerprintService, services::ShortLinkService};
use std::sync::Arc;

/// API state that is shared across all request handlers
/// Uses Arc to avoid cloning the services on every request
#[derive(Clone)]
pub struct ApiState {
    pub fingerprint_service: Arc<FingerprintService>,
    pub short_link_service: Arc<ShortLinkService>,
}
