use crate::Storage;
use chrono::Local;
use directories::ProjectDirs;
use surrealdb::{
    engine::local::{Db, Mem, SurrealKv}, Error,
    Surreal,
};

impl Storage {
    /// Creates a new Storage instance with the given SurrealDB connection.
    ///
    /// # Arguments
    /// * `db` - A SurrealDB instance connected to a specific engine
    ///
    /// # Returns
    /// * The storage instance or an error
    pub fn new(db: Surreal<Db>) -> Self {
        Self { db }
    }

    /// Initialize the database with required initialization script
    /// Runs the init.surql script included at compile time.
    pub async fn init(&self) -> Result<(), Error> {
        let init_script = include_str!("../../../resources/init.surql");
        self.db.query(init_script).await?;
        Ok(())
    }

    /// Creates a new Storage instance using in-memory database mode.
    ///
    /// # Returns
    /// * The storage instance or an error
    pub async fn new_mem() -> Result<Self, Error> {
        let db = Surreal::new::<Mem>(()).await?;
        db.use_ns("test").use_db("test").await?;
        let storage = Self::new(db);
        storage.init().await?;
        Ok(storage)
    }

    /// Creates a new Storage instance using SurrealKV database mode.
    /// Uses platform-specific data directory via ProjectDirs.
    /// If the database cannot be opened, backs up the existing database
    /// to `quilum.db.{timestamp}.copy` and creates a fresh database.
    ///
    /// # Returns
    /// * The storage instance or an error
    pub async fn new_surrealkv() -> Result<Self, Error> {
        let proj_dirs = ProjectDirs::from("com", "quilum", "quilum")
            .expect("Failed to get project directories");

        let data_dir = proj_dirs.data_dir().to_path_buf();
        std::fs::create_dir_all(&data_dir).expect("Failed to create data directory");
        let db_path = data_dir.join("quilum.db");

        let db = match Surreal::new::<SurrealKv>(db_path.clone()).await {
            Ok(db) => db,
            Err(e) => {
                if db_path.exists() {
                    let timestamp = Local::now().format("%Y-%m-%d_%H-%M-%S");
                    let backup_path = data_dir.join(format!("quilum.db.{}.copy", timestamp));
                    std::fs::rename(&db_path, &backup_path).map_err(|rename_err| {
                        Error::query(
                            format!("Failed to backup corrupted database: {rename_err}"),
                            None,
                        )
                    })?;
                    Surreal::new::<SurrealKv>(db_path).await?
                } else {
                    return Err(e);
                }
            }
        };

        db.use_ns("quilum").use_db("main").await?;
        let storage = Self::new(db);
        storage.init().await?;
        Ok(storage)
    }
}
