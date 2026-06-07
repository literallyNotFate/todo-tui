use std::path::PathBuf;
use thiserror::Error;

/// Errors in application (storage - refers to save/load, task - state methods, keymap - keymap configuration)
#[derive(Error, Debug, PartialEq)]
pub enum ApplicationError {
    #[error(transparent)]
    Task(#[from] TaskError),

    #[error(transparent)]
    Storage(#[from] StorageError),

    #[error(transparent)]
    KeyMap(#[from] KeyMapError),
}

/// Errors related to task operations only
#[derive(Error, Debug, PartialEq)]
pub enum TaskError {
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
    #[error("Requested path was not found: {path}")]
    PathNotFound { path: PathBuf },

    #[error("IO error at {path}: {src}")]
    IO { path: PathBuf, src: String },

    #[error("Failed to parse JSON at {path}: {src}")]
    JSON { path: PathBuf, src: String },

    #[error("Failed to parse TOML at {path}: {src}")]
    TOML { path: PathBuf, src: String },

    #[error("Failed to determine {context} directory: check your OS environment variables")]
    Environment { context: String },
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
        let error = TaskError::TaskNotSelected;
        let mut s = String::new();
        write!(&mut s, "{}", error).unwrap();
        assert_eq!(s, "No task was selected!");
    }

    #[test]
    fn should_return_text_for_task_not_found_error() {
        let error = TaskError::TaskNotFound;
        let mut s = String::new();
        write!(&mut s, "{}", error).unwrap();
        assert_eq!(s, "Task was not found by the provided id!");
    }

    #[test]
    fn should_return_text_for_empty_title_error() {
        let error = TaskError::EmptyTitle;
        let mut s = String::new();
        write!(&mut s, "{}", error).unwrap();
        assert_eq!(s, "Task title cannot be empty!");
    }

    #[test]
    fn should_return_text_for_list_empty_error() {
        let error = TaskError::ListEmpty;
        let mut s = String::new();
        write!(&mut s, "{}", error).unwrap();
        assert_eq!(s, "Cannot clear the tasks! The list is already empty!");
    }

    #[test]
    fn should_return_text_for_move_forbidden_error() {
        let error = TaskError::MoveForbidden;
        let mut s = String::new();
        write!(&mut s, "{}", error).unwrap();
        assert_eq!(s, "Cannot move the tasks!");
    }

    #[test]
    fn should_return_text_for_path_not_found() {
        let path = PathBuf::from("/test/path");
        let error = StorageError::PathNotFound { path: path.clone() };

        let mut s = String::new();
        write!(&mut s, "{}", error).unwrap();

        assert_eq!(s, "Requested path was not found: /test/path");
    }

    #[test]
    fn should_return_text_for_io_error() {
        let path = PathBuf::from("config.toml");
        let error = StorageError::IO {
            path: path.clone(),
            src: "permission denied".to_string(),
        };

        let mut s = String::new();
        write!(&mut s, "{}", error).unwrap();

        assert_eq!(s, "IO error at config.toml: permission denied");
    }

    #[test]
    fn should_return_text_for_json_error() {
        let path = PathBuf::from("data.json");
        let error = StorageError::JSON {
            path: path.clone(),
            src: "unexpected end of file".to_string(),
        };

        let mut s = String::new();
        write!(&mut s, "{}", error).unwrap();

        assert_eq!(
            s,
            "Failed to parse JSON at data.json: unexpected end of file"
        );
    }

    #[test]
    fn should_return_text_for_toml_error() {
        let path = PathBuf::from("theme.toml");
        let error = StorageError::TOML {
            path: path.clone(),
            src: "invalid color format".to_string(),
        };

        let mut s = String::new();
        write!(&mut s, "{}", error).unwrap();

        assert_eq!(
            s,
            "Failed to parse TOML at theme.toml: invalid color format"
        );
    }

    #[test]
    fn should_return_text_for_environment_error() {
        let error = StorageError::Environment {
            context: "config".to_string(),
        };
        let mut s = String::new();
        write!(&mut s, "{}", error).unwrap();
        assert_eq!(
            s,
            "Failed to determine config directory: check your OS environment variables"
        );
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
