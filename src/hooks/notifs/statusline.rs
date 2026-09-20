//! Pure StatusLine notification producers.
//!
//! Maps to the mount-time warning in official `components/StatusLine.tsx` when
//! `executeStatusLineCommand(...)` is skipped because workspace trust has not
//! been accepted. This module only builds a main-screen runtime notification;
//! it does not execute commands or mutate trust/settings state.

use crate::context::notifications::{Notification, NotificationColor, NotificationPriority};
use crate::utils::settings::SettingsJson;

pub const STATUS_LINE_TRUST_BLOCKED_NOTIFICATION_KEY: &str = "statusline-trust-blocked";

pub fn status_line_trust_blocked_notification(
    settings: &SettingsJson,
    workspace_trusted: bool,
) -> Option<Notification> {
    if workspace_trusted {
        return None;
    }
    if !settings
        .status_line
        .as_ref()
        .is_some_and(|status_line| status_line.is_command_type())
    {
        return None;
    }

    Some(
        Notification::text(
            STATUS_LINE_TRUST_BLOCKED_NOTIFICATION_KEY,
            "statusline skipped · restart to fix",
            NotificationPriority::Low,
        )
        .with_color(NotificationColor::Warning),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::settings::types::StatusLineSettings;

    fn settings_with_status_line() -> SettingsJson {
        SettingsJson {
            status_line: Some(StatusLineSettings {
                kind: Some("command".to_string()),
                command: "printf status".to_string(),
                padding: None,
            }),
            ..Default::default()
        }
    }

    #[test]
    fn status_line_trust_blocked_notification_matches_official_mount_warning() {
        let notification =
            status_line_trust_blocked_notification(&settings_with_status_line(), false)
                .expect("configured untrusted status line should enqueue warning");

        assert_eq!(notification.key, STATUS_LINE_TRUST_BLOCKED_NOTIFICATION_KEY);
        assert_eq!(notification.text, "statusline skipped · restart to fix");
        assert_eq!(notification.priority, NotificationPriority::Low);
        assert_eq!(notification.color, Some(NotificationColor::Warning));
    }

    #[test]
    fn status_line_trust_blocked_notification_requires_configured_command_and_untrusted_workspace()
    {
        assert!(status_line_trust_blocked_notification(&SettingsJson::default(), false).is_none());
        assert!(
            status_line_trust_blocked_notification(&settings_with_status_line(), true).is_none()
        );
    }
}
