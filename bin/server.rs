use clap::Parser;
use std::sync::Arc;
use tokio::sync::oneshot;

use links::{api, config};
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

    // Channels for graceful shutdowns
    let (api_tx, api_rx) = oneshot::channel::<()>();

    info!("Starting server v{}", config::VERSION);

    // Spawn API service
    let api_handle: tokio::task::JoinHandle<()>;
    {
        let api_shutdown = async {
            let _ = api_rx.await;
        };
        api_handle = tokio::spawn(async move {
            let server = api::Server::new(config.address.clone());
            server.start(api_shutdown).await;
        });
    }

    // Wait for shutdown signal (Ctrl+C)
    {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to listen for shutdown");
        tracing::info!("shutdown signal received");
    }

    // Wait for tasks to complete
    let _ = api_handle.await;

    Ok(())
}

/// Initialize logging with configurable levels and formats
/// This function sets up the logging system using `tracing` and `tracing_subscriber`.
fn init_logging(log_level: &str, environment: &str) {
    // 1) Фильтр: RUST_LOG=debug или APP_LOG=info
    /* let env_filter = EnvFilter::try_from_env("APP_LOG")
    .or_else(|_| EnvFilter::try_from_default_env())
    .unwrap_or_else(|_| EnvFilter::new("info")); */

    // 1) Фильтр: RUST_LOG=debug или APP_LOG=info
    let env_filter = EnvFilter::builder()
        .with_default_directive(
            log_level
                .parse()
                .unwrap_or_else(|_| "info".parse().unwrap()),
        )
        .from_env_lossy();

    // 2) Консольный формат
    let console_layer = fmt::layer()
        .with_target(false) // не печатать имя модуля
        .with_ansi(true)
        .with_thread_ids(true);

    // 3) Файловый аппендер: новый файл каждый день
    let file_appender = rolling::RollingFileAppender::builder()
        .rotation(rolling::Rotation::DAILY) // ежедневная ротация
        .max_log_files(7) // хранить не более 7 файлов
        .build("logs") // каталог для логов
        .expect("Failed to create file appender");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
    let file_layer = fmt::layer()
        .with_ansi(environment != "production") // ANSI в dev, plain в prod
        .with_writer(non_blocking)
        .json(); // JSON-формат для парсинга

    // 4) Инициализация глобального подписчика
    tracing_subscriber::registry()
        .with(env_filter)
        .with(console_layer)
        .with(file_layer)
        .init();
}
