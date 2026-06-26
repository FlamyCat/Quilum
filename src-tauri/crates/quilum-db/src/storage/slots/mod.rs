use crate::{Storage, slot::Slot};
use chrono::NaiveDateTime;
use surrealdb::{Error, types::RecordId};

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
}
