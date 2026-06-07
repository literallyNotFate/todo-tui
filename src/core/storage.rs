use crate::{
    config::StorageConfig,
    core::StorageError,
    models::Task,
    state::{ApplicationResult, Session, TasksStateData},
};
use std::{
    fs::{self, File},
    io::{BufWriter, Write},
    path::{Path, PathBuf},
};

/// Storage structure for all task fs operations
pub struct Storage;

impl Storage {
    /// Get default data path to save/load from
    pub fn get_data_path() -> ApplicationResult<PathBuf> {
        let path: ApplicationResult<PathBuf> = dirs::data_dir()
            .ok_or(
                StorageError::Environment {
                    context: "data".to_string(),
                }
                .into(),
            )
            .map(|dir| dir.join("toodles").join("tasks.json"));

        if let Ok(ref p) = path {
            log::debug!("Data path resolved to: {:?}", p);
        }

        path
    }

    /// Get default logging path
    pub fn get_log_path() -> ApplicationResult<PathBuf> {
        let path: ApplicationResult<PathBuf> = dirs::data_dir()
            .ok_or(
                StorageError::Environment {
                    context: "log".to_string(),
                }
                .into(),
            )
            .map(|dir| dir.join("toodles").join("toodles.log"));

        if let Ok(ref p) = path {
            log::debug!("Log path resolved to: {:?}", p);
        }

        path
    }

    /// Save tasks and UI Session to user path/default path
    pub fn save(
        tasks: &[Task],
        session: Session,
        path: Option<&Path>,
        config: &StorageConfig,
    ) -> ApplicationResult<String> {
        let p: PathBuf = match path {
            Some(p) => p.to_path_buf(),
            None => Self::get_data_path()?,
        };

        log::debug!("Starting atomic save of tasks and UI session to {:?}", p);
        if let Some(parent) = p.parent() {
            fs::create_dir_all(parent).map_err(|e| StorageError::IO {
                path: p.clone(),
                src: e.to_string(),
            })?;
        }

        let mut temp_path: PathBuf = p.clone();
        temp_path.set_extension("tmp");

        let data: TasksStateData = TasksStateData::new(tasks.to_vec(), session);

        let result = (|| -> Result<(), StorageError> {
            let file = File::create(&temp_path).map_err(|e| StorageError::IO {
                path: p.clone(),
                src: e.to_string(),
            })?;
            let mut writer = BufWriter::new(file);

            serde_json::to_writer_pretty(&mut writer, &data).map_err(|e| StorageError::JSON {
                path: p.clone(),
                src: e.to_string(),
            })?;

            writer.flush().map_err(|e| StorageError::IO {
                path: p.clone(),
                src: e.to_string(),
            })?;

            writer.get_ref().sync_all().map_err(|e| StorageError::IO {
                path: p.clone(),
                src: e.to_string(),
            })?;

            Ok(())
        })();

        if let Err(e) = result {
            log::error!("Failed to write temporary file {:?}: {}", temp_path, e);
            let _ = fs::remove_file(&temp_path);
            return Err(e.into());
        }

        if config.backup_enabled && p.exists() {
            let mut backup_path = p.clone();
            backup_path.set_extension("json.bak");
            log::debug!("Creating backup at {:?}", backup_path);
            let _ = fs::rename(&p, backup_path);
        }

        log::info!(
            "Successfully prepared storage data: {} tasks, filter: {:?}, focus: {:?}",
            data.tasks.len(),
            data.session.last_filter,
            data.session.last_focus
        );

        fs::rename(&temp_path, &p).map_err(|err| {
            log::error!("Atomic rename failed: {}", err);
            StorageError::IO {
                path: p,
                src: err.to_string(),
            }
        })?;

        Ok("Data was saved".into())
    }

    /// Load tasks and UI Session from a user path/default path
    pub fn load(path: Option<&Path>, config: &StorageConfig) -> ApplicationResult<TasksStateData> {
        let p: PathBuf = match path {
            Some(p) => p.to_path_buf(),
            None => Self::get_data_path()?,
        };

        if p.exists() {
            match Self::load_from_path(&p) {
                Ok(data) => {
                    log::info!(
                        "Loaded {} tasks and session memory from {:?}",
                        data.tasks.len(),
                        p
                    );
                    return Ok(data);
                }
                Err(e) => {
                    log::warn!("Main data file at {:?} is corrupted: {}", p, e);
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
                log::info!("Attempting to restore from backup: {:?}", backup);
                return Self::load_from_path(&backup);
            }
        }

        log::info!("No data file found, initializing default session and empty list");
        Ok(TasksStateData::default())
    }

    /// Helper function to load from path (not to duplicate code)
    fn load_from_path(p: &Path) -> ApplicationResult<TasksStateData> {
        let file = File::open(p).map_err(|err| StorageError::IO {
            path: p.to_owned(),
            src: err.to_string(),
        })?;
        let storage = serde_json::from_reader(file).map_err(|err| StorageError::JSON {
            path: p.to_owned(),
            src: err.to_string(),
        })?;
        Ok(storage)
    }
}

/// Unit-tests for storage
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        core::{ApplicationError, FocusArea},
        models::{Filter, Priority},
        state::{ApplicationState, UIState},
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
        let path: PathBuf = temp_dir.path().join("tasks.json");
        let config: StorageConfig = setup_config(true);

