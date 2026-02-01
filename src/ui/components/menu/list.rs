use crate::{
    enums::FocusArea, models::Todo, state::UIState, theme::ThemeColors, traits::InteractableEnum,
    ui::center,
};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style, Stylize},
    text::Line,
    widgets::{Block, Cell, Paragraph, Row, TableState},
};

pub struct TaskList;

impl TaskList {
    pub fn render(
        frame: &mut Frame,
        area: Rect,
        ui: &UIState,
        select_state: &mut TableState,
        theme: &ThemeColors,
        todos: &[Todo],
    ) {
        use ratatui::{
            style::Color,
            text::Text,
            widgets::{Cell, Row, Table},
        };

        let focused_style: Style = ui.focused_on(&FocusArea::MainContent);

        let filtered: Vec<Todo> = ui.current_filter.filter(todos);
        if filtered.is_empty() {
            Self::render_empty_state(frame, area, focused_style, theme);
            return;
        }

        let main_layout: std::rc::Rc<[Rect]> = Self::main_layout(area);

        let main_block: Block = Block::bordered()
            .title(" Tasks ")
            .border_style(focused_style);

        let table_area: Rect = main_block.inner(main_layout[0]);
        frame.render_widget(main_block, main_layout[0]);

        let table_layout: std::rc::Rc<[Rect]> = Self::table_layout(table_area);
        let inner_table: std::rc::Rc<[Rect]> = Self::inner_table_layout(table_layout[1]);

        let rows = filtered.iter().map(|todo| {
            let priority_color: Color = todo.priority.color(theme);
            let (icon, icon_color): (&str, Color) = if todo.completed {
                ("✓", theme.success)
            } else {
                ("☐", priority_color)
            };

            Row::new(vec![
                Cell::from(icon).style(Style::default().fg(icon_color)),
                Cell::from(todo.title.as_str()).style(Style::default().fg(theme.text_primary)),
                Cell::from(Line::from(todo.priority.to_string()).alignment(Alignment::Center))
                    .style(Style::default().fg(priority_color)),
                Cell::from(Line::from(todo.time_ago()).alignment(Alignment::Center))
                    .style(Style::default().fg(theme.text_dim)),
            ])
            .height(1)
        });

        let table: Table = Table::new(rows, Self::table_measurements())
            .header(Self::table_header(theme))
            .row_highlight_style(Style::default().bg(theme.surface))
            .highlight_symbol(Text::styled(">> ", Style::default().fg(theme.accent)));

        Self::render_scrollbar_if_needed(
            frame,
            inner_table[1],
            filtered.len(),
            select_state,
            theme,
        );
        frame.render_stateful_widget(table, inner_table[0], select_state);
        Self::render_description_for_selected(frame, main_layout[1], select_state, filtered, theme);
    }

    // Render fallback if list is empty
    fn render_empty_state(
        frame: &mut Frame,
        area: Rect,
        focused_style: Style,
        theme: &ThemeColors,
    ) {
        let outer_block: Block = Block::bordered()
            .title(" Tasks ")
            .border_style(focused_style);
        frame.render_widget(outer_block, area);

        let message_area: Rect = center(50, 20, area);

        let message = vec![
            Line::from("All clear!").style(
                Style::default()
                    .fg(theme.success)
                    .add_modifier(Modifier::BOLD),
            ),
            Line::from(""),
            Line::from("Press 'a' to add a new task").style(Style::default().fg(theme.text_dim)),
        ];

        let paragraph = Paragraph::new(message).alignment(Alignment::Center);
        frame.render_widget(paragraph, message_area);
    }

    // Render description for selected task
    fn render_description_for_selected(
        frame: &mut Frame,
        area: Rect,
        select_state: &mut TableState,
        filtered: Vec<Todo>,
        theme: &ThemeColors,
    ) {
        use ratatui::widgets::Wrap;

        if let Some(selected_index) = select_state.selected() {
            if let Some(todo) = filtered.get(selected_index) {
                let details_block = Block::bordered()
                    .title(format!(" Details: {} ", todo.title))
                    .border_style(Style::default().fg(theme.border));

                let details: Paragraph = Paragraph::new(todo.description.as_str())
                    .block(details_block)
                    .wrap(Wrap { trim: true });

                frame.render_widget(details, area);
            }
        }
    }

    // Dynamic scroll for list
    pub fn render_scrollbar_if_needed(
        frame: &mut Frame,
        area: Rect,
        content_lines: usize,
        select_state: &mut TableState,
        theme: &ThemeColors,
    ) {
        use ratatui::widgets::{Scrollbar, ScrollbarOrientation, ScrollbarState};

        let visible_height: usize = area.height.saturating_sub(2) as usize;

        if content_lines > visible_height {
            let scrollbar = Scrollbar::default()
                .orientation(ScrollbarOrientation::VerticalRight)
                .begin_symbol(Some("↑"))
                .end_symbol(Some("↓"))
                .track_symbol(Some("│"))
                .thumb_symbol("▉")
                .thumb_style(Style::default().fg(theme.accent))
                .track_style(Style::default().fg(theme.border))
                .begin_style(Style::default().fg(theme.accent))
                .end_style(Style::default().fg(theme.accent));

            let mut scrollbar_state: ScrollbarState =
                ScrollbarState::new(content_lines).position(select_state.selected().unwrap_or(0));

            frame.render_stateful_widget(scrollbar, area, &mut scrollbar_state);
        }
    }

    // Layout for list (list + details)
    fn main_layout(area: Rect) -> std::rc::Rc<[Rect]> {
        Layout::default()
            .constraints(vec![Constraint::Min(0), Constraint::Length(10)])
            .split(area)
    }

    // Layout for table
    fn table_layout(area: Rect) -> std::rc::Rc<[Rect]> {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(2), // Top margin table
                Constraint::Min(0),    // Table
            ])
            .split(area)
    }

    // Inner layout for table w/scrollbar
    fn inner_table_layout(area: Rect) -> std::rc::Rc<[Rect]> {
        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Min(0), Constraint::Length(3)])
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

    fn table_header(theme: &ThemeColors) -> Row<'static> {
        Row::new(vec![
            Cell::from(""),
            Cell::from(Line::from(" Title ").alignment(Alignment::Center)),
            Cell::from(Line::from(" Priority ").alignment(Alignment::Center)),
            Cell::from(Line::from(" Created ").alignment(Alignment::Center)),
        ])
        .style(Style::default().fg(theme.accent).bold())
        .bottom_margin(1)
    }
}
