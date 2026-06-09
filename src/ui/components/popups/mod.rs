pub mod details;
pub mod form;
pub mod help;
pub mod message;

pub use details::DetailsComponent;
pub use help::HelpComponent;
pub use message::MessageComponent;

use crate::{
    state::AdaptiveScroll,
    ui::{RenderContext, WidgetResponse, widgets::modal::ModalResult},
};
use ratatui::{crossterm::event::KeyEvent, layout::Rect};

/// Popup component main trait
pub trait PopupComponent {
    fn render(&self, ctx: &mut RenderContext, area: Rect);
    fn handle_key(&mut self, _event: &KeyEvent) -> WidgetResponse {
        WidgetResponse::Continue
    }

    fn is_scrollable(&self) -> bool {
        false
    }

    fn scroll_down(&self) {}
    fn scroll_up(&self) {}
    fn set_scroll(&mut self, _scroll: AdaptiveScroll) {}

    fn to_modal_result(&self) -> ModalResult {
        ModalResult::Cancelled
    }
}
