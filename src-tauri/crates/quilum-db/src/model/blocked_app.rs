use std::{path, path::PathBuf};

use serde::{Deserialize, Serialize};
use surrealdb::types::{RecordId, SurrealValue};

use crate::model::app_identifier::AppIdentifier;

pub const BLOCKED_APPS_TABLE: &str = "blocked_apps";

#[derive(Debug, Clone, Serialize, Deserialize, SurrealValue)]
pub struct BlockedApp {
    pub id: RecordId,
    pub identifier: String,
    pub display_name: String,
}

impl BlockedApp {
    pub fn app_identifier(&self) -> AppIdentifier {
        if self.identifier.contains(path::MAIN_SEPARATOR) {
            AppIdentifier::Path(PathBuf::from(self.identifier.clone()))
        } else {
            AppIdentifier::BundleId(self.identifier.clone())
        }
    }

    pub fn new(identifier: String, display_name: String) -> Self {
        Self {
            id: RecordId::new(BLOCKED_APPS_TABLE, identifier.clone()),
            identifier,
            display_name,
        }
    }
}
