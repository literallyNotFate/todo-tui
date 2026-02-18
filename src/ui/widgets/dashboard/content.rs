use crate::{
    core::ApplicationMode,
    models::Todo,
    state::{ApplicationState, UIState},
    ui::{FeedbackKind, FeedbackWidget, RenderContext, widgets::dashboard::list::ListTasks},
};
use ratatui::layout::Rect;

/// Content widget (tasks table/form)
pub struct ContentWidget<'a> {
    state: &'a mut ApplicationState,
    ui: &'a UIState,
}

impl<'a> ContentWidget<'a> {
    pub fn new(state: &'a mut ApplicationState, ui: &'a UIState) -> Self {
        Self { state, ui }
    }

    /// Content rendering
    pub fn render(&mut self, ctx: &mut RenderContext, area: Rect) {
        match ctx.mode() {
            ApplicationMode::Form => {
                if let Some(form) = &self.ui.task_form {
                    form.render(ctx, area);
                }
            }
            _ => {
                let query: &str = &self.ui.search_query();
                let filtered: Vec<&Todo> = self.ui.current_filter.apply(&self.state.todos, query);

                if filtered.is_empty() && query.is_empty() {
                    FeedbackWidget::new(FeedbackKind::EmptyList).render(ctx, area);
                    return;
                }

                ListTasks::new(self.ui, filtered, query, &self.state.sort).render(
                    ctx,
                    area,
                    &mut self.state.select_state,
                    &self.ui.desc_scroll,
                );
            }
        }
    }
}
