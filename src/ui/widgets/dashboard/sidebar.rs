use crate::{
    core::FocusArea,
    models::{Filter, Priority, Todo},
    state::UIState,
    theme::ThemePalette,
    ui::RenderContext,
};
use chrono::{Datelike, Local};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::List,
};
use strum::IntoEnumIterator;

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
        let focus_area: FocusArea = FocusArea::Sidebar;
        let is_dimmed: bool = ctx.is_dimmed;

        let sidebar_layout: std::rc::Rc<[Rect]> = self.layout(area);

        let system_block: Block = ctx.block("System", None).bg(palette.bg2);
        let system_inner_area: Rect = system_block.inner(sidebar_layout[0]);
        let system_text: Vec<Line> = self.system_text(&palette, is_dimmed);

        ctx.render_widget(system_block, sidebar_layout[0]);
        ctx.render_widget(Paragraph::new(system_text), system_inner_area);

        let filters_block: Block = ctx.block("Filters", Some(focus_area)).bg(palette.bg2);
        let filters_inner_area: Rect = filters_block.inner(sidebar_layout[1]);
        let filter_tab_layout: std::rc::Rc<[Rect]> = self.filters_tab_layout(filters_inner_area);

        let query: &str = self.ui.search_query();
        let list: List =
            self.construct_list(&query, ctx.is_focused(focus_area), &palette, is_dimmed);

        let mut state: ListState = ListState::default();
        state.select(Some(self.ui.filter.index()));

        ctx.render_widget(filters_block, sidebar_layout[1]);
        ctx.render_stateful_widget(list, filter_tab_layout[1], &mut state);

        let progress_block: Block = ctx.block("Progress", None).bg(palette.bg2);
        let progress_inner_area: Rect = progress_block.inner(sidebar_layout[2]);
        let progress_text: Vec<Line> = self.progress_text(&palette, is_dimmed);

        ctx.render_widget(progress_block, sidebar_layout[2]);
        ctx.render_widget(Paragraph::new(progress_text), progress_inner_area);

        let focus_block: Block = ctx.block("Focus", None).bg(palette.bg2);
        let focus_inner_area: Rect = focus_block.inner(sidebar_layout[3]);
        let focus_text: Vec<Line> = self.focus_text(focus_inner_area.width, &palette, is_dimmed);

        ctx.render_widget(focus_block, sidebar_layout[3]);
        ctx.render_widget(Paragraph::new(focus_text), focus_inner_area);

        let chart_block: Block = ctx.block("Priority Chart", None).bg(palette.bg2);
        let chart_inner_area: Rect = chart_block.inner(sidebar_layout[4]);
        let chart_text: Vec<Line> =
            self.priority_chart_text(self.todos, &palette, chart_inner_area.width, is_dimmed);

        ctx.render_widget(chart_block, sidebar_layout[4]);
        ctx.render_widget(Paragraph::new(chart_text), chart_inner_area);
    }

    /// Construct a list based on filtered todo values
    fn construct_list(
        &self,
        query: &str,
        focused: bool,
        palette: &ThemePalette,
        is_dimmed: bool,
    ) -> List<'static> {
        use ratatui::widgets::ListItem;

        let items: Vec<ListItem> = Filter::iter()
            .map(|tab| {
                let count = tab.count(self.todos, query);
                let text = format!(" {} ({})", tab.to_string(), count);

                let style = if tab == self.ui.filter.value {
                    if is_dimmed {
                        Style::default().fg(palette.muted).bold()
                    } else {
                        Style::default().fg(palette.bg).bold()
                    }
                } else {
                    let fg: Color = if is_dimmed || !focused {
                        palette.muted
                    } else {
                        palette.fg
                    };
                    Style::default().fg(fg)
                };

                ListItem::new(Span::styled(text, style))
            })
            .collect();

        let highlight_bg: Color = if is_dimmed {
            palette.bg
        } else if focused {
            palette.info
        } else {
            palette.muted
        };

        List::new(items)
            .highlight_style(Style::default().bg(highlight_bg))
            .highlight_symbol(if is_dimmed { "  " } else { "→ " })
    }

    /// Focus text
    fn focus_text(&self, width: u16, palette: &ThemePalette, is_dimmed: bool) -> Vec<Line<'_>> {
        let max_title_width = (width as usize).saturating_sub(12);
        let color = |c: Color| if is_dimmed { palette.muted } else { c };

        let focus_text = if let Some(todo) = self
            .todos
            .iter()
            .find(|t| !t.completed && t.priority == Priority::High)
        {
            Line::from(vec![
                Span::styled(
                    " ⊙ Focus: ",
                    Style::default().fg(color(palette.accent)).bold(),
                ),
                Span::styled(
                    RenderContext::truncate(&todo.title, max_title_width),
                    Style::default().fg(if is_dimmed { palette.muted } else { palette.fg }),
                ),
            ])
        } else if let Some(todo) = self.todos.iter().find(|t| !t.completed) {
            Line::from(vec![
                Span::styled(" ◆ Next: ", Style::default().fg(color(palette.secondary))),
                Span::styled(
                    RenderContext::truncate(&todo.title, max_title_width),
                    Style::default().fg(palette.muted),
                ),
            ])
        } else {
            Line::from(vec![
                Span::styled(" ✓ ", Style::default().fg(color(palette.success)).bold()),
                Span::styled("All tasks completed", Style::default().fg(palette.muted)),
            ])
        };

        vec![Line::from(""), focus_text]
    }

    /// Get progress text
    fn progress_text(&self, palette: &ThemePalette, is_dimmed: bool) -> Vec<Line<'static>> {
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
        let gauge_color: Color = if is_dimmed {
            palette.muted
        } else {
            palette.success
        };

        vec![
            Line::from(""),
            Line::from(vec![
                Span::styled(" Completion: ", Style::default().fg(palette.muted)),
                Span::styled(
                    format!("{}%", percent),
                    Style::default().fg(gauge_color).bold(),
                ),
            ]),
            Line::from(vec![
                Span::styled(gauge, Style::default().fg(gauge_color)),
                Span::styled(" (", Style::default().fg(palette.muted)),
                Span::styled(format!("{}", completed), Style::default().fg(gauge_color)),
                Span::styled(" / ", Style::default().fg(palette.muted)),
                Span::styled(
                    format!("{}", total),
                    Style::default()
                        .fg(if is_dimmed {
                            palette.muted
                        } else {
                            palette.error
                        })
                        .bold(),
                ),
                Span::styled(")", Style::default().fg(palette.muted)),
            ]),
        ]
    }

    /// Get system date text
    fn system_text(&self, palette: &ThemePalette, is_dimmed: bool) -> Vec<Line<'static>> {
        let now = Local::now();
        let current_day = now.weekday().number_from_monday() as usize;

        let active_bg: Color = if is_dimmed {
            palette.muted
        } else {
            palette.accent
        };
        let active_fg: Color = if is_dimmed { palette.fg } else { palette.bg };

        let calendar_spans = ["M", "T", "W", "T", "F", "S", "S"]
            .iter()
            .enumerate()
            .map(|(i, name)| {
                if i + 1 == current_day {
                    Span::styled(
                        format!(" {} ", name),
                        Style::default().bg(active_bg).fg(active_fg).bold(),
                    )
                } else {
                    Span::styled(format!(" {} ", name), Style::default().fg(palette.muted))
                }
            })
            .collect::<Vec<_>>();

        vec![
            Line::from(""),
            Line::from(vec![Span::raw(now.format("%A, %d %b").to_string())]).centered(),
            Line::from(""),
            Line::from(calendar_spans).centered(),
        ]
    }

    /// Get priority horizontal chart
    fn priority_chart_text(
        &self,
        todos: &[Todo],
        palette: &ThemePalette,
        width: u16,
        is_dimmed: bool,
    ) -> Vec<Line<'static>> {
        let (mut high, mut med, mut low) = (0, 0, 0);
        for t in todos {
            match t.priority {
                Priority::High => high += 1,
                Priority::Medium => med += 1,
                Priority::Low => low += 1,
            }
        }

        let total = (high + med + low).max(1);
        let bar_max_width = (width as usize).saturating_sub(15).max(10);

        let mut lines = Vec::new();
        lines.push(Line::from(""));

        let priorities = [
            ("HIGH", high, palette.error),
            ("MED ", med, palette.warning),
            ("LOW ", low, palette.success),
        ];

        for (label, count, base_color) in priorities {
            let active_color: Color = if is_dimmed { palette.muted } else { base_color };
            let count_color: Color = if is_dimmed { palette.muted } else { palette.fg };

            let filled_len: usize = (count * bar_max_width) / total;
            let empty_len: usize = bar_max_width.saturating_sub(filled_len);

            let f_sym: &str = "━";
            let e_sym: &str = "─";

            lines.push(Line::from(vec![
                Span::styled(
                    format!(" {:<5} ", label),
                    Style::default().fg(active_color).bold(),
                ),
                Span::styled(f_sym.repeat(filled_len), Style::default().fg(active_color)),
                Span::styled(e_sym.repeat(empty_len), Style::default().fg(palette.muted)),
                Span::styled(
                    format!(" {:>2}", count),
                    Style::default().fg(count_color).bold(),
                ),
            ]));
        }

        lines
    }

    /// Layout for sidebar
    fn layout(&self, area: Rect) -> std::rc::Rc<[Rect]> {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(6), // System
                Constraint::Fill(1),   // Filters
                Constraint::Length(5), // Progress
                Constraint::Length(4), // Focus
                Constraint::Length(6), // Chart
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
}

