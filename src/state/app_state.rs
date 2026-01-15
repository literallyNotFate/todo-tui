use crate::{
    models::Todo,
    state::{ApplicationError, StorageError, TodoError},
    utils::constants::text::{CLEARED_TASKS_TEXT, REMOVED_TASK_TEXT, SAVED_TASKS_TEXT},
};
use ratatui::widgets::ListState;
use std::{
    fs::{self, File},
    io::BufWriter,
    path::{Path, PathBuf},
};

#[derive(Debug, Default)]
pub struct ApplicationState {
    pub todos: Vec<Todo>,
    pub select_state: ListState,
}

pub type ApplicationResult<T> = Result<T, ApplicationError>;

impl ApplicationState {
    pub fn new() -> Self {
        let state = Self::load().unwrap_or_default();
        if state.todos.is_empty() {
            let _ = state.save();
        }
        state
    }

    // Main service todo
    pub fn append_todo(&mut self, new_title: impl Into<String>) -> ApplicationResult<String> {
        let title: String = new_title.into();

        if title.is_empty() {
            return Err(TodoError::EmptyTitle.into());
        }

        if self.todo_by_title(&title).is_some() {
            return Err(TodoError::TaskAlreadyExists(title).into());
        }

        self.todos.push(Todo::new(&title));
        self.select_state.select(Some(self.todos.len() - 1));
        Ok(format!("Task {} was added to the list!", title))
    }

    pub fn rename_todo(&mut self, new_title: impl Into<String>) -> ApplicationResult<String> {
        let new_title: String = new_title.into();

        if new_title.is_empty() {
            return Err(TodoError::EmptyTitle.into());
        }

        let index: usize = self
            .select_state
            .selected()
            .ok_or(TodoError::TaskNotSelected)?;

        let current_title: &String = &self.todos[index].title;
        if new_title != *current_title && self.todo_by_title(&new_title).is_some() {
            return Err(TodoError::TaskAlreadyExists(new_title).into());
        }

        self.todos[index].rename(&new_title);
        Ok(format!(
            "Task ({} / {}) was renamed to {}!",
            index + 1,
            self.todos.len(),
            new_title
        ))
    }

    pub fn remove_todo(&mut self) -> ApplicationResult<String> {
        if self.todos.is_empty() {
            return Err(TodoError::CannotRemoveFromEmpty.into());
        }

        let index: usize = self
            .select_state
            .selected()
            .ok_or(TodoError::TaskNotSelected)?;

        self.todos.remove(index);

        if self.todos.is_empty() {
            self.select_state.select(None);
        } else {
            let new_index = index.min(self.todos.len() - 1);
            self.select_state.select(Some(new_index));
        }

        Ok(String::from(REMOVED_TASK_TEXT))
    }

    pub fn toggle_current(&mut self) {
        if let Some(index) = self.select_state.selected() {
            self.todos[index].toggle_done();
        }
    }

    pub fn clear_todos(&mut self) -> ApplicationResult<String> {
        if self.todos.is_empty() {
            return Err(TodoError::ListEmpty.into());
        }

        self.todos = Vec::new();
        self.select_state.select(None);

        Ok(String::from(CLEARED_TASKS_TEXT))
    }

    // Save and load
    pub fn load() -> ApplicationResult<Self> {
        let path = Self::get_data_path()?;
        Self::load_from_path(&path)
    }

    pub fn save(&self) -> ApplicationResult<String> {
        let path = Self::get_data_path()?;
        Self::save_to_path(self, &path)
    }

    // Other actions/helper functions
    pub fn current_todo(&self) -> Option<&Todo> {
        self.select_state.selected().and_then(|i| self.todos.get(i))
    }

    pub fn todo_by_title(&self, target: impl Into<String>) -> Option<&Todo> {
        let title: String = target.into();
        self.todos.iter().find(|todo| todo.title == title)
    }

