use crate::{
    core::ApplicationMode,
    enums::FocusArea,
    models::{Sort, Todo},
    state::{AdaptiveScroll, UIState},
    theme::ThemeColors,
    traits::{Input, InteractableEnum},
    ui::{FeedbackKind, FeedbackWidget, scrollable},
};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Style, Stylize},
    text::Line,
    widgets::{Block, Cell, Paragraph, Row, TableState, Widget, Wrap},
};

/// List widget for tasks
pub struct ListTasks<'a> {
    ui: &'a UIState,
    todos: Vec<&'a Todo>,
    query: &'a str,
    sort: &'a Sort,
    mode: &'a ApplicationMode,
    theme: &'a ThemeColors,
}

impl<'a> ListTasks<'a> {
    pub fn new(
        ui: &'a UIState,
        todos: Vec<&'a Todo>,
        query: &'a str,
        sort: &'a Sort,
        mode: &'a ApplicationMode,
        theme: &'a ThemeColors,
    ) -> Self {
        Self {
            ui,
            theme,
            todos,
            sort,
            mode,
            query,
        }
    }

    /// List rendering
    pub fn render(
        &self,
        frame: &mut Frame,
        area: Rect,
        select_state: &mut TableState,
        scroll: &AdaptiveScroll,
    ) {
        use ratatui::text::Span;

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
            .title(" Tasks ".bold())
            .title_top(
                Line::styled(
                    " todo-tui ",
                    Style::default().fg(self.theme.text_primary).bold(),
                )
                .right_aligned(),
            )
            .title_bottom(
                Line::from(vec![
                    Span::styled(
                        " Sort: ",
                        Style::default().fg(self.theme.text_primary).bold(),
                    ),
                    Span::styled(
                        self.sort.parameter.label(),
                        Style::default().fg(self.theme.accent).bold(),
                    ),
                    Span::styled(
                        format!(" {} ", self.sort.order.icon()),
                        Style::default().fg(self.theme.warning).bold(),
                    ),
                ])
                .right_aligned(),
            )
            .border_style(focused_style);

        let inner_tasks_area: Rect = main_block.inner(tasks_area);
        frame.render_widget(main_block, tasks_area);

        if !self.todos.is_empty() {
            self.build_table(frame, inner_tasks_area, select_state, focused_style);
            self.render_description(frame, desc_area, select_state, scroll);
        } else {
            FeedbackWidget::new(FeedbackKind::NoResults(self.query.to_string()), self.theme)
                .render(tasks_area, frame.buffer_mut());
        }
    }

    /// Render description for selected task with scroll
    fn render_description(
        &self,
        frame: &mut Frame,
        area: Rect,
        select_state: &mut TableState,
        scroll: &AdaptiveScroll,
    ) {
        if let Some(selected_index) = select_state.selected() {
            if let Some(todo) = self.todos.get(selected_index) {
                let content = todo.description.lines().map(Line::from).collect::<Vec<_>>();

                let desc_block: Block = Block::bordered()
                    .title(format!(" Description: {} ", todo.title))
                    .border_style(Style::default().fg(self.theme.border));

                scrollable(
                    frame,
                    area,
                    desc_block,
                    scroll,
                    &content,
                    false,
                    Style::default().fg(self.theme.border),
                    |f, rect| {
                        let p = Paragraph::new(content.clone())
                            .wrap(Wrap { trim: false })
                            .scroll((scroll.current.get(), 0))
                            .style(Style::default().fg(self.theme.text_primary));
                        f.render_widget(p, rect);
                    },
                );
            }
        }
    }

    /// Helper method to build table
    fn build_table(
        &self,
        frame: &mut Frame,
        area: Rect,
        select_state: &mut TableState,
        focused: Style,
    ) {
        use ratatui::{
            style::Color,
            text::Text,
            widgets::{Cell, Row, Table},
        };

        let [_, table_area] = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(2), Constraint::Min(0)])
            .areas(area);

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
                Cell::from(Line::from(todo.priority.to_string()).centered())
                    .style(Style::default().fg(priority_color)),
                Cell::from(Line::from(todo.time_ago()).centered())
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

        let total_rows = self.todos.len();
        let current_selected = select_state.selected().unwrap_or(0);

        let temp_scroll = AdaptiveScroll::default();
        temp_scroll.current.set(current_selected as u16);
        let dummy_content = vec![Line::from(""); total_rows];

        scrollable(
            frame,
            table_area,
            Block::default(),
            &temp_scroll,
            &dummy_content,
            true,
            focused,
            |f, rect| {
                f.render_stateful_widget(tasks_table, rect, select_state);
            },
        );
    }

    /// Calculate main layout for TaskList (list + details w/dynamic search)
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
            Cell::from(Line::from(" Title ").centered()),
            Cell::from(Line::from(" Priority ").centered()),
            Cell::from(Line::from(" Created ").centered()),
        ])
        .style(Style::default().fg(self.theme.accent).bold())
        .bottom_margin(1)
    }

    /// Highlight title if satisfies query string
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
}
