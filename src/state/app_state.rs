use crate::{
    config::StorageConfig,
    core::{ApplicationError, Storage},
    models::{Filter, Priority, Sort, Todo},
    ui::Notification,
};
use chrono::Local;
use ratatui::widgets::TableState;
use std::{
    cell::Cell,
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

    pub saved_hash: u64,
    current_hash: Cell<u64>,
    needs_rehash: Cell<bool>,
    is_unsaved_cache: Cell<bool>,
}

/// Service response (data or TodoError/StorageError)
pub type ApplicationResult<T> = Result<T, ApplicationError>;

impl ApplicationState {
    pub fn new(config: &StorageConfig) -> Self {
        let mut state = Self::load(None, config).unwrap_or_default();
        if state.todos.is_empty() {
            let _ = state.save(None, config);
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
            saved_hash: 0,
            current_hash: Cell::new(0),
            needs_rehash: Cell::new(true),
            is_unsaved_cache: Cell::new(false),
        }
    }

    /// Method that calculates current hash only if its needed so
    pub(crate) fn hash_state(&self) -> u64 {
        if self.needs_rehash.get() {
            let mut hasher: DefaultHasher = DefaultHasher::new();
            self.todos.hash(&mut hasher);

            let new_hash: u64 = hasher.finish();
            let old_hash: u64 = self.current_hash.get();
            log::trace!(
                "Rehashing state: old={:016X}, new={:016X}",
                old_hash,
                new_hash
            );

            self.current_hash.set(new_hash);
            self.is_unsaved_cache.set(new_hash != self.saved_hash);
            self.needs_rehash.set(false);
        }

        self.current_hash.get()
    }

    /// Check if there any unsaved changes by comparing hash
    pub fn any_unsaved_changes(&self) -> bool {
        self.hash_state();
        self.is_unsaved_cache.get()
    }

    /// Marks state as dirty (to be called in dispatch operations)
    pub fn mark_as_dirty(&self) {
        log::trace!("State marked as dirty (needs rehash)");
        self.needs_rehash.set(true);
    }

    /// Navigate through tasks
    pub fn move_selection(&mut self, delta: i32, displayed_count: usize, wrap: bool) {
        if displayed_count == 0 {
            self.select_state.select(None);
            return;
        }

        let current: usize = self.select_state.selected().unwrap_or(0);

        let next: usize = if delta > 0 {
            if current >= displayed_count - 1 {
                if wrap { 0 } else { current }
            } else {
                current + 1
            }
        } else {
            if current == 0 {
                if wrap { displayed_count - 1 } else { 0 }
            } else {
                current - 1
            }
        };

        log::trace!(
            "Selection move: {:?} -> Some({}) (total visible: {})",
            current,
            next,
            displayed_count
        );
        self.select_state.select(Some(next));
    }

    /// Save todos to a file
    pub fn save(
        &mut self,
        path: Option<&Path>,
        config: &StorageConfig,
    ) -> ApplicationResult<String> {
        Storage::save(&self.todos, path, config)?;
        self.saved_hash = self.hash_state();
        self.is_unsaved_cache.set(false);

        log::info!(
            "Data state synchronized. Saved hash: {:016X}",
            self.saved_hash
        );
        Ok("Tasks were saved!".to_string())
    }

    /// Load todos from a file
    pub fn load(path: Option<&Path>, config: &StorageConfig) -> ApplicationResult<Self> {
        log::debug!("Loading application state...");
        let todos: Vec<Todo> = Storage::load(path, config)?;
        let mut state: ApplicationState = Self {
            todos,
            ..Self::default()
        };

        let current: u64 = state.hash_state();
        state.saved_hash = current;
        state.is_unsaved_cache.set(false);

        if state.todos.is_empty() {
            state.select_state.select(None);
        } else {
            state.select_state.select_last();
        }

        log::info!(
            "State loaded successfully. Tasks count: {}",
            state.todos.len()
        );
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
        log::debug!(
            "Calculating swap: current_idx={:?}, delta={}",
            self.select_state.selected(),
            delta
        );

        let filtered: Vec<&Todo> = filter.apply(&self.todos, query);
        let current_index: usize = self.select_state.selected()?;

        let target_index: usize = if delta > 0 {
            current_index.checked_add(delta.unsigned_abs() as usize)?
        } else {
            current_index.checked_sub(delta.unsigned_abs() as usize)?
        };

        if target_index >= filtered.len() {
            log::trace!(
                "Swap cancelled: target_index {} out of filtered bounds",
                target_index
            );
            return None;
        }

        let current_id: Uuid = filtered[current_index].id;
        let target_id: Uuid = filtered[target_index].id;

        let index_a: usize = self.todos.iter().position(|t| t.id == current_id)?;
        let index_b: usize = self.todos.iter().position(|t| t.id == target_id)?;

        log::trace!(
            "Resolved global indices for swap: {} <-> {}",
            index_a,
            index_b
        );
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

        state.saved_hash = state.hash_state();
        state.is_unsaved_cache.set(false);
        assert!(!state.any_unsaved_changes());

        state.todos.push(Todo::new("Task", "", None));
        state.mark_as_dirty();

        assert!(
            state.any_unsaved_changes(),
            "Hash should be changed after append"
        );

        state.saved_hash = state.hash_state();
        state.is_unsaved_cache.set(false);
        assert!(!state.any_unsaved_changes());

        state.todos[0].title = "Changed".to_string();
        state.mark_as_dirty();

        assert!(
            state.any_unsaved_changes(),
            "Hash should be changed after field edit"
        );

        state.todos[0].title = "Task".to_string();
        state.mark_as_dirty();
        assert!(
            !state.any_unsaved_changes(),
            "Should be saved when reverted back"
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
