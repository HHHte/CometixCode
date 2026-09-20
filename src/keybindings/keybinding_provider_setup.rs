//! Maps to: CC `keybindings/KeybindingProviderSetup.tsx`.
//!
//! iocraft adaptation: the interceptor is a bypass terminal-event listener.
//! It observes every key while per-component hooks retain ownership of
//! single-keystroke propagation. Startup bindings and watcher I/O are prepared
//! before this retained hook mounts.

use iocraft::prelude::*;
use std::time::Duration;

use super::keybinding_context::KeybindingRuntime;
use super::matcher::key_event_to_keystroke;

/// Maps to: CC `KeybindingProviderSetup.tsx` `CHORD_TIMEOUT_MS`.
pub(super) const CHORD_TIMEOUT: Duration = Duration::from_millis(1000);
const WARNING_NOTIFICATION_KEY: &str = "keybinding-config-warning";

/// Maps to: CC `useKeybindingWarnings()` notification projection.
pub fn sync_keybinding_warning_notification(
    store: &crate::state::store::AppStore,
    warnings: &[crate::keybindings::validate::KeybindingWarning],
) {
    use crate::context::notifications::{
        Notification, NotificationColor, NotificationPriority, NotificationsWriter,
    };
    use crate::keybindings::types::KeybindingWarningSeverity;

    let mut writer = NotificationsWriter::new(store.clone());
    if warnings.is_empty() {
        writer.remove_notification(WARNING_NOTIFICATION_KEY);
        return;
    }
    let errors = warnings
        .iter()
        .filter(|warning| warning.severity == KeybindingWarningSeverity::Error)
        .count();
    let warning_count = warnings.len() - errors;
    let plural = |count: usize, singular: &str| {
        if count == 1 {
            singular.to_string()
        } else {
            format!("{singular}s")
        }
    };
    let message = if errors > 0 && warning_count > 0 {
        format!(
            "Found {errors} keybinding {} and {warning_count} {}",
            plural(errors, "error"),
            plural(warning_count, "warning")
        )
    } else if errors > 0 {
        format!("Found {errors} keybinding {}", plural(errors, "error"))
    } else {
        format!(
            "Found {warning_count} keybinding {}",
            plural(warning_count, "warning")
        )
    } + " · /doctor for details";
    writer.add_notification(
        Notification::text(
            WARNING_NOTIFICATION_KEY,
            message,
            if errors > 0 {
                NotificationPriority::Immediate
            } else {
                NotificationPriority::High
            },
        )
        .with_color(if errors > 0 {
            NotificationColor::Error
        } else {
            NotificationColor::Warning
        })
        .with_timeout_ms(60_000),
    );
}

/// Mount the keybinding runtime + ChordInterceptor. Call once from the
/// interactive launcher root and inject the returned runtime via ContextProvider.
/// Maps to: CC `<KeybindingSetup>` wrapping `<ChordInterceptor>` + children.
pub fn use_keybinding_setup(
    hooks: &mut Hooks,
    startup_runtime: KeybindingRuntime,
) -> KeybindingRuntime {
    let runtime_state = hooks.use_state(move || startup_runtime);
    let runtime = runtime_state.read().clone();

    // ChordInterceptor: bypass listener — observes every key regardless of
    // downstream consumption (see module docs for why swallowing is not
    // needed at this stage).
    hooks.use_terminal_events({
        let runtime = runtime.clone();
        move |event| {
            if let TerminalEvent::Key(key_event) = &event {
                if let Some(keystroke) = key_event_to_keystroke(key_event) {
                    runtime.observe_keystroke(&keystroke);
                }
            }
        }
    });

    runtime
}

/// Test harness provider for component-level single-key action tests. Chord
/// interceptor tests mount `use_keybinding_setup` explicitly.
#[cfg(test)]
pub fn test_keybinding_root(child: AnyElement<'static>) -> AnyElement<'static> {
    element! {
        ContextProvider(value: Context::owned(KeybindingRuntime::with_default_bindings())) {
            #(vec![child])
        }
    }
    .into_any()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::notifications::{NotificationColor, NotificationPriority};
    use crate::keybindings::validate::KeybindingWarning;
    use crate::state::app_state_store::AppState;
    use crate::state::store::AppStore;

    #[test]
    fn warning_projection_matches_official_notification_copy_and_priority() {
        crate::utils::process_runtime::initialize_test_process_runtime();
        let store = AppStore::new(AppState::default(), None);
        sync_keybinding_warning_notification(
            &store,
            &[
                KeybindingWarning::error("bad binding", None::<String>),
                KeybindingWarning::warning("risky binding", None::<String>),
            ],
        );
        let current = store
            .get()
            .notifications
            .current
            .clone()
            .expect("immediate keybinding warning");
        assert_eq!(
            current.text,
            "Found 1 keybinding error and 1 warning · /doctor for details"
        );
        assert_eq!(current.priority, NotificationPriority::Immediate);
        assert_eq!(current.color, Some(NotificationColor::Error));
        assert_eq!(current.timeout_ms, Some(60_000));

        sync_keybinding_warning_notification(&store, &[]);
        assert!(store.get().notifications.current.is_none());
    }
}
