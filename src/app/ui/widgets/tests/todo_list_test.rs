// Unit-tests for TodoList widget
#[cfg(test)]
mod tests {
    use crate::app::{
        models::todo::Todo, ui::widgets::todo_list::utils::lines_based_on_list,
        utils::constants::theme::*,
    };
    use ratatui::text::Line;

    #[test]
    fn should_generate_lines_based_on_empty_list() {
        let todos: Vec<Todo> = vec![];
        let selected: Option<usize> = None;
        let (top, bottom_center, bottom_left): (Line, Line, Line) =
            lines_based_on_list(&todos, selected);

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
        let (_, _, bottom_left): (Line, Line, Line) = lines_based_on_list(&todos, selected);

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
        let (_, _, bottom_left): (Line, Line, Line) = lines_based_on_list(&todos, selected);

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
        let (_, _, bottom_left): (Line, Line, Line) = lines_based_on_list(&todos, selected);

        assert_eq!(bottom_left.spans[0].content, " 1 / 1 ");
        assert_eq!(bottom_left.spans[2].content, "Undone ");
    }
}
