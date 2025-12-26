// Unit-tests for ApplicationState
#[cfg(test)]
mod tests {
    use crate::app::{
        models::todo::Todo,
        state::{
            error::ApplicationStateError,
            state::{ApplicationResult, ApplicationState},
        },
    };

    // Helper function to setup list with multiple tasks (non empty)
    fn setup_with_n_todos(n: usize) -> ApplicationState {
        let mut state: ApplicationState = ApplicationState::new();
        for i in 1..=n {
            let _: ApplicationResult<()> = state.append_todo(format!("Task {}", i));
        }

        state
    }

    #[test]
    fn should_create_new_state() {
        let state: ApplicationState = ApplicationState::new();
        assert!(state.todos.is_empty());
        assert_eq!(state.select_state.selected(), Some(0));
    }

    #[test]
    fn should_append_todo() {
        let mut state: ApplicationState = ApplicationState::new();

        assert!(state.append_todo("First task").is_ok());

        assert_eq!(state.todos.len(), 1);
        assert_eq!(state.todos[0].title, "First task");
        assert!(!state.todos[0].done);
        assert_eq!(state.select_state.selected(), Some(0));
    }

    #[test]
    fn should_invoke_empty_title_error_on_append() {
        let mut state: ApplicationState = ApplicationState::new();

        let result: ApplicationResult<()> = state.append_todo("");
        assert!(matches!(result, Err(ApplicationStateError::EmptyTitle)));

        assert!(state.todos.is_empty());
    }

    #[test]
    fn should_invoke_task_exists_on_append() {
        let mut state: ApplicationState = setup_with_n_todos(2);

        let result: ApplicationResult<()> = state.append_todo("Task 1");
        assert!(matches!(
            result,
            Err(ApplicationStateError::TaskAlreadyExists(_))
        ));
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
        let _: ApplicationResult<()> = state.rename_todo("Renamed task");

        assert_eq!(state.todos[1].title, "Renamed task");
        assert_eq!(state.todos[0].title, "Task 1");
    }

    #[test]
    fn should_invoke_empty_title_error_on_rename() {
        let mut state: ApplicationState = setup_with_n_todos(1);

        let result: ApplicationResult<()> = state.rename_todo("");
        assert!(matches!(result, Err(ApplicationStateError::EmptyTitle)));
        assert_eq!(state.todos[0].title, "Task 1");
    }

    #[test]
    fn should_invoke_task_exists_on_rename() {
        let mut state: ApplicationState = setup_with_n_todos(3);

        let result: ApplicationResult<()> = state.rename_todo("Task 1");
        assert!(matches!(
            result,
            Err(ApplicationStateError::TaskAlreadyExists(_))
        ));
        assert_eq!(state.todos[2].title, "Task 3");
    }

    #[test]
    fn should_invoke_not_selected_error_on_rename() {
        let mut state: ApplicationState = ApplicationState::new();
        state.select_state.select(None);

        let result: ApplicationResult<()> = state.rename_todo("Should fail");
        assert!(matches!(
            result,
            Err(ApplicationStateError::TaskNotSelected)
        ));
    }

    #[test]
    fn should_remove_todo_in_the_middle() {
        let mut state: ApplicationState = setup_with_n_todos(3);
        state.select_state.select(Some(1));

        let _: ApplicationResult<()> = state.remove_todo();

        assert_eq!(state.todos.len(), 2);
        assert_eq!(state.todos[0].title, "Task 1");
        assert_eq!(state.todos[1].title, "Task 3");
        assert_eq!(state.select_state.selected(), Some(1));
    }

    #[test]
    fn should_remove_todo_last() {
        let mut state: ApplicationState = setup_with_n_todos(2);
        state.select_state.select(Some(1));

        let _: ApplicationResult<()> = state.remove_todo();

        assert_eq!(state.todos.len(), 1);
        assert_eq!(state.todos[0].title, "Task 1");
        assert_eq!(state.select_state.selected(), Some(0));
    }

    #[test]
    fn should_remove_only_one_element() {
        let mut state: ApplicationState = setup_with_n_todos(1);
        let _: ApplicationResult<()> = state.remove_todo();

        assert!(state.todos.is_empty());
        assert_eq!(state.select_state.selected(), None);
    }

    #[test]
    fn should_invoke_cannot_remove_empty_error_on_remove() {
        let mut state: ApplicationState = ApplicationState::new();

        let result = state.remove_todo();
        assert!(matches!(
            result,
            Err(ApplicationStateError::CannotRemoveFromEmpty)
        ));
    }

    #[test]
    fn should_invoke_not_selected_error_on_remove() {
        let mut state: ApplicationState = setup_with_n_todos(1);
        state.select_state.select(None);

        let result: ApplicationResult<()> = state.remove_todo();
        assert!(matches!(
            result,
            Err(ApplicationStateError::TaskNotSelected)
        ));
    }

    #[test]
    fn should_toggle_current_todo() {
        let mut state: ApplicationState = setup_with_n_todos(2);
        state.select_state.select(Some(0));

        assert!(!state.todos[0].done);

        state.toggle_current();

        assert!(state.todos[0].done);
        assert!(!state.todos[1].done);

        state.toggle_current();
        assert!(!state.todos[0].done);
    }

    #[test]
    fn should_toggle_current_on_empty() {
        let mut state: ApplicationState = ApplicationState::new();
        state.select_state.select(None);
        state.toggle_current();

        assert!(state.todos.is_empty());
    }

    #[test]
    fn should_clear_todos() {
        let mut state: ApplicationState = setup_with_n_todos(5);
        let _: ApplicationResult<()> = state.clear_todos();

        assert!(state.todos.is_empty());
    }

    #[test]
    fn should_invoke_list_empty_on_clear() {
        let mut state: ApplicationState = ApplicationState::new();

        let result = state.clear_todos();
        assert!(matches!(result, Err(ApplicationStateError::ListEmpty)));

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
        let state: ApplicationState = ApplicationState::new();
        assert!(state.current_todo().is_none());
    }

    #[test]
    fn should_return_current_todo_safe_out_of_bounds() {
        let mut state: ApplicationState = setup_with_n_todos(1);
        state.select_state.select(Some(999));

        assert!(state.current_todo().is_none());
    }
}
