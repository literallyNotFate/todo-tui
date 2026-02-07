use crate::{
    enums::{ApplicationMode, FocusArea},
    models::Todo,
    state::{AdaptiveScroll, UIState},
    theme::ThemeColors,
    traits::{Input, InteractableEnum},
    ui::{FeedbackKind, FeedbackWidget},
};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Style, Stylize},
    text::Line,
    widgets::{
        Block, Cell, Paragraph, Row, Scrollbar, ScrollbarOrientation, ScrollbarState, TableState,
        Widget,
    },
};

pub struct ListTasks<'a> {
    ui: &'a UIState<'a>,
    todos: &'a [Todo],
    query: &'a str,
    mode: &'a ApplicationMode,
    theme: &'a ThemeColors,
}

impl<'a> ListTasks<'a> {
    pub fn new(
        ui: &'a UIState,
        todos: &'a [Todo],
        query: &'a str,
        mode: &'a ApplicationMode,
        theme: &'a ThemeColors,
    ) -> Self {
        Self {
            ui,
            theme,
            todos,
            mode,
            query,
        }
    }

    // Rendering
    pub fn render(
        &self,
        frame: &mut Frame,
        area: Rect,
        select_state: &mut TableState,
        scroll: &mut AdaptiveScroll,
    ) {
        let focused_style: Style = self.ui.focused_on(&FocusArea::MainContent);
        let is_search_visible: bool =
            *self.mode == ApplicationMode::Search || !self.query.is_empty();

        let [search_area, tasks_area, desc_area] =
            self.calculate_main_layout(area, is_search_visible, !self.todos.is_empty());

        if is_search_visible {
            if let Some(input) = self.ui.search_input.as_ref() {
                input.render(
                    frame,
                    search_area,
                    *self.mode == ApplicationMode::Search,
                    self.theme,
                );
            }
        }

        let main_block: Block = Block::bordered()
            .title(" Tasks ")
            .border_style(focused_style);

        let inner_tasks_area: Rect = main_block.inner(tasks_area);
        frame.render_widget(main_block, tasks_area);

        if !self.todos.is_empty() {
            self.build_table(frame, inner_tasks_area, select_state);
            scroll.max_scroll =
                self.max_scroll_for_description(frame, desc_area, select_state, scroll);
        } else {
            FeedbackWidget::new(FeedbackKind::NoResults(self.query.to_string()), self.theme)
                .render(tasks_area, frame.buffer_mut());
        }
    }

