use crate::{
    enums::ApplicationMode,
    models::{Filter, Sort, Todo},
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

/// Sidebar widget
pub struct SidebarWidget<'a> {
    ui: &'a UIState,
    todos: &'a [Todo],
    mode: &'a ApplicationMode,
    sort: Sort,
    theme: &'a ThemeColors,
}

impl<'a> SidebarWidget<'a> {
    pub fn new(
        ui: &'a UIState,
        todos: &'a [Todo],
        mode: &'a ApplicationMode,
        sort: Sort,
        theme: &'a ThemeColors,
    ) -> Self {
        Self {
            ui,
            todos,
            mode,
            sort,
            theme,
        }
    }

    /// Sidebar rendering
    pub fn render(&self, frame: &mut Frame, area: Rect) {
        use crate::enums::FocusArea;
        use ratatui::widgets::{Block, ListState, Paragraph};

        let (hotkeys, hotkeys_len): (&str, u16) = self.hotkeys();

        let sidebar_layout: std::rc::Rc<[Rect]> = self.layout(area, hotkeys_len + 4);
        let focused_style: Style = self.ui.focused_on(&FocusArea::LeftPanel);

        let filters_block: Block = Block::bordered()
            .title(" Filters ")
            .border_style(focused_style)
            .bg(self.theme.bg_dim);

        let filters_inner_area: Rect = filters_block.inner(sidebar_layout[0]);
        let filter_tab_layout: std::rc::Rc<[Rect]> = self.filters_tab_layout(filters_inner_area);

        let query: String = self.ui.search_query();
        let list: List = self.construct_list(&query);

        let mut state: ListState = ListState::default();
        state.select(Some(self.ui.current_filter.index()));

        frame.render_widget(filters_block, sidebar_layout[0]);
        frame.render_stateful_widget(list, filter_tab_layout[1], &mut state);

        let summary_block: Block = Block::bordered()
            .title(" Summary ")
            .border_style(Style::default().fg(self.theme.border))
            .bg(self.theme.bg_dim);

        let summary_inner_area: Rect = summary_block.inner(sidebar_layout[1]);
        let summary_inner_layout: std::rc::Rc<[Rect]> = self.summary_layout(summary_inner_area);
        let summary_text: Vec<Line> = self.summary_text();

        frame.render_widget(summary_block, sidebar_layout[1]);
        frame.render_widget(Paragraph::new(summary_text), summary_inner_layout[1]);

        let hotkeys_block: Block = Block::bordered()
            .title(" Hotkeys ")
            .border_style(Style::default().fg(self.theme.border))
            .bg(self.theme.bg_dim);

        let hotkeys_inner_area: Rect = hotkeys_block.inner(sidebar_layout[2]);
        let hotkeys_layout: std::rc::Rc<[Rect]> = self.hotkeys_layout(hotkeys_inner_area);

        frame.render_widget(hotkeys_block, sidebar_layout[2]);
        frame.render_widget(
            Paragraph::new(hotkeys).style(Style::default().fg(self.theme.text_primary)),
            hotkeys_layout[1],
        );
    }

