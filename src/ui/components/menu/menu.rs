use crate::{
    enums::ApplicationMode,
    state::{ApplicationState, UIState},
};
use ratatui::{Frame, layout::Rect};

pub struct Menu;

impl Menu {
    pub fn render(
        frame: &mut Frame,
        (left, content, bottom): (Rect, Rect, Rect),
        state: &ApplicationState,
        ui: &UIState,
        mode: &ApplicationMode,
    ) {
        use super::{bottom::MenuBottomBar, content::MenuContent, sidebar::MenuSidebar};

        MenuSidebar::render(frame, left, ui.current_filter, &state.todos, &ui.focus_area);
        MenuContent::render(frame, content, state, ui, &mode);
        MenuBottomBar::render(frame, bottom, state, &mode);
    }
}
