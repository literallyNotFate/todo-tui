use crate::{
    models::{Filter, Priority, Todo},
    state::{AdaptiveScroll, ApplicationError, StorageError, TodoError},
    ui::Notification,
};
use chrono::Local;
use ratatui::widgets::TableState;
use std::{
    fs::{self, File},
    hash::{DefaultHasher, Hash, Hasher},
    io::BufWriter,
    path::{Path, PathBuf},
};
use uuid::Uuid;

#[derive(Debug, Default)]
pub struct ApplicationState {
    pub todos: Vec<Todo>,
    pub select_state: TableState,
    pub scroll: AdaptiveScroll,

    pub notification: Option<Notification>,
    pub saved_todos_hash: u64,
}

pub type ApplicationResult<T> = Result<T, ApplicationError>;

impl ApplicationState {
    pub fn new() -> Self {
        let mut state = Self::load().unwrap_or_default();
        if state.todos.is_empty() {
            let _ = state.save();
        }

        state
    }

    // Create default state (for testing usually)
    pub fn default() -> Self {
        Self {
            todos: Vec::new(),
            select_state: TableState::default(),
            notification: None,
            saved_todos_hash: 0,
            scroll: AdaptiveScroll::default(),
        }
    }

    // Next task
    pub fn next_task(&mut self) {
        if self.todos.is_empty() {
            self.select_state.select(None);
            return;
        }

        let i = match self.select_state.selected() {
            Some(i) => {
                if i >= self.todos.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };

        self.scroll.reset();
        self.select_state.select(Some(i));
    }

    // Previous task
    pub fn prev_task(&mut self) {
        if self.todos.is_empty() {
            self.select_state.select(None);
            return;
        }

        let i = match self.select_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.todos.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };

        self.scroll.reset();
        self.select_state.select(Some(i));
    }

    // Check if there any unsaved changes by comparing hash
    pub fn any_unsaved_changes(&self) -> bool {
        self.calculate_todos_hash() != self.saved_todos_hash
    }

    // Move tasks
    pub fn move_task_up(&mut self) {
        if let Some(i) = self.select_state.selected() {
            if i > 0 {
                if self.todos[i].priority == self.todos[i - 1].priority {
                    self.todos.swap(i, i - 1);
                    self.select_state.select(Some(i - 1));
                }
            }
        }
    }

    pub fn move_task_down(&mut self) {
        if let Some(i) = self.select_state.selected() {
            if i < self.todos.len() - 1 {
                if self.todos[i].priority == self.todos[i + 1].priority {
                    self.todos.swap(i, i + 1);
                    self.select_state.select(Some(i + 1));
                }
            }
        }
    }

    // Stabilize soritng order (by priority) with keeping focused on (if needed)
    pub(crate) fn stabilize_order(&mut self, focus_id: Option<Uuid>) {
        self.todos.sort_by_key(|t| t.priority);

        if let Some(id) = focus_id {
            if let Some(new_pos) = self.todos.iter().position(|t| t.id == id) {
                self.select_state.select(Some(new_pos));
            }
        } else if self.select_state.selected().is_none() && !self.todos.is_empty() {
            self.select_state.select(Some(0));
        }
    }

    //
    // Main service todo
    //

    // Add new task to the end of list
    pub fn append(&mut self, new_task: Todo) -> ApplicationResult<String> {
        let title: String = new_task.title.clone();

        if title.trim().is_empty() {
            return Err(TodoError::EmptyTitle.into());
        }

        let id: Uuid = new_task.id;
        self.todos.push(new_task);

        self.stabilize_order(Some(id));

        Ok(format!("Task '{}' was added to the list!", title))
    }

