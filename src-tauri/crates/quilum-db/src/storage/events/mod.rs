use crate::{Storage, event::Event};
use chrono::NaiveDateTime;
use surrealdb::{Error, types::RecordId};

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

#[cfg(test)]
mod tests;
