pub mod confirm;
pub mod popup;

pub use confirm::Confirm;
pub use popup::Popup;

use crate::{core::Action, models::Priority, theme::ThemeId, ui::RenderContext};
use ratatui::{crossterm::event::KeyEvent, layout::Rect};
use uuid::Uuid;

/// Modal trait for interactable widgets such as popup and confirm
pub trait Modal {
    fn area(&self, frame_area: Rect) -> Rect;
    fn render(&self, ctx: &mut RenderContext, area: Rect);
    fn handle_action(&mut self, action: Option<Action>, event: &KeyEvent) -> Option<ModalResult>;
}

/// Returns the result of the confirm operation
#[derive(Debug, PartialEq)]
pub enum ModalResult {
    Confirmed,
    Cancelled,
    TaskSubmitted {
        id: Option<Uuid>,
        title: String,
        description: String,
        priority: Priority,
    },
    FolderSubmitted {
        id: Option<Uuid>,
        name: String,
        color: String,
    },
    Changed {
        theme_id: ThemeId,
    },
}

/// Actions that being performed after confirmation
#[derive(Debug, Clone, PartialEq)]
pub enum ModalAction {
    None,
    Remove,
    Clear,
    Save,
    UnsavedExit,

    RemoveFolder(Uuid),
}

/// Modal size
#[derive(Debug, Clone, Copy)]
pub enum ModalSize {
    Small,
    Medium,
    Large,
    Custom { width: u16, height: u16 },
}

impl ModalSize {
    pub fn percentages(&self) -> (u16, u16) {
        match self {
            ModalSize::Small => (40, 30),
            ModalSize::Medium => (60, 50),
            ModalSize::Large => (90, 80),
            ModalSize::Custom { width, height } => (*width, *height),
        }
    }
}
