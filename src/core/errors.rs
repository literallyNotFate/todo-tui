use thiserror::Error;

/// Errors in application (storage - refers to save/load, todo - state methods)
#[derive(Error, Debug, PartialEq)]
pub enum ApplicationError {
    #[error("{0}")]
    Todo(TodoError),
    #[error("{0}")]
    Storage(StorageError),
}

/// Errors related to todo operations only
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
    #[error("Cannot move the tasks!")]
    MoveForbidden,
}

/// Errors related to storage operations only
#[derive(Error, Debug, PartialEq)]
pub enum StorageError {
    #[error("Requested path was not found!")]
    PathNotFound,
    #[error("Cannot read/write to file: {0}")]
    IOError(String),
    #[error("Cannot read/write to JSON: {0}")]
    JSONError(String),
    #[error("Cannot read/write to TOML: {0}")]
    TOMLError(String),
    #[error("Failed to determine home or config directory")]
    EnvironmentError,
}

/// Casting
impl From<std::io::Error> for StorageError {
    fn from(err: std::io::Error) -> Self {
        StorageError::IOError(err.to_string())
    }
}

impl From<serde_json::Error> for StorageError {
    fn from(err: serde_json::Error) -> Self {
        StorageError::JSONError(err.to_string())
    }
}

impl From<toml::de::Error> for StorageError {
    fn from(err: toml::de::Error) -> Self {
        StorageError::TOMLError(err.to_string())
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

/// Unit-tests for application state errors (testing messages)
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
    fn should_return_text_for_move_forbidden_error() {
        let error = TodoError::MoveForbidden;
        let mut s = String::new();
        write!(&mut s, "{}", error).unwrap();
        assert_eq!(s, "Cannot move the tasks!");
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
        let error = StorageError::IOError("some error".to_string());
        let mut s = String::new();
        write!(&mut s, "{}", error).unwrap();
        assert_eq!(s, "Cannot read/write to file: some error");
    }

    #[test]
    fn should_return_text_for_json_error() {
        let error = StorageError::JSONError("some error".to_string());
        let mut s = String::new();
        write!(&mut s, "{}", error).unwrap();
        assert_eq!(s, "Cannot read/write to JSON: some error");
    }

    #[test]
    fn should_return_text_for_toml_error() {
        let error = StorageError::TOMLError("some error".to_string());
        let mut s = String::new();
        write!(&mut s, "{}", error).unwrap();
        assert_eq!(s, "Cannot read/write to TOML: some error");
    }

    #[test]
    fn should_return_text_for_environment_error() {
        let error = StorageError::EnvironmentError;
        let mut s = String::new();
        write!(&mut s, "{}", error).unwrap();
        assert_eq!(s, "Failed to determine home or config directory");
    }
}
