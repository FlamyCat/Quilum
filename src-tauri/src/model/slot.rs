use chrono::NaiveDateTime;

pub(crate) struct Slot {
    starts_at: NaiveDateTime,
    ends_at: NaiveDateTime,
}

impl Slot {
    pub fn new(starts_at: NaiveDateTime, ends_at: NaiveDateTime) -> Self {
        Self { starts_at, ends_at }
    }

    pub fn starts_at(&self) -> NaiveDateTime {
        self.starts_at
    }

    pub fn ends_at(&self) -> NaiveDateTime {
        self.ends_at
    }
}
