use crate::{
    core::Autosave,
    state::{ApplicationState, UIState},
    ui::RenderContext,
};
use ratatui::layout::Rect;

/// Main menu widget (combines sidebar, content, bottom)
pub struct Dashboard<'a> {
    pub state: &'a mut ApplicationState,
    pub ui: &'a UIState,
    autosave: &'a Autosave,
}

impl<'a> Dashboard<'a> {
    pub fn new(state: &'a mut ApplicationState, ui: &'a UIState, autosave: &'a Autosave) -> Self {
        Self {
            state,
            ui,
            autosave,
        }
    }

    /// Dashboard rendering
    pub fn render(self, ctx: &mut RenderContext, area: Rect) {
        use crate::ui::{
            main_layout,
            widgets::dashboard::{
                bottom::BottomBarWidget, content::ContentWidget, sidebar::SidebarWidget,
            },
        };

        let (left_area, content_area, bottom_area): (Rect, Rect, Rect) = main_layout(area);

        SidebarWidget::new(self.ui, &self.state.todos).render(ctx, left_area);
        ContentWidget::new(self.state, self.ui).render(ctx, content_area);
        BottomBarWidget::new(self.state, self.autosave).render(ctx, bottom_area);
    }
}
