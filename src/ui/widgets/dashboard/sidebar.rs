use crate::{
    enums::FocusArea,
    models::{Filter, Priority, Todo},
    state::UIState,
    theme::ThemePalette,
    traits::InteractableEnum,
    ui::{RenderContext, scrollable, utils},
};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{List, Wrap},
};

/// Sidebar widget
pub struct SidebarWidget<'a> {
    ui: &'a UIState,
    todos: &'a [Todo],
}

impl<'a> SidebarWidget<'a> {
    pub fn new(ui: &'a UIState, todos: &'a [Todo]) -> Self {
        Self { ui, todos }
    }

    /// Sidebar rendering
    pub fn render(&self, ctx: &mut RenderContext, area: Rect) {
        use ratatui::widgets::{Block, ListState, Paragraph};

        let palette: ThemePalette = ctx.palette();
        let focus_area: FocusArea = FocusArea::LeftPanel;

        let sidebar_layout: std::rc::Rc<[Rect]> = self.layout(area);

        let filters_block: Block = ctx.block("Filters", focus_area).bg(palette.bg2);
        let filters_inner_area: Rect = filters_block.inner(sidebar_layout[0]);
        let filter_tab_layout: std::rc::Rc<[Rect]> = self.filters_tab_layout(filters_inner_area);

        let query: &str = self.ui.search_query();
        let list: List = self.construct_list(&query, ctx.is_focused(focus_area), &palette);

        let mut state: ListState = ListState::default();
        state.select(Some(self.ui.current_filter.index()));

        ctx.render_widget(filters_block, sidebar_layout[0]);
        ctx.render_stateful_widget(list, filter_tab_layout[1], &mut state);

        let summary_block: Block = ctx.static_block("Summary").bg(palette.bg2);
        let summary_inner_area: Rect = summary_block.inner(sidebar_layout[1]);
        let summary_inner_layout: std::rc::Rc<[Rect]> = self.summary_layout(summary_inner_area);
        let summary_text: Vec<Line> = self.summary_text(summary_inner_area.width, &palette);

        ctx.render_widget(summary_block, sidebar_layout[1]);
        ctx.render_widget(Paragraph::new(summary_text), summary_inner_layout[1]);

        let hotkeys_block = Block::bordered()
            .title(" Hotkeys ")
            .border_style(Style::default().fg(palette.muted))
            .border_type(ctx.config.border_type.into())
            .bg(palette.bg2);

        let mut hotkeys_lines = ctx.hotkeys();
        hotkeys_lines.insert(0, Line::from(""));

        scrollable(
            ctx,
            sidebar_layout[2],
            hotkeys_block,
            &self.ui.sidebar_scroll,
            &hotkeys_lines,
            false,
            Style::default().fg(palette.muted),
            |f, rect| {
                let p = Paragraph::new(hotkeys_lines.clone())
                    .wrap(Wrap { trim: false })
                    .scroll((self.ui.sidebar_scroll.current.get(), 0))
                    .style(Style::default().fg(palette.fg));
                f.render_widget(p, rect);
            },
        );
    }

    /// Construct a list based on filtered todo values
    fn construct_list(&self, query: &str, focused: bool, palette: &ThemePalette) -> List<'static> {
        use ratatui::widgets::ListItem;

        let items: Vec<ListItem> = Filter::all()
            .iter()
            .map(|tab| {
                let count = tab.count(self.todos, query);
                let text = format!(" {} ({})", tab.to_string(), count);
                let style = if *tab == self.ui.current_filter {
                    Style::default().fg(Color::Rgb(0, 0, 0)).bold()
                } else {
                    Style::default().fg(if focused { palette.fg } else { palette.muted })
                };

                ListItem::new(Span::styled(text, style))
            })
            .collect();

