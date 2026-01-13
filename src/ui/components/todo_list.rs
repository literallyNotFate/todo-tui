use crate::{
    models::Todo,
    utils::widgets::todo_list::{
        generate_stateful_list, render_lines_based_on_list, render_scrollbar_if_needed,
    },
};
use ratatui::{
    Frame,
    text::Line,
    widgets::{ListState, Widget},
};

pub struct TodoList;

impl TodoList {
    pub fn render(frame: &mut Frame, todos: &[Todo], select_state: &mut ListState) {
        use crate::utils::constants::theme::{BG_PRIMARY, ITEM_LIST_PRIMARY};
        use ratatui::{
            layout::{Constraint, Layout},
            style::Stylize,
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

// Unit-tests for TodoList widget
#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::constants::theme::COLOR_GREEN;

    #[test]
    fn should_generate_lines_based_on_empty_list() {
        let todos: Vec<Todo> = vec![];
        let selected: Option<usize> = None;
        let (top, bottom_center, bottom_left): (Line, Line, Line) =
            render_lines_based_on_list(&todos, selected);

        assert_eq!(top.spans[0].content, " List of what's to complete ");
        assert_eq!(bottom_center.spans.len(), 2);
        assert_eq!(bottom_center.spans[0].content, " Help");
        assert_eq!(bottom_center.spans[1].content, " <?> ");
        assert_eq!(bottom_left, Line::default());
    }

    #[test]
    fn should_generate_lines_based_on_item_list_with_no_selection() {
        let todos: Vec<Todo> = vec![
            Todo {
                title: "Task 1".to_string(),
                done: false,
            },
            Todo {
                title: "Task 2".to_string(),
                done: true,
            },
        ];
        let selected: Option<usize> = None;
        let (_, _, bottom_left): (Line, Line, Line) = render_lines_based_on_list(&todos, selected);

        assert_eq!(bottom_left, Line::default());
    }

    #[test]
    fn should_generate_lines_based_on_item_list_with_selection() {
        let todos: Vec<Todo> = vec![
            Todo {
                title: "Uncompleted".to_string(),
                done: false,
            },
            Todo {
                title: "Completed".to_string(),
                done: true,
            },
        ];
        let selected: Option<usize> = Some(1);
        let (_, _, bottom_left): (Line, Line, Line) = render_lines_based_on_list(&todos, selected);

        assert_eq!(bottom_left.spans[0].content, " 2 / 2 ");
        assert_eq!(bottom_left.spans[2].content, "Done ");
        assert_eq!(bottom_left.spans[2].style.fg, Some(COLOR_GREEN));
    }

    #[test]
    fn should_generate_lines_based_on_item_list_with_out_of_bounds() {
        let todos: Vec<Todo> = vec![Todo {
            title: "Test".to_string(),
            done: false,
        }];
        let selected: Option<usize> = Some(999);
        let (_, _, bottom_left): (Line, Line, Line) = render_lines_based_on_list(&todos, selected);

        assert_eq!(bottom_left.spans[0].content, " 1 / 1 ");
        assert_eq!(bottom_left.spans[2].content, "Undone ");
    }
}
