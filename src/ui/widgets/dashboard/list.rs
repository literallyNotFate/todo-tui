use crate::{
    core::ApplicationMode,
    enums::FocusArea,
    models::{Sort, Todo},
    state::{AdaptiveScroll, UIState},
    theme::ThemeColors,
    traits::{Input, InteractableEnum},
    ui::{FeedbackKind, FeedbackWidget, RenderContext, scrollable, utils},
};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Style, Stylize},
    text::Line,
    widgets::{Block, Cell, Paragraph, Row, TableState, Wrap},
};

/// List widget for tasks
pub struct ListTasks<'a> {
    ui: &'a UIState,
    todos: Vec<&'a Todo>,
    query: &'a str,
    sort: &'a Sort,
}

impl<'a> ListTasks<'a> {
    pub fn new(ui: &'a UIState, todos: Vec<&'a Todo>, query: &'a str, sort: &'a Sort) -> Self {
        Self {
            ui,
            todos,
            sort,
            query,
        }
    }

    /// List rendering
    pub fn render(
        &self,
        ctx: &mut RenderContext,
        area: Rect,
        select_state: &mut TableState,
        scroll: &AdaptiveScroll,
    ) {
        use ratatui::text::Span;

        let theme = ctx.theme;
        let mode = ctx.mode();

        let focused_style: Style = ctx.focused_style(FocusArea::MainContent);
        let is_search_visible: bool = mode == ApplicationMode::Search || !self.query.is_empty();

        let [search_area, tasks_area, desc_area] =
            self.calculate_main_layout(area, is_search_visible, !self.todos.is_empty());

        if is_search_visible {
            if let Some(input) = self.ui.search_input.as_ref() {
                input.render(ctx, search_area, mode == ApplicationMode::Search);
            }
        }

        let main_block: Block = Block::bordered()
            .title(" Tasks ".bold())
            .title_top(
                Line::styled(" todo-tui ", Style::default().fg(theme.text_primary).bold())
                    .right_aligned(),
            )
            .title_bottom(
                Line::from(vec![
                    Span::styled(" Sort: ", Style::default().fg(theme.text_primary).bold()),
                    Span::styled(
                        self.sort.parameter.label(),
                        Style::default().fg(theme.accent).bold(),
                    ),
                    Span::styled(
                        format!(" {} ", self.sort.order.icon()),
                        Style::default().fg(theme.warning).bold(),
                    ),
                ])
                .right_aligned(),
            )
            .border_style(focused_style);

        let inner_tasks_area: Rect = main_block.inner(tasks_area);
        ctx.render_widget(main_block, tasks_area);

        if !self.todos.is_empty() {
            self.build_table(ctx, inner_tasks_area, select_state, focused_style);
            self.render_description(ctx, desc_area, select_state, scroll);
        } else {
            FeedbackWidget::new(FeedbackKind::NoResults(self.query.to_string()))
                .render(ctx, tasks_area);
        }
    }

    /// Render description for selected task with scroll
    fn render_description(
        &self,
        ctx: &mut RenderContext,
        area: Rect,
        select_state: &mut TableState,
        scroll: &AdaptiveScroll,
    ) {
        if let Some(selected_index) = select_state.selected() {
            if let Some(todo) = self.todos.get(selected_index) {
                let theme = ctx.theme;
                let content = todo.description.lines().map(Line::from).collect::<Vec<_>>();

                let desc_block: Block = Block::bordered()
                    .title(format!(
                        " Description: {} ",
                        utils::truncate(&todo.title, area.width.saturating_sub(20) as usize)
                    ))
                    .border_style(Style::default().fg(theme.border));

                scrollable(
                    ctx,
                    area,
                    desc_block,
                    scroll,
                    &content,
                    false,
                    Style::default().fg(theme.border),
                    |f, rect| {
                        let p = Paragraph::new(content.clone())
                            .wrap(Wrap { trim: false })
                            .scroll((scroll.current.get(), 0))
                            .style(Style::default().fg(theme.text_primary));
                        f.render_widget(p, rect);
                    },
                );
            }
        }
    }

    /// Helper method to build table
    fn build_table(
        &self,
        ctx: &mut RenderContext,
        area: Rect,
        select_state: &mut TableState,
        focused: Style,
    ) {
        use ratatui::{
            style::Color,
            text::Text,
            widgets::{Cell, Row, Table},
        };

        let theme = ctx.theme;
        let title_column_width = (area.width as usize).saturating_sub(40);

        let [_, table_area] = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(2), Constraint::Min(0)])
            .areas(area);

        let rows = self.todos.iter().map(|todo| {
            let priority_color: Color = todo.priority.color(&theme);
            let (icon, icon_color): (&str, Color) = if todo.completed {
                ("✓", theme.success)
            } else {
                ("☐", priority_color)
            };

            let truncated_title = utils::truncate(&todo.title, title_column_width);

            let title_content = if !self.query.is_empty() {
                self.highlight_search(&truncated_title, self.query, &theme)
            } else {
                Line::from(truncated_title)
            };

            Row::new(vec![
                Cell::from(icon).style(Style::default().fg(icon_color)),
                Cell::from(title_content).style(Style::default().fg(theme.text_primary)),
                Cell::from(Line::from(todo.priority.to_string()).centered())
                    .style(Style::default().fg(priority_color)),
                Cell::from(Line::from(todo.time_ago()).centered())
                    .style(Style::default().fg(theme.text_dim)),
            ])
            .height(1)
        });

        let tasks_table: Table = Table::new(rows, self.table_measurements())
            .header(self.table_header(&theme))
            .row_highlight_style(Style::default().bg(theme.surface))
            .highlight_symbol(Text::styled(">>   ", Style::default().fg(theme.accent)));

        let total_rows = self.todos.len();
        let current_selected = select_state.selected().unwrap_or(0);

        let temp_scroll = AdaptiveScroll::default();
        temp_scroll.current.set(current_selected as u16);
        let dummy_content = vec![Line::from(""); total_rows];

        scrollable(
            ctx,
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

    fn table_header(&self, theme: &ThemeColors) -> Row<'static> {
        Row::new(vec![
            Cell::from(""),
            Cell::from(Line::from(" Title ").centered()),
            Cell::from(Line::from(" Priority ").centered()),
            Cell::from(Line::from(" Created ").centered()),
        ])
        .style(Style::default().fg(theme.accent).bold())
        .bottom_margin(1)
    }

    /// Highlight title if satisfies query string
    fn highlight_search(&self, title: &str, query: &str, theme: &ThemeColors) -> Line<'static> {
        use ratatui::text::Span;

        let query_lower: String = query.to_lowercase();
        let title_lower: String = title.to_lowercase();

        if let Some(start) = title_lower.find(&query_lower) {
            let end = start + query.len();
            Line::from(vec![
                Span::raw(title[..start].to_string()),
                Span::styled(
                    title[start..end].to_string(),
                    Style::default().fg(theme.accent).bold(),
                ),
                Span::raw(title[end..].to_string()),
            ])
        } else {
            Line::from(title.to_string())
        }
    }
}
