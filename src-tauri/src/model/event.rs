use chrono::NaiveDateTime;
use surrealdb::RecordId;

pub(crate) struct Event {
    id: RecordId,
    name: String,
    description: String,
    starts_at: NaiveDateTime,
    ends_at: NaiveDateTime,
}

impl Event {
    pub fn new(
        id: RecordId,
        name: String,
        description: String,
        starts_at: NaiveDateTime,
        ends_at: NaiveDateTime,
    ) -> Self {
        Self {
            id,
            name,
            description,
            starts_at,
            ends_at,
        }
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

    pub fn starts_at(&self) -> NaiveDateTime {
        self.starts_at
    }

    pub fn ends_at(&self) -> NaiveDateTime {
        self.ends_at
    }
}
