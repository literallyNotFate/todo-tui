pub mod app_state;
pub mod scroll;
pub mod session;
pub mod ui_state;

pub use app_state::ApplicationState;
pub use scroll::AdaptiveScroll;
pub use session::Session;
pub use ui_state::{SidebarTab, UIState};

use crate::{
    core::ApplicationError,
    models::{Folder, Task},
    ui::widgets::modal::{Modal, ModalAction},
};
use serde::{Deserialize, Serialize};

/// Service response (data or TaskError/StorageError)
pub type ApplicationResult<T> = Result<T, ApplicationError>;

/// Active modal widget with modal itself and its action like save etc.
pub struct ActiveModal {
    pub modal: Box<dyn Modal>,
    pub action: ModalAction,
}

/// What is being deserialized from the database
#[derive(Deserialize, Default)]
pub struct TasksStateData {
    pub tasks: Vec<Task>,
    pub folders: Vec<Folder>,
    pub session: Session,
}

impl TasksStateData {
    pub fn new(tasks: Vec<Task>, folders: Vec<Folder>, session: Session) -> Self {
        Self {
            tasks,
            folders,
            session,
        }
    }
}

/// What is being serialized to the database
#[derive(Serialize)]
pub struct TasksStateSave<'a> {
    pub tasks: &'a [Task],
    pub folders: &'a [Folder],
    pub session: &'a Session,
}

impl<'a> TasksStateSave<'a> {
    pub fn new(tasks: &'a [Task], folders: &'a [Folder], session: &'a Session) -> Self {
        Self {
            tasks,
            folders,
            session,
        }
    }
}
