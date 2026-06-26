use crate::{Storage, slot::Slot, task::Task};
use chrono::NaiveDateTime;
use surrealdb::{Error, types::RecordId};

impl Storage {
    /// Relates a task to a slot with the scheduled_for timestamp.
    ///
    /// # Arguments
    /// * `slot_id` - The slot record ID
    /// * `task_id` - The task record ID
    /// * `scheduled_for` - The timestamp when the task is scheduled
    ///
    /// # Returns
    /// * Success or error
    pub async fn relate_task_to_slot(
        &self,
        slot_id: &RecordId,
        task_id: &RecordId,
        scheduled_for: NaiveDateTime,
    ) -> Result<(), Error> {
        let sql = format!(
            "RELATE {}->contains->{} SET scheduled_for = {}",
            Self::record_id_to_string(slot_id),
            Self::record_id_to_string(task_id),
            scheduled_for.and_utc().timestamp()
        );
        self.db.query(sql).await?;
        Ok(())
    }

    /// Deletes task-slot relations (contains edges) for the specified tasks.
    ///
    /// # Arguments
    /// * `task_ids` - Vector of task record IDs to unlink from slots
    ///
    /// # Returns
    /// * Success or error
    pub async fn delete_task_slot_relations(&self, task_ids: &[RecordId]) -> Result<(), Error> {
        for task_id in task_ids {
            let sql = format!(
                "DELETE FROM contains WHERE out = {}",
                Self::record_id_to_string(task_id)
            );
            self.db.query(sql).await?;
        }
        Ok(())
    }

    /// Gets the slot for a task (if scheduled).
    ///
    /// # Arguments
    /// * `task_id` - The task record ID
    ///
    /// # Returns
    /// * The slot if found, None if not scheduled
    pub async fn get_slot_for_task(&self, task_id: &RecordId) -> Result<Option<Slot>, Error> {
        let sql = format!(
            "SELECT * FROM ONLY slot WHERE id IN (SELECT out FROM contains WHERE in = {}) LIMIT 1",
            Self::record_id_to_string(task_id)
        );
        let mut result = self.db.query(sql).await?;
        let slot: Option<Slot> = result.take(0)?;
        Ok(slot)
    }

    /// Gets all tasks in a slot.
    ///
    /// # Arguments
    /// * `slot_id` - The slot record ID
    ///
    /// # Returns
    /// * The tasks in the slot
    pub async fn get_tasks_in_slot(&self, slot_id: &RecordId) -> Result<Vec<Task>, Error> {
        let sql = format!(
            "SELECT * FROM task WHERE id IN (SELECT in FROM contains WHERE out = {})",
            Self::record_id_to_string(slot_id)
        );
        let mut result = self.db.query(sql).await?;
        let tasks: Vec<Task> = result.take(0)?;
        Ok(tasks)
    }

    /// Relates a task to a task list.
    ///
    /// # Arguments
    /// * `task_id` - The task record ID
    /// * `list_id` - The task list record ID
    ///
    /// # Returns
    /// * Success or error
    pub async fn relate_task_to_list(
        &self,
        task_id: &RecordId,
        list_id: &RecordId,
    ) -> Result<(), Error> {
        let sql = format!(
            "RELATE {}->belongs_to->{}",
            Self::record_id_to_string(task_id),
            Self::record_id_to_string(list_id)
        );
        self.db.query(sql).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests;
