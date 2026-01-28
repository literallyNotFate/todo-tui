use crate::{
    enums::ApplicationMode,
    models::{Filter, Todo},
    state::UIState,
    theme::ThemeColors,
    traits::InteractableEnum,
};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Style, Stylize},
    text::{Line, Span},
    widgets::List,
};

pub struct MenuSidebar;

impl MenuSidebar {
    pub fn render(
        frame: &mut Frame,
        area: Rect,
        ui: &UIState,
        theme: &ThemeColors,
        todos: &[Todo],
        mode: &ApplicationMode,
    ) {
        use ratatui::{
            style::{Style, Stylize},
            text::Line,
            widgets::{Block, List, ListState, Paragraph},
        };

        let sidebar_layout: std::rc::Rc<[Rect]> = Self::layout(area);
        let focused_style: Style = ui.styles_on_focus();

        let filters_block: Block = Block::bordered()
            .title(" Filters ")
            .border_style(focused_style)
            .bg(theme.bg_dim);

        let filters_inner_area: Rect = filters_block.inner(sidebar_layout[0]);
        let filter_tab_layout: std::rc::Rc<[Rect]> = Self::filters_tab_layout(filters_inner_area);

        let list: List = Self::construct_list(todos, &ui.current_filter, theme);

        let mut state: ListState = ListState::default();
        state.select(Some(ui.current_filter.index()));

        frame.render_widget(filters_block, sidebar_layout[0]);
        frame.render_stateful_widget(list, filter_tab_layout[1], &mut state);

        let summary_block: Block = Block::bordered()
            .title(" Summary ")
            .border_style(Style::default().fg(theme.border))
            .bg(theme.bg_dim);

        let summary_inner_area: Rect = summary_block.inner(sidebar_layout[1]);
        let summary_inner_layout: std::rc::Rc<[Rect]> = Self::summary_layout(summary_inner_area);
        let summary_text: Vec<Line> = Self::summary_text(todos, theme);

        frame.render_widget(summary_block, sidebar_layout[1]);
        frame.render_widget(Paragraph::new(summary_text), summary_inner_layout[1]);

        let controls_block: Block = Block::bordered()
            .title(" Controls ")
            .border_style(Style::default().fg(theme.border))
            .bg(theme.bg_dim);

        let controls_inner_area: Rect = controls_block.inner(sidebar_layout[2]);
        let controls_layout: std::rc::Rc<[Rect]> = Self::controls_layout(controls_inner_area);

        let hotkeys: &str = Self::hotkeys(mode);

        frame.render_widget(controls_block, sidebar_layout[2]);
        frame.render_widget(
            Paragraph::new(hotkeys).style(Style::default().fg(theme.text_primary)),
            controls_layout[1],
        );
    }

    // Layout for sidebar
    fn layout(area: Rect) -> std::rc::Rc<[Rect]> {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(0),     // Filters
                Constraint::Length(7),  // Summary
                Constraint::Length(15), // Controls
            ])
            .split(area)
    }

    // Layout for filters tab
    fn filters_tab_layout(area: Rect) -> std::rc::Rc<[Rect]> {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1), // Margin
                Constraint::Min(0),    // Filter list
            ])
            .split(area)
    }

    // Layout for summary
    fn summary_layout(area: Rect) -> std::rc::Rc<[Rect]> {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1), // Summary margin
                Constraint::Min(0),    // Summary
            ])
            .split(area)
    }

    // Layout for controls
    fn controls_layout(area: Rect) -> std::rc::Rc<[Rect]> {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1), // Controls margin
                Constraint::Min(0),    // Controls
            ])
            .split(area)
    }

    // Construct a list based on filtered todo values
    fn construct_list(
        todos: &[Todo],
        current_filter: &Filter,
        theme: &ThemeColors,
    ) -> List<'static> {
        use ratatui::widgets::ListItem;

        let items: Vec<ListItem> = Filter::all_variants()
            .iter()
            .map(|tab| {
                let count = tab.count(todos);
                let text = format!(" {} ({})", tab.to_string(), count);
                let style = if tab == current_filter {
                    Style::default().fg(theme.accent).bold()
                } else {
                    Style::default().fg(theme.text_primary)
                };

                ListItem::new(Span::styled(text, style))
            })
            .collect();

        List::new(items)
            .highlight_style(Style::default().bg(theme.surface))
            .highlight_symbol("→ ")
    }

    // Get summary text
    fn summary_text(todos: &[Todo], theme: &ThemeColors) -> Vec<Line<'static>> {
        use ratatui::text::Span;

        let (total, completed): (usize, usize) =
            (todos.len(), todos.iter().filter(|t| t.completed).count());

        let percent: u8 = if total > 0 {
            (completed as f32 / total as f32 * 100.0) as u8
        } else {
            0
        };

        let filled: usize = (percent as f32 / 10.0).round() as usize;
        let gauge: String = format!(" [{}{}] ", "■".repeat(filled), "□".repeat(10 - filled));

        vec![
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
        ]
    }

    // Get hotkeys depending on application mode
    fn hotkeys(mode: &ApplicationMode) -> &'static str {
        match mode {
            ApplicationMode::Browsing => {
                " Esc/q -> Quit \n Enter -> Toggle \n a -> Add \n e -> Edit \n d -> Delete \n x -> Clear \n t -> Theme \n <C-s> -> Save \n h/l -> Focus \n ▲/▼/j/k -> Navigate \n J/K -> Move "
            }
            ApplicationMode::Task => " Esc -> Cancel \n ▲/▼ -> Next \n ◄/► -> Priority ",
        }
    }
}
