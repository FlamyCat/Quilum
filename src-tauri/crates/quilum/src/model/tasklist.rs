use serde::{Deserialize, Serialize};
use surrealdb::types::{RecordId, SurrealValue};

#[derive(Serialize, Deserialize, SurrealValue)]
pub(crate) struct TaskList {
    #[serde(skip_serializing)]
    pub(crate) id: RecordId,
    pub(crate) title: String,
}

impl TaskList {
    pub fn id(&self) -> &RecordId {
        &self.id
    }
}
