use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShortLink {
    pub slug: String,
    pub params: HashMap<String, String>,
    pub author: String,
    pub updated_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}
