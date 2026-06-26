use crate::{Storage, slot::Slot, task::Task};
use surrealdb::Error;

impl Storage {
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

#[cfg(test)]
mod tests;
