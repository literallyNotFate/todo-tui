use super::{
    state::state::ApplicationState,
    ui::renderer::{renderer::Renderer, state::UIState},
};
use crate::app::{handlers::key::handle_key_event, utils::constants::terminal::is_terminal_small};
use color_eyre::eyre::Result;
use ratatui::{
    DefaultTerminal, Frame,
    crossterm::event::{self, Event},
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

    pub fn run(&mut self, mut terminal: DefaultTerminal) -> Result<()> {
        use std::time::{Duration, Instant};

        const TICK_RATE: Duration = Duration::from_millis(100);
        let mut last_tick: Instant = Instant::now();

        while self.running {
            terminal.draw(|frame| self.render(frame))?;

            let timeout: Duration = TICK_RATE.saturating_sub(last_tick.elapsed());

            if event::poll(timeout)?
                && let Event::Key(key_event) = event::read()?
            {
                if let Ok(size) = terminal.size()
                    && is_terminal_small(size.width, size.height)
                {
                    continue;
                }

                handle_key_event(self, key_event);
            }

            if last_tick.elapsed() >= TICK_RATE {
                self.tick();
                last_tick = Instant::now();
            }
        }

        Ok(())
    }

    // Tick function (for notification)
    pub fn tick(&mut self) {
        self.ui.expire_notification();
    }

    // Rendering
    pub fn render(&mut self, frame: &mut Frame) {
        self.renderer.render(
            frame,
            &self.state.todos,
            &mut self.state.select_state,
            &self.ui,
        );
    }
}
