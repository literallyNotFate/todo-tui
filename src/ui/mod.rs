pub mod components;
pub mod context;
pub mod renderer;
pub mod widgets;

pub use components::*;
pub use context::RenderContext;
pub use renderer::Renderer;
pub use widgets::{
    Confirm, Dashboard, EnumInput, FeedbackKind, FeedbackWidget, Field, FieldType, Form,
    Notification, Popup, TextInput,
};

/// Min possible terminal size values
pub const MIN_WIDTH: u16 = 80;
pub const MIN_HEIGHT: u16 = 24;

/// Check if terminal/frame area is small
pub fn is_terminal_small(width: u16, height: u16) -> bool {
    width < MIN_WIDTH || height < MIN_HEIGHT
}

/// What is being returned from handle_key() widget function
#[derive(Debug, PartialEq)]
pub enum WidgetResponse {
    Continue,
    Submit,
    Cancel,
}

use ratatui::layout::{Constraint, Direction, Layout, Rect};

/// Main layout struct for convenience
pub struct MainLayout {
    pub sidebar: Rect,
    pub content: Rect,
    pub footer: Rect,
}

impl MainLayout {
    /// Splits area into layouts for dashboard w/sidebar config toggling
    pub fn split(area: Rect, show_sidebar: bool) -> Self {
        let [upper_area, footer] = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(0), Constraint::Length(2)])
            .areas(area);

        if show_sidebar {
            let [sidebar, main] = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(25), Constraint::Percentage(75)])
                .areas(upper_area);
            Self {
                sidebar,
                content: main,
                footer,
            }
        } else {
            Self {
                sidebar: Rect::default(),
                content: upper_area,
                footer,
            }
        }
    }
}

/// Center a widget in a rect
pub fn center(rect: Rect, width_pct: u16, height_pct: u16) -> Rect {
    let vertical = Layout::vertical([
        Constraint::Percentage((100 - height_pct) / 2),
        Constraint::Percentage(height_pct),
        Constraint::Percentage((100 - height_pct) / 2),
    ])
    .split(rect);

    Layout::horizontal([
        Constraint::Percentage((100 - width_pct) / 2),
        Constraint::Percentage(width_pct),
        Constraint::Percentage((100 - width_pct) / 2),
    ])
    .split(vertical[1])[1]
}
