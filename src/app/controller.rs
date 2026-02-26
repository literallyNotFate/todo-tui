use crate::{
    app::TodoService,
    config::Config,
    core::TodoError,
    models::{Priority, Todo},
    state::{ApplicationState, UIState},
    traits::InteractableEnum,
};
use uuid::Uuid;

/// Application controller (binds application state and UI)
pub struct ApplicationController<'a> {
    pub state: &'a mut ApplicationState,
    pub ui: &'a mut UIState,
    pub config: &'a mut Config,
}

impl<'a> ApplicationController<'a> {
    pub fn new(
        state: &'a mut ApplicationState,
        ui: &'a mut UIState,
        config: &'a mut Config,
    ) -> Self {
        Self { state, ui, config }
    }

    /// Handle appending a task
    pub fn dispatch_append(
        &mut self,
        title: impl Into<String>,
        desc: impl Into<String>,
        priority: Option<Priority>,
    ) {
        let title_string: String = title.into();
        log::debug!("Dispatching append for task: '{}'", title_string);

        let task: Todo = Todo::new(title_string, desc, priority);
        let id: Uuid = task.id;

        match TodoService::append_task(&mut self.state.todos, task, &self.state.sort) {
            Ok(added) => {
                self.stabilize_ui_focus(Some(id));
                self.state.mark_as_dirty();

                self.ui.push_notification(
                    self.state,
                    Ok(format!("Task '{}' was added to the list!", added)),
                );
            }
            Err(e) => self.ui.push_notification(self.state, Err(e)),
        }
    }

    /// Handle updating an existing todo
    pub fn dispatch_update(&mut self, id: Uuid, task: Todo) {
        match TodoService::update_task(&mut self.state.todos, &id, task, &self.state.sort) {
            Ok(index) => {
                log::debug!("Dispatching update for task (ID: {})", id);
                self.stabilize_ui_focus(Some(id));
                self.state.mark_as_dirty();

                let msg = format!(
                    "Task {} / {} was updated",
                    index + 1,
                    self.state.todos.len()
                );

                self.ui.push_notification(self.state, Ok(msg));
            }
            Err(e) => self.ui.push_notification(self.state, Err(e)),
        }
    }

    /// Handle removing task
    pub fn dispatch_remove(&mut self) {
        if let Some(id) = self.ui.selected_id(self.state) {
            match TodoService::remove_task(&mut self.state.todos, &id) {
                Ok(removed) => {
                    log::debug!("Dispatching remove for task '{}'", removed);
                    self.stabilize_ui_focus(None);
                    self.state.mark_as_dirty();

                    self.ui.push_notification(
                        self.state,
                        Ok(format!("Task '{}' was removed!", removed)),
                    );
                }
                Err(e) => self.ui.push_notification(self.state, Err(e)),
            }
        } else {
            self.ui
                .push_notification(self.state, Err(TodoError::TaskNotFound.into()));
        }
    }

    /// Handle task completion toggling
    pub fn dispatch_toggle(&mut self) {
        if let Some(id) = self.ui.selected_id(self.state) {
            if TodoService::toggle_task(&mut self.state.todos, &id).is_ok() {
                self.stabilize_ui_focus(Some(id));
                self.state.mark_as_dirty();
            }
        }
    }

    /// Handle moving a task
    pub fn dispatch_move_tasks(&mut self, delta: i32) {
        if let Some((index_a, index_b)) =
            self.state
                .swap_indices(&self.ui.current_filter, &self.ui.search_query(), delta)
        {
            match TodoService::move_tasks(&mut self.state.todos, index_a, index_b) {
                Ok(_) => {
                    let current_index: usize = self.state.select_state.selected().unwrap_or(0);
                    let new_index: usize = if delta > 0 {
                        current_index + 1
                    } else {
                        current_index.saturating_sub(1)
                    };

                    self.state.select_state.select(Some(new_index));
                    self.state.mark_as_dirty();
                }
                Err(e) => self.ui.push_notification(self.state, Err(e)),
            }
        }
    }

    /// Handle clearing tasks by filter
    pub fn dispatch_clear(&mut self) {
        let removed: usize = TodoService::clear(&mut self.state.todos, &self.ui.current_filter);

        if removed > 0 {
            log::info!("Clear successful: {} tasks removed", removed);
            self.state.mark_as_dirty();
            self.stabilize_ui_focus(None);

            let msg: String = format!(
                "Cleared {} tasks from '{}'",
                removed,
                self.ui.current_filter.to_string()
            );
            self.ui.push_notification(self.state, Ok(msg));
        } else {
            log::debug!("Clear skipped: no tasks matched current filter");
            self.ui
                .push_notification(self.state, Err(TodoError::ListEmpty.into()));
        }
    }

