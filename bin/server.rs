use clap::Parser;
use std::sync::Arc;
use tokio::sync::oneshot;

use links::{api, config, database, services};
use tracing::{info /* debug, trace, warn, error */};
use tracing_appender::rolling;
use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse command line arguments - this will automatically handle --help
    let config = Arc::new(config::Config::parse());
    // Initialize logging
    init_logging(
        &config.log_level,
        config.environment.as_deref().unwrap_or("production"),
    );

    info!("Starting Links Server v{}", config::VERSION);

    // Initialize database
    info!("Connecting to database: {}", config.database_connection);
    let db_type = database::DatabaseType::from_connection_string(&config.database_connection)?;
    let db = database::create_database(&config.database_connection, Some(db_type)).await?;

    // Run migrations
    db.migrate().await?;

    // Health check
    db.health_check().await?;

    info!("Spawning API task");

    // Create services wrapped in Arc for efficient sharing
    let fingerprint_service = Arc::new(services::FingerprintService::new(db.clone()));
    let short_link_service = Arc::new(services::ShortLinkService::new(db.clone()));

    // Channels for graceful shutdowns
    let (api_tx, api_rx) = oneshot::channel::<()>();

    // Spawn API service
    let api_handle: tokio::task::JoinHandle<()>;
    {
        let api_shutdown = async {
            let _ = api_rx.await;
        };
        api_handle = tokio::spawn(async move {
            let server = api::Server::new(api::ServerConfig {
                address: config.address.clone(),
                fingerprint_service,
                short_link_service,
                google_client_id: config.google_client_id.clone(),
                allowed_emails: config.allowed_emails.clone(),
                cors_origins: config.cors_origins.clone(),
            });
            server.start(api_shutdown).await;
        });
    }

    // Wait for shutdown signal (Ctrl+C)
    {
        info!("Press Ctrl+C to shut down");
        tokio::signal::ctrl_c()
            .await
            .expect("failed to listen for shutdown");
        tracing::debug!("shutdown signal received");
    }

    info!("Shutting down services");
    let _ = api_tx.send(());

    // Shutdown database
    info!("Closing database connection");
    db.close().await;

    // Wait for tasks to complete
    let _ = api_handle.await;

    Ok(())
}

/// Initialize logging with configurable levels and formats
/// This function sets up the logging system using `tracing` and `tracing_subscriber`.
fn init_logging(log_level: &str, environment: &str) {
    /* let env_filter = EnvFilter::builder().parse_lossy(log_level); */

    // 1) Filter: RUST_LOG=debug or CONFIG_LOGS=info
    // If RUST_LOG is not set, use the provided log_level from configuration
    let env_filter = EnvFilter::builder()
        .with_default_directive(
            log_level
                .parse()
                .unwrap_or_else(|_| "info".parse().unwrap()),
        )
        .from_env_lossy();

    // 2) Console layer: human-readable output to stdout
    let console_layer = fmt::layer()
        .with_target(false) // не печатать имя модуля
        .with_ansi(true)
        .with_thread_ids(true);

    // 3) File layer: JSON format to rolling file appender
    let file_appender = rolling::RollingFileAppender::builder()
        .rotation(rolling::Rotation::DAILY) // every day a new file
        .max_log_files(7) // keep logs for 7 days
        .build("logs") // directory for logs
        .expect("Failed to create file appender");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
    let file_layer = fmt::layer()
        .with_ansi(environment != "production") // ANSI in dev, plain in prod
        .with_writer(non_blocking)
        .json(); // JSON format for parsing

    // 4) Инициализация глобального подписчика
    tracing_subscriber::registry()
        .with(env_filter)
        .with(console_layer)
        .with(file_layer)
        .init();
}
