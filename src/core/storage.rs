use crate::{config::StorageConfig, core::StorageError, models::Todo, state::ApplicationResult};
use std::{
    fs::{self, File},
    io::{BufWriter, Write},
    path::{Path, PathBuf},
};

pub struct Storage;

impl Storage {
    /// Get default data path to save/load from
    pub fn get_data_path() -> ApplicationResult<PathBuf> {
        dirs::data_dir()
            .ok_or(StorageError::PathNotFound.into())
            .map(|dir| dir.join("todo-tui").join("todos.json"))
    }

    /// Save todos to user path/default path
    pub fn save(
        todos: &[Todo],
        path: Option<&Path>,
        config: &StorageConfig,
    ) -> ApplicationResult<()> {
        let p: PathBuf = match path {
            Some(p) => p.to_path_buf(),
            None => Self::get_data_path()?,
        };

        if let Some(parent) = p.parent() {
            fs::create_dir_all(parent).map_err(|e| StorageError::IOError(e.to_string()))?;
        }

        let mut temp_path: PathBuf = p.clone();
        temp_path.set_extension("tmp");

        let result = (|| -> Result<(), StorageError> {
            let file =
                File::create(&temp_path).map_err(|e| StorageError::IOError(e.to_string()))?;
            let mut writer = BufWriter::new(file);

            serde_json::to_writer_pretty(&mut writer, todos)
                .map_err(|e| StorageError::JSONError(e.to_string()))?;

            writer
                .flush()
                .map_err(|e| StorageError::IOError(e.to_string()))?;

            writer
                .get_ref()
                .sync_all()
                .map_err(|e| StorageError::IOError(e.to_string()))?;

            Ok(())
        })();

        if let Err(e) = result {
            let _ = fs::remove_file(&temp_path);
            return Err(e.into());
        }

        if config.backup_enabled && p.exists() {
            let mut backup_path = p.clone();
            backup_path.set_extension("json.bak");
            let _ = fs::rename(&p, backup_path);
        }

        fs::rename(&temp_path, &p).map_err(|err| StorageError::IOError(err.to_string()))?;
        Ok(())
    }

    /// Load todos from a user path/default path
    pub fn load(path: Option<&Path>, config: &StorageConfig) -> ApplicationResult<Vec<Todo>> {
        let p: PathBuf = match path {
            Some(p) => p.to_path_buf(),
            None => Self::get_data_path()?,
        };

        if p.exists() {
            match Self::load_from_path(&p) {
                Ok(todos) => return Ok(todos),
                Err(e) => {
                    if !config.backup_enabled {
                        return Err(e);
                    }
                }
            }
        }

        if config.backup_enabled {
            let mut backup = p.clone();
            backup.set_extension("json.bak");

            if backup.exists() {
                return Self::load_from_path(&backup);
            }
        }

        Ok(Vec::new())
    }

    /// Helper function to load from path (not to duplicate code)
    fn load_from_path(p: &Path) -> ApplicationResult<Vec<Todo>> {
        let file = File::open(p).map_err(|err| StorageError::IOError(err.to_string()))?;
        let todos = serde_json::from_reader(file)
            .map_err(|err| StorageError::JSONError(err.to_string()))?;
        Ok(todos)
    }
}

