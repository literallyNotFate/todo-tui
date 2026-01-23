use super::{Priority, Todo};
use crate::traits::InteractableEnum;

#[derive(Default, Clone, Copy, Debug, PartialEq)]
pub enum Filter {
    #[default]
    All,
    Active,
    Completed,
    HighPriority,
}

impl InteractableEnum for Filter {
    fn all_variants() -> &'static [Self] {
        &[Self::All, Self::Active, Self::Completed, Self::HighPriority]
    }

    fn to_string(&self) -> &'static str {
        match self {
            Self::All => "All",
            Self::Active => "Active",
            Self::Completed => "Completed",
            Self::HighPriority => "High Priority",
        }
    }
}

impl Filter {
    pub fn count(&self, todos: &[Todo]) -> usize {
        self.filter(todos).len()
    }

    pub fn filter(&self, todos: &[Todo]) -> Vec<Todo> {
        match self {
            Self::All => todos.to_vec(),
            Self::Active => todos.iter().filter(|t| !t.completed).cloned().collect(),
            Self::Completed => todos.iter().filter(|t| t.completed).cloned().collect(),
            Self::HighPriority => todos
                .iter()
                .filter(|t| matches!(t.priority, Priority::High))
                .cloned()
                .collect(),
        }
    }
}

// Unit-tests for filter
#[cfg(test)]
mod tests {
    use super::*;

    fn setup_test_todos() -> Vec<Todo> {
        vec![
            Todo::new("Task 1", "Desc", Some(Priority::Low)),
            {
                let mut t = Todo::new("Task 2", "Desc", Some(Priority::Medium));
                t.completed = true;
                t
            },
            Todo::new("Task 3", "Desc", Some(Priority::High)),
            {
                let mut t = Todo::new("Task 4", "Desc", Some(Priority::High));
                t.completed = true;
                t
            },
        ]
    }

    #[test]
    fn should_filter_todos_based_on_enum_value() {
        let todos = setup_test_todos();

        assert_eq!(Filter::All.count(&todos), 4);

        let active = Filter::Active.filter(&todos);
        assert_eq!(active.len(), 2);
        assert!(active.iter().all(|t| !t.completed));

        let completed = Filter::Completed.filter(&todos);
        assert_eq!(completed.len(), 2);
        assert!(completed.iter().all(|t| t.completed));

        let high = Filter::HighPriority.filter(&todos);
        assert_eq!(high.len(), 2);
        assert!(high.iter().all(|t| matches!(t.priority, Priority::High)));
    }
}
