use crate::{
    app_identifier::AppIdentifier,
    blocked_app::BlockedApp,
    event::Event,
    focus_session::FocusSession,
    slot::Slot,
    task::{Priority, Task},
    tasklist::TaskList,
};
use chrono::{NaiveDate, NaiveDateTime, TimeDelta};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use surrealdb::{
    Error, Surreal,
    engine::local::{Db, Mem, RocksDb},
    types::{RecordId, SurrealValue},
};

/// Struct for returning slots with their scheduled tasks
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SlotWithTasks {
    pub slot: Slot,
    pub tasks: Vec<(Task, i64)>,
}

/// Struct for returning task lists with their tasks
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TaskListWithTasks {
    pub list: TaskList,
    pub tasks: Vec<Task>,
}

/// Storage struct that holds a handle to a SurrealDB instance
/// and exposes CRUD methods for events, tasks, and app blocking.
#[derive(Clone)]
pub struct Storage {
    db: Surreal<Db>,
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
    pub async fn add_blocked_app(
        &self,
        identifier: AppIdentifier,
        display_name: &str,
    ) -> Result<BlockedApp, Error> {
        let id_str = match identifier {
            AppIdentifier::Path(p) => p.to_string_lossy().to_string(),
            AppIdentifier::BundleId(s) => s,
        };
        let data = serde_json::json!({
            "identifier": id_str,
            "display_name": display_name
        });
        let created: Option<BlockedApp> = self.db.create("blocked_app").content(data).await?;
        created.ok_or_else(|| Error::query("Failed to create blocked app".to_string(), None))
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
        let apps: Vec<BlockedApp> = result.take(0).unwrap_or_default();
        Ok(apps)
    }

    /// Upserts (creates or updates) a blocked app.
    ///
    /// # Arguments
    /// * `identifier` - The app identifier (path or bundle ID)
    /// * `display_name` - The display name of the app
    ///
    /// # Returns
    /// * The upserted blocked app record
    pub async fn upsert_blocked_app(
        &self,
        identifier: AppIdentifier,
        display_name: &str,
    ) -> Result<BlockedApp, Error> {
        let id_str = match &identifier {
            AppIdentifier::Path(p) => p.to_string_lossy().to_string(),
            AppIdentifier::BundleId(s) => s.clone(),
        };

        // First check if record exists by querying
        let apps = self.get_blocked_apps().await?;
        let existing = apps.iter().find(|app| app.identifier == id_str);

        if let Some(existing_app) = existing {
            // Record exists - update it using query builder
            let key = Self::record_id_key(&existing_app.id);
            let data = serde_json::json!({
                "identifier": id_str,
                "display_name": display_name
            });
            let updated: Option<BlockedApp> =
                self.db.update(("blocked_app", key)).content(data).await?;
            return updated
                .ok_or_else(|| Error::query("Failed to update blocked app".to_string(), None));
        } else {
            // Record doesn't exist - create new one using query builder
            let data = serde_json::json!({
                "identifier": id_str,
                "display_name": display_name
            });
            let created: Option<BlockedApp> = self.db.create("blocked_app").content(data).await?;
            return created
                .ok_or_else(|| Error::query("Failed to create blocked app".to_string(), None));
        }
    }

