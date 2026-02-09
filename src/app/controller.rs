use crate::{
    app::TodoService,
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
}

impl<'a> ApplicationController<'a> {
    pub fn new(state: &'a mut ApplicationState, ui: &'a mut UIState) -> Self {
        Self { state, ui }
    }

    /// Handle appending a task
    pub fn dispatch_append(
        &mut self,
        title: impl Into<String>,
        desc: impl Into<String>,
        priority: Option<Priority>,
    ) {
        let task: Todo = Todo::new(title, desc, priority);
        let id: Uuid = task.id;

        match TodoService::append_task(&mut self.state.todos, task) {
            Ok(added) => {
                self.stabilize_ui_focus(Some(id));
                self.state.hash_state();

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
        match TodoService::update_task(&mut self.state.todos, &id, task) {
            Ok(index) => {
                self.state.select_state.select(Some(index));
                self.state.hash_state();

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
                    self.stabilize_ui_focus(None);
                    self.state.hash_state();

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
                self.state.hash_state();
                self.stabilize_ui_focus(Some(id));
            }
        }
    }

    /// Handle moving a task
    pub fn dispatch_move_task(&mut self, delta: i32) {
        if let Some(current) = self.state.select_state.selected() {
            match TodoService::move_task(&mut self.state.todos, current, delta) {
                Ok(index) => {
                    self.state.select_state.select(Some(index));
                    self.state.hash_state();
                }
                Err(e) => self.ui.push_notification(self.state, Err(e)),
            }
        }
    }

    /// Handle clearing tasks by filter
    pub fn dispatch_clear(&mut self) {
        let removed = TodoService::clear(&mut self.state.todos, &self.ui.current_filter);

        if removed > 0 {
            self.state.hash_state();
            self.stabilize_ui_focus(None);

            let msg: String = format!(
                "Cleared {} tasks from filter {}",
                removed,
                self.ui.current_filter.to_string()
            );
            self.ui.push_notification(self.state, Ok(msg));
        } else {
            self.ui
                .push_notification(self.state, Err(TodoError::ListEmpty.into()));
        }
    }

    /// Handle saving on Ctrl+S
    pub fn dispatch_save(&mut self) {
        match self.state.save(None) {
            Ok(_) => self.ui.show_result_popup(Ok("Tasks were saved!".into())),
            Err(e) => self.ui.show_result_popup(Err(e)),
        }
    }

    /// Helper function to synchronize cursor and data
    pub fn stabilize_ui_focus(&mut self, focus_id: Option<Uuid>) {
        if let Some(id) = focus_id {
            if let Some(pos) = self.state.todos.iter().position(|t| t.id == id) {
                self.state.select_state.select(Some(pos));
                return;
            }
        }

        let len: usize = self.state.todos.len();
        if len == 0 {
            self.state.select_state.select(None);
        } else if self
            .state
            .select_state
            .selected()
            .map_or(true, |s| s >= len)
        {
            self.state.select_state.select(Some(len.saturating_sub(1)));
        }
    }
}

/// Unit-tests for application controller
#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::Notification;

    fn setup() -> (ApplicationState, UIState) {
        (ApplicationState::default(), UIState::default())
    }

    #[test]
    fn should_append_task_and_set_notification() {
        let (mut state, mut ui) = setup();
        let mut ctrl = ApplicationController::new(&mut state, &mut ui);

        ctrl.dispatch_append("Test", "Desc", Some(Priority::High));

        assert_eq!(state.todos.len(), 1);
        assert_eq!(state.todos[0].title, "Test");
        assert_eq!(state.select_state.selected(), Some(0));
        assert!(state.notification.is_some());
        assert!(state.notification.unwrap().message.contains("was added"));
    }

    #[test]
    fn should_handle_empty_title_error_on_append() {
        let (mut state, mut ui) = setup();
        let mut ctrl = ApplicationController::new(&mut state, &mut ui);

        ctrl.dispatch_append("  ", "Description", None);
        assert_eq!(state.todos.len(), 0);
        assert!(state.notification.is_some());

        let note: &Notification = state.notification.as_ref().unwrap();
        assert_eq!(note.message, "Task title cannot be empty!");
    }

    #[test]
    fn should_handle_update_and_maintain_focus() {
        let (mut state, mut ui) = setup();
        let task: Todo = Todo::new("Low", "", Some(Priority::Low));
        let id: Uuid = task.id;

        state.todos.push(task);
        state
            .todos
            .push(Todo::new("High", "", Some(Priority::High)));

        TodoService::sorting(&mut state.todos);
        state.select_state.select(Some(1));

        let mut updated_task: Todo = state.todos[1].clone();
        let mut ctrl = ApplicationController::new(&mut state, &mut ui);

        updated_task.title = "Now First".into();
        updated_task.priority = Priority::High;

        ctrl.dispatch_update(id, updated_task);

        assert_eq!(state.select_state.selected(), Some(0));
        assert_eq!(state.todos[0].title, "Now First");

        let note: &Notification = state.notification.as_ref().unwrap();
        assert_eq!(note.message, "Task 1 / 2 was updated");
    }

    #[test]
    fn should_handle_empty_title_error_on_update() {
        let (mut state, mut ui) = setup();
        let task: Todo = Todo::new("Task", "", Some(Priority::Low));
        let id: Uuid = task.id;

        state.todos.push(task);
        let mut updated_task: Todo = state.todos[0].clone();
        let mut ctrl = ApplicationController::new(&mut state, &mut ui);

        updated_task.title = "".into();
        ctrl.dispatch_update(id, updated_task);

        let note: &Notification = state.notification.as_ref().unwrap();
        assert!(state.notification.is_some());
        assert_eq!(note.message, "Task title cannot be empty!")
    }

    #[test]
    fn should_handle_update_non_existent_task() {
        let (mut state, mut ui) = setup();
        let mut ctrl = ApplicationController::new(&mut state, &mut ui);
        let fake_id: Uuid = Uuid::new_v4();
        let task: Todo = Todo::new("Title", "", None);

        ctrl.dispatch_update(fake_id, task);

        let note: &Notification = state.notification.as_ref().unwrap();
        assert!(state.notification.is_some());
        assert_eq!(note.message, "Task was not found by the provided id!");
    }

    #[test]
    fn should_remove_task_and_adjust_selection() {
        let (mut state, mut ui) = setup();
        state.todos.push(Todo::new("T1", "", None));
        state.todos.push(Todo::new("T2", "", None));
        state.select_state.select(Some(1));

        let mut ctrl = ApplicationController::new(&mut state, &mut ui);
        ctrl.dispatch_remove();

        assert_eq!(state.todos.len(), 1);
        assert_eq!(state.select_state.selected(), Some(0));
        assert!(state.notification.unwrap().message.contains("removed"));
    }

    #[test]
    fn should_handle_remove_non_existent_task() {
        let (mut state, mut ui) = setup();
        state.todos.push(Todo::new("Task", "", None));
        state.select_state.select(Some(999));

        let mut ctrl = ApplicationController::new(&mut state, &mut ui);
        ctrl.dispatch_remove();

        let note: &Notification = state.notification.as_ref().unwrap();
        assert!(state.notification.is_some());
        assert_eq!(note.message, "Task was not found by the provided id!");
    }

    #[test]
    fn should_handle_move_task_logic_and_errors() {
        let (mut state, mut ui) = setup();

        let t1 = Todo::new("T1", "", Some(Priority::Low));
        let t1_id = t1.id;
        state.todos.push(t1);
        state.todos.push(Todo::new("T2", "", Some(Priority::Low)));
        state
            .todos
            .push(Todo::new("High", "", Some(Priority::High)));

        TodoService::sorting(&mut state.todos);

        let initial_pos: usize = state
            .todos
            .iter()
            .position(|t| t.id == t1_id)
            .expect("T1 must exist");
        state.select_state.select(Some(initial_pos));

        {
            let mut ctrl = ApplicationController::new(&mut state, &mut ui);
            ctrl.dispatch_move_task(-1);
        }

        let new_pos: usize = state
            .todos
            .iter()
            .position(|t| t.id == t1_id)
            .expect("T1 must exist");

        assert_eq!(state.select_state.selected(), Some(new_pos));
        assert!(new_pos <= initial_pos);
    }

    #[test]
    fn should_clear_with_empty_list_error() {
        let (mut state, mut ui) = setup();
        let mut ctrl = ApplicationController::new(&mut state, &mut ui);

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
        let (mut state, mut ui) = setup();
        let mut ctrl = ApplicationController::new(&mut state, &mut ui);

        ctrl.dispatch_save();
        assert!(ui.modal.is_some());
    }
}
