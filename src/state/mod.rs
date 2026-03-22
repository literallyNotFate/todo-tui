pub mod app_state;
pub mod scroll;
pub mod ui_state;

pub use app_state::ApplicationState;
pub use scroll::AdaptiveScroll;
use serde::{Deserialize, Serialize};
pub use ui_state::UIState;

use crate::{
    common::default_bool_is_true,
    core::{ApplicationError, FocusArea, Selectable},
    models::{Filter, Todo},
    ui::{
        TextInput,
        widgets::modal::{Modal, ModalAction},
    },
};

/// Service response (data or TodoError/StorageError)
pub type ApplicationResult<T> = Result<T, ApplicationError>;

/// Active modal widget with modal itself and its action like save etc.
pub struct ActiveModal {
    pub modal: Box<dyn Modal>,
    pub action: ModalAction,
}

/// What is being stored in todos.json (data with saved UI state)
#[derive(Serialize, Deserialize, Default)]
pub struct TasksStateData {
    pub todos: Vec<Todo>,
    pub session: Session,
}

impl TasksStateData {
    pub fn new(todos: Vec<Todo>, session: Session) -> Self {
        Self { todos, session }
    }
}

/// Session (current UI state) to save to file/load from
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Session {
    pub last_selected_id: Option<uuid::Uuid>,
    pub last_focus: Selectable<FocusArea>,
    pub last_filter: Selectable<Filter>,
    pub last_query: String,
    pub description_scroll_pos: u16,
    pub hotkeys_scroll_pos: u16,
    #[serde(default = "default_bool_is_true")]
    pub use_system_theme: bool,
}

impl Session {
    /// Creates session based on current state of the UI
    pub fn from_state(ui: &UIState, selected_id: Option<uuid::Uuid>) -> Self {
        Self {
            last_selected_id: selected_id,
            last_focus: ui.focused.clone(),
            last_filter: ui.filter.clone(),
            last_query: ui
                .search_input
                .as_ref()
                .map(|i| i.buffer.clone())
                .unwrap_or_default(),
            use_system_theme: ui.config.use_system_theme,
            description_scroll_pos: ui.desc_scroll.current.get(),
            hotkeys_scroll_pos: ui.hotkeys_scroll.current.get(),
        }
    }

    /// Modifies UI state using current session data
    pub fn apply_to(&self, ui: &mut UIState) {
        ui.filter = self.last_filter.clone();
        ui.focused = self.last_focus.clone();
        ui.desc_scroll = AdaptiveScroll::with_position(self.description_scroll_pos);
        ui.hotkeys_scroll = AdaptiveScroll::with_position(self.hotkeys_scroll_pos);
        ui.config.use_system_theme = self.use_system_theme;

        ui.refresh_theme();

        ui.search_input =
            (!self.last_query.is_empty()).then(|| TextInput::from(self.last_query.clone()));
    }
}
