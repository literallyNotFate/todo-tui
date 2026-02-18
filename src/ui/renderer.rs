use crate::{
    core::{ApplicationMode, Autosave},
    state::{ApplicationState, UIState},
    ui::RenderContext,
};
use ratatui::Frame;

/// Application render
pub struct Renderer;

impl Renderer {
    pub fn render(
        &self,
        frame: &mut Frame,
        state: &mut ApplicationState,
        ui: &UIState,
        mode: ApplicationMode,
        autosave: &Autosave,
    ) {
        use crate::ui::{Dashboard, FeedbackKind, FeedbackWidget};
        use ratatui::{
            layout::Rect,
            style::{Style, Stylize},
            widgets::{Block, Clear},
        };

        let mut ctx = RenderContext::new(frame, ui, mode);
        let area = ctx.frame.area();

        if ctx.is_small() {
            FeedbackWidget::new(FeedbackKind::SmallTerminal).render(&mut ctx, area);
            return;
        }

        Dashboard::new(state, ui, autosave).render(&mut ctx, area);

        let blackout: Block = Block::default().style(Style::default().bg(ctx.theme.modal_bg).dim());

        if let Some(dialog) = &ui.modal {
            let dialog_area: Rect = dialog.modal.area(area);

            ctx.render_widget(&blackout, area);
            ctx.render_widget(Clear, dialog_area);
            dialog.modal.render(&mut ctx, dialog_area);
        }
    }
}
