use crate::{
    core::StorageError,
    models::{Priority, Task},
    state::ApplicationResult,
};
use chrono::{DateTime, Utc};
use rusqlite::{Connection, params};
use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};
use uuid::Uuid;

/// Task repository to manage all its database operations
pub struct TaskRepository {
    pub(super) conn: Arc<Mutex<Connection>>,
    pub(super) db_path: PathBuf,
}

impl TaskRepository {
    pub fn new(conn: Arc<Mutex<Connection>>, db_path: PathBuf) -> Self {
        Self { conn, db_path }
    }

    /// Save tasks to the database
    pub fn save(&self, tasks: &[Task]) -> ApplicationResult<()> {
        let mut conn = self.conn.lock().unwrap();
        let tx = conn.transaction().map_err(|e| StorageError::Database {
            path: self.db_path.clone(),
            src: e.to_string(),
        })?;

        {
            let mut stmt = tx.prepare(
                "INSERT INTO tasks (id, title, description, completed, priority, pinned, created_at, updated_at, folder_id)
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
                ON CONFLICT(id) DO UPDATE SET
                    title = excluded.title,
                    description = excluded.description,
                    completed = excluded.completed,
                    priority = excluded.priority,
                    pinned = excluded.pinned,
                    updated_at = excluded.updated_at,
                    folder_id = excluded.folder_id"
            ).map_err(|e| StorageError::Database { path: self.db_path.clone(), src: e.to_string() })?;

            for task in tasks {
                let folder_id_str: Option<String> = task.folder_id.map(|id| id.to_string());
                stmt.execute(params![
                    task.id.to_string(),
                    task.title,
                    task.description,
                    task.completed,
                    task.priority.to_string(),
                    task.pinned,
                    task.created_at.to_rfc3339(),
                    task.updated_at.to_rfc3339(),
                    folder_id_str,
                ])
                .map_err(|e| StorageError::Database {
                    path: self.db_path.clone(),
                    src: e.to_string(),
                })?;
            }
        }

        if !tasks.is_empty() {
            let ids_placeholder: Vec<String> =
                tasks.iter().map(|t| format!("'{}'", t.id)).collect();
            let query = format!(
                "DELETE FROM tasks WHERE id NOT IN ({})",
                ids_placeholder.join(",")
            );
            tx.execute(&query, []).map_err(|e| StorageError::Database {
                path: self.db_path.clone(),
                src: e.to_string(),
            })?;
        } else {
            tx.execute("DELETE FROM tasks", [])
                .map_err(|e| StorageError::Database {
                    path: self.db_path.clone(),
                    src: e.to_string(),
                })?;
        }

        tx.commit().map_err(|e| StorageError::Database {
            path: self.db_path.clone(),
            src: e.to_string(),
        })?;

        Ok(())
    }

    /// Load tasks from the database
    pub fn load(&self) -> ApplicationResult<Vec<Task>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn
                .prepare("SELECT id, title, description, completed, priority, pinned, created_at, updated_at, folder_id FROM tasks")
                .map_err(|e| StorageError::Database { path: self.db_path.clone(), src: e.to_string() })?;

        let task_iter = stmt
            .query_map([], |row| {
                let id_str = row.get_ref(0)?.as_str()?;
                let title_str = row.get_ref(1)?.as_str()?;
                let priority_str = row.get_ref(4)?.as_str()?;
                let created_str = row.get_ref(6)?.as_str()?;
                let updated_str = row.get_ref(7)?.as_str()?;

                let id = Uuid::parse_str(&id_str).unwrap_or_else(|_| Uuid::new_v4());
                let priority: Priority = priority_str.parse().unwrap_or(Priority::Low);
                let folder_id = row
                    .get::<_, Option<String>>(8)?
                    .and_then(|s| Uuid::parse_str(&s).ok());

                let created_at = DateTime::parse_from_rfc3339(&created_str)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now());

                let updated_at = DateTime::parse_from_rfc3339(&updated_str)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now());

                let title_lower: String = title_str.to_lowercase();
                let id_formatted = format!("#{}", id.to_string().split('-').next().unwrap_or(""));

                Ok(Task {
                    id,
                    title: title_str.to_string(),
                    description: row.get(2)?,
                    completed: row.get(3)?,
                    priority,
                    pinned: row.get(5)?,
                    folder_id,
                    created_at,
                    updated_at,
                    title_lower,
                    id_formatted,
                })
            })
            .map_err(|e| StorageError::Database {
                path: self.db_path.clone(),
                src: e.to_string(),
            })?;

        let tasks: Vec<Task> = task_iter
            .map(|t| {
                t.map_err(|e| StorageError::Database {
                    path: self.db_path.clone(),
                    src: e.to_string(),
                })
            })
            .collect::<Result<Vec<Task>, StorageError>>()?;
        Ok(tasks)
    }
}

/// Unit-tests for task repository
#[cfg(test)]
mod tests {
    use super::*;

    fn setup_tasks_db() -> Arc<Mutex<Connection>> {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE tasks (
                id TEXT PRIMARY KEY NOT NULL,
                title TEXT NOT NULL,
                description TEXT NOT NULL,
                completed BOOLEAN NOT NULL DEFAULT 0,
                priority TEXT NOT NULL,
                pinned BOOLEAN NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                folder_id TEXT
            );",
        )
        .unwrap();

        Arc::new(Mutex::new(conn))
    }

    #[test]
    fn should_handle_save_load_for_tasks_repository() {
        let conn = setup_tasks_db();
        let repo = TaskRepository::new(conn, PathBuf::from("memory.db"));

        let folder_id = Uuid::new_v4();
        let task1: Task = Task::new("Repo Task 1")
            .with_description("Desc 1")
            .with_priority(Priority::High)
            .with_folder(folder_id);

        let task2: Task = Task::new("Repo Task 2").with_description("Desc 2");
        let tasks: Vec<Task> = vec![task1.clone(), task2.clone()];

        assert!(repo.save(&tasks).is_ok());

        let loaded: Vec<Task> = repo.load().unwrap();
        assert_eq!(loaded.len(), 2);
        assert_eq!(loaded[0].id, task1.id);
        assert_eq!(loaded[0].folder_id, Some(folder_id));
        assert_eq!(loaded[1].title, "Repo Task 2");
        assert_eq!(loaded[1].folder_id, None);
    }

    #[test]
    fn should_handle_deletions_sync_for_tasks_repository() {
        let conn = setup_tasks_db();
        let repo = TaskRepository::new(conn, PathBuf::from("memory.db"));

        let task1: Task = Task::new("Keep Me");
        let task2: Task = Task::new("Delete Me");

        repo.save(&vec![task1.clone(), task2.clone()]).unwrap();
        repo.save(&vec![task1.clone()]).unwrap();

        let loaded: Vec<Task> = repo.load().unwrap();
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].title, "Keep Me");
    }

    #[test]
    fn should_truncate_table_if_empty_tasks_provided() {
        let conn = setup_tasks_db();
        let repo = TaskRepository::new(conn, PathBuf::from("memory.db"));

        let task: Task = Task::new("Task");
        repo.save(&vec![task]).unwrap();

        repo.save(&[]).unwrap();

        let loaded: Vec<Task> = repo.load().unwrap();
        assert!(loaded.is_empty());
    }
}
