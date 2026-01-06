use crate::app::{models::todo::Todo, utils::colors::theme::*};
use ratatui::{
    Frame,
    widgets::{ListState, Widget},
};

pub struct TodoList;

// Todo list methods implementation
impl TodoList {
    // Rendering
    pub fn render(frame: &mut Frame, todos: &[Todo], select_state: &mut ListState) {
        use super::utils::{generate_stateful_list, lines_based_on_list};
        use ratatui::{
            layout::{Constraint, Layout},
            style::Stylize,
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
            .bg(BG_PRIMARY)
            .padding(Padding::uniform(2))
            .fg(COLOR_ORANGE)
            .render(main_layout, frame.buffer_mut());

        let (top_line, bottom_center, bottom_left): (Line, Line, Line) =
            lines_based_on_list(todos, select_state.selected());

        let list_block: Block = Block::bordered()
            .border_type(BorderType::Rounded)
            .title_top(top_line)
            .title_bottom(bottom_center)
            .title_bottom(bottom_left)
            .padding(Padding::uniform(1));

        let list_widget: List = generate_stateful_list(todos, list_block, ">");
        frame.render_stateful_widget(list_widget, inner_layout, select_state);
    }
}
