use crate::{enums::WidgetResponse, theme::ThemePalette, ui::RenderContext};
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
            ModalSize::Small => (30, 20),
            ModalSize::Medium => (50, 40),
            ModalSize::Large => (85, 80),
            ModalSize::Custom { width, height } => (*width, *height),
        }
    }
}

/// Trait made for EnumInput field, so you are able to switch enum value (in that case Priority/Filter)
pub trait InteractableEnum: Sized + Copy + PartialEq + 'static {
    fn all() -> &'static [Self];
    fn to_string(&self) -> &'static str;

    fn index(&self) -> usize {
        Self::all().iter().position(|t| t == self).unwrap_or(0)
    }

    fn next(&self) -> Self {
        let variants = Self::all();
        let idx = self.index();
        variants[(idx + 1) % variants.len()]
    }

    fn prev(&self) -> Self {
        let variants = Self::all();
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
    fn on_focused(&self, focused: bool, palette: &ThemePalette) -> (Style, Style);
}