    // Update selected task
    pub fn update(&mut self, id: &Uuid, updated_data: Todo) -> ApplicationResult<String> {
        if updated_data.title.trim().is_empty() {
            return Err(TodoError::EmptyTitle.into());
        }

        let index: usize = self
            .select_state
            .selected()
            .ok_or(TodoError::TaskNotSelected)?;
        let task: &mut Todo = self.todo_by_id_mut(id).ok_or(TodoError::TaskNotFound)?;

        task.title = updated_data.title;
        task.description = updated_data.description;
        task.priority = updated_data.priority;

        self.stabilize_order(None);

        Ok(format!(
            "Task {} / {} was updated",
            index + 1,
            self.todos.len()
        ))
    }

    // Remove task selected by id
    pub fn remove(
        &mut self,
        filter: &Filter,
        ui_index: Option<usize>,
    ) -> ApplicationResult<String> {
        let ui_index: usize = ui_index.ok_or(TodoError::TaskNotSelected)?;

        let real_index: usize = self
            .filtered_stream(filter)
            .nth(ui_index)
            .map(|(index, _)| index)
            .ok_or(TodoError::TaskNotSelected)?;

        let removed: Todo = self.todos.remove(real_index);
        let tasks_count: usize = self.todos.len();

        if tasks_count == 0 {
            self.select_state.select(None);
        } else if ui_index >= tasks_count {
            self.select_state.select(Some(tasks_count - 1));
        } else {
            self.select_state.select(Some(ui_index));
        }

        Ok(format!("Task '{}' was removed!", removed.title))
    }

    // Toggle selected task as completed
    pub fn toggle(&mut self, filter: &Filter, ui_index: Option<usize>) {
        let ui_index: usize = match ui_index {
            Some(index) => index,
            None => return,
        };

        let real_index: Option<usize> = self
            .filtered_stream(filter)
            .nth(ui_index)
            .map(|(idx, _)| idx);

        if let Some(idx) = real_index {
            if let Some(todo) = self.todos.get_mut(idx) {
                todo.toggle_completed();
            }
        }
    }

    // Clear todos with selected filter
    pub fn clear(&mut self, filter: &Filter) -> ApplicationResult<String> {
        let old_count: usize = self.todos.len();

        match filter {
            Filter::All => self.todos.clear(),
            Filter::Completed => self.todos.retain(|t| !t.completed),
            Filter::Active => self.todos.retain(|t| t.completed),
            Filter::HighPriority => self.todos.retain(|t| t.priority != Priority::High),
            Filter::Today => {
                let today = Local::now().date_naive();
                self.todos
                    .retain(|t| t.created_at.with_timezone(&Local).date_naive() != today)
            }
        }

        let removed_count: usize = old_count - self.todos.len();
        if removed_count == 0 {
            self.select_state.select(None);
            return Err(TodoError::ListEmpty.into());
        }

        if let Some(selected) = self.select_state.selected() {
            if selected >= removed_count {
                self.select_state.select(Some(removed_count - 1));
            }
        } else {
            self.select_state.select(Some(0));
        }

        Ok(format!("Cleared {} tasks from current view", removed_count))
    }

    // Save and load
    pub fn load() -> ApplicationResult<Self> {
        let path = Self::data_path()?;
        Self::load_from_path(&path)
    }

    pub fn save(&mut self) -> ApplicationResult<String> {
        let path = Self::data_path()?;
        Self::save_to_path(self, &path)
    }

    // Other actions/helper functions
    pub fn todo_by_id_mut(&mut self, id: &Uuid) -> Option<&mut Todo> {
        self.todos.iter_mut().find(|t| t.id == *id)
    }