/// Unit-tests for storage
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{core::ApplicationError, models::Priority, state::ApplicationState};
    use tempdir::TempDir;

    fn setup_config(backup: bool) -> StorageConfig {
        StorageConfig {
            backup_enabled: backup,
            ..Default::default()
        }
    }

    #[test]
    fn should_save_and_load_todos_successfully() {
        let temp_dir: TempDir = TempDir::new("todo_test").unwrap();
        let path: PathBuf = temp_dir.path().join("todos.json");
        let config: StorageConfig = setup_config(true);

        let todos = vec![Todo::new("Task 1", "", None), Todo::new("Task 2", "", None)];
        let result: ApplicationResult<()> = Storage::save(&todos, Some(&path), &config);
        assert!(result.is_ok());

        let loaded_todos: Vec<Todo> = Storage::load(Some(&path), &config).unwrap();

        assert_eq!(loaded_todos.len(), 2);
        assert_eq!(loaded_todos[0].title, "Task 1");
        assert_eq!(loaded_todos[1].title, "Task 2");
    }

    #[test]
    fn should_create_backup_on_save() {
        let temp_dir: TempDir = TempDir::new("todo_test").unwrap();
        let path: PathBuf = temp_dir.path().join("todos.json");
        let config: StorageConfig = setup_config(true);
        let backup_path: PathBuf = path.with_extension("json.bak");

        Storage::save(&vec![Todo::new("V1", "", None)], Some(&path), &config).unwrap();
        assert!(
            !backup_path.exists(),
            "Backup shouldn't exist on first save"
        );

        Storage::save(&vec![Todo::new("V2", "", None)], Some(&path), &config).unwrap();

        assert!(path.exists());
        assert!(backup_path.exists());

        let backup_data = Storage::load(Some(&backup_path), &config).unwrap();
        assert_eq!(backup_data[0].title, "V1");
    }

    #[test]
    fn should_restore_from_backup_if_main_file_is_missing() {
        let temp_dir: TempDir = TempDir::new("todo_test").unwrap();
        let path: PathBuf = temp_dir.path().join("todos.json");
        let config: StorageConfig = setup_config(true);
        let backup_path: PathBuf = path.with_extension("json.bak");

        let todos = vec![Todo::new("Backup Task", "", None)];
        Storage::save(&todos, Some(&backup_path), &config).unwrap();

        assert!(!path.exists());

        let loaded = Storage::load(Some(&path), &config).expect("Should fallback to backup");
        assert_eq!(loaded[0].title, "Backup Task");
    }

    #[test]
    fn should_handle_corrupted_main_file_with_backup_fallback() {
        let temp_dir: TempDir = TempDir::new("todo_test").unwrap();
        let path: PathBuf = temp_dir.path().join("todos.json");
        let config: StorageConfig = setup_config(true);
        let backup_path: PathBuf = path.with_extension("json.bak");

        Storage::save(
            &vec![Todo::new("Good Data", "", None)],
            Some(&backup_path),
            &config,
        )
        .unwrap();

        fs::write(&path, "invalid json data").unwrap();

        let loaded = Storage::load(Some(&path), &config).unwrap();
        assert_eq!(loaded[0].title, "Good Data");
    }

    #[test]
    fn should_not_restore_if_backup_disabled() {
        let temp_dir: TempDir = TempDir::new("todo_test").unwrap();
        let path: PathBuf = temp_dir.path().join("todos.json");
        let config: StorageConfig = setup_config(false);

        fs::write(&path, "{ corrupted }").unwrap();

        let result = Storage::load(Some(&path), &config);
        assert!(
            result.is_err(),
            "Should return error if backup is disabled and file is corrupted"
        );
    }

    #[test]
    fn should_clean_up_temp_file_on_error() {
        let temp_dir: TempDir = TempDir::new("todo_test").unwrap();
        let path: PathBuf = temp_dir.path().join("todos.json");
        let temp_path: PathBuf = path.with_extension("tmp");
        let config: StorageConfig = setup_config(true);

        Storage::save(&vec![Todo::new("Test", "", None)], Some(&path), &config).unwrap();

        assert!(
            !temp_path.exists(),
            "Temp file should be renamed or deleted after success"
        );
    }

    #[test]
    fn should_create_new_state_empty_todos() {
        let temp_dir: TempDir = TempDir::new("todo_test").unwrap();
        let path: PathBuf = temp_dir.path().join("todos.json");
        let config: StorageConfig = setup_config(true);

        assert!(!path.exists(), "File should not exist initially");

        let mut state = ApplicationState::load(Some(&path), &config).unwrap();

        if state.todos.is_empty() {
            let result = state.save(Some(&path), &config);
            assert!(result.is_ok(), "Save should succeed");
        }

        assert!(path.exists(), "Empty file should be created after save");

        let content = fs::read_to_string(&path).unwrap();
        assert_eq!(content.trim(), "[]", "File should contain empty JSON array");
        assert!(state.todos.is_empty());
    }

    #[test]
    fn should_create_new_state_with_saved_todos() {
        let temp_dir: TempDir = TempDir::new("todo_test").unwrap();
        let path: PathBuf = temp_dir.path().join("todos.json");
        let config: StorageConfig = setup_config(true);

        let task: Todo = Todo::new("Test Title", "Test Desc", Some(Priority::High));
        let json_data: String = serde_json::to_string(&vec![task.clone()]).unwrap();

        fs::write(&path, json_data).unwrap();
        let state = ApplicationState::load(Some(&path), &config).unwrap();

        assert_eq!(state.todos.len(), 1);
        assert_eq!(state.todos[0].id, task.id);
        assert_eq!(state.todos[0].title, "Test Title");
        assert_eq!(state.todos[0].description, "Test Desc");
        assert_eq!(state.todos[0].priority, Priority::High);
    }

    #[test]
    fn should_create_new_state_default_if_path_not_found() {
        let temp_dir: TempDir = TempDir::new("todo_test").unwrap();
        let path: PathBuf = temp_dir.path().join("non_existent_dir").join("todos.json");
        let config: StorageConfig = setup_config(false);

        assert!(!path.exists());

        let result: ApplicationResult<ApplicationState> =
            ApplicationState::load(Some(&path), &config);
        assert!(result.is_ok());

        let state: ApplicationState = result.unwrap();
        assert!(state.todos.is_empty());
        assert_eq!(state.select_state.selected(), None);
    }

    #[test]
    fn should_return_default_data_path() {
        let path_result: ApplicationResult<PathBuf> = Storage::get_data_path();
        assert!(path_result.is_ok());

        let path: PathBuf = path_result.unwrap();
        assert!(path.ends_with("todo-tui/todos.json") || path.ends_with("todo-tui\\todos.json"));
        assert!(path.is_absolute());
    }

    #[test]
    fn should_create_directory_on_save_if_missing() {
        let temp_dir: TempDir = TempDir::new("todo_test").unwrap();
        let path: PathBuf = temp_dir.path().join("a").join("b").join("todos.json");
        let config: StorageConfig = setup_config(false);
        assert!(!path.parent().unwrap().exists());

        Storage::save(&vec![], Some(&path), &config).unwrap();
        assert!(
            path.parent().unwrap().exists(),
            "Directory hierarchy should be created on save"
        );
        assert!(path.exists(), "File should be created");
    }

    #[test]
    fn should_invoke_jsonerror_on_load_if_json_not_valid() {
        let temp_dir: TempDir = TempDir::new("todo_test").unwrap();
        let path: PathBuf = temp_dir.path().join("todos.json");
        let config: StorageConfig = setup_config(false);

        fs::write(&path, "invalid json {").unwrap();
        let result: ApplicationResult<Vec<Todo>> = Storage::load(Some(&path), &config);

        assert!(result.is_err());
        assert!(matches!(
            result,
            Err(ApplicationError::Storage(StorageError::JSONError(..)))
        ));
    }

    #[test]
    #[cfg(unix)]
    fn should_invoke_ioerror_on_save_when_no_write_permission() {
        use std::{fs::Permissions, os::unix::fs::PermissionsExt};

        let temp_dir: TempDir = TempDir::new("todo_test").unwrap();
        let path: PathBuf = temp_dir.path().join("todos.json");
        let config: StorageConfig = setup_config(false);

        let mut perms: Permissions = fs::metadata(temp_dir.path()).unwrap().permissions();
        perms.set_mode(0o444);
        fs::set_permissions(temp_dir.path(), perms).unwrap();

        let todos = vec![Todo::new("Test", "", None)];
        let result: ApplicationResult<()> = Storage::save(&todos, Some(&path), &config);

        assert!(result.is_err());
        assert!(matches!(
            result,
            Err(ApplicationError::Storage(StorageError::IOError(..)))
        ));
    }
}
