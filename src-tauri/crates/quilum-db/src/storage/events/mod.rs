use chrono::NaiveDateTime;
use surrealdb::{types::RecordId, Error};

use crate::{
    model::event::{Event, EVENTS_TABLE},
    Storage,
};

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
        let event = Event::new(name, description, starts_at.and_utc(), ends_at.and_utc());

        let created = self.db.create(EVENTS_TABLE).content(event).await?;

        Ok(created.expect("SurrealDB is not explicitly instructed to return None, so the operation result is expected to be Some(...)"))
    }

    /// Reads an event record from the database by its ID.
    ///
    /// # Arguments
    /// * `id` - The ID of the event to read
    ///
    /// # Returns
    /// * The event
    pub async fn read_event(&self, id: &RecordId) -> Result<Event, Error> {
        let event = self.db.select(id).await?;
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
        let _: Option<Event> = self.db.update(&event.id).content(event).await?;
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
        let _: Option<Event> = self.db.delete(id).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests;
