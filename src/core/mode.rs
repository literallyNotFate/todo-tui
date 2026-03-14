use crate::{
    enums::FocusArea,
    theme::{Theme, ThemePalette},
};
use ratatui::{
    style::{Style, Stylize},
    text::{Line, Span},
};

/// Which mode user selects (for input handling)
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ApplicationMode {
    Browsing,
    List,
    Form,
    Search,
}

impl ApplicationMode {
    /// Returns lines of hotkeys (w/commands and sections)
    pub fn hotkeys(&self, theme: &Theme, focus: &FocusArea) -> Vec<Line<'static>> {
        let mut lines = Vec::new();
        let palette: &ThemePalette = &theme.palette();

        match self {
            ApplicationMode::Browsing | ApplicationMode::List => {
                self.add_section(&mut lines, "Navigation", palette);
                self.add_command(&mut lines, "h/l", "Switch focus", palette);

                match focus {
                    FocusArea::LeftPanel => {
                        self.add_command(&mut lines, "j/k", "Change filter", palette);
                        self.add_command(&mut lines, "1-5", "Quick filter", palette);
                    }
                    FocusArea::MainContent => {
                        self.add_command(&mut lines, "j/k", "Select task", palette);
                        self.add_command(&mut lines, "J/K", "Move task", palette);
                    }
                }

                self.add_section(&mut lines, "Actions", palette);
                self.add_command(&mut lines, "a", "New task", palette);

                if *focus == FocusArea::MainContent {
                    self.add_command(&mut lines, "Enter", "Mark completed", palette);
                    self.add_command(&mut lines, "e", "Update task", palette);
                    self.add_command(&mut lines, "d", "Remove task", palette);
                    self.add_command(&mut lines, "i/Tab", "Show details", palette);
                    self.add_command(&mut lines, "/", "Search task", palette);
                }

                self.add_command(&mut lines, "x", "Clear tasks", palette);

                self.add_section(&mut lines, "System", palette);
                self.add_command(&mut lines, "s", "Sort tasks", palette);
                self.add_command(&mut lines, "r", "Reverse sort", palette);
                self.add_command(&mut lines, "t", "Next theme", palette);
                self.add_command(&mut lines, "<C-t>", "Previous theme", palette);
                self.add_command(&mut lines, "m", "Change theme mode", palette);
                self.add_command(&mut lines, "b", "Toggle sidebar", palette);
                self.add_command(&mut lines, "C-s", "Save", palette);
                self.add_command(&mut lines, "A-a", "Toggle autosave", palette);
                self.add_command(&mut lines, "q", "Quit", palette);
            }
            _ => {}
        }

        lines
    }

    /// Adds hotkey command
    fn add_command(
        &self,
        lines: &mut Vec<Line<'static>>,
        key: &'static str,
        desc: &'static str,
        palette: &ThemePalette,
    ) {
        let key_col_width = 11;
        let key_str = format!("{:>width$}", key, width = key_col_width);

        lines.push(Line::from(vec![
            Span::styled(key_str, Style::default().fg(palette.info).bold()),
            Span::styled(" │ ", Style::default().fg(palette.muted)),
            Span::styled(format!("{}", desc), Style::default().fg(palette.fg)),
        ]));
    }

    /// Add hotkeys section
    fn add_section(
        &self,
        lines: &mut Vec<Line<'static>>,
        title: &'static str,
        palette: &ThemePalette,
    ) {
        if !lines.is_empty() {
            lines.push(Line::from(""));
        }

        lines.push(
            Line::from(vec![
                Span::styled("──── ", Style::default().fg(palette.accent)),
                Span::styled(title, Style::default().fg(palette.accent).bold()),
                Span::styled(" ────", Style::default().fg(palette.accent)),
            ])
            .centered(),
        );
    }
}

/// Unit-tests for mode hotkeys
#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::ThemeName;

    #[test]
    fn should_test_hotkeys_alignment_consistency() {
        let mode = ApplicationMode::Browsing;
        let lines = mode.hotkeys(&ThemeName::GruvboxDark.into(), &FocusArea::MainContent);

        for line in lines {
            let spans: Vec<_> = line.spans.iter().collect();
            if spans.len() >= 3 && spans[1].content.contains('│') {
                assert_eq!(spans[0].content.len(), 11, "Key column width must be 11");
            }
        }
    }

    #[test]
    fn should_handle_hotkeys_focus_logic() {
        let mode = ApplicationMode::Browsing;

        let left_lines = mode.hotkeys(&ThemeName::GruvboxDark.into(), &FocusArea::LeftPanel);
        let left_content: String = left_lines
            .iter()
            .flat_map(|l| l.spans.iter().map(|s| s.content.as_ref()))
            .collect();

        assert!(!left_content.contains("Select"));

        let main_lines = mode.hotkeys(&ThemeName::GruvboxDark.into(), &FocusArea::MainContent);
        let main_content: String = main_lines
            .iter()
            .flat_map(|l| l.spans.iter().map(|s| s.content.as_ref()))
            .collect();

        assert!(main_content.contains("Select"));
        assert!(main_content.contains("Toggle"));
    }

    #[test]
    fn should_add_section_spacing() {
        let mode = ApplicationMode::Browsing;
        let lines = mode.hotkeys(&ThemeName::GruvboxDark.into(), &FocusArea::LeftPanel);

        let empty_lines = lines
            .iter()
            .filter(|l| l.spans.is_empty() || (l.spans.len() == 1 && l.spans[0].content == ""))
            .count();
        assert!(
            empty_lines >= 2,
            "Sections should be separated by empty lines"
        );
    }
}
