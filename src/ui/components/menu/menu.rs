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
        state: &mut ApplicationState,
        ui: &UIState,
        mode: &ApplicationMode,
    ) {
        use super::{bottom::MenuBottomBar, content::MenuContent, sidebar::MenuSidebar};

        MenuSidebar::render(frame, left, ui, &state.todos);
        MenuContent::render(frame, content, state, ui, &mode);
        MenuBottomBar::render(frame, bottom, state, &mode);
    }
}
