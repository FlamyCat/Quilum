use chrono::NaiveDateTime;
use surrealdb::types::RecordId;

#[derive(Clone, Debug)]
pub(crate) struct Plan {
    scheduled: Vec<(RecordId, RecordId, NaiveDateTime)>, // (task_id, slot_id, scheduled_for)
    discarded_task_ids: Vec<RecordId>,
    score: u64,
}

impl Plan {
    pub fn new() -> Self {
        Self {
            scheduled: Vec::new(),
            discarded_task_ids: Vec::new(),
            score: 0,
        }
    }

    pub fn add_task(&mut self, task_id: RecordId, slot_id: RecordId, scheduled_for: NaiveDateTime, priority: u64) {
        self.scheduled.push((task_id, slot_id, scheduled_for));
        self.score += priority;
    }

    pub fn with_task(self, task_id: RecordId, slot_id: RecordId, scheduled_for: NaiveDateTime, priority: u64) -> Self {
        let mut scheduled = self.scheduled;
        scheduled.push((task_id, slot_id, scheduled_for));

        Self {
            score: self.score + priority,
            scheduled,
            ..self
        }
    }

    pub fn discard_task(&mut self, task_id: RecordId) {
        self.discarded_task_ids.push(task_id);
    }

    pub fn discard_tasks(&mut self, task_ids: impl Iterator<Item = RecordId>) {
        task_ids.collect_into(&mut self.discarded_task_ids);
    }

    pub fn score(&self) -> u64 {
        self.score
    }

    pub fn tasks(&self) -> &Vec<(RecordId, RecordId, NaiveDateTime)> {
        &self.scheduled
    }

    pub fn discarded_tasks(&self) -> &Vec<RecordId> {
        &self.discarded_task_ids
    }
}
