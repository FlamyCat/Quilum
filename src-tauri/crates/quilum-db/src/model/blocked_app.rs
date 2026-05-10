use crate::app_identifier::AppIdentifier;
use serde::{Deserialize, Serialize};
use surrealdb::types::SurrealValue;

#[derive(Debug, Clone, Serialize, Deserialize, SurrealValue)]
pub struct BlockedApp {
    pub id: surrealdb::types::RecordId,
    pub identifier: String,
    pub display_name: String,
}

impl BlockedApp {
    pub fn app_identifier(&self) -> AppIdentifier {
        let s = &self.identifier;
        if s.contains(std::path::MAIN_SEPARATOR) || (cfg!(windows) && s.contains('\\')) {
            AppIdentifier::Path(std::path::PathBuf::from(s))
        } else if s.contains('/') {
            AppIdentifier::Path(std::path::PathBuf::from(s))
        } else {
            AppIdentifier::BundleId(s.clone())
        }
    }
}
