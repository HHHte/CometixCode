//! Maps to: CC `tools/BashTool/modeValidation.ts`.

use crate::tool::ToolPermissionContext;
use crate::types::permissions::PermissionMode;
use crate::utils::permissions::permission_result::{PermissionDecisionReason, PermissionResult};

const ACCEPT_EDITS_ALLOWED_COMMANDS: &[&str] =
    &["mkdir", "touch", "rm", "rmdir", "mv", "cp", "sed"];

/// Maps to CC `getAutoAllowedCommands(mode)`.
pub fn get_auto_allowed_commands(mode: PermissionMode) -> &'static [&'static str] {
    if mode == PermissionMode::AcceptEdits {
        ACCEPT_EDITS_ALLOWED_COMMANDS
    } else {
        &[]
    }
}

/// Maps to CC `checkPermissionMode(input, toolPermissionContext)`.
pub fn check_permission_mode(command: &str, context: &ToolPermissionContext) -> PermissionResult {
    if context.mode == PermissionMode::BypassPermissions {
        return PermissionResult::Passthrough {
            message: "Bypass mode is handled in main permission flow".to_string(),
            decision_reason: None,
            suggestions: Vec::new(),
            blocked_path: None,
            pending_classifier_check: None,
        };
    }
    if context.mode == PermissionMode::DontAsk {
        return PermissionResult::Passthrough {
            message: "DontAsk mode is handled in main permission flow".to_string(),
            decision_reason: None,
            suggestions: Vec::new(),
            blocked_path: None,
            pending_classifier_check: None,
        };
    }

    for subcommand in crate::utils::bash::commands::split_command_deprecated(command) {
        let result = validate_command_for_mode(subcommand.trim(), context);
        if !matches!(result, PermissionResult::Passthrough { .. }) {
            return result;
        }
    }

    PermissionResult::Passthrough {
        message: "No mode-specific validation required".to_string(),
        decision_reason: None,
        suggestions: Vec::new(),
        blocked_path: None,
        pending_classifier_check: None,
    }
}

fn validate_command_for_mode(command: &str, context: &ToolPermissionContext) -> PermissionResult {
    let base_command = command.split_whitespace().next();
    let Some(base_command) = base_command else {
        return PermissionResult::Passthrough {
            message: "Base command not found".to_string(),
            decision_reason: None,
            suggestions: Vec::new(),
            blocked_path: None,
            pending_classifier_check: None,
        };
    };

    if context.mode == PermissionMode::AcceptEdits
        && ACCEPT_EDITS_ALLOWED_COMMANDS.contains(&base_command)
    {
        return PermissionResult::Allow {
            updated_input: Some(serde_json::json!({"command": command})),
            user_modified: None,
            decision_reason: Some(PermissionDecisionReason::Mode {
                mode: PermissionMode::AcceptEdits,
            }),
            tool_use_id: None,
            accept_feedback: None,
            content_blocks: Vec::new(),
        };
    }

    PermissionResult::Passthrough {
        message: format!(
            "No mode-specific handling for '{base_command}' in {:?} mode",
            context.mode
        ),
        decision_reason: None,
        suggestions: Vec::new(),
        blocked_path: None,
        pending_classifier_check: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accept_edits_auto_allows_filesystem_commands() {
        let context = ToolPermissionContext {
            mode: PermissionMode::AcceptEdits,
            ..ToolPermissionContext::default()
        };
        let result = check_permission_mode("echo hi && mkdir target", &context);
        assert!(matches!(
            result,
            PermissionResult::Allow {
                decision_reason: Some(PermissionDecisionReason::Mode {
                    mode: PermissionMode::AcceptEdits
                }),
                ..
            }
        ));
    }

    #[test]
    fn bypass_dontask_and_default_passthrough_like_official() {
        let bypass = ToolPermissionContext {
            mode: PermissionMode::BypassPermissions,
            ..ToolPermissionContext::default()
        };
        assert!(matches!(
            check_permission_mode("mkdir target", &bypass),
            PermissionResult::Passthrough { ref message, .. }
                if message == "Bypass mode is handled in main permission flow"
        ));
        let dont_ask = ToolPermissionContext {
            mode: PermissionMode::DontAsk,
            ..ToolPermissionContext::default()
        };
        assert!(matches!(
            check_permission_mode("mkdir target", &dont_ask),
            PermissionResult::Passthrough { ref message, .. }
                if message == "DontAsk mode is handled in main permission flow"
        ));
        assert!(matches!(
            check_permission_mode("mkdir target", &ToolPermissionContext::default()),
            PermissionResult::Passthrough { .. }
        ));
        assert_eq!(
            get_auto_allowed_commands(PermissionMode::AcceptEdits),
            ACCEPT_EDITS_ALLOWED_COMMANDS
        );
        assert!(get_auto_allowed_commands(PermissionMode::Default).is_empty());
    }
}