        List::new(items)
            .highlight_style(Style::default().bg(if focused {
                palette.info
            } else {
                palette.muted
            }))
            .highlight_symbol("→ ")
    }

    /// Get summary text
    fn summary_text(&self, width: u16, palette: &ThemePalette) -> Vec<Line<'static>> {
        let max_title_width = (width as usize).saturating_sub(12);

        let focus_line = if let Some(todo) = self
            .todos
            .iter()
            .find(|t| !t.completed && t.priority == Priority::High)
        {
            Line::from(vec![
                Span::styled(" ⊙ Focus: ", Style::default().fg(palette.accent).bold()),
                Span::styled(
                    utils::truncate(&todo.title, max_title_width),
                    Style::default().fg(palette.fg),
                ),
            ])
        } else if let Some(todo) = self.todos.iter().find(|t| !t.completed) {
            Line::from(vec![
                Span::styled(" ◆ Next: ", Style::default().fg(palette.secondary)),
                Span::styled(
                    utils::truncate(&todo.title, max_title_width),
                    Style::default().fg(palette.muted),
                ),
            ])
        } else {
            Line::from(vec![
                Span::styled(" ✓ ", Style::default().fg(palette.success).bold()),
                Span::styled("All tasks completed", Style::default().fg(palette.muted)),
            ])
        };

        let (total, completed): (usize, usize) = (
            self.todos.len(),
            self.todos.iter().filter(|t| t.completed).count(),
        );

        let percent: u8 = if total > 0 {
            (completed as f32 / total as f32 * 100.0) as u8
        } else {
            0
        };

        let filled: usize = (percent as f32 / 10.0).round() as usize;
        let gauge: String = format!(" [{}{}] ", "■".repeat(filled), "□".repeat(10 - filled));

        vec![
            focus_line,
            Line::from(""),
            Line::from(vec![
                Span::styled(" Progress: ", Style::default().fg(palette.muted)),
                Span::styled(
                    format!("{}%", percent),
                    Style::default().fg(palette.success).bold(),
                ),
            ]),
            Line::from(Span::styled(gauge, Style::default().fg(palette.success))),
        ]
    }

    /// Layout for sidebar
    fn layout(&self, area: Rect) -> std::rc::Rc<[Rect]> {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Fill(1),   // Filters
                Constraint::Length(8), // Summary
                Constraint::Max(16),   // Hotkeys
            ])
            .split(area)
    }

    /// Layout for filters tab
    fn filters_tab_layout(&self, area: Rect) -> std::rc::Rc<[Rect]> {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1), // Margin
                Constraint::Min(0),    // Filter list
            ])
            .split(area)
    }

    /// Layout for summary
    fn summary_layout(&self, area: Rect) -> std::rc::Rc<[Rect]> {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1), // Summary margin
                Constraint::Min(0),    // Summary
            ])
            .split(area)
    }
}

/// Unit-tests for sidebar
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_make_summary_for_todos_with_progress() {
        let todos = vec![
            Todo {
                completed: true,
                ..Default::default()
            },
            Todo {
                completed: false,
                ..Default::default()
            },
        ];

        let ui = UIState::default();
        let sidebar: SidebarWidget = SidebarWidget::new(&ui, &todos);

        let summary: Vec<Line> = sidebar.summary_text(50, &ui.theme.palette());
        let line_text: String = summary[2].to_string();
        assert!(line_text.contains("50%"));

        let gauge_text: String = summary[3].to_string();
        assert!(gauge_text.contains("■■■■■□□□□□"));
    }

    #[test]
    fn should_make_summary_for_empty_todos() {
        let todos: Vec<Todo> = vec![];

        let ui = UIState::default();
        let sidebar: SidebarWidget = SidebarWidget::new(&ui, &todos);

        let summary: Vec<Line> = sidebar.summary_text(50, &ui.theme.palette());
        assert!(summary[0].to_string().contains("All tasks completed"));
        assert!(summary[2].to_string().contains("0%"));
        assert!(summary[3].to_string().contains("□□□□□□□□□□"));
    }

    #[test]
    fn should_construct_list_with_highlighting() {
        let todos = vec![];

        let mut ui = UIState::default();
        ui.current_filter = Filter::All;
        let sidebar: SidebarWidget = SidebarWidget::new(&ui, &todos);

        let list: List = sidebar.construct_list("", true, &ui.theme.palette());
        assert_eq!(list.len(), Filter::all().len());
    }

    #[test]
    fn should_prioritize_high_priority_focus_task() {
        let todos = vec![
            Todo {
                title: "Normal Task".to_string(),
                priority: Priority::Low,
                completed: false,
                ..Default::default()
            },
            Todo {
                title: "Urgent Task".to_string(),
                priority: Priority::High,
                completed: false,
                ..Default::default()
            },
        ];

        let ui = UIState::default();
        let sidebar = SidebarWidget::new(&ui, &todos);
        let summary = sidebar.summary_text(50, &ui.theme.palette());

        let focus_text = summary[0].to_string();
        assert!(focus_text.contains("⊙ Focus:"));
        assert!(focus_text.contains("Urgent Task"));
        assert!(!focus_text.contains("Normal Task"));
    }

    #[test]
    fn should_show_next_task_if_no_high_priority() {
        let todos = vec![Todo {
            title: "Only Task".to_string(),
            priority: Priority::Low,
            completed: false,
            ..Default::default()
        }];

        let ui = UIState::default();
        let sidebar = SidebarWidget::new(&ui, &todos);
        let summary = sidebar.summary_text(50, &ui.theme.palette());

        let focus_text = summary[0].to_string();
        assert!(focus_text.contains("◆ Next:"));
        assert!(focus_text.contains("Only Task"));
    }
}
