use std::collections::HashSet;
use chrono::{NaiveDateTime, TimeDelta};
use surrealdb::{types::RecordId, Error};

use crate::{
    model::task::{Priority, Task, TASKS_TABLE},
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
    ) -> Result<Task, Error> {
        let data = Task::new(
            name,
            description,
            priority,
            estimated_duration.to_std().expect("Duration should be positive"),
            deadline.and_utc(),
            false,
            None,
        );

        let created: Option<Task> = self.db.create(TASKS_TABLE).content(data).await?;
        created.ok_or_else(|| Error::query("Failed to create task".to_string(), None))
    }

    /// Reads a task record from the database by its ID.
    ///
    /// # Arguments
    /// * `id` - The ID of the task to read
    ///
    /// # Returns
    /// * The task
    pub async fn read_task(&self, id: &RecordId) -> Result<Task, Error> {
        let key = Self::record_id_key(id);
        let task: Option<Task> = self.db.select((TASKS_TABLE, key)).await?;
        task.ok_or_else(|| Error::query("Task not found".to_string(), None))
    }

    /// Updates a task record in the database.
    ///
    /// # Arguments
    /// * `task` - The task to update
    ///
    /// # Returns
    /// * Success or error
    pub async fn update_task(&self, task: Task) -> Result<(), Error> {
        let key = Self::record_id_key(&task.id());
        let _: Option<Task> = self.db.update((TASKS_TABLE, key)).content(task).await?;
        Ok(())
    }

    /// Deletes a task record from the database by its ID.
    ///
    /// The task is also unscheduled.
    ///
    /// # Arguments
    /// * `id` - The ID of the task to delete
    ///
    /// # Returns
    /// * Success or error
    pub async fn delete_task(&self, id: &RecordId) -> Result<(), Error> {
        self.unschedule_tasks(HashSet::from([id.clone()])).await?;
        let _: Option<Task<Unscheduled>> = self.db.delete(id).await?;
        Ok(())
    }
}
