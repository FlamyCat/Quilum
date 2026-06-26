use crate::{Storage, TaskListWithTasks, task::Task, tasklist::TaskList};
use surrealdb::{Error, types::RecordId};

impl Storage {
    /// Creates a new task list record in the database.
    ///
    /// # Arguments
    /// * `title` - Task list title
    ///
    /// # Returns
    /// * The created task list
    pub async fn create_task_list(&self, title: String) -> Result<TaskList, Error> {
        let data = serde_json::json!({
            "title": title
        });
        let created: Option<TaskList> = self.db.create("task_list").content(data).await?;
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
        let key = Self::record_id_key(id);
        let list: Option<TaskList> = self.db.select(("task_list", key)).await?;
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
        let key = Self::record_id_key(&list.id());
        let _: Option<TaskList> = self.db.update(("task_list", key)).content(list).await?;
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
        let key = Self::record_id_key(id);
        let _: Option<TaskList> = self.db.delete(("task_list", key)).await?;
        Ok(())
    }

    /// Deletes all tasks in a task list.
    ///
    /// # Arguments
    /// * `list_id` - The task list record ID
    ///
    /// # Returns
    /// * Success or error
    pub async fn delete_tasks_in_list(&self, list_id: &RecordId) -> Result<(), Error> {
        let sql = format!(
            "DELETE FROM task WHERE id IN (SELECT in FROM belongs_to WHERE out = {})",
            Self::record_id_to_string(list_id)
        );
        self.db.query(sql).await?;
        Ok(())
    }

    /// Gets all tasks in a task list.
    ///
    /// # Arguments
    /// * `list_id` - The task list record ID
    ///
    /// # Returns
    /// * The tasks in the list
    pub async fn get_tasks_in_list(&self, list_id: &RecordId) -> Result<Vec<Task>, Error> {
        let sql = format!(
            "SELECT in.* FROM belongs_to WHERE out = {}",
            Self::record_id_to_string(list_id)
        );
        let mut result = self.db.query(sql).await?;
        let raw: Vec<serde_json::Value> = result.take(0).unwrap_or_default();

        let mut tasks = Vec::new();
        for item in raw {
            if let Some(task_value) = item.get("in") {
                if let Some(task_obj) = task_value.as_object() {
                    let mut task_json = serde_json::Map::new();
                    for (k, v) in task_obj {
                        if k == "id" {
                            if let Some(id_str) = v.as_str() {
                                let parts: Vec<&str> = id_str.split(':').collect();
                                if parts.len() == 2 {
                                    let mut id_obj = serde_json::Map::new();
                                    id_obj.insert(
                                        "table".to_string(),
                                        serde_json::Value::String(parts[0].to_string()),
                                    );
                                    id_obj.insert(
                                        "key".to_string(),
                                        serde_json::json!({"String": parts[1]}),
                                    );
                                    task_json.insert(
                                        "id".to_string(),
                                        serde_json::Value::Object(id_obj),
                                    );
                                }
                            }
                        } else if k == "priority" {
                            if let Some(priority_obj) = v.as_object() {
                                if let Some(first_key) = priority_obj.keys().next() {
                                    task_json.insert(
                                        "priority".to_string(),
                                        serde_json::Value::String(first_key.clone()),
                                    );
                                }
                            }
                        } else {
                            task_json.insert(k.clone(), v.clone());
                        }
                    }
                    let task: Task = serde_json::from_value(serde_json::Value::Object(task_json))
                        .map_err(|e| {
                        Error::query(format!("Failed to deserialize task: {}", e), None)
                    })?;
                    tasks.push(task);
                }
            }
        }
        Ok(tasks)
    }

    /// Gets all task lists with their tasks.
    ///
    /// # Returns
    /// * Vector of task lists with their tasks
    pub async fn get_all_task_lists_with_tasks(&self) -> Result<Vec<TaskListWithTasks>, Error> {
        let sql = "SELECT * FROM task_list".to_string();
        let mut result = self.db.query(sql).await?;
        let lists: Vec<TaskList> = result.take(0).unwrap_or_default();

        let mut result_lists = Vec::new();
        for list in lists {
            let tasks = self.get_tasks_in_list(list.id()).await?;
            result_lists.push(TaskListWithTasks { list, tasks });
        }

        Ok(result_lists)
    }
}

#[cfg(test)]
mod tests;
