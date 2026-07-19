use chrono::{NaiveDateTime, TimeDelta};
use surrealdb::{types::RecordId, Error};

use crate::{
    storage::model::task::{Priority, Task, TaskData, Unscheduled, TASKS_TABLE},
    Storage,
};

impl Storage {
    /// Creates a new task record in the database.
    ///
    /// # Arguments
    /// * `name` - Task name
    /// * `description` - Task description
    /// * `priority` - Task priority
    /// * `estimated_duration` - Estimated duration
    /// * `deadline` - Deadline
    ///
    /// # Returns
    /// * The created task
    pub async fn create_task(
        &self,
        name: String,
        description: String,
        priority: Priority,
        estimated_duration: TimeDelta,
        deadline: NaiveDateTime,
    ) -> Result<Task<Unscheduled>, Error> {
        let data = TaskData::new(
            name,
            description,
            priority,
            estimated_duration.num_seconds(),
            deadline.and_utc().timestamp(),
            false,
            None,
        );

        let created: Option<Task<_>> = self.db.create(TASKS_TABLE).content(data).await?;
        created.ok_or_else(|| Error::query("Failed to create task".to_string(), None))
    }

    /// Reads a task record from the database by its ID.
    ///
    /// # Arguments
    /// * `id` - The ID of the task to read
    ///
    /// # Returns
    /// * The task
    pub async fn read_task(&self, id: &RecordId) -> Result<Task<Unscheduled>, Error> {
        let key = Self::record_id_key(id);
        let task: Option<Task<_>> = self.db.select((TASKS_TABLE, key)).await?;
        task.ok_or_else(|| Error::query("Task not found".to_string(), None))
    }

    /// Updates a task record in the database.
    ///
    /// # Arguments
    /// * `task` - The task to update
    ///
    /// # Returns
    /// * Success or error
    pub async fn update_task(&self, task: Task<Unscheduled>) -> Result<(), Error> {
        let key = Self::record_id_key(&task.id());
        let _: Option<Task<Unscheduled>> = self.db.update((TASKS_TABLE, key)).content(task).await?;
        Ok(())
    }

    /// Deletes a task record from the database by its ID.
    ///
    /// # Arguments
    /// * `id` - The ID of the task to delete
    ///
    /// # Returns
    /// * Success or error
    pub async fn delete_task(&self, id: &RecordId) -> Result<(), Error> {
        self.delete_task_slot_relations(&[id.clone()]).await?;
        let key = Self::record_id_key(id);
        let _: Option<Task<Unscheduled>> = self.db.delete((TASKS_TABLE, key)).await?;
        Ok(())
    }
}
