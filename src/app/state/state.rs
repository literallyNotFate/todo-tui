use super::error::ApplicationStateError;
use crate::app::models::todo::Todo;
use ratatui::widgets::ListState;

#[derive(Debug, Default)]
pub struct ApplicationState {
    pub todos: Vec<Todo>,
    pub select_state: ListState,
}

pub type ApplicationResult<T> = Result<T, ApplicationStateError>;

impl ApplicationState {
    pub fn new() -> Self {
        Self {
            todos: Vec::new(),
            select_state: ListState::default().with_selected(Some(0)),
        }
    }

    // Main service todo
    pub fn append_todo(&mut self, new_title: impl Into<String>) -> ApplicationResult<()> {
        let title: String = new_title.into();

        if title.is_empty() {
            return Err(ApplicationStateError::EmptyTitle);
        }

        self.todos.push(Todo::new(title));
        self.select_state.select(Some(self.todos.len() - 1));

        Ok(())
    }

    pub fn rename_todo(&mut self, new_title: impl Into<String>) -> ApplicationResult<()> {
        let title: String = new_title.into();

        if title.is_empty() {
            return Err(ApplicationStateError::EmptyTitle);
        }

        let index: usize = self
            .select_state
            .selected()
            .ok_or(ApplicationStateError::TaskNotSelected)?;

        self.todos[index].rename(title);
        Ok(())
    }

    pub fn remove_todo(&mut self) -> ApplicationResult<()> {
        if self.todos.is_empty() {
            return Err(ApplicationStateError::CannotRemoveFromEmpty);
        }

        let index: usize = self
            .select_state
            .selected()
            .ok_or(ApplicationStateError::TaskNotSelected)?;

        self.todos.remove(index);

        if self.todos.is_empty() {
            self.select_state.select(None);
        } else {
            let new_index = index.min(self.todos.len() - 1);
            self.select_state.select(Some(new_index));
        }

        Ok(())
    }

    pub fn toggle_current(&mut self) {
        if let Some(index) = self.select_state.selected() {
            self.todos[index].toggle_done();
        }
    }

    pub fn clear_todos(&mut self) -> ApplicationResult<()> {
        if self.todos.is_empty() {
            return Err(ApplicationStateError::ListEmpty);
        }

        self.todos = Vec::new();
        Ok(())
    }

    // Other actions
    pub fn current_todo(&self) -> Option<&Todo> {
        self.select_state.selected().and_then(|i| self.todos.get(i))
    }
}
