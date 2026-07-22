use std::collections::HashSet;

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
            BEGIN TRANSACTION;
            RELATE $task -> scheduled_in:[$task, $slot] -> $slot;
            UPDATE $task SET scheduled_for = $time;
            COMMIT;
        ";

        self.db
            .query(sql)
            .bind(("task", task_id))
            .bind(("slot", slot_id))
            .bind(("time", scheduled_for))
            .await?;

        Ok(())
    }

    /// Unschedules tasks by IDs.
    ///
    /// This method removes the edges connecting respective tasks with their slots and also sets
    /// `scheduled_for` to `NONE`.
    ///
    /// # Arguments
    /// * `task_ids` - Vector of task record IDs to unschedule.
    ///
    /// # Returns
    /// * Success or error
    pub async fn unschedule_tasks(&self, task_ids: HashSet<RecordId>) -> Result<(), Error> {
        let sql = "
            BEGIN TRANSACTION;
            UPDATE $tasks SET scheduled_for = NONE;
            DELETE scheduled_in WHERE in IN $tasks;
            COMMIT;
        ";

        self.db.query(sql).bind(("tasks", task_ids)).await?;
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
    pub async fn put_task_into_list(
        &self,
        task_id: RecordId,
        list_id: RecordId,
    ) -> Result<(), Error> {
        let sql = "
            RELATE $list -> contains:[$list, $task] -> $task;
        ";

        self.db
            .query(sql)
            .bind(("task", task_id))
            .bind(("list", list_id))
            .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests;
