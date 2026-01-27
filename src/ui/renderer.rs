use crate::{
    enums::ApplicationMode,
    state::{ApplicationState, UIState},
    theme::ThemeColors,
};
use ratatui::Frame;

pub struct Renderer;

impl Renderer {
    pub fn render(
        &self,
        frame: &mut Frame,
        state: &mut ApplicationState,
        ui: &UIState,
        mode: ApplicationMode,
    ) {
        use super::Fallback;
        use crate::ui::{Menu, is_terminal_small, main_layout};
        use ratatui::{
            layout::Rect,
            style::{Modifier, Style},
            widgets::{Block, Clear},
        };

        let theme_colors: ThemeColors = ui.theme.data();
        let area: Rect = frame.area();

        if is_terminal_small(area.width, area.height) {
            Fallback::render(frame, area, &theme_colors);
            return;
        }

        let layouts: (Rect, Rect, Rect) = main_layout(area);
        Menu::render(frame, layouts, state, ui, &mode);

        let blackout: Block = Block::default().style(
            Style::default()
                .bg(theme_colors.modal_bg)
                .add_modifier(Modifier::DIM),
        );

        if let Some(dialog) = &ui.modal {
            let dialog_area: Rect = dialog.modal.area(area);
            frame.render_widget(&blackout, area);
            frame.render_widget(Clear, dialog_area);
            dialog.modal.render(frame, dialog_area, &theme_colors);
        }
    }
}
