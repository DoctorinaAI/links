use crate::database::SharedDatabase;

pub struct ShortLinkService {
    db: SharedDatabase,
}

impl ShortLinkService {
    pub fn new(db: SharedDatabase) -> Self {
        Self { db }
    }

    /// Get a reference to the database
    pub fn database(&self) -> &SharedDatabase {
        &self.db
    }
}
