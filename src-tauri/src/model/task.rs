use std::ops::Deref;
use chrono::{NaiveDateTime, TimeDelta};
use surrealdb::RecordId;

#[derive(Clone, Debug)]
pub(crate) struct Task {
    id: RecordId,
    name: String,
    description: String,
    priority: Priority,
    estimated_duration: TimeDelta,
}

impl Task {
    pub fn new(
        id: RecordId,
        name: String,
        description: String,
        priority: Priority,
        estimated_duration: TimeDelta,
    ) -> Self {
        Self {
            id,
            name,
            description,
            priority,
            estimated_duration,
        }
    }

    pub fn schedule(&self, schedule_for: NaiveDateTime) -> ScheduledTask {
        ScheduledTask::new(
            &self,
            schedule_for
        )
    }

    pub fn id(&self) -> &RecordId {
        &self.id
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
}

#[derive(Copy, Clone, Debug)]
pub(crate) enum Priority {
    Low,
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

#[derive(Copy, Clone, Debug)]
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
