use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use surrealdb::types::{RecordId, SurrealValue};

pub const BLOCKED_APPS_TABLE: &str = "blocked_apps";

#[derive(Debug, Clone, Serialize, Deserialize, SurrealValue)]
pub struct BlockedApp {
    pub id: RecordId,
    pub identifier: String,
    pub display_name: String,
}

impl BlockedApp {
    pub fn app_identifier(&self) -> PathBuf {
        PathBuf::from(&self.identifier)
    }

    pub fn new(identifier: String, display_name: String) -> Self {
        Self {
            id: RecordId::new(BLOCKED_APPS_TABLE, identifier.clone()),
            identifier,
            display_name,
        }
    }
}
