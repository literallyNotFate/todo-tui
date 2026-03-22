use crate::{
    app::TodoService,
    config::{Config, KeyMaps},
    core::{Storage, TodoError},
    models::{Priority, Todo, todo::TodoEditor},
    state::{ApplicationState, Session, UIState},
};
use uuid::Uuid;

/// Application controller (binds application state and UI)
pub struct ApplicationController<'a> {
    pub state: &'a mut ApplicationState,
    pub ui: &'a mut UIState,
    pub config: &'a mut Config,
    pub keymaps: &'a KeyMaps,
}

impl<'a> ApplicationController<'a> {
    pub fn new(
        state: &'a mut ApplicationState,
        ui: &'a mut UIState,
        config: &'a mut Config,
        keymaps: &'a KeyMaps,
    ) -> Self {
        Self {
            state,
            ui,
            config,
            keymaps,
        }
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
                self.stabilize(Some(id));
                self.state.mark_as_dirty();

                self.ui.push_notification(
                    self.state,
                    Ok(format!(
                        "Task '{}' was added to the list!",
                        added.task.title
                    )),
                );
            }
            Err(e) => self.ui.push_notification(self.state, Err(e)),
        }
    }

    /// Handle updating an existing todo
    pub fn dispatch_update(&mut self, id: Uuid, editor: TodoEditor) {
        match TodoService::update_task(&mut self.state.todos, &id, editor, &self.state.sort) {
            Ok(result) => {
                log::debug!("Dispatching update for task (ID: {})", id);
                self.stabilize(Some(id));
                self.state.mark_as_dirty();

                let msg = self.format_update_service_message(&result.old, &result.new);
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
                    log::debug!("Dispatching remove for task '{}'", removed.task.title);
                    self.stabilize(None);
                    self.state.mark_as_dirty();

                    self.ui.push_notification(
                        self.state,
                        Ok(format!("Task '{}' was removed!", removed.task.title)),
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
                self.stabilize(Some(id));
                self.state.mark_as_dirty();
            }
        }
    }

    /// Handle moving a task
    pub fn dispatch_move_tasks(&mut self, delta: i32) {
        if let Some((index_a, index_b)) =
            self.state
                .swap_indices(&self.ui.filter.value, &self.ui.search_query(), delta)
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
        let removed: usize = TodoService::clear(&mut self.state.todos, &self.ui.filter);

        if removed > 0 {
            log::info!("Clear successful: {} tasks removed", removed);
            self.state.mark_as_dirty();
            self.stabilize(None);

            let msg: String = format!("Cleared {} tasks from '{}'", removed, self.ui.filter);
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

        let current_id =
            self.state
                .selected_id(&self.state.todos, &self.ui.filter, &self.ui.search_query());
        let session = Session::from_state(self.ui, current_id);

        match Storage::save(&self.state.todos, session, None, &self.config.storage) {
            Ok(msg) => {
                self.state.mark_saved();
                let _ = self.config.save(None);
                self.ui.show_result_popup(Ok(msg));

                true
            }
            Err(e) => {
                log::error!("Save failed: {}", e);
                self.ui.show_result_popup(Err(e));
                true
            }
        }
    }

    /// Handle sorting
    pub fn dispatch_sorting(&mut self) {
        let selected_id = self
            .state
            .selected(&self.state.todos, &self.ui.filter, &self.ui.search_query())
            .map(|t| t.id);

        TodoService::sorting(&mut self.state.todos, &self.state.sort);
        self.state.mark_as_dirty();

        if let Some(id) = selected_id {
            let filtered = self
                .ui
                .filter
                .value
                .apply(&self.state.todos, &self.ui.search_query());
            let new_pos = filtered.iter().position(|t| t.id == id);

            self.state.select_state.select(new_pos);
        }
    }

    /// Handle selection change
    pub fn dispatch_move_selection(&mut self, delta: i32) {
        let len = self.state.filter(&self.ui.filter).count();
        let wrap: bool = self.config.behavior.wrap_scrolling;
        self.state.move_selection(delta, len, wrap);
        self.ui.desc_scroll.reset();
    }

    /// Function to synchronize cursor and data
    pub fn stabilize(&mut self, focus_id: Option<Uuid>) {
        let visible_ids: Vec<Uuid> = self
            .ui
            .filter
            .apply(&self.state.todos, &self.ui.search_query())
            .iter()
            .map(|t| t.id)
            .collect();

        log::trace!(
            "Stabilizing UI focus. Focus ID: {:?}, filtered count: {}",
            focus_id,
            visible_ids.len()
        );
        self.state.sync_with_ids(&visible_ids, focus_id);
    }

    /// Helper function to generate update task text based on diff between states
    fn format_update_service_message(&self, old: &Todo, new: &Todo) -> String {
        if old.title != new.title {
            format!("Title: '{}' → '{}'", old.title, new.title)
        } else if old.priority != new.priority {
            format!(
                "Priority: {:?} → {:?} for '{}'",
                old.priority, new.priority, new.title
            )
        } else if old.description != new.description {
            format!("Description updated for '{}'", new.title)
        } else {
            format!("Saved '{}' without changes!", new.title)
        }
    }
}

/// Unit-tests for application controller
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        config::StorageConfig,
        core::{Selectable, Sort, SortBy, SortOrder},
        ui::Notification,
    };
    use std::path::{Path, PathBuf};
    use tempdir::TempDir;

    fn setup() -> (ApplicationState, UIState, Config, KeyMaps) {
        (
            ApplicationState::default(),
            UIState::default(),
            Config::default(),
            KeyMaps::default(),
        )
    }

    fn mock_dispatch_save(
        state: &mut ApplicationState,
        ui: &mut UIState,
        path: &Path,
        config: &StorageConfig,
    ) {
        let selected_id = state.selected_id(&state.todos, &ui.filter, &ui.search_query());
        let session = Session::from_state(ui, selected_id);
        let result = Storage::save(&state.todos, session, Some(path), config);
        ui.show_result_popup(result);
    }

    #[test]
    fn should_append_task_and_set_notification() {
        let (mut state, mut ui, mut config, keymaps) = setup();
        let mut ctrl = ApplicationController::new(&mut state, &mut ui, &mut config, &keymaps);

        ctrl.dispatch_append("Test", "Desc", Some(Priority::High));

        assert_eq!(state.todos.len(), 1);
        assert_eq!(state.todos[0].title, "Test");
        assert_eq!(state.select_state.selected(), Some(0));
        assert!(state.notification.is_some());
        assert!(state.notification.unwrap().message.contains("was added"));
    }

    #[test]
    fn should_handle_empty_title_error_on_append() {
        let (mut state, mut ui, mut config, keymaps) = setup();
        let mut ctrl = ApplicationController::new(&mut state, &mut ui, &mut config, &keymaps);

        ctrl.dispatch_append("  ", "Description", None);
        assert_eq!(state.todos.len(), 0);
        assert!(state.notification.is_some());

        let note: &Notification = state.notification.as_ref().unwrap();
        assert_eq!(note.message, "Task title cannot be empty!");
    }

    #[test]
    fn should_handle_update_and_maintain_focus() {
        let (mut state, mut ui, mut config, keymaps) = setup();

        let task_high = Todo::new("High Task", "", Some(Priority::High));
        let task_low = Todo::new("Low Task", "", Some(Priority::Low));
        let low_id = task_low.id;

        state.todos = vec![task_high, task_low];
        TodoService::sorting(&mut state.todos, &state.sort);
        state.select_state.select(Some(1));

        let editor = TodoEditor {
            title: "Now High".into(),
            description: "".into(),
            priority: Selectable::new(Priority::High),
        };

        let mut ctrl = ApplicationController::new(&mut state, &mut ui, &mut config, &keymaps);
        ctrl.dispatch_update(low_id, editor);

        let new_pos = state
            .todos
            .iter()
            .position(|t| t.id == low_id)
            .expect("Task must exist in list");

        assert_eq!(
            state.select_state.selected(),
            Some(new_pos),
            "Selection must follow the task to its new sorted position"
        );

        assert_eq!(state.todos[new_pos].title, "Now High");

        let note = state
            .notification
            .as_ref()
            .expect("Notification should be present");

        assert!(
            note.message.contains("Title: 'Low Task' → 'Now High'"),
            "Notification should show the title change. Current: {}",
            note.message
        );
    }
    #[test]
    fn should_handle_empty_title_error_on_update() {
        let (mut state, mut ui, mut config, keymaps) = setup();
        let task: Todo = Todo::new("Task", "", Some(Priority::Low));
        let id: Uuid = task.id;

        state.todos.push(task);
        let mut ctrl = ApplicationController::new(&mut state, &mut ui, &mut config, &keymaps);

        let editor: TodoEditor = TodoEditor {
            title: "".into(),
            description: "".into(),
            priority: Selectable::default(),
        };

        ctrl.dispatch_update(id, editor);

        let note: &Notification = state.notification.as_ref().unwrap();
        assert!(state.notification.is_some());
        assert_eq!(note.message, "Task title cannot be empty!")
    }

    #[test]
    fn should_handle_update_non_existent_task() {
        let (mut state, mut ui, mut config, keymaps) = setup();
        let mut ctrl = ApplicationController::new(&mut state, &mut ui, &mut config, &keymaps);
        let fake_id: Uuid = Uuid::new_v4();
        let editor: TodoEditor = TodoEditor {
            title: "Title".into(),
            description: "".into(),
            priority: Selectable::default(),
        };

        ctrl.dispatch_update(fake_id, editor);

        let note: &Notification = state.notification.as_ref().unwrap();
        assert!(state.notification.is_some());
        assert_eq!(note.message, "Task was not found by the provided id!");
    }

    #[test]
    fn should_remove_task_and_adjust_selection() {
        let (mut state, mut ui, mut config, keymaps) = setup();
        state.todos.push(Todo::new("T1", "", None));
        state.todos.push(Todo::new("T2", "", None));
        state.select_state.select(Some(1));

        let mut ctrl = ApplicationController::new(&mut state, &mut ui, &mut config, &keymaps);
        ctrl.dispatch_remove();

        assert_eq!(state.todos.len(), 1);
        assert_eq!(state.select_state.selected(), Some(0));
        assert!(state.notification.unwrap().message.contains("removed"));
    }

    #[test]
    fn should_handle_remove_non_existent_task() {
        let (mut state, mut ui, mut config, keymaps) = setup();
        state.todos.push(Todo::new("Task", "", None));
        state.select_state.select(Some(999));

        let mut ctrl = ApplicationController::new(&mut state, &mut ui, &mut config, &keymaps);
        ctrl.dispatch_remove();

        let note: &Notification = state.notification.as_ref().unwrap();
        assert!(state.notification.is_some());
        assert_eq!(note.message, "Task was not found by the provided id!");
    }

    #[test]
    fn should_sort_with_focus_stabilized() {
        let (mut state, mut ui, mut config, keymaps) = setup();
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
        let mut ctrl = ApplicationController::new(&mut state, &mut ui, &mut config, &keymaps);

        ctrl.dispatch_sorting();

        assert_eq!(ctrl.state.select_state.selected(), Some(1));
        assert_eq!(ctrl.state.todos[1].id, id_a);
    }

    #[test]
    fn should_stabilize_focus_out_of_bounds() {
        let (mut state, mut ui, mut config, keymaps) = setup();
        state.todos = vec![Todo::new("One", "", Some(Priority::Low))];
        state.select_state.select(Some(10));

        let mut ctrl = ApplicationController::new(&mut state, &mut ui, &mut config, &keymaps);
        ctrl.stabilize(None);

        assert_eq!(ctrl.state.select_state.selected(), Some(0));
    }

    #[test]
    fn should_clear_with_empty_list_error() {
        let (mut state, mut ui, mut config, keymaps) = setup();
        let mut ctrl = ApplicationController::new(&mut state, &mut ui, &mut config, &keymaps);

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

        let (mut state, mut ui, config, _) = setup();

        mock_dispatch_save(&mut state, &mut ui, &path, &config.storage);
        assert!(ui.modal.is_some());
    }
}
