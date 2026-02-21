use crate::{
    theme::ThemePalette,
    traits::{Modal, ModalResult},
    ui::{RenderContext, center},
};
use ratatui::{
    crossterm::event::KeyCode,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Span},
};

pub const POPUP_WIDTH: u16 = 40;
pub const POPUP_HEIGHT: u16 = 25;

/// Defines how popup is getting closed (on any key or on specific)
#[derive(Debug, Clone, PartialEq)]
pub enum PopupCloseBehavior {
    AnyKey,
    Specific(KeyCode),
}

/// Type of a popup
#[derive(Debug, Clone, PartialEq)]
pub enum PopupKind {
    Info,
    Error,
    Success,
}

/// Popup modal widget
#[derive(Debug, Clone)]
pub struct Popup {
    pub message: String,
    pub title: String,
    pub kind: PopupKind,
    pub close_behavior: PopupCloseBehavior,
}

impl Popup {
    /// Creating info popup template
    pub fn info(message: impl Into<String>) -> Self {
        Self {
            kind: PopupKind::Info,
            title: String::from(" Info "),
            message: message.into(),
            close_behavior: PopupCloseBehavior::Specific(KeyCode::Esc),
        }
    }

    /// Creating success popup template
    pub fn success(message: impl Into<String>) -> Self {
        Self {
            kind: PopupKind::Success,
            title: String::from(" Success "),
            ..Self::info(message)
        }
    }

    /// Creating error popup template
    pub fn error(message: impl Into<String>) -> Self {
        Self {
            kind: PopupKind::Error,
            title: String::from(" Error "),
            ..Self::info(message)
        }
    }

    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
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

    /// Vertical layout for inner content
    fn vertical_layout(&self, area: Rect) -> std::rc::Rc<[Rect]> {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Fill(1), Constraint::Min(1), Constraint::Fill(1)])
            .split(area)
    }

    /// Horizontal layout for inner content
    fn horizontal_layout(&self, area: Rect) -> std::rc::Rc<[Rect]> {
        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(10), // Left
                Constraint::Percentage(80),
                Constraint::Percentage(10), // Right
            ])
            .split(area)
    }

    /// Generate bottom title based on close behavior
    fn bottom_title(&self, palette: &ThemePalette) -> Line<'static> {
        let key: String = match self.close_behavior {
            PopupCloseBehavior::AnyKey => "any key".to_string(),
            PopupCloseBehavior::Specific(c) => format!("<{}>", c),
        };

        Line::from(vec![
            Span::styled(" Press ", Style::default().fg(palette.fg)),
            Span::styled(key, Style::default().fg(palette.success).bold()),
            Span::styled(" to close this popup. ", Style::default().fg(palette.fg)),
        ])
        .centered()
    }

    /// Return color based on kind
    fn color_on_kind(&self, palette: &ThemePalette) -> Color {
        match self.kind {
            PopupKind::Info => palette.accent,
            PopupKind::Success => palette.success,
            PopupKind::Error => palette.error,
        }
    }
}

impl Modal for Popup {
    /// Calculate area for popup
    fn area(&self, frame_area: Rect) -> Rect {
        center(frame_area, POPUP_WIDTH, POPUP_HEIGHT)
    }

    /// Popup rendering
    fn render(&self, ctx: &mut RenderContext, area: Rect) {
        use ratatui::widgets::{Block, BorderType, Paragraph, Wrap};

        let palette: ThemePalette = ctx.palette();
        let popup_block: Block = Block::bordered()
            .border_type(BorderType::Rounded)
            .title_alignment(Alignment::Center)
            .title(self.title.as_str())
            .title_bottom(self.bottom_title(&palette))
            .border_style(self.color_on_kind(&palette))
            .fg(palette.fg);

        let inner_area: Rect = popup_block.inner(area);
        ctx.render_widget(popup_block, area);

        let vertical_chunks: std::rc::Rc<[Rect]> = self.vertical_layout(inner_area);
        let message_area: Rect = self.horizontal_layout(vertical_chunks[1])[1];

        let message: Paragraph = Paragraph::new(self.message.as_str())
            .centered()
            .wrap(Wrap { trim: true });

        ctx.render_widget(message, message_area);
    }

    /// Key event handling
    fn handle_key(&mut self, key: KeyCode) -> Option<ModalResult> {
        match self.close_behavior {
            PopupCloseBehavior::AnyKey => Some(ModalResult::Cancelled),
            PopupCloseBehavior::Specific(k) if k == key => Some(ModalResult::Cancelled),
            _ => None,
        }
    }
}

