use super::Priority;
use serde::{Deserialize, Serialize};
use std::hash::Hash;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone, Default, Hash)]
pub struct Todo {
    pub id: Uuid,
    pub title: String,
    pub description: String,
    pub completed: bool,
    pub priority: Priority,
}

impl Todo {
    pub fn new(
        title: impl Into<String>,
        description: impl Into<String>,
        priority: Option<Priority>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            title: title.into(),
            description: description.into(),
            completed: false,
            priority: priority.unwrap_or_default(),
        }
    }

    pub fn toggle_completed(&mut self) {
        self.completed = !self.completed;
    }
}

// Unit-tests for todo model (basic methods)
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_create_todo_item() {
        let todo: Todo = Todo::new("Test", "Test", None);

        assert_eq!(todo.title, "Test");
        assert_eq!(todo.title, "Test");
        assert_eq!(todo.priority, Priority::Low);
        assert!(!todo.completed);
    }

    #[test]
    fn should_generate_unique_id_for_todos() {
        let title: &str = "Test Task";
        let desc: &str = "Description";

        let todo1 = Todo::new(title, desc, None);
        let todo2 = Todo::new(title, desc, None);

        assert!(!todo1.id.is_nil(), "UUID should not be nil");
        assert_ne!(todo1.id, todo2.id, "each todo must have a unique UUID");

        assert_eq!(
            todo1.id.get_version(),
            Some(uuid::Version::Random),
            "UUID should be version 4"
        );
    }

    #[test]
    fn should_toggle_completed() {
        let mut todo: Todo = Todo::new("Test", "Test", Some(Priority::Medium));
        assert_eq!(todo.priority, Priority::Medium);

        todo.toggle_completed();
        assert!(todo.completed);

        todo.toggle_completed();
        assert!(!todo.completed);
    }
}
