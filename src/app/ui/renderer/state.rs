use crate::app::{
    state::error::ApplicationStateError,
    ui::{
        dialogs::dialog::{Dialog, DialogIntent},
        widgets::{input::input::Input, notification::notification::Notification},
    },
};

#[derive(Default, Debug, Clone, PartialEq)]
pub enum Anchor {
    #[default]
    Center,
    TopRight,
    TopLeft,
    BottomRight,
    BottomLeft,
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

    pub fn notify<T>(&mut self, result: Result<T, ApplicationStateError>, success_message: &str) {
        match result {
            Ok(_) => {
                self.show_notification(Notification::success(success_message));
            }
            Err(err) => {
                self.show_notification(Notification::error(err.to_string()));
            }
        }
    }
}
