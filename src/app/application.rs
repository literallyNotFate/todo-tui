use color_eyre::Result;
use ratatui::{
    DefaultTerminal, Frame,
    crossterm::event::{self, Event, KeyCode, KeyModifiers},
};

use super::{
    state::ApplicationState,
    ui::{
        components::help_popup,
        renderer::Renderer,
        state::UIState,
        widgets::{
            confirm_widget::{action::ConfirmAction, confirm::Confirm},
            inputbox::{
                input::InputBox,
                state::{InputMode, InputResult},
            },
            popup_widget::popup::PopupCloseBehavior,
        },
    },
};

pub struct Application {
    pub state: ApplicationState,
    pub running: bool,
    pub ui: UIState,
    pub renderer: Renderer,
}

impl Application {
    pub fn new() -> Self {
        Self {
            state: ApplicationState::new(),
            running: true,
            ui: UIState::default(),
            renderer: Renderer,
        }
    }

    fn handle_key(&mut self, key: KeyCode, modifiers: KeyModifiers) {
        if key == KeyCode::Char('c') && modifiers.contains(KeyModifiers::CONTROL) {
            self.running = false;
            return;
        }

        if let Some(confirm) = self.ui.confirm.as_mut() {
            if let Some(result) = confirm.handle_key(key) {
                let action = confirm.action.take();
                self.ui.close_confirm();

                if result && let Some(action) = action {
                    match action {
                        ConfirmAction::Append(text) => {
                            self.state.append_todo(text);
                        }
                        ConfirmAction::Remove => {
                            self.state.remove_todo();
                        }
                        ConfirmAction::Rename(text) => {
                            self.state.rename_todo(text);
                        }
                    }
                }

                return;
            } else {
                return;
            }
        }

        if let Some(popup) = &self.ui.popup {
            match popup.close_behavior {
                PopupCloseBehavior::AnyKey => {
                    self.ui.close_popup();
                }
                PopupCloseBehavior::Specific(k) if k == key => {
                    self.ui.close_popup();
                }
                _ => {}
            }

            return;
        }

        if let Some(input) = self.ui.inputbox.as_mut() {
            match input.handle_key(key) {
                InputResult::Continue => (),
                InputResult::Cancel => self.ui.close_input(),
                InputResult::Submit(text) => {
                    match input.mode {
                        InputMode::Insert => self.ui.show_confirm(
                            Confirm::new()
                                .with_message("Append this todo?")
                                .action(ConfirmAction::Append(text)),
                        ),
                        InputMode::Edit => self.ui.show_confirm(
                            Confirm::new()
                                .with_message("Rename this todo?")
                                .action(ConfirmAction::Rename(text)),
                        ),
                    }

                    self.ui.close_input();
                }
            }
            return;
        }

        match key {
            KeyCode::Char('q') | KeyCode::Esc => self.running = false,
            KeyCode::Char('k') | KeyCode::Up => self.state.select_state.select_previous(),
            KeyCode::Char('j') | KeyCode::Down => self.state.select_state.select_next(),
            KeyCode::Char('a') => self.ui.show_input(InputBox::insert()),
            KeyCode::Char('r') => self
                .ui
                .show_input(InputBox::edit(self.state.get_current_todo().title)),
            KeyCode::Char('d') => self.ui.show_confirm(
                Confirm::new()
                    .with_message("Remove this todo?")
                    .action(ConfirmAction::Remove),
            ),
            KeyCode::Enter => self.state.toggle_current(),
            KeyCode::Char('?') => self.ui.show_popup(help_popup::help_popup()),
            _ => {}
        }
    }

    pub fn run(&mut self, mut terminal: DefaultTerminal) -> Result<()> {
        while self.running {
            terminal.draw(|frame| self.render(frame))?;

            if let Event::Key(key) = event::read()? {
                self.handle_key(key.code, key.modifiers);
            }
        }

        Ok(())
    }

    pub fn render(&mut self, frame: &mut Frame) {
        self.renderer.render(
            frame,
            &self.state.todos,
            &mut self.state.select_state,
            &self.ui,
        );
    }
}
