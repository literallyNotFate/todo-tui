pub mod enum_input;
pub mod select_input;
pub mod text_input;

pub use enum_input::EnumInput;
pub use select_input::SelectInput;
pub use text_input::TextInput;

use crate::{
    theme::ThemePalette,
    ui::{RenderContext, WidgetResponse},
};
use ratatui::{crossterm::event::KeyCode, layout::Rect, style::Style};

/// Trait for input widgets (TextInput/EnumInput<T>)
pub trait Input {
    fn title(self, title: impl Into<String>) -> Self;
    fn handle_key(&mut self, key: &KeyCode) -> WidgetResponse;
    fn reset(&mut self);
    fn render(&self, ctx: &mut RenderContext, area: Rect, focused: bool);
    fn on_focused(&self, focused: bool, palette: &ThemePalette) -> (Style, Style);
}
