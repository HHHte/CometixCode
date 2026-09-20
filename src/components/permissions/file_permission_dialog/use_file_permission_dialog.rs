//! Maps to: CC `components/permissions/FilePermissionDialog/useFilePermissionDialog.ts`.
//!
//! React state/keybinding registration remains in the iocraft component. These
//! pure helpers preserve the official focused-option/input-mode state
//! transitions used by Tab-to-amend and shift+tab session approval.

use super::permission_options::{PermissionOption, PermissionOptionWithLabel};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FilePermissionDialogState {
    pub accept_feedback: String,
    pub reject_feedback: String,
    pub focused_option: String,
    pub yes_input_mode: bool,
    pub no_input_mode: bool,
    pub yes_feedback_mode_entered: bool,
    pub no_feedback_mode_entered: bool,
}

impl Default for FilePermissionDialogState {
    fn default() -> Self {
        Self {
            accept_feedback: String::new(),
            reject_feedback: String::new(),
            focused_option: "yes".to_string(),
            yes_input_mode: false,
            no_input_mode: false,
            yes_feedback_mode_entered: false,
            no_feedback_mode_entered: false,
        }
    }
}

/// Maps to: CC `useFilePermissionDialog.ts#handleFocusedOptionChange`.
pub fn handle_focused_option_change(state: &mut FilePermissionDialogState, value: &str) {
    if value != "yes" && state.yes_input_mode && state.accept_feedback.trim().is_empty() {
        state.yes_input_mode = false;
    }
    if value != "no" && state.no_input_mode && state.reject_feedback.trim().is_empty() {
        state.no_input_mode = false;
    }
    state.focused_option = value.to_string();
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FeedbackModeEvent {
    AcceptEntered,
    AcceptCollapsed,
    RejectEntered,
    RejectCollapsed,
}

/// Maps to: CC `useFilePermissionDialog.ts#handleInputModeToggle`.
pub fn handle_input_mode_toggle(
    state: &mut FilePermissionDialogState,
    value: &str,
) -> Option<FeedbackModeEvent> {
    match value {
        "yes" => {
            if state.yes_input_mode {
                state.yes_input_mode = false;
                Some(FeedbackModeEvent::AcceptCollapsed)
            } else {
                state.yes_input_mode = true;
                state.yes_feedback_mode_entered = true;
                Some(FeedbackModeEvent::AcceptEntered)
            }
        }
        "no" => {
            if state.no_input_mode {
                state.no_input_mode = false;
                Some(FeedbackModeEvent::RejectCollapsed)
            } else {
                state.no_input_mode = true;
                state.no_feedback_mode_entered = true;
                Some(FeedbackModeEvent::RejectEntered)
            }
        }
        _ => None,
    }
}

/// Maps to: CC `useFilePermissionDialog.ts#handleCycleMode` session-option
/// lookup for `confirm:cycleMode`.
pub fn cycle_mode_session_option(
    options: &[PermissionOptionWithLabel],
) -> Option<PermissionOption> {
    options.iter().find_map(|option| match &option.option {
        PermissionOption::AcceptSession { .. } => Some(option.option.clone()),
        _ => None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::permissions::file_permission_dialog::permission_options::{
        FilePermissionOptionsParams, get_file_permission_options_from_params,
    };

    #[test]
    fn focused_option_change_collapses_empty_input_modes_like_official() {
        let mut state = FilePermissionDialogState {
            yes_input_mode: true,
            no_input_mode: true,
            ..FilePermissionDialogState::default()
        };
        handle_focused_option_change(&mut state, "yes-session");
        assert!(!state.yes_input_mode);
        assert!(!state.no_input_mode);
        assert_eq!(state.focused_option, "yes-session");

        state.yes_input_mode = true;
        state.accept_feedback = "keep this".to_string();
        handle_focused_option_change(&mut state, "no");
        assert!(state.yes_input_mode);
        assert_eq!(state.focused_option, "no");
    }

    #[test]
    fn input_mode_toggle_tracks_entered_flags_and_events() {
        let mut state = FilePermissionDialogState::default();
        assert_eq!(
            handle_input_mode_toggle(&mut state, "yes"),
            Some(FeedbackModeEvent::AcceptEntered)
        );
        assert!(state.yes_input_mode);
        assert!(state.yes_feedback_mode_entered);
        assert_eq!(
            handle_input_mode_toggle(&mut state, "yes"),
            Some(FeedbackModeEvent::AcceptCollapsed)
        );
        assert!(!state.yes_input_mode);
        assert_eq!(
            handle_input_mode_toggle(&mut state, "no"),
            Some(FeedbackModeEvent::RejectEntered)
        );
        assert!(state.no_feedback_mode_entered);
    }

    #[test]
    fn cycle_mode_selects_first_accept_session_option() {
        let options =
            get_file_permission_options_from_params(&FilePermissionOptionsParams::default());
        assert!(matches!(
            cycle_mode_session_option(&options),
            Some(PermissionOption::AcceptSession { .. })
        ));
    }
}
