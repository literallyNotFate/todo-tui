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
            ApplicationMode::Form => {
                if let Some(form) = &ui.task_form {
                    form.render(frame, area, theme);
                }
            }
            _ => {
                let query = ui.search_query();
                let filtered = ui.current_filter.apply(&state.todos, &query);

                TaskList::render(
                    frame,
                    area,
                    ui,
                    &mut state.select_state,
                    theme,
                    &filtered,
                    mode,
                );
            }
        }
    }
}
