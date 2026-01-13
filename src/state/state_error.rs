use std::fmt::{Display, Formatter, Result};

// Errors in application
#[derive(Debug, PartialEq)]
pub enum ApplicationStateError {
    TaskNotSelected,
    TaskAlreadyExists(String),
    EmptyTitle,
    ListEmpty,
    CannotRemoveFromEmpty,
}

// to_string() for errors
impl Display for ApplicationStateError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            ApplicationStateError::ListEmpty => {
                write!(f, "Cannot clear the tasks! The list is already empty!")
            }
            ApplicationStateError::TaskNotSelected => write!(f, "No task was selected!"),
            ApplicationStateError::TaskAlreadyExists(task) => {
                write!(f, "Task with title \"{}\" already exists!", task)
            }
            ApplicationStateError::EmptyTitle => write!(f, "Task title cannot be empty!"),
            ApplicationStateError::CannotRemoveFromEmpty => {
                write!(f, "Cannot remove the task from the empty list!")
            }
        }
    }
}

// Unit-tests for application state errors (testing messages)
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_return_text_for_task_not_selected_error() {
        let error: ApplicationStateError = ApplicationStateError::TaskNotSelected;
        assert_eq!(error.to_string(), "No task was selected!");
    }

    #[test]
    fn should_return_text_for_empty_title_error() {
        let error: ApplicationStateError = ApplicationStateError::EmptyTitle;
        assert_eq!(error.to_string(), "Task title cannot be empty!");
    }

    #[test]
    fn should_return_text_for_list_empty_error() {
        let error: ApplicationStateError = ApplicationStateError::ListEmpty;
        assert_eq!(
            error.to_string(),
            "Cannot clear the tasks! The list is already empty!"
        );
    }

    #[test]
    fn should_return_text_for_cannot_remove_from_empty_error() {
        let error: ApplicationStateError = ApplicationStateError::CannotRemoveFromEmpty;
        assert_eq!(
            error.to_string(),
            "Cannot remove the task from the empty list!"
        );
    }

    #[test]
    fn should_return_text_for_task_already_exists_error() {
        let error: ApplicationStateError =
            ApplicationStateError::TaskAlreadyExists("Some task".to_string());
        assert_eq!(
            error.to_string(),
            "Task with title \"Some task\" already exists!"
        );

        let error: ApplicationStateError = ApplicationStateError::TaskAlreadyExists("".to_string());
        assert_eq!(error.to_string(), "Task with title \"\" already exists!");
    }
}
