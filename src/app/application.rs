use super::{
    state::ApplicationState,
    ui::{renderer::Renderer, state::UIState},
};
use crate::app::ui::state::DialogIntent;
use color_eyre::eyre::Result;
use ratatui::{
    DefaultTerminal, Frame,
    crossterm::event::{self, Event, KeyCode, KeyModifiers},
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

        // Ctrl + C exit
        if key == KeyCode::Char('c') && modifiers.contains(KeyModifiers::CONTROL) {
            self.running = false;
            return;
        }

        // Handling notification
        if let Some(notification) = &self.ui.notification
            && notification.created_at.elapsed() > notification.duration
        {
            self.ui.notification = None;
        }

        // Handling dialog (confirm/popup)
        if let Some(active) = self.ui.dialog.as_mut() {
            if let Some(result) = active.modal.handle_key(key) {
                match result {
                    DialogResult::Cancelled => {
                        self.ui.close_dialog();
                        return;
                    }

                    DialogResult::Confirmed => {
                        match &active.intent {
                            DialogIntent::Remove => self.state.remove_todo(),
                            DialogIntent::Clear => self.state.clear_todos(),
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

        // Handling input box widget
        if let Some(input) = self.ui.input.as_mut() {
            match input.handle_key(key) {
                InputResult::Continue => (),
                InputResult::Cancel => self.ui.close_input(),
                InputResult::Submit(text) => {
                    match input.mode {
                        InputMode::Insert => self.state.append_todo(text),
                        InputMode::Edit => self.state.rename_todo(text),
                    }

                    self.ui.close_input();
                }
            }
            return;
        }

        // Other keys
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
                    self.ui.show_dialog(
                        Components::remove_todo_confirm(self.state.get_current_todo().title),
                        DialogIntent::Remove,
                    )
                }
            }
            KeyCode::Char('x') => {
                if !self.state.todos.is_empty() {
                    self.ui.show_dialog(
                        Components::clear_todos_confirm(self.state.todos.len()),
                        DialogIntent::Clear,
                    )
                }
            }
            KeyCode::Enter => self.state.toggle_current(),
            KeyCode::Char('?') => self
                .ui
                .show_dialog(Components::help_popup(), DialogIntent::None),
            _ => {}
        }
    }

    pub fn tick(&mut self) {
        if let Some(n) = &self.ui.notification
            && n.created_at.elapsed() >= n.duration
        {
            self.ui.notification = None;
        }
    }

    pub fn run(&mut self, mut terminal: DefaultTerminal) -> Result<()> {
        use std::time::{Duration, Instant};

        let tick_rate = Duration::from_millis(100);
        let mut last_tick = Instant::now();

        while self.running {
            terminal.draw(|frame| self.render(frame))?;

            let timeout = tick_rate
                .checked_sub(last_tick.elapsed())
                .unwrap_or(Duration::ZERO);

            if event::poll(timeout)?
                && let Event::Key(key) = event::read()?
            {
                self.handle_key(key.code, key.modifiers);
            }

            if last_tick.elapsed() >= tick_rate {
                self.tick();
                last_tick = Instant::now();
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
