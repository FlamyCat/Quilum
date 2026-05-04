use serde::{Deserialize, Serialize};
use crate::app_identifier::AppIdentifier;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockedApp {
    pub id: surrealdb::types::RecordId,
    pub identifier: AppIdentifier,
    pub display_name: String,
}
