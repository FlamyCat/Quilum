use crate::{Storage, focus_session::FocusSession};
use chrono::NaiveDateTime;
use surrealdb::{Error, types::RecordId};

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
