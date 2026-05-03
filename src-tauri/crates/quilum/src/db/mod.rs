use chrono::{NaiveDate, NaiveDateTime, TimeDelta};
use serde::{Deserialize, Serialize};
use surrealdb::{
    engine::local::{Db, Mem, RocksDb},
    types::{RecordId, SurrealValue},
    Error,
    Surreal
};

use crate::{
    model::{
        event::Event,
        slot::Slot,
        task::Task,
        tasklist::TaskList,
    }
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct Record<T> {
    pub(crate) id: RecordId,
    pub(crate) data: T,
}

impl<T: SurrealValue + std::fmt::Debug> SurrealValue for Record<T> {
    fn kind_of() -> surrealdb::types::Kind {
        T::kind_of()
    }

    fn is_value(value: &surrealdb::types::Value) -> bool {
        T::is_value(value)
    }

    fn into_value(self) -> surrealdb::types::Value {
        let mut obj = surrealdb::types::Object::new();
        obj.insert("id", SurrealValue::into_value(self.id));
        obj.insert("data", SurrealValue::into_value(self.data));
        surrealdb::types::Value::Object(obj)
    }

    fn from_value(value: surrealdb::types::Value) -> Result<Self, Error> {
        let obj = if let surrealdb::types::Value::Object(obj) = value {
            obj
        } else {
            return Err(Error::thrown(
                format!("Expected Object, got {:?}", value),
            ));
        };

        let id: RecordId = SurrealValue::from_value(obj.get("id").cloned().unwrap_or_default())?;
        let data: T = SurrealValue::from_value(obj.get("data").cloned().unwrap_or_default())?;

        Ok(Record { id, data })
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct ScheduledTask {
    pub(crate) task: Record<Task>,
    pub(crate) scheduled_for: i64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct SlotWithTasks {
    pub(crate) slot: Record<Slot>,
    pub(crate) tasks: Vec<ScheduledTask>,
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

    /// Base create method that works with any serializable type.
    ///
    /// # Arguments
    /// * `table` - The table name to insert into
    /// * `data` - The data to insert
    ///
    /// # Returns
    /// * The created record with its ID and data
    async fn create_base<T: serde::Serialize + serde::de::DeserializeOwned>(
        &self,
        table: &str,
        data: T,
    ) -> Result<Record<T>, Error> {
        let json_value = serde_json::to_value(&data)
            .map_err(|e| Error::query(format!("Serialization error: {}", e), None))?;
        let created: Option<serde_json::Value> = self.db.create(table).content(json_value).await?;

        let value =
            created.ok_or_else(|| Error::query("Failed to create record".to_string(), None))?;

        let id = Self::extract_record_id_from_value(&value, table);
        let result_data: T = serde_json::from_value(value)
            .map_err(|e| Error::query(format!("Failed to deserialize: {}", e), None))?;

        Ok(Record {
            id,
            data: result_data,
        })
    }

    fn extract_record_id_from_value(value: &serde_json::Value, default_table: &str) -> RecordId {
        match value {
            serde_json::Value::Object(map) => {
                let full_id = map.get("id").and_then(|v| v.as_str()).unwrap_or("unknown");

                // The id field contains the full record ID (e.g., "event:didtm716xkyfjuinldb6")
                // We need to extract just the key part after the table name
                let (table, key) = if let Some(colon_pos) = full_id.find(':') {
                    let t = &full_id[..colon_pos];
                    let k = &full_id[colon_pos + 1..];
                    (t, k)
                } else {
                    (default_table, full_id)
                };

                RecordId::new(table, key)
            }
            _ => RecordId::new(default_table, "unknown"),
        }
    }

    /// Base delete method that works with any type.
    ///
    /// # Arguments
    /// * `table` - The table name to delete from
    /// * `id` - The record ID to delete
    ///
    /// # Returns
    /// * Success or error
    async fn delete_base(&self, table: &str, id: &RecordId) -> Result<(), Error> {
        let key = match &id.key {
            surrealdb::types::RecordIdKey::String(s) => s.as_str(),
            _ => "unknown",
        };
        let _: Option<serde_json::Value> = self.db.delete((table, key)).await?;
        Ok(())
    }

    /// Base read method that works with any deserializable type.
    ///
    /// # Arguments
    /// * `table` - The table name to select from
    /// * `id` - The record ID to read
    ///
    /// # Returns
    /// * The record with its ID and data
    async fn read_base<T: serde::de::DeserializeOwned>(
        &self,
        table: &str,
        id: &RecordId,
    ) -> Result<Record<T>, Error> {
        let key = match &id.key {
            surrealdb::types::RecordIdKey::String(s) => s.as_str(),
            _ => "unknown",
        };
        let value: Option<serde_json::Value> = self.db.select((table, key)).await?;

        let data = value.ok_or_else(|| Error::query("Record not found".to_string(), None))?;
        let parsed: T = serde_json::from_value(data)
            .map_err(|e| Error::query(format!("Failed to deserialize: {}", e), None))?;

        Ok(Record {
            id: id.clone(),
            data: parsed,
        })
    }

    /// Base update method that works with any serializable type.
    ///
    /// # Arguments
    /// * `table` - The table name to update
    /// * `record` - The record containing ID and data to update
    ///
    /// # Returns
    /// * Success or error
    async fn update_base<T: serde::Serialize>(
        &self,
        table: &str,
        record: Record<T>,
    ) -> Result<(), Error> {
        let key = match &record.id.key {
            surrealdb::types::RecordIdKey::String(s) => s.as_str(),
            _ => "unknown",
        };
        let json_value = serde_json::to_value(&record.data)
            .map_err(|e| Error::query(format!("Serialization error: {}", e), None))?;
        let _: Option<serde_json::Value> = self.db.update((table, key)).content(json_value).await?;
        Ok(())
    }
}

impl Storage {
    /// Creates a new event record in the database.
    ///
    /// # Arguments
    /// * `event` - The event data to store
    ///
    /// # Returns
    /// * The created event record with its ID
    pub async fn create_event(&self, event: Event) -> Result<Record<Event>, Error> {
        self.create_base("event", event).await
    }

    /// Reads an event record from the database by its ID.
    ///
    /// # Arguments
    /// * `id` - The ID of the event to read
    ///
    /// # Returns
    /// * The event record with its ID and data
    pub async fn read_event(&self, id: &RecordId) -> Result<Record<Event>, Error> {
        self.read_base("event", id).await
    }

    /// Updates an event record in the database.
    ///
    /// # Arguments
    /// * `record` - The event record containing ID and updated data
    ///
    /// # Returns
    /// * Success or error
    pub async fn update_event(&self, record: Record<Event>) -> Result<(), Error> {
        self.update_base("event", record).await
    }

    /// Deletes an event record from the database by its ID.
    ///
    /// # Arguments
    /// * `id` - The ID of the event to delete
    ///
    /// # Returns
    /// * Success or error
    pub async fn delete_event(&self, id: &RecordId) -> Result<(), Error> {
        self.delete_base("event", id).await
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
    /// * Vector of event records occurring in the date range
    pub async fn get_events_for_date_range(
        &self,
        start: NaiveDate,
        end: NaiveDate,
    ) -> Result<Vec<Record<Event>>, Error> {
        let range_start = start.and_hms_opt(0, 0, 0).unwrap().and_utc().timestamp();
        let range_end = end.and_hms_opt(0, 0, 0).unwrap().and_utc().timestamp();

        let sql = format!(
            "SELECT * FROM event WHERE starts_at < {} AND ends_at > {}",
            range_end, range_start
        );
        let mut result = self.db.query(sql).await?;
        
        // Take raw JSON values and convert to Record<Event>
        let values: Vec<serde_json::Value> = result.take(0).unwrap_or_default();
        
        let events: Vec<Record<Event>> = values
            .into_iter()
            .filter_map(|value| {
                // Use existing helper to extract RecordId
                let id = Self::extract_record_id_from_value(&value, "event");
                
                // Get the data by removing id and deserializing the rest
                if let serde_json::Value::Object(mut obj) = value {
                    obj.remove("id");
                    let data: Event = serde_json::from_value(serde_json::Value::Object(obj)).ok()?;
                    Some(Record { id, data })
                } else {
                    None
                }
            })
            .collect();
        
        Ok(events)
    }

    /// Gets events occurring on a specific date.
    /// Events that overlap with the date are returned (even if they span multiple days).
    ///
    /// # Arguments
    /// * `date` - The date to query
    ///
    /// # Returns
    /// * Vector of event records occurring on the date
    pub async fn get_events_for_date(
        &self,
        date: NaiveDate,
    ) -> Result<Vec<Record<Event>>, Error> {
        let next_day = date + TimeDelta::days(1);
        self.get_events_for_date_range(date, next_day).await
    }
}

impl Storage {
    /// Creates a new task record in the database.
    ///
    /// # Arguments
    /// * `task` - The task data to store
    ///
    /// # Returns
    /// * The created task record with its ID
    pub async fn create_task(&self, task: Task) -> Result<Record<Task>, Error> {
        self.create_base("task", task).await
    }

    /// Reads a task record from the database by its ID.
    ///
    /// # Arguments
    /// * `id` - The ID of the task to read
    ///
    /// # Returns
    /// * The task record with its ID and data
    pub async fn read_task(&self, id: &RecordId) -> Result<Record<Task>, Error> {
        self.read_base("task", id).await
    }

    /// Updates a task record in the database.
    ///
    /// # Arguments
    /// * `record` - The task record containing ID and updated data
    ///
    /// # Returns
    /// * Success or error
    pub async fn update_task(&self, record: Record<Task>) -> Result<(), Error> {
        self.update_base("task", record).await
    }

    /// Deletes a task record from the database by its ID.
    ///
    /// # Arguments
    /// * `id` - The ID of the task to delete
    ///
    /// # Returns
    /// * Success or error
    pub async fn delete_task(&self, id: &RecordId) -> Result<(), Error> {
        self.delete_base("task", id).await
    }
}

impl Storage {
    /// Creates a new slot record in the database.
    ///
    /// # Arguments
    /// * `slot` - The slot data to store
    ///
    /// # Returns
    /// * The created slot record with its ID
    pub async fn create_slot(&self, slot: Slot) -> Result<Record<Slot>, Error> {
        self.create_base("slot", slot).await
    }

    /// Reads a slot record from the database by its ID.
    ///
    /// # Arguments
    /// * `id` - The ID of the slot to read
    ///
    /// # Returns
    /// * The slot record with its ID and data
    pub async fn read_slot(&self, id: &RecordId) -> Result<Record<Slot>, Error> {
        self.read_base("slot", id).await
    }

    /// Updates a slot record in the database.
    ///
    /// # Arguments
    /// * `record` - The slot record containing ID and updated data
    ///
    /// # Returns
    /// * Success or error
    pub async fn update_slot(&self, record: Record<Slot>) -> Result<(), Error> {
        self.update_base("slot", record).await
    }

    /// Deletes a slot record from the database by its ID.
    ///
    /// # Arguments
    /// * `id` - The ID of the slot to delete
    ///
    /// # Returns
    /// * Success or error
    pub async fn delete_slot(&self, id: &RecordId) -> Result<(), Error> {
        self.delete_base("slot", id).await
    }
}

impl Storage {
    /// Creates a new task list record in the database.
    ///
    /// # Arguments
    /// * `list` - The task list data to store
    ///
    /// # Returns
    /// * The created task list record with its ID
    pub async fn create_task_list(&self, list: TaskList) -> Result<Record<TaskList>, Error> {
        self.create_base("task_list", list).await
    }

    /// Reads a task list record from the database by its ID.
    ///
    /// # Arguments
    /// * `id` - The ID of the task list to read
    ///
    /// # Returns
    /// * The task list record with its ID and data
    pub async fn read_task_list(&self, id: &RecordId) -> Result<Record<TaskList>, Error> {
        self.read_base("task_list", id).await
    }

    /// Updates a task list record in the database.
    ///
    /// # Arguments
    /// * `record` - The task list record containing ID and updated data
    ///
    /// # Returns
    /// * Success or error
    pub async fn update_task_list(&self, record: Record<TaskList>) -> Result<(), Error> {
        self.update_base("task_list", record).await
    }

    /// Deletes a task list record from the database by its ID.
    ///
    /// # Arguments
    /// * `id` - The ID of the task list to delete
    ///
    /// # Returns
    /// * Success or error
    pub async fn delete_task_list(&self, id: &RecordId) -> Result<(), Error> {
        self.delete_base("task_list", id).await
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
            "RELATE {}->scheduled_in->{} SET scheduled_for = {}",
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
    /// * The slot record if found, None if not scheduled
    pub async fn get_slot_for_task(
        &self,
        task_id: &RecordId,
    ) -> Result<Option<Record<Slot>>, Error> {
        let sql = format!(
            "SELECT * FROM ONLY slot WHERE id IN (SELECT out FROM scheduled_in WHERE in = {}) LIMIT 1",
            Self::record_id_to_string(task_id)
        );
        let mut result = self.db.query(sql).await?;
        let slot: Option<Record<Slot>> = result.take(0)?;
        Ok(slot)
    }

    /// Gets all tasks in a slot.
    ///
    /// # Arguments
    /// * `slot_id` - The slot record ID
    ///
    /// # Returns
    /// * The task records in the slot
    pub async fn get_tasks_in_slot(
        &self,
        slot_id: &RecordId,
    ) -> Result<Vec<Record<Task>>, Error> {
        let sql = format!(
            "SELECT * FROM task WHERE id IN (SELECT in FROM scheduled_in WHERE out = {})",
            Self::record_id_to_string(slot_id)
        );
        let mut result = self.db.query(sql).await?;
        let tasks: Vec<Record<Task>> = result.take(0)?;
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
    /// * The task records in the list
    pub async fn get_tasks_in_list(
        &self,
        list_id: &RecordId,
    ) -> Result<Vec<Record<Task>>, Error> {
        let sql = format!(
            "SELECT * FROM task WHERE id IN (SELECT in FROM belongs_to WHERE out = {})",
            Self::record_id_to_string(list_id)
        );
        let mut result = self.db.query(sql).await?;
        let tasks: Vec<Record<Task>> = result.take(0)?;
        Ok(tasks)
    }

    /// Helper to convert RecordId to string for SQL queries
    fn record_id_to_string(id: &RecordId) -> String {
        match &id.key {
            surrealdb::types::RecordIdKey::String(s) => format!("{}:{}", id.table, s),
            _ => format!("{}:unknown", id.table),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    #[tokio::test]
    async fn test_storage_mem_creation() {
        let _storage = Storage::new_mem().await.expect("Failed to create storage");
    }

    #[tokio::test]
    async fn test_event_crud() {
        let storage = Storage::new_mem().await.expect("Failed to create storage");

        let start_time = NaiveDate::from_ymd_opt(2026, 5, 1)
            .unwrap()
            .and_hms_opt(10, 0, 0)
            .unwrap();
        let end_time = NaiveDate::from_ymd_opt(2026, 5, 1)
            .unwrap()
            .and_hms_opt(12, 0, 0)
            .unwrap();

        let event = Event::new(
            "Test Event".to_string(),
            "A test event".to_string(),
            start_time,
            end_time,
        );

        let record = storage
            .create_event(event)
            .await
            .expect("Failed to create event");
        let key_is_set =
            matches!(&record.id.key, surrealdb::types::RecordIdKey::String(s) if !s.is_empty());
        assert!(key_is_set, "Record ID key should be set");

        let read_record = storage
            .read_event(&record.id)
            .await
            .expect("Failed to read event");
        assert_eq!(read_record.data.name(), "Test Event");

        let mut updated_event = read_record.data;
        updated_event.set_name("Updated Event".to_string());
        let updated_record = Record {
            id: read_record.id.clone(),
            data: updated_event,
        };
        storage
            .update_event(updated_record)
            .await
            .expect("Failed to update event");

        let updated_read_record = storage
            .read_event(&record.id)
            .await
            .expect("Failed to read updated event");
        assert_eq!(updated_read_record.data.name(), "Updated Event");

        storage
            .delete_event(&record.id)
            .await
            .expect("Failed to delete event");

        let result = storage.read_event(&record.id).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_get_events_for_date_range_basic() {
        let storage = Storage::new_mem().await.expect("Failed to create storage");

        let date = NaiveDate::from_ymd_opt(2026, 5, 1).unwrap();

        // Create events on different dates
        let event_a = Event::new(
            "Event A".to_string(),
            "On May 1".to_string(),
            date.and_hms_opt(10, 0, 0).unwrap(),
            date.and_hms_opt(12, 0, 0).unwrap(),
        );
        let event_b = Event::new(
            "Event B".to_string(),
            "On May 3".to_string(),
            NaiveDate::from_ymd_opt(2026, 5, 3).unwrap().and_hms_opt(10, 0, 0).unwrap(),
            NaiveDate::from_ymd_opt(2026, 5, 3).unwrap().and_hms_opt(12, 0, 0).unwrap(),
        );
        let event_c = Event::new(
            "Event C".to_string(),
            "On May 5".to_string(),
            NaiveDate::from_ymd_opt(2026, 5, 5).unwrap().and_hms_opt(10, 0, 0).unwrap(),
            NaiveDate::from_ymd_opt(2026, 5, 5).unwrap().and_hms_opt(12, 0, 0).unwrap(),
        );

        storage.create_event(event_a).await.expect("Failed to create event A");
        storage.create_event(event_b).await.expect("Failed to create event B");
        storage.create_event(event_c).await.expect("Failed to create event C");

        // Query range: May 2 to May 4 (should only return event B)
        let start = NaiveDate::from_ymd_opt(2026, 5, 2).unwrap();
        let end = NaiveDate::from_ymd_opt(2026, 5, 4).unwrap();
        let events = storage.get_events_for_date_range(start, end).await.expect("Failed to query events");

        assert_eq!(events.len(), 1, "Should return exactly 1 event");
        assert_eq!(events[0].data.name(), "Event B");
    }

    #[tokio::test]
    async fn test_get_events_for_date_range_overlapping() {
        let storage = Storage::new_mem().await.expect("Failed to create storage");

        // Create event that spans multiple days: May 1 20:00 to May 3 06:00
        let event = Event::new(
            "Multi-day Event".to_string(),
            "Spans 3 days".to_string(),
            NaiveDate::from_ymd_opt(2026, 5, 1).unwrap().and_hms_opt(20, 0, 0).unwrap(),
            NaiveDate::from_ymd_opt(2026, 5, 3).unwrap().and_hms_opt(6, 0, 0).unwrap(),
        );
        storage.create_event(event).await.expect("Failed to create event");

        // Query for May 2 (middle day) - should return the event
        let date = NaiveDate::from_ymd_opt(2026, 5, 2).unwrap();
        let events = storage.get_events_for_date(date).await.expect("Failed to query events");

        assert_eq!(events.len(), 1, "Should return the overlapping event");
        assert_eq!(events[0].data.name(), "Multi-day Event");
    }

    #[tokio::test]
    async fn test_get_events_for_date_single() {
        let storage = Storage::new_mem().await.expect("Failed to create storage");

        let date1 = NaiveDate::from_ymd_opt(2026, 5, 1).unwrap();
        let date2 = NaiveDate::from_ymd_opt(2026, 5, 2).unwrap();
        let date3 = NaiveDate::from_ymd_opt(2026, 5, 3).unwrap();

        // Create events on different dates
        let event1 = Event::new(
            "Event 1".to_string(),
            "On May 1".to_string(),
            date1.and_hms_opt(10, 0, 0).unwrap(),
            date1.and_hms_opt(12, 0, 0).unwrap(),
        );
        let event2 = Event::new(
            "Event 2".to_string(),
            "On May 2".to_string(),
            date2.and_hms_opt(10, 0, 0).unwrap(),
            date2.and_hms_opt(12, 0, 0).unwrap(),
        );
        let event3 = Event::new(
            "Event 3".to_string(),
            "On May 3".to_string(),
            date3.and_hms_opt(10, 0, 0).unwrap(),
            date3.and_hms_opt(12, 0, 0).unwrap(),
        );

        storage.create_event(event1).await.expect("Failed to create event 1");
        storage.create_event(event2).await.expect("Failed to create event 2");
        storage.create_event(event3).await.expect("Failed to create event 3");

        // Query for May 2 only
        let events = storage.get_events_for_date(date2).await.expect("Failed to query events");

        assert_eq!(events.len(), 1, "Should return exactly 1 event");
        assert_eq!(events[0].data.name(), "Event 2");
    }

    #[tokio::test]
    async fn test_get_events_for_date_range_empty() {
        let storage = Storage::new_mem().await.expect("Failed to create storage");

        // Query range with no events
        let start = NaiveDate::from_ymd_opt(2026, 5, 10).unwrap();
        let end = NaiveDate::from_ymd_opt(2026, 5, 20).unwrap();
        let events = storage.get_events_for_date_range(start, end).await.expect("Failed to query events");

        assert!(events.is_empty(), "Should return empty vec");
    }

    #[tokio::test]
    async fn test_get_events_for_date_multiple_same_day() {
        let storage = Storage::new_mem().await.expect("Failed to create storage");

        let date = NaiveDate::from_ymd_opt(2026, 5, 1).unwrap();

        // Create 3 events on the same day
        for i in 1..=3 {
            let event = Event::new(
                format!("Event {}", i),
                format!("On May 1, event {}", i),
                date.and_hms_opt(i as u32 * 2, 0, 0).unwrap(),
                date.and_hms_opt(i as u32 * 2 + 1, 0, 0).unwrap(),
            );
            storage.create_event(event).await.expect("Failed to create event");
        }

        // Query for May 1
        let events = storage.get_events_for_date(date).await.expect("Failed to query events");

        assert_eq!(events.len(), 3, "Should return all 3 events");
    }
}
