use crate::{
    state::Anchor,
    ui::{Dialog, DialogResult},
    utils::{
        constants::theme::TEXT_PRIMARY,
        widgets::popup::{color_based_on_popup_kind, render_lines_based_on_popup},
    },
};
use ratatui::{
    Frame,
    crossterm::event::KeyCode,
    layout::Rect,
    style::{Color, Style, Stylize},
    text::Line,
    widgets::Padding,
};

#[derive(Debug, Clone, PartialEq)]
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
    pub show_title: bool,
}

#[derive(Debug, Clone)]
pub struct Popup {
    pub kind: PopupKind,
    pub message: String,
    pub title: Option<String>,
    pub close_behavior: PopupCloseBehavior,
    pub anchor: Anchor,
    pub styles: PopupStyles,
}

impl Dialog for Popup {
    fn new() -> Self {
        Self {
            kind: PopupKind::Info,
            message: "".to_string(),
            title: None,
            close_behavior: PopupCloseBehavior::Specific(KeyCode::Esc),
            anchor: Anchor::Center,
            styles: PopupStyles {
                border_color: color_based_on_popup_kind(PopupKind::Info),
                padding: Padding {
                    right: 2,
                    left: 2,
                    top: 1,
                    bottom: 1,
                },
                show_title: true,
            },
        }
    }

    // Calculate area for popup
    fn area(&self, frame_area: Rect) -> Rect {
        use crate::utils::{
            anchored, calculate_content_size, constants::size::POPUP_PERCENTAGE_WIDTH,
        };

        let (top_len, bottom_len): (usize, usize) = self.titles_len();
        let (width, height): (u16, u16) = calculate_content_size(
            frame_area,
            &self.message,
            top_len,
            bottom_len,
            self.styles.padding,
            POPUP_PERCENTAGE_WIDTH,
        );

        anchored(frame_area, width, height, self.anchor.clone())
    }

    // Calculate titles length
    fn titles_len(&self) -> (usize, usize) {
        let titles: (Line, Line) = render_lines_based_on_popup(
            self.title.clone(),
            self.kind.clone(),
            self.close_behavior.clone(),
            self.styles.show_title,
        );

        (titles.0.width(), titles.1.width())
    }

    // Rendering
    fn render(&self, frame: &mut Frame, area: Rect) {
        use crate::utils::wrap_text;
        use ratatui::{
            layout::Alignment,
            widgets::{Block, BorderType, Paragraph, Wrap},
        };

        let content_width = area.width.saturating_sub(4) as usize;
        let wrapped = wrap_text(&self.message, content_width);
        let titles = render_lines_based_on_popup(
            self.title.clone(),
            self.kind.clone(),
            self.close_behavior.clone(),
            self.styles.show_title,
        );

        let block = Block::bordered()
            .border_type(BorderType::Rounded)
            .title_alignment(Alignment::Center)
            .title(titles.0)
            .title_bottom(titles.1)
            .border_style(Style::default().fg(self.styles.border_color))
            .fg(TEXT_PRIMARY)
            .padding(self.styles.padding);

        let paragraph = Paragraph::new(wrapped.join("\n"))
            .block(block)
            .wrap(Wrap { trim: false });

        frame.render_widget(paragraph, area);
    }

    // Key event handling
    fn handle_key(&mut self, key: KeyCode) -> Option<DialogResult> {
        match self.close_behavior {
            PopupCloseBehavior::AnyKey => Some(DialogResult::Cancelled),
            PopupCloseBehavior::Specific(k) if k == key => Some(DialogResult::Cancelled),
            _ => None,
        }
    }
}

// Other methods implementation
impl Popup {
    pub fn kind(mut self, kind: PopupKind) -> Self {
        self.kind = kind.clone();
        self.styles.border_color = color_based_on_popup_kind(kind);
        self
    }

    pub fn with_message(mut self, message: impl Into<String>) -> Self {
        self.message = message.into();
        self
    }

