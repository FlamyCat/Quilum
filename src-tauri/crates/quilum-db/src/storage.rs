use crate::{
    slot::Slot,
    focus_session::FocusSession,
    event::Event,
    blocked_app::BlockedApp,
    app_identifier::AppIdentifier,
    task::{Priority, Task},
    tasklist::TaskList
};
use chrono::{NaiveDate, NaiveDateTime, TimeDelta};
use surrealdb::{
    engine::local::{Db, Mem, RocksDb},
    types::RecordId,
    Error, Surreal,
};

/// Storage struct that holds a handle to a SurrealDB instance
/// and exposes CRUD methods for events, tasks, and app blocking.
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
        db.use_ns("quilum").use_db("main").await?;
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
        db.use_ns("quilum").use_db("main").await?;
        Ok(Self::new(db)?)
    }

    /// Initializes the database by running setup queries from init.surql
    pub async fn init(&self) -> Result<(), Error> {
        let init_sql = include_str!("../resources/init.surql");
        self.db.query(init_sql).await?;
        Ok(())
    }

    /// Helper to convert RecordId to string for SQL queries
    fn record_id_to_string(id: &RecordId) -> String {
        match &id.key {
            surrealdb::types::RecordIdKey::String(s) => format!("{}:{}", id.table, s),
            _ => format!("{}:unknown", id.table),
        }
    }

}

// Event CRUD methods
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
        let sql = format!(
            "CREATE event CONTENT {{ name: '{}', description: '{}', starts_at: {}, ends_at: {} }}",
            name.replace('\'', "''"),
            description.replace('\'', "''"),
            starts_at.and_utc().timestamp(),
            ends_at.and_utc().timestamp()
        );
        let mut result = self.db.query(sql).await?;
        let value: Option<serde_json::Value> = result.take(0)?;
        let event: Event = serde_json::from_value(value.ok_or_else(|| Error::query("Failed to create event".to_string(), None))?).map_err(|e| Error::query(format!("Deserialization error: {}", e), None))?;
        Ok(event)
    }

    /// Reads an event record from the database by its ID.
    ///
    /// # Arguments
    /// * `id` - The ID of the event to read
    ///
    /// # Returns
    /// * The event
    pub async fn read_event(&self, id: &RecordId) -> Result<Event, Error> {
        let key = Self::record_id_to_string(id);
        let sql = format!("SELECT * FROM {}", key);
        let mut result = self.db.query(sql).await?;
        let value: Option<serde_json::Value> = result.take(0)?;
        let event: Event = serde_json::from_value(value.ok_or_else(|| Error::query("Event not found".to_string(), None))?).map_err(|e| Error::query(format!("Deserialization error: {}", e), None))?;
        Ok(event)
    }

    /// Updates an event record in the database.
    ///
    /// # Arguments
    /// * `event` - The event to update
    ///
    /// # Returns
    /// * Success or error
    pub async fn update_event(&self, event: Event) -> Result<(), Error> {
        let key = Self::record_id_to_string(&event.id());
        let sql = format!(
            "UPDATE {} CONTENT {{ name: '{}', description: '{}', starts_at: {}, ends_at: {} }}",
            key,
            event.name.replace('\'', "''"),
            event.description.replace('\'', "''"),
            event.starts_at,
            event.ends_at
        );
        let _: Option<serde_json::Value> = self.db.query(sql).await?.take(0)?;
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
        let key = Self::record_id_to_string(id);
        let sql = format!("DELETE {}", key);
        let _: Option<serde_json::Value> = self.db.query(sql).await?.take(0)?;
        Ok(())
    }
}

// Event query methods
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
        let values: Vec<serde_json::Value> = result.take(0).unwrap_or_default();
        let events: Vec<Event> = values.into_iter().map(|v| serde_json::from_value(v).map_err(|e| Error::query(format!("Deserialization error: {}", e), None))).collect::<Result<Vec<_>, _>>()?;
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

// Scheduled tasks query methods
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

