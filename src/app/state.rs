use super::models::todo::Todo;
use ratatui::widgets::ListState;

#[derive(Debug, Default)]
pub struct ApplicationState {
    pub todos: Vec<Todo>,
    pub select_state: ListState,
}

impl ApplicationState {
    pub fn new() -> Self {
        Self {
            todos: Vec::new(),
            select_state: ListState::default().with_selected(Some(0)),
        }
    }

    pub fn append_todo(&mut self, title: impl Into<String>) {
        self.todos.push(Todo::new(title));
        self.select_state.select(Some(self.todos.len()));
    }

    pub fn rename_todo(&mut self, new_title: impl Into<String>) {
        if let Some(index) = self.select_state.selected() {
            self.todos[index].rename(new_title);
        }
    }

    pub fn remove_todo(&mut self) {
        if let Some(index) = self.select_state.selected() {
            self.todos.remove(index);
        }
    }

    pub fn get_current_todo(&self) -> Todo {
        let index: usize = self.select_state.selected().unwrap_or(0);
        self.todos[index].clone()
    }

    pub fn toggle_current(&mut self) {
        if let Some(index) = self.select_state.selected() {
            self.todos[index].toggle_done();
        }
    }
}
