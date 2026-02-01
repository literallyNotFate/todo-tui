use crate::{
    enums::ApplicationMode,
    state::{ApplicationState, UIState},
    theme::ThemeColors,
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
        let theme_colors: ThemeColors = ui.theme.colors();

        MenuSidebar::render(frame, left, ui, &theme_colors, &state.todos, mode);
        MenuContent::render(frame, content, state, ui, &theme_colors, mode);
        MenuBottomBar::render(frame, bottom, state, &theme_colors);
    }
}
