use crate::{
    core::ApplicationMode,
    state::{ApplicationState, UIState},
};
use ratatui::{Frame, widgets::Widget};

/// Application render
pub struct Renderer;

impl Renderer {
    pub fn render(
        &self,
        frame: &mut Frame,
        state: &mut ApplicationState,
        ui: &UIState,
        mode: ApplicationMode,
        autosave_enabled: bool,
    ) {
        use crate::{
            theme::ThemeColors,
            ui::{Dashboard, FeedbackKind, FeedbackWidget, is_terminal_small},
        };
        use ratatui::{
            layout::Rect,
            style::{Style, Stylize},
            widgets::{Block, Clear},
        };

        let colors: ThemeColors = ui.theme.colors();
        let area: Rect = frame.area();

        if is_terminal_small(area.width, area.height) {
            FeedbackWidget::new(FeedbackKind::SmallTerminal, &colors)
                .render(area, frame.buffer_mut());
            return;
        }

        Dashboard::new(state, ui, &mode, autosave_enabled, &colors).render(frame, area);

        let blackout: Block = Block::default().style(Style::default().bg(colors.modal_bg).dim());

        if let Some(dialog) = &ui.modal {
            let dialog_area: Rect = dialog.modal.area(area);
            frame.render_widget(&blackout, area);
            frame.render_widget(Clear, dialog_area);
            dialog.modal.render(frame, dialog_area, &colors);
        }
    }
}