    pub fn anchor(mut self, anchor: Anchor) -> Self {
        self.anchor = anchor;
        self
    }

    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    pub fn no_title(mut self) -> Self {
        self.styles.show_title = false;
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

    pub fn not_closable(mut self) -> Self {
        self.close_behavior = PopupCloseBehavior::None;
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
}

// Unit-tests for popup widget
#[cfg(test)]
mod tests {
    use ratatui::{style::Modifier, text::Span};

    use crate::utils::constants::theme::{
        COLOR_GREEN, ERROR_POPUP_FG, HELP_POPUP_FG, INFO_POPUP_FG, SUCCESS_POPUP_FG,
    };

    use super::*;

    // Helper function to create frame for popup
    fn create_helper_frame() -> Rect {
        Rect::new(0, 0, 100, 50)
    }

    #[test]
    fn should_create_default_popup() {
        let popup: Popup = Popup::new();

        assert_eq!(popup.kind, PopupKind::Info);
        assert_eq!(popup.message, "");
        assert_eq!(popup.title, None);
        assert_eq!(
            popup.close_behavior,
            PopupCloseBehavior::Specific(KeyCode::Esc)
        );
        assert_eq!(popup.anchor, Anchor::Center);
        assert!(popup.styles.show_title);
        assert_eq!(popup.styles.padding, Padding::new(2, 2, 1, 1));
    }

    #[test]
    fn should_create_popup_with_chaining_api() {
        let popup: Popup = Popup::new()
            .kind(PopupKind::Success)
            .with_message("Task completed!")
            .title("Great!")
            .close_on_any_key()
            .anchor(Anchor::TopRight);

        assert_eq!(popup.kind, PopupKind::Success);
        assert_eq!(popup.message, "Task completed!");
        assert_eq!(popup.title, Some("Great!".to_string()));
        assert_eq!(popup.close_behavior, PopupCloseBehavior::AnyKey);
        assert_eq!(popup.anchor, Anchor::TopRight);
        assert_eq!(
            popup.styles.border_color,
            color_based_on_popup_kind(PopupKind::Success)
        );
    }

    #[test]
    fn should_create_area_for_popup() {
        let frame: Rect = create_helper_frame();
        let popup: Popup = Popup::new().with_message("Short message").title("Test");
        let area: Rect = popup.area(frame);

        assert!(area.x > 20 && area.x < 60);
        assert!(area.y > 10 && area.y < 30);
        assert!(area.width > 15 && area.width < 50);
        assert!(area.height > 3 && area.height < 15);
    }

    #[test]
    fn should_calculate_popup_titles_length() {
        let popup: Popup = Popup::new().title("Test").close_on(KeyCode::Enter);
        let (top_len, bottom_len): (usize, usize) = popup.titles_len();

        assert!(top_len > 0);
        assert!(bottom_len > 0);
    }

    #[test]
    fn should_calculate_popup_titles_length_with_no_top_title() {
        let popup: Popup = Popup::new().no_title();
        let (top_len, bottom_len): (usize, usize) = popup.titles_len();

        assert_eq!(top_len, 0);
        assert!(bottom_len > 0);
    }

    #[test]
    fn should_popup_close_on_any_key() {
        let mut popup: Popup = Popup::new().close_on_any_key();

        assert_eq!(
            popup.handle_key(KeyCode::Char('q')),
            Some(DialogResult::Cancelled)
        );
        assert_eq!(
            popup.handle_key(KeyCode::Enter),
            Some(DialogResult::Cancelled)
        );
        assert_eq!(
            popup.handle_key(KeyCode::Esc),
            Some(DialogResult::Cancelled)
        );
    }

    #[test]
    fn should_popup_close_on_specific_key() {
        let mut popup: Popup = Popup::new().close_on(KeyCode::Char('y'));

        assert_eq!(
            popup.handle_key(KeyCode::Char('y')),
            Some(DialogResult::Cancelled)
        );
        assert_eq!(popup.handle_key(KeyCode::Char('n')), None);
        assert_eq!(popup.handle_key(KeyCode::Esc), None);
    }

    #[test]
    fn should_popup_not_close() {
        let mut popup: Popup = Popup::new().not_closable();

        assert_eq!(popup.handle_key(KeyCode::Char('q')), None);
        assert_eq!(popup.handle_key(KeyCode::Char('n')), None);
        assert_eq!(popup.handle_key(KeyCode::Esc), None);
    }

    // Utils functions
    #[test]
    fn should_return_border_color_based_on_popup_kind() {
        let mut popup: Popup = Popup::new().kind(PopupKind::Error);
        assert_eq!(popup.styles.border_color, ERROR_POPUP_FG);

        popup = popup.kind(PopupKind::Success);
        assert_eq!(popup.styles.border_color, SUCCESS_POPUP_FG);

        popup = popup.kind(PopupKind::Help);
        assert_eq!(popup.styles.border_color, HELP_POPUP_FG);

        popup = popup.kind(PopupKind::Info);
        assert_eq!(popup.styles.border_color, INFO_POPUP_FG);
    }

    #[test]
    fn should_return_corresponding_lines_for_popup_with_title() {
        let lines: (Line, Line) = render_lines_based_on_popup(
            Some("Test".to_string()),
            PopupKind::Info,
            PopupCloseBehavior::Specific(KeyCode::Enter),
            true,
        );

        let expected_top_line: Line = Line::from(Span::styled(
            " Test ",
            Style::default()
                .fg(TEXT_PRIMARY)
                .add_modifier(Modifier::BOLD),
        ));
        assert_eq!(lines.0, expected_top_line);

        let expected_bottom_line: Line = Line::from(vec![
            Span::styled(" Press ", Style::default().fg(TEXT_PRIMARY)),
            Span::styled(
                "<Return>",
                Style::default()
                    .fg(COLOR_GREEN)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" to close this popup. ", Style::default().fg(TEXT_PRIMARY)),
        ]);
        assert_eq!(lines.1, expected_bottom_line);
    }

    #[test]
    fn should_return_corresponding_lines_for_popup_without_title() {
        let lines: (Line, Line) =
            render_lines_based_on_popup(None, PopupKind::Info, PopupCloseBehavior::AnyKey, false);
        assert_eq!(lines.0, Line::default());

        let expected_bottom_line: Line = Line::from(vec![
            Span::styled(" Press ", Style::default().fg(TEXT_PRIMARY)),
            Span::styled(
                "any key",
                Style::default()
                    .fg(COLOR_GREEN)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" to close this popup. ", Style::default().fg(TEXT_PRIMARY)),
        ]);
        assert_eq!(lines.1, expected_bottom_line);
    }

    #[test]
    fn should_return_corresponding_lines_for_popup_defaults() {
        let lines: (Line, Line) =
            render_lines_based_on_popup(None, PopupKind::Error, PopupCloseBehavior::None, true);

        let expected_top_line: Line = Line::from(Span::styled(
            " Error ",
            Style::default()
                .fg(TEXT_PRIMARY)
                .add_modifier(Modifier::BOLD),
        ));

        assert_eq!(lines.0, expected_top_line);
        assert_eq!(lines.1, Line::default());
    }
}
