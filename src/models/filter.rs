use super::Task;
use serde::{Deserialize, Serialize};

/// Selected filter enum
#[derive(
    Default,
    Serialize,
    Deserialize,
    Clone,
    Copy,
    Debug,
    PartialEq,
    strum::EnumIter,
    strum::Display,
    strum::EnumString,
)]
#[strum(serialize_all = "title_case")]
pub enum Filter {
    #[default]
    All,
    Active,
    Completed,
    HighPriority,
    Today,
}

impl Filter {
    pub fn count(&self, tasks: &[Task], query: &str) -> usize {
        self.apply(tasks, query).len()
    }

    pub fn apply<'a>(&self, tasks: &'a [Task], query: &str) -> Vec<&'a Task> {
        let query_lower: String = query.to_lowercase().trim().to_string();
        let is_empty: bool = query_lower.is_empty();
        let today = chrono::Local::now().date_naive();

        tasks
            .iter()
            .filter(|t| {
                if !t.matches_filter(self, &today) {
                    return false;
                }
                is_empty || t.title_lower.contains(&query_lower)
            })
            .collect()
    }
}

// Unit-tests for filter
#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::task::Priority;
    use chrono::{Duration, Utc};

    fn setup_test_tasks() -> Vec<Task> {
        vec![
            Task::new("Task 1"),
            {
                let mut t = Task::new("Task 2")
                    .with_description("Desc")
                    .with_priority(Priority::Medium);
                t.completed = true;
                t.created_at = Utc::now() - Duration::days(1);
                t
            },
            Task::new("Task 3")
                .with_description("Desc")
                .with_priority(Priority::High),
            {
                let mut t = Task::new("Task 4").with_priority(Priority::High);
                t.completed = true;
                t.created_at = Utc::now() - Duration::weeks(2);
                t
            },
        ]
    }

    #[test]
    fn should_filter_tasks_based_on_enum_value() {
        let tasks = setup_test_tasks();
        assert_eq!(Filter::All.count(&tasks, ""), 4);

        let active = Filter::Active.apply(&tasks, "");
        assert_eq!(active.len(), 2);
        assert!(active.iter().all(|t| !t.completed));

        let completed = Filter::Completed.apply(&tasks, "");
        assert_eq!(completed.len(), 2);
        assert!(completed.iter().all(|t| t.completed));

        let high = Filter::HighPriority.apply(&tasks, "");
        assert_eq!(high.len(), 2);
        assert!(high.iter().all(|t| matches!(t.priority, Priority::High)));

        let today = Filter::Today.apply(&tasks, "");
        assert_eq!(today.len(), 2);
    }

    #[test]
    fn should_filter_tasks_based_on_search_query() {
        let tasks = setup_test_tasks();
        let results = Filter::Active.apply(&tasks, "Task 4");

        assert_eq!(
            results.len(),
            0,
            "Should not find completed tasks when Active filter is on"
        );

        let high_results = Filter::HighPriority.apply(&tasks, "Task");
        assert!(
            high_results
                .iter()
                .all(|t| matches!(t.priority, Priority::High))
        );
    }
}
