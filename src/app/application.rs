use color_eyre::eyre::Result;
use ratatui::{
    DefaultTerminal, Frame,
    crossterm::event::{self, Event, KeyCode, KeyModifiers},
};

use crate::app::ui::state::DialogIntent;

use super::{
    state::ApplicationState,
    ui::{renderer::Renderer, state::UIState},
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
        use super::ui::{
            components::components::Components,
            dialogs::dialog::DialogResult,
            widgets::input::input::{Input, InputMode, InputResult},
        };

        if key == KeyCode::Char('c') && modifiers.contains(KeyModifiers::CONTROL) {
            self.running = false;
            return;
        }

        if let Some(active) = self.ui.dialog.as_mut() {
            if let Some(result) = active.modal.handle_key(key) {
                match result {
                    DialogResult::None => {
                        return;
                    }

                    DialogResult::Cancelled => {
                        self.ui.close_dialog();
                        return;
                    }

                    DialogResult::Confirmed => {
                        match &active.intent {
                            DialogIntent::Append(text) => {
                                self.state.append_todo(text);
                            }
                            DialogIntent::Rename(text) => {
                                self.state.rename_todo(text);
                            }
                            DialogIntent::Remove => {
                                self.state.remove_todo();
                            }
                            DialogIntent::None => (),
                        }

                        self.ui.close_dialog();
                        return;
                    }
                }
            } else {
                return;
            }
        }

        if let Some(input) = self.ui.input.as_mut() {
            match input.handle_key(key) {
                InputResult::Continue => (),
                InputResult::Cancel => self.ui.close_input(),
                InputResult::Submit(text) => {
                    match input.mode {
                        InputMode::Insert => self
                            .ui
                            .show_dialog(Components::append_confirm(), DialogIntent::Append(text)),
                        InputMode::Edit => self
                            .ui
                            .show_dialog(Components::rename_confirm(), DialogIntent::Rename(text)),
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
            KeyCode::Char('a') => self.ui.show_input(Input::insert()),
            KeyCode::Char('r') => {
                if !self.state.todos.is_empty() {
                    self.ui
                        .show_input(Input::edit(self.state.get_current_todo().title))
                }
            }
            KeyCode::Char('d') => {
                if !self.state.todos.is_empty() {
                    self.ui
                        .show_dialog(Components::remove_confirm(), DialogIntent::Remove)
                }
            }
            KeyCode::Enter => self.state.toggle_current(),
            KeyCode::Char('?') => self
                .ui
                .show_dialog(Components::help_popup(), DialogIntent::None),
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
