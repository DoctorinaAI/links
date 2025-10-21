mod db;

pub use db::{Database, DatabaseType, SharedDatabase, create_database};

#[cfg(test)]
mod tests;