// Task CRUD methods
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
        let priority_str = match priority {
            Priority::Low => "Low",
            Priority::Medium => "Medium",
            Priority::High => "High",
        };
        let sql = format!(
            "CREATE task CONTENT {{ name: '{}', description: '{}', priority: {}, estimated_duration: {}, deadline: {} }}",
            name.replace('\'', "''"),
            description.replace('\'', "''"),
            priority_str,
            estimated_duration.num_seconds(),
            deadline.and_utc().timestamp()
        );
        let mut result = self.db.query(sql).await?;
        let value: Option<serde_json::Value> = result.take(0)?;
        let task: Task = serde_json::from_value(value.ok_or_else(|| Error::query("Failed to create task".to_string(), None))?)
            .map_err(|e| Error::query(format!("Deserialization error: {}", e), None))?;
        Ok(task)
    }

    /// Reads a task record from the database by its ID.
    ///
    /// # Arguments
    /// * `id` - The ID of the task to read
    ///
    /// # Returns
    /// * The task
    pub async fn read_task(&self, id: &RecordId) -> Result<Task, Error> {
        let key = Self::record_id_to_string(id);
        let sql = format!("SELECT * FROM {}", key);
        let mut result = self.db.query(sql).await?;
        let value: Option<serde_json::Value> = result.take(0)?;
        let task: Task = serde_json::from_value(value.ok_or_else(|| Error::query("Task not found".to_string(), None))?)
            .map_err(|e| Error::query(format!("Deserialization error: {}", e), None))?;
        Ok(task)
    }

    /// Updates a task record in the database.
    ///
    /// # Arguments
    /// * `task` - The task to update
    ///
    /// # Returns
    /// * Success or error
    pub async fn update_task(&self, task: Task) -> Result<(), Error> {
        let key = Self::record_id_to_string(&task.id());
        let priority_str = match task.priority {
            Priority::Low => "Low",
            Priority::Medium => "Medium",
            Priority::High => "High",
        };
        let sql = format!(
            "UPDATE {} CONTENT {{ name: '{}', description: '{}', priority: {}, estimated_duration: {}, deadline: {} }}",
            key,
            task.name.replace('\'', "''"),
            task.description.replace('\'', "''"),
            priority_str,
            task.estimated_duration,
            task.deadline
        );
        let _: Option<serde_json::Value> = self.db.query(sql).await?.take(0)?;
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
        let key = Self::record_id_to_string(id);
        let sql = format!("DELETE {}", key);
        let _: Option<serde_json::Value> = self.db.query(sql).await?.take(0)?;
        Ok(())
    }
}

// Slot CRUD methods
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
        let sql = format!(
            "CREATE slot CONTENT {{ starts_at: {}, ends_at: {} }}",
            starts_at.and_utc().timestamp(),
            ends_at.and_utc().timestamp()
        );
        let mut result = self.db.query(sql).await?;
        let value: Option<serde_json::Value> = result.take(0)?;
        let slot: Slot = serde_json::from_value(value.ok_or_else(|| Error::query("Failed to create slot".to_string(), None))?)
            .map_err(|e| Error::query(format!("Deserialization error: {}", e), None))?;
        Ok(slot)
    }

    /// Reads a slot record from the database by its ID.
    ///
    /// # Arguments
    /// * `id` - The ID of the slot to read
    ///
    /// # Returns
    /// * The slot
    pub async fn read_slot(&self, id: &RecordId) -> Result<Slot, Error> {
        let key = Self::record_id_to_string(id);
        let sql = format!("SELECT * FROM {}", key);
        let mut result = self.db.query(sql).await?;
        let value: Option<serde_json::Value> = result.take(0)?;
        let slot: Slot = serde_json::from_value(value.ok_or_else(|| Error::query("Slot not found".to_string(), None))?)
            .map_err(|e| Error::query(format!("Deserialization error: {}", e), None))?;
        Ok(slot)
    }

    /// Updates a slot record in the database.
    ///
    /// # Arguments
    /// * `slot` - The slot to update
    ///
    /// # Returns
    /// * Success or error
    pub async fn update_slot(&self, slot: Slot) -> Result<(), Error> {
        let key = Self::record_id_to_string(&slot.id());
        let sql = format!(
            "UPDATE {} CONTENT {{ starts_at: {}, ends_at: {} }}",
            key,
            slot.starts_at,
            slot.ends_at
        );
        let _: Option<serde_json::Value> = self.db.query(sql).await?.take(0)?;
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
        let key = Self::record_id_to_string(id);
        let sql = format!("DELETE {}", key);
        let _: Option<serde_json::Value> = self.db.query(sql).await?.take(0)?;
        Ok(())
    }
}

