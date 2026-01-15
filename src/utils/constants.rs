use ratatui::style::Color;

// Terminal constants for fallback message (if small)
pub mod terminal {
    use super::{
        Color,
        theme::{COLOR_GREEN, COLOR_RED},
    };
    use ratatui::layout::Rect;

    pub const MIN_WIDTH: u16 = 80;
    pub const MIN_HEIGHT: u16 = 24;

    pub fn is_terminal_small(width: u16, height: u16) -> bool {
        width < MIN_WIDTH || height < MIN_HEIGHT
    }

    pub fn dimension_colors(area: Rect) -> (Color, Color) {
        let width_color: Color = if area.width >= MIN_WIDTH {
            COLOR_GREEN
        } else {
            COLOR_RED
        };

        let height_color: Color = if area.height >= MIN_HEIGHT {
            COLOR_GREEN
        } else {
            COLOR_RED
        };

        (width_color, height_color)
    }
}

// Colors palette for application
pub mod theme {
    use super::Color;

    pub const BG_PRIMARY: Color = Color::Rgb(25, 25, 25);
    pub const BG_DIM: Color = Color::Rgb(15, 15, 15);

    pub const TEXT_PRIMARY: Color = Color::Rgb(252, 252, 252);
    pub const TEXT_DIMMED: Color = Color::Rgb(120, 120, 120);

    pub const ITEM_LIST_PRIMARY: Color = Color::Rgb(249, 192, 122);
    pub const ITEM_LIST_SELECTED: Color = Color::Rgb(249, 158, 47);

    pub const SUCCESS_POPUP_FG: Color = Color::Rgb(144, 185, 159);
    pub const ERROR_POPUP_FG: Color = Color::Rgb(245, 161, 145);
    pub const HELP_POPUP_FG: Color = Color::Rgb(226, 158, 202);
    pub const INFO_POPUP_FG: Color = Color::Rgb(172, 161, 207);

    pub const INPUT_ADD_FG: Color = Color::Rgb(245, 161, 145);
    pub const INPUT_EDIT_FG: Color = Color::Rgb(234, 141, 165);

    pub const CONFIRM_YES_FG_ACTIVE: Color = Color::Rgb(180, 230, 190);
    pub const CONFIRM_CANCEL_FG_ACTIVE: Color = Color::Rgb(230, 180, 180);

    pub const COLOR_GREEN: Color = Color::Rgb(152, 195, 121);
    pub const COLOR_ORANGE: Color = Color::Rgb(252, 223, 108);
    pub const COLOR_RED: Color = Color::Rgb(255, 180, 180);
    pub const COLOR_YELLOW: Color = Color::Rgb(252, 244, 0);
    pub const COLOR_BLUE: Color = Color::Rgb(130, 117, 249);
    pub const COLOR_PURPLE: Color = Color::Rgb(196, 117, 249);
}

// Prepared text for components
pub mod text {
    pub const HELP_MESSAGE_TEXT: &str = "a -> append a todo
r -> rename a todo
d -> delete a todo
x -> clear all todos
Enter -> mark as completed
k/Up -> go up
j/Down -> go down
<C-s> -> save todos
q/Esc -> quit
? -> toggle help";

    pub const REMOVED_TASK_TEXT: &str = "Task was removed!";
    pub const CLEARED_TASKS_TEXT: &str = "Task were cleared!";
    pub const SAVED_TASKS_TEXT: &str = "Tasks were saved!";
    pub const UNSAVED_EXIT_TEXT: &str = "You have unsaved changes. Save before exit?";
}

// Default sizes
pub mod size {
    pub const POPUP_PERCENTAGE_WIDTH: f32 = 70.0;
    pub const CONFIRM_PERCENTAGE_WIDTH: f32 = 60.0;
    pub const NOTIFICATION_PERCENTAGE_WIDTH: f32 = 30.0;

    pub const INPUT_MAX_CHARS: usize = 46;
    pub const INPUT_WIDTH: u16 = 50;
    pub const INPUT_HEIGHT: u16 = 3;

    pub const FALLBACK_WIDTH: u16 = 50;
    pub const FALLBACK_HEIGHT: u16 = 10;
}
