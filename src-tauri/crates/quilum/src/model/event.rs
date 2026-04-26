use chrono::NaiveDateTime;

pub(crate) struct Event {
    name: String,
    description: String,
    starts_at: NaiveDateTime,
    ends_at: NaiveDateTime,
}

impl Event {
    pub fn new(
        name: String,
        description: String,
        starts_at: NaiveDateTime,
        ends_at: NaiveDateTime,
    ) -> Self {
        Self {
            name,
            description,
            starts_at,
            ends_at,
        }
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
