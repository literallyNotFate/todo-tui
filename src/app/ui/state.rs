use super::{
    dialogs::dialog::Dialog,
    widgets::{input::input::Input, notification::notification::Notification},
};

// Defines where widget is going to be rendered
#[derive(Default, Debug, Clone)]
pub enum WidgetPosition {
    #[default]
    Center,

    TopRight,
    TopLeft,
    BottomRight,
    BottomLeft,
}

// Dialog actions (remove todo, clear, save, load, none - for popup)
pub enum DialogIntent {
    None,
    Remove,
    Clear,
}

#[derive(Default)]
pub struct UIState {
    pub dialog: Option<ActiveDialog>,
    pub input: Option<Input>,
    pub notification: Option<Notification>,
}

pub struct ActiveDialog {
    pub modal: Box<dyn Dialog>,
    pub intent: DialogIntent,
}

impl UIState {
    // Dialog
    pub fn show_dialog<D: Dialog + 'static>(&mut self, dialog: D, intent: DialogIntent) {
        self.dialog = Some(ActiveDialog {
            modal: Box::new(dialog),
            intent,
        });
    }

    pub fn close_dialog(&mut self) {
        self.dialog = None;
    }

    // Input
    pub fn show_input(&mut self, input: Input) {
        self.input = Some(input);
    }

    pub fn close_input(&mut self) {
        self.input = None;
    }

    // Notification
    pub fn show_notification(&mut self, notification: Notification) {
        self.notification = Some(notification);
    }
}
