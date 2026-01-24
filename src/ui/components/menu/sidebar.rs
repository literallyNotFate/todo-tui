use crate::{
    enums::FocusArea,
    models::{Filter, Todo},
    state::UIState,
    traits::InteractableEnum,
};
use ratatui::{Frame, layout::Rect};

pub struct MenuSidebar;

impl MenuSidebar {
    pub fn render(frame: &mut Frame, area: Rect, ui: &UIState, todos: &[Todo]) {
        use ratatui::{
            style::{Color, Modifier, Style},
            text::Span,
            widgets::{Block, List, ListItem, ListState},
        };

        let items: Vec<ListItem> = Filter::all_variants()
            .iter()
            .map(|tab| {
                let count = tab.count(todos);
                let text = format!("{} ({})", tab.to_string(), count);
                let style = if *tab == ui.current_filter {
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::White)
                };
                ListItem::new(Span::styled(text, style))
            })
            .collect();

        let is_focused = ui.focus_area == FocusArea::LeftPanel;
        let focused_style: Style = if is_focused {
            Style::default().fg(Color::Green)
        } else {
            Style::default()
        };

        let list = List::new(items)
            .block(
                Block::bordered()
                    .title(" Filters ")
                    .border_style(focused_style),
            )
            .highlight_style(Style::default().bg(Color::DarkGray))
            .highlight_symbol("→ ");

        let mut state = ListState::default();
        state.select(Some(ui.current_filter.index()));

        frame.render_stateful_widget(list, area, &mut state);
    }
}
