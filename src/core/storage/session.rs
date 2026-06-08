use crate::{
    core::{Selectable, StorageError},
    state::{ApplicationResult, Session},
};
use rusqlite::{Connection, OptionalExtension, params};
use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};
use uuid::Uuid;

/// Session repository to manage all its database operations
pub struct SessionRepository {
    pub(super) conn: Arc<Mutex<Connection>>,
    pub(super) db_path: PathBuf,
}

impl SessionRepository {
    pub fn new(conn: Arc<Mutex<Connection>>, db_path: PathBuf) -> Self {
        Self { conn, db_path }
    }

    /// Save session data to the database
    pub fn save(&self, session: &Session) -> ApplicationResult<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
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
        ).map_err(|e| StorageError::Database { path: self.db_path.clone(), src: e.to_string() })?;
        Ok(())
    }

    /// Load session data from the database
    pub fn load(&self) -> ApplicationResult<Option<Session>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn
            .prepare(
                "SELECT last_selected_id, last_focus, last_filter, last_query,
                    description_scroll_pos, hotkeys_scroll_pos, use_system_theme
             FROM session LIMIT 1",
            )
            .map_err(|e| StorageError::Database {
                path: self.db_path.clone(),
                src: e.to_string(),
            })?;

        let session: Option<Session> = stmt
            .query_row([], |row| {
                let id_str: Option<String> = row.get(0)?;
                let focus_str: String = row.get(1)?;
                let filter_str: String = row.get(2)?;
                let last_selected_id = id_str.and_then(|s| Uuid::parse_str(&s).ok());

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
            .optional()
            .map_err(|e| StorageError::Database {
                path: self.db_path.clone(),
                src: e.to_string(),
            })?;

        Ok(session)
    }
}

/// Unit-tests for session repository
#[cfg(test)]
mod tests {
    use super::*;

    fn setup_session_db() -> Arc<Mutex<Connection>> {
        let conn: Connection = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE session (
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
        .unwrap();

        Arc::new(Mutex::new(conn))
    }

    #[test]
    fn should_return_none_if_session_repository_is_empty() {
        let conn = setup_session_db();
        let repo = SessionRepository::new(conn, PathBuf::from("memory.db"));

        let session: Option<Session> = repo.load().unwrap();
        assert!(session.is_none());
    }

    #[test]
    fn should_handle_save_load_for_session_repository() {
        let conn = setup_session_db();
        let repo = SessionRepository::new(conn, PathBuf::from("memory.db"));

        let target_id: Uuid = Uuid::new_v4();
        let session = Session {
            last_selected_id: Some(target_id),
            last_query: "search query".to_string(),
            description_scroll_pos: 42,
            use_system_theme: true,
            ..Session::default()
        };

        assert!(repo.save(&session).is_ok());

        let loaded: Session = repo.load().unwrap().unwrap();
        assert_eq!(loaded.last_selected_id, Some(target_id));
        assert_eq!(loaded.last_query, "search query");
        assert_eq!(loaded.description_scroll_pos, 42);
        assert!(loaded.use_system_theme);
    }
}
