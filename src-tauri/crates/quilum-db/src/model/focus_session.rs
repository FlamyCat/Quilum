use serde::{Deserialize, Serialize};
use chrono::{DateTime, NaiveDateTime};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FocusSession {
    pub id: surrealdb::types::RecordId,
    pub start_time: i64,  // Unix timestamp (seconds)
    pub end_time: i64,    // Unix timestamp (seconds)
    pub task_id: Option<surrealdb::types::RecordId>,
}

impl FocusSession {
    /// Create a new FocusSession with NaiveDateTime values
    pub fn new(start: NaiveDateTime, end: NaiveDateTime, task_id: Option<surrealdb::types::RecordId>) -> Self {
        Self {
            id: surrealdb::types::RecordId::new("focus_session", "temp"),
            start_time: start.and_utc().timestamp(),
            end_time: end.and_utc().timestamp(),
            task_id,
        }
    }

    /// Get start_time as NaiveDateTime
    pub fn start_time(&self) -> NaiveDateTime {
        DateTime::from_timestamp(self.start_time, 0)
            .map(|dt| dt.naive_utc())
            .unwrap_or_default()
    }

    /// Get end_time as NaiveDateTime
    pub fn end_time(&self) -> NaiveDateTime {
        DateTime::from_timestamp(self.end_time, 0)
            .map(|dt| dt.naive_utc())
            .unwrap_or_default()
    }
}
