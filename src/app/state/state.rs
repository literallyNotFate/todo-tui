use super::error::ApplicationStateError;
use crate::app::{
    models::todo::Todo,
    utils::constants::text::{CLEARED_TASKS_TEXT, REMOVED_TASK_TEXT},
};
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
    pub fn append_todo(&mut self, new_title: impl Into<String>) -> ApplicationResult<String> {
        let title: String = new_title.into();

        if title.is_empty() {
            return Err(ApplicationStateError::EmptyTitle);
        }

        if self.todo_by_title(&title).is_some() {
            return Err(ApplicationStateError::TaskAlreadyExists(title));
        }

        self.todos.push(Todo::new(&title));
        self.select_state.select(Some(self.todos.len() - 1));

        Ok(format!("Task {} was added to the list!", title))
    }

    pub fn rename_todo(&mut self, new_title: impl Into<String>) -> ApplicationResult<String> {
        let new_title: String = new_title.into();

        if new_title.is_empty() {
            return Err(ApplicationStateError::EmptyTitle);
        }

        let index: usize = self
            .select_state
            .selected()
            .ok_or(ApplicationStateError::TaskNotSelected)?;

        let current_title: &String = &self.todos[index].title;

        if new_title != *current_title && self.todo_by_title(&new_title).is_some() {
            return Err(ApplicationStateError::TaskAlreadyExists(new_title));
        }

        self.todos[index].rename(&new_title);

        Ok(format!(
            "Task ({} / {}) was renamed to {}!",
            index,
            self.todos.len(),
            new_title
        ))
    }

    pub fn remove_todo(&mut self) -> ApplicationResult<String> {
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

        Ok(String::from(REMOVED_TASK_TEXT))
    }

    pub fn toggle_current(&mut self) {
        if let Some(index) = self.select_state.selected() {
            self.todos[index].toggle_done();
        }
    }

    pub fn clear_todos(&mut self) -> ApplicationResult<String> {
        if self.todos.is_empty() {
            return Err(ApplicationStateError::ListEmpty);
        }

        self.todos = Vec::new();
        Ok(String::from(CLEARED_TASKS_TEXT))
    }

    // Other actions
    pub fn current_todo(&self) -> Option<&Todo> {
        self.select_state.selected().and_then(|i| self.todos.get(i))
    }

    pub fn todo_by_title(&self, target: impl Into<String>) -> Option<&Todo> {
        let title: String = target.into();
        self.todos.iter().find(|todo| todo.title == title)
    }
}
