// Unit-tests for ApplicationState
#[cfg(test)]
mod tests {
    use crate::app::{
        models::todo::Todo,
        state::{
            error::ApplicationStateError,
            state::{ApplicationResult, ApplicationState},
        },
        utils::constants::text::{CLEARED_TASKS_TEXT, REMOVED_TASK_TEXT},
    };

    // Helper function to setup list with multiple tasks (non empty)
    fn setup_with_n_todos(n: usize) -> ApplicationState {
        let mut state: ApplicationState = ApplicationState::new();
        for i in 1..=n {
            let _: ApplicationResult<String> = state.append_todo(format!("Task {}", i));
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
        let result: ApplicationResult<String> = state.append_todo("Test");

        assert_eq!(result, Ok(String::from("Task Test was added to the list!")));
        assert_eq!(state.todos.len(), 1);
        assert_eq!(state.todos[0].title, "Test");
        assert!(!state.todos[0].done);
        assert_eq!(state.select_state.selected(), Some(0));
    }

    #[test]
    fn should_invoke_empty_title_error_on_append() {
        let mut state: ApplicationState = ApplicationState::new();
        let result: ApplicationResult<String> = state.append_todo("");

        assert_eq!(result, Err(ApplicationStateError::EmptyTitle));
        assert!(state.todos.is_empty());
    }

    #[test]
    fn should_invoke_task_exists_on_append() {
        let mut state: ApplicationState = setup_with_n_todos(2);
        let title: String = String::from("Task 1");
        let result: ApplicationResult<String> = state.append_todo(&title);

        assert_eq!(result, Err(ApplicationStateError::TaskAlreadyExists(title)));
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
                1, 2, new_title
            )),
        );
        assert_eq!(state.todos[1].title, new_title);
        assert_eq!(state.todos[0].title, "Task 1");
    }

    #[test]
    fn should_invoke_empty_title_error_on_rename() {
        let mut state: ApplicationState = setup_with_n_todos(1);
        let result: ApplicationResult<String> = state.rename_todo("");

        assert_eq!(result, Err(ApplicationStateError::EmptyTitle));
        assert_eq!(state.todos[0].title, "Task 1");
    }

    #[test]
    fn should_invoke_task_exists_on_rename() {
        let mut state: ApplicationState = setup_with_n_todos(3);
        let title: String = "Task 1".to_string();
        let result: ApplicationResult<String> = state.rename_todo(&title);

        assert_eq!(result, Err(ApplicationStateError::TaskAlreadyExists(title)));
        assert_eq!(state.todos[2].title, "Task 3");
    }

    #[test]
    fn should_invoke_not_selected_error_on_rename() {
        let mut state: ApplicationState = ApplicationState::new();
        state.select_state.select(None);
        let result: ApplicationResult<String> = state.rename_todo("Should fail");

        assert_eq!(result, Err(ApplicationStateError::TaskNotSelected));
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
        let mut state: ApplicationState = ApplicationState::new();
        let result: ApplicationResult<String> = state.remove_todo();

        assert_eq!(result, Err(ApplicationStateError::CannotRemoveFromEmpty));
    }

    #[test]
    fn should_invoke_not_selected_error_on_remove() {
        let mut state: ApplicationState = setup_with_n_todos(1);
        state.select_state.select(None);
        let result: ApplicationResult<String> = state.remove_todo();

        assert_eq!(result, Err(ApplicationStateError::TaskNotSelected));
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
        let mut state: ApplicationState = ApplicationState::new();
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
        let mut state: ApplicationState = ApplicationState::new();
        let result: ApplicationResult<String> = state.clear_todos();

        assert_eq!(result, Err(ApplicationStateError::ListEmpty));
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
    fn should_return_last_todo_if_is_out_of_bounds() {
        let mut state: ApplicationState = setup_with_n_todos(1);
        state.select_state.select(Some(999));

        assert!(state.current_todo().is_none());
    }
}
