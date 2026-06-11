use crate::{
    models::{Task, TaskFilter},
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
        let query: &str = self.ui.search_query();
        let filter: TaskFilter = TaskFilter::new(self.ui.active_tab, self.ui.active_folder, &query);
        let filtered: Vec<&Task> = filter.apply(&self.state.tasks);

        if filtered.is_empty() && query.is_empty() {
            FeedbackWidget::new(FeedbackKind::EmptyList).render(ctx, area);
            return;
        }

        ListTasks::new(
            &self.ui,
            filtered,
            &self.state.folders,
            &query,
            &self.state.sort,
        )
        .render(ctx, area, &mut self.state.select_state);
    }
}
