pub mod controller;
pub mod service;

pub use controller::ApplicationController;
pub use service::TaskService;

use crate::models::Task;

/// What is being returned after successful append operation
pub struct TaskCreatedResult {
    pub index: usize,
    pub task: Task,
}

/// What is being returned after successful update operation
pub struct TaskUpdatedResult {
    pub index: usize,
    pub old: Task,
    pub new: Task,
}

/// What is being return after successful remove operation
pub struct TaskRemovedResult {
    pub task: Task,
}
