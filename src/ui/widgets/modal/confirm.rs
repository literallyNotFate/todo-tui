use crate::{
    core::Action,
    theme::ThemePalette,
    traits::{Modal, ModalResult, ModalSize},
    ui::{RenderContext, center},
};
use ratatui::{
    crossterm::event::KeyCode,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Style, Stylize},
    text::{Line, Span},
};

/// Confirm selection options
#[derive(Debug, Clone, PartialEq)]
pub enum ConfirmOption {
    Yes,
    Cancel,
}

/// Popup modal widget
pub struct Confirm {
    pub message: String,
    pub select: ConfirmOption,
    pub size: ModalSize,
}

impl Confirm {
    /// New confirm widget
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            select: ConfirmOption::Cancel,
            size: ModalSize::Small,
        }
    }

    /// With modal size
    pub fn with_size(mut self, size: ModalSize) -> Self {
        self.size = size;
        self
    }

    /// Vertical layout for inner content
    fn vertical_layout(&self, area: Rect) -> std::rc::Rc<[Rect]> {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Fill(1),
                Constraint::Length(2),
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Fill(1),
            ])
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

    /// Style for buttons based on selection
    fn button_styles(&self, palette: &ThemePalette) -> (Style, Style) {
        match self.select {
            ConfirmOption::Yes => (
                Style::default().fg(palette.success).bold(),
                Style::default().fg(palette.muted),
            ),
            ConfirmOption::Cancel => (
                Style::default().fg(palette.muted),
                Style::default().fg(palette.error).bold(),
            ),
        }
    }

    /// Render buttons
    fn button_line(&self, styles: (Style, Style), palette: &ThemePalette) -> Line<'static> {
        Line::from(vec![
            Span::styled("[ ", Style::default().fg(palette.fg)),
            Span::styled("Yes", styles.0),
            Span::styled(" ]", Style::default().fg(palette.fg)),
            Span::raw("    "),
            Span::styled("[ ", Style::default().fg(palette.fg)),
            Span::styled("Cancel", styles.1),
            Span::styled(" ]", Style::default().fg(palette.fg)),
        ])
    }

    /// Bottom hotkeys
    fn bottom_keys(&self, palette: &ThemePalette) -> Line<'static> {
        Line::from(vec![
            Span::styled(" y", Style::default().fg(palette.success).bold()),
            Span::styled(":yes ", Style::default().fg(palette.muted)),
            Span::styled("n", Style::default().fg(palette.error).bold()),
            Span::styled(":no ", Style::default().fg(palette.muted)),
            Span::styled(" <h/l>", Style::default().fg(palette.accent)),
            Span::styled(":move ", Style::default().fg(palette.muted)),
        ])
    }
}

impl Modal for Confirm {
    /// Calculate area for confirm
    fn area(&self, frame_area: Rect) -> Rect {
        let (width, height) = self.size.percentages();
        center(frame_area, width, height)
    }

    /// Confirm rendering
    fn render(&self, ctx: &mut RenderContext, area: Rect) {
        use ratatui::{
            style::Stylize,
            widgets::{Block, Paragraph, Wrap},
        };

        let palette: ThemePalette = ctx.palette();
        let confirm_block: Block = Block::bordered()
            .fg(palette.fg)
            .bg(palette.bg)
            .border_style(Style::default().fg(palette.info))
            .title_top(Line::from(" Action ").centered())
            .title_bottom(self.bottom_keys(&palette).centered())
            .border_type(ctx.config.border_type.into());

        let inner_area: Rect = confirm_block.inner(area);
        ctx.render_widget(confirm_block.clone(), area);

        let vertical_chunks: std::rc::Rc<[Rect]> = self.vertical_layout(inner_area);

        let message_area: Rect = self.horizontal_layout(vertical_chunks[1])[1];
        let buttons_area: Rect = self.horizontal_layout(vertical_chunks[3])[1];

        let message: Paragraph = Paragraph::new(self.message.clone())
            .centered()
            .wrap(Wrap { trim: true });

        ctx.render_widget(message, message_area);

        let button_styles: (Style, Style) = self.button_styles(&palette);
        let buttons: Line = self.button_line(button_styles, &palette);

        let buttons_widget = Paragraph::new(buttons).centered();
        ctx.render_widget(buttons_widget, buttons_area);
    }

    /// Key event handling
    fn handle_action(&mut self, action: Option<Action>, key: KeyCode) -> Option<ModalResult> {
        match key {
            KeyCode::Enter => {
                return Some(match self.select {
                    ConfirmOption::Yes => ModalResult::Confirmed,
                    ConfirmOption::Cancel => ModalResult::Cancelled,
                });
            }
            KeyCode::Esc | KeyCode::Char('n') => return Some(ModalResult::Cancelled),
            KeyCode::Char('y') => return Some(ModalResult::Confirmed),
            _ => {}
        }

        if let Some(act) = action {
            match act {
                Action::MoveRight => self.select = ConfirmOption::Cancel,
                Action::MoveLeft => self.select = ConfirmOption::Yes,
                _ => {}
            }
        }

        None
    }
}

