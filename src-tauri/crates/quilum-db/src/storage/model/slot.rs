use chrono::{DateTime, NaiveDateTime, TimeDelta};
use serde::{Deserialize, Serialize};
use surrealdb::types::{RecordId, RecordIdKey, SurrealValue};

pub const SLOTS_TABLE: &str = "slots";

#[derive(Clone, Debug, Serialize, Deserialize, SurrealValue)]
pub struct Slot {
    pub id: RecordId,
    pub starts_at: i64,
    pub ends_at: i64,
}

impl Slot {
    pub fn id(&self) -> &RecordId {
        &self.id
    }

    pub fn starts_at(&self) -> NaiveDateTime {
        DateTime::from_timestamp(self.starts_at, 0)
            .unwrap_or_default()
            .naive_utc()
    }

    pub fn ends_at(&self) -> NaiveDateTime {
        DateTime::from_timestamp(self.ends_at, 0)
            .unwrap_or_default()
            .naive_utc()
    }

    pub fn duration(&self) -> TimeDelta {
        self.ends_at() - self.starts_at()
    }

    pub fn new(starts_at: i64, ends_at: i64) -> Self {
        Self {
            id: RecordId::new(SLOTS_TABLE, RecordIdKey::ulid()),
            starts_at,
            ends_at,
        }
    }
}
