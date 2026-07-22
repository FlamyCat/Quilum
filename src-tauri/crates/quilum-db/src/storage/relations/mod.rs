use chrono::{DateTime, Utc};
use surrealdb::{types::RecordId, Error};

use crate::Storage;

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
    pub async fn schedule_task(
        &self,
        slot_id: RecordId,
        task_id: RecordId,
        scheduled_for: DateTime<Utc>,
    ) -> Result<(), Error> {
        let sql = "
            fn::schedule_task($task, $slot, $time)
        ";

        self.db
            .query(sql)
            .bind(("task", task_id))
            .bind(("slot", slot_id))
            .bind(("time", scheduled_for))
            .await?;

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