    /// Handle saving all (todos + config) on Ctrl+S
    pub fn dispatch_save(&mut self) -> bool {
        self.config.update_from_ui(self.ui);

        if let Err(e) = self.config.save(None) {
            self.ui.show_result_popup(Err(e));
            return false;
        }

        match self.state.save(None, &self.config.storage) {
            Ok(msg) => {
                self.ui.show_result_popup(Ok(msg));
                true
            }
            Err(e) => {
                self.ui.show_result_popup(Err(e));
                false
            }
        }
    }

    /// Handle sorting
    pub fn dispatch_sorting(&mut self) {
        let selected_id = self
            .state
            .selected(
                &self.state.todos,
                &self.ui.current_filter,
                &self.ui.search_query(),
            )
            .map(|t| t.id);

        TodoService::sorting(&mut self.state.todos, &self.state.sort);
        self.state.mark_as_dirty();

        if let Some(id) = selected_id {
            let filtered = self
                .ui
                .current_filter
                .apply(&self.state.todos, &self.ui.search_query());
            let new_pos = filtered.iter().position(|t| t.id == id);

            self.state.select_state.select(new_pos);
        }
    }

    /// Handle selection change
    pub fn dispatch_move_selection(&mut self, delta: i32) {
        let len = self.state.filter(&self.ui.current_filter).count();
        let wrap: bool = self.config.behavior.wrap_scrolling;
        self.state.move_selection(delta, len, wrap);
        self.ui.desc_scroll.reset();
    }

    /// Helper function to synchronize cursor and data
    pub fn stabilize_ui_focus(&mut self, focus_id: Option<Uuid>) {
        let filtered_todos = self
            .ui
            .current_filter
            .apply(&self.state.todos, &self.ui.search_query());
        let len: usize = filtered_todos.len();
        log::trace!(
            "Stabilizing UI focus. Focus ID: {:?}, Filtered count: {}",
            focus_id,
            len
        );

        if let Some(id) = focus_id {
            if let Some(pos) = filtered_todos.iter().position(|t| t.id == id) {
                log::trace!("Focus matched ID {} at position {}", id, pos);
                self.state.select_state.select(Some(pos));
                return;
            }
        }

        if len == 0 {
            self.state.select_state.select(None);
        } else {
            let current_selected = self.state.select_state.selected();
            if current_selected.is_none_or(|s| s >= len) {
                self.state.select_state.select(Some(len.saturating_sub(1)));
            }
        }
    }
}

