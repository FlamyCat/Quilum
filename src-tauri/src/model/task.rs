use chrono::{NaiveDateTime, TimeDelta};
use surrealdb::RecordId;

pub(crate) struct Task {
    id: RecordId,
    name: String,
    description: String,
    priority: Priority,
    estimated_duration: TimeDelta,
    scheduled_for: Option<NaiveDateTime>,
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
            scheduled_for: None,
        }
    }

    pub fn schedule(&mut self, datetime: NaiveDateTime) {
        self.scheduled_for = Some(datetime);
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

    pub fn scheduled_for(&self) -> Option<NaiveDateTime> {
        self.scheduled_for
    }
}

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
