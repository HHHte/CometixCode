//! Maps to: CC `hooks/toolPermission/handlers/coordinatorHandler.ts`.
//!
//! Official coordinator handling awaits permission hooks first, then classifier
//! auto-approval, and falls through to the interactive dialog when neither
//! resolves or an automated check fails. The Rust permission runtime does not
//! yet expose the full async `PermissionContext`, so this module preserves the
//! official ordering and fallthrough/error semantics over already-produced
//! automated check outcomes.

use crate::types::permissions::{PermissionUpdate, PromptDecision};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CoordinatorPermissionParams {
    pub pending_classifier_check: bool,
    pub updated_input: Option<serde_json::Value>,
    pub suggestions: Vec<PermissionUpdate>,
    pub permission_mode: Option<String>,
    pub hook_result: AutomatedPermissionCheck,
    pub classifier_result: AutomatedPermissionCheck,
    pub bash_classifier_enabled: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum AutomatedPermissionCheck {
    #[default]
    Unresolved,
    Resolved(PromptDecision),
    Failed(String),
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CoordinatorPermissionOutcome {
    pub decision: Option<PromptDecision>,
    pub fell_through_to_interactive: bool,
    pub logged_errors: Vec<String>,
}

/// Maps to: CC `handleCoordinatorPermission(...)`.
pub fn handle_coordinator_permission(
    params: CoordinatorPermissionParams,
) -> CoordinatorPermissionOutcome {
    let mut logged_errors = Vec::new();
    match params.hook_result {
        AutomatedPermissionCheck::Resolved(decision) => {
            return CoordinatorPermissionOutcome {
                decision: Some(decision),
                fell_through_to_interactive: false,
                logged_errors,
            };
        }
        AutomatedPermissionCheck::Failed(error) => logged_errors.push(error),
        AutomatedPermissionCheck::Unresolved => {}
    }

    if params.bash_classifier_enabled {
        match params.classifier_result {
            AutomatedPermissionCheck::Resolved(decision) => {
                return CoordinatorPermissionOutcome {
                    decision: Some(decision),
                    fell_through_to_interactive: false,
                    logged_errors,
                };
            }
            AutomatedPermissionCheck::Failed(error) => logged_errors.push(error),
            AutomatedPermissionCheck::Unresolved => {}
        }
    }

    let _ = (
        params.pending_classifier_check,
        params.updated_input,
        params.suggestions,
        params.permission_mode,
    );
    CoordinatorPermissionOutcome {
        decision: None,
        fell_through_to_interactive: true,
        logged_errors,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::permissions::{PermissionBehavior, PermissionPromptChoice};

    fn decision(transcript: &str) -> PromptDecision {
        PromptDecision {
            behavior: PermissionBehavior::Allow,
            choice: PermissionPromptChoice::AllowOnce,
            updates: Vec::new(),
            transcript: transcript.to_string(),
            updated_input: Some(serde_json::json!({"command":"echo ok"})),
        }
    }

    fn params() -> CoordinatorPermissionParams {
        CoordinatorPermissionParams {
            pending_classifier_check: true,
            updated_input: None,
            suggestions: Vec::new(),
            permission_mode: Some("default".to_string()),
            hook_result: AutomatedPermissionCheck::Unresolved,
            classifier_result: AutomatedPermissionCheck::Unresolved,
            bash_classifier_enabled: true,
        }
    }

    #[test]
    fn coordinator_permission_prefers_hooks_before_classifier() {
        let outcome = handle_coordinator_permission(CoordinatorPermissionParams {
            hook_result: AutomatedPermissionCheck::Resolved(decision("hook")),
            classifier_result: AutomatedPermissionCheck::Resolved(decision("classifier")),
            ..params()
        });
        assert_eq!(outcome.decision.unwrap().transcript, "hook");
        assert!(!outcome.fell_through_to_interactive);
    }

    #[test]
    fn coordinator_permission_uses_classifier_after_unresolved_hooks_when_enabled() {
        let outcome = handle_coordinator_permission(CoordinatorPermissionParams {
            classifier_result: AutomatedPermissionCheck::Resolved(decision("classifier")),
            ..params()
        });
        assert_eq!(outcome.decision.unwrap().transcript, "classifier");
    }

    #[test]
    fn coordinator_permission_falls_through_and_logs_failed_checks() {
        let outcome = handle_coordinator_permission(CoordinatorPermissionParams {
            hook_result: AutomatedPermissionCheck::Failed("hook boom".to_string()),
            classifier_result: AutomatedPermissionCheck::Failed("classifier boom".to_string()),
            ..params()
        });
        assert!(outcome.decision.is_none());
        assert!(outcome.fell_through_to_interactive);
        assert_eq!(outcome.logged_errors, vec!["hook boom", "classifier boom"]);

        let disabled = handle_coordinator_permission(CoordinatorPermissionParams {
            classifier_result: AutomatedPermissionCheck::Resolved(decision("classifier")),
            bash_classifier_enabled: false,
            ..params()
        });
        assert!(disabled.decision.is_none());
        assert!(disabled.fell_through_to_interactive);
    }
}