    pub(crate) fn load_from_path(path: &Path) -> ApplicationResult<Self> {
        if let Some(parent) = path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)
                    .map_err(|_| ApplicationError::Storage(StorageError::IOError))?;
            }
        }

        if !path.exists() {
            return Ok(Self::default());
        }

        let file: File = File::open(path).map_err(|_| StorageError::IOError)?;
        let todos: Vec<Todo> =
            serde_json::from_reader(file).map_err(|_| StorageError::JSONError)?;

        let mut state = Self {
            todos,
            ..Self::default()
        };

        state.select_state.select_last();
        state.saved_todos_hash = state.calculate_todos_hash();
        Ok(state)
    }

    pub(crate) fn save_to_path(&mut self, path: &Path) -> ApplicationResult<String> {
        fs::create_dir_all(path.parent().unwrap()).map_err(|_| StorageError::IOError)?;

        let file: File = File::create(path).map_err(|_| StorageError::IOError)?;
        let writer: BufWriter<File> = BufWriter::new(file);

        serde_json::to_writer_pretty(writer, &self.todos).map_err(|_| StorageError::JSONError)?;
        self.saved_todos_hash = self.calculate_todos_hash();
        Ok(String::from("Tasks were saved!"))
    }

    pub(crate) fn data_path() -> ApplicationResult<PathBuf> {
        dirs::data_dir()
            .ok_or(StorageError::PathNotFound.into())
            .map(|dir| dir.join("todo-tui").join("todos.json"))
    }

    // Get todos hash to compare to current (to track unsaved changes)
    pub(crate) fn calculate_todos_hash(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.todos.hash(&mut hasher);
        hasher.finish()
    }

    // Filter tasks by filter
    pub(crate) fn filtered_stream(&self, filter: &Filter) -> impl Iterator<Item = (usize, &Todo)> {
        let today = Local::now().date_naive();

        self.todos
            .iter()
            .enumerate()
            .filter(move |(_, todo)| match filter {
                Filter::All => true,
                Filter::Active => !todo.completed,
                Filter::Completed => todo.completed,
                Filter::HighPriority => todo.priority == Priority::High,
                Filter::Today => todo.created_at.with_timezone(&Local).date_naive() == today,
            })
    }

    // Show notification after action made
    pub fn notify(&mut self, result: ApplicationResult<String>) {
        match result {
            Ok(msg) => self.notification = Some(Notification::success(msg)),
            Err(e) => self.notification = Some(Notification::error(e.to_string())),
        }
    }
}

// Unit-tests for ApplicationState
#[cfg(test)]
mod tests {
    use super::*;
    use tempdir::TempDir;

    #[test]
    fn should_navigate_through_todos() {
        let mut state = ApplicationState::default();
        state.append(Todo::new("T1", "", None)).unwrap();
        state.append(Todo::new("T2", "", None)).unwrap();

        state.select_state.select(Some(1));
        state.next_task();
        assert_eq!(state.select_state.selected(), Some(0));

        state.prev_task();
        assert_eq!(state.select_state.selected(), Some(1));
    }

    #[test]
    fn should_create_new_state_empty_todos() {
        let temp_dir: TempDir = TempDir::new("todo_test").unwrap();
        let path: PathBuf = temp_dir.path().join("todos.json");

        assert!(!path.exists(), "File should not exist initially");

        let mut state = ApplicationState::load_from_path(&path).unwrap();

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

        let task: Todo = Todo::new("Test Title", "Test Desc", Some(Priority::High));
        let json_data: String = serde_json::to_string(&vec![task.clone()]).unwrap();

        fs::write(&path, json_data).unwrap();
        let state = ApplicationState::load_from_path(&path).unwrap();

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

        let result: ApplicationResult<ApplicationState> = ApplicationState::load_from_path(&path);
        assert!(result.is_ok());

        let state: ApplicationState = result.unwrap();
        assert!(state.todos.is_empty());
        assert_eq!(state.select_state.selected(), None);
    }

    #[test]
    fn should_append_todo() {
        let mut state: ApplicationState = ApplicationState::default();
        let result: ApplicationResult<String> = state.append(Todo::new(
            "Buy stuff",
            "Just buy stuff",
            Some(Priority::High),
        ));

        assert_eq!(
            result,
            Ok(String::from("Task 'Buy stuff' was added to the list!"))
        );
        assert_eq!(state.todos.len(), 1);
        assert_eq!(state.todos[0].title, "Buy stuff");
        assert_eq!(state.todos[0].description, "Just buy stuff");
        assert!(!state.todos[0].completed);
        assert_eq!(state.todos[0].priority, Priority::High);
        assert_eq!(state.select_state.selected(), Some(0));
    }

