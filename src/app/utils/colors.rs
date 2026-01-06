use ratatui::style::Color;

// Colors palette for application
pub mod theme {
    use super::Color;

    pub const BG_PRIMARY: Color = Color::Rgb(25, 25, 25);
    pub const BG_DIM: Color = Color::Rgb(15, 15, 15);

    pub const TEXT_PRIMARY: Color = Color::Rgb(252, 252, 252);
    pub const TEXT_DIMMED: Color = Color::Rgb(120, 120, 120);
    pub const TEXT_SELECTED: Color = Color::Rgb(229, 218, 156);

    pub const SUCCESS_POPUP_FG: Color = Color::Rgb(144, 185, 159);
    pub const ERROR_POPUP_FG: Color = Color::Rgb(245, 161, 145);
    pub const HELP_POPUP_FG: Color = Color::Rgb(226, 158, 202);
    pub const INFO_POPUP_FG: Color = Color::Rgb(172, 161, 207);

    pub const INPUT_ADD_FG: Color = Color::Rgb(245, 161, 145);
    pub const INPUT_EDIT_FG: Color = Color::Rgb(234, 141, 165);

    pub const CONFIRM_YES_FG_ACTIVE: Color = Color::Rgb(180, 230, 190);
    pub const CONFIRM_CANCEL_FG_ACTIVE: Color = Color::Rgb(230, 180, 180);

    pub const COLOR_GREEN: Color = Color::Rgb(165, 252, 115);
    pub const COLOR_ORANGE: Color = Color::Rgb(252, 223, 108);
    pub const COLOR_RED: Color = Color::Rgb(255, 180, 180);
    pub const COLOR_YELLOW: Color = Color::Rgb(252, 244, 0);
}