/// Unit-tests for popup widget
#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::ThemeName;

    fn create_helper_frame() -> Rect {
        Rect::new(0, 0, 100, 100)
    }

    #[test]
    fn should_create_default_popup() {
        let mut popup: Popup = Popup::success("Success");

        assert_eq!(popup.kind, PopupKind::Success);
        assert_eq!(popup.message, "Success");
        assert_eq!(popup.title, " Success ");
        assert_eq!(
            popup.close_behavior,
            PopupCloseBehavior::Specific(KeyCode::Esc)
        );

        popup = Popup::info("Info");
        assert_eq!(popup.kind, PopupKind::Info);
        assert_eq!(popup.message, "Info");

        popup = Popup::error("Error");
        assert_eq!(popup.kind, PopupKind::Error);
        assert_eq!(popup.message, "Error");
    }

    #[test]
    fn should_create_popup_with_chaining_api() {
        let popup: Popup = Popup::success("Task completed!")
            .title("Some title")
            .close_on_any_key();

        assert_eq!(popup.kind, PopupKind::Success);
        assert_eq!(popup.message, "Task completed!");
        assert_eq!(popup.title, "Some title");
        assert_eq!(popup.close_behavior, PopupCloseBehavior::AnyKey);
    }

    #[test]
    fn should_create_area_for_popup() {
        let frame: Rect = create_helper_frame();
        let popup: Popup = Popup::info("Test");
        let area: Rect = popup.area(frame);

        let expected_x = (100 - POPUP_WIDTH) / 2;
        let expected_y = (100 - POPUP_HEIGHT) / 2;

        assert_eq!(area.x, expected_x as u16);
        assert_eq!(area.y, expected_y as u16);
    }

    #[test]
    fn should_popup_close_on_any_key() {
        let mut popup: Popup = Popup::info("Test").close_on_any_key();

        assert_eq!(
            popup.handle_key(KeyCode::Char('q')),
            Some(ModalResult::Cancelled)
        );
        assert_eq!(
            popup.handle_key(KeyCode::Enter),
            Some(ModalResult::Cancelled)
        );
        assert_eq!(popup.handle_key(KeyCode::Esc), Some(ModalResult::Cancelled));
    }

    #[test]
    fn should_popup_close_on_specific_key() {
        let mut popup: Popup = Popup::error("Test").close_on(KeyCode::Char('y'));

        assert_eq!(
            popup.handle_key(KeyCode::Char('y')),
            Some(ModalResult::Cancelled)
        );
        assert_eq!(popup.handle_key(KeyCode::Char('n')), None);
        assert_eq!(popup.handle_key(KeyCode::Esc), None);
    }

    #[test]
    fn should_return_bottom_title_for_popup() {
        let mut popup = Popup::success("Test");
        let mut palette: ThemePalette = ThemeName::GruvboxDark.palette();
        let mut bottom_title: Line = popup.bottom_title(&palette);

        let expected: Line = Line::from(vec![
            Span::styled(" Press ", Style::default().fg(Color::Rgb(235, 219, 178))),
            Span::styled(
                "<Esc>",
                Style::default().fg(Color::Rgb(184, 187, 38)).bold(),
            ),
            Span::styled(
                " to close this popup. ",
                Style::default().fg(Color::Rgb(235, 219, 178)),
            ),
        ])
        .centered();

        assert_eq!(bottom_title, expected);

        popup = Popup::error("Test").close_on_any_key();
        bottom_title = popup.bottom_title(&palette);

        assert_eq!(
            bottom_title.spans[1],
            Span::styled(
                "any key",
                Style::default().fg(Color::Rgb(184, 187, 38)).bold(),
            )
        );

        popup = Popup::info("Test").close_on(KeyCode::Char('q'));
        palette = ThemeName::CatppuccinMocha.palette();
        bottom_title = popup.bottom_title(&palette);

        assert_eq!(
            bottom_title.spans[1],
            Span::styled("<q>", Style::default().fg(Color::Rgb(166, 227, 161)).bold())
        );
    }

    #[test]
    fn should_return_color_based_on_popup_kind_with_theme() {
        let mut popup = Popup::success("Test");
        let mut palette: ThemePalette = ThemeName::GruvboxDark.palette();
        let mut color: Color = popup.color_on_kind(&palette);
        assert_eq!(color, Color::Rgb(184, 187, 38));

        popup = Popup::info("Test");
        color = popup.color_on_kind(&palette);
        assert_eq!(color, Color::Rgb(250, 189, 47));

        popup = Popup::error("Test");
        palette = ThemeName::CatppuccinMocha.palette();
        color = popup.color_on_kind(&palette);
        assert_ne!(color, Color::Rgb(251, 73, 52));
    }
}
