use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::Line,
    widgets::{Block, BorderType, List, ListItem, ListState, Padding, Widget},
};

use super::{state::UIState, widgets::popup_widget::utils::calculate_popup_area};
use crate::app::{models::todo::Todo, utils::layout::center};

pub struct Renderer;

impl Renderer {
    pub fn render(
        &self,
        frame: &mut Frame,
        todos: &[Todo],
        select_state: &mut ListState,
        ui: &UIState,
    ) {
        self.render_todo_list(frame, todos, select_state);

        if let Some(popup) = &ui.popup {
            let popup_area: Rect = calculate_popup_area(popup.clone(), frame.area());
            self.render_overlay_except(frame, popup_area);
            popup.render(frame, popup_area);
        }

        if let Some(input) = &ui.inputbox {
            let input_area: Rect = center(frame.area(), 50, 3);
            self.render_overlay_except(frame, input_area);
            input.clone().render(frame, input_area);
        }

        if let Some(confirm) = &ui.confirm {
            let confirm_area: Rect = center(frame.area(), 40, 10);
            self.render_overlay_except(frame, confirm_area);
            confirm.render(frame, confirm_area);
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

    fn render_todo_list(&self, frame: &mut Frame, todos: &[Todo], select_state: &mut ListState) {
        let [main_layout] = Layout::vertical([Constraint::Fill(1)])
            .margin(1)
            .areas(frame.area());

        let [inner_layout] = Layout::vertical([Constraint::Fill(1)])
            .margin(3)
            .areas(main_layout);

        Block::default()
            .fg(Color::Rgb(230, 185, 157))
            .padding(Padding::uniform(2))
            .render(main_layout, frame.buffer_mut());

        let list_block = Block::bordered()
            .border_type(BorderType::Rounded)
            .title(" List of what's to complete ")
            .title_bottom(
                Line::from(" Help <?> ")
                    .fg(Color::Rgb(252, 252, 252))
                    .centered(),
            )
            .padding(Padding::uniform(1));

        let list_widget = List::new(todos.iter().map(|item| {
            let prefix = if item.done { " [✓] " } else { " [ ] " };
            ListItem::new(format!("{}{}", prefix, item.title))
        }))
        .block(list_block)
        .highlight_symbol(">")
        .highlight_style(Style::default().fg(Color::Rgb(229, 218, 156)));

        frame.render_stateful_widget(list_widget, inner_layout, select_state);
    }
}
