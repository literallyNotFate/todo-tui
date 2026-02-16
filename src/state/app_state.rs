use crate::{
    core::{ApplicationError, Storage},
    models::{Filter, Priority, Sort, Todo},
    ui::Notification,
};
use chrono::Local;
use ratatui::widgets::TableState;
use std::{
    hash::{DefaultHasher, Hash, Hasher},
    path::Path,
};
use uuid::Uuid;

/// Application state (related to data only)
#[derive(Debug, Default)]
pub struct ApplicationState {
    pub todos: Vec<Todo>,
    pub select_state: TableState,
    pub sort: Sort,

    pub notification: Option<Notification>,
    pub saved_todos_hash: u64,
}

/// Service response (data or TodoError/StorageError)
pub type ApplicationResult<T> = Result<T, ApplicationError>;

impl ApplicationState {
    pub fn new() -> Self {
        let mut state = Self::load(None).unwrap_or_default();
        if state.todos.is_empty() {
            let _ = state.save(None);
        }

        state
    }

    /// Create default state (for testing usually)
    pub fn default() -> Self {
        Self {
            todos: Vec::new(),
            select_state: TableState::default(),
            notification: None,
            sort: Sort::default(),
            saved_todos_hash: 0,
        }
    }

    /// Get todos hash to compare to current (to track unsaved changes)
    pub(crate) fn hash_state(&self) -> u64 {
        let mut hasher: DefaultHasher = DefaultHasher::new();
        self.todos.hash(&mut hasher);
        hasher.finish()
    }

    /// Check if there any unsaved changes by comparing hash
    pub fn any_unsaved_changes(&self) -> bool {
        self.hash_state() != self.saved_todos_hash
    }

    /// Navigate through tasks
    pub fn move_selection(&mut self, delta: i32, displayed_count: usize) {
        if displayed_count == 0 {
            self.select_state.select(None);
            return;
        }

        let current: usize = self.select_state.selected().unwrap_or(0);
        let next: usize = if delta > 0 {
            (current + 1) % displayed_count
        } else {
            (current + displayed_count - 1) % displayed_count
        };

        self.select_state.select(Some(next));
    }

    /// Save todos to a file
    pub fn save(&mut self, path: Option<&Path>) -> ApplicationResult<String> {
        Storage::save(&self.todos, path)?;
        self.saved_todos_hash = self.hash_state();
        Ok("Tasks were saved!".to_string())
    }

    /// Load todos from a file
    pub fn load(path: Option<&Path>) -> ApplicationResult<Self> {
        let todos: Vec<Todo> = Storage::load(path)?;
        let mut state: ApplicationState = Self {
            todos,
            ..Self::default()
        };

        state.saved_todos_hash = state.hash_state();
        if state.todos.is_empty() {
            state.select_state.select(None);
        } else {
            state.select_state.select_last();
        }

        Ok(state)
    }

    /// Return filtered tasks based on active filter selection
    pub(crate) fn filter(&self, filter: &Filter) -> impl Iterator<Item = (usize, &Todo)> {
        let today = Local::now().date_naive();

        self.todos
            .iter()
            .enumerate()
            .filter(move |(_, todo)| match filter {
                Filter::All => true,
                Filter::Active => !todo.completed,
                Filter::Completed => todo.completed,
                Filter::HighPriority => todo.priority == Priority::High,
                Filter::Today => todo.created_at.with_timezone(&Local).date_naive() == today,
            })
    }

    /// Return indices of tasks to swap
    pub fn swap_indices(&self, filter: &Filter, query: &str, delta: i32) -> Option<(usize, usize)> {
        let filtered: Vec<&Todo> = filter.apply(&self.todos, query);
        let current_index: usize = self.select_state.selected()?;

        let target_index: usize = if delta > 0 {
            current_index.checked_add(delta.unsigned_abs() as usize)?
        } else {
            current_index.checked_sub(delta.unsigned_abs() as usize)?
        };

        if target_index >= filtered.len() {
            return None;
        }

        let current_id: Uuid = filtered[current_index].id;
        let target_id: Uuid = filtered[target_index].id;

        let index_a: usize = self.todos.iter().position(|t| t.id == current_id)?;
        let index_b: usize = self.todos.iter().position(|t| t.id == target_id)?;

        Some((index_a, index_b))
    }

    /// Return currently selected todo
    pub fn selected<'a>(
        &self,
        todos: &'a [Todo],
        filter: &Filter,
        query: &str,
    ) -> Option<&'a Todo> {
        let filtered = filter.apply(todos, query);
        let index = self.select_state.selected()?;
        filtered.get(index).copied()
    }
}

/// Unit-tests for ApplicationState
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_determine_unsaved_changes() {
        let mut state = ApplicationState::default();
        state.saved_todos_hash = state.hash_state();
        assert!(!state.any_unsaved_changes());

        state.todos.push(Todo::new("Task", "", None));
        assert!(
            state.any_unsaved_changes(),
            "Hash should be changed after append"
        );

        state.saved_todos_hash = state.hash_state();
        assert!(!state.any_unsaved_changes());

        state.todos[0].title = "Changed".to_string();
        assert!(
            state.any_unsaved_changes(),
            "Hash should be changed after field edit"
        );
    }

    #[test]
    fn should_return_swap_indices_for_move() {
        let mut state = ApplicationState::default();
        state.todos = vec![
            Todo::new("Task 1", "", Some(Priority::High)),
            Todo::new("Task 2", "", Some(Priority::Low)),
            Todo::new("Task 3", "", Some(Priority::High)),
        ];

        let filter: Filter = Filter::HighPriority;
        let query: &str = "";

        state.select_state.select(Some(0));
        let indices: Option<(usize, usize)> = state.swap_indices(&filter, query, 1);

        assert_eq!(indices, Some((0, 2)));
    }

    #[test]
    fn should_test_swap_indices_boundaries() {
        let mut state = ApplicationState::default();
        state.todos = vec![Todo::new("1", "", None)];
        state.select_state.select(Some(0));

        let up = state.swap_indices(&Filter::All, "", -1);
        assert_eq!(up, None, "Should not move above 0");

        let down = state.swap_indices(&Filter::All, "", 1);
        assert_eq!(down, None, "Should not move below last element");
    }

    #[test]
    fn should_return_currently_selected_todo() {
        let mut state = ApplicationState::default();
        state.todos = vec![
            Todo::new("A", "", Some(Priority::Low)),
            Todo::new("B", "", Some(Priority::Low)),
        ];
        state.select_state.select(Some(1));

        let selected: Option<&Todo> = state.selected(&state.todos, &Filter::All, "");
        assert_eq!(selected.unwrap().title, "B");
    }
}
