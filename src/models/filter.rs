use super::{Priority, Todo};
use crate::traits::InteractableEnum;
use chrono::Local;

#[derive(Default, Clone, Copy, Debug, PartialEq)]
pub enum Filter {
    #[default]
    All,
    Active,
    Completed,
    HighPriority,
    Today,
}

impl InteractableEnum for Filter {
    fn all_variants() -> &'static [Self] {
        &[
            Self::All,
            Self::Active,
            Self::Completed,
            Self::HighPriority,
            Self::Today,
        ]
    }

    fn to_string(&self) -> &'static str {
        match self {
            Self::All => "All",
            Self::Active => "Active",
            Self::Completed => "Completed",
            Self::HighPriority => "High Priority",
            Self::Today => "Today",
        }
    }
}

impl Filter {
    pub fn count(&self, todos: &[Todo], query: &str) -> usize {
        self.apply(todos, query).len()
    }

    pub fn apply(&self, todos: &[Todo], query: &str) -> Vec<Todo> {
        todos
            .iter()
            .filter(|t| {
                let matches_type = match self {
                    Self::All => true,
                    Self::Active => !t.completed,
                    Self::Completed => t.completed,
                    Self::HighPriority => matches!(t.priority, Priority::High),
                    Self::Today => {
                        let today = Local::now().date_naive();
                        t.created_at.with_timezone(&Local).date_naive() == today
                    }
                };

                let matches_query = query.is_empty() || t.title.to_lowercase().contains(&query);
                matches_type && matches_query
            })
            .cloned()
            .collect()
    }
}

// Unit-tests for filter
#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Duration, Utc};

    fn setup_test_todos() -> Vec<Todo> {
        vec![
            Todo::new("Task 1", "Desc", Some(Priority::Low)),
            {
                let mut t = Todo::new("Task 2", "Desc", Some(Priority::Medium));
                t.completed = true;
                t.created_at = Utc::now() - Duration::days(1);
                t
            },
            Todo::new("Task 3", "Desc", Some(Priority::High)),
            {
                let mut t = Todo::new("Task 4", "Desc", Some(Priority::High));
                t.completed = true;
                t.created_at = Utc::now() - Duration::weeks(2);
                t
            },
        ]
    }

    #[test]
    fn should_filter_todos_based_on_enum_value() {
        let todos = setup_test_todos();
        assert_eq!(Filter::All.count(&todos, ""), 4);

        let active = Filter::Active.apply(&todos, "");
        assert_eq!(active.len(), 2);
        assert!(active.iter().all(|t| !t.completed));

        let completed = Filter::Completed.apply(&todos, "");
        assert_eq!(completed.len(), 2);
        assert!(completed.iter().all(|t| t.completed));

        let high = Filter::HighPriority.apply(&todos, "");
        assert_eq!(high.len(), 2);
        assert!(high.iter().all(|t| matches!(t.priority, Priority::High)));

        let today = Filter::Today.apply(&todos, "");
        assert_eq!(today.len(), 2);
    }

    #[test]
    fn should_filter_todos_based_on_search_query() {
        let todos = setup_test_todos();
        let results = Filter::Active.apply(&todos, "Task 4");

        assert_eq!(
            results.len(),
            0,
            "Should not find completed tasks when Active filter is on"
        );

        let high_results = Filter::HighPriority.apply(&todos, "Task");
        assert!(
            high_results
                .iter()
                .all(|t| matches!(t.priority, Priority::High))
        );
    }
}
