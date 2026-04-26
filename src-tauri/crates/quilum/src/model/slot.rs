use super::task::Task;
use chrono::{NaiveDateTime, TimeDelta};
use surrealdb::types::RecordId;

#[derive(Clone, Debug)]
pub(crate) struct Slot {
    starts_at: NaiveDateTime,
    ends_at: NaiveDateTime,
    tasks: Vec<Task>,
}

impl Slot {
    pub fn new(starts_at: NaiveDateTime, ends_at: NaiveDateTime) -> Self {
        Self {
            starts_at,
            ends_at,
            tasks: Vec::new(),
        }
    }

    pub fn starts_at(&self) -> NaiveDateTime {
        self.starts_at
    }

    pub fn ends_at(&self) -> NaiveDateTime {
        self.ends_at
    }

    pub fn duration(&self) -> TimeDelta {
        self.ends_at - self.starts_at
    }
}

#[derive(Clone, Debug)]
struct SlotRecord {
    id: RecordId,
    starts_at: NaiveDateTime,
    ends_at: NaiveDateTime,
}