    /// Deletes all blocked apps (clears the table).
    ///
    /// # Returns
    /// * Success or error
    pub async fn delete_all_blocked_apps(&self) -> Result<(), Error> {
        self.db.query("DELETE FROM blocked_app").await?;
        Ok(())
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
    pub async fn start_session(
        &self,
        start: NaiveDateTime,
        end: NaiveDateTime,
        task_id: Option<RecordId>,
    ) -> Result<FocusSession, Error> {
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
        let session: FocusSession = serde_json::from_value(
            value
                .ok_or_else(|| Error::query("Failed to create focus session".to_string(), None))?,
        )
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

    /// Initialize the database with required schema (indexes, etc.)
    /// Runs the init.surql script included at compile time.
    pub async fn init(&self) -> Result<(), Error> {
        let init_script = include_str!("../../resources/init.surql");
        self.db.query(init_script).await?;
        Ok(())
    }

    /// Creates a new Storage instance using in-memory database mode.
    ///
    /// # Returns
    /// * The storage instance or an error
    pub async fn new_mem() -> Result<Self, Error> {
        let db = Surreal::new::<Mem>(()).await?;
        db.use_ns("test").use_db("test").await?;
        let storage = Self::new(db)?;
        storage.init().await?;
        Ok(storage)
    }

    /// Creates a new Storage instance using RocksDB database mode.
    /// Uses platform-specific data directory via ProjectDirs.
    ///
    /// # Returns
    /// * The storage instance or an error
    pub async fn new_rocksdb() -> Result<Self, Error> {
        let proj_dirs = ProjectDirs::from("com", "quilum", "quilum")
            .expect("Failed to get project directories");
        let data_dir = proj_dirs.data_dir().to_path_buf();
        std::fs::create_dir_all(&data_dir).expect("Failed to create data directory");
        let db_path = data_dir.join("quilum.db");
        let db = Surreal::new::<RocksDb>(db_path).await?;
        db.use_ns("quilum").use_db("main").await?;
        let storage = Self::new(db)?;
        storage.init().await?;
        Ok(storage)
    }

    // new_ws() removed - use new_rocksdb() on all platforms for now
    // TODO: Implement WebSocket connection with proper type handling

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
    pub async fn get_events_for_date(&self, date: NaiveDate) -> Result<Vec<Event>, Error> {
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
            if let (Some(task_value), Some(scheduled_for)) =
                (item.get("out"), item.get("scheduled_for"))
            {
                if let (Some(task_obj), Some(sf)) = (task_value.as_object(), scheduled_for.as_i64())
                {
                    // Convert the task object to proper JSON, handling id field
                    let mut task_json = serde_json::Map::new();
                    for (k, v) in task_obj {
                        if k == "id" {
                            // id is a string like "task:xxx", convert to object format for RecordId
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
                            // Handle priority enum - extract the variant name
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
                    scheduled_tasks.push((task, sf));
                }
            }
        }
        Ok(scheduled_tasks)
    }

    /// Gets all slots within a date range along with their scheduled tasks.
    /// Returns ALL slots (including empty ones) grouped with their tasks.
    ///
    /// # Arguments
    /// * `start` - The start date (inclusive, at 00:00:00)
    /// * `end` - The end date (exclusive, at 00:00:00)
    ///
    /// # Returns
    /// * Vector of slots with their scheduled tasks
    pub async fn get_slots_with_tasks_for_date_range(
        &self,
        start: NaiveDate,
        end: NaiveDate,
    ) -> Result<Vec<SlotWithTasks>, Error> {
        let range_start = start.and_hms_opt(0, 0, 0).unwrap().and_utc().timestamp();
        let range_end = end.and_hms_opt(0, 0, 0).unwrap().and_utc().timestamp();

        // Query using graph syntax to get slots + their tasks in one query
        let sql = format!(
            "SELECT *, ->(SELECT out.*, scheduled_for FROM contains) AS tasks \
             FROM slot WHERE starts_at < {} AND ends_at > {}",
            range_end, range_start
        );
        let mut result = self.db.query(sql).await?;
        let slot_values: Vec<serde_json::Value> = result.take(0).unwrap_or_default();

        let mut slots_with_tasks: Vec<SlotWithTasks> = Vec::new();

        for mut slot_value in slot_values {
            // === Parse slot ===
            let _slot_id = {
                let id_str = slot_value
                    .get("id")
                    .and_then(|v| v.as_str())
                    .unwrap_or_default();
                let parts: Vec<&str> = id_str.split(':').collect();
                if parts.len() == 2 {
                    RecordId::new(parts[0], parts[1])
                } else {
                    continue;
                }
            };

            // Clone tasks BEFORE removing from slot_value
            let tasks_value = slot_value.get("tasks").cloned().unwrap_or_default();

            // Remove non-Slot fields (but keep id for deserialization)
            if let Some(obj) = slot_value.as_object_mut() {
                obj.remove("tasks");
            }

            // Convert the slot object to proper JSON, handling id field
            let mut slot_json = serde_json::Map::new();
            if let Some(obj) = slot_value.as_object() {
                for (k, v) in obj {
                    if k == "id" {
                        // id is a string like "slot:xxx", convert to object format for RecordId
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
                                slot_json
                                    .insert("id".to_string(), serde_json::Value::Object(id_obj));
                            }
                        }
                    } else {
                        slot_json.insert(k.clone(), v.clone());
                    }
                }
            }

            let slot: Slot = serde_json::from_value(serde_json::Value::Object(slot_json))
                .map_err(|e| Error::query(format!("Failed to deserialize slot: {}", e), None))?;

            // === Parse tasks ===
            let tasks: Vec<(Task, i64)> = if let serde_json::Value::Array(arr) = tasks_value {
                arr.into_iter()
                    .filter_map(|task_item| {
                        // task_item = {"out": {...}, "scheduled_for": 123...}

                        // Extract scheduled_for from top level
                        let scheduled_for =
                            task_item.get("scheduled_for").and_then(|v| v.as_i64())?;

                        // Extract "out" object which contains task data
                        let out_value = task_item.get("out").cloned()?;
                        let out_obj = if let serde_json::Value::Object(obj) = out_value {
                            obj
                        } else {
                            return None;
                        };

                        // Convert the task object to proper JSON, handling id field
                        let mut task_json = serde_json::Map::new();
                        for (k, v) in &out_obj {
                            if k == "id" {
                                // id is a string like "task:xxx", convert to object format for RecordId
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
                                // Handle priority enum - extract the variant name
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

                        let task: Task =
                            serde_json::from_value(serde_json::Value::Object(task_json)).ok()?;

                        Some((task, scheduled_for))
                    })
                    .collect()
            } else {
                vec![]
            };

            slots_with_tasks.push(SlotWithTasks { slot, tasks });
        }

        Ok(slots_with_tasks)
    }

    /// Gets today's timetable: events + scheduled tasks for today view.
    ///
    /// # Arguments
    /// * `today` - The date to get timetable for
    ///
    /// # Returns
    /// * Tuple of (events, scheduled_tasks) for today
    pub async fn get_today_timetable(
        &self,
        today: NaiveDate,
    ) -> Result<(Vec<Event>, Vec<(Task, i64)>), Error> {
        let tomorrow = today + TimeDelta::days(1);

        let events = self.get_events_for_date(today).await?;
        let scheduled_tasks = self
            .get_scheduled_tasks_for_date_range(today, tomorrow)
            .await?;

        Ok((events, scheduled_tasks))
    }

    /// Gets week's timetable: events + slots with tasks for week view.
    /// Derives week_end as week_start + 7 days (half-open range).
    ///
    /// # Arguments
    /// * `week_start` - The first day of the week
    ///
    /// # Returns
    /// * Tuple of (events, slots_with_tasks) for the week
    pub async fn get_week_timetable(
        &self,
        week_start: NaiveDate,
    ) -> Result<(Vec<Event>, Vec<SlotWithTasks>), Error> {
        let week_end = week_start + TimeDelta::days(7);

        let events = self.get_events_for_date_range(week_start, week_end).await?;
        let slots_with_tasks = self
            .get_slots_with_tasks_for_date_range(week_start, week_end)
            .await?;

        Ok((events, slots_with_tasks))
    }

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

    /// Gets all uncompleted tasks that are not overdue.
    ///
    /// # Returns
    /// * Vector of tasks where completed = false AND deadline > now
    pub async fn get_uncompleted_tasks(&self) -> Result<Vec<Task>, Error> {
        let now = chrono::Utc::now().naive_utc().and_utc().timestamp();
        let sql = format!(
            "SELECT * FROM task WHERE completed = false AND deadline > {}",
            now
        );
        let mut result = self.db.query(sql).await?;
        let tasks: Vec<Task> = result.take(0)?;
        Ok(tasks)
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
    let sql = format!(
        "DELETE FROM contains WHERE out = {}",
        Self::record_id_to_string(id)
    );
    self.db.query(sql).await?;
    
    let key = Self::record_id_key(id);
    let _: Option<Slot> = self.db.delete(("slot", key)).await?;
    Ok(())
}

    /// Gets all future slots (slots that haven't ended yet).
    ///
    /// # Returns
    /// * Vector of slots where ends_at > now
    pub async fn get_future_slots(&self) -> Result<Vec<Slot>, Error> {
        let now = chrono::Utc::now().naive_utc().and_utc().timestamp();
        let sql = format!(
            "SELECT * FROM slot WHERE ends_at > {} ORDER BY starts_at ASC",
            now
        );
        let mut result = self.db.query(sql).await?;
        let slots: Vec<Slot> = result.take(0)?;
        Ok(slots)
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
}

#[cfg(test)]
mod tests;