/// Unit-tests for application controller
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        config::StorageConfig,
        models::{Sort, SortBy, SortOrder},
        ui::Notification,
    };
    use std::path::{Path, PathBuf};
    use tempdir::TempDir;

    fn setup() -> (ApplicationState, UIState, Config) {
        (
            ApplicationState::default(),
            UIState::default(),
            Config::default(),
        )
    }

    fn mock_dispatch_save(
        state: &mut ApplicationState,
        ui: &mut UIState,
        path: &Path,
        config: &StorageConfig,
    ) {
        match state.save(Some(path), config) {
            Ok(string) => ui.show_result_popup(Ok(string)),
            Err(e) => ui.show_result_popup(Err(e)),
        }
    }

    #[test]
    fn should_append_task_and_set_notification() {
        let (mut state, mut ui, mut config) = setup();
        let mut ctrl = ApplicationController::new(&mut state, &mut ui, &mut config);

        ctrl.dispatch_append("Test", "Desc", Some(Priority::High));

        assert_eq!(state.todos.len(), 1);
        assert_eq!(state.todos[0].title, "Test");
        assert_eq!(state.select_state.selected(), Some(0));
        assert!(state.notification.is_some());
        assert!(state.notification.unwrap().message.contains("was added"));
    }

    #[test]
    fn should_handle_empty_title_error_on_append() {
        let (mut state, mut ui, mut config) = setup();
        let mut ctrl = ApplicationController::new(&mut state, &mut ui, &mut config);

        ctrl.dispatch_append("  ", "Description", None);
        assert_eq!(state.todos.len(), 0);
        assert!(state.notification.is_some());

        let note: &Notification = state.notification.as_ref().unwrap();
        assert_eq!(note.message, "Task title cannot be empty!");
    }

    #[test]
    fn should_handle_update_and_maintain_focus() {
        let (mut state, mut ui, mut config) = setup();

        let task_high = Todo::new("High Task", "", Some(Priority::High));
        let task_low = Todo::new("Low Task", "", Some(Priority::Low));
        let low_id = task_low.id;

        state.todos = vec![task_high, task_low];
        TodoService::sorting(&mut state.todos, &state.sort);
        state.select_state.select(Some(1));

        let mut updated_task = state.todos[1].clone();
        updated_task.title = "Updated to High".into();
        updated_task.priority = Priority::High;

        let mut ctrl = ApplicationController::new(&mut state, &mut ui, &mut config);
        ctrl.dispatch_update(low_id, updated_task);

        let new_pos = state.todos.iter().position(|t| t.id == low_id).unwrap();

        assert_eq!(
            state.select_state.selected(),
            Some(new_pos),
            "Focus must follow the task"
        );
        assert_eq!(state.todos[new_pos].title, "Updated to High");

        let note: &Notification = state.notification.as_ref().unwrap();
        assert!(note.message.contains("updated"),);
    }

    #[test]
    fn should_handle_empty_title_error_on_update() {
        let (mut state, mut ui, mut config) = setup();
        let task: Todo = Todo::new("Task", "", Some(Priority::Low));
        let id: Uuid = task.id;

        state.todos.push(task);
        let mut updated_task: Todo = state.todos[0].clone();
        let mut ctrl = ApplicationController::new(&mut state, &mut ui, &mut config);

        updated_task.title = "".into();
        ctrl.dispatch_update(id, updated_task);

        let note: &Notification = state.notification.as_ref().unwrap();
        assert!(state.notification.is_some());
        assert_eq!(note.message, "Task title cannot be empty!")
    }

    #[test]
    fn should_handle_update_non_existent_task() {
        let (mut state, mut ui, mut config) = setup();
        let mut ctrl = ApplicationController::new(&mut state, &mut ui, &mut config);
        let fake_id: Uuid = Uuid::new_v4();
        let task: Todo = Todo::new("Title", "", None);

        ctrl.dispatch_update(fake_id, task);

        let note: &Notification = state.notification.as_ref().unwrap();
        assert!(state.notification.is_some());
        assert_eq!(note.message, "Task was not found by the provided id!");
    }

    #[test]
    fn should_remove_task_and_adjust_selection() {
        let (mut state, mut ui, mut config) = setup();
        state.todos.push(Todo::new("T1", "", None));
        state.todos.push(Todo::new("T2", "", None));
        state.select_state.select(Some(1));

        let mut ctrl = ApplicationController::new(&mut state, &mut ui, &mut config);
        ctrl.dispatch_remove();

        assert_eq!(state.todos.len(), 1);
        assert_eq!(state.select_state.selected(), Some(0));
        assert!(state.notification.unwrap().message.contains("removed"));
    }

    #[test]
    fn should_handle_remove_non_existent_task() {
        let (mut state, mut ui, mut config) = setup();
        state.todos.push(Todo::new("Task", "", None));
        state.select_state.select(Some(999));

        let mut ctrl = ApplicationController::new(&mut state, &mut ui, &mut config);
        ctrl.dispatch_remove();

        let note: &Notification = state.notification.as_ref().unwrap();
        assert!(state.notification.is_some());
        assert_eq!(note.message, "Task was not found by the provided id!");
    }

    #[test]
    fn should_sort_with_focus_stabilized() {
        let (mut state, mut ui, mut config) = setup();
        let id_a = Uuid::new_v4();
        let id_b = Uuid::new_v4();

        state.todos = vec![
            Todo {
                id: id_a,
                title: "B".into(),
                ..Default::default()
            },
            Todo {
                id: id_b,
                title: "A".into(),
                ..Default::default()
            },
        ];

        state.select_state.select(Some(0));
        state.sort = Sort::new(SortBy::Title, SortOrder::Asc);
        let mut ctrl = ApplicationController::new(&mut state, &mut ui, &mut config);

        ctrl.dispatch_sorting();

        assert_eq!(ctrl.state.select_state.selected(), Some(1));
        assert_eq!(ctrl.state.todos[1].id, id_a);
    }

    #[test]
    fn should_stabilize_focus_out_of_bounds() {
        let (mut state, mut ui, mut config) = setup();
        state.todos = vec![Todo::new("One", "", Some(Priority::Low))];
        state.select_state.select(Some(10));

        let mut ctrl = ApplicationController::new(&mut state, &mut ui, &mut config);
        ctrl.stabilize_ui_focus(None);

        assert_eq!(ctrl.state.select_state.selected(), Some(0));
    }

    #[test]
    fn should_clear_with_empty_list_error() {
        let (mut state, mut ui, mut config) = setup();
        let mut ctrl = ApplicationController::new(&mut state, &mut ui, &mut config);

        ctrl.dispatch_clear();

        let note: &Notification = state.notification.as_ref().unwrap();
        assert!(state.notification.is_some());
        assert_eq!(
            note.message,
            "Cannot clear the tasks! The list is already empty!"
        );
    }

    #[test]
    fn should_trigger_popup_on_save() {
        let temp_dir: TempDir = TempDir::new("todo_test").unwrap();
        let path: PathBuf = temp_dir.path().join("todos.json");

        let (mut state, mut ui, config) = setup();

        mock_dispatch_save(&mut state, &mut ui, &path, &config.storage);
        assert!(ui.modal.is_some());
    }
}
