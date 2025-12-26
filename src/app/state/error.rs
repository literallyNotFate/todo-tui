use std::fmt::{Display, Formatter, Result};

// Errors in application
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
