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

    // Closing behaviour chaining methods
    pub fn close_on_any_key(mut self) -> Self {
        self.close_behavior = PopupCloseBehavior::AnyKey;
        self
    }

    pub fn close_on(mut self, key: KeyCode) -> Self {
        self.close_behavior = PopupCloseBehavior::Specific(key);
        self
    }

    // Rendering
    pub fn render(&self, frame: &mut Frame) {
        // 30 %
        let max_width: u16 = (frame.area().width as f32 * 0.3) as u16;

        let wrapped_lines: Vec<String> = other_utils::wrap_text(&self.message, max_width as usize);
        let text_height: u16 = wrapped_lines.len() as u16;

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

        let popup_area: Rect =
            widgets_utils::get_popup_area(frame.area(), max_width + 4, text_height + 4);

        frame.render_widget(text, popup_area);
    }
}
