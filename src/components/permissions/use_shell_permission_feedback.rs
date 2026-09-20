//! Maps to: CC `components/permissions/useShellPermissionFeedback.ts`.
//!
//! React hook state is represented here as pure state-transition helpers. The
//! permission dialog components own the iocraft event wiring; this module keeps
//! the official yes/no feedback-mode, focus, and reject semantics reusable for
//! Bash and PowerShell without emitting analytics.

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ShellPermissionFeedbackState {
    pub yes_input_mode: bool,
    pub no_input_mode: bool,
    pub yes_feedback_mode_entered: bool,
    pub no_feedback_mode_entered: bool,
    pub accept_feedback: String,
    pub reject_feedback: String,
    pub focused_option: String,
}

impl Default for ShellPermissionFeedbackState {
    fn default() -> Self {
        Self {
            yes_input_mode: false,
            no_input_mode: false,
            yes_feedback_mode_entered: false,
            no_feedback_mode_entered: false,
            accept_feedback: String::new(),
            reject_feedback: String::new(),
            focused_option: "yes".to_string(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShellFeedbackModeEvent {
    AcceptFeedbackModeEntered,
    AcceptFeedbackModeCollapsed,
    RejectFeedbackModeEntered,
    RejectFeedbackModeCollapsed,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ShellFocusOutcome {
    pub user_interaction: bool,
    pub collapsed_yes_input: bool,
    pub collapsed_no_input: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ShellRejectOutcome {
    pub feedback: Option<String>,
    pub has_feedback: bool,
    pub escape_without_feedback: bool,
    pub explainer_visible: bool,
}

/// Maps to: CC `useShellPermissionFeedback.ts#handleInputModeToggle`.
pub fn handle_shell_input_mode_toggle(
    state: &mut ShellPermissionFeedbackState,
    option: &str,
) -> Option<ShellFeedbackModeEvent> {
    match option {
        "yes" => {
            if state.yes_input_mode {
                state.yes_input_mode = false;
                Some(ShellFeedbackModeEvent::AcceptFeedbackModeCollapsed)
            } else {
                state.yes_input_mode = true;
                state.yes_feedback_mode_entered = true;
                Some(ShellFeedbackModeEvent::AcceptFeedbackModeEntered)
            }
        }
        "no" => {
            if state.no_input_mode {
                state.no_input_mode = false;
                Some(ShellFeedbackModeEvent::RejectFeedbackModeCollapsed)
            } else {
                state.no_input_mode = true;
                state.no_feedback_mode_entered = true;
                Some(ShellFeedbackModeEvent::RejectFeedbackModeEntered)
            }
        }
        _ => None,
    }
}

/// Maps to: CC `useShellPermissionFeedback.ts#handleFocus`.
pub fn handle_shell_focus(
    state: &mut ShellPermissionFeedbackState,
    value: &str,
) -> ShellFocusOutcome {
    let mut outcome = ShellFocusOutcome {
        user_interaction: value != state.focused_option,
        ..ShellFocusOutcome::default()
    };

    if value != "yes" && state.yes_input_mode && state.accept_feedback.trim().is_empty() {
        state.yes_input_mode = false;
        outcome.collapsed_yes_input = true;
    }
    if value != "no" && state.no_input_mode && state.reject_feedback.trim().is_empty() {
        state.no_input_mode = false;
        outcome.collapsed_no_input = true;
    }
    state.focused_option = value.to_string();
    outcome
}

/// Maps to: CC `useShellPermissionFeedback.ts#handleReject`.
pub fn handle_shell_reject(feedback: Option<&str>, explainer_visible: bool) -> ShellRejectOutcome {
    let trimmed = feedback.map(str::trim).filter(|value| !value.is_empty());
    ShellRejectOutcome {
        feedback: trimmed.map(str::to_string),
        has_feedback: trimmed.is_some(),
        escape_without_feedback: trimmed.is_none(),
        explainer_visible,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shell_feedback_toggle_matches_official_yes_no_modes() {
        let mut state = ShellPermissionFeedbackState::default();
        assert_eq!(
            handle_shell_input_mode_toggle(&mut state, "yes"),
            Some(ShellFeedbackModeEvent::AcceptFeedbackModeEntered)
        );
        assert!(state.yes_input_mode);
        assert!(state.yes_feedback_mode_entered);
        assert_eq!(
            handle_shell_input_mode_toggle(&mut state, "yes"),
            Some(ShellFeedbackModeEvent::AcceptFeedbackModeCollapsed)
        );
        assert!(!state.yes_input_mode);
        assert_eq!(
            handle_shell_input_mode_toggle(&mut state, "no"),
            Some(ShellFeedbackModeEvent::RejectFeedbackModeEntered)
        );
        assert!(state.no_input_mode);
        assert!(state.no_feedback_mode_entered);
    }

    #[test]
    fn shell_feedback_focus_collapses_empty_modes_but_keeps_typed_feedback() {
        let mut state = ShellPermissionFeedbackState {
            yes_input_mode: true,
            no_input_mode: true,
            ..ShellPermissionFeedbackState::default()
        };
        let outcome = handle_shell_focus(&mut state, "yes-session");
        assert!(outcome.user_interaction);
        assert!(outcome.collapsed_yes_input);
        assert!(outcome.collapsed_no_input);
        assert!(!state.yes_input_mode);
        assert!(!state.no_input_mode);

        state.focused_option = "yes".to_string();
        state.yes_input_mode = true;
        state.accept_feedback = "keep instructions".to_string();
        let outcome = handle_shell_focus(&mut state, "no");
        assert!(outcome.user_interaction);
        assert!(!outcome.collapsed_yes_input);
        assert!(state.yes_input_mode);
    }

    #[test]
    fn shell_reject_trims_feedback_and_marks_escape_without_feedback() {
        let with_feedback = handle_shell_reject(Some("  try a safer command  "), true);
        assert_eq!(
            with_feedback.feedback.as_deref(),
            Some("try a safer command")
        );
        assert!(with_feedback.has_feedback);
        assert!(!with_feedback.escape_without_feedback);
        assert!(with_feedback.explainer_visible);

        let escape = handle_shell_reject(Some("   "), false);
        assert_eq!(escape.feedback, None);
        assert!(!escape.has_feedback);
        assert!(escape.escape_without_feedback);
    }
}
