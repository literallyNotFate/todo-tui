use crate::{
    enums::ApplicationMode,
    state::{ApplicationState, UIState},
};
use ratatui::{Frame, layout::Rect};

pub struct MenuContent;

impl MenuContent {
    pub fn render(
        frame: &mut Frame,
        area: Rect,
        state: &ApplicationState,
        ui: &UIState,
        mode: &ApplicationMode,
    ) {
        use super::list::TaskList;

        match mode {
            ApplicationMode::Browsing => {
                let filtered = ui.current_filter.filter(&state.todos);
                TaskList::render(
                    frame,
                    area,
                    &filtered,
                    &state.select_state,
                    ui.current_filter,
                    &ui.focus_area,
                );
            }
            ApplicationMode::Task => {
                if let Some(form) = &ui.task_form {
                    form.render(frame, area);
                }
            }
        }
    }
}
