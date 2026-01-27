use crate::{enums::ApplicationMode, state::ApplicationState, theme::ThemeColors};
use ratatui::{Frame, layout::Rect};

pub struct MenuBottomBar;

impl MenuBottomBar {
    pub fn render(
        frame: &mut Frame,
        area: Rect,
        state: &ApplicationState,
        theme: &ThemeColors,
        mode: &ApplicationMode,
    ) {
        use ratatui::{
            layout::{Alignment, Constraint, Direction, Layout},
            style::{Color, Stylize},
            widgets::Paragraph,
        };

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1), // Hotkeys
                Constraint::Length(2),
                Constraint::Length(1), // Stats + Messages + Status
            ])
            .split(area);

        let hotkeys: &str = match mode {
            ApplicationMode::Browsing => {
                " Esc/q:Quit │ Enter:Toggle │ a:Add │ e:Edit │ d:Delete │ x:Clear │ t:Theme │ <C-s>:Save │ h/l:Focus │ ▲/▼/j/k:Navigate │ J/K:Move "
            }
            ApplicationMode::Task => " Esc:Cancel │ ▲/▼:Next │ ◄/►:Priority ",
        };
        frame.render_widget(
            Paragraph::new(hotkeys).centered().fg(theme.text_dim),
            chunks[0],
        );

        let status_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(30), // App name
                Constraint::Percentage(40), // Notification
                Constraint::Percentage(30), // Status
            ])
            .split(chunks[2]);

        frame.render_widget(
            Paragraph::new("todo-tui").fg(theme.accent),
            status_layout[0],
        );

        if let Some(n) = &state.notification {
            if !n.is_expired() {
                n.render(frame, status_layout[1], theme);
            }
        }

        let (status_str, status_color): (&str, Color) = if state.any_unsaved_changes() {
            ("● Unsaved ", theme.error)
        } else {
            ("✓ Saved ", theme.success)
        };

        frame.render_widget(
            Paragraph::new(status_str)
                .alignment(Alignment::Right)
                .fg(status_color),
            status_layout[2],
        );
    }
}
