use serde::{Deserialize, Serialize};
use surrealdb::types::{RecordId, SurrealValue};

#[derive(Serialize, Deserialize, SurrealValue)]
pub struct TaskList {
    #[serde(skip_serializing)]
    pub id: RecordId,
    pub title: String,
}

impl TaskList {
    pub fn id(&self) -> &RecordId {
        &self.id
    }
}
