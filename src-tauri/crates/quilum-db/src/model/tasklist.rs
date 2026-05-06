use serde::{Deserialize, Serialize};
use surrealdb::types::{RecordId, SurrealValue};

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
}
