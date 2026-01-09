use super::popup::{PopupCloseBehavior, PopupKind};
use crate::app::utils::constants::theme::*;
use ratatui::{style::Color, text::Line};

// Color based on popup kind (info, error, success, help)
pub fn color_based_on_popup_kind(kind: PopupKind) -> Color {
    match kind {
        PopupKind::Error => ERROR_POPUP_FG,
        PopupKind::Success => SUCCESS_POPUP_FG,
        PopupKind::Help => HELP_POPUP_FG,
        PopupKind::Info => INFO_POPUP_FG,
    }
}

// Pre-rendered lines based on popup
pub fn render_lines_based_on_popup<'a>(
    title: Option<String>,
    kind: PopupKind,
    close_behavior: PopupCloseBehavior,
    show_title: bool,
) -> (Line<'a>, Line<'a>) {
    use ratatui::{
        style::{Modifier, Style},
        text::Span,
    };

    let top_line: Line = if show_title {
        if let Some(title) = title {
            Line::from(Span::styled(
                format!(" {} ", title),
                Style::default()
                    .fg(TEXT_PRIMARY)
                    .add_modifier(Modifier::BOLD),
            ))
        } else {
            let defaults: &str = match kind {
                PopupKind::Help => " Help ",
                PopupKind::Error => " Error ",
                PopupKind::Success => " Success ",
                PopupKind::Info => " Info ",
            };
            Line::from(Span::styled(
                defaults,
                Style::default()
                    .fg(TEXT_PRIMARY)
                    .add_modifier(Modifier::BOLD),
            ))
        }
    } else {
        Line::default()
    };

    let key: String = match close_behavior {
        PopupCloseBehavior::AnyKey => "any key".to_string(),
        PopupCloseBehavior::Specific(c) => format!("<{}>", c),
        _ => "".to_string(),
    };

    let bottom_line: Line = if close_behavior != PopupCloseBehavior::None {
        Line::from(vec![
            Span::styled(" Press ", Style::default().fg(TEXT_PRIMARY)),
            Span::styled(
                key,
                Style::default()
                    .fg(COLOR_GREEN)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" to close this popup. ", Style::default().fg(TEXT_PRIMARY)),
        ])
    } else {
        Line::default()
    };

    (top_line, bottom_line)
}
