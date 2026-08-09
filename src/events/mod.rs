mod actions;
mod modal;
mod search;

use crate::{
    app::ApplicationController,
    config::KeyMaps,
    core::{Action, ApplicationMode},
    ui::is_terminal_small,
};

/// Handling all possible keys
pub struct EventHandler;

impl EventHandler {
    /// Main key event handling
    pub fn handle_key(app: &mut crate::Application, event: ratatui::crossterm::event::KeyEvent) {
        if KeyMaps::is_kill_process(&event) {
            app.running = false;
            return;
        }

        if is_terminal_small(app.size.0, app.size.1) {
            if app.keymaps.is(&event, Action::Quit) {
                app.running = false;
            }
            app.ui.request_redraw();
            return;
        }

        let mut ctrl =
            ApplicationController::new(&mut app.data, &mut app.ui, &mut app.config, &app.keymaps);

        if ctrl.ui.modal.is_some() {
            modal::handle_modal(event, &mut ctrl, &mut app.storage, &mut app.running);
            return;
        }

        if let ApplicationMode::Search = app.mode {
            search::handle_search(event, &mut ctrl, &mut app.mode);
            return;
        }

        if let Some(action) = app.keymaps.action(&event) {
            actions::handle_action(
                action,
                &mut ctrl,
                &mut app.storage,
                &mut app.mode,
                &mut app.autosave,
                &mut app.running,
            );
        }
    }
}
