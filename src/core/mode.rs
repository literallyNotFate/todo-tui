use crate::{enums::FocusArea, theme::ThemeColors};
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
    pub fn hotkeys(&self, theme: &ThemeColors, focus: &FocusArea) -> Vec<Line<'static>> {
        let mut lines = Vec::new();

        match self {
            ApplicationMode::Browsing | ApplicationMode::List => {
                self.add_section(&mut lines, "Navigation", theme);
                self.add_command(&mut lines, "h/l", "Panels", theme);
                self.add_command(&mut lines, "A-j/k", "Scroll", theme);

                match focus {
                    FocusArea::LeftPanel => {
                        self.add_command(&mut lines, "j/k", "Filter", theme);
                        self.add_command(&mut lines, "1-5", "Quick", theme);
                    }
                    FocusArea::MainContent => {
                        self.add_command(&mut lines, "j/k", "Select", theme);
                        self.add_command(&mut lines, "J/K", "Move", theme);
                        self.add_command(&mut lines, "[/]", "Desc", theme);
                    }
                }

                self.add_section(&mut lines, "Actions", theme);
                self.add_command(&mut lines, "a", "New", theme);

                if *focus == FocusArea::MainContent {
                    self.add_command(&mut lines, "Enter", "Check", theme);
                    self.add_command(&mut lines, "e", "Edit", theme);
                    self.add_command(&mut lines, "d", "Delete", theme);
                    self.add_command(&mut lines, "/", "Find", theme);
                }

                self.add_command(&mut lines, "x", "Clear", theme);

                self.add_section(&mut lines, "System", theme);
                self.add_command(&mut lines, "s", "Sort", theme);
                self.add_command(&mut lines, "r", "Reverse", theme);
                self.add_command(&mut lines, "t", "Theme", theme);
                self.add_command(&mut lines, "C-s", "Save", theme);
                self.add_command(&mut lines, "A-a", "Autosave", theme);
                self.add_command(&mut lines, "q", "Quit", theme);
            }

            ApplicationMode::Form => {
                self.add_section(&mut lines, "Form", theme);
                self.add_command(&mut lines, "A-Enter", "Submit", theme);
                self.add_command(&mut lines, "◄/►", "Priority", theme);
                self.add_command(&mut lines, "Esc", "Cancel", theme);
            }

            ApplicationMode::Search => {
                self.add_section(&mut lines, "Search", theme);
                self.add_command(&mut lines, "Enter", "Confirm", theme);
                self.add_command(&mut lines, "Esc", "Cancel", theme);
            }
        }

        lines
    }

    /// Adds hotkey command
    fn add_command(
        &self,
        lines: &mut Vec<Line<'static>>,
        key: &'static str,
        desc: &'static str,
        theme: &ThemeColors,
    ) {
        let key_col_width = 11;
        let key_str = format!("{:>width$}", key, width = key_col_width);

        lines.push(Line::from(vec![
            Span::styled(key_str, Style::default().fg(theme.accent).bold()),
            Span::styled(" │ ", Style::default().fg(theme.border)),
            Span::styled(format!("{}", desc), Style::default().fg(theme.text_primary)),
        ]));
    }

    /// Add hotkeys section
    fn add_section(
        &self,
        lines: &mut Vec<Line<'static>>,
        title: &'static str,
        theme: &ThemeColors,
    ) {
        if !lines.is_empty() {
            lines.push(Line::from(""));
        }

        lines.push(
            Line::from(vec![
                Span::styled("──── ", Style::default().fg(theme.accent)),
                Span::styled(title, Style::default().fg(theme.accent).bold()),
                Span::styled(" ────", Style::default().fg(theme.accent)),
            ])
            .centered(),
        );
    }
}

/// Unit-tests for mode hotkeys
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_test_hotkeys_alignment_consistency() {
        let mode = ApplicationMode::Browsing;
        let lines = mode.hotkeys(&ThemeColors::GRUVBOX, &FocusArea::MainContent);

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

        let left_lines = mode.hotkeys(&ThemeColors::GRUVBOX, &FocusArea::LeftPanel);
        let left_content: String = left_lines
            .iter()
            .flat_map(|l| l.spans.iter().map(|s| s.content.as_ref()))
            .collect();

        assert!(left_content.contains("Filter"));
        assert!(!left_content.contains("Select"));

        let main_lines = mode.hotkeys(&ThemeColors::GRUVBOX, &FocusArea::MainContent);
        let main_content: String = main_lines
            .iter()
            .flat_map(|l| l.spans.iter().map(|s| s.content.as_ref()))
            .collect();

        assert!(main_content.contains("Select"));
        assert!(main_content.contains("Check"),);
    }

    #[test]
    fn should_add_section_spacing() {
        let mode = ApplicationMode::Browsing;
        let lines = mode.hotkeys(&ThemeColors::GRUVBOX, &FocusArea::LeftPanel);

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
