// Unit-tests for TodoList widget
#[cfg(test)]
mod tests {
    use crate::app::{
        models::todo::Todo,
        ui::widgets::todo_list::utils::{render_lines_based_on_list, render_scrollbar_if_needed},
        utils::constants::theme::*,
    };
    use ratatui::{Terminal, backend::TestBackend, buffer::Buffer, layout::Rect, text::Line};

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

    #[test]
    fn should_render_scrollbar_if_needed_no_scroll() {
        let backend: TestBackend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.clear().unwrap();

        let area: Rect = Rect::new(0, 0, 80, 24);

        terminal
            .draw(|frame| {
                render_scrollbar_if_needed(frame, area, 10, 20, 0);
            })
            .unwrap();

        let buffer: Buffer = terminal.backend().buffer().clone();

        let right_column = buffer.area.right() - 1;
        let has_scrollbar_symbols = (0..buffer.area.height).any(|y| {
            let cell = buffer[(right_column, y)].clone();
            "↑↓│▉".chars().any(|ch| cell.symbol().contains(ch))
        });

        assert!(
            !has_scrollbar_symbols,
            "Scrollbar should not be rendered when not needed"
        );
    }

    #[test]
    fn should_render_scrollbar_if_needed_with_scroll() {
        let backend: TestBackend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.clear().unwrap();

        let area: Rect = Rect::new(0, 0, 80, 24);

        let content_lines: usize = 50;
        let visible_lines: usize = 20;
        let offset: usize = 15;

        terminal
            .draw(|frame| {
                render_scrollbar_if_needed(frame, area, content_lines, visible_lines, offset);
            })
            .unwrap();

        let buffer: Buffer = terminal.backend().buffer().clone();

        let right_column = buffer.area.right() - 1;
        let has_thumb = (0..buffer.area.height).any(|y| buffer[(right_column, y)].symbol() == "▉");

        let has_track = (0..buffer.area.height).any(|y| buffer[(right_column, y)].symbol() == "│");

        assert!(has_thumb, "Scroll bar thumb symbol should be present");
        assert!(
            has_track || has_thumb,
            "At least track or thumb should be visible"
        );
    }

    #[test]
    fn should_test_scrollbar_position_at_end() {
        let backend: TestBackend = TestBackend::new(80, 10);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.clear().unwrap();

        let area: Rect = Rect::new(0, 0, 80, 10);

        let content_lines: usize = 100;
        let visible_lines: usize = 8;
        let offset: usize = content_lines - visible_lines;

        terminal
            .draw(|frame| {
                render_scrollbar_if_needed(frame, area, content_lines, visible_lines, offset);
            })
            .unwrap();

        let buffer: Buffer = terminal.backend().buffer().clone();
        let right_column = buffer.area.right() - 1;

        let thumb_positions: Vec<u16> = (0..buffer.area.height)
            .filter(|&y| buffer[(right_column, y)].symbol() == "▉")
            .collect();

        assert!(
            !thumb_positions.is_empty(),
            "Thumb should be visible at the end"
        );
        let lowest_thumb = *thumb_positions.iter().max().unwrap();
        assert!(
            lowest_thumb >= buffer.area.height - 3,
            "Thumb should be near the bottom"
        );
    }
}
