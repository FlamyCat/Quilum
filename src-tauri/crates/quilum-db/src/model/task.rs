use std::{cmp::Ordering, fmt::Debug, time::Duration};

use chrono::{DateTime, TimeDelta, Utc};
use serde::{Deserialize, Serialize};
use surrealdb::types::{RecordId, RecordIdKey, SurrealValue};

#[derive(Clone, Debug, Hash, PartialEq, Eq, Serialize, Deserialize, SurrealValue)]
pub struct Task {
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

impl Task {
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

impl PartialOrd for Task {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Task {
    fn cmp(&self, other: &Self) -> Ordering {
        fn to_priority_tuple(task: &Task) -> (u64, DateTime<Utc>, &String, &String) {
            (
                u64::from(task.priority),
                task.deadline,
                &task.title,
                &task.description,
            )
        }

        to_priority_tuple(self).cmp(&to_priority_tuple(other))
    }
}

impl Task {
    pub fn name(&self) -> &str {
        &self.title
    }

    pub fn description(&self) -> &str {
        &self.description
    }

    pub fn priority(&self) -> &Priority {
        &self.priority
    }

    pub fn deadline(&self) -> DateTime<Utc> {
        self.deadline
    }

    pub fn id(&self) -> &RecordId {
        &self.id
    }

    pub fn estimated_duration(&self) -> Duration {
        self.estimated_duration
    }

    pub fn estimated_duration_timedelta(&self) -> TimeDelta {
        TimeDelta::from_std(self.estimated_duration()).unwrap()
    }

    pub fn scheduled_for(&self) -> Option<DateTime<Utc>> {
        self.scheduled_for
    }

    pub fn set_name(&mut self, name: String) {
        self.title = name;
    }

    pub fn set_description(&mut self, description: String) {
        self.description = description;
    }

    pub fn set_priority(&mut self, priority: Priority) {
        self.priority = priority;
    }

    pub fn set_estimated_duration(&mut self, estimated_duration: Duration) {
        self.estimated_duration = estimated_duration;
    }

    pub fn set_deadline(&mut self, deadline: DateTime<Utc>) {
        self.deadline = deadline;
    }

    pub fn completed(&self) -> bool {
        self.completed
    }

    pub fn set_completed(&mut self, completed: bool) {
        self.completed = completed;
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
