use chrono::{DateTime, TimeDelta, Utc};
use serde::{Deserialize, Serialize};
use surrealdb::types::{RecordId, RecordIdKey, SurrealValue};

pub const SLOTS_TABLE: &str = "slots";

#[derive(Clone, Debug, Serialize, Deserialize, SurrealValue)]
pub struct Slot {
    pub id: RecordId,
    pub starts_at: DateTime<Utc>,
    pub ends_at: DateTime<Utc>,
}

impl Slot {
    pub fn id(&self) -> &RecordId {
        &self.id
    }

    pub fn starts_at(&self) -> DateTime<Utc> {
        self.starts_at
    }

    pub fn ends_at(&self) -> DateTime<Utc> {
        self.ends_at
    }

    pub fn duration(&self) -> TimeDelta {
        self.ends_at() - self.starts_at()
    }

    pub fn new(starts_at: DateTime<Utc>, ends_at: DateTime<Utc>) -> Self {
        Self {
            id: RecordId::new(SLOTS_TABLE, RecordIdKey::ulid()),
            starts_at,
            ends_at,
        }
    }
}
