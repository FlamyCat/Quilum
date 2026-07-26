use serde::{Deserialize, Serialize};
use surrealdb::types::{RecordId, RecordIdKey, SurrealValue};

pub const TASKLISTS_TABLE: &str = "tasklists";

#[derive(Clone, Debug, Serialize, Deserialize, SurrealValue)]
pub struct TaskList {
    pub id: RecordId,
    pub title: String,
}

impl TaskList {
    pub fn id(&self) -> &RecordId {
        &self.id
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn new(title: String) -> Self {
        Self {
            id: RecordId::new(TASKLISTS_TABLE, RecordIdKey::ulid()),
            title,
        }
    }
}
