pub mod folders;
pub mod session;
pub mod tasks;

pub use folders::FolderRepository;
pub use session::SessionRepository;
pub use tasks::TaskRepository;

use rusqlite::Connection;
use std::{
    fs,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

use crate::{
    config::StorageConfig,
    core::StorageError,
    models::{Folder, Task},
    state::{ApplicationResult, Session, TasksStateData, TasksStateSave},
};

/// Storage structure for all task SQLite database operations
pub struct Storage {
    conn: Arc<Mutex<Connection>>,
    path: PathBuf,
}

impl Storage {
    /// Get default database path to save/load from
    pub fn get_data_path() -> ApplicationResult<PathBuf> {
        let path: ApplicationResult<PathBuf> = dirs::data_dir()
            .ok_or_else(|| {
                StorageError::Environment {
                    context: "data".to_string(),
                }
                .into()
            })
            .map(|dir| dir.join("toodles").join("toodles.db"));

        if let Ok(ref p) = path {
            log::debug!("Database path resolved to: {:?}", p);
        }

        path
    }

    /// Get default logging path
    pub fn get_log_path() -> ApplicationResult<PathBuf> {
        let path: ApplicationResult<PathBuf> = dirs::data_dir()
            .ok_or_else(|| {
                StorageError::Environment {
                    context: "log".to_string(),
                }
                .into()
            })
            .map(|dir| dir.join("toodles").join("toodles.log"));

        if let Ok(ref p) = path {
            log::debug!("Log path resolved to: {:?}", p);
        }

        path
    }

    /// Initializes Storage: creates folders, makes backups (if needed) and opens connection
    pub fn init(path: Option<&Path>, config: &StorageConfig) -> ApplicationResult<Self> {
        let p: PathBuf = match path {
            Some(p) => p.to_path_buf(),
            None => Self::get_data_path()?,
        };

        if let Some(parent) = p.parent() {
            fs::create_dir_all(parent).map_err(|e| StorageError::IO {
                path: p.clone(),
                src: e.to_string(),
            })?;
        }

        if config.backup_enabled && p.exists() {
            let mut backup_path: PathBuf = p.clone();
            backup_path.set_extension("db.bak");
            log::debug!("Creating database backup at {:?}", backup_path);
            let _ = fs::copy(&p, backup_path);
        }

        log::debug!("Opening SQLite connection at {:?}", p);
        let conn: Connection = Connection::open(&p).map_err(|e| StorageError::Database {
            path: p.clone(),
            src: e.to_string(),
        })?;

        let _ = conn.execute("PRAGMA foreign_keys = ON;", []);
        let storage = Self {
            conn: Arc::new(Mutex::new(conn)),
            path: p,
        };
        storage.create_tables()?;

        Ok(storage)
    }

    /// Method to get tasks repository
    pub fn tasks(&self) -> TaskRepository {
        TaskRepository::new(Arc::clone(&self.conn), self.path.clone())
    }

    /// Method to get session repository
    pub fn session(&self) -> SessionRepository {
        SessionRepository::new(Arc::clone(&self.conn), self.path.clone())
    }

    /// Method to get folders repository
    pub fn folders(&self) -> FolderRepository {
        FolderRepository::new(Arc::clone(&self.conn), self.path.clone())
    }

    /// Save all data to the SQLite database using repositories
    pub fn save(&mut self, state: &TasksStateSave) -> ApplicationResult<String> {
        log::debug!("Delegating save operations to repositories");

        self.folders().save(&state.folders)?;
        self.tasks().save(&state.tasks)?;
        self.session().save(&state.session)?;

        log::info!(
            "Successfully saved storage data: {} folders, {} tasks, tab: {:?}, focus: {:?}",
            state.folders.len(),
            state.tasks.len(),
            state.session.last_tab,
            state.session.last_focus
        );

        Ok("Data was successfully saved to the database!".into())
    }

    /// Load all data from the SQLite database using repositories
    pub fn load(&self) -> ApplicationResult<TasksStateData> {
        log::debug!(
            "Loading folders, tasks and session from SQLite database at {:?}",
            self.path
        );

        let folders: Vec<Folder> = self.folders().load()?;
        let tasks: Vec<Task> = self.tasks().load()?;
        let session: Session = self.session().load()?.unwrap_or_default();

        log::info!(
            "Loaded {} folders, {} tasks and session memory from SQLite",
            folders.len(),
            tasks.len()
        );

        Ok(TasksStateData::new(tasks, folders, session))
    }

    /// Helper function to create db tables: tasks and session
    fn create_tables(&self) -> ApplicationResult<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS folders (
                id TEXT PRIMARY KEY NOT NULL,
                name TEXT NOT NULL,
                color TEXT NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS tasks (
                id TEXT PRIMARY KEY NOT NULL,
                title TEXT NOT NULL,
                description TEXT NOT NULL,
                completed BOOLEAN NOT NULL DEFAULT 0,
                priority TEXT NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                folder_id TEXT,
                FOREIGN KEY (folder_id) REFERENCES folders(id) ON DELETE SET NULL
            );

            CREATE TABLE IF NOT EXISTS session (
                id INTEGER PRIMARY KEY CHECK (id = 1),
                last_selected_id TEXT,
                last_focus TEXT NOT NULL,
                last_tab TEXT NOT NULL,
                last_folder_id TEXT,
                last_query TEXT NOT NULL,
                description_scroll_pos INTEGER NOT NULL,
                hotkeys_scroll_pos INTEGER NOT NULL,
                use_system_theme BOOLEAN NOT NULL
            );",
        )
        .map_err(|e| StorageError::Database {
            path: self.path.clone(),
            src: e.to_string(),
        })?;

        Ok(())
    }
}
