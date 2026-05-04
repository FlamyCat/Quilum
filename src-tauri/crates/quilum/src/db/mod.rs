use chrono::{NaiveDate, NaiveDateTime, TimeDelta};
use serde::{Deserialize, Serialize};
use surrealdb::{
    types::{RecordId, SurrealValue},
    Error,
    Surreal
};
use surrealdb::engine::local::{Db, Mem, RocksDb};

use crate::{
    model::{
        event::Event,
        slot::Slot,
        task::{self, Task},
        tasklist::TaskList,
    }
};

/// Helper struct for creating tasks without the id field
#[derive(Serialize, Deserialize, SurrealValue)]
struct TaskCreate {
    name: String,
    description: String,
    priority: task::Priority,
    estimated_duration: i64,
    deadline: i64,
}

/// Storage struct that holds a handle to a SurrealDB instance
/// and exposes CRUD methods for events and tasks.
pub struct Storage {
    db: Surreal<Db>,
}

impl Storage {
    /// Creates a new Storage instance with the given SurrealDB connection.
    ///
    /// # Arguments
    /// * `db` - A SurrealDB instance connected to a specific engine
    ///
    /// # Returns
    /// * The storage instance or an error
    pub fn new(db: Surreal<Db>) -> Result<Self, Error> {
        Ok(Self { db })
    }

    /// Creates a new Storage instance using in-memory database mode.
    ///
    /// # Returns
    /// * The storage instance or an error
    pub async fn new_mem() -> Result<Self, Error> {
        let db = Surreal::new::<Mem>(()).await?;
        db.use_ns("test").use_db("test").await?;
        Ok(Self::new(db)?)
    }

    /// Creates a new Storage instance using RocksDB database mode.
    ///
    /// # Arguments
    /// * `path` - File path where the RocksDB database will be stored
    ///
    /// # Returns
    /// * The storage instance or an error
    pub async fn new_rocksdb(path: &str) -> Result<Self, Error> {
        let db = Surreal::new::<RocksDb>(path).await?;
        db.use_ns("test").use_db("test").await?;
        Ok(Self::new(db)?)
    }

    /// Helper to convert RecordId to string for SQL queries
    fn record_id_to_string(id: &RecordId) -> String {
        match &id.key {
            surrealdb::types::RecordIdKey::String(s) => format!("{}:{}", id.table, s),
            _ => format!("{}:unknown", id.table),
        }
    }

    /// Helper to extract key from RecordId for SurrealDB operations
    fn record_id_key(id: &RecordId) -> String {
        match &id.key {
            surrealdb::types::RecordIdKey::String(s) => s.as_str().to_string(),
            _ => "unknown".to_string(),
        }
    }
}

impl Storage {
    /// Creates a new event record in the database.
    ///
    /// # Arguments
    /// * `name` - Event name
    /// * `description` - Event description
    /// * `starts_at` - Start time
    /// * `ends_at` - End time
    ///
    /// # Returns
    /// * The created event
    pub async fn create_event(
        &self,
        name: String,
        description: String,
        starts_at: NaiveDateTime,
        ends_at: NaiveDateTime,
    ) -> Result<Event, Error> {
        let data = serde_json::json!({
            "name": name,
            "description": description,
            "starts_at": starts_at.and_utc().timestamp(),
            "ends_at": ends_at.and_utc().timestamp()
        });
        let created: Option<Event> = self.db.create("event").content(data).await?;
        created.ok_or_else(|| Error::query("Failed to create event".to_string(), None))
    }

    /// Reads an event record from the database by its ID.
    ///
    /// # Arguments
    /// * `id` - The ID of the event to read
    ///
    /// # Returns
    /// * The event
    pub async fn read_event(&self, id: &RecordId) -> Result<Event, Error> {
        let key = Self::record_id_key(id);
        let event: Option<Event> = self.db.select(("event", key)).await?;
        event.ok_or_else(|| Error::query("Event not found".to_string(), None))
    }

    /// Updates an event record in the database.
    ///
    /// # Arguments
    /// * `event` - The event to update
    ///
    /// # Returns
    /// * Success or error
    pub async fn update_event(&self, event: Event) -> Result<(), Error> {
        let key = Self::record_id_key(&event.id());
        let _: Option<Event> = self.db.update(("event", key)).content(event).await?;
        Ok(())
    }

