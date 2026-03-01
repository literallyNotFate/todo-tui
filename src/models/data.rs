use super::{Filter, Todo};
use crate::enums::FocusArea;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
pub struct StorageData {
    pub todos: Vec<Todo>,
    pub ui_session: UISession,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UISession {
    pub last_selected_id: Option<Uuid>,
    pub last_focus: FocusArea,
    pub last_filter: Filter,
    pub last_query: String,
    pub description_scroll_pos: u16,
    pub hotkeys_scroll_pos: u16,
    pub use_system_theme: bool,
}

impl Default for UISession {
    fn default() -> Self {
        Self {
            last_selected_id: None,
            last_focus: FocusArea::default(),
            last_filter: Filter::default(),
            last_query: String::new(),
            description_scroll_pos: 0,
            hotkeys_scroll_pos: 0,
            use_system_theme: true,
        }
    }
}

impl Default for StorageData {
    fn default() -> Self {
        Self {
            ui_session: UISession::default(),
            todos: Vec::new(),
        }
    }
}

impl StorageData {
    pub fn new(todos: Vec<Todo>, ui_session: UISession) -> Self {
        Self { todos, ui_session }
    }
}
