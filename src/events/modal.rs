use crate::{
    app::ApplicationController,
    core::{Action, Selectable, Storage},
    models::{FolderColor, FolderEditor, TaskEditor},
    ui::widgets::modal::{ModalAction, ModalResult},
};

/// Handle keys in modal widgets
pub fn handle_modal(
    event: ratatui::crossterm::event::KeyEvent,
    ctrl: &mut ApplicationController,
    storage: &mut Storage,
    running: &mut bool,
) {
    let action: Option<Action> = ctrl.keymaps.action(&event);
    let result: Option<ModalResult> = {
        let modal_wrapper = match ctrl.ui.modal.as_mut() {
            Some(m) => m,
            None => return,
        };
        modal_wrapper.modal.handle_action(action, &event)
    };

    if let Some(result) = result {
        if let ModalResult::Changed { theme_id } = result {
            ctrl.ui.apply_theme_id(theme_id);
            ctrl.ui.request_redraw();
            return;
        }

        let modal_action = ctrl.ui.modal.as_ref().unwrap().action.clone();
        ctrl.ui.close_modal();

        match result {
            ModalResult::Changed { .. } => unreachable!(),
            ModalResult::TaskSubmitted {
                id,
                title,
                description,
                priority,
            } => {
                let editor = TaskEditor {
                    title,
                    description,
                    priority: Selectable::new(priority),
                    folder_id: None,
                };

                if let Some(task_id) = id {
                    ctrl.dispatch_update_task(task_id, editor);
                } else {
                    ctrl.dispatch_append_task(
                        editor.title,
                        editor.description,
                        Some(*editor.priority),
                    );
                }
            }
            ModalResult::FolderSubmitted { id, name, color } => {
                use std::str::FromStr;
                let folder_color = FolderColor::from_str(&color).unwrap_or_default();
                let editor = FolderEditor::new(name, folder_color);

                if let Some(folder_id) = id {
                    ctrl.dispatch_update_folder(folder_id, editor);
                } else {
                    ctrl.dispatch_append_folder(editor.name, editor.color);
                }
            }
            ModalResult::Confirmed => {
                log::debug!("Modal confirmed: action={:?}", modal_action);
                match modal_action {
                    ModalAction::Remove => ctrl.dispatch_remove_task(),
                    ModalAction::Clear => ctrl.dispatch_clear(),
                    ModalAction::RemoveFolder(folder_id) => ctrl.dispatch_remove_folder(folder_id),
                    ModalAction::Save => {
                        ctrl.dispatch_save(storage);
                    }
                    ModalAction::UnsavedExit => {
                        if ctrl.dispatch_save(storage) {
                            *running = false;
                        }
                    }
                    _ => {}
                }
            }
            ModalResult::Cancelled => {
                log::debug!("Modal cancelled: action={:?}", modal_action);
            }
        }
    }

    ctrl.ui.request_redraw();
}

