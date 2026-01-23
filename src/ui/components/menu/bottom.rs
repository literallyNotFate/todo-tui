use crate::{enums::ApplicationMode, state::ApplicationState};
use ratatui::{Frame, layout::Rect};

pub struct MenuBottomBar;

impl MenuBottomBar {
    pub fn render(frame: &mut Frame, area: Rect, state: &ApplicationState, mode: &ApplicationMode) {
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
                " Esc/q:Quit │ Enter:Toggle │ a:Add │ e:Edit │ d:Delete │ x:Clear │ <C-s>:Save │ h/l:Focus | ▲/▼/j/k:Filter "
            }
            ApplicationMode::Task => " Esc:Cancel │ Enter:Save │ ▲/▼:Next │ ◄/►:Priority ",
        };
        frame.render_widget(Paragraph::new(hotkeys).fg(Color::DarkGray), chunks[0]);

        let status_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(30), // Stats
                Constraint::Percentage(40), // Notification
                Constraint::Percentage(30), // Status
            ])
            .split(chunks[2]);

        let (total, active): (usize, usize) = state.stats();
        let stats_text = format!(" Tasks: {} active / {} total", active, total);
        frame.render_widget(Paragraph::new(stats_text).fg(Color::Cyan), status_layout[0]);

        if let Some(n) = &state.notification {
            if !n.is_expired() {
                n.render(frame, status_layout[1]);
            }
        }

        let (status_str, status_color): (&str, Color) = if state.any_unsaved_changes() {
            ("● Unsaved ", Color::LightRed)
        } else {
            ("✓ Saved ", Color::Green)
        };

        frame.render_widget(
            Paragraph::new(status_str)
                .alignment(Alignment::Right)
                .fg(status_color),
            status_layout[2],
        );
    }
}
