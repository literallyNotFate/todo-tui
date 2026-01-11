// Unit-tests for UIState
#[cfg(test)]
mod tests {
    use crate::app::{
        state::error::ApplicationStateError,
        ui::{
            dialogs::{
                dialog::{Dialog, DialogIntent},
                popup::popup::Popup,
            },
            renderer::state::{Anchor, UIState},
            widgets::{
                input::input::Input,
                notification::notification::{Notification, NotificationKind},
            },
        },
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