// TaskList CRUD methods
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
        let sql = format!(
            "CREATE task_list CONTENT {{ title: '{}' }}",
            title.replace('\'', "''")
        );
        let mut result = self.db.query(sql).await?;
        let value: Option<serde_json::Value> = result.take(0)?;
        let list: TaskList = serde_json::from_value(value.ok_or_else(|| Error::query("Failed to create task list".to_string(), None))?)
            .map_err(|e| Error::query(format!("Deserialization error: {}", e), None))?;
        Ok(list)
    }

    /// Reads a task list record from the database by its ID.
    ///
    /// # Arguments
    /// * `id` - The ID of the task list to read
    ///
    /// # Returns
    /// * The task list
    pub async fn read_task_list(&self, id: &RecordId) -> Result<TaskList, Error> {
        let key = Self::record_id_to_string(id);
        let sql = format!("SELECT * FROM {}", key);
        let mut result = self.db.query(sql).await?;
        let value: Option<serde_json::Value> = result.take(0)?;
        let list: TaskList = serde_json::from_value(value.ok_or_else(|| Error::query("Task list not found".to_string(), None))?)
            .map_err(|e| Error::query(format!("Deserialization error: {}", e), None))?;
        Ok(list)
    }

    /// Updates a task list record in the database.
    ///
    /// # Arguments
    /// * `list` - The task list to update
    ///
    /// # Returns
    /// * Success or error
    pub async fn update_task_list(&self, list: TaskList) -> Result<(), Error> {
        let key = Self::record_id_to_string(&list.id());
        let sql = format!(
            "UPDATE {} CONTENT {{ title: '{}' }}",
            key,
            list.title.replace('\'', "''")
        );
        let _: Option<serde_json::Value> = self.db.query(sql).await?.take(0)?;
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
        let key = Self::record_id_to_string(id);
        let sql = format!("DELETE {}", key);
        let _: Option<serde_json::Value> = self.db.query(sql).await?.take(0)?;
        Ok(())
    }
}

// Relation methods
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
        let value: Option<serde_json::Value> = result.take(0)?;
        let slot: Option<Slot> = value.and_then(|v| serde_json::from_value(v).ok());
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
        let values: Vec<serde_json::Value> = result.take(0).unwrap_or_default();
        let tasks: Vec<Task> = values.into_iter().filter_map(|v| serde_json::from_value(v).ok()).collect();
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
        let values: Vec<serde_json::Value> = result.take(0).unwrap_or_default();
        let tasks: Vec<Task> = values.into_iter().filter_map(|v| serde_json::from_value(v).ok()).collect();
        Ok(tasks)
    }
}

// App blocking methods
impl Storage {
    /// Adds an app to the blocked apps list.
    ///
    /// # Arguments
    /// * `identifier` - The app identifier (path or bundle ID)
    /// * `display_name` - The display name of the app
    ///
    /// # Returns
    /// * The created blocked app record
    pub async fn add_blocked_app(&self, identifier: AppIdentifier, display_name: &str) -> Result<BlockedApp, Error> {
        let id_str = match &identifier {
            AppIdentifier::Path(p) => p.to_string_lossy().to_string(),
            AppIdentifier::BundleId(s) => s.clone(),
        };
        let sql = format!(
            "CREATE blocked_app CONTENT {{ identifier: '{}', display_name: '{}' }}",
            id_str.replace('\'', "''"),
            display_name.replace('\'', "''")
        );
        let mut result = self.db.query(sql).await?;
        let value: Option<serde_json::Value> = result.take(0)?;
        let app: BlockedApp = serde_json::from_value(value.ok_or_else(|| Error::query("Failed to create blocked app".to_string(), None))?)
            .map_err(|e| Error::query(format!("Deserialization error: {}", e), None))?;
        Ok(app)
    }

