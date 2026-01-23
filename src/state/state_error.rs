use thiserror::Error;

// Errors in application (storage - refers to save/load, todo - state methods)
#[derive(Error, Debug, PartialEq)]
pub enum ApplicationError {
    #[error("{0}")]
    Todo(TodoError),
    #[error("{0}")]
    Storage(StorageError),
}

#[derive(Error, Debug, PartialEq)]
pub enum TodoError {
    #[error("No task was selected!")]
    TaskNotSelected,
    #[error("Task was not found by the provided id!")]
    TaskNotFound,
    #[error("Task title cannot be empty!")]
    EmptyTitle,
    #[error("Cannot clear the tasks! The list is already empty!")]
    ListEmpty,
}

#[derive(Error, Debug, PartialEq)]
pub enum StorageError {
    #[error("Requested path was not found!")]
    PathNotFound,
    #[error("Cannot read/write tasks file!")]
    IOError,
    #[error("Cannot write to JSON!")]
    JSONError,
}

// For error casting
impl From<std::io::Error> for StorageError {
    fn from(_: std::io::Error) -> Self {
        StorageError::IOError
    }
}

impl From<serde_json::Error> for StorageError {
    fn from(_: serde_json::Error) -> Self {
        StorageError::JSONError
    }
}

impl From<TodoError> for ApplicationError {
    fn from(err: TodoError) -> Self {
        ApplicationError::Todo(err)
    }
}

impl From<StorageError> for ApplicationError {
    fn from(err: StorageError) -> Self {
        ApplicationError::Storage(err)
    }
}

// Unit-tests for application state errors (testing messages)
#[cfg(test)]
mod tests {
    use super::*;
    use std::fmt::Write;

    #[test]
    fn should_return_text_for_task_not_selected_error() {
        let error = TodoError::TaskNotSelected;
        let mut s = String::new();
        write!(&mut s, "{}", error).unwrap();
        assert_eq!(s, "No task was selected!");
    }

    #[test]
    fn should_return_text_for_task_not_found_error() {
        let error = TodoError::TaskNotFound;
        let mut s = String::new();
        write!(&mut s, "{}", error).unwrap();
        assert_eq!(s, "Task was not found by the provided id!");
    }

    #[test]
    fn should_return_text_for_empty_title_error() {
        let error = TodoError::EmptyTitle;
        let mut s = String::new();
        write!(&mut s, "{}", error).unwrap();
        assert_eq!(s, "Task title cannot be empty!");
    }

    #[test]
    fn should_return_text_for_list_empty_error() {
        let error = TodoError::ListEmpty;
        let mut s = String::new();
        write!(&mut s, "{}", error).unwrap();
        assert_eq!(s, "Cannot clear the tasks! The list is already empty!");
    }

    #[test]
    fn should_return_text_for_path_not_found() {
        let error = StorageError::PathNotFound;
        let mut s = String::new();
        write!(&mut s, "{}", error).unwrap();
        assert_eq!(s, "Requested path was not found!");
    }

    #[test]
    fn should_return_text_for_io_error() {
        let error = StorageError::IOError;
        let mut s = String::new();
        write!(&mut s, "{}", error).unwrap();
        assert_eq!(s, "Cannot read/write tasks file!");
    }

    #[test]
    fn should_return_text_for_json_error() {
        let error = StorageError::JSONError;
        let mut s = String::new();
        write!(&mut s, "{}", error).unwrap();
        assert_eq!(s, "Cannot write to JSON!");
    }
}
