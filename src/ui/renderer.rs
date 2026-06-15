use crate::{
    config::{Config, KeyMaps},
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
        config: &Config,
        keymaps: &KeyMaps,
    ) {
        use crate::ui::{Dashboard, FeedbackKind, FeedbackWidget};
        use ratatui::{layout::Rect, widgets::Clear};

        let has_modal: bool = ui.modal.is_some();
        let mut ctx: RenderContext =
            RenderContext::new(frame, ui, config, keymaps, mode).with_dimmed(has_modal);
        let area: Rect = ctx.area();

        if ctx.is_small() {
            FeedbackWidget::new(FeedbackKind::SmallTerminal).render(&mut ctx, area);
            return;
        }

        Dashboard::new(state, ui, autosave).render(&mut ctx, area);

        if let Some(dialog) = &ui.modal {
            let dialog_area: Rect = dialog.modal.area(area);
            ctx.render_modal_overlay();
            ctx.render_widget(Clear, dialog_area);

            ctx.set_dimmed(false);
            dialog.modal.render(&mut ctx, dialog_area);
        }
    }
}
