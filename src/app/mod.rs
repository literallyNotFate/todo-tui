pub mod controller;
pub mod service;

pub use controller::ApplicationController;
pub use service::TodoService;

use crate::models::Todo;

/// What is being returned after successful append operation
pub struct TaskCreatedResult {
    pub index: usize,
    pub task: Todo,
}

/// What is being returned after successful update operation
pub struct TaskUpdatedResult {
    pub index: usize,
    pub old: Todo,
    pub new: Todo,
}

/// What is being return after successful remove operation
pub struct TaskRemovedResult {
    pub task: Todo,
}
