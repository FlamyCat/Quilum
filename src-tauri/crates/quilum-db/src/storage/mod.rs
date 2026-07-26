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

use serde::{Deserialize, Serialize};
use surrealdb::{engine::local::Db, types::SurrealValue, Surreal};

use crate::{slot::Slot, task::Task, tasklist::TaskList};

/// Struct for returning slots with their scheduled tasks
#[derive(Clone, Debug, Serialize, Deserialize, SurrealValue)]
pub struct SlotWithTasks {
    pub slot: Slot,
    pub tasks: Vec<(Task, i64)>,
}

/// Struct for returning task lists with their tasks
#[derive(Clone, Debug, Serialize, Deserialize, SurrealValue)]
pub struct TaskListWithTasks {
    pub list: TaskList,
    pub tasks: Vec<Task>,
}

/// Struct that exposes storage methods.
#[derive(Clone)]
pub struct Storage {
    db: Surreal<Db>,
}
