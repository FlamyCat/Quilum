use crate::model::app_identifier::AppIdentifier;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppInfo {
    pub identifier: AppIdentifier,
    pub display_name: String,
}

impl AppInfo {
    pub fn new(identifier: AppIdentifier, display_name: String) -> Self {
        Self {
            identifier,
            display_name,
        }
    }
}
