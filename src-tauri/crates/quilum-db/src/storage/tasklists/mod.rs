use surrealdb::{types::RecordId, Error};

use crate::{
    model::{
        task::Task,
        tasklist::TaskList,
    }, tasklist::TASKLISTS_TABLE,
    Storage,
    TaskListWithTasks,
};

impl Storage {
    /// Creates a new task list record in the database.
    ///
    /// # Arguments
    /// * `title` - Task list title
    ///
    /// # Returns
    /// * The created task list
    pub async fn create_task_list(&self, title: String) -> Result<TaskList, Error> {
        let tasklist = TaskList::new(title);

        let created: Option<TaskList> = self.db.create(TASKLISTS_TABLE).content(tasklist).await?;
        created.ok_or_else(|| Error::query("Failed to create task list".to_string(), None))
    }

    /// Reads a task list record from the database by its ID.
    ///
    /// # Arguments
    /// * `id` - The ID of the task list to read
    ///
    /// # Returns
    /// * The task list
    pub async fn read_task_list(&self, id: &RecordId) -> Result<TaskList, Error> {
        let list = self.db.select(id).await?;
        list.ok_or_else(|| Error::query("Task list not found".to_string(), None))
    }

    /// Updates a task list record in the database.
    ///
    /// # Arguments
    /// * `list` - The task list to update
    ///
    /// # Returns
    /// * Success or error
    pub async fn update_task_list(&self, list: TaskList) -> Result<(), Error> {
        let _: Option<TaskList> = self.db.update(list.id()).content(list).await?;
        Ok(())
    }

    /// Deletes a task list record from the database by its ID.
    ///
    /// # Arguments
    /// * `id` - The ID of the task list to delete
    ///
    /// # Returns
    /// * Success or error
    pub async fn delete_task_list(&self, id: &RecordId) -> Result<(), Error> {
        let _: Option<TaskList> = self.db.delete(id).await?;
        Ok(())
    }

    /// Gets all tasks in a task list.
    ///
    /// # Arguments
    /// * `list_id` - The task list record ID
    ///
    /// # Returns
    /// * The tasks in the list
    pub async fn get_tasks_in_list(
        &self,
        list_id: RecordId,
    ) -> Result<Vec<Task>, Error> {
        let sql = "SELECT ->contains->tasks AS tasks FROM $tasklist_id FETCH tasks";
        let tasks = self
            .db
            .query(sql)
            .bind(("tasklist_id", list_id))
            .await?
            .take::<Vec<_>>(0)?;

        Ok(tasks)
    }

    /// Gets all task lists with their tasks.
    ///
    /// # Returns
    /// * Vector of task lists with their tasks
    pub async fn get_all_task_lists_with_tasks(&self) -> Result<Vec<TaskListWithTasks>, Error> {
        let sql = "SELECT *, ->contains->tasks AS tasks FROM tasklists FETCH tasks";
        let lists = self.db.query(sql).await?.take(0)?;

        Ok(lists)
    }
}

#[cfg(test)]
mod tests;