/// Unit-tests for confirm widget
#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::ThemeName;

    fn create_helper_frame() -> Rect {
        Rect::new(0, 0, 100, 100)
    }

    #[test]
    fn should_create_default_confirm() {
        let confirm: Confirm = Confirm::new("Test");

        assert_eq!(confirm.message, "Test");
        assert_eq!(confirm.select, ConfirmOption::Cancel);
    }

    #[test]
    fn should_calculate_dynamic_area_for_confirm() {
        let frame: Rect = create_helper_frame();
        let small_confirm = Confirm::new("Small").with_size(ModalSize::Small);
        let large_confirm = Confirm::new("Large").with_size(ModalSize::Large);

        let small_area = small_confirm.area(frame);
        let large_area = large_confirm.area(frame);

        assert!(large_area.width > small_area.width);
        assert!(large_area.height > small_area.height);
    }

    #[test]
    fn should_handle_left_right_key_confirm() {
        let mut confirm: Confirm = Confirm::new("Test");
        assert_eq!(confirm.select, ConfirmOption::Cancel);

        confirm.handle_action(Some(Action::MoveLeft), KeyCode::Null);
        assert_eq!(confirm.select, ConfirmOption::Yes);

        confirm.handle_action(Some(Action::MoveRight), KeyCode::Null);
        assert_eq!(confirm.select, ConfirmOption::Cancel);
    }

    #[test]
    fn should_handle_key_enter_confirm() {
        let mut confirm: Confirm = Confirm::new("Test");
        confirm.select = ConfirmOption::Yes;
        assert_eq!(
            confirm.handle_action(None, KeyCode::Enter),
            Some(ModalResult::Confirmed)
        );

        confirm.select = ConfirmOption::Cancel;
        assert_eq!(
            confirm.handle_action(None, KeyCode::Enter),
            Some(ModalResult::Cancelled)
        );
    }

    #[test]
    fn should_handle_key_esc_confirm() {
        let mut confirm: Confirm = Confirm::new("Test");

        assert_eq!(
            confirm.handle_action(None, KeyCode::Esc),
            Some(ModalResult::Cancelled)
        );
    }

    #[test]
    fn should_handle_key_other_keys_confirm() {
        let mut confirm: Confirm = Confirm::new("Test");

        assert_eq!(confirm.handle_action(None, KeyCode::Char('a')), None);
        assert_eq!(confirm.handle_action(None, KeyCode::Down), None);
        assert_eq!(confirm.select, ConfirmOption::Cancel);

        assert_eq!(
            confirm.handle_action(None, KeyCode::Char('y')),
            Some(ModalResult::Confirmed)
        );
        assert_eq!(
            confirm.handle_action(None, KeyCode::Char('n')),
            Some(ModalResult::Cancelled)
        );
    }

    #[test]
    fn should_return_proper_styles_for_buttons_with_theme() {
        let mut confirm = Confirm::new("Test");
        let palette = ThemeName::GruvboxDark.palette();
        let mut styles = confirm.button_styles(&palette);

        assert_eq!(styles.0, Style::default().fg(palette.muted));
        assert_eq!(styles.1, Style::default().fg(palette.error).bold());

        confirm.select = ConfirmOption::Yes;
        styles = confirm.button_styles(&palette);

        assert_eq!(styles.0, Style::default().fg(palette.success).bold());
        assert_eq!(styles.1, Style::default().fg(palette.muted));
    }

    #[test]
    fn should_generate_buttons_with_theme() {
        let mut confirm = Confirm::new("Test");
        let palette = ThemeName::GruvboxDark.palette();

        let styles = confirm.button_styles(&palette);
        let buttons = confirm.button_line(styles, &palette);

        let expected_line = Line::from(vec![
            Span::styled("[ ", Style::default().fg(palette.fg)),
            Span::styled("Yes", Style::default().fg(palette.muted)),
            Span::styled(" ]", Style::default().fg(palette.fg)),
            Span::raw("    "),
            Span::styled("[ ", Style::default().fg(palette.fg)),
            Span::styled("Cancel", Style::default().fg(palette.error).bold()),
            Span::styled(" ]", Style::default().fg(palette.fg)),
        ]);

        assert_eq!(buttons, expected_line);

        confirm.select = ConfirmOption::Yes;
        let styles = confirm.button_styles(&palette);
        let buttons = confirm.button_line(styles, &palette);

        let expected_line_yes = Line::from(vec![
            Span::styled("[ ", Style::default().fg(palette.fg)),
            Span::styled("Yes", Style::default().fg(palette.success).bold()),
            Span::styled(" ]", Style::default().fg(palette.fg)),
            Span::raw("    "),
            Span::styled("[ ", Style::default().fg(palette.fg)),
            Span::styled("Cancel", Style::default().fg(palette.muted)),
            Span::styled(" ]", Style::default().fg(palette.fg)),
        ]);

        assert_eq!(buttons, expected_line_yes);
    }
}