    #[test]
    fn should_sort_after_append_by_priority() {
        let mut state = ApplicationState::default();

        state.append(Todo::new("Low", "", None)).unwrap();
        state
            .append(Todo::new("High", "", Some(Priority::High)))
            .unwrap();

        assert_eq!(state.todos[0].title, "High");
        assert_eq!(state.todos[1].title, "Low");
    }

    #[test]
    fn should_invoke_empty_title_error_on_append() {
        let mut state: ApplicationState = ApplicationState::default();
        let result: ApplicationResult<String> = state.append(Todo::new("", "", None));

        assert_eq!(result, Err(ApplicationError::Todo(TodoError::EmptyTitle)));
        assert!(state.todos.is_empty());
    }

    #[test]
    fn should_select_last_after_multiple_append() {
        let mut state: ApplicationState = ApplicationState::default();

        state.append(Todo::new("Task 1", "Desc 1", None)).unwrap();
        state.append(Todo::new("Task 2", "Desc 2", None)).unwrap();
        state.append(Todo::new("Task 3", "Desc 3", None)).unwrap();

        assert_eq!(state.todos.len(), 3);
        assert_eq!(state.select_state.selected(), Some(2));
        assert_eq!(state.todos[2].title, "Task 3");
    }

    #[test]
    fn should_update_todo() {
        let mut state = ApplicationState::default();
        state
            .append(Todo::new("Task", "Desc", Some(Priority::Low)))
            .unwrap();
        let id: Uuid = state.todos[0].id;

        let updated: Todo = Todo::new("New Title", "New Desc", Some(Priority::High));
        let result: ApplicationResult<String> = state.update(&id, updated);

        assert_eq!(result, Ok(String::from("Task 1 / 1 was updated")));

        assert_eq!(state.todos[0].title, "New Title");
        assert_eq!(state.todos[0].description, "New Desc");
        assert_eq!(state.todos[0].priority, Priority::High);
    }

    #[test]
    fn should_sort_after_update_by_priority() {
        let mut state: ApplicationState = ApplicationState::default();
        let id: Uuid = Uuid::new_v4();

        state
            .append(Todo::new("First High", "", Some(Priority::High)))
            .unwrap();

        let mut updated = Todo::new("Initially Low", "", Some(Priority::Low));
        updated.id = id;
        state.append(updated.clone()).unwrap();

        updated.priority = Priority::High;
        state.update(&id, updated).unwrap();

        assert_eq!(state.todos[1].title, "Initially Low");
        assert_eq!(state.todos[1].priority, Priority::High);
    }

    #[test]
    fn should_invoke_empty_title_error_on_update() {
        let mut state = ApplicationState::default();
        state
            .append(Todo::new("Task", "Desc", Some(Priority::Low)))
            .unwrap();
        let id: Uuid = state.todos[0].id;

        let updated: Todo = Todo::new(" ", "", Some(Priority::High));
        let result: ApplicationResult<String> = state.update(&id, updated);

        assert_eq!(result, Err(ApplicationError::Todo(TodoError::EmptyTitle)));
        assert_eq!(state.todos[0].title, "Task");
        assert_eq!(state.todos[0].priority, Priority::Low);
    }

    #[test]
    fn should_invoke_not_selected_error_on_update() {
        let mut state: ApplicationState = ApplicationState::default();
        state.append(Todo::new("Task", "Desc", None)).unwrap();

        let id: Uuid = state.todos[0].id;
        let updated: Todo = Todo::new("Updated", "Updated", Some(Priority::High));

        state.select_state.select(None);
        let result: ApplicationResult<String> = state.update(&id, updated);
        assert_eq!(
            result,
            Err(ApplicationError::Todo(TodoError::TaskNotSelected))
        );
    }