    fn load_from_path(path: &Path) -> ApplicationResult<Self> {
        if let Some(parent) = path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)
                    .map_err(|_| ApplicationError::Storage(StorageError::IOError))?;
            }
        }

        if !path.exists() {
            return Ok(Self {
                todos: Vec::new(),
                select_state: ListState::default(),
            });
        }

        let file: File = File::open(&path).map_err(|_| StorageError::IOError)?;
        let todos: Vec<Todo> =
            serde_json::from_reader(file).map_err(|_| StorageError::JSONError)?;

        let mut state = Self {
            todos,
            select_state: ListState::default(),
        };

        state.select_state.select_last();
        Ok(state)
    }

    fn save_to_path(&self, path: &Path) -> ApplicationResult<String> {
        fs::create_dir_all(path.parent().unwrap()).map_err(|_| StorageError::IOError)?;

        let file: File = File::create(path).map_err(|_| StorageError::IOError)?;
        let writer: BufWriter<File> = BufWriter::new(file);

        serde_json::to_writer_pretty(writer, &self.todos).map_err(|_| StorageError::JSONError)?;
        Ok(String::from(SAVED_TASKS_TEXT))
    }

    pub fn get_data_path() -> ApplicationResult<PathBuf> {
        dirs::data_dir()
            .ok_or(StorageError::PathNotFound.into())
            .map(|dir| dir.join("todo-tui").join("todos.json"))
    }
}

// Unit-tests for ApplicationState
#[cfg(test)]
mod tests {
    use super::*;
    use tempdir::TempDir;

    // Helper function to setup list with multiple tasks (non empty)
    fn setup_with_n_todos(n: usize) -> ApplicationState {
        let mut state: ApplicationState = ApplicationState::default();
        for i in 1..=n {
            let _: ApplicationResult<String> = state.append_todo(format!("Task {}", i));
        }

        state
    }