    /// Deletes an event record from the database by its ID.
    ///
    /// # Arguments
    /// * `id` - The ID of the event to delete
    ///
    /// # Returns
    /// * Success or error
    pub async fn delete_event(&self, id: &RecordId) -> Result<(), Error> {
        let key = Self::record_id_key(id);
        let _: Option<Event> = self.db.delete(("event", key)).await?;
        Ok(())
    }
}

impl Storage {
    /// Gets events occurring within a date range (inclusive start, exclusive end).
    /// Events that overlap with the date range are returned (even if they span multiple days).
    ///
    /// # Arguments
    /// * `start` - The start date (inclusive, at 00:00:00)
    /// * `end` - The end date (exclusive, at 00:00:00)
    ///
    /// # Returns
    /// * Vector of events occurring in the date range
    pub async fn get_events_for_date_range(
        &self,
        start: NaiveDate,
        end: NaiveDate,
    ) -> Result<Vec<Event>, Error> {
        let range_start = start.and_hms_opt(0, 0, 0).unwrap().and_utc().timestamp();
        let range_end = end.and_hms_opt(0, 0, 0).unwrap().and_utc().timestamp();

        let sql = format!(
            "SELECT * FROM event WHERE starts_at < {} AND ends_at > {}",
            range_end, range_start
        );
        let mut result = self.db.query(sql).await?;
        let events: Vec<Event> = result.take(0).unwrap_or_default();
        Ok(events)
    }

    /// Gets events occurring on a specific date.
    /// Events that overlap with the date are returned (even if they span multiple days).
    ///
    /// # Arguments
    /// * `date` - The date to query
    ///
    /// # Returns
    /// * Vector of events occurring on the date
    pub async fn get_events_for_date(
        &self,
        date: NaiveDate,
    ) -> Result<Vec<Event>, Error> {
        let next_day = date + TimeDelta::days(1);
        self.get_events_for_date_range(date, next_day).await
    }
}

impl Storage {
    /// Gets all tasks scheduled in slots within a date range.
    /// Returns flat list of tasks with their scheduled_for timestamps (for today view).
    ///
    /// # Arguments
    /// * `start` - The start date (inclusive, at 00:00:00)
    /// * `end` - The end date (exclusive, at 00:00:00)
    ///
    /// # Returns
    /// * Vector of tasks with their scheduled_for timestamps
    pub async fn get_scheduled_tasks_for_date_range(
        &self,
        start: NaiveDate,
        end: NaiveDate,
    ) -> Result<Vec<(Task, i64)>, Error> {
        let range_start = start.and_hms_opt(0, 0, 0).unwrap().and_utc().timestamp();
        let range_end = end.and_hms_opt(0, 0, 0).unwrap().and_utc().timestamp();

        // Use graph traversal to get task data directly
        let sql = format!(
            "SELECT out.*, scheduled_for FROM contains \
            WHERE in.starts_at < {} AND in.ends_at > {}",
            range_end, range_start
        );
        let mut result = self.db.query(sql).await?;
        let raw: Vec<serde_json::Value> = result.take(0).unwrap_or_default();
        
        let mut scheduled_tasks = Vec::new();
        for item in raw {
            if let (Some(task_value), Some(scheduled_for)) = (item.get("out"), item.get("scheduled_for")) {
                if let (Some(task_obj), Some(sf)) = (task_value.as_object(), scheduled_for.as_i64()) {
                    // Convert the task object to proper JSON, handling id field
                    let mut task_json = serde_json::Map::new();
                    for (k, v) in task_obj {
                        if k == "id" {
                            // id is a string like "task:xxx", convert to object format for RecordId
                            if let Some(id_str) = v.as_str() {
                                let parts: Vec<&str> = id_str.split(':').collect();
                                if parts.len() == 2 {
                                    let mut id_obj = serde_json::Map::new();
                                    id_obj.insert("table".to_string(), serde_json::Value::String(parts[0].to_string()));
                                    id_obj.insert("key".to_string(), serde_json::json!({"String": parts[1]}));
                                    task_json.insert("id".to_string(), serde_json::Value::Object(id_obj));
                                }
                            }
                        } else if k == "priority" {
                            // Handle priority enum - extract the variant name
                            if let Some(priority_obj) = v.as_object() {
                                if let Some(first_key) = priority_obj.keys().next() {
                                    task_json.insert("priority".to_string(), serde_json::Value::String(first_key.clone()));
                                }
                            }
                        } else {
                            task_json.insert(k.clone(), v.clone());
                        }
                    }
                    let task: Task = serde_json::from_value(serde_json::Value::Object(task_json))
                        .map_err(|e| Error::query(format!("Failed to deserialize task: {}", e), None))?;
                    scheduled_tasks.push((task, sf));
                }
            }
        }
        Ok(scheduled_tasks)
    }
}

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
        priority: crate::model::task::Priority,
        estimated_duration: TimeDelta,
        deadline: NaiveDateTime,
    ) -> Result<Task, Error> {
        let task_data = TaskCreate {
            name,
            description,
            priority,
            estimated_duration: estimated_duration.num_seconds(),
            deadline: deadline.and_utc().timestamp(),
        };
        let created: Option<Task> = self.db.create("task").content(task_data).await?;
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
        let key = Self::record_id_key(id);
        let _: Option<Task> = self.db.delete(("task", key)).await?;
        Ok(())
    }
}

