use crate::{
    core::Sort,
    models::{Filter, Priority, Task},
    ui::Notification,
};
use chrono::Local;
use ratatui::widgets::TableState;
use std::{
    cell::Cell,
    hash::{DefaultHasher, Hash, Hasher},
};
use uuid::Uuid;

/// Application state (related to data only)
#[derive(Debug, Default)]
pub struct ApplicationState {
    pub tasks: Vec<Task>,
    pub select_state: TableState,
    pub sort: Sort,

    pub notification: Option<Notification>,

    pub saved_hash: u64,
    current_hash: Cell<u64>,
    needs_rehash: Cell<bool>,
    is_unsaved_cache: Cell<bool>,

    pub last_selected_id: Option<Uuid>,
}

impl ApplicationState {
    pub fn new(mut tasks: Vec<Task>) -> Self {
        for task in &mut tasks {
            task.title_lower = task.title.to_lowercase();
        }

        let mut state: Self = Self {
            tasks: tasks,
            ..Self::default()
        };

        state.mark_saved();
        if let Some(last) = state.tasks.last() {
            state.select_state.select(Some(state.tasks.len() - 1));
            state.last_selected_id = Some(last.id);
        }

        log::debug!("ApplicationState initialized. Tasks: {}", state.tasks.len());
        state
    }

    /// Create default state (for testing usually)
    pub fn default() -> Self {
        Self {
            tasks: Vec::new(),
            select_state: TableState::default(),
            notification: None,
            sort: Sort::default(),
            saved_hash: 0,
            current_hash: Cell::new(0),
            needs_rehash: Cell::new(true),
            is_unsaved_cache: Cell::new(false),
            last_selected_id: None,
        }
    }

    /// Method that calculates current hash only if its needed so
    pub(crate) fn hash_state(&self) -> u64 {
        if self.needs_rehash.get() {
            let mut hasher: DefaultHasher = DefaultHasher::new();
            self.tasks.hash(&mut hasher);

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

    /// State sync: marks current hash as saved
    pub fn mark_saved(&mut self) {
        let current: u64 = self.hash_state();
        self.saved_hash = current;
        self.is_unsaved_cache.set(false);
        log::debug!("State marked as saved at hash: {:016X}", current);
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

    /// Return filtered tasks based on active filter selection
    pub(crate) fn filter(&self, filter: &Filter) -> impl Iterator<Item = (usize, &Task)> {
        let today = Local::now().date_naive();

        self.tasks
            .iter()
            .enumerate()
            .filter(move |(_, task)| match filter {
                Filter::All => true,
                Filter::Active => !task.completed,
                Filter::Completed => task.completed,
                Filter::HighPriority => task.priority == Priority::High,
                Filter::Today => task.created_at.with_timezone(&Local).date_naive() == today,
            })
    }

    /// Return indices of tasks to swap
    pub fn swap_indices(&self, filter: &Filter, query: &str, delta: i32) -> Option<(usize, usize)> {
        log::debug!(
            "Calculating swap: current_idx={:?}, delta={}",
            self.select_state.selected(),
            delta
        );

        let filtered: Vec<&Task> = filter.apply(&self.tasks, query);
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

        let index_a: usize = self.tasks.iter().position(|t| t.id == current_id)?;
        let index_b: usize = self.tasks.iter().position(|t| t.id == target_id)?;

        log::trace!(
            "Resolved global indices for swap: {} <-> {}",
            index_a,
            index_b
        );
        Some((index_a, index_b))
    }

    /// Helper function to select from visible amount of tasks
    pub fn clamp_selection(&mut self, visible_count: usize) {
        if visible_count == 0 {
            self.select_state.select(None);
            self.last_selected_id = None;
            return;
        }

        let new_idx = self
            .select_state
            .selected()
            .map(|idx| idx.min(visible_count.saturating_sub(1)))
            .unwrap_or(0);

        self.select_state.select(Some(new_idx));
    }

    /// Return currently selected task
    pub fn selected<'a>(
        &self,
        tasks: &'a [Task],
        filter: &Filter,
        query: &str,
    ) -> Option<&'a Task> {
        let filtered = filter.apply(tasks, query);
        let index = self.select_state.selected()?;
        filtered.get(index).copied()
    }

    /// Return id of current selected task
    pub fn selected_id(&self, tasks: &[Task], filter: &Filter, query: &str) -> Option<Uuid> {
        self.selected(tasks, filter, query).map(|t| t.id)
    }

    /// Return task of a given id
    pub fn find_by_id(&self, id: Uuid) -> Option<&Task> {
        self.tasks.iter().find(|t| t.id == id)
    }

    /// Sync focus with current state
    pub fn sync_with_ids(&mut self, visible_ids: &[Uuid], focus_id: Option<Uuid>) {
        let len: usize = visible_ids.len();

        if let Some(id) = focus_id {
            if let Some(pos) = visible_ids.iter().position(|&i| i == id) {
                self.select_state.select(Some(pos));
                self.last_selected_id = Some(id);
                return;
            }
        }

        if len == 0 {
            self.select_state.select(None);
            self.last_selected_id = None;
        } else {
            let current_idx = self.select_state.selected().unwrap_or(0);
            let new_idx = current_idx.min(len.saturating_sub(1));

            self.select_state.select(Some(new_idx));
            self.last_selected_id = visible_ids.get(new_idx).copied();
        }
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

        state.tasks.push(Task::new("Task", "", None));
        state.mark_as_dirty();

        assert!(
            state.any_unsaved_changes(),
            "Hash should be changed after append"
        );

        state.saved_hash = state.hash_state();
        state.is_unsaved_cache.set(false);
        assert!(!state.any_unsaved_changes());

        state.tasks[0].title = "Changed".to_string();
        state.mark_as_dirty();

        assert!(
            state.any_unsaved_changes(),
            "Hash should be changed after field edit"
        );

        state.tasks[0].title = "Task".to_string();
        state.mark_as_dirty();
        assert!(
            !state.any_unsaved_changes(),
            "Should be saved when reverted back"
        );
    }

    #[test]
    fn should_return_swap_indices_for_move() {
        let mut state = ApplicationState::default();
        state.tasks = vec![
            Task::new("Task 1").with_priority(Priority::High),
            Task::new("Task 2").with_priority(Priority::Low),
            Task::new("Task 3").with_priority(Priority::High),
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
        state.tasks = vec![Task::new("1")];
        state.select_state.select(Some(0));

        let up = state.swap_indices(&Filter::All, "", -1);
        assert_eq!(up, None, "Should not move above 0");

        let down = state.swap_indices(&Filter::All, "", 1);
        assert_eq!(down, None, "Should not move below last element");
    }

    #[test]
    fn should_return_currently_selected_task() {
        let mut state = ApplicationState::default();
        state.tasks = vec![Task::new("A"), Task::new("B")];

        state.select_state.select(Some(1));
        let id: Uuid = state.tasks[1].id;
        let selected: Option<&Task> = state.selected(&state.tasks, &Filter::All, "");
        let selected_id: Option<Uuid> = state.selected_id(&state.tasks, &Filter::All, "");

        assert_eq!(selected.unwrap().title, "B");
        assert_eq!(selected_id.unwrap(), id);
    }

    #[test]
    fn should_return_task_by_id() {
        let mut state = ApplicationState::default();
        state.tasks = vec![Task::new("A"), Task::new("B")];
        let id_to_find: Uuid = state.tasks[1].id;
        let selected: Option<&Task> = state.find_by_id(id_to_find);

        assert_eq!(selected.unwrap().title, "B");
        assert_eq!(selected.unwrap().id, id_to_find);
    }
}
