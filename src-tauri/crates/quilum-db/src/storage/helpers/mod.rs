use crate::Storage;
use surrealdb::types::RecordId;

impl Storage {
    /// Helper to convert RecordId to string for SQL queries
    pub(super) fn record_id_to_string(id: &RecordId) -> String {
        match &id.key {
            surrealdb::types::RecordIdKey::String(s) => format!("{}:{}", id.table, s),
            _ => format!("{}:unknown", id.table),
        }
    }

    /// Helper to extract key from RecordId for SurrealDB operations
    pub(super) fn record_id_key(id: &RecordId) -> String {
        match &id.key {
            surrealdb::types::RecordIdKey::String(s) => s.as_str().to_string(),
            _ => "unknown".to_string(),
        }
    }
}
