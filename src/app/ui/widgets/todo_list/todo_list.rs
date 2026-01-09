use crate::app::{models::todo::Todo, utils::constants::theme::*};
use ratatui::{
    Frame,
    widgets::{ListState, Widget},
};

pub struct TodoList;

impl TodoList {
    pub fn render(frame: &mut Frame, todos: &[Todo], select_state: &mut ListState) {
        use super::utils::*;
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
            .fg(ITEM_LIST_PRIMARY)
            .render(main_layout, frame.buffer_mut());

        let (top_line, bottom_center, bottom_left): (Line, Line, Line) =
            render_lines_based_on_list(todos, select_state.selected());

        let list_block: Block = Block::bordered()
            .border_type(BorderType::Rounded)
            .title_top(top_line)
            .title_bottom(bottom_center)
            .title_bottom(bottom_left)
            .padding(Padding::uniform(1));

        let list_widget: List = generate_stateful_list(todos, list_block, ">").scroll_padding(0);

        frame.render_stateful_widget(list_widget, inner_layout, select_state);

        let visible_lines: usize = inner_layout.height.saturating_sub(3) as usize;
        let content_lines: usize = todos.len();

        render_scrollbar_if_needed(
            frame,
            inner_layout,
            content_lines,
            visible_lines,
            select_state.offset(),
        );
    }
}