/// Unit-tests for modal key handler
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        config::{Config, KeyMaps},
        core::Storage,
        models::Task,
        state::{ApplicationState, UIState},
        ui::Popup,
    };
    use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    use tempdir::TempDir;

    fn key_event(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    struct TestContext {
        _temp_dir: TempDir,
        storage: Storage,
        state: ApplicationState,
        ui: UIState,
        config: Config,
        keymaps: KeyMaps,
        running: bool,
    }

    impl TestContext {
        fn new() -> Self {
            let temp_dir = TempDir::new("modal_event_test").unwrap();
            let db_path = temp_dir.path().join("test_modal.db");
            let config = Config::default();
            let storage = Storage::init(Some(&db_path), &config.storage).unwrap();

            Self {
                _temp_dir: temp_dir,
                storage,
                state: ApplicationState::default(),
                ui: UIState::default(),
                config,
                keymaps: KeyMaps::default(),
                running: true,
            }
        }

        pub fn components(
            &mut self,
        ) -> (
            &mut ApplicationState,
            &mut UIState,
            &mut Config,
            &KeyMaps,
            &mut Storage,
            &mut bool,
        ) {
            (
                &mut self.state,
                &mut self.ui,
                &mut self.config,
                &self.keymaps,
                &mut self.storage,
                &mut self.running,
            )
        }
    }

    #[test]
    fn should_confirm_remove_task() {
        let mut ctx = TestContext::new();
        let (state, ui, config, keymaps, storage, running) = ctx.components();

        state.tasks.push(Task::new("To be deleted"));
        state.select_state.select(Some(0));

        ui.remove_task_confirm();
        let mut ctrl = ApplicationController::new(state, ui, config, keymaps);
        handle_modal(key_event(KeyCode::Char('y')), &mut ctrl, storage, running);

        assert_eq!(ctrl.state.tasks.len(), 0);
        assert!(ctrl.ui.modal.is_none());
        assert!(*running, "App should be running still");
    }

    #[test]
    fn should_cancel_clear_all_tasks() {
        let mut ctx = TestContext::new();
        let (state, ui, config, keymaps, storage, running) = ctx.components();
        state.tasks.push(Task::new("Keep me"));

        ui.clear_confirm();
        let mut ctrl = ApplicationController::new(state, ui, config, keymaps);
        handle_modal(key_event(KeyCode::Esc), &mut ctrl, storage, running);

        assert_eq!(ctrl.state.tasks.len(), 1);
        assert!(ctrl.ui.modal.is_none());
    }

    #[test]
    fn should_save_and_exit_on_unsaved_exit_confirm() {
        let mut ctx = TestContext::new();
        let (state, ui, config, keymaps, storage, running) = ctx.components();

        state.tasks.push(Task::new("Task to DB"));
        ui.unsaved_confirm();

        let mut ctrl = ApplicationController::new(state, ui, config, keymaps);
        handle_modal(key_event(KeyCode::Char('y')), &mut ctrl, storage, running);

        assert!(!(*running), "App should be closed");
    }

    #[test]
    fn should_not_exit_on_unsaved_exit_cancel() {
        let mut ctx = TestContext::new();
        let (state, ui, config, keymaps, storage, running) = ctx.components();
        ui.unsaved_confirm();

        let mut ctrl = ApplicationController::new(state, ui, config, keymaps);
        handle_modal(key_event(KeyCode::Char('n')), &mut ctrl, storage, running);

        assert!(*running, "App should not be closed if cancelled");
        assert!(ctrl.ui.modal.is_none());
    }

    #[test]
    fn should_do_nothing_if_no_modal_exists() {
        let mut ctx = TestContext::new();
        let (state, ui, config, keymaps, storage, running) = ctx.components();
        ui.modal = None;
        let mut ctrl = ApplicationController::new(state, ui, config, keymaps);
        handle_modal(key_event(KeyCode::Enter), &mut ctrl, storage, running);

        assert!(*running);
        assert!(ctrl.ui.modal.is_none());
    }

    #[test]
    fn should_create_new_task_on_submit() {
        let mut ctx = TestContext::new();
        let (state, ui, config, keymaps, storage, running) = ctx.components();

        ui.show_modal(Popup::append_task(), ModalAction::None);
        let mut ctrl = ApplicationController::new(state, ui, config, keymaps);

        handle_modal(key_event(KeyCode::Char('G')), &mut ctrl, storage, running);
        handle_modal(key_event(KeyCode::Char('o')), &mut ctrl, storage, running);

        handle_modal(key_event(KeyCode::Down), &mut ctrl, storage, running);
        handle_modal(key_event(KeyCode::Down), &mut ctrl, storage, running);

        handle_modal(key_event(KeyCode::Char('T')), &mut ctrl, storage, running);
        handle_modal(key_event(KeyCode::Char('U')), &mut ctrl, storage, running);
        handle_modal(key_event(KeyCode::Char('I')), &mut ctrl, storage, running);

        handle_modal(key_event(KeyCode::Down), &mut ctrl, storage, running);
        handle_modal(key_event(KeyCode::Enter), &mut ctrl, storage, running);

        assert!(ctrl.ui.modal.is_none());
        assert_eq!(ctrl.state.tasks.len(), 1);
        assert_eq!(ctrl.state.tasks[0].title, "Go");
        assert_eq!(ctrl.state.tasks[0].description, "TUI");
    }
}
