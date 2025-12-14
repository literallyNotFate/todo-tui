use super::popup::{Popup, PopupCloseBehavior, PopupKind};
use ratatui::{style::Color, text::Line};

// Color based on popup kind (info, error, success, help)
pub fn color_based_on_popup_kind(kind: PopupKind) -> Color {
    match kind {
        PopupKind::Error => Color::Rgb(245, 161, 145),
        PopupKind::Success => Color::Rgb(144, 185, 159),
        PopupKind::Help => Color::Rgb(226, 158, 202),
        PopupKind::Info => Color::Rgb(172, 161, 207),
    }
}

// Pre-rendered lines based on popup
pub fn lines_based_on_popup<'a>(popup: Popup) -> (Line<'a>, Line<'a>) {
    use ratatui::{
        style::{Modifier, Style},
        text::Span,
    };

    let top_line: Line = if popup.styles.show_title {
        if let Some(ref user_title) = popup.title {
            Line::from(Span::styled(
                format!(" {} ", user_title),
                Style::default()
                    .fg(Color::Rgb(252, 252, 252))
                    .add_modifier(Modifier::BOLD),
            ))
        } else {
            let defaults: &str = match popup.kind {
                PopupKind::Help => " Help ",
                PopupKind::Error => " Error ",
                PopupKind::Success => " Success ",
                PopupKind::Info => " Info ",
            };
            Line::from(Span::styled(
                defaults,
                Style::default()
                    .fg(Color::Rgb(252, 252, 252))
                    .add_modifier(Modifier::BOLD),
            ))
        }
    } else {
        Line::default()
    };

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

    (top_line, bottom_line)
}
