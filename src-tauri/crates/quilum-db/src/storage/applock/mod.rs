use surrealdb::Error;

use crate::{
    app_identifier::AppIdentifier,
    blocked_app::{BlockedApp, BLOCKED_APPS_TABLE},
    Storage,
};

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

        let blocked_app = BlockedApp::new(id_str, display_name.to_string());
        let created: Option<BlockedApp> = self
            .db
            .create(BLOCKED_APPS_TABLE)
            .content(blocked_app)
            .await?;

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

        let _: Option<BlockedApp> = self.db.delete((BLOCKED_APPS_TABLE, id_str)).await?;
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
    ) -> surrealdb::Result<BlockedApp> {
        let id_str = match &identifier {
            AppIdentifier::Path(p) => p.to_string_lossy().to_string(),
            AppIdentifier::BundleId(s) => s.clone(),
        };

        let blocked_app = BlockedApp::new(id_str.clone(), display_name.to_string());

        self.db
            .upsert((BLOCKED_APPS_TABLE, id_str))
            .content(blocked_app)
            .await
            .map(Option::unwrap)
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
