pub mod models;
pub mod ui;
pub mod utils;

use color_eyre::Result;
use ratatui::{
    DefaultTerminal, Frame,
    crossterm::event::{self, Event, KeyCode, KeyModifiers},
};

use crate::app::models::todo::Todo;
use ui::{
    UIState,
    popup::{Popup, PopupCloseBehavior, PopupKind},
};

pub struct Application {
    pub todos: Vec<Todo>,
    pub running: bool,
    pub ui: UIState,
}

impl Application {
    pub fn new() -> Self {
        Self {
            todos: Vec::new(),
            running: true,
            ui: UIState::default(),
        }
    }

    fn handle_key(&mut self, key: KeyCode, modifiers: KeyModifiers) {
        if key == KeyCode::Char('c') && modifiers.contains(KeyModifiers::CONTROL) {
            self.running = false;
            return;
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

        match key {
            KeyCode::Char('q') | KeyCode::Esc => self.running = false,
            KeyCode::Char('?') => self.ui.show_popup(
                Popup::new(PopupKind::Success, "Testing message").close_on(KeyCode::Char('q')),
            ),
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

    pub fn render(&self, frame: &mut Frame) {
        frame.render_widget("todo-tui", frame.area());

        self.ui.render(frame);
    }
}
