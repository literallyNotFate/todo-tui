use crate::{
    models::{Folder, Task},
    state::SidebarTab,
};
use uuid::Uuid;

/// Trait to define common properties for entities that can be searched via CLI
pub trait Identifiable {
    fn id(&self) -> Uuid;
    fn display_name(&self) -> String;
}

impl Identifiable for Task {
    fn id(&self) -> Uuid {
        self.id
    }
    fn display_name(&self) -> String {
        self.title.clone()
    }
}

impl Identifiable for Folder {
    fn id(&self) -> Uuid {
        self.id
    }
    fn display_name(&self) -> String {
        self.name.clone()
    }
}

/// Generic container for search results
#[derive(Debug)]
pub enum EntitySelector<T> {
    Selected(T),
    Ambiguous(Vec<(Uuid, String)>),
    NotFound,
}
impl<T: Identifiable + Clone> EntitySelector<T> {
    /// Executes an action if the entity was successfully selected.
    /// Prints error messages automatically for NotFound or Ambiguous states
    pub fn execute<F>(self, id_query: &str, mut action: F) -> color_eyre::Result<()>
    where
        F: FnMut(T) -> color_eyre::Result<()>,
    {
        match self {
            EntitySelector::NotFound => {
                eprintln!("Entity with ID starting with '{}' not found", id_query);
            }
            EntitySelector::Ambiguous(list) => {
                eprintln!("ID '{}' is ambiguous. Matches:", id_query);
                list.iter()
                    .for_each(|(id, title)| println!("  {} - {}", id, title));
            }
            EntitySelector::Selected(item) => {
                action(item)?;
            }
        }
        Ok(())
    }

    /// Finds an entity in a slice by a prefix of its UUID
    pub fn find(items: &[T], id_query: &str) -> Self {
        let matches: Vec<(Uuid, String)> = items
            .iter()
            .filter(|i| i.id().to_string().starts_with(id_query))
            .map(|i| (i.id(), i.display_name()))
            .collect();

        match matches.len() {
            0 => Self::NotFound,
            1 => {
                let item = items
                    .iter()
                    .find(|i| i.id() == matches[0].0)
                    .unwrap()
                    .clone();
                Self::Selected(item)
            }
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
        let result = EntitySelector::find(&tasks, "nonexistent");
        assert!(matches!(result, EntitySelector::NotFound));
    }

    #[test]
    fn should_handle_exact_found_for_find_result() {
        let tasks = setup_tasks();
        let id_str = tasks[0].id.to_string();
        let query = &id_str[..8];

        let result = EntitySelector::find(&tasks, query);
        match result {
            EntitySelector::Selected(task) => assert_eq!(task.id, tasks[0].id),
            _ => panic!("Expected Selected, got {:?}", result),
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
        let result = EntitySelector::find(&tasks, "");

        assert!(matches!(result, EntitySelector::Ambiguous(_)));
    }
}
