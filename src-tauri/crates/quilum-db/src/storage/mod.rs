mod applock;
mod events;
mod focus;
mod helpers;
mod init;
mod relations;
mod scheduler;
mod slots;
mod tasklists;
mod tasks;
mod timetable;

use crate::{slot::Slot, task::Task, tasklist::TaskList};
use serde::{Deserialize, Serialize};
use surrealdb::{Surreal, engine::local::Db};

/// Struct for returning slots with their scheduled tasks
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SlotWithTasks {
    pub slot: Slot,
    pub tasks: Vec<(Task, i64)>,
}

/// Struct for returning task lists with their tasks
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TaskListWithTasks {
    pub list: TaskList,
    pub tasks: Vec<Task>,
}

/// Storage struct that holds a handle to a SurrealDB instance
/// and exposes CRUD methods for events, tasks, and app blocking.
#[derive(Clone)]
pub struct Storage {
    db: Surreal<Db>,
}
