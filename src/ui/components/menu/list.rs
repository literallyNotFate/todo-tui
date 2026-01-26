use crate::{
    enums::FocusArea,
    models::{Priority, Todo},
    state::UIState,
    traits::InteractableEnum,
    ui::center,
    utils::constants::theme::TEXT_DIMMED,
};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::Line,
    widgets::{Block, Paragraph, TableState},
};

pub struct TaskList;

impl TaskList {
    pub fn render(
        frame: &mut Frame,
        area: Rect,
        ui: &UIState,
        select_state: &mut TableState,
        todos: &[Todo],
    ) {
        use ratatui::{
            style::{Color, Style, Stylize},
            widgets::{Cell, Row, Scrollbar, ScrollbarOrientation, ScrollbarState, Table, Wrap},
        };

        let focused_style: Style = if ui.focus_area == FocusArea::MainContent {
            Style::default().fg(Color::Green)
        } else {
            Style::default()
        };

        let filtered = ui.current_filter.filter(todos);
        if filtered.is_empty() {
            Self::render_empty_state(frame, area, focused_style);
            return;
        }

        let main_layout: std::rc::Rc<[Rect]> = Self::main_layout(area);

        let main_block = Block::bordered()
            .title(" Tasks ")
            .border_style(focused_style);

        let inner_area: Rect = main_block.inner(main_layout[0]);
        frame.render_widget(main_block, main_layout[0]);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(2), // Top margin table
                Constraint::Min(0),    // Table
            ])
            .split(inner_area);

        let table_area = chunks[1];
        let details_area = main_layout[1];

        if let Some(selected_index) = select_state.selected() {
            if let Some(todo) = filtered.get(selected_index) {
                let details_block = Block::bordered()
                    .title(format!(" Details: {} ", todo.title))
                    .border_style(Style::default().fg(Color::Cyan).dim());

                let details: Paragraph = Paragraph::new(todo.description.as_str())
                    .block(details_block)
                    .wrap(Wrap { trim: true });

                frame.render_widget(details, details_area);
            }
        }

        let rows = filtered.iter().map(|todo| {
            let priority_color: Color = match todo.priority {
                Priority::High => Color::LightRed,
                Priority::Medium => Color::LightYellow,
                Priority::Low => Color::LightGreen,
            };

            Row::new(vec![
                Cell::from(if todo.completed { "✓" } else { "☐" }).style(Style::default().fg(
                    if todo.completed {
                        Color::Green
                    } else {
                        priority_color
                    },
                )),
                Cell::from(todo.title.as_str()),
                Cell::from(Line::from(todo.priority.to_string()).alignment(Alignment::Center))
                    .style(Style::default().fg(priority_color)),
                Cell::from(Line::from(todo.time_ago()).alignment(Alignment::Center)),
            ])
            .height(1)
        });

        let table = Table::new(rows, Self::table_measurements())
            .header(
                Row::new(vec![
                    Cell::from(""),
                    Cell::from(Line::from(" Title ").alignment(Alignment::Center)),
                    Cell::from(Line::from(" Priority ").alignment(Alignment::Center)),
                    Cell::from(Line::from(" Created ").alignment(Alignment::Center)),
                ])
                .style(Style::default().bold())
                .bottom_margin(1),
            )
            .row_highlight_style(Style::default().bg(Color::DarkGray))
            .highlight_symbol(">> ");

        let table_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Min(0), Constraint::Length(3)])
            .split(table_area);

        let final_table_area = table_layout[0];
        let scrollbar_area = table_layout[1];

        let scrollbar = Scrollbar::default()
            .orientation(ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("↑"))
            .end_symbol(Some("↓"))
            .track_symbol(Some("│"));

        let mut scrollbar_state =
            ScrollbarState::new(filtered.len()).position(select_state.selected().unwrap_or(0));

        frame.render_stateful_widget(scrollbar, scrollbar_area, &mut scrollbar_state);
        frame.render_stateful_widget(table, final_table_area, select_state);
    }

    // Render fallback if list is empty
    fn render_empty_state(frame: &mut Frame, area: Rect, focused_style: Style) {
        let outer_block: Block = Block::bordered()
            .title(" Tasks ")
            .border_style(focused_style);
        frame.render_widget(outer_block, area);

        let message_area: Rect = center(50, 20, area);

        let message = vec![
            Line::from("All clear!").style(Style::default().add_modifier(Modifier::BOLD)),
            Line::from(""),
            Line::from("Press 'a' to add a new task").style(Style::default().fg(TEXT_DIMMED)),
        ];

        let paragraph = Paragraph::new(message).alignment(Alignment::Center);
        frame.render_widget(paragraph, message_area);
    }

    // Layout methods
    fn main_layout(area: Rect) -> std::rc::Rc<[Rect]> {
        Layout::default()
            .constraints(vec![Constraint::Min(0), Constraint::Length(10)])
            .split(area)
    }

    fn table_measurements() -> [Constraint; 5] {
        [
            Constraint::Length(5),  // Status
            Constraint::Min(15),    // Title
            Constraint::Length(10), // Priority
            Constraint::Length(15), // Created At
            Constraint::Length(1),  // Space for scrollbar
        ]
    }
}
