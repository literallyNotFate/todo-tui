pub mod models;

use color_eyre::Result;
use ratatui::{
    DefaultTerminal, Frame,
    crossterm::event::{self, Event, KeyCode},
};

use crate::app::models::todo::Todo;

pub struct Application {
    pub todos: Vec<Todo>,
}

impl Application {
    pub fn new() -> Result<Self> {
        Ok(Self { todos: Vec::new() })
    }

    pub fn run(&mut self, mut terminal: DefaultTerminal) -> Result<()> {
        loop {
            terminal.draw(|frame| self.render(frame))?;

            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => break,
                    _ => {}
                }
            }
        }

        Ok(())
    }

    pub fn render(&self, frame: &mut Frame) {
        frame.render_widget("todo-tui", frame.area());
    }
}
