use chrono::{DateTime, Utc};
use surrealdb::{types::RecordId, Error};

use crate::{model::slot::Slot, slot::SLOTS_TABLE, Storage};

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
        starts_at: DateTime<Utc>,
        ends_at: DateTime<Utc>,
    ) -> Result<Slot, Error> {
        let slot = Slot::new(starts_at, ends_at);

        let created: Option<Slot> = self.db.create(SLOTS_TABLE).content(slot).await?;
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
        let slot: Option<Slot> = self.db.select(id).await?;
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
        let _: Option<Slot> = self.db.update(slot.id()).content(slot).await?;
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
        let _: Option<Slot> = self.db.delete(id).await?;
        Ok(())
    }
}
