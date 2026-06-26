use crate::{Storage, app_identifier::AppIdentifier, blocked_app::BlockedApp};
use surrealdb::Error;

// App blocking methods
impl Storage {
    /// Adds an app to the blocked apps list.
    ///
    /// # Arguments
    /// * `identifier` - The app identifier (path or bundle ID)
    /// * `display_name` - The display name of the app
    ///
    /// # Returns
    /// * The created blocked app record
    pub async fn add_blocked_app(
        &self,
        identifier: AppIdentifier,
        display_name: &str,
    ) -> Result<BlockedApp, Error> {
        let id_str = match identifier {
            AppIdentifier::Path(p) => p.to_string_lossy().to_string(),
            AppIdentifier::BundleId(s) => s,
        };
        let data = serde_json::json!({
            "identifier": id_str,
            "display_name": display_name
        });
        let created: Option<BlockedApp> = self.db.create("blocked_app").content(data).await?;
        created.ok_or_else(|| Error::query("Failed to create blocked app".to_string(), None))
    }

    /// Removes an app from the blocked apps list.
    ///
    /// # Arguments
    /// * `identifier` - The app identifier to remove
    ///
    /// # Returns
    /// * Success or error
    pub async fn remove_blocked_app(&self, identifier: &AppIdentifier) -> Result<(), Error> {
        let id_str = match identifier {
            AppIdentifier::Path(p) => p.to_string_lossy().to_string(),
            AppIdentifier::BundleId(s) => s.clone(),
        };
        let sql = format!(
            "DELETE FROM blocked_app WHERE identifier = '{}'",
            id_str.replace('\'', "''")
        );
        self.db.query(sql).await?;
        Ok(())
    }

    /// Gets all blocked apps.
    ///
    /// # Returns
    /// * Vector of blocked apps
    pub async fn get_blocked_apps(&self) -> Result<Vec<BlockedApp>, Error> {
        let sql = "SELECT * FROM blocked_app".to_string();
        let mut result = self.db.query(sql).await?;
        let apps: Vec<BlockedApp> = result.take(0).unwrap_or_default();
        Ok(apps)
    }

    /// Upserts (creates or updates) a blocked app.
    ///
    /// # Arguments
    /// * `identifier` - The app identifier (path or bundle ID)
    /// * `display_name` - The display name of the app
    ///
    /// # Returns
    /// * The upserted blocked app record
    pub async fn upsert_blocked_app(
        &self,
        identifier: AppIdentifier,
        display_name: &str,
    ) -> Result<BlockedApp, Error> {
        let id_str = match &identifier {
            AppIdentifier::Path(p) => p.to_string_lossy().to_string(),
            AppIdentifier::BundleId(s) => s.clone(),
        };

        // First check if record exists by querying
        let apps = self.get_blocked_apps().await?;
        let existing = apps.iter().find(|app| app.identifier == id_str);

        if let Some(existing_app) = existing {
            // Record exists - update it using query builder
            let key = Self::record_id_key(&existing_app.id);
            let data = serde_json::json!({
                "identifier": id_str,
                "display_name": display_name
            });
            let updated: Option<BlockedApp> =
                self.db.update(("blocked_app", key)).content(data).await?;
            return updated
                .ok_or_else(|| Error::query("Failed to update blocked app".to_string(), None));
        } else {
            // Record doesn't exist - create new one using query builder
            let data = serde_json::json!({
                "identifier": id_str,
                "display_name": display_name
            });
            let created: Option<BlockedApp> = self.db.create("blocked_app").content(data).await?;
            return created
                .ok_or_else(|| Error::query("Failed to create blocked app".to_string(), None));
        }
    }

    /// Deletes all blocked apps (clears the table).
    ///
    /// # Returns
    /// * Success or error
    pub async fn delete_all_blocked_apps(&self) -> Result<(), Error> {
        self.db.query("DELETE FROM blocked_app").await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests;
