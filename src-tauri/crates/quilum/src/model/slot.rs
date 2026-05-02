use chrono::{DateTime, NaiveDateTime, TimeDelta};
use serde::{Deserialize, Serialize};
use surrealdb::types::SurrealValue;

#[derive(Clone, Debug, Serialize, Deserialize, SurrealValue)]
pub(crate) struct Slot {
    starts_at: i64,
    ends_at: i64,
}

impl Slot {
    pub fn new(starts_at: NaiveDateTime, ends_at: NaiveDateTime) -> Self {
        Self {
            starts_at: starts_at.and_utc().timestamp(),
            ends_at: ends_at.and_utc().timestamp(),
        }
    }

    pub fn starts_at(&self) -> NaiveDateTime {
        DateTime::from_timestamp(self.starts_at, 0).unwrap_or_default().naive_utc()
    }

    pub fn ends_at(&self) -> NaiveDateTime {
        DateTime::from_timestamp(self.ends_at, 0).unwrap_or_default().naive_utc()
    }

    pub fn duration(&self) -> TimeDelta {
        self.ends_at() - self.starts_at()
    }
}