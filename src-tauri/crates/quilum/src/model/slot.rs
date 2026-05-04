use chrono::{DateTime, NaiveDateTime, TimeDelta};
use serde::{Deserialize, Serialize};
use surrealdb::types::{RecordId, SurrealValue};

#[derive(Clone, Debug, Serialize, Deserialize, SurrealValue)]
pub(crate) struct Slot {
    #[serde(skip_serializing)]
    pub(crate) id: RecordId,
    pub(crate) starts_at: i64,
    pub(crate) ends_at: i64,
}

impl Slot {
    pub fn id(&self) -> &RecordId {
        &self.id
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