impl Storage {
    /// Creates a new slot record in the database.
    ///
    /// # Arguments
    /// * `starts_at` - Start time
    /// * `ends_at` - End time
    ///
    /// # Returns
    /// * The created slot
    pub async fn create_slot(
        &self,
        starts_at: NaiveDateTime,
        ends_at: NaiveDateTime,
    ) -> Result<Slot, Error> {
        let data = serde_json::json!({
            "starts_at": starts_at.and_utc().timestamp(),
            "ends_at": ends_at.and_utc().timestamp()
        });
        let created: Option<Slot> = self.db.create("slot").content(data).await?;
        created.ok_or_else(|| Error::query("Failed to create slot".to_string(), None))
    }

    /// Reads a slot record from the database by its ID.
    ///
    /// # Arguments
    /// * `id` - The ID of the slot to read
    ///
    /// # Returns
    /// * The slot
    pub async fn read_slot(&self, id: &RecordId) -> Result<Slot, Error> {
        let key = Self::record_id_key(id);
        let slot: Option<Slot> = self.db.select(("slot", key)).await?;
        slot.ok_or_else(|| Error::query("Slot not found".to_string(), None))
    }

    /// Updates a slot record in the database.
    ///
    /// # Arguments
    /// * `slot` - The slot to update
    ///
    /// # Returns
    /// * Success or error
    pub async fn update_slot(&self, slot: Slot) -> Result<(), Error> {
        let key = Self::record_id_key(&slot.id());
        let _: Option<Slot> = self.db.update(("slot", key)).content(slot).await?;
        Ok(())
    }

    /// Deletes a slot record from the database by its ID.
    ///
    /// # Arguments
    /// * `id` - The ID of the slot to delete
    ///
    /// # Returns
    /// * Success or error
    pub async fn delete_slot(&self, id: &RecordId) -> Result<(), Error> {
        let key = Self::record_id_key(id);
        let _: Option<Slot> = self.db.delete(("slot", key)).await?;
        Ok(())
    }
}

impl Storage {
    /// Creates a new task list record in the database.
    ///
    /// # Arguments
    /// * `title` - Task list title
    ///
    /// # Returns
    /// * The created task list
    pub async fn create_task_list(
        &self,
        title: String,
    ) -> Result<TaskList, Error> {
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
}

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

    /// Gets the slot for a task (if scheduled).
    ///
    /// # Arguments
    /// * `task_id` - The task record ID
    ///
    /// # Returns
    /// * The slot if found, None if not scheduled
    pub async fn get_slot_for_task(
        &self,
        task_id: &RecordId,
    ) -> Result<Option<Slot>, Error> {
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
    pub async fn get_tasks_in_slot(
        &self,
        slot_id: &RecordId,
    ) -> Result<Vec<Task>, Error> {
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

    /// Gets all tasks in a task list.
    ///
    /// # Arguments
    /// * `list_id` - The task list record ID
    ///
    /// # Returns
    /// * The tasks in the list
    pub async fn get_tasks_in_list(
        &self,
        list_id: &RecordId,
    ) -> Result<Vec<Task>, Error> {
        let sql = format!(
            "SELECT * FROM task WHERE id IN (SELECT in FROM belongs_to WHERE out = {})",
            Self::record_id_to_string(list_id)
        );
        let mut result = self.db.query(sql).await?;
        let tasks: Vec<Task> = result.take(0)?;
        Ok(tasks)
    }
}

#[cfg(test)]
mod tests;