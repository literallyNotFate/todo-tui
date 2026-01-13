use crate::{
    state::ApplicationResult,
    ui::{Dialog, DialogIntent, Input, Notification},
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

    pub fn expire_notification(&mut self) {
        if let Some(n) = &self.notification
            && n.is_expired()
        {
            self.notification = None;
        }
    }

    pub fn notify(&mut self, result: ApplicationResult<String>) {
        match result {
            Ok(msg) => self.show_notification(Notification::success(msg)),
            Err(err) => self.show_notification(Notification::error(err.to_string())),
        }
    }
}

// Unit-tests for UIState
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        state::ApplicationStateError,
        ui::{NotificationKind, Popup},
    };
    use std::{
        thread::sleep,
        time::{Duration, Instant},
    };

    #[test]
    fn should_show_close_dialog_with_ui_state() {
        let mut ui = UIState::default();

        ui.show_dialog(Popup::new(), DialogIntent::Remove);

        assert!(ui.dialog.is_some());
        assert_eq!(ui.dialog.as_ref().unwrap().intent, DialogIntent::Remove);

        ui.close_dialog();
        assert!(ui.dialog.is_none());
    }

    #[test]
    fn should_show_close_input_with_ui_state() {
        let mut ui = UIState::default();

        ui.show_input(Input::insert());
        assert!(ui.input.is_some());

        ui.close_input();
        assert!(ui.input.is_none());
    }

    #[test]
    fn should_expire_notification_with_ui_state() {
        let mut ui = UIState::default();

        let expired: Notification = Notification {
            created_at: Instant::now(),
            duration: Duration::from_millis(100),
            anchor: Anchor::TopRight,
            kind: NotificationKind::Success,
            message: String::from("Test"),
        };
        ui.notification = Some(expired);

        sleep(Duration::from_millis(100));

        ui.expire_notification();
        assert!(ui.notification.is_none());
    }

    #[test]
    fn should_notify_with_ui_state() {
        let mut ui = UIState::default();

        ui.notify(Ok("Success".to_string()));

        let success_notification: Notification =
            ui.notification.take().expect("Could not get notification");
        assert_eq!(success_notification.message, "Success");
        assert_eq!(success_notification.kind, NotificationKind::Success);

        ui.notify(Err(ApplicationStateError::TaskNotSelected));

        let error_notification: Notification = ui.notification.unwrap();
        assert_eq!(error_notification.message, "No task was selected!");
        assert_eq!(error_notification.kind, NotificationKind::Error);
    }
}
