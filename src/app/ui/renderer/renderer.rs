use super::state::UIState;
use crate::app::{
    models::todo::Todo,
    utils::{
        constants::{
            size::{FALLBACK_HEIGHT, FALLBACK_WIDTH},
            terminal::*,
            theme::*,
        },
        layout::center,
    },
};
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

        let area: Rect = frame.area();
        if is_terminal_small(frame.area()) {
            self.render_fallback(frame, area);
            return;
        }

        TodoList::render(frame, todos, select_state);

        if let Some(notification) = &ui.notification {
            let notification_area: Rect = notification.area(area);
            frame.render_widget(Clear, notification_area);
            notification.render(frame, notification_area);
        }

        if let Some(dialog) = &ui.dialog {
            let dialog_area: Rect = dialog.modal.area(area);
            frame.render_widget(Clear, dialog_area);
            self.render_overlay_except(frame, dialog_area);
            dialog.modal.render(frame, dialog_area);
        }

        if let Some(input) = &ui.input {
            let input_area: Rect = input.area(area);
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

    fn render_fallback(&self, frame: &mut Frame, frame_area: Rect) {
        use crate::app::utils::constants::terminal::dimension_colors;
        use ratatui::{
            style::{Color, Modifier, Style},
            text::{Line, Span, Text},
            widgets::{Block, Paragraph, Wrap},
        };

        let area: Rect = center(frame_area, FALLBACK_WIDTH, FALLBACK_HEIGHT);
        let colors: (Color, Color) = dimension_colors(frame_area);

        let message: Vec<Line> = vec![
            Line::styled("Terminal is too small!", Style::default().fg(COLOR_RED)).centered(),
            Line::from(""),
            Line::from(vec![
                Span::styled("Required: ", Style::default().fg(TEXT_PRIMARY)),
                Span::styled(
                    format!("{}", MIN_WIDTH),
                    Style::default().fg(COLOR_RED).add_modifier(Modifier::BOLD),
                ),
                Span::raw(" x "),
                Span::styled(
                    format!("{}", MIN_HEIGHT),
                    Style::default().fg(COLOR_RED).add_modifier(Modifier::BOLD),
                ),
            ])
            .centered(),
            Line::from(""),
            Line::from(vec![
                Span::raw("Current:  "),
                Span::styled(
                    format!("{}", frame_area.width),
                    Style::default().fg(colors.0).add_modifier(Modifier::BOLD),
                ),
                Span::raw(" x "),
                Span::styled(
                    format!("{}", frame_area.height),
                    Style::default().fg(colors.1).add_modifier(Modifier::BOLD),
                ),
            ])
            .centered(),
        ];

        let paragraph: Paragraph = Paragraph::new(Text::from(message))
            .style(Style::default().fg(TEXT_PRIMARY))
            .wrap(Wrap { trim: false })
            .block(Block::default());

        frame.render_widget(Clear, frame_area);
        frame.render_widget(paragraph, area);
    }
}
