use crate::theme::ThemeName;
use ratatui::style::Color;

/// A semantic color palette for a theme
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ThemePalette {
    /// Primary accent color for highlights and active elements
    pub accent: Color,

    /// Secondary accent color for less prominent highlights
    pub secondary: Color,

    /// Main background color
    pub bg: Color,

    /// Primary foreground/text color
    pub fg: Color,

    /// Muted/dimmed text color
    pub muted: Color,

    /// Selection/highlight background color
    pub selection: Color,

    /// Error/red color for critical states
    pub error: Color,

    /// Warning/yellow color for caution states
    pub warning: Color,

    /// Success/green color for positive states
    pub success: Color,

    /// Info/blue color for informational states
    pub info: Color,
}

impl ThemePalette {
    /// Check if this is a light theme based on background brightness
    pub fn is_light(&self) -> bool {
        if let Color::Rgb(r, g, b) = self.bg {
            let brightness = (u32::from(r) * 299 + u32::from(g) * 587 + u32::from(b) * 114) / 1000;
            brightness > 127
        } else {
            false
        }
    }

    /// Check if this is a dark theme
    pub fn is_dark(&self) -> bool {
        !self.is_light()
    }
}

impl Default for ThemePalette {
    /// Returns the default palette
    fn default() -> Self {
        ThemeName::default().palette()
    }
}
