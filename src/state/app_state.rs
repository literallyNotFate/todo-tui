use crate::{
    core::{ApplicationError, Storage},
    models::{Filter, Priority, Todo},
    state::AdaptiveScroll,
    ui::Notification,
};
use chrono::Local;
use ratatui::widgets::TableState;
use std::{
    hash::{DefaultHasher, Hash, Hasher},
    path::Path,
};

/// Application state (related to data only)
#[derive(Debug, Default)]
pub struct ApplicationState {
    pub todos: Vec<Todo>,
    pub select_state: TableState,
    pub scroll: AdaptiveScroll,

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
            saved_todos_hash: 0,
            scroll: AdaptiveScroll::default(),
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
        self.scroll.reset();
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
}
