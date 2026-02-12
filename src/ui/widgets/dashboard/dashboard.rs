use crate::{
    enums::ApplicationMode,
    state::{ApplicationState, UIState},
    theme::ThemeColors,
};
use ratatui::{Frame, layout::Rect};

/// Main menu widget (combines sidebar, content, bottom)
pub struct Dashboard<'a> {
    pub state: &'a mut ApplicationState,
    pub ui: &'a UIState,
    pub mode: &'a ApplicationMode,
    pub theme: &'a ThemeColors,
}

impl<'a> Dashboard<'a> {
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

    // Dashboard rendering
    pub fn render(self, frame: &mut Frame, area: Rect) {
        use crate::ui::{
            main_layout,
            widgets::dashboard::{
                bottom::BottomBarWidget, content::ContentWidget, sidebar::SidebarWidget,
            },
        };

        let (left_area, content_area, bottom_area): (Rect, Rect, Rect) = main_layout(area);

        SidebarWidget::new(
            self.ui,
            &self.state.todos,
            self.mode,
            self.state.sort.clone(),
            self.theme,
        )
        .render(frame, left_area);
        ContentWidget::new(self.state, self.ui, self.mode, self.theme).render(frame, content_area);
        BottomBarWidget::new(self.state, self.theme).render(frame, bottom_area);
    }
}