    /// Layout for sidebar
    fn layout(&self, area: Rect, hotkeys_length: u16) -> std::rc::Rc<[Rect]> {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Fill(1),                // Filters
                Constraint::Length(8),              // Summary
                Constraint::Length(hotkeys_length), // Hotkeys
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

    /// Layout for hotkeys
    fn hotkeys_layout(&self, area: Rect) -> std::rc::Rc<[Rect]> {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1), // Hotkeys margin
                Constraint::Min(0),    // Hotkeys
            ])
            .split(area)
    }

    // Construct a list based on filtered todo values
    fn construct_list(&self, query: &str) -> List<'static> {
        use ratatui::widgets::ListItem;

        let items: Vec<ListItem> = Filter::all_variants()
            .iter()
            .map(|tab| {
                let count = tab.count(self.todos, query);
                let text = format!(" {} ({})", tab.to_string(), count);
                let style = if *tab == self.ui.current_filter {
                    Style::default().fg(self.theme.accent).bold()
                } else {
                    Style::default().fg(self.theme.text_primary)
                };

                ListItem::new(Span::styled(text, style))
            })
            .collect();

        List::new(items)
            .highlight_style(Style::default().bg(self.theme.surface))
            .highlight_symbol("→ ")
    }

    /// Get summary text
    fn summary_text(&self) -> Vec<Line<'static>> {
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
            Line::from(vec![
                Span::styled(" Theme: ", Style::default().fg(self.theme.text_dim)),
                Span::styled(
                    self.theme.name,
                    Style::default().fg(self.theme.accent).bold(),
                ),
            ]),
            Line::from(vec![
                Span::styled(" Progress: ", Style::default().fg(self.theme.text_dim)),
                Span::styled(
                    format!("{}%", percent),
                    Style::default().fg(self.theme.success).bold(),
                ),
            ]),
            Line::from(Span::styled(gauge, Style::default().fg(self.theme.success))),
            Line::from(vec![
                Span::styled(" Sort: ", Style::default().fg(self.theme.text_dim)),
                Span::styled(
                    self.sort.label(),
                    Style::default().fg(self.theme.accent).bold(),
                ),
            ]),
        ]
    }

    /// Get hotkeys depending on application mode
    fn hotkeys(&self) -> (&str, u16) {
        match self.mode {
            ApplicationMode::Browsing => (
                " Esc/q:Quit \n a:Add \n x:Clear \n t:Theme \n s:Select sort \n r:Reverse sort order \n <C-s>:Save \n h/l:Focus \n ▲/▼/j/k:Navigate ",
                9,
            ),
            ApplicationMode::List => (
                " Esc/q:Quit \n Enter:Toggle \n /:Search \n a:Add \n e:Edit \n d:Delete \n x:Clear \n t:Theme \n s:Select sort \n r:Reverse sort order \n <C-s>:Save \n h/l:Focus \n ▲/▼/j/k:Navigate \n J/K:Move \n ]/[:Scroll description ",
                15,
            ),
            ApplicationMode::Form => (
                " <A-Enter>:Submit \n Esc:Cancel \n ▲/▼:Next \n ◄/►:Priority ",
                4,
            ),
            ApplicationMode::Search => (
                " Esc:Quit \n Enter:Search \n Backspace:Remove char \n ◄/►:Cursor ",
                4,
            ),
        }
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
        let sidebar: SidebarWidget = SidebarWidget::new(
            &ui,
            &todos,
            &ApplicationMode::Browsing,
            Sort::default(),
            &ThemeColors::GRUVBOX,
        );

        let summary: Vec<Line> = sidebar.summary_text();
        let line_text: String = summary[1].to_string();
        assert!(line_text.contains("50%"));

        let gauge_text: String = summary[2].to_string();
        assert!(gauge_text.contains("■■■■■□□□□□"));

        let sort_text: String = summary[3].to_string();
        assert_eq!(sort_text, " Sort: Priority ▼");
    }

    #[test]
    fn should_make_summary_for_empty_todos() {
        let todos: Vec<Todo> = vec![];

        let ui = UIState::default();
        let sidebar: SidebarWidget = SidebarWidget::new(
            &ui,
            &todos,
            &ApplicationMode::Browsing,
            Sort::default(),
            &ThemeColors::GRUVBOX,
        );

        let summary: Vec<Line> = sidebar.summary_text();
        assert!(summary[1].to_string().contains("0%"));
        assert!(summary[2].to_string().contains("□□□□□□□□□□"));
    }

    #[test]
    fn should_construct_list_with_highlighting() {
        let todos = vec![];

        let mut ui = UIState::default();
        ui.current_filter = Filter::All;
        let sidebar: SidebarWidget = SidebarWidget::new(
            &ui,
            &todos,
            &ApplicationMode::Browsing,
            Sort::default(),
            &ThemeColors::GRUVBOX,
        );

        let list: List = sidebar.construct_list("");
        assert_eq!(list.len(), Filter::all_variants().len());
    }

    #[test]
    fn should_return_hotkeys_for_browsing_mode() {
        let todos = vec![];
        let ui = UIState::default();
        let sidebar: SidebarWidget = SidebarWidget::new(
            &ui,
            &todos,
            &ApplicationMode::Browsing,
            Sort::default(),
            &ThemeColors::GRUVBOX,
        );

        let (browsing_keys, browsing_key_len) = sidebar.hotkeys();

        assert!(browsing_keys.contains("Quit"));
        assert_eq!(browsing_key_len, 9);
    }

    #[test]
    fn should_return_hotkeys_for_form_mode() {
        let todos = vec![];
        let ui = UIState::default();
        let sidebar: SidebarWidget = SidebarWidget::new(
            &ui,
            &todos,
            &ApplicationMode::Form,
            Sort::default(),
            &ThemeColors::GRUVBOX,
        );

        let (task_keys, task_key_len) = sidebar.hotkeys();

        assert!(task_keys.contains("Cancel"));
        assert_eq!(task_key_len, 4);
    }
}