    #[test]
    fn should_invoke_not_found_error_on_update() {
        let mut state: ApplicationState = ApplicationState::default();
        state.append(Todo::new("Task", "Desc", None)).unwrap();

        let updated: Todo = Todo::new("Updated", "Updated", Some(Priority::High));
        let id: Uuid = Uuid::new_v4();

        let result: ApplicationResult<String> = state.update(&id, updated);
        assert_eq!(result, Err(ApplicationError::Todo(TodoError::TaskNotFound)));
    }

    #[test]
    fn should_stabilize_order_properly() {
        let mut state: ApplicationState = ApplicationState::default();

        state
            .append(Todo::new("Task A", "", Some(Priority::High)))
            .unwrap();
        state
            .append(Todo::new("Task B", "", Some(Priority::High)))
            .unwrap();

        state.todos.swap(0, 1);
        assert_eq!(state.todos[0].title, "Task B");

        state.stabilize_order(None);

        assert_eq!(
            state.todos[0].title, "Task B",
            "Stability failed: swap was reverted"
        );
    }

    #[test]
    fn should_remove_todo_with_filter() {
        let mut state = ApplicationState::default();
        state.append(Todo::new("Task 1", "", None)).unwrap();
        state.append(Todo::new("Task 2", "", None)).unwrap();
        state.todos[0].completed = true;

        state.remove(&Filter::Active, Some(0)).unwrap();

        assert_eq!(state.todos.len(), 1);
        assert_eq!(state.todos[0].title, "Task 1",);
    }

    #[test]
    fn should_invoke_not_selected_error_on_remove() {
        let mut state: ApplicationState = ApplicationState::default();
        state.append(Todo::new("Task", "", None)).unwrap();

        let result = state.remove(&Filter::All, None);

        assert_eq!(
            result,
            Err(ApplicationError::Todo(TodoError::TaskNotSelected))
        );
    }

    #[test]
    fn should_toggle_with_filter_logic() {
        let mut state = ApplicationState::default();
        state.append(Todo::new("Toggle Me", "", None)).unwrap();

        assert!(!state.todos[0].completed);

        state.toggle(&Filter::All, Some(0));
        assert!(state.todos[0].completed);

        state.toggle(&Filter::All, Some(0));
        assert!(!state.todos[0].completed);
    }

    #[test]
    fn should_clear_todos_with_specific_filter() {
        let mut state = ApplicationState::default();
        state.append(Todo::new("Active", "", None)).unwrap();
        state.append(Todo::new("Done", "", None)).unwrap();
        state.todos[1].completed = true;

        state.clear(&Filter::Completed).unwrap();
        assert_eq!(state.todos.len(), 1);
        assert_eq!(state.todos[0].title, "Active");

        let result: ApplicationResult<String> = state.clear(&Filter::Completed);
        assert_eq!(result, Err(TodoError::ListEmpty.into()))
    }

    #[test]
    fn should_clear_all() {
        let mut state = ApplicationState::default();
        state.append(Todo::new("T1", "", None)).unwrap();
        state.append(Todo::new("T2", "", None)).unwrap();

        let result: ApplicationResult<String> = state.clear(&Filter::All);
        assert_eq!(
            result,
            Ok(String::from("Cleared 2 tasks from current view"))
        );
        assert!(state.todos.is_empty());
    }

    #[test]
    fn should_save_and_load_todos_successfully() {
        let temp_dir: TempDir = TempDir::new("todo_test").unwrap();
        let path: PathBuf = temp_dir.path().join("todos.json");

        let mut state = ApplicationState::default();
        state.append(Todo::new("Task 1", "", None)).unwrap();
        state.append(Todo::new("Task 2", "", None)).unwrap();

        let result: ApplicationResult<String> = state.save_to_path(&path);
        assert_eq!(result, Ok(String::from("Tasks were saved!")));

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

        let mut state: ApplicationState = ApplicationState::default();
        let result: ApplicationResult<String> = state.save_to_path(&path);

        assert!(result.is_err());
        assert!(matches!(
            result,
            Err(ApplicationError::Storage(StorageError::IOError))
        ));
    }

