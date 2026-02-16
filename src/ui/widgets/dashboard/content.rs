use crate::{
    core::ApplicationMode,
    models::Todo,
    state::{ApplicationState, UIState},
    theme::ThemeColors,
    ui::{FeedbackKind, FeedbackWidget, widgets::dashboard::list::ListTasks},
};
use ratatui::{Frame, layout::Rect};

/// Content widget (tasks table/form)
pub struct ContentWidget<'a> {
    state: &'a mut ApplicationState,
    ui: &'a UIState,
    mode: &'a ApplicationMode,
    theme: &'a ThemeColors,
}

impl<'a> ContentWidget<'a> {
    pub fn new(
        state: &'a mut ApplicationState,
        ui: &'a UIState,
        mode: &'a ApplicationMode,
        theme: &'a ThemeColors,
    ) -> Self {
        Self {
            state,
            ui,
            mode,
            theme,
        }
    }

    // Content rendering
    pub fn render(&mut self, frame: &mut Frame, area: Rect) {
        match self.mode {
            ApplicationMode::Form => {
                if let Some(form) = &self.ui.task_form {
                    form.render(frame, area, self.theme);
                }
            }
            _ => {
                let query: &str = &self.ui.search_query();
                let filtered: Vec<&Todo> = self.ui.current_filter.apply(&self.state.todos, query);

                if filtered.is_empty() && query.is_empty() {
                    frame.render_widget(
                        FeedbackWidget::new(FeedbackKind::EmptyList, self.theme),
                        area,
                    );
                    return;
                }

                ListTasks::new(self.ui, filtered, query, self.mode, self.theme).render(
                    frame,
                    area,
                    &mut self.state.select_state,
                    &self.ui.desc_scroll,
                );
            }
        }
    }
}
