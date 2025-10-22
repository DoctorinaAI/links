use std::net::SocketAddr;
use std::sync::Arc;

use axum::response::{Html, IntoResponse};
use axum::routing::{delete, get, post, put};
use axum::{Json, Router};
use utoipa::OpenApi;

use crate::api::{middleware, routes_private, routes_public, state::ApiState};

use tokio::net::TcpListener;
use tracing::info;

/// OpenAPI documentation structure
#[derive(OpenApi)]
#[openapi(
    info(
        title = "Links API",
        version = "0.0.1",
        description = "A simple link shortener service with analytics",
        contact(
            name = "Mike Matiunin",
            email = "plugfox@gmail.com"
        ),
        license(
            name = "MIT",
        )
    ),
    paths(
        routes_public::get_health,
        routes_public::get_about,
        routes_public::get_resolve_short_link,
        routes_public::post_click_short_link,
        routes_public::not_found,
        routes_private::get_check,
        routes_private::post_create_short_link,
        routes_private::get_list_short_links,
        routes_private::get_short_link,
        routes_private::put_update_short_link,
        routes_private::delete_short_link,
        routes_private::get_click_stats,
    ),
    components(
        schemas(
            routes_public::AboutInfo,
            routes_public::ResolveResponse,
            routes_private::CreateShortLinkRequest,
            routes_private::ShortLinkResponse,
            routes_private::ShortLinksListResponse,
            routes_private::ClickStatsResponse,
            crate::api::response::ApiError,
        )
    ),
    tags(
        (name = "public", description = "Public endpoints for link resolution and tracking"),
        (name = "admin", description = "Admin endpoints for link management (requires authentication)")
    )
)]
struct ApiDoc;

/// Server configuration using Arc to avoid unnecessary cloning
pub struct ServerConfig {
    pub address: String,
    pub fingerprint_service: Arc<crate::services::FingerprintService>,
    pub short_link_service: Arc<crate::services::ShortLinkService>,
    pub google_client_id: Option<String>,
    pub allowed_emails: Option<String>,
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

        // Handler for OpenAPI JSON spec
        async fn openapi_spec() -> impl IntoResponse {
            Json(ApiDoc::openapi())
        }

        // Handler for Scalar UI
        async fn scalar_ui() -> impl IntoResponse {
            Html(include_str!("../../tools/scalar.html"))
        }

        // Initialize Google authentication if configured
        let google_auth = if let Some(client_id) = &self.config.google_client_id {
            if !client_id.is_empty() {
                let service = crate::services::GoogleAuthService::new(client_id.clone());
                info!("Google authentication enabled");
                Some(service)
            } else {
                tracing::warn!("CONFIG_GOOGLE_CLIENT_ID is empty - authentication disabled");
                None
            }
        } else {
            tracing::warn!("CONFIG_GOOGLE_CLIENT_ID not configured - authentication disabled");
            None
        };

        // Build private routes with conditional authentication
        let private_routes = if let Some(auth_service) = google_auth {
            // Parse allowed emails if configured
            let allowed_emails = if let Some(emails) = &self.config.allowed_emails {
                if !emails.is_empty() {
                    info!("Email filtering enabled: {}", emails);
                    emails
                        .split(',')
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty())
                        .collect::<Vec<_>>()
                } else {
                    info!("No email restrictions configured - all authenticated users allowed");
                    Vec::new()
                }
            } else {
                info!("No email restrictions configured - all authenticated users allowed");
                Vec::new()
            };

            let allowed_emails_list = middleware::AllowedEmails::new(allowed_emails);

            // Apply authentication middleware with state
            Self::private_routes().layer(axum::middleware::from_fn_with_state(
                (auth_service, allowed_emails_list),
                middleware::google_auth_with_email_check_middleware,
            ))
        } else {
            // No authentication - pass through all requests
            Self::private_routes()
        };

        // Build API v1 routes (combines public and private)
        let api_v1 = Router::new()
            // Public routes (no authentication)
            .merge(Self::public_routes())
            // Private/Admin routes (with authentication if configured)
            .nest("/admin", private_routes)
            .with_state(ApiState {
                fingerprint_service: self.config.fingerprint_service.clone(),
                short_link_service: self.config.short_link_service.clone(),
            });

        // Create the main router/app
        let app = Router::new()
            .route("/api-docs/openapi.json", get(openapi_spec))
            .route("/scalar", get(scalar_ui))
            // Mount API v1 under /api/v1 prefix
            .nest("/api/v1", api_v1)
            // Add CORS middleware - allows all origins, methods, and headers
            .layer(middleware::create_cors_layer());

        info!(%addr, "Starting api server");
        info!("API v1 available at: http://{}/api/v1", addr);
        info!("API documentation available at: http://{}/scalar", addr);
        info!(
            "OpenAPI spec available at: http://{}/api-docs/openapi.json",
            addr
        );

        // Start the server with graceful shutdown support
        // Enable ConnectInfo support to get client IP addresses
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
            // Public short link routes
            .route("/link/{slug}", get(routes_public::get_resolve_short_link))
            .route("/click/{slug}", post(routes_public::post_click_short_link))
    }

    fn private_routes() -> Router<ApiState> {
        Router::new()
            .route("/check", get(routes_private::get_check))
            // Admin short link management routes
            .route("/links", get(routes_private::get_list_short_links))
            .route("/links", post(routes_private::post_create_short_link))
            .route("/links/{slug}", get(routes_private::get_short_link))
            .route("/links/{slug}", put(routes_private::put_update_short_link))
            .route("/links/{slug}", delete(routes_private::delete_short_link))
            .route("/links/{slug}/stats", get(routes_private::get_click_stats))
    }
}
