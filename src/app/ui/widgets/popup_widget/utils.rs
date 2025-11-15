use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
};

use super::popup::{Popup, PopupCloseBehavior, PopupKind};
use crate::app::utils::{layout::center, text::wrap_text};

// Calculate popup area based on context
pub fn calculate_area(popup: Popup, frame_area: Rect) -> Rect {
    use crate::app::utils::math::*;

    let (top, bottom) = lines_based_on_popup(popup.clone());

    let top_title_width: usize = top.width();
    let bottom_title_width: usize = bottom.width();

    let max_allowed_width: usize = percentage_of(frame_area.width, 70.0);
    let raw_lines: Vec<&str> = popup.message.lines().collect::<Vec<_>>();
    let extra_space_width: usize = 6;

    let content_max_line: usize = calculate_max_line_len(&raw_lines);

    let base_width: usize = content_max_line.min(max_allowed_width);
    let content_height: usize = wrap_text(&popup.message, base_width).len();

    let mut height = content_height + 2;
    height += (popup.styles.padding.top + popup.styles.padding.bottom) as usize;

    let mut width = base_width + extra_space_width;
    width = width.max(bottom_title_width);

    if !top.spans.is_empty() {
        width = width.max(top_title_width);
    }

    width += (popup.styles.padding.left + popup.styles.padding.right) as usize;

    width = width.min(frame_area.width as usize);
    height = height.min(frame_area.height as usize);

    center(frame_area, width as u16, height as u16)
}

pub fn color_based_on_popup_kind(kind: PopupKind) -> Color {
    match kind {
        PopupKind::Error => Color::Rgb(245, 161, 145),
        PopupKind::Success => Color::Rgb(144, 185, 159),
        PopupKind::Help => Color::Rgb(226, 158, 202),
        PopupKind::Info => Color::Rgb(172, 161, 207),
    }
}

pub fn lines_based_on_popup<'a>(popup: Popup) -> (Line<'a>, Line<'a>) {
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
