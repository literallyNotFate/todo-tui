use super::state::UIState;
use crate::app::models::todo::Todo;
use ratatui::{Frame, layout::Rect, widgets::ListState};

pub struct Renderer;

impl Renderer {
    pub fn render(
        &self,
        frame: &mut Frame,
        todos: &[Todo],
        select_state: &mut ListState,
        ui: &UIState,
    ) {
        use crate::app::{
            ui::widgets::{fallback::fallback::Fallback, todo_list::todo_list::TodoList},
            utils::constants::{terminal::*, theme::*},
        };
        use ratatui::{
            style::{Modifier, Style},
            widgets::{Block, Clear},
        };

        let area: Rect = frame.area();
        if is_terminal_small(area.width, area.height) {
            Fallback::render(frame, area);
            return;
        }

        TodoList::render(frame, todos, select_state);

        if let Some(notification) = &ui.notification {
            let notification_area: Rect = notification.area(area);
            frame.render_widget(Clear, notification_area);
            notification.render(frame, notification_area);
        }

        let blackout: Block =
            Block::default().style(Style::default().bg(BG_DIM).add_modifier(Modifier::DIM));

        if let Some(dialog) = &ui.dialog {
            let dialog_area: Rect = dialog.modal.area(area);
            frame.render_widget(&blackout, area);
            frame.render_widget(Clear, dialog_area);
            dialog.modal.render(frame, dialog_area);
        }

        if let Some(input) = &ui.input {
            let input_area: Rect = input.area(area);
            frame.render_widget(&blackout, area);
            frame.render_widget(Clear, input_area);
            input.clone().render(frame, input_area);
        }
    }
}
