use crate::{
    enums::FocusArea,
    models::{Filter, Priority, Todo},
};
use ratatui::{Frame, layout::Rect, widgets::ListState};

pub struct TaskList;

impl TaskList {
    pub fn render(
        frame: &mut Frame,
        area: Rect,
        todos: &[Todo],
        select_state: &ListState,
        current_filter: Filter,
        focused_area: &FocusArea,
    ) {
        use ratatui::{
            style::{Color, Style, Stylize},
            text::{Line, Span},
            widgets::{Block, List, ListItem},
        };

        let filtered = current_filter.filter(todos);

        let list_items = filtered.iter().map(|todo| {
            let priority_style: Style = match todo.priority {
                Priority::High => Style::default().fg(Color::LightRed),
                Priority::Medium => Style::default().fg(Color::LightYellow),
                Priority::Low => Style::default().fg(Color::LightGreen),
            };

            let title = if todo.completed {
                Span::styled(
                    format!("✓ {}", todo.title),
                    Style::default().fg(Color::Green),
                )
            } else {
                Span::styled(todo.title.clone(), priority_style)
            };

            ListItem::new(Line::from(vec![
                title,
                Span::raw(" "),
                Span::styled(
                    todo.description.chars().take(30).collect::<String>(),
                    Style::default().dim(),
                ),
            ]))
        });

        let focused_style: Style = if *focused_area == FocusArea::MainContent {
            Style::default().fg(Color::Green)
        } else {
            Style::default()
        };

        let list = List::new(list_items)
            .block(
                Block::bordered()
                    .title(" Tasks ")
                    .border_style(focused_style),
            )
            .highlight_style(Style::default().bg(Color::DarkGray))
            .highlight_symbol(">> ");

        frame.render_stateful_widget(list, area, &mut select_state.clone());
    }
}
