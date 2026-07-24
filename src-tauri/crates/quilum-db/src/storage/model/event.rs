use chrono::{DateTime, NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use surrealdb::types::{RecordId, RecordIdKey, SurrealValue};

pub const EVENTS_TABLE: &str = "events";

#[derive(Clone, Debug, Serialize, Deserialize, SurrealValue)]
pub struct Event {
    pub id: RecordId,
    pub title: String,
    pub description: String,
    pub starts_at: DateTime<Utc>,
    pub ends_at: DateTime<Utc>,
}

impl Event {
    pub fn id(&self) -> &RecordId {
        &self.id
    }

    pub fn name(&self) -> &str {
        &self.title
    }

    pub fn description(&self) -> &str {
        &self.description
    }

    pub fn starts_at(&self) -> DateTime<Utc> {
        self.starts_at
    }

    pub fn ends_at(&self) -> DateTime<Utc> {
        self.ends_at
    }

    pub fn set_name(&mut self, name: String) {
        self.title = name;
    }

    pub fn set_description(&mut self, description: String) {
        self.description = description;
    }

    pub fn set_starts_at(&mut self, starts_at: NaiveDateTime) {
        self.starts_at = starts_at.and_utc();
    }

    pub fn set_ends_at(&mut self, ends_at: NaiveDateTime) {
        self.ends_at = ends_at.and_utc();
    }

    pub fn new(title: String, description: String, starts_at: DateTime<Utc>, ends_at: DateTime<Utc>) -> Self {
        Self {
            id: RecordId::new(EVENTS_TABLE, RecordIdKey::ulid()),
            title,
            description,
            starts_at,
            ends_at,
        }
    }
}
