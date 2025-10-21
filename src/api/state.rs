use crate::{services::FingerprintService, services::ShortLinkService};

#[derive(Clone)]
pub struct ApiState {
    pub fingerprint_service: FingerprintService,
    pub short_link_service: ShortLinkService,
}
