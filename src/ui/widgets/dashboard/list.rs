use crate::{
    core::{Action, ApplicationMode, FocusArea, Sort},
    models::{Folder, FolderColor, Task},
    state::{AdaptiveScroll, UIState},
    theme::ThemePalette,
    ui::{FeedbackKind, FeedbackWidget, RenderContext, scrollable, widgets::input::Input},
};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Cell, Row, TableState},
};
use std::collections::HashMap;
use uuid::Uuid;

/// List widget for tasks
pub struct ListTasks<'a> {
    ui: &'a UIState,
    tasks: Vec<&'a Task>,
    folders_map: &'a HashMap<Uuid, Folder>,
    query: &'a str,
    sort: &'a Sort,
}

impl<'a> ListTasks<'a> {
    pub fn new(
        ui: &'a UIState,
        tasks: Vec<&'a Task>,
        folders_map: &'a HashMap<Uuid, Folder>,
        query: &'a str,
        sort: &'a Sort,
    ) -> Self {
        Self {
            ui,
            tasks,
            folders_map,
            sort,
            query,
        }
    }

    /// List rendering
    pub fn render(&self, ctx: &mut RenderContext, area: Rect, select_state: &mut TableState) {
        let palette: ThemePalette = ctx.palette();
        let mode: ApplicationMode = ctx.mode();
        let focus_area: FocusArea = FocusArea::Main;
        let is_focused: bool = ctx.is_focused(focus_area);

        let is_search_visible: bool = mode == ApplicationMode::Search || !self.query.is_empty();
        let [search_area, tasks_area] = self.calculate_main_layout(area, is_search_visible);

        if is_search_visible {
            if let Some(input) = self.ui.search_input.as_ref() {
                input.render(ctx, search_area, mode == ApplicationMode::Search);
            }
        }

        let filter_name: &str = if let Some(id) = self.ui.active_folder {
            self.folders_map
                .get(&id)
                .map(|f| f.name.as_str())
                .unwrap_or("?")
        } else {
            &self.ui.active_tab.to_string()
        };

        let auto_label: &str = ctx
            .config
            .task
            .auto_sort
            .then(|| " [auto-sorted]")
            .unwrap_or("");

        let main_block = Block::bordered()
            .title(format!(" Tasks: {} {} ", filter_name.bold(), auto_label))
            .title_top(
                Line::styled(
                    " toodles ",
                    Style::default()
                        .fg(ctx.focused_color(palette.fg, focus_area))
                        .bold(),
                )
                .right_aligned(),
            )
            .title_bottom(Line::from(self.render_hotkeys(ctx)).left_aligned())
            .title_bottom(
                Line::from(vec![
                    Span::styled(
                        " Sort: ",
                        Style::default()
                            .fg(ctx.focused_color(palette.fg, focus_area))
                            .bold(),
                    ),
                    Span::styled(
                        format!("{}", self.sort.parameter),
                        Style::default()
                            .fg(ctx.focused_color(palette.accent, focus_area))
                            .bold(),
                    ),
                    Span::styled(
                        format!(" {} ", self.sort.order),
                        Style::default()
                            .fg(ctx.focused_color(palette.warning, focus_area))
                            .bold(),
                    ),
                ])
                .right_aligned(),
            )
            .border_type(ctx.config.ui.border_type.into())
            .border_style(ctx.focused_color(palette.accent, focus_area))
            .bg(palette.bg);

        let inner_tasks_area: Rect = main_block.inner(tasks_area);
        ctx.render_widget(main_block, tasks_area);

        if !self.tasks.is_empty() {
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
            text::Text,
            widgets::{Cell, Row, Table},
        };

        let palette = ctx.palette();
        let focus_area = FocusArea::Main;
        let max_config_width: usize = ctx.config.task.max_title_length as usize;
        let title_width: usize = (area.width as usize).min(max_config_width);
        let query_lower: String = self.query.to_lowercase();

        let [_, table_area] = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(2), Constraint::Min(0)])
            .areas(area);

        let rows = self.tasks.iter().map(|task| {
            let (icon, color) = task.get_display_info(&ctx.config.ui, &palette);
            let folder_span = task
                .folder_id
                .and_then(|id| self.folders_map.get(&id))
                .map(|f| {
                    Span::styled(
                        format!("[{}] ", f.name),
                        Style::default().fg(FolderColor::from_string(&f.color)),
                    )
                })
                .unwrap_or_default();

            let truncated_title = RenderContext::truncate(&task.title, title_width);
            let title_spans =
                self.highlight_search(&truncated_title, &query_lower, &palette, ctx.is_dimmed);
            let display_date: String = task.format_date(ctx.config.ui.use_24h);
            let final_title = ctx.config.task.build(task, title_spans, folder_span);

            let cells = [
                Cell::from(icon).style(Style::default().fg(ctx.focused_color(color, focus_area))),
                Cell::from(final_title),
                Cell::from(Line::from(task.priority.as_str()).centered()).style(
                    Style::default()
                        .fg(ctx.focused_color(task.priority.palette(&palette), focus_area)),
                ),
                Cell::from(Line::from(display_date).centered())
                    .style(Style::default().fg(palette.muted)),
            ];
            Row::new(cells).height(1)
        });

        let tasks_table = Table::new(rows, self.table_measurements())
            .header(self.table_header(focused, &palette, ctx.is_dimmed))
            .bg(palette.bg)
            .row_highlight_style(Style::default().bg(palette.selection))
            .highlight_symbol(Text::styled(
                format!("{}  ", ctx.config.ui.symbols.selection),
                Style::default().fg(ctx.focused_color(palette.secondary, focus_area)),
            ));

        let temp_scroll = AdaptiveScroll::default();
        temp_scroll
            .current
            .set(select_state.selected().unwrap_or(0) as u16);

        scrollable(
            ctx,
            table_area,
            Block::default(),
            &temp_scroll,
            &vec![Line::from(""); self.tasks.len()],
            true,
            Style::default().fg(ctx.focused_color(palette.accent, focus_area)),
            |f, rect| f.render_stateful_widget(tasks_table, rect, select_state),
        );
    }

    /// Highlight title if satisfies query string
    fn highlight_search(
        &self,
        title: &str,
        query: &str,
        palette: &ThemePalette,
        is_dimmed: bool,
    ) -> Vec<Span<'static>> {
        let highlight_color = if is_dimmed {
            palette.muted
        } else {
            palette.warning
        };

        if let Some(start) = title.to_lowercase().find(&query.to_lowercase()) {
            let end = start + query.len();
            vec![
                Span::raw(title[..start].to_string()),
                Span::styled(
                    title[start..end].to_string(),
                    Style::default().fg(highlight_color).bold(),
                ),
                Span::raw(title[end..].to_string()),
            ]
        } else {
            vec![Span::raw(title.to_string())]
        }
    }

    /// Render hotkeys for list using preconfigured keymaps
    fn render_hotkeys(&self, ctx: &mut RenderContext) -> Line<'_> {
        let mut spans = Vec::new();
        let palette: ThemePalette = ctx.palette();

        match ctx.focus() {
            FocusArea::Sidebar => {
                spans.extend(ctx.key_hint(Action::ShowHelp, "help", palette.accent));
                spans.extend(ctx.key_hint(Action::MoveRight, "list", palette.secondary));
                spans.extend(ctx.key_hint(Action::ToggleSidebar, "focus", palette.success));
                spans.extend(ctx.key_hint(Action::Quit, "exit", palette.error));
            }
            FocusArea::Main => {
                spans.extend(ctx.key_hint(Action::MoveLeft, "sidebar", palette.accent));
                spans.extend(ctx.key_hint(Action::AddTask, "add", palette.secondary));
                spans.extend(ctx.key_hint(Action::UpdateTask, "edit", palette.warning));
                spans.extend(ctx.key_hint(Action::RemoveTask, "remove", palette.error));
                spans.extend(ctx.key_hint(Action::Complete, "done", palette.success));
                spans.extend(ctx.key_hint(Action::ShowDetails, "details", palette.info));
            }
        }

        Line::from(spans).centered()
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

    fn table_header(&self, focused: bool, palette: &ThemePalette, is_dimmed: bool) -> Row<'static> {
        let base_color = if focused { palette.info } else { palette.muted };
        let header_color = if is_dimmed { palette.muted } else { base_color };

        Row::new(vec![
            Cell::from(""),
            Cell::from(Line::from(" Title ").centered()),
            Cell::from(Line::from(" Priority ").centered()),
            Cell::from(Line::from(" Created ").centered()),
        ])
        .style(Style::default().fg(header_color).bold())
        .bottom_margin(1)
    }
}