    #[test]
    fn should_determine_unsaved_changes() {
        let mut state = ApplicationState::default();
        state.saved_todos_hash = state.calculate_todos_hash();
        assert!(!state.any_unsaved_changes());

        state.append(Todo::new("Task", "", None)).unwrap();
        assert!(
            state.any_unsaved_changes(),
            "Hash should be changed after append"
        );

        state.saved_todos_hash = state.calculate_todos_hash();
        assert!(!state.any_unsaved_changes());

        state.todos[0].title = "Changed".to_string();
        assert!(
            state.any_unsaved_changes(),
            "Hash should be changed after field edit"
        );
    }

    #[test]
    fn should_find_todo_by_id() {
        let mut state = ApplicationState::default();
        let task: Todo = Todo::new("Find me", "", None);
        let target_id: Uuid = task.id;
        state.append(task).unwrap();

        let found = state.todo_by_id_mut(&target_id);
        assert!(found.is_some());
        assert_eq!(found.unwrap().title, "Find me");

        let missing = state.todo_by_id_mut(&uuid::Uuid::new_v4());
        assert!(missing.is_none());
    }

    #[test]
    fn should_test_filtered_stream_mapping() {
        let mut state = ApplicationState::default();
        state.append(Todo::new("Active", "", None)).unwrap();
        state
            .append(Todo::new("High", "", Some(Priority::High)))
            .unwrap();
        state.append(Todo::new("Done", "", None)).unwrap();
        state.todos[2].completed = true;

        let high_priority: Vec<_> = state.filtered_stream(&Filter::HighPriority).collect();
        assert_eq!(high_priority.len(), 1);
        assert_eq!(high_priority[0].0, 0);

        let completed: Vec<_> = state.filtered_stream(&Filter::Completed).collect();
        assert_eq!(completed.len(), 1);
        assert_eq!(completed[0].0, 2);
        assert_eq!(completed[0].1.title, "Done");
    }

    #[test]
    fn shoukd_handle_notification_creation() {
        let mut state = ApplicationState::default();

        state.notify(Ok("Success message".to_string()));
        assert!(state.notification.is_some());

        let error_res: ApplicationResult<String> = Err(TodoError::EmptyTitle.into());
        state.notify(error_res);
        assert!(state.notification.is_some());
    }

    #[test]
    fn should_move_tasks_successully_with_same_priority() {
        let mut state: ApplicationState = ApplicationState::default();

        state
            .append(Todo::new("Task 1", "", Some(Priority::High)))
            .unwrap();
        state
            .append(Todo::new("Task 2", "", Some(Priority::High)))
            .unwrap();

        state.select_state.select(Some(0));
        state.move_task_down();

        assert_eq!(state.todos[0].title, "Task 2");
        assert_eq!(state.todos[1].title, "Task 1");
        assert_eq!(state.select_state.selected(), Some(1));
    }

    #[test]
    fn should_not_move_tasks_with_different_priorities() {
        let mut state = ApplicationState::default();

        state
            .append(Todo::new("High Task", "", Some(Priority::High)))
            .unwrap();
        state
            .append(Todo::new("Medium Task", "", Some(Priority::Medium)))
            .unwrap();

        state.select_state.select(Some(1));
        state.move_task_up();

        assert_eq!(state.todos[0].title, "High Task");
        assert_eq!(state.todos[1].title, "Medium Task");
        assert_eq!(state.select_state.selected(), Some(1));
    }

    #[test]
    fn test_move_task_boundaries() {
        let mut state = ApplicationState::default();
        state.append(Todo::new("Task 1", "", None)).unwrap();

        state.select_state.select(Some(0));

        state.move_task_up();
        state.move_task_down();

        assert_eq!(state.select_state.selected(), Some(0));
    }
}
