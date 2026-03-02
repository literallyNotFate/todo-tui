use crate::{
    core::ApplicationMode,
    enums::FocusArea,
    models::{Sort, Todo},
    state::{AdaptiveScroll, UIState},
    theme::ThemePalette,
    traits::{Input, InteractableEnum},
    ui::{FeedbackKind, FeedbackWidget, RenderContext, scrollable, utils},
};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Cell, Row, TableState},
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
    pub fn render(&self, ctx: &mut RenderContext, area: Rect, select_state: &mut TableState) {
        let palette: ThemePalette = ctx.palette();
        let mode: ApplicationMode = ctx.mode();
        let focus_area: FocusArea = FocusArea::MainContent;
        let is_focused: bool = ctx.is_focused(focus_area);

        let is_search_visible: bool = mode == ApplicationMode::Search || !self.query.is_empty();
        let [search_area, tasks_area] = self.calculate_main_layout(area, is_search_visible);

        if is_search_visible {
            if let Some(input) = self.ui.search_input.as_ref() {
                input.render(ctx, search_area, mode == ApplicationMode::Search);
            }
        }

        let main_block: Block = Block::bordered()
            .title(format!(" Tasks: ({}) ", ctx.filter()).bold())
            .title_top(
                Line::styled(
                    " todo-tui ",
                    Style::default()
                        .fg(ctx.focused_color(palette.fg, focus_area))
                        .bold(),
                )
                .right_aligned(),
            )
            .title_bottom(
                Line::from(vec![
                    Span::styled(
                        " Sort: ",
                        Style::default()
                            .fg(ctx.focused_color(palette.fg, focus_area))
                            .bold(),
                    ),
                    Span::styled(
                        self.sort.parameter.label(),
                        Style::default()
                            .fg(ctx.focused_color(palette.accent, focus_area))
                            .bold(),
                    ),
                    Span::styled(
                        format!(" {} ", self.sort.order.icon()),
                        Style::default()
                            .fg(ctx.focused_color(palette.warning, focus_area))
                            .bold(),
                    ),
                ])
                .right_aligned(),
            )
            .border_type(ctx.config.border_type.into())
            .border_style(ctx.focused_color(palette.accent, focus_area))
            .bg(palette.bg);

        let inner_tasks_area: Rect = main_block.inner(tasks_area);
        ctx.render_widget(main_block, tasks_area);

        if !self.todos.is_empty() {
            self.build_table(ctx, inner_tasks_area, select_state, is_focused);
        } else {
            FeedbackWidget::new(FeedbackKind::NoResults(self.query.to_string()))
                .render(ctx, tasks_area);
        }
    }

    /// Helper method to build table
    fn build_table(
        &self,
        ctx: &mut RenderContext,
        area: Rect,
        select_state: &mut TableState,
        focused: bool,
    ) {
        use ratatui::{
            style::Color,
            text::Text,
            widgets::{Cell, Row, Table},
        };

        let palette: ThemePalette = ctx.palette();
        let title_column_width: usize = (area.width as usize).saturating_sub(40);
        let focus_area: FocusArea = FocusArea::MainContent;

        let [_, table_area] = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(2), Constraint::Min(0)])
            .areas(area);

        let rows = self.todos.iter().map(|todo| {
            let priority_color: Color = todo.priority.palette(&palette);
            let (icon, icon_color): (String, Color) = if todo.completed {
                (
                    ctx.config.symbols.completed.clone(),
                    ctx.focused_color(palette.success, focus_area),
                )
            } else {
                (
                    ctx.config.symbols.pending.clone(),
                    ctx.focused_color(palette.success, focus_area),
                )
            };

            let truncated_title = utils::truncate(&todo.title, title_column_width);

            let title_content = if !self.query.is_empty() {
                self.highlight_search(&truncated_title, self.query, &palette)
            } else {
                Line::from(truncated_title)
            };

            let display_date: String =
                if todo.created_at.date_naive() == chrono::Local::now().date_naive() {
                    let format_str: &str = if ctx.config.use_24h {
                        "%H:%M"
                    } else {
                        "%I:%M %p"
                    };

                    todo.created_at
                        .with_timezone(&chrono::Local)
                        .format(format_str)
                        .to_string()
                } else {
                    todo.created_at
                        .with_timezone(&chrono::Local)
                        .format("%d %b")
                        .to_string()
                };

            Row::new(vec![
                Cell::from(icon).style(Style::default().fg(icon_color)),
                Cell::from(title_content)
                    .style(Style::default().fg(ctx.focused_color(palette.fg, focus_area))),
                Cell::from(Line::from(todo.priority.to_string()).centered())
                    .style(Style::default().fg(ctx.focused_color(priority_color, focus_area))),
                Cell::from(Line::from(display_date).centered())
                    .style(Style::default().fg(palette.muted)),
            ])
            .height(1)
        });

        let tasks_table: Table = Table::new(rows, self.table_measurements())
            .header(self.table_header(focused, &palette))
            .bg(palette.bg)
            .row_highlight_style(Style::default().bg(palette.selection))
            .highlight_symbol(Text::styled(
                format!("{}   ", ctx.config.symbols.selection),
                Style::default().fg(ctx.focused_color(palette.secondary, focus_area)),
            ));

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
            Style::default().fg(ctx.focused_color(palette.accent, focus_area)),
            |f, rect| {
                f.render_stateful_widget(tasks_table, rect, select_state);
            },
        );
    }

    /// Highlight title if satisfies query string
    fn highlight_search(&self, title: &str, query: &str, palette: &ThemePalette) -> Line<'static> {
        let query_lower: String = query.to_lowercase();
        let title_lower: String = title.to_lowercase();

        if let Some(start) = title_lower.find(&query_lower) {
            let end = start + query.len();
            Line::from(vec![
                Span::raw(title[..start].to_string()),
                Span::styled(
                    title[start..end].to_string(),
                    Style::default().fg(palette.warning).bold(),
                ),
                Span::raw(title[end..].to_string()),
            ])
        } else {
            Line::from(title.to_string())
        }
    }

    /// Calculate main layout for TaskList (list + details w/dynamic search)
    fn calculate_main_layout(&self, area: Rect, show_search: bool) -> [Rect; 2] {
        let search_constraint: Constraint = if show_search {
            Constraint::Length(3)
        } else {
            Constraint::Length(0)
        };

        Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                search_constraint,  // Search?
                Constraint::Min(0), // Table / Empty state message
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

    fn table_header(&self, focused: bool, palette: &ThemePalette) -> Row<'static> {
        Row::new(vec![
            Cell::from(""),
            Cell::from(Line::from(" Title ").centered()),
            Cell::from(Line::from(" Priority ").centered()),
            Cell::from(Line::from(" Created ").centered()),
        ])
        .style(
            Style::default()
                .fg(if focused { palette.info } else { palette.muted })
                .bold(),
        )
        .bottom_margin(1)
    }
}
