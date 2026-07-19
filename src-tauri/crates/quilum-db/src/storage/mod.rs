mod applock;
mod events;
mod focus;
mod helpers;
mod init;
pub mod model;
mod relations;
mod scheduler;
mod slots;
mod tasklists;
mod tasks;
mod timetable;

use serde::{Deserialize, Serialize};
use surrealdb::{engine::local::Db, types::SurrealValue, Surreal};

use self::model::{slot::Slot, task::Task, tasklist::TaskList};
use crate::storage::model::task::{Scheduled, Unscheduled};

/// Struct for returning slots with their scheduled tasks
#[derive(Clone, Debug, Serialize, Deserialize, SurrealValue)]
pub struct SlotWithTasks {
    pub slot: Slot,
    pub tasks: Vec<Task<Scheduled>>,
}

/// Struct for returning task lists with their tasks
#[derive(Clone, Debug, Serialize, Deserialize, SurrealValue)]
pub struct TaskListWithTasks {
    pub list: TaskList,
    pub tasks: Vec<Task<Unscheduled>>,
}

/// Storage struct that holds a handle to a SurrealDB instance
/// and exposes CRUD methods for events, tasks, and app blocking.
#[derive(Clone)]
pub struct Storage {
    db: Surreal<Db>,
}
