use crate::{
    enums::FocusArea,
    models::{Filter, Todo},
    state::UIState,
    theme::ThemeColors,
    traits::InteractableEnum,
};
use ratatui::{Frame, layout::Rect};

pub struct MenuSidebar;

impl MenuSidebar {
    pub fn render(
        frame: &mut Frame,
        area: Rect,
        ui: &UIState,
        theme: &ThemeColors,
        todos: &[Todo],
    ) {
        use ratatui::{
            layout::{Constraint, Direction, Layout},
            style::{Style, Stylize},
            text::{Line, Span},
            widgets::{Block, List, ListItem, ListState, Paragraph},
        };

        let sidebar_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(0),    // Filters
                Constraint::Length(8), // Summary
            ])
            .split(area);

        let is_focused = ui.focus_area == FocusArea::LeftPanel;
        let filters_block: Block = Block::bordered()
            .title(" Filters ")
            .border_style(if is_focused {
                Style::default().fg(theme.accent)
            } else {
                Style::default().fg(theme.border)
            })
            .bg(theme.bg_dim);

        let filters_inner_area: Rect = filters_block.inner(sidebar_layout[0]);

        let filters_content_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1), // Margin
                Constraint::Min(0),    // Filter list
            ])
            .split(filters_inner_area);

        let items: Vec<ListItem> = Filter::all_variants()
            .iter()
            .map(|tab| {
                let count = tab.count(todos);
                let text = format!(" {} ({})", tab.to_string(), count);
                let style = if *tab == ui.current_filter {
                    Style::default().fg(theme.accent).bold()
                } else {
                    Style::default().fg(theme.text_primary)
                };
                ListItem::new(Span::styled(text, style))
            })
            .collect();

        let list = List::new(items)
            .highlight_style(Style::default().bg(theme.surface))
            .highlight_symbol("→ ");

        let mut state = ListState::default();
        state.select(Some(ui.current_filter.index()));

        frame.render_widget(filters_block, sidebar_layout[0]);
        frame.render_stateful_widget(list, filters_content_layout[1], &mut state);

        let info_block = Block::bordered()
            .title(" Summary ")
            .border_style(Style::default().fg(theme.border))
            .bg(theme.bg_dim);

        let info_inner_area = info_block.inner(sidebar_layout[1]);
        let info_content_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1), // Summary margin
                Constraint::Min(0),    // Summary
            ])
            .split(info_inner_area);

        let (total, completed) = (todos.len(), todos.iter().filter(|t| t.completed).count());
        let percent = if total > 0 {
            (completed as f32 / total as f32 * 100.0) as u8
        } else {
            0
        };
        let filled = (percent as f32 / 10.0).round() as usize;
        let gauge = format!(" [{}{}] ", "■".repeat(filled), "□".repeat(10 - filled));

        let info_text = vec![
            Line::from(vec![
                Span::styled(" Theme: ", Style::default().fg(theme.text_dim)),
                Span::styled(theme.name, Style::default().fg(theme.accent).bold()),
            ]),
            Line::from(vec![
                Span::styled(" Progress: ", Style::default().fg(theme.text_dim)),
                Span::styled(
                    format!("{}%", percent),
                    Style::default().fg(theme.success).bold(),
                ),
            ]),
            Line::from(Span::styled(gauge, Style::default().fg(theme.success))),
        ];

        frame.render_widget(info_block, sidebar_layout[1]);
        frame.render_widget(Paragraph::new(info_text), info_content_layout[1]);
    }
}
