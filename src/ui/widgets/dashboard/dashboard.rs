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
            MainLayout,
            widgets::dashboard::{
                content::ContentWidget, footer::FooterWidget, sidebar::SidebarWidget,
            },
        };

        let show_sidebar: bool = ctx.config.show_sidebar;
        let layout: MainLayout = MainLayout::split(area, ctx.config.show_sidebar);

        if show_sidebar {
            SidebarWidget::new(self.ui, &self.state.tasks).render(ctx, layout.sidebar);
        }

        ContentWidget::new(self.state, self.ui).render(ctx, layout.content);
        FooterWidget::new(self.state, self.autosave).render(ctx, layout.footer);
    }
}
