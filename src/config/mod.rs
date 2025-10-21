use clap::Parser;

/// Name of the application
pub const NAME: &str = env!("CARGO_PKG_NAME");

/// Version of the application
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Author of the application
pub const AUTHOR: &str = env!("CARGO_PKG_AUTHORS");

/// Description of the application
pub const DESCRIPTION: &str = env!("CARGO_PKG_DESCRIPTION");

/// License of the application
pub const LICENSE: &str = env!("CARGO_PKG_LICENSE");

/// Homepage URL of the application
pub const HOMEPAGE: &str = env!("CARGO_PKG_HOMEPAGE");

/// Repository URL of the application
pub const REPOSITORY: &str = env!("CARGO_PKG_REPOSITORY");

/// Is the application running in development mode?
#[cfg(debug_assertions)]
pub const IS_DEV: bool = true;
#[cfg(not(debug_assertions))]
pub const IS_DEV: bool = false;

/// Application settings loaded from environment or CLI args
#[derive(Parser, Debug, Clone)]
#[command(
    name = "links",
    version,
    author = "mikhail.matiunin@doctorina.com",
    about = "Links shorter service for generating and managing short URLs.",
    long_about = "A simple and efficient links shorter service built with Rust. \
                  It allows users to create, manage, and track short URLs with ease. \
                  Perfect for sharing links on"
)]
pub struct Config {
    /// Environment to run the server in (CLI > ENV > default)
    #[arg(
        short = 'e',
        long = "env",
        env = "CONFIG_ENVIRONMENT",
        default_value = "development",
        aliases = ["mode", "runmode", "runtime", "env"],
        help = "Environment to run the server in (development, production, etc.)"
    )]
    pub environment: Option<String>,

    /// Level of logging (CLI > ENV > default)
    /// This can be set to trace, debug, info, warn, or error
    #[arg(
        short = 'l',
        long = "logs",
        env = "CONFIG_LOGS",
        default_value = "info",
        aliases = ["log", "level", "verbose", "verbosity", "loglevel"],
        help = "Logging level (trace, debug, info, warn, error)"
    )]
    pub log_level: String,

    /// Address to bind the server to (CLI > ENV > default)
    /// e.g. 127.0.0.1:8000
    #[arg(
        short = 'a',
        long,
        env = "CONFIG_ADDRESS",
        default_value = "0.0.0.0:8000",
        aliases = ["host", "addr", "api", "connection"],
        help = "Address to bind the server to"
    )]
    pub address: String,

    /// SQLite connection URL (CLI > ENV > default)
    /// e.g. sqlite::memory: or sqlite://data/links.db?mode=rwc
    #[arg(
        short = 'd',
        long,
        env = "CONFIG_DATABASE_CONNECTION",
        aliases = ["db", "sqlite", "sqlite3", "sql", "storage"],
        default_value = "sqlite://data/links.db?mode=rwc",
        help = "Database connection URL (e.g. sqlite://links.db or postgres://user:pass@host/db)"
    )]
    pub database_connection: String,
    /*
    /// Secret admin API key for API authentication (CLI > ENV > default)
    //#[arg(
    //    short = 's',
    //    long,
    //    env = "CONFIG_SECRET",
    //    aliases = ["admin", "apikey", "key", "auth"],
    //    help = "Secret admin API key for API authentication"
    //)]
    //pub secret: String,
    */

    /*
    /// Chats to monitor (CLI > ENV > default)
    /// e.g. 123456789, 987654321
    #[arg(
        short = 'c',
        long = "chats",
        env = "CONFIG_CHATS",
        aliases = ["chat", "groups", "channels", "monitored", "watched", "observed"],
        help = "Chats to monitor (e.g. 123456789, 987654321)",
    )]
    pub chats: Vec<String>,
    */
}
