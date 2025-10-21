use crate::database::SharedDatabase;

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
