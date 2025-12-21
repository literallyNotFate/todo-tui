use ratatui::{
    Frame,
    widgets::{ListState, Widget},
};

use crate::app::models::todo::Todo;

pub struct TodoList {}

// Todo list methods implementation
impl TodoList {
    // Rendering
    pub fn render(frame: &mut Frame, todos: &[Todo], select_state: &mut ListState) {
        use super::utils::{generate_stateful_list, lines_based_on_list};
        use ratatui::{
            layout::{Constraint, Layout},
            style::{Color, Stylize},
            text::Line,
            widgets::{Block, BorderType, List, Padding},
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

        let titles: (Line, Line) = lines_based_on_list();

        let list_block: Block = Block::bordered()
            .border_type(BorderType::Rounded)
            .title(titles.0)
            .title_bottom(titles.1)
            .padding(Padding::uniform(1));

        let list_widget: List = generate_stateful_list(todos, list_block, ">");
        frame.render_stateful_widget(list_widget, inner_layout, select_state);
    }
}
