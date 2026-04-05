use thiserror::Error;

/// Errors in application (storage - refers to save/load, todo - state methods, keymap - keymap configuration)
#[derive(Error, Debug, PartialEq)]
pub enum ApplicationError {
    #[error(transparent)]
    Todo(#[from] TodoError),

    #[error(transparent)]
    Storage(#[from] StorageError),

    #[error(transparent)]
    KeyMap(#[from] KeyMapError),
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
    #[error("IO error: {0}")]
    IO(String),
    #[error("JSON error: {0}")]
    JSON(String),
    #[error("TOML error: {0}")]
    TOML(String),
    #[error("Failed to determine home or config directory")]
    Environment,
}

#[derive(Error, Debug, PartialEq)]
pub enum KeyMapError {
    #[error("Key conflict: '{key}' assigned to '{first_action}' and '{second_action}'")]
    DuplicateKey {
        key: String,
        first_action: String,
        second_action: String,
    },
    #[error("Unknown action: {0}")]
    UnknownAction(String),
    #[error("Invalid key format: '{0}'")]
    InvalidKeyFormat(String),
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
        let error = StorageError::IO("some error".to_string());
        let mut s = String::new();
        write!(&mut s, "{}", error).unwrap();
        assert_eq!(s, "IO error: some error");
    }

    #[test]
    fn should_return_text_for_json_error() {
        let error = StorageError::JSON("some error".to_string());
        let mut s = String::new();
        write!(&mut s, "{}", error).unwrap();
        assert_eq!(s, "JSON error: some error");
    }

    #[test]
    fn should_return_text_for_toml_error() {
        let error = StorageError::TOML("some error".to_string());
        let mut s = String::new();
        write!(&mut s, "{}", error).unwrap();
        assert_eq!(s, "TOML error: some error");
    }

    #[test]
    fn should_return_text_for_environment_error() {
        let error = StorageError::Environment;
        let mut s = String::new();
        write!(&mut s, "{}", error).unwrap();
        assert_eq!(s, "Failed to determine home or config directory");
    }

    #[test]
    fn should_return_text_for_duplicate_key_error() {
        let error = KeyMapError::DuplicateKey {
            key: "ctrl+s".to_string(),
            first_action: "Save".to_string(),
            second_action: "Quit".to_string(),
        };

        let s = format!("{}", error);
        assert_eq!(s, "Key conflict: 'ctrl+s' assigned to 'Save' and 'Quit'");
    }

    #[test]
    fn should_return_text_for_unknown_action_error() {
        let error = KeyMapError::UnknownAction("super_punch".to_string());
        let s = format!("{}", error);
        assert_eq!(s, "Unknown action: super_punch");
    }

    #[test]
    fn should_return_text_for_invalid_key_format_error() {
        let error = KeyMapError::InvalidKeyFormat("ctrl-alt-del-extra".to_string());
        let s = format!("{}", error);
        assert_eq!(s, "Invalid key format: 'ctrl-alt-del-extra'");
    }
}
