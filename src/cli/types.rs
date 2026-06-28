use crate::{models::Task, state::SidebarTab};
use uuid::Uuid;

/// Enum to find tasks by id
#[derive(Debug)]
pub enum FindResult {
    Found(Uuid),
    Ambiguous(Vec<(Uuid, String)>),
    NotFound,
}

impl FindResult {
    pub fn find(tasks: &[Task], id_query: &str) -> Self {
        let matches: Vec<(Uuid, String)> = tasks
            .iter()
            .filter(|t| t.id.to_string().starts_with(id_query))
            .map(|t| (t.id, t.title.clone()))
            .collect();

        match matches.len() {
            0 => Self::NotFound,
            1 => Self::Found(matches[0].0),
            _ => Self::Ambiguous(matches),
        }
    }
}

/// Enum to filter tasks via CLI (list command)
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, clap::ValueEnum, Debug)]
pub enum FilterMode {
    Active,
    Completed,
    High,
    Today,
    All,
}

impl From<FilterMode> for SidebarTab {
    fn from(filter: FilterMode) -> Self {
        match filter {
            FilterMode::Active => SidebarTab::Active,
            FilterMode::Completed => SidebarTab::Completed,
            FilterMode::High => SidebarTab::HighPriority,
            FilterMode::All => SidebarTab::Inbox,
            FilterMode::Today => SidebarTab::Today,
        }
    }
}

/// Unit-tests for types
#[cfg(test)]
mod tests {
    use super::*;

    /// Helper method to setup task vector
    fn setup_tasks() -> Vec<Task> {
        let t1 = Task::new("Task 1".to_string());
        let t2 = Task::new("Task 2".to_string());
        let t3 = Task::new("Task 3".to_string());

        vec![t1, t2, t3]
    }

    #[test]
    fn should_handle_not_found_for_find_result() {
        let tasks: Vec<Task> = setup_tasks();
        let result = FindResult::find(&tasks, "nonexistent");
        assert!(matches!(result, FindResult::NotFound));
    }

    #[test]
    fn should_handle_exact_found_for_find_result() {
        let tasks = setup_tasks();
        let id_str = tasks[0].id.to_string();
        let query = &id_str[..8];

        let result = FindResult::find(&tasks, query);
        match result {
            FindResult::Found(id) => assert_eq!(id, tasks[0].id),
            _ => panic!("Expected Found, got {:?}", result),
        }
    }

    #[test]
    fn should_ambiguous_find_for_find_result() {
        let id = Uuid::new_v4();
        let mut t1 = Task::new("Alpha".to_string());
        let mut t2 = Task::new("Apple".to_string());

        t1.id = id;
        t2.id = id;
        let tasks = vec![t1, t2];
        let result = FindResult::find(&tasks, "");

        assert!(matches!(result, FindResult::Ambiguous(_)));
    }
}
