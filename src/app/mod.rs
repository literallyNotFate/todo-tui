pub mod models;
pub mod ui;
pub mod utils;

use color_eyre::Result;
use models::todo::Todo;
use ratatui::{
    DefaultTerminal, Frame,
    crossterm::event::{self, Event, KeyCode, KeyModifiers},
};

use ui::{
    UIState,
    components::popup::{Popup, PopupCloseBehavior, PopupKind},
    renderer::Renderer,
};

pub struct Application {
    pub todos: Vec<Todo>,
    pub running: bool,
    pub ui: UIState,
    pub renderer: Renderer,
}

impl Application {
    pub fn new() -> Self {
        Self {
            todos: Vec::new(),
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
            KeyCode::Char('?') => {
                let usage: Vec<&str> = vec![
                    " a -> append a todo",
                    " r -> rename a todo",
                    " d -> delete a todo",
                    " Enter -> mark as completed",
                    " k/Up -> go up",
                    " j/Down -> go down",
                    " q/Esc -> quit",
                    " ? -> toggle help",
                ];

                let usage_text = usage.join("\n");
                self.ui.show_popup(
                    Popup::new(usage_text)
                        .kind(PopupKind::Help)
                        .title("Controls")
                        .close_on(KeyCode::Char('?')),
                )
            }
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
        self.renderer.render(frame, self);
    }
}
