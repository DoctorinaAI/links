use crate::database::SharedDatabase;
use anyhow::Result;

pub trait FingerprintOperations {
    fn create_fingerprint(&self, data: &str) -> Result<String>;
    fn verify_fingerprint(&self, fingerprint: &str) -> Result<bool>;
}

#[derive(Clone)]
pub struct FingerprintService {
    db: SharedDatabase,
}

impl FingerprintService {
    pub fn new(db: SharedDatabase) -> Self {
        Self { db }
    }

    /// Get a reference to the database
    pub fn database(&self) -> &SharedDatabase {
        &self.db
    }
}