/// Unit-tests for sidebar
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_generate_progress_with_todos() {
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

        let summary: Vec<Line> = sidebar.progress_text(&ui.theme.palette(), false);
        let line_text: String = summary[1].to_string();
        assert!(line_text.contains("50%"));

        let gauge_text: String = summary[2].to_string();
        assert!(gauge_text.contains("■■■■■□□□□□"));
    }

    #[test]
    fn should_generate_progress_with_empty_todos() {
        let todos: Vec<Todo> = vec![];

        let ui = UIState::default();
        let sidebar: SidebarWidget = SidebarWidget::new(&ui, &todos);

        let summary: Vec<Line> = sidebar.progress_text(&ui.theme.palette(), false);
        assert!(summary[1].to_string().contains("0%"));
        assert!(summary[2].to_string().contains("□□□□□□□□□□"));
    }

    #[test]
    fn should_construct_list_with_highlighting() {
        let todos = vec![];

        let mut ui = UIState::default();
        ui.filter.value = Filter::All;
        let sidebar: SidebarWidget = SidebarWidget::new(&ui, &todos);

        let list: List = sidebar.construct_list("", true, &ui.theme.palette(), false);
        assert_eq!(list.len(), Filter::iter().len());
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
        let summary = sidebar.focus_text(50, &ui.theme.palette(), false);

        let focus_text = summary[1].to_string();
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
        let summary = sidebar.focus_text(50, &ui.theme.palette(), false);

        let focus_text = summary[1].to_string();
        assert!(focus_text.contains("◆ Next:"));
        assert!(focus_text.contains("Only Task"));
    }

    #[test]
    fn should_test_priority_chart_math() {
        let ui = UIState::default();
        let todos = vec![
            Todo {
                title: "H".to_string(),
                priority: Priority::High,
                completed: false,
                ..Default::default()
            },
            Todo {
                title: "M".to_string(),
                priority: Priority::Medium,
                completed: false,
                ..Default::default()
            },
            Todo {
                title: "L".to_string(),
                priority: Priority::Low,
                completed: false,
                ..Default::default()
            },
            Todo {
                title: "L2".to_string(),
                priority: Priority::Low,
                completed: false,
                ..Default::default()
            },
        ];

        let sidebar = SidebarWidget::new(&ui, &todos);
        let lines = sidebar.priority_chart_text(&todos, &ui.theme.palette(), 40, false);

        let high_line = lines[1].to_string();
        let low_line = lines[3].to_string();

        assert!(high_line.contains("HIGH"));
        assert!(high_line.contains(" 1"));
        assert!(low_line.contains(" 2"));
    }
}
