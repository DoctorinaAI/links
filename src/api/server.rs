use std::net::SocketAddr;

//use crate::services::FingerprintService;
//use crate::services::ShortLinkService;
use axum::{Router, routing::get};

use crate::api::{routes_private, routes_public, state::ApiState};

use tokio::net::TcpListener;
use tracing::info;

pub struct ServerConfig {
    pub address: String,
    pub fingerprint_service: crate::services::FingerprintService,
    pub short_link_service: crate::services::ShortLinkService,
}

pub struct Server {
    config: ServerConfig,
}

impl Server {
    pub fn new(config: ServerConfig) -> Self {
        Self { config }
    }

    /// Run the HTTP API server with graceful shutdown
    pub async fn start(
        &self,
        shutdown_signal: impl std::future::Future<Output = ()> + Send + 'static,
    ) {
        // Try to parse the address from the configuration
        let addr: SocketAddr = self.config.address.parse().expect("invalid address format");

        // Listen on the specified address and handle incoming connections
        let listener = TcpListener::bind(&addr)
            .await
            .expect("failed to bind to address");

        // Public routes without authentication
        let api_routes_public = Self::public_routes();

        // Private routes that require authentication
        let api_routes_protected = Self::private_routes();
        // TODO: Add authentication middleware
        // .layer(from_fn_with_state(ApiState, middleware::auth_middleware));

        // Create the router/app with state
        let app = Router::new()
            .nest("/api", api_routes_public)
            .nest("/api/admin", api_routes_protected)
            // TODO: Add fallback route
            // .fallback(routes_public::public_not_found)
            .with_state(ApiState {});

        info!(%addr, "Starting api server");

        // Start the server with graceful shutdown support
        // Включаем поддержку ConnectInfo для получения IP адресов клиентов
        axum::serve(listener, app.into_make_service())
            .with_graceful_shutdown(shutdown_signal)
            .await
            .expect("api server crashed");

        // Right after the server stops, we log that the server has stopped
        info!("api server stopped");
    }

    fn public_routes() -> Router<ApiState> {
        Router::new()
            .route("/health", get(routes_public::get_health))
            .route("/healthz", get(routes_public::get_health))
            .route("/status", get(routes_public::get_health))
            .route("/about", get(routes_public::get_about))
            .route("/version", get(routes_public::get_about))
            .route("/404", get(routes_public::not_found))
    }

    fn private_routes() -> Router<ApiState> {
        Router::new().route("/check", get(routes_private::get_check))
    }
}
