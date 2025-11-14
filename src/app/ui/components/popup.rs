use ratatui::crossterm::event::KeyCode;
use ratatui::layout::Alignment;
use ratatui::text::Line;
use ratatui::widgets::{BorderType, Padding, Wrap};
use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, Paragraph},
};

use crate::app::utils::{other as other_utils, widgets as widgets_utils};

#[derive(Debug, Clone)]
pub enum PopupCloseBehavior {
    AnyKey,
    Specific(KeyCode),
    None,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PopupKind {
    Help,
    Info,
    Error,
    Success,
}

#[derive(Debug, Clone)]
pub struct Popup {
    pub kind: PopupKind,
    pub message: String,
    pub close_behavior: PopupCloseBehavior,
    pub color: Color,
}

impl Popup {
    pub fn new(kind: PopupKind, message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            kind: kind.clone(),
            color: widgets_utils::color_based_on_popup_kind(kind),
            close_behavior: PopupCloseBehavior::Specific(KeyCode::Esc),
        }
    }

    // Chaining methods
    pub fn close_on_any_key(mut self) -> Self {
        self.close_behavior = PopupCloseBehavior::AnyKey;
        self
    }

    pub fn close_on(mut self, key: KeyCode) -> Self {
        self.close_behavior = PopupCloseBehavior::Specific(key);
        self
    }

    pub fn with_color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    // Calculate popup size
    pub fn calculate_area(&self, frame_area: Rect) -> Rect {
        let max_allowed_content: usize = ((frame_area.width as f32) * 0.7).floor() as usize;
        let max_allowed_content: usize = max_allowed_content.saturating_sub(0).max(1);

        let content_max_line: usize = self
            .message
            .lines()
            .map(|l| l.chars().count())
            .max()
            .unwrap_or(0);

        let titles: (Line, Line) = widgets_utils::lines_based_on_popup(self.clone());
        let bottom_title_len: usize = titles.1.to_string().chars().count();
        let top_title_len: usize = titles.0.to_string().chars().count();

        let mut base_width = content_max_line.max(bottom_title_len);
        if base_width == 0 {
            base_width = 1;
        }
        if base_width > max_allowed_content {
            base_width = max_allowed_content;
        }

        let wrapped_lines = other_utils::wrap_text(&self.message, base_width);
        let content_height = wrapped_lines.len();

        let title_lines = (top_title_len > 0) as usize + (bottom_title_len > 0) as usize;

        let horiz_extra = 2 + 2;
        let vert_extra = 1 + 1;

        let popup_content_width = base_width;
        let popup_width = (popup_content_width + horiz_extra) as u16 + 2;
        let popup_content_height = content_height + title_lines;
        let popup_height = (popup_content_height + vert_extra) as u16 + 2;

        let popup_height = popup_height.min(frame_area.height);
        let popup_width = popup_width.min(frame_area.width);

        widgets_utils::get_popup_area(frame_area, popup_width, popup_height)
    }

    // Rendering
    pub fn render(&self, frame: &mut Frame, area: Rect) {
        let content_width: usize = area.width.saturating_sub(4) as usize;
        let wrapped_lines: Vec<String> = other_utils::wrap_text(&self.message, content_width);

        let titles: (Line, Line) = widgets_utils::lines_based_on_popup(self.clone());

        let block: Block = Block::bordered()
            .border_type(BorderType::Rounded)
            .title_alignment(Alignment::Center)
            .title(titles.0)
            .title_bottom(titles.1)
            .border_style(Style::default().fg(self.color))
            .padding(Padding {
                left: 2,
                right: 2,
                top: 1,
                bottom: 1,
            });

        let text: Paragraph = Paragraph::new(wrapped_lines.join("\n"))
            .block(block)
            .wrap(Wrap { trim: false });

        frame.render_widget(text, area);
    }
}
