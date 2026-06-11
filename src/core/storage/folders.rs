use crate::{core::StorageError, models::Folder, state::ApplicationResult};
use chrono::{DateTime, Utc};
use rusqlite::{Connection, params};
use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};
use uuid::Uuid;

/// Folder repository to manage all its database operations
pub struct FolderRepository {
    pub(super) conn: Arc<Mutex<Connection>>,
    pub(super) db_path: PathBuf,
}

impl FolderRepository {
    pub fn new(conn: Arc<Mutex<Connection>>, db_path: PathBuf) -> Self {
        Self { conn, db_path }
    }

    /// Save folder to the database
    pub fn save(&self, folders: &[Folder]) -> ApplicationResult<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM folders", [])
            .map_err(|e| StorageError::Database {
                path: self.db_path.clone(),
                src: e.to_string(),
            })?;

        let mut stmt = conn
            .prepare("INSERT INTO folders (id, name, color, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5)")
            .map_err(|e| StorageError::Database {
                path: self.db_path.clone(),
                src: e.to_string(),
            })?;

        for folder in folders {
            stmt.execute(params![
                folder.id.to_string(),
                folder.name,
                folder.color,
                folder.created_at.to_rfc3339(),
                folder.updated_at.to_rfc3339(),
            ])
            .map_err(|e| StorageError::Database {
                path: self.db_path.clone(),
                src: e.to_string(),
            })?;
        }

        Ok(())
    }

    /// Load folders from the database sorted by time created
    pub fn load(&self) -> ApplicationResult<Vec<Folder>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn
            .prepare("SELECT id, name, color, created_at, updated_at FROM folders ORDER BY created_at ASC")
            .map_err(|e| StorageError::Database {
                path: self.db_path.clone(),
                src: e.to_string(),
            })?;

        let folder_iter = stmt
            .query_map([], |row| {
                let id_str: String = row.get(0)?;
                let name: String = row.get(1)?;
                let color: String = row.get(2)?;
                let created_str: String = row.get(3)?;
                let updated_str: String = row.get(4)?;

                let id: Uuid = Uuid::parse_str(&id_str).unwrap_or_else(|_| Uuid::new_v4());
                let created_at = DateTime::parse_from_rfc3339(&created_str)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now());

                let updated_at = DateTime::parse_from_rfc3339(&updated_str)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now());

                Ok(Folder {
                    id,
                    name,
                    color,
                    created_at,
                    updated_at,
                    name_lower: row.get::<_, String>(1)?.to_lowercase(),
                })
            })
            .map_err(|e| StorageError::Database {
                path: self.db_path.clone(),
                src: e.to_string(),
            })?;

        let mut folders = Vec::new();
        for folder in folder_iter {
            folders.push(folder.map_err(|e| StorageError::Database {
                path: self.db_path.clone(),
                src: e.to_string(),
            })?);
        }
        Ok(folders)
    }
}

/// Unit-tests for folder repository
#[cfg(test)]
mod tests {
    use super::*;

    fn setup_folders_db() -> Arc<Mutex<Connection>> {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE folders (
                id TEXT PRIMARY KEY NOT NULL,
                name TEXT NOT NULL,
                color TEXT NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );",
        )
        .unwrap();

        Arc::new(Mutex::new(conn))
    }

    #[test]
    fn should_handle_save_load_for_folders_repository() {
        let conn = setup_folders_db();
        let repo = FolderRepository::new(conn, PathBuf::from("memory.db"));

        let folder1 = Folder::new("Work", "Red");
        let folder2 = Folder::new("Personal", "Blue");
        let folders = vec![folder1.clone(), folder2.clone()];

        assert!(repo.save(&folders).is_ok());
        let loaded = repo.load().unwrap();

        assert_eq!(loaded.len(), 2);
        assert_eq!(loaded[0].id, folder1.id);
        assert_eq!(loaded[0].color, "Red");
        assert_eq!(loaded[1].name, "Personal");
        assert_eq!(loaded[1].color, "Blue");
    }
}