    // Helper method to build table
    fn build_table(&self, frame: &mut Frame, area: Rect, select_state: &mut TableState) {
        use ratatui::{
            style::Color,
            text::Text,
            widgets::{Cell, Row, Table},
        };

        let [_, table_layout] = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(2), // Top margin table
                Constraint::Min(0),    // Table
            ])
            .areas(area);

        let [table, scrollbar] = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Min(0), Constraint::Length(3)])
            .areas(table_layout);

        let rows = self.todos.iter().map(|todo| {
            let priority_color: Color = todo.priority.color(self.theme);
            let (icon, icon_color): (&str, Color) = if todo.completed {
                ("✓", self.theme.success)
            } else {
                ("☐", priority_color)
            };

            let title_content = if !self.query.is_empty() {
                self.highlight_search(&todo.title, self.query)
            } else {
                Line::from(todo.title.as_str())
            };

            Row::new(vec![
                Cell::from(icon).style(Style::default().fg(icon_color)),
                Cell::from(title_content).style(Style::default().fg(self.theme.text_primary)),
                Cell::from(Line::from(todo.priority.to_string()).alignment(Alignment::Center))
                    .style(Style::default().fg(priority_color)),
                Cell::from(Line::from(todo.time_ago()).alignment(Alignment::Center))
                    .style(Style::default().fg(self.theme.text_dim)),
            ])
            .height(1)
        });

        let tasks_table: Table = Table::new(rows, self.table_measurements())
            .header(self.table_header())
            .row_highlight_style(Style::default().bg(self.theme.surface))
            .highlight_symbol(Text::styled(
                ">>   ",
                Style::default().fg(self.theme.accent),
            ));

        self.render_scrollbar_if_needed(frame, scrollbar, select_state);
        frame.render_stateful_widget(tasks_table, table, select_state);
    }

    // Calculate main layout for TaskList (list + details w/dynamic search)
    fn calculate_main_layout(&self, area: Rect, show_search: bool, has_results: bool) -> [Rect; 3] {
        let search_constraint: Constraint = if show_search {
            Constraint::Length(3)
        } else {
            Constraint::Length(0)
        };

        let details_constraint: Constraint = if has_results {
            Constraint::Length(10)
        } else {
            Constraint::Length(0)
        };

        Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                search_constraint,  // Search?
                Constraint::Min(0), // Table / Empty state message
                details_constraint, // Details?
            ])
            .areas(area)
    }

    fn table_measurements(&self) -> [Constraint; 5] {
        [
            Constraint::Length(5),  // Status
            Constraint::Min(15),    // Title
            Constraint::Length(10), // Priority
            Constraint::Length(15), // Created At
            Constraint::Length(1),  // Space for scrollbar
        ]
    }

    fn table_header(&self) -> Row<'static> {
        Row::new(vec![
            Cell::from(""),
            Cell::from(Line::from(" Title ").alignment(Alignment::Center)),
            Cell::from(Line::from(" Priority ").alignment(Alignment::Center)),
            Cell::from(Line::from(" Created ").alignment(Alignment::Center)),
        ])
        .style(Style::default().fg(self.theme.accent).bold())
        .bottom_margin(1)
    }

    // Highlight title if satisfies query string
    fn highlight_search(&self, title: &'a str, query: &str) -> Line<'a> {
        use ratatui::text::Span;

        let query_lower: String = query.to_lowercase();
        let title_lower: String = title.to_lowercase();

        if let Some(start) = title_lower.find(&query_lower) {
            let end = start + query.len();
            Line::from(vec![
                Span::raw(&title[..start]),
                Span::styled(
                    &title[start..end],
                    Style::default().fg(self.theme.accent).bold(),
                ),
                Span::raw(&title[end..]),
            ])
        } else {
            Line::from(title)
        }
    }

    // Render description for selected task with scroll
    fn max_scroll_for_description(
        &self,
        frame: &mut Frame,
        area: Rect,
        select_state: &mut TableState,
        scroll: &AdaptiveScroll,
    ) -> u16 {
        use ratatui::widgets::Wrap;

        if let Some(selected_index) = select_state.selected() {
            if let Some(todo) = self.todos.get(selected_index) {
                let description: &str = todo.description.as_str();

                let inner_width: u16 = area.width.saturating_sub(2);
                let inner_height: u16 = area.height.saturating_sub(2);

                let wrapped_lines: usize = textwrap::wrap(description, inner_width as usize).len();
                let max_scroll: u16 = wrapped_lines.saturating_sub(inner_height as usize) as u16;

                let effective_scroll: u16 = scroll.current.min(max_scroll);

                let desc_block: Block = Block::bordered()
                    .title(format!(" Description: {} ", todo.title))
                    .border_style(Style::default().fg(self.theme.border));

                let desc: Paragraph = Paragraph::new(description)
                    .block(desc_block)
                    .style(Style::default().fg(self.theme.text_primary))
                    .wrap(Wrap { trim: true })
                    .scroll((effective_scroll, 0));

                frame.render_widget(desc, area);

                if wrapped_lines > inner_height as usize {
                    let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
                        .begin_symbol(Some("↑"))
                        .end_symbol(Some("↓"))
                        .track_symbol(Some("│"))
                        .thumb_symbol("▉")
                        .thumb_style(Style::default().fg(self.theme.border))
                        .track_style(Style::default().fg(self.theme.border))
                        .begin_style(Style::default().fg(self.theme.border))
                        .end_style(Style::default().fg(self.theme.border));

                    let mut scrollbar_state = ScrollbarState::new(max_scroll as usize)
                        .position(effective_scroll as usize);

                    frame.render_stateful_widget(scrollbar, area, &mut scrollbar_state);
                }

                return max_scroll;
            }
        }

        0
    }

    // Dynamic scroll for list
    fn render_scrollbar_if_needed(
        &self,
        frame: &mut Frame,
        area: Rect,
        select_state: &mut TableState,
    ) {
        let visible_height: usize = area.height.saturating_sub(2) as usize;

        if self.todos.len() > visible_height {
            let scrollbar = Scrollbar::default()
                .orientation(ScrollbarOrientation::VerticalRight)
                .begin_symbol(Some("↑"))
                .end_symbol(Some("↓"))
                .track_symbol(Some("│"))
                .thumb_symbol("▉")
                .thumb_style(Style::default().fg(self.theme.accent))
                .track_style(Style::default().fg(self.theme.border))
                .begin_style(Style::default().fg(self.theme.accent))
                .end_style(Style::default().fg(self.theme.accent));

            let mut scrollbar_state: ScrollbarState = ScrollbarState::new(self.todos.len())
                .position(select_state.selected().unwrap_or(0));

            frame.render_stateful_widget(scrollbar, area, &mut scrollbar_state);
        }
    }
}
