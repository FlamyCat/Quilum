use surrealdb::engine::local::{Db, Mem, RocksDb};
use surrealdb::Surreal;
use surrealdb::types::RecordId;
use surrealdb::Error;

use crate::model::event::Event;
use crate::model::task::Task;

pub(crate) struct Record<T> {
    pub(crate) id: RecordId,
    pub(crate) data: T,
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
    /// * `Result<Self, Error>` - The storage instance or an error
    pub fn new(db: Surreal<Db>) -> Result<Self, Error> {
        Ok(Self { db })
    }

    /// Creates a new Storage instance using in-memory database mode.
    ///
    /// # Returns
    /// * `Result<Storage, Error>` - The storage instance or an error
    pub async fn new_mem() -> Result<Self, Error> {
        let db = Surreal::new::<Mem>(()).await?;
        db.use_ns("test").use_db("test").await;
        Ok(Self::new(db)?)
    }

    /// Creates a new Storage instance using RocksDB database mode.
    ///
    /// # Arguments
    /// * `path` - File path where the RocksDB database will be stored
    ///
    /// # Returns
    /// * `Result<Storage, Error>` - The storage instance or an error
    pub async fn new_rocksdb(path: &str) -> Result<Self, Error> {
        let db = Surreal::new::<RocksDb>(path).await?;
        db.use_ns("test").use_db("test").await;
        Ok(Self::new(db)?)
    }

    /// Base create method that works with any serializable type.
    ///
    /// # Arguments
    /// * `table` - The table name to insert into
    /// * `data` - The data to insert
    ///
    /// # Returns
    /// * `Result<Record<T>, Error>` - The created record with its ID and data
    async fn create_base<T: serde::Serialize + serde::de::DeserializeOwned>(
        &self,
        table: &str,
        data: T,
    ) -> Result<Record<T>, Error> {
        let json_value = serde_json::to_value(&data).map_err(|e| Error::query(format!("Serialization error: {}", e), None))?;
        let created: Option<serde_json::Value> = self.db.create(table).content(json_value).await?;

        let value = created.ok_or_else(|| Error::query("Failed to create record".to_string(), None))?;

        let id = RecordId::new(table, "auto");
        let result_data: T = serde_json::from_value(value).map_err(|e| Error::query(format!("Failed to deserialize: {}", e), None))?;

        Ok(Record { id, data: result_data })
    }

    /// Base delete method that works with any type.
    ///
    /// # Arguments
    /// * `table` - The table name to delete from
    /// * `id` - The record ID to delete
    ///
    /// # Returns
    /// * `Result<(), Error>` - Success or error
    async fn delete_base(
        &self,
        table: &str,
        id: &RecordId,
    ) -> Result<(), Error> {
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
    /// * `Result<Record<T>, Error>` - The record with its ID and data
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
        let parsed: T = serde_json::from_value(data).map_err(|e| Error::query(format!("Failed to deserialize: {}", e), None))?;

        Ok(Record { id: id.clone(), data: parsed })
    }

    /// Base update method that works with any serializable type.
    ///
    /// # Arguments
    /// * `table` - The table name to update
    /// * `record` - The record containing ID and data to update
    ///
    /// # Returns
    /// * `Result<(), Error>` - Success or error
    async fn update_base<T: serde::Serialize>(
        &self,
        table: &str,
        record: Record<T>,
    ) -> Result<(), Error> {
        let key = match &record.id.key {
            surrealdb::types::RecordIdKey::String(s) => s.as_str(),
            _ => "unknown",
        };
        let json_value = serde_json::to_value(&record.data).map_err(|e| Error::query(format!("Serialization error: {}", e), None))?;
        let _: Option<serde_json::Value> = self.db.update((table, key))
            .content(json_value)
            .await?;
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
    /// * `Result<Record<Event>, Error>` - The created event record with its ID
    pub async fn create_event(&self, event: Event) -> Result<Record<Event>, Error> {
        self.create_base("event", event).await
    }

    /// Reads an event record from the database by its ID.
    ///
    /// # Arguments
    /// * `id` - The ID of the event to read
    ///
    /// # Returns
    /// * `Result<Record<Event>, Error>` - The event record with its ID and data
    pub async fn read_event(&self, id: &RecordId) -> Result<Record<Event>, Error> {
        self.read_base("event", id).await
    }

    /// Updates an event record in the database.
    ///
    /// # Arguments
    /// * `record` - The event record containing ID and updated data
    ///
    /// # Returns
    /// * `Result<(), Error>` - Success or error
    pub async fn update_event(&self, record: Record<Event>) -> Result<(), Error> {
        self.update_base("event", record).await
    }

    /// Deletes an event record from the database by its ID.
    ///
    /// # Arguments
    /// * `id` - The ID of the event to delete
    ///
    /// # Returns
    /// * `Result<(), Error>` - Success or error
    pub async fn delete_event(&self, id: &RecordId) -> Result<(), Error> {
        self.delete_base("event", id).await
    }
}

impl Storage {
    /// Creates a new task record in the database.
    ///
    /// # Arguments
    /// * `task` - The task data to store
    ///
    /// # Returns
    /// * `Result<Record<Task>, Error>` - The created task record with its ID
    pub async fn create_task(&self, task: Task) -> Result<Record<Task>, Error> {
        self.create_base("task", task).await
    }

    /// Reads a task record from the database by its ID.
    ///
    /// # Arguments
    /// * `id` - The ID of the task to read
    ///
    /// # Returns
    /// * `Result<Record<Task>, Error>` - The task record with its ID and data
    pub async fn read_task(&self, id: &RecordId) -> Result<Record<Task>, Error> {
        self.read_base("task", id).await
    }

    /// Updates a task record in the database.
    ///
    /// # Arguments
    /// * `record` - The task record containing ID and updated data
    ///
    /// # Returns
    /// * `Result<(), Error>` - Success or error
    pub async fn update_task(&self, record: Record<Task>) -> Result<(), Error> {
        self.update_base("task", record).await
    }

    /// Deletes a task record from the database by its ID.
    ///
    /// # Arguments
    /// * `id` - The ID of the task to delete
    ///
    /// # Returns
    /// * `Result<(), Error>` - Success or error
    pub async fn delete_task(&self, id: &RecordId) -> Result<(), Error> {
        self.delete_base("task", id).await
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

        let start_time = NaiveDate::from_ymd_opt(2026, 5, 1).unwrap()
            .and_hms_opt(10, 0, 0).unwrap();
        let end_time = NaiveDate::from_ymd_opt(2026, 5, 1).unwrap()
            .and_hms_opt(12, 0, 0).unwrap();

        let event = Event::new(
            "Test Event".to_string(),
            "A test event".to_string(),
            start_time,
            end_time,
        );

        let record = storage.create_event(event).await.expect("Failed to create event");
        let key_is_set = matches!(&record.id.key, surrealdb::types::RecordIdKey::String(s) if !s.is_empty());
        assert!(key_is_set, "Record ID key should be set");

        let read_record = storage.read_event(&record.id).await.expect("Failed to read event");
        assert_eq!(read_record.data.name(), "Test Event");

        let mut updated_event = read_record.data;
        updated_event.set_name("Updated Event".to_string());
        let updated_record = Record { id: read_record.id.clone(), data: updated_event };
        storage.update_event(updated_record).await.expect("Failed to update event");

        let updated_read_record = storage.read_event(&record.id).await.expect("Failed to read updated event");
        assert_eq!(updated_read_record.data.name(), "Updated Event");

        storage.delete_event(&record.id).await.expect("Failed to delete event");

        let result = storage.read_event(&record.id).await;
        assert!(result.is_err());
    }
}
