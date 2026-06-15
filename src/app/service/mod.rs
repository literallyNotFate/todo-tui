pub mod folder_service;
pub mod task_service;

pub use folder_service::FolderService;
pub use task_service::TaskService;

use crate::models::{Folder, Task};

/// Enum to specify return value from the services
pub enum OperationResult {
    TaskCreated { task: Task },
    TaskUpdated { old: Task, new: Task },
    TaskRemoved { task: Task },

    FolderCreated { folder: Folder },
    FolderUpdated { old: Folder, new: Folder },
    FolderRemoved { folder: Folder },
}

macro_rules! unwrap_variant {
    ($name:ident, $variant:ident, $ret:ty, $($field:ident),+) => {
        pub fn $name(self) -> $ret {
            match self {
                Self::$variant { $($field),+ } => ($($field),+),
                _ => panic!(concat!("Expected ", stringify!($variant), ", but got another variant")),
            }
        }
    };
}

impl OperationResult {
    unwrap_variant!(unwrap_task_created, TaskCreated, Task, task);
    unwrap_variant!(unwrap_task_updated, TaskUpdated, (Task, Task), old, new);
    unwrap_variant!(unwrap_task_removed, TaskRemoved, Task, task);
    unwrap_variant!(unwrap_folder_created, FolderCreated, Folder, folder);
    unwrap_variant!(
        unwrap_folder_updated,
        FolderUpdated,
        (Folder, Folder),
        old,
        new
    );
    unwrap_variant!(unwrap_folder_removed, FolderRemoved, Folder, folder);

    /// Get entity title from result
    pub fn entity_title(&self) -> &str {
        match self {
            Self::TaskCreated { task } => &task.title,
            Self::TaskRemoved { task } => &task.title,
            Self::TaskUpdated { new, .. } => &new.title,
            Self::FolderCreated { folder } => &folder.name,
            Self::FolderUpdated { new, .. } => &new.name,
            Self::FolderRemoved { folder } => &folder.name,
        }
    }
}