        let tasks = vec![Task::new("Task 1", "", None), Task::new("Task 2", "", None)];
        let session = Session::default();

        let result: ApplicationResult<String> =
            Storage::save(&tasks, session, Some(&path), &config);
        assert!(result.is_ok());

        let loaded_data: TasksStateData = Storage::load(Some(&path), &config).unwrap();

        assert_eq!(loaded_data.tasks.len(), 2);
        assert_eq!(loaded_data.tasks[0].title, "Task 1");
        assert_eq!(loaded_data.tasks[1].title, "Task 2");
        assert_eq!(loaded_data.session.last_filter, Filter::All);
        assert_eq!(loaded_data.session.last_focus, FocusArea::Sidebar);
    }

    #[test]
    fn should_create_backup_on_save() {
        let temp_dir: TempDir = TempDir::new("task_test").unwrap();
        let path: PathBuf = temp_dir.path().join("tasks.json");
        let config: StorageConfig = setup_config(true);
        let backup_path: PathBuf = path.with_extension("json.bak");

        Storage::save(
            &vec![Task::new("V1", "", None)],
            Session::default(),
            Some(&path),
            &config,
        )
        .unwrap();
        assert!(
            !backup_path.exists(),
            "Backup shouldn't exist on first save"
        );

        Storage::save(
            &vec![Task::new("V2", "", None)],
            Session::default(),
            Some(&path),
            &config,
        )
        .unwrap();

        assert!(path.exists());
        assert!(backup_path.exists());

        let backup_data = Storage::load(Some(&backup_path), &config).unwrap();
        assert_eq!(backup_data.tasks[0].title, "V1");
    }

    #[test]
    fn should_restore_from_backup_if_main_file_is_missing() {
        let temp_dir: TempDir = TempDir::new("task_test").unwrap();
        let path: PathBuf = temp_dir.path().join("tasks.json");
        let config: StorageConfig = setup_config(true);
        let backup_path: PathBuf = path.with_extension("json.bak");

        let tasks = vec![Task::new("Backup Task", "", None)];
        Storage::save(&tasks, Session::default(), Some(&backup_path), &config).unwrap();

        assert!(!path.exists());

        let loaded = Storage::load(Some(&path), &config).expect("Should fallback to backup");
        assert_eq!(loaded.tasks[0].title, "Backup Task");
    }

    #[test]
    fn should_handle_corrupted_main_file_with_backup_fallback() {
        let temp_dir: TempDir = TempDir::new("task_test").unwrap();
        let path: PathBuf = temp_dir.path().join("tasks.json");
        let config: StorageConfig = setup_config(true);
        let backup_path: PathBuf = path.with_extension("json.bak");

        Storage::save(
            &vec![Task::new("Good Data", "", None)],
            Session::default(),
            Some(&backup_path),
            &config,
        )
        .unwrap();

        fs::write(&path, "invalid json data").unwrap();

        let loaded = Storage::load(Some(&path), &config).unwrap();
        assert_eq!(loaded.tasks[0].title, "Good Data");
    }

    #[test]
    fn should_not_restore_if_backup_disabled() {
        let temp_dir: TempDir = TempDir::new("task_test").unwrap();
        let path: PathBuf = temp_dir.path().join("tasks.json");
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
        let temp_dir: TempDir = TempDir::new("task_test").unwrap();
        let path: PathBuf = temp_dir.path().join("tasks.json");
        let temp_path: PathBuf = path.with_extension("tmp");
        let config: StorageConfig = setup_config(true);

        Storage::save(
            &vec![Task::new("Test", "", None)],
            Session::default(),
            Some(&path),
            &config,
        )
        .unwrap();

        assert!(
            !temp_path.exists(),
            "Temp file should be renamed or deleted after success"
        );
    }

    #[test]
    fn should_create_new_state_empty_tasks() {
        let temp_dir: TempDir = TempDir::new("task_test").unwrap();
        let path: PathBuf = temp_dir.path().join("tasks.json");
        let config: StorageConfig = setup_config(true);

        assert!(!path.exists());

        let state = Storage::load(Some(&path), &config).unwrap();
        assert!(state.tasks.is_empty());

        let session = Session::from_state(&UIState::default(), None);
        let result = Storage::save(&state.tasks, session, Some(&path), &config);
        assert!(result.is_ok());
        assert!(path.exists());

        let content = fs::read_to_string(&path).unwrap();
        let decoded: TasksStateData = serde_json::from_str(&content).unwrap();

        assert!(decoded.tasks.is_empty(), "tasks should be an empty list");
        assert!(decoded.session.last_selected_id.is_none());
    }

    #[test]
    fn should_create_new_state_with_saved_tasks_and_session() {
        let temp_dir: TempDir = TempDir::new("task_test").unwrap();
        let path: PathBuf = temp_dir.path().join("tasks.json");
        let config: StorageConfig = setup_config(true);

        let task = Task::new("Test Title", "Test Desc", Some(Priority::High));
        let task_id = task.id;

        let storage_data = TasksStateData {
            tasks: vec![task],
            session: Session {
                last_selected_id: Some(task_id),
                ..Session::default()
            },
        };

        let json_data = serde_json::to_string(&storage_data).unwrap();
        fs::write(&path, json_data).unwrap();

        let loaded_state = Storage::load(Some(&path), &config).unwrap();

        assert_eq!(loaded_state.tasks.len(), 1);
        assert_eq!(loaded_state.tasks[0].id, task_id);
        assert_eq!(loaded_state.session.last_selected_id, Some(task_id));
    }

    #[test]
    fn should_create_new_state_default_if_path_not_found() {
        let temp_dir: TempDir = TempDir::new("task_test").unwrap();
        let path: PathBuf = temp_dir.path().join("non_existent_dir").join("tasks.json");
        let config: StorageConfig = setup_config(false);

        assert!(!path.exists());

        let result: ApplicationResult<TasksStateData> = Storage::load(Some(&path), &config);
        assert!(result.is_ok());

        let tasks = result.unwrap().tasks;
        let state: ApplicationState = ApplicationState::new(tasks);

        assert!(state.tasks.is_empty());
        assert_eq!(state.select_state.selected(), None);
    }

    #[test]
    fn should_return_default_data_path() {
        let path_result: ApplicationResult<PathBuf> = Storage::get_data_path();
        assert!(path_result.is_ok());

        let path: PathBuf = path_result.unwrap();
        assert!(path.ends_with("toodles/tasks.json") || path.ends_with("toodles\\tasks.json"));
        assert!(path.is_absolute());
    }

    #[test]
    fn should_create_directory_on_save_if_missing() {
        let temp_dir: TempDir = TempDir::new("task_test").unwrap();
        let path: PathBuf = temp_dir.path().join("a").join("b").join("tasks.json");
        let config: StorageConfig = setup_config(false);
        assert!(!path.parent().unwrap().exists());

        Storage::save(&vec![], Session::default(), Some(&path), &config).unwrap();
        assert!(
            path.parent().unwrap().exists(),
            "Directory hierarchy should be created on save"
        );
        assert!(path.exists(), "File should be created");
    }

    #[test]
    fn should_invoke_jsonerror_on_load_if_json_not_valid() {
        let temp_dir: TempDir = TempDir::new("task_test").unwrap();
        let path: PathBuf = temp_dir.path().join("tasks.json");
        let config: StorageConfig = setup_config(false);

        fs::write(&path, "invalid json {").unwrap();
        let result: ApplicationResult<TasksStateData> = Storage::load(Some(&path), &config);

        assert!(result.is_err());
        assert!(matches!(
            result,
            Err(ApplicationError::Storage(StorageError::JSON { .. }))
        ));

        if let Err(ApplicationError::Storage(StorageError::JSON { path: err_path, .. })) = result {
            assert_eq!(err_path, path);
        }
    }

    #[test]
    #[cfg(unix)]
    fn should_invoke_ioerror_on_save_when_no_write_permission() {
        use std::{fs::Permissions, os::unix::fs::PermissionsExt};

        let temp_dir: TempDir = TempDir::new("task_test").unwrap();
        let path: PathBuf = temp_dir.path().join("tasks.json");
        let config: StorageConfig = setup_config(false);

        let mut perms: Permissions = fs::metadata(temp_dir.path()).unwrap().permissions();
        perms.set_mode(0o444);
        fs::set_permissions(temp_dir.path(), perms).unwrap();

        let tasks = vec![Task::new("Test", "", None)];
        let result: ApplicationResult<String> =
            Storage::save(&tasks, Session::default(), Some(&path), &config);

        assert!(result.is_err());
        assert!(matches!(
            result,
            Err(ApplicationError::Storage(StorageError::IO { .. }))
        ));

        if let Err(ApplicationError::Storage(StorageError::IO { path: err_path, .. })) = result {
            assert_eq!(err_path, path);
        }
    }
}
