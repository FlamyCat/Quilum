use serde::{Deserialize, Serialize};
use surrealdb::types::SurrealValue;

#[derive(Serialize, Deserialize, SurrealValue)]
pub(crate) struct TaskList {
    pub(crate) title: String,
}

impl TaskList {
    pub(crate) fn new(title: String) -> Self {
        Self {
            title,
        }
    }
}
