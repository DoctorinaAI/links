use std::net::SocketAddr;
use std::sync::Arc;

use axum::response::{Html, IntoResponse};
use axum::routing::{delete, get, post, put};
use axum::{Json, Router};
use utoipa::OpenApi;

use crate::api::{jwt_middleware, middleware, routes_private, routes_public, state::ApiState};

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
        routes_public::post_auth_google,
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
            routes_public::GoogleAuthRequest,
            routes_public::GoogleAuthResponse,
            routes_public::UserInfo,
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
    pub google_client_id: String,
    pub jwt_secret: String,
    pub allowed_emails: Vec<String>,
    pub cors_origins: Vec<String>,
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

        // Initialize Google authentication
        let auth_service = Arc::new(crate::services::GoogleAuthService::new(
            self.config.google_client_id.clone(),
        ));
        info!("Google authentication enabled with built-in caching (keys: 1h, tokens: 5min)");

        // Initialize JWT service for internal tokens
        let jwt_service = Arc::new(crate::services::JwtService::new(
            self.config.jwt_secret.clone(),
        ));
        info!("JWT service initialized (non-expiring tokens)");

        // Configure email filtering
        if self.config.allowed_emails.is_empty() {
            info!("No email restrictions - all authenticated users allowed");
        } else {
            info!(
                "Email filtering enabled: {}",
                self.config.allowed_emails.join(", ")
            );
        }

        let allowed_emails_list =
            jwt_middleware::AllowedEmails::new(self.config.allowed_emails.clone());

        // Configure CORS
        if self.config.cors_origins.is_empty() {
            info!("CORS: Permissive mode - all origins allowed");
        } else {
            info!(
                "CORS: Restricted mode - allowed origins: {}",
                self.config.cors_origins.join(", ")
            );
        }

        // Build private routes with authentication middleware
        let private_routes = Self::private_routes().layer(axum::middleware::from_fn_with_state(
            (jwt_service.clone(), allowed_emails_list),
            jwt_middleware::internal_jwt_with_email_check_middleware,
        ));

        // Build API v1 routes (combines public and private)
        let api_v1 = Router::new()
            // Public routes (no authentication)
            .merge(Self::public_routes())
            // Auth route with special state (needs google_auth and jwt_service)
            .route(
                "/auth/google",
                post(routes_public::post_auth_google)
                    .with_state((auth_service.clone(), jwt_service.clone())),
            )
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
            // Add CORS middleware with configured origins
            .layer(middleware::create_cors_layer(
                self.config.cors_origins.clone(),
            ));

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
