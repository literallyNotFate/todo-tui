use ratatui::crossterm::event::KeyCode;
use ratatui::layout::Alignment;
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

#[derive(Debug, Clone, PartialEq)]
pub struct PopupStyles {
    pub border_color: Color,
    pub padding: Padding,
    pub max_width: Option<u16>,
}

#[derive(Debug, Clone)]
pub struct Popup {
    pub kind: PopupKind,
    pub message: String,
    pub title: Option<String>,
    pub close_behavior: PopupCloseBehavior,

    pub styles: PopupStyles,
}

impl Popup {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            kind: PopupKind::Info,
            message: message.into(),
            title: None,
            close_behavior: PopupCloseBehavior::Specific(KeyCode::Esc),

            styles: PopupStyles {
                border_color: widgets_utils::color_based_on_popup_kind(PopupKind::Info),
                padding: Padding {
                    right: 2,
                    left: 2,
                    top: 1,
                    bottom: 1,
                },
                max_width: None,
            },
        }
    }

    // Rendering
    pub fn render(&self, frame: &mut Frame, area: Rect) {
        let content_width = area.width.saturating_sub(4) as usize;
        let wrapped = other_utils::wrap_text(&self.message, content_width);
        let titles = widgets_utils::lines_based_on_popup(self.clone());

        let block = Block::bordered()
            .border_type(BorderType::Rounded)
            .title_alignment(Alignment::Center)
            .title(titles.0)
            .title_bottom(titles.1)
            .border_style(Style::default().fg(self.styles.border_color))
            .padding(self.styles.padding);

        let paragraph = Paragraph::new(wrapped.join("\n"))
            .block(block)
            .wrap(Wrap { trim: false });

        frame.render_widget(paragraph, area);
    }

    pub fn calculate_area(&self, frame_area: Rect) -> Rect {
        let titles = widgets_utils::lines_based_on_popup(self.clone());

        let max_allowed_width: usize = ((frame_area.width as f32) * 0.7).floor() as usize;
        let raw_lines = self.message.lines().collect::<Vec<_>>();

        let extra_space_width: usize = 6;

        let content_max_line = raw_lines
            .iter()
            .map(|l| l.chars().count())
            .max()
            .unwrap_or(1)
            .max(1);

        let base_width = content_max_line.min(max_allowed_width);

        let wrapped = other_utils::wrap_text(&self.message, base_width);
        let content_height = wrapped.len();

        let mut height = content_height;

        if !titles.0.spans.is_empty() {
            height += 1;
        }

        if !titles.1.spans.is_empty() {
            height += 1;
        }

        height += (self.styles.padding.top + self.styles.padding.bottom) as usize;

        let mut width = base_width + extra_space_width;
        width += (self.styles.padding.left + self.styles.padding.right) as usize;

        width = width.min(frame_area.width as usize);
        height = height.min(frame_area.height as usize);

        widgets_utils::get_popup_area(frame_area, width as u16, height as u16)
    }

    // Chaining API
    pub fn kind(mut self, kind: PopupKind) -> Self {
        self.kind = kind.clone();
        self.styles.border_color = widgets_utils::color_based_on_popup_kind(kind);
        self
    }

    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    pub fn close_on_any_key(mut self) -> Self {
        self.close_behavior = PopupCloseBehavior::AnyKey;
        self
    }

    pub fn close_on(mut self, key: KeyCode) -> Self {
        self.close_behavior = PopupCloseBehavior::Specific(key);
        self
    }

    pub fn with_border_color(mut self, color: Color) -> Self {
        self.styles.border_color = color;
        self
    }

    pub fn with_padding(mut self, padding: Padding) -> Self {
        self.styles.padding = padding;
        self
    }

    pub fn with_max_width(mut self, width: u16) -> Self {
        self.styles.max_width = Some(width);
        self
    }
}
