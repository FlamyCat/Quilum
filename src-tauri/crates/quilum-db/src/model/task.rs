use chrono::{DateTime, NaiveDateTime, TimeDelta};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use surrealdb::types::{RecordId, SurrealValue};

#[derive(Clone, Debug, Hash, PartialEq, Eq, Serialize, Deserialize, SurrealValue)]
pub struct Task {
    #[serde(skip_serializing)]
    pub id: RecordId,
    pub name: String,
    pub description: String,
    pub priority: Priority,
    pub estimated_duration: i64,
    pub deadline: i64,
}

impl Task {
    pub fn id(&self) -> &RecordId {
        &self.id
    }

    pub fn estimated_duration(&self) -> TimeDelta {
        TimeDelta::seconds(self.estimated_duration)
    }

    pub fn deadline_datetime(&self) -> NaiveDateTime {
        DateTime::from_timestamp(self.deadline, 0).unwrap_or_default().naive_utc()
    }
}

impl PartialOrd for Task {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Task {
    fn cmp(&self, other: &Self) -> Ordering {
        fn to_priority_tuple(task: &Task) -> (u64, i64, &String, &String) {
            (
                u64::from(task.priority),
                task.deadline,
                &task.name,
                &task.description,
            )
        }

        to_priority_tuple(self).cmp(&to_priority_tuple(other))
    }
}

impl Task {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn description(&self) -> &str {
        &self.description
    }

    pub fn priority(&self) -> &Priority {
        &self.priority
    }

    pub fn deadline(&self) -> i64 {
        self.deadline
    }

    pub fn deadline_as_datetime(&self) -> NaiveDateTime {
        DateTime::from_timestamp(self.deadline, 0).unwrap_or_default().naive_utc()
    }

    pub fn set_name(&mut self, name: String) {
        self.name = name;
    }

    pub fn set_description(&mut self, description: String) {
        self.description = description;
    }

    pub fn set_priority(&mut self, priority: Priority) {
        self.priority = priority;
    }

    pub fn set_estimated_duration(&mut self, estimated_duration: TimeDelta) {
        self.estimated_duration = estimated_duration.num_seconds();
    }

    pub fn set_deadline(&mut self, deadline: NaiveDateTime) {
        self.deadline = deadline.and_utc().timestamp();
    }
}

#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq, Default, Serialize, Deserialize, SurrealValue)]
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
