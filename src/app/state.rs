use super::models::todo::Todo;
use ratatui::widgets::ListState;

#[derive(Debug, Default)]
pub struct ApplicationState {
    pub todos: Vec<Todo>,
    pub select_state: ListState,
}

impl ApplicationState {
    pub fn new(todos: Vec<Todo>) -> Self {
        let mut select_state = ListState::default();
        select_state.select(Some(0));

        Self {
            todos,
            select_state,
        }
    }
}
