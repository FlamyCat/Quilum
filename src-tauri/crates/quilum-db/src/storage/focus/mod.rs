use surrealdb::Error;

use crate::{
    storage::model::task::{Scheduled, Task},
    Storage,
};

// Focus session methods
impl Storage {
    /// Gets the active focus session in a form of the first upcoming scheduled task (if any).
    ///
    /// # Returns
    /// * Optional focus session if active
    pub async fn get_active_session(&self) -> Result<Option<Task<Scheduled>>, Error> {
        let sql = "
            SELECT *
            FROM ONLY tasks
            WHERE
                scheduled_for + estimated_duration >= time::now()
                AND completed == false
            ORDER BY scheduled_for
            LIMIT 1;
        ";

        self.db.query(sql).await?.take(0)
    }
}
