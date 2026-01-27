use crate::{
    enums::ApplicationMode,
    state::{ApplicationState, UIState},
    theme::ThemeColors,
};
use ratatui::{Frame, layout::Rect};

pub struct MenuContent;

impl MenuContent {
    pub fn render(
        frame: &mut Frame,
        area: Rect,
        state: &mut ApplicationState,
        ui: &UIState,
        theme: &ThemeColors,
        mode: &ApplicationMode,
    ) {
        use super::list::TaskList;

        match mode {
            ApplicationMode::Browsing => {
                let filtered = ui.current_filter.filter(&state.todos);
                TaskList::render(frame, area, ui, &mut state.select_state, theme, &filtered);
            }
            ApplicationMode::Task => {
                if let Some(form) = &ui.task_form {
                    form.render(frame, area, theme);
                }
            }
        }
    }
}
