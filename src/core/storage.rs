use crate::{core::StorageError, models::Todo, state::ApplicationResult};
use std::{
    fs::{self, File},
    io::BufWriter,
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
    pub fn save(todos: &[Todo], path: Option<&Path>) -> ApplicationResult<()> {
        let p = match path {
            Some(p) => p.to_path_buf(),
            None => Self::get_data_path()?,
        };

        if let Some(parent) = p.parent() {
            fs::create_dir_all(parent).map_err(|err| StorageError::IOError(err.to_string()))?;
        }

        let file: File = File::create(p).map_err(|err| StorageError::IOError(err.to_string()))?;
        let writer: BufWriter<File> = BufWriter::new(file);

        serde_json::to_writer_pretty(writer, todos)
            .map_err(|err| StorageError::JSONError(err.to_string()))?;
        Ok(())
    }

    /// Load todos from a user path/default path
    pub fn load(path: Option<&Path>) -> ApplicationResult<Vec<Todo>> {
        let p = match path {
            Some(p) => p.to_path_buf(),
            None => Self::get_data_path()?,
        };

        if !p.exists() {
            return Ok(Vec::new());
        }

        let file: File = File::open(p).map_err(|err| StorageError::IOError(err.to_string()))?;
        let todos: Vec<Todo> = serde_json::from_reader(file)
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

    #[test]
    fn should_save_and_load_todos_successfully() {
        let temp_dir: TempDir = TempDir::new("todo_test").unwrap();
        let path: PathBuf = temp_dir.path().join("todos.json");

        let todos = vec![Todo::new("Task 1", "", None), Todo::new("Task 2", "", None)];
        let result: ApplicationResult<()> = Storage::save(&todos, Some(&path));
        assert!(result.is_ok());

        let loaded_todos: Vec<Todo> = Storage::load(Some(&path)).unwrap();

        assert_eq!(loaded_todos.len(), 2);
        assert_eq!(loaded_todos[0].title, "Task 1");
        assert_eq!(loaded_todos[1].title, "Task 2");
    }

    #[test]
    fn should_create_new_state_empty_todos() {
        let temp_dir: TempDir = TempDir::new("todo_test").unwrap();
        let path: PathBuf = temp_dir.path().join("todos.json");

        assert!(!path.exists(), "File should not exist initially");

        let mut state = ApplicationState::load(Some(&path)).unwrap();

        if state.todos.is_empty() {
            let result = state.save(Some(&path));
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

        let task: Todo = Todo::new("Test Title", "Test Desc", Some(Priority::High));
        let json_data: String = serde_json::to_string(&vec![task.clone()]).unwrap();

        fs::write(&path, json_data).unwrap();
        let state = ApplicationState::load(Some(&path)).unwrap();

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
        assert!(!path.exists());

        let result: ApplicationResult<ApplicationState> = ApplicationState::load(Some(&path));
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
        assert!(!path.parent().unwrap().exists());

        Storage::save(&vec![], Some(&path)).unwrap();
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

        fs::write(&path, "invalid json {").unwrap();
        let result: ApplicationResult<Vec<Todo>> = Storage::load(Some(&path));

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

        let mut perms: Permissions = fs::metadata(temp_dir.path()).unwrap().permissions();
        perms.set_mode(0o444);
        fs::set_permissions(temp_dir.path(), perms).unwrap();

        let todos = vec![Todo::new("Test", "", None)];
        let result: ApplicationResult<()> = Storage::save(&todos, Some(&path));

        assert!(result.is_err());
        assert!(matches!(
            result,
            Err(ApplicationError::Storage(StorageError::IOError(..)))
        ));
    }
}
