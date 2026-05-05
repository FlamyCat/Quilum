use chrono::{DateTime, NaiveDateTime};
use serde::{Deserialize, Serialize};
use surrealdb::types::{RecordId, SurrealValue};

#[derive(Clone, Debug, Serialize, Deserialize, SurrealValue)]
pub struct Event {
    pub id: RecordId,
    pub name: String,
    pub description: String,
    pub starts_at: i64,
    pub ends_at: i64,
}

impl Event {
    pub fn id(&self) -> &RecordId {
        &self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn description(&self) -> &str {
        &self.description
    }

    pub fn starts_at(&self) -> NaiveDateTime {
        DateTime::from_timestamp(self.starts_at, 0).unwrap_or_default().naive_utc()
    }

    pub fn ends_at(&self) -> NaiveDateTime {
        DateTime::from_timestamp(self.ends_at, 0).unwrap_or_default().naive_utc()
    }

    pub fn set_name(&mut self, name: String) {
        self.name = name;
    }

    pub fn set_description(&mut self, description: String) {
        self.description = description;
    }

    pub fn set_starts_at(&mut self, starts_at: NaiveDateTime) {
        self.starts_at = starts_at.and_utc().timestamp();
    }

    pub fn set_ends_at(&mut self, ends_at: NaiveDateTime) {
        self.ends_at = ends_at.and_utc().timestamp();
    }
}
