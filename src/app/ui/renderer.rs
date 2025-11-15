use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::Block,
};

use super::widgets::popup_widget::utils::calculate_area;
use crate::app::{application::Application, models::todo::Todo};

pub struct Renderer;

impl Renderer {
    pub fn render(&self, frame: &mut Frame, app: &Application) {
        self.render_todo_list(frame, &app.todos);

        if let Some(popup) = &app.ui.popup {
            let popup_area: Rect = calculate_area(popup.clone(), frame.area());
            self.render_overlay_except(frame, popup_area);
            popup.render(frame, popup_area);
        }
    }

    fn render_overlay_except(&self, frame: &mut Frame, popup_area: Rect) {
        let full: Rect = frame.area();

        let blackout: Block = Block::default().style(
            Style::default()
                .bg(Color::Rgb(0, 0, 0))
                .add_modifier(Modifier::DIM),
        );

        if popup_area.y > 0 {
            let top = Rect::new(full.x, full.y, full.width, popup_area.y);
            frame.render_widget(&blackout, top);
        }

        let bottom_y = popup_area.y + popup_area.height;
        if bottom_y < full.height {
            let bottom = Rect::new(full.x, bottom_y, full.width, full.height - bottom_y);
            frame.render_widget(&blackout, bottom);
        }

        if popup_area.x > 0 {
            let left = Rect::new(full.x, popup_area.y, popup_area.x, popup_area.height);
            frame.render_widget(&blackout, left);
        }

        let right_x = popup_area.x + popup_area.width;
        if right_x < full.width {
            let right = Rect::new(
                right_x,
                popup_area.y,
                full.width - right_x,
                popup_area.height,
            );

            frame.render_widget(&blackout, right);
        }
    }

    fn render_todo_list(&self, frame: &mut Frame, todos: &[Todo]) {
        frame.render_widget("your list goes here )", frame.area());
    }
}
