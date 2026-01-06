use super::state::UIState;
use crate::app::{models::todo::Todo, utils::colors::theme::*};
use ratatui::{
    Frame,
    layout::Rect,
    widgets::{Clear, ListState},
};

pub struct Renderer;

impl Renderer {
    pub fn render(
        &self,
        frame: &mut Frame,
        todos: &[Todo],
        select_state: &mut ListState,
        ui: &UIState,
    ) {
        use crate::app::ui::widgets::todo_list::todo_list::TodoList;

        TodoList::render(frame, todos, select_state);

        if let Some(notification) = &ui.notification {
            let notification_area: Rect = notification.area(frame.area());
            frame.render_widget(Clear, notification_area);
            notification.render(frame, notification_area);
        }

        if let Some(dialog) = &ui.dialog {
            let dialog_area: Rect = dialog.modal.area(frame.area());
            frame.render_widget(Clear, dialog_area);
            self.render_overlay_except(frame, dialog_area);
            dialog.modal.render(frame, dialog_area);
        }

        if let Some(input) = &ui.input {
            let input_area: Rect = input.area(frame.area());
            frame.render_widget(Clear, input_area);
            self.render_overlay_except(frame, input_area);
            input.clone().render(frame, input_area);
        }
    }

    fn render_overlay_except(&self, frame: &mut Frame, widget_area: Rect) {
        use ratatui::{
            style::{Modifier, Style},
            widgets::Block,
        };

        let full: Rect = frame.area();

        let blackout: Block =
            Block::default().style(Style::default().bg(BG_DIM).add_modifier(Modifier::DIM));

        if widget_area.y > 0 {
            let top = Rect::new(full.x, full.y, full.width, widget_area.y);
            frame.render_widget(&blackout, top);
        }

        let bottom_y = widget_area.y + widget_area.height;
        if bottom_y < full.height {
            let bottom = Rect::new(full.x, bottom_y, full.width, full.height - bottom_y);
            frame.render_widget(&blackout, bottom);
        }

        if widget_area.x > 0 {
            let left = Rect::new(full.x, widget_area.y, widget_area.x, widget_area.height);
            frame.render_widget(&blackout, left);
        }

        let right_x = widget_area.x + widget_area.width;
        if right_x < full.width {
            let right = Rect::new(
                right_x,
                widget_area.y,
                full.width - right_x,
                widget_area.height,
            );

            frame.render_widget(&blackout, right);
        }
    }
}
