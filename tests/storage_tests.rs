use std::{fs, path::PathBuf};
use tempdir::TempDir;
use toodles::{
    config::StorageConfig,
    core::{ApplicationError, FocusArea, Storage, StorageError},
    models::Task,
    state::{Session, SidebarTab, TasksStateSave},
};

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

    let tasks: Vec<Task> = vec![Task::new("Task 1"), Task::new("Task 2")];
    let session: Session = Session::default();
    let folders = vec![];

    storage
        .save(&TasksStateSave::new(&tasks, &folders, &session))
        .unwrap();
    let loaded_data = storage.load().unwrap();

    assert_eq!(loaded_data.tasks.len(), 2);
    assert_eq!(loaded_data.tasks[0].title, "Task 1");
    assert_eq!(loaded_data.session.last_tab.value, SidebarTab::Inbox);
    assert_eq!(loaded_data.session.last_folder_id, None);
    assert_eq!(loaded_data.session.last_focus.value, FocusArea::Main);
}

#[test]
fn should_create_backup_on_init_if_file_exists() {
    let temp_dir = TempDir::new("task_test").unwrap();
    let path = temp_dir.path().join("toodles.db");
    let config = setup_config(true);
    let backup_path = path.with_extension("db.bak");

    let mut storage = Storage::init(Some(&path), &config).unwrap();
    assert!(!backup_path.exists());

    let tasks: Vec<Task> = vec![Task::new("Task 1"), Task::new("Task 2")];
    let session: Session = Session::default();
    let folders = vec![];

    storage
        .save(&TasksStateSave::new(&tasks, &folders, &session))
        .unwrap();
    drop(storage);

    let _storage2 = Storage::init(Some(&path), &config).unwrap();
    assert!(path.exists());
    assert!(backup_path.exists());
    drop(_storage2);

    let backup_storage = Storage::init(Some(&backup_path), &config).unwrap();
    let backup_data = backup_storage.load().unwrap();
    assert_eq!(backup_data.tasks.len(), 2);
    assert_eq!(backup_data.tasks[0].title, "Task 1");
    assert_eq!(backup_data.tasks[1].title, "Task 2");
}

#[test]
fn should_create_directory_on_init_if_missing() {
    let temp_dir = TempDir::new("task_test").unwrap();
    let path = temp_dir.path().join("a").join("b").join("toodles.db");
    let config = setup_config(false);
    assert!(!path.parent().unwrap().exists());

    let _storage = Storage::init(Some(&path), &config).unwrap();
    assert!(path.parent().unwrap().exists());
    assert!(path.exists());
}

#[test]
fn should_invoke_database_error_on_init_if_corrupted() {
    let temp_dir: TempDir = TempDir::new("task_test").unwrap();
    let path: PathBuf = temp_dir.path().join("toodles.db");
    let config: StorageConfig = setup_config(false);

    fs::write(&path, "not a sqlite database file").unwrap();
    let result = Storage::init(Some(&path), &config);

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

    let result = Storage::init(Some(&path), &config);
    assert!(result.is_err());
}
