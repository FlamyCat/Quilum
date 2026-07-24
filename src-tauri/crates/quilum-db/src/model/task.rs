use std::{cmp::Ordering, fmt::Debug, marker::PhantomData, time::Duration};

use chrono::{DateTime, NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use surrealdb::types::{RecordId, RecordIdKey, SurrealValue};

#[derive(Clone, Debug, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct Task<S> {
    data: TaskData,
    _schedule_info: PhantomData<S>,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, Serialize, Deserialize, SurrealValue)]
pub struct TaskData {
    pub id: RecordId,
    pub title: String,
    pub description: String,
    pub priority: Priority,
    pub estimated_duration: Duration,
    pub deadline: DateTime<Utc>,
    pub completed: bool,
    pub scheduled_for: Option<DateTime<Utc>>,
}

pub const TASKS_TABLE: &str = "tasks";

impl TaskData {
    pub fn new(
        title: String,
        description: String,
        priority: Priority,
        estimated_duration: Duration,
        deadline: DateTime<Utc>,
        completed: bool,
        scheduled_for: Option<DateTime<Utc>>,
    ) -> Self {
        Self {
            id: RecordId::new(TASKS_TABLE, RecordIdKey::ulid()),
            title,
            description,
            priority,
            estimated_duration,
            deadline,
            completed,
            scheduled_for,
        }
    }
}

impl<S: Ord> PartialOrd for Task<S> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<S: Ord> Ord for Task<S> {
    fn cmp(&self, other: &Self) -> Ordering {
        fn to_priority_tuple<S>(task: &Task<S>) -> (u64, DateTime<Utc>, &String, &String) {
            (
                u64::from(task.data.priority),
                task.data.deadline,
                &task.data.title,
                &task.data.description,
            )
        }

        to_priority_tuple(self).cmp(&to_priority_tuple(other))
    }
}

impl<S> Task<S> {
    pub fn name(&self) -> &str {
        &self.data.title
    }

    pub fn description(&self) -> &str {
        &self.data.description
    }

    pub fn priority(&self) -> &Priority {
        &self.data.priority
    }

    pub fn deadline(&self) -> DateTime<Utc> {
        self.data.deadline
    }

    pub fn deadline_as_datetime(&self) -> NaiveDateTime {
        self.data.deadline.naive_utc()
    }

    pub fn id(&self) -> &RecordId {
        &self.data.id
    }

    pub fn estimated_duration(&self) -> Duration {
        self.data.estimated_duration
    }

    pub fn deadline_datetime(&self) -> DateTime<Utc> {
        self.data.deadline
    }

    pub fn set_name(&mut self, name: String) {
        self.data.title = name;
    }

    pub fn set_description(&mut self, description: String) {
        self.data.description = description;
    }

    pub fn set_priority(&mut self, priority: Priority) {
        self.data.priority = priority;
    }

    pub fn set_estimated_duration(&mut self, estimated_duration: Duration) {
        self.data.estimated_duration = estimated_duration;
    }

    pub fn set_deadline(&mut self, deadline: DateTime<Utc>) {
        self.data.deadline = deadline;
    }

    pub fn completed(&self) -> bool {
        self.data.completed
    }

    pub fn set_completed(&mut self, completed: bool) {
        self.data.completed = completed;
    }

    /// Transforms the task into scheduled one, with `scheduled_for` set to `timestamp`.
    pub fn schedule_for(self, timestamp: DateTime<Utc>) -> Task<Scheduled> {
        Task::<Scheduled> {
            data: TaskData {
                scheduled_for: Some(timestamp),
                ..self.data
            },
            _schedule_info: PhantomData,
        }
    }
}

impl Task<Scheduled> {
    pub fn scheduled_for(&self) -> Option<DateTime<Utc>> {
        self.data.scheduled_for
    }
}

#[derive(
    Copy, Clone, Debug, Hash, Eq, PartialEq, Default, Serialize, Deserialize, SurrealValue,
)]
pub enum Priority {
    Low,
    #[default]
    Medium,
    High,
}

impl From<Priority> for u64 {
    fn from(value: Priority) -> Self {
        let priority_as_number: u64 = match value {
            Priority::Low => 1,
            Priority::Medium => 2,
            Priority::High => 3,
        };

        priority_as_number.pow(2)
    }
}

impl From<&Priority> for u64 {
    fn from(value: &Priority) -> Self {
        u64::from(*value)
    }
}
