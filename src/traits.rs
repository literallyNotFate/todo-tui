use crate::{enums::WidgetResponse, theme::ThemeColors, ui::RenderContext};
use ratatui::{crossterm::event::KeyCode, layout::Rect, style::Style};

/// Modal trait for interactable widgets such as popup and confirm
pub trait Modal {
    fn area(&self, frame_area: Rect) -> Rect;
    fn render(&self, ctx: &mut RenderContext, area: Rect);
    fn handle_key(&mut self, key: KeyCode) -> Option<ModalResult>;
}

#[derive(Debug, PartialEq)]
pub enum ModalResult {
    Confirmed,
    Cancelled,
}

/// Actions that being performed after confirmation
#[derive(Debug, Clone, PartialEq)]
pub enum ModalAction {
    None,
    Remove,
    Clear,
    Save,
    UnsavedExit,
}

/// Trait made for EnumInput field, so you are able to switch enum value (in that case Priority/Filter)
pub trait InteractableEnum: Sized + Copy + PartialEq + 'static {
    fn all_variants() -> &'static [Self];
    fn to_string(&self) -> &'static str;

    fn index(&self) -> usize {
        Self::all_variants()
            .iter()
            .position(|t| t == self)
            .unwrap_or(0)
    }

    fn next(&self) -> Self {
        let variants = Self::all_variants();
        let idx = self.index();
        variants[(idx + 1) % variants.len()]
    }

    fn prev(&self) -> Self {
        let variants = Self::all_variants();
        let idx = self.index();
        variants[(idx + variants.len() - 1) % variants.len()]
    }
}

/// Trait for input widgets (TextInput/EnumInput<T>)
pub trait Input {
    fn title(self, title: impl Into<String>) -> Self;
    fn handle_key(&mut self, key: &KeyCode) -> WidgetResponse;
    fn reset(&mut self);
    fn render(&self, ctx: &mut RenderContext, area: Rect, focused: bool);
    fn on_focused(&self, focused: bool, theme: &ThemeColors) -> (Style, Style);
}
