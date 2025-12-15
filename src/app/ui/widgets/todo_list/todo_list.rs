use ratatui::{
    Frame,
    widgets::{ListState, Widget},
};

use crate::app::models::todo::Todo;

pub struct TodoList {}

impl TodoList {
    pub fn new() -> Self {
        Self {}
    }

    pub fn render(&self, frame: &mut Frame, todos: &[Todo], select_state: &mut ListState) {
        use ratatui::{
            layout::{Constraint, Layout},
            style::{Color, Style, Stylize},
            text::Line,
            widgets::{Block, BorderType, List, ListItem, Padding},
        };

        let [main_layout] = Layout::vertical([Constraint::Fill(1)])
            .margin(1)
            .areas(frame.area());

        let [inner_layout] = Layout::vertical([Constraint::Fill(1)])
            .margin(3)
            .areas(main_layout);

        Block::default()
            .padding(Padding::uniform(2))
            .fg(Color::Rgb(230, 185, 157))
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
