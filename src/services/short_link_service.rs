use crate::{database::SharedDatabase, models::ShortLink};
use anyhow::Result;

pub trait ShortLinkOperations {
    /// Create a short link for the given URL
    fn create_short_link(&self, request: ShortLink) -> Result<ShortLink>;

    /// Resolve a short link by its slug
    fn resolve_short_link(&self, slug: &str) -> Result<Option<ShortLink>>;

    /// Delete a short link by its slug
    fn delete_short_link(&self, slug: &str) -> Result<()>;

    /// List all short links
    fn list_short_links(&self) -> Result<Vec<ShortLink>>;

    /// Update a short link by its slug
    fn update_short_link(&self, slug: &str, params: ShortLink) -> Result<ShortLink>;
}

#[derive(Clone)]
pub struct ShortLinkService {
    db: SharedDatabase,
}

impl ShortLinkService for ShortLinkService {
    pub fn new(db: SharedDatabase) -> Self {
        Self { db }
    }

    /// Get a reference to the database
    pub fn database(&self) -> &SharedDatabase {
        &self.db
    }
}