    #[test]
    fn should_create_new_state_empty_todos() {
        let temp_dir: TempDir = TempDir::new("todo_test").unwrap();
        let path: PathBuf = temp_dir.path().join("todos.json");

        assert!(!path.exists(), "File should not exist initially");

        let state = ApplicationState::load_from_path(&path).unwrap();

        if state.todos.is_empty() {
            let result = state.save_to_path(&path);
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

        fs::write(&path, r#"[{"title":"Test","done":false}]"#).unwrap();
        let state = ApplicationState::load_from_path(&path).unwrap();

        assert_eq!(state.todos.len(), 1);
        assert_eq!(state.todos[0].title, "Test");
        assert!(!state.todos[0].done);
    }

    #[test]
    fn should_create_new_state_default_if_path_not_found() {
        let temp_dir: TempDir = TempDir::new("todo_test").unwrap();
        let path: PathBuf = temp_dir.path().join("non_existent_dir").join("todos.json");
        assert!(!path.exists());

        let result: ApplicationResult<ApplicationState> = ApplicationState::load_from_path(&path);
        assert!(result.is_ok());

        let state: ApplicationState = result.unwrap();
        assert!(state.todos.is_empty());
        assert_eq!(state.select_state.selected(), None);
    }

    #[test]
    fn should_append_todo() {
        let mut state: ApplicationState = ApplicationState::default();
        let result: ApplicationResult<String> = state.append_todo("Test");

        assert_eq!(result, Ok(String::from("Task Test was added to the list!")));
        assert_eq!(state.todos.len(), 1);
        assert_eq!(state.todos[0].title, "Test");
        assert!(!state.todos[0].done);
        assert_eq!(state.select_state.selected(), Some(0));
    }

    #[test]
    fn should_invoke_empty_title_error_on_append() {
        let mut state: ApplicationState = ApplicationState::default();
        let result: ApplicationResult<String> = state.append_todo("");

        assert_eq!(result, Err(ApplicationError::Todo(TodoError::EmptyTitle)));
        assert!(state.todos.is_empty());
    }

    #[test]
    fn should_invoke_task_exists_on_append() {
        let mut state: ApplicationState = setup_with_n_todos(2);
        let title: String = String::from("Task 1");
        let result: ApplicationResult<String> = state.append_todo(&title);

        assert_eq!(
            result,
            Err(ApplicationError::Todo(TodoError::TaskAlreadyExists(title)))
        );
        assert_eq!(state.todos.len(), 2);
    }

    #[test]
    fn should_select_last_after_multiple_append() {
        let state: ApplicationState = setup_with_n_todos(3);

        assert_eq!(state.todos.len(), 3);
        assert_eq!(state.select_state.selected(), Some(2));
        assert_eq!(state.todos[2].title, "Task 3");
    }

    #[test]
    fn should_rename_todo() {
        let mut state: ApplicationState = setup_with_n_todos(2);
        assert_eq!(state.todos[1].title, "Task 2");

        let new_title: &str = "Renamed task";
        let result: ApplicationResult<String> = state.rename_todo(new_title);

        assert_eq!(
            result,
            Ok(format!(
                "Task ({} / {}) was renamed to {}!",
                2, 2, new_title
            )),
        );
        assert_eq!(state.todos[1].title, new_title);
        assert_eq!(state.todos[0].title, "Task 1");
    }

    #[test]
    fn should_invoke_empty_title_error_on_rename() {
        let mut state: ApplicationState = setup_with_n_todos(1);
        let result: ApplicationResult<String> = state.rename_todo("");

        assert_eq!(result, Err(ApplicationError::Todo(TodoError::EmptyTitle)));
        assert_eq!(state.todos[0].title, "Task 1");
    }

    #[test]
    fn should_invoke_task_exists_on_rename() {
        let mut state: ApplicationState = setup_with_n_todos(3);
        let title: String = "Task 1".to_string();
        let result: ApplicationResult<String> = state.rename_todo(&title);

        assert_eq!(
            result,
            Err(ApplicationError::Todo(TodoError::TaskAlreadyExists(title)))
        );
        assert_eq!(state.todos[2].title, "Task 3");
    }

    #[test]
    fn should_invoke_not_selected_error_on_rename() {
        let mut state: ApplicationState = ApplicationState::default();
        state.select_state.select(None);
        let result: ApplicationResult<String> = state.rename_todo("Should fail");

        assert_eq!(
            result,
            Err(ApplicationError::Todo(TodoError::TaskNotSelected))
        );
    }

    #[test]
    fn should_remove_todo_in_the_middle() {
        let mut state: ApplicationState = setup_with_n_todos(3);
        state.select_state.select(Some(1));
        let result: ApplicationResult<String> = state.remove_todo();

        assert_eq!(result, Ok(String::from(REMOVED_TASK_TEXT)));
        assert_eq!(state.todos.len(), 2);
        assert_eq!(state.todos[0].title, "Task 1");
        assert_eq!(state.todos[1].title, "Task 3");
        assert_eq!(state.select_state.selected(), Some(1));
    }

    #[test]
    fn should_remove_todo_last() {
        let mut state: ApplicationState = setup_with_n_todos(2);
        state.select_state.select(Some(1));
        let result: ApplicationResult<String> = state.remove_todo();

        assert_eq!(result, Ok(String::from(REMOVED_TASK_TEXT)));
        assert_eq!(state.todos.len(), 1);
        assert_eq!(state.todos[0].title, "Task 1");
        assert_eq!(state.select_state.selected(), Some(0));
    }

    #[test]
    fn should_remove_only_one_element() {
        let mut state: ApplicationState = setup_with_n_todos(1);
        let result: ApplicationResult<String> = state.remove_todo();

        assert_eq!(result, Ok(String::from(REMOVED_TASK_TEXT)));
        assert!(state.todos.is_empty());
        assert_eq!(state.select_state.selected(), None);
    }

    #[test]
    fn should_invoke_cannot_remove_empty_error_on_remove() {
        let mut state: ApplicationState = ApplicationState::default();
        let result: ApplicationResult<String> = state.remove_todo();

        assert_eq!(
            result,
            Err(ApplicationError::Todo(TodoError::CannotRemoveFromEmpty))
        );
    }

    #[test]
    fn should_invoke_not_selected_error_on_remove() {
        let mut state: ApplicationState = setup_with_n_todos(1);
        state.select_state.select(None);
        let result: ApplicationResult<String> = state.remove_todo();

        assert_eq!(
            result,
            Err(ApplicationError::Todo(TodoError::TaskNotSelected))
        );
    }

    #[test]
    fn should_toggle_current_todo() {
        let mut state: ApplicationState = setup_with_n_todos(2);
        state.select_state.select(Some(0));

        assert!(!state.todos[0].done);

        state.toggle_current();
        assert!(state.todos[0].done);

        state.toggle_current();
        assert!(!state.todos[0].done);
    }

    #[test]
    fn should_toggle_current_on_empty() {
        let mut state: ApplicationState = ApplicationState::default();
        state.select_state.select(None);
        state.toggle_current();

        assert!(state.todos.is_empty());
    }

    #[test]
    fn should_clear_todos() {
        let mut state: ApplicationState = setup_with_n_todos(5);
        let result: ApplicationResult<String> = state.clear_todos();

        assert_eq!(result, Ok(String::from(CLEARED_TASKS_TEXT)));
        assert!(state.todos.is_empty());
    }

    #[test]
    fn should_invoke_list_empty_on_clear() {
        let mut state: ApplicationState = ApplicationState::default();
        let result: ApplicationResult<String> = state.clear_todos();

        assert_eq!(result, Err(ApplicationError::Todo(TodoError::ListEmpty)));
        assert!(state.todos.is_empty());
    }

    #[test]
    fn should_return_current_todo() {
        let mut state: ApplicationState = setup_with_n_todos(3);
        state.select_state.select(Some(1));
        let current: Option<&Todo> = state.current_todo();

        assert!(current.is_some());
        assert_eq!(current.unwrap().title, "Task 2");
    }

    #[test]
    fn should_return_none_if_no_selection_found() {
        let mut state: ApplicationState = setup_with_n_todos(3);
        state.select_state.select(None);

        assert!(state.current_todo().is_none());
    }

    #[test]
    fn should_return_none_if_list_empty() {
        let state: ApplicationState = ApplicationState::default();
        assert!(state.current_todo().is_none());
    }

    #[test]
    fn should_return_last_todo_if_is_out_of_bounds() {
        let mut state: ApplicationState = setup_with_n_todos(1);
        state.select_state.select(Some(999));

        assert!(state.current_todo().is_none());
    }

    #[test]
    fn should_save_and_load_todos_successfully() {
        let temp_dir: TempDir = TempDir::new("todo_test").unwrap();
        let path: PathBuf = temp_dir.path().join("todos.json");

        let mut state = ApplicationState::default();
        state.append_todo("Task 1").unwrap();
        state.append_todo("Task 2").unwrap();

        let result: ApplicationResult<String> = state.save_to_path(&path);
        assert_eq!(result, Ok(String::from(SAVED_TASKS_TEXT)));

        let loaded: ApplicationState = ApplicationState::load_from_path(&path).unwrap();

        assert_eq!(loaded.todos.len(), 2);
        assert_eq!(loaded.todos[0].title, "Task 1");
        assert_eq!(loaded.todos[1].title, "Task 2");
    }

    #[test]
    fn should_create_directory_on_load_if_missing_and_return_default() {
        let temp_dir: TempDir = TempDir::new("todo_test").unwrap();
        let path: PathBuf = temp_dir.path().join("new_subdir").join("todos.json");

        assert!(!path.parent().unwrap().exists());

        let state: ApplicationState = ApplicationState::load_from_path(&path).unwrap();

        assert!(
            path.parent().unwrap().exists(),
            "Directory should be created"
        );
        assert!(state.todos.is_empty());
    }

    #[test]
    fn should_invoke_jsonerror_on_load_if_json_not_valid() {
        let temp_dir: TempDir = TempDir::new("todo_test").unwrap();
        let path: PathBuf = temp_dir.path().join("todos.json");

        fs::write(&path, "invalid json {").unwrap();
        let result: ApplicationResult<ApplicationState> = ApplicationState::load_from_path(&path);

        assert!(result.is_err());
        assert!(matches!(
            result,
            Err(ApplicationError::Storage(StorageError::JSONError))
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

        let state: ApplicationState = ApplicationState::default();
        let result: ApplicationResult<String> = state.save_to_path(&path);

        assert!(result.is_err());
        assert!(matches!(
            result,
            Err(ApplicationError::Storage(StorageError::IOError))
        ));
    }
}
