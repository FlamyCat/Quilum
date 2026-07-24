use surrealdb::Error;

use crate::{
    Storage,
    model::{slot::Slot, task::Task},
    slot::SLOTS_TABLE,
};

impl Storage {
    /// Gets all uncompleted tasks that theoretically can be done on time.
    ///
    /// # Returns
    /// * Vector of tasks where `completed == false` and `now + estimated_duration <= deadline`
    pub async fn get_uncompleted_tasks(&self) -> Result<Vec<Task<Unscheduled>>, Error> {
        let sql = "
            SELECT *
            FROM tasks
            WHERE
                completed == false
                AND time::now() + estimated_duration <= deadline;
        ";

        let mut result = self.db.query(sql).await?;
        let tasks: Vec<Task> = result.take(0)?;
        Ok(tasks)
    }

    /// Gets all future slots (slots that haven't ended yet).
    ///
    /// # Returns
    /// * Vector of slots where ends_at > now
    pub async fn get_future_slots(&self) -> Result<Vec<Slot>, Error> {
        let sql = format!(
            "
            SELECT * FROM {} WHERE ends_at > time::now() ORDER BY starts_at ASC
        ",
            SLOTS_TABLE
        );

        let slots = self.db.query(sql).await?.take(0)?;
        Ok(slots)
    }
}

#[cfg(test)]
mod tests;
