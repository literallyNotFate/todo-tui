use ratatui::style::Color;

// Colors palette for application
pub mod theme {
    use super::Color;

    pub const BG_DIM: Color = Color::Rgb(15, 15, 15);
    pub const TEXT_PRIMARY: Color = Color::Rgb(252, 252, 252);
    pub const TEXT_DIMMED: Color = Color::Rgb(120, 120, 120);
}

// Default sizes
pub mod size {
    pub const POPUP_WIDTH: u16 = 40;
    pub const POPUP_HEIGHT: u16 = 25;

    pub const CONFIRM_WIDTH: u16 = 30;
    pub const CONFIRM_HEIGHT: u16 = 25;

    pub const TEXT_INPUT_MAX_CHARS: usize = 256;
}
