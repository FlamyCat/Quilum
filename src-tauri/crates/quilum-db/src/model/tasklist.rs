use serde::{Deserialize, Serialize};
use surrealdb::types::RecordId;

#[derive(Serialize, Deserialize)]
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
