use std::cmp::Ordering;
use std::ops::Deref;
use chrono::{NaiveDateTime, TimeDelta};
use surrealdb::RecordId;

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub(crate) struct Task {
    name: String,
    description: String,
    priority: Priority,
    estimated_duration: TimeDelta,
    deadline: NaiveDateTime
}

impl PartialOrd for Task {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Task {
    fn cmp(&self, other: &Self) -> Ordering {
        u64::from(self.priority).cmp(&u64::from(other.priority))
    }
}

impl Task {
    pub fn new(
        name: String,
        description: String,
        priority: Priority,
        estimated_duration: TimeDelta,
        deadline: NaiveDateTime
    ) -> Self {
        Self {
            name,
            description,
            priority,
            estimated_duration,
            deadline
        }
    }

    pub fn schedule(&self, schedule_for: NaiveDateTime) -> ScheduledTask {
        ScheduledTask::new(
            self,
            schedule_for
        )
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn description(&self) -> &str {
        &self.description
    }

    pub fn priority(&self) -> &Priority {
        &self.priority
    }

    pub fn estimated_duration(&self) -> TimeDelta {
        self.estimated_duration
    }

    pub fn deadline(&self) -> NaiveDateTime {
        self.deadline
    }
}

#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq, Default)]
pub(crate) enum Priority {
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
            Priority::High => 3
        };

        priority_as_number.pow(2)
    }
}

impl From<&Priority> for u64 {
    fn from(value: &Priority) -> Self {
        u64::from(*value)
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub(crate) struct ScheduledTask<'a> {
    task: &'a Task,
    scheduled_for: NaiveDateTime
}

impl<'a> Deref for ScheduledTask<'a> {
    type Target = Task;

    fn deref(&self) -> &Self::Target {
        self.task
    }
}

impl<'a> ScheduledTask<'a> {
    pub fn new(task: &'a Task, scheduled_for: NaiveDateTime) -> Self {
        Self { task, scheduled_for }
    }

    pub fn task(&self) -> &'a Task {
        self.task
    }

    pub fn scheduled_for(&self) -> NaiveDateTime {
        self.scheduled_for
    }
}
