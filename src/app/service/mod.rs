pub mod folder_service;
pub mod task_service;

pub use folder_service::FolderService;
pub use task_service::TaskService;

use crate::models::{Folder, Task};

/// Enum to specify return value from the services
pub enum OperationResult {
    TaskCreated {
        index: usize,
        task: Task,
    },
    TaskUpdated {
        index: usize,
        old: Task,
        new: Task,
    },
    TaskRemoved {
        task: Task,
    },

    FolderCreated {
        index: usize,
        folder: Folder,
    },
    FolderUpdated {
        index: usize,
        old: Folder,
        new: Folder,
    },
    FolderRemoved {
        folder: Folder,
    },
}

impl OperationResult {
    pub fn unwrap_task_created(self) -> (usize, Task) {
        match self {
            Self::TaskCreated { index, task } => (index, task),
            _ => panic!("Expected OperationResult::TaskCreated, but got another variant"),
        }
    }

    pub fn unwrap_task_updated(self) -> (usize, Task, Task) {
        match self {
            Self::TaskUpdated { index, old, new } => (index, old, new),
            _ => panic!("Expected OperationResult::TaskUpdated, but got another variant"),
        }
    }

    pub fn unwrap_task_removed(self) -> Task {
        match self {
            Self::TaskRemoved { task } => task,
            _ => panic!("Expected OperationResult::TaskRemoved, but got another variant"),
        }
    }

    pub fn unwrap_folder_created(self) -> (usize, Folder) {
        match self {
            Self::FolderCreated { index, folder } => (index, folder),
            _ => panic!("Expected OperationResult::FolderCreated, but got another variant"),
        }
    }

    pub fn unwrap_folder_updated(self) -> (usize, Folder, Folder) {
        match self {
            Self::FolderUpdated { index, old, new } => (index, old, new),
            _ => panic!("Expected OperationResult::FolderUpdated, but got another variant"),
        }
    }

    pub fn unwrap_folder_removed(self) -> Folder {
        match self {
            Self::FolderRemoved { folder } => folder,
            _ => panic!("Expected OperationResult::FolderRemoved, but got another variant"),
        }
    }

    pub fn entity_title(&self) -> &str {
        match self {
            Self::TaskCreated { task, .. } => &task.title,
            Self::TaskRemoved { task } => &task.title,
            Self::TaskUpdated { new, .. } => &new.title,
            Self::FolderCreated { folder, .. } => &folder.name,
            Self::FolderUpdated { new, .. } => &new.name,
            Self::FolderRemoved { folder } => &folder.name,
        }
    }
}