    /// Removes an app from the blocked apps list.
    ///
    /// # Arguments
    /// * `identifier` - The app identifier to remove
    ///
    /// # Returns
    /// * Success or error
    pub async fn remove_blocked_app(&self, identifier: &AppIdentifier) -> Result<(), Error> {
        let id_str = match identifier {
            AppIdentifier::Path(p) => p.to_string_lossy().to_string(),
            AppIdentifier::BundleId(s) => s.clone(),
        };
        let sql = format!(
            "DELETE FROM blocked_app WHERE identifier = '{}'",
            id_str.replace('\'', "''")
        );
        self.db.query(sql).await?;
        Ok(())
    }

    /// Gets all blocked apps.
    ///
    /// # Returns
    /// * Vector of blocked apps
    pub async fn get_blocked_apps(&self) -> Result<Vec<BlockedApp>, Error> {
        let sql = "SELECT * FROM blocked_app".to_string();
        let mut result = self.db.query(sql).await?;
        let values: Vec<serde_json::Value> = result.take(0).unwrap_or_default();
        let apps: Vec<BlockedApp> = values.into_iter().filter_map(|v| serde_json::from_value(v).ok()).collect();
        Ok(apps)
    }
}

// Focus session methods
impl Storage {
    /// Starts a new focus session.
    ///
    /// # Arguments
    /// * `start` - Session start time
    /// * `end` - Session end time
    /// * `task_id` - Optional task ID associated with the session
    ///
    /// # Returns
    /// * The created focus session
    pub async fn start_session(&self, start: NaiveDateTime, end: NaiveDateTime, task_id: Option<RecordId>) -> Result<FocusSession, Error> {
        let task_id_str = match task_id {
            Some(ref id) => Self::record_id_to_string(id),
            None => "NONE".to_string(),
        };
        let sql = format!(
            "CREATE focus_session CONTENT {{ start_time: {}, end_time: {}, task_id: {} }}",
            start.and_utc().timestamp(),
            end.and_utc().timestamp(),
            task_id_str
        );
        let mut result = self.db.query(sql).await?;
        let value: Option<serde_json::Value> = result.take(0)?;
        let session: FocusSession = serde_json::from_value(value.ok_or_else(|| Error::query("Failed to create focus session".to_string(), None))?)
            .map_err(|e| Error::query(format!("Deserialization error: {}", e), None))?;
        Ok(session)
    }

    /// Ends the current focus session (deletes it).
    ///
    /// # Returns
    /// * Success or error
    pub async fn end_session(&self) -> Result<(), Error> {
        let sql = "DELETE FROM focus_session WHERE end_time > time::now()".to_string();
        self.db.query(sql).await?;
        Ok(())
    }

    /// Gets the active focus session (if any).
    ///
    /// # Returns
    /// * Optional focus session if active
    pub async fn get_active_session(&self) -> Result<Option<FocusSession>, Error> {
        let now = chrono::Utc::now().naive_utc();
        let sql = format!(
            "SELECT * FROM focus_session WHERE start_time <= {} AND end_time > {} LIMIT 1",
            now.and_utc().timestamp(),
            now.and_utc().timestamp()
        );
        let mut result = self.db.query(sql).await?;
        let value: Option<serde_json::Value> = result.take(0)?;
        let session: Option<FocusSession> = value.and_then(|v| serde_json::from_value(v).ok());
        Ok(session)
    }

    /// Checks if blocking is currently active (i.e., there's an active focus session).
    ///
    /// # Returns
    /// * true if blocking is active, false otherwise
    pub async fn is_blocking_active(&self) -> Result<bool, Error> {
        let session = self.get_active_session().await?;
        Ok(session.is_some())
    }
}
