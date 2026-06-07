use crate::{
    config::StorageConfig,
    core::{Selectable, StorageError},
    models::{Priority, Task},
    state::{ApplicationResult, Session, TasksStateData},
};
use rusqlite::{Connection, params};
use std::{
    fs::{self},
    path::{Path, PathBuf},
};

/// Storage structure for all task SQLite database operations
pub struct Storage {
    conn: Connection,
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

        let storage = Self { conn, path: p };
        storage.create_tables()?;

        Ok(storage)
    }

    /// Save tasks and UI Session to the SQLite database
    pub fn save(&mut self, tasks: &[Task], session: Session) -> ApplicationResult<String> {
        log::debug!("Starting database transaction to save tasks and UI session");

        let tx = self
            .conn
            .transaction()
            .map_err(|e| StorageError::Database {
                path: self.path.clone(),
                src: e.to_string(),
            })?;

        let mut insert_todo_stmt = tx.prepare(
            "INSERT INTO tasks (id, title, description, completed, priority, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
             ON CONFLICT(id) DO UPDATE SET
                title = excluded.title,
                description = excluded.description,
                completed = excluded.completed,
                priority = excluded.priority,
                updated_at = excluded.updated_at"
        ).map_err(|e| StorageError::Database { path: self.path.clone(), src: e.to_string() })?;

        for todo in tasks {
            insert_todo_stmt
                .execute(params![
                    todo.id.to_string(),
                    todo.title,
                    todo.description,
                    todo.completed,
                    todo.priority.to_string(),
                    todo.created_at.to_rfc3339(),
                    todo.updated_at.to_rfc3339(),
                ])
                .map_err(|e| StorageError::Database {
                    path: self.path.clone(),
                    src: e.to_string(),
                })?;
        }
        drop(insert_todo_stmt);

        if !tasks.is_empty() {
            let ids_placeholder: Vec<String> =
                tasks.iter().map(|t| format!("'{}'", t.id)).collect();
            let query = format!(
                "DELETE FROM tasks WHERE id NOT IN ({})",
                ids_placeholder.join(",")
            );
            tx.execute(&query, []).map_err(|e| StorageError::Database {
                path: self.path.clone(),
                src: e.to_string(),
            })?;
        } else {
            tx.execute("DELETE FROM tasks", [])
                .map_err(|e| StorageError::Database {
                    path: self.path.clone(),
                    src: e.to_string(),
                })?;
        }

        tx.execute(
            "INSERT INTO session (id, last_selected_id, last_focus, last_filter, last_query, description_scroll_pos, hotkeys_scroll_pos, use_system_theme)
             VALUES (1, ?1, ?2, ?3, ?4, ?5, ?6, ?7)
             ON CONFLICT(id) DO UPDATE SET
                last_selected_id = excluded.last_selected_id,
                last_focus = excluded.last_focus,
                last_filter = excluded.last_filter,
                last_query = excluded.last_query,
                description_scroll_pos = excluded.description_scroll_pos,
                hotkeys_scroll_pos = excluded.hotkeys_scroll_pos,
                use_system_theme = excluded.use_system_theme",
            params![
                session.last_selected_id.map(|id| id.to_string()),
                session.last_focus.to_string(),
                session.last_filter.to_string(),
                session.last_query,
                session.description_scroll_pos,
                session.hotkeys_scroll_pos,
                session.use_system_theme,
            ],
        ).map_err(|e| StorageError::Database { path: self.path.clone(), src: e.to_string() })?;

        tx.commit().map_err(|e| StorageError::Database {
            path: self.path.clone(),
            src: e.to_string(),
        })?;

        log::info!(
            "Successfully saved storage data: {} tasks, filter: {:?}, focus: {:?}",
            tasks.len(),
            session.last_filter,
            session.last_focus
        );

        Ok("Data was successfully save to the database!".into())
    }

    /// Load tasks and session info from the SQLite database
    pub fn load(&self) -> ApplicationResult<TasksStateData> {
        log::debug!(
            "Loading tasks and session from SQLite database at {:?}",
            self.path
        );

        let mut stmt = self.conn
            .prepare("SELECT id, title, description, completed, priority, created_at, updated_at FROM tasks")
            .map_err(|e| StorageError::Database { path: self.path.clone(), src: e.to_string() })?;

        let task_iter = stmt
            .query_map([], |row| {
                let id_str: String = row.get(0)?;
                let title_str: String = row.get(1)?;
                let priority_str: String = row.get(4)?;
                let created_str: String = row.get(5)?;
                let updated_str: String = row.get(6)?;

                let id = uuid::Uuid::parse_str(&id_str).unwrap_or_else(|_| uuid::Uuid::new_v4());
                let priority = priority_str.parse().unwrap_or(Priority::Low);

                let created_at = chrono::DateTime::parse_from_rfc3339(&created_str)
                    .map(|dt| dt.with_timezone(&chrono::Utc))
                    .unwrap_or_else(|_| chrono::Utc::now());

                let updated_at = chrono::DateTime::parse_from_rfc3339(&updated_str)
                    .map(|dt| dt.with_timezone(&chrono::Utc))
                    .unwrap_or_else(|_| chrono::Utc::now());

                let title_lower: String = title_str.to_lowercase();
                Ok(Task {
                    id,
                    title: title_str,
                    description: row.get(2)?,
                    completed: row.get(3)?,
                    priority,
                    created_at,
                    updated_at,
                    title_lower,
                })
            })
            .map_err(|e| StorageError::Database {
                path: self.path.clone(),
                src: e.to_string(),
            })?;

        let mut tasks = Vec::new();
        for task in task_iter {
            tasks.push(task.map_err(|e| StorageError::Database {
                path: self.path.clone(),
                src: e.to_string(),
            })?);
        }

        let session: Session = self.load_session()?.unwrap_or_default();
        log::info!(
            "Loaded {} tasks and session memory from SQLite",
            tasks.len()
        );
        Ok(TasksStateData::new(tasks, session))
    }

    /// Loads session status (все поля маппятся на реальную структуру таблицы)
    pub fn load_session(&self) -> ApplicationResult<Option<Session>> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT last_selected_id, last_focus, last_filter, last_query,
                    description_scroll_pos, hotkeys_scroll_pos, use_system_theme
             FROM session LIMIT 1",
            )
            .map_err(|e| StorageError::Database {
                path: self.path.clone(),
                src: e.to_string(),
            })?;

        let mut session_iter = stmt
            .query_map([], |row| {
                let id_str: Option<String> = row.get(0)?;
                let focus_str: String = row.get(1)?;
                let filter_str: String = row.get(2)?;

                let last_selected_id = id_str.and_then(|s| uuid::Uuid::parse_str(&s).ok());

                Ok(Session {
                    last_selected_id,
                    last_focus: Selectable::new(focus_str.parse().unwrap_or_default()),
                    last_filter: Selectable::new(filter_str.parse().unwrap_or_default()),
                    last_query: row.get(3)?,
                    description_scroll_pos: row.get(4)?,
                    hotkeys_scroll_pos: row.get(5)?,
                    use_system_theme: row.get(6)?,
                })
            })
            .map_err(|e| StorageError::Database {
                path: self.path.clone(),
                src: e.to_string(),
            })?;

        if let Some(result) = session_iter.next() {
            Ok(Some(result.map_err(|e| StorageError::Database {
                path: self.path.clone(),
                src: e.to_string(),
            })?))
        } else {
            Ok(None)
        }
    }

    /// Helper function to create db tables: tasks and session
    fn create_tables(&self) -> ApplicationResult<()> {
        self.conn
            .execute_batch(
                "CREATE TABLE IF NOT EXISTS tasks (
                    id TEXT PRIMARY KEY NOT NULL,
                    title TEXT NOT NULL,
                    description TEXT NOT NULL,
                    completed BOOLEAN NOT NULL DEFAULT 0,
                    priority TEXT NOT NULL,
                    created_at TEXT NOT NULL,
                    updated_at TEXT NOT NULL
                );

                CREATE TABLE IF NOT EXISTS session (
                    id INTEGER PRIMARY KEY CHECK (id = 1),
                    last_selected_id TEXT,
                    last_focus TEXT NOT NULL,
                    last_filter TEXT NOT NULL,
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

/// Unit-tests for storage
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        core::{ApplicationError, FocusArea},
        models::Filter,
        state::UIState,
    };
    use tempdir::TempDir;

    fn setup_config(backup: bool) -> StorageConfig {
        StorageConfig {
            backup_enabled: backup,
            ..Default::default()
        }
    }

    #[test]
    fn should_save_and_load_data_successfully() {
        let temp_dir: TempDir = TempDir::new("task_test").unwrap();
        let path: PathBuf = temp_dir.path().join("toodles.db");
        let config: StorageConfig = setup_config(true);
        let mut storage = Storage::init(Some(&path), &config).unwrap();

        let tasks = vec![Task::new("Task 1", "", None), Task::new("Task 2", "", None)];
        let session = Session::default();

        let result: ApplicationResult<String> = storage.save(&tasks, session);
        assert!(result.is_ok());

        let loaded_data = storage.load().unwrap();

        assert_eq!(loaded_data.tasks.len(), 2);
        assert_eq!(loaded_data.tasks[0].title, "Task 1");
        assert_eq!(loaded_data.tasks[1].title, "Task 2");
        assert_eq!(loaded_data.session.last_filter, Filter::All);
        assert_eq!(loaded_data.session.last_focus, FocusArea::Sidebar);
    }

    #[test]
    fn should_create_backup_on_init_if_file_exists() {
        let temp_dir: TempDir = TempDir::new("task_test").unwrap();
        let path: PathBuf = temp_dir.path().join("toodles.db");
        let config: StorageConfig = setup_config(true);
        let backup_path: PathBuf = path.with_extension("db.bak");

        let mut storage: Storage = Storage::init(Some(&path), &config).unwrap();
        assert!(
            !backup_path.exists(),
            "Backup shouldn't exist on first init"
        );

        storage
            .save(&vec![Task::new("V1", "", None)], Session::default())
            .unwrap();
        drop(storage);

        let _storage2 = Storage::init(Some(&path), &config).unwrap();

        assert!(path.exists());
        assert!(backup_path.exists());

        drop(_storage2);

        let backup_storage = Storage::init(Some(&backup_path), &config).unwrap();
        let backup_data = backup_storage.load().unwrap();
        assert_eq!(backup_data.tasks[0].title, "V1");
    }

    #[test]
    fn should_handle_sync_deletions() {
        let temp_dir: TempDir = TempDir::new("task_test").unwrap();
        let path: PathBuf = temp_dir.path().join("toodles.db");
        let config: StorageConfig = setup_config(false);
        let mut storage: Storage = Storage::init(Some(&path), &config).unwrap();

        let task1: Task = Task::new("Task 1", "", None);
        let task2: Task = Task::new("Task 2", "", None);

        storage
            .save(&vec![task1.clone(), task2.clone()], Session::default())
            .unwrap();

        storage.save(&vec![task1], Session::default()).unwrap();

        let loaded = storage.load().unwrap();
        assert_eq!(loaded.tasks.len(), 1);
        assert_eq!(loaded.tasks[0].title, "Task 1");
    }

    #[test]
    fn should_create_new_state_empty_tasks() {
        let temp_dir: TempDir = TempDir::new("task_test").unwrap();
        let path: PathBuf = temp_dir.path().join("toodles.db");
        let config: StorageConfig = setup_config(true);

        assert!(!path.exists());

        let mut storage: Storage = Storage::init(Some(&path), &config).unwrap();
        let state = storage.load().unwrap();
        assert!(state.tasks.is_empty());

        let session: Session = Session::from_state(&UIState::default(), None);
        let result: ApplicationResult<String> = storage.save(&state.tasks, session);
        assert!(result.is_ok());
        assert!(path.exists());
    }

    #[test]
    fn should_return_default_data_path() {
        let path_result = Storage::get_data_path();
        assert!(path_result.is_ok());

        let path: PathBuf = path_result.unwrap();
        assert!(path.ends_with("toodles/toodles.db") || path.ends_with("toodles\\toodles.db"));
        assert!(path.is_absolute());
    }

    #[test]
    fn should_create_new_state_with_saved_tasks_and_session() {
        let temp_dir: TempDir = TempDir::new("task_test").unwrap();
        let path: PathBuf = temp_dir.path().join("tasks.db");
        let config: StorageConfig = setup_config(true);
        let mut storage = Storage::init(Some(&path), &config).unwrap();

        let task = Task::new("Test Title", "Test Desc", Some(Priority::High));
        let task_id = task.id;

        let session = Session {
            last_selected_id: Some(task_id),
            ..Session::default()
        };

        let tasks_to_save = vec![task];
        storage.save(&tasks_to_save, session).unwrap();
        let loaded_state = storage.load().unwrap();

        assert_eq!(loaded_state.tasks.len(), 1);
        assert_eq!(loaded_state.tasks[0].id, task_id);
        assert_eq!(loaded_state.tasks[0].title, "Test Title");
        assert_eq!(loaded_state.session.last_selected_id, Some(task_id));
    }

    #[test]
    fn should_create_directory_on_init_if_missing() {
        let temp_dir: TempDir = TempDir::new("task_test").unwrap();
        let path: PathBuf = temp_dir.path().join("a").join("b").join("toodles.db");
        let config: StorageConfig = setup_config(false);
        assert!(!path.parent().unwrap().exists());

        let _storage = Storage::init(Some(&path), &config).unwrap();
        assert!(
            path.parent().unwrap().exists(),
            "Directory hierarchy should be created on init"
        );
        assert!(path.exists(), "Database file should be created");
    }

    #[test]
    fn should_invoke_database_error_on_init_if_corrupted() {
        let temp_dir: TempDir = TempDir::new("task_test").unwrap();
        let path: PathBuf = temp_dir.path().join("toodles.db");
        let config: StorageConfig = setup_config(false);

        fs::write(&path, "not a sqlite database file").unwrap();

        let result: ApplicationResult<Storage> = Storage::init(Some(&path), &config);

        assert!(result.is_err());
        assert!(matches!(
            result,
            Err(ApplicationError::Storage(StorageError::Database { .. }))
        ));
    }

    #[test]
    #[cfg(unix)]
    fn should_invoke_io_error_on_init_when_no_write_permission() {
        use std::{fs::Permissions, os::unix::fs::PermissionsExt};

        let temp_dir: TempDir = TempDir::new("task_test").unwrap();
        let path: PathBuf = temp_dir.path().join("toodles.db");
        let config: StorageConfig = setup_config(false);

        let mut perms: Permissions = fs::metadata(temp_dir.path()).unwrap().permissions();
        perms.set_mode(0o444);
        fs::set_permissions(temp_dir.path(), perms).unwrap();

        let result: ApplicationResult<Storage> = Storage::init(Some(&path), &config);

        assert!(result.is_err());
        assert!(matches!(
            result,
            Err(ApplicationError::Storage(StorageError::Database { .. }))
        ));
    }
}
