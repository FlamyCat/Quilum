use crate::{Storage, task::{Priority, Task}};
use chrono::{NaiveDateTime, TimeDelta};
use serde::{Deserialize, Serialize};
use surrealdb::{
    Error,
    types::{RecordId, SurrealValue},
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
        #[derive(Clone, Debug, Hash, PartialEq, Eq, Serialize, Deserialize, SurrealValue)]
        struct TaskCreate {
            name: String,
            description: String,
            priority: Priority,
            estimated_duration: i64,
            deadline: i64,
            completed: bool,
        }

        let data = TaskCreate {
            name: name,
            description: description,
            priority: priority,
            estimated_duration: estimated_duration.num_seconds(),
            deadline: deadline.and_utc().timestamp(),
            completed: false,
        };

        let created: Option<Task> = self.db.create("task").content(data).await?;
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
        let task: Option<Task> = self.db.select(("task", key)).await?;
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
        let _: Option<Task> = self.db.update(("task", key)).content(task).await?;
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
        let _: Option<Task> = self.db.delete(("task", key)).await?;
        Ok(())
    }
}
