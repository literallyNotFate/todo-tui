use crate::app::ui::components::popup::{Popup, PopupCloseBehavior, PopupKind};
use ratatui::{
    layout::{Constraint, Flex, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
};

// For popup

pub fn color_based_on_popup_kind(kind: PopupKind) -> Color {
    match kind {
        PopupKind::Error => Color::Rgb(245, 161, 145),
        PopupKind::Success => Color::Rgb(144, 185, 159),
        PopupKind::Help => Color::Rgb(226, 158, 202),
        PopupKind::Info => Color::Rgb(172, 161, 207),
    }
}

pub fn lines_based_on_popup<'a>(popup: Popup) -> (Line<'a>, Line<'a>) {
    let main_line: Line = Line::from(Span::styled(
        match popup.kind {
            PopupKind::Help => " Help ",
            PopupKind::Error => " Error ",
            PopupKind::Success => " Success ",
            PopupKind::Info => " Info ",
        },
        Style::default()
            .fg(Color::Rgb(252, 252, 252))
            .add_modifier(Modifier::BOLD),
    ));

    let key: String = match popup.close_behavior {
        PopupCloseBehavior::AnyKey => "any key".to_string(),
        PopupCloseBehavior::Specific(c) => format!("<{}>", c),
        PopupCloseBehavior::None => "".to_string(),
    };

    let bottom_line: Line = Line::from(vec![
        Span::styled(" Press ", Style::default().fg(Color::Rgb(252, 252, 252))),
        Span::styled(
            key,
            Style::default()
                .fg(Color::Rgb(165, 252, 115))
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            " to close this popup. ",
            Style::default().fg(Color::Rgb(252, 252, 252)),
        ),
    ]);

    (main_line, bottom_line)
}

pub fn get_popup_area(area: Rect, x_length: u16, y_length: u16) -> Rect {
    let vertical = Layout::vertical([Constraint::Max(y_length)]).flex(Flex::Center);
    let horizontal = Layout::horizontal([Constraint::Max(x_length)]).flex(Flex::Center);
    let [area] = vertical.areas(area);
    let [area] = horizontal.areas(area);

    area
}
