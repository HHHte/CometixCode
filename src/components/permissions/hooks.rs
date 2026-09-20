//! Maps to: CC `components/permissions/hooks.ts`.
//!
//! Official React hooks log permission prompt analytics and unary events. This
//! port keeps the event names, metadata, and decision-log formatting as pure
//! values. Telemetry side effects remain intentionally disabled.

use crate::types::permissions::{PermissionBehavior, PromptDecision, ToolUseConfirm};
use crate::utils::permissions::permission_rule_parser::permission_rule_value_to_string;
use crate::utils::permissions::permission_update::extract_rules;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UnaryEvent {
    pub completion_type: String,
    pub language_name: String,
}

impl UnaryEvent {
    pub fn tool_use_single() -> Self {
        Self {
            completion_type: "tool_use_single".to_string(),
            language_name: "none".to_string(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PermissionRequestLoggingRecord {
    pub event_name: String,
    pub message_id: String,
    pub tool_name: String,
    pub is_mcp: bool,
    pub decision_reason_type: Option<String>,
    pub sandbox_enabled: bool,
    pub unary_event: UnaryEvent,
    pub telemetry_sent: bool,
}

/// Maps to: CC `permissionResultToLog(...)` for the currently ported Rust
/// permission decision shape. The full CC `decisionReason` union is not yet in
/// `PromptDecision`, so this formats behavior, transcript, and suggested
/// rule updates without inventing runtime reasons.
pub fn permission_result_to_log(permission_result: &PromptDecision) -> String {
    match permission_result.behavior {
        PermissionBehavior::Allow => "allow".to_string(),
        PermissionBehavior::Ask => {
            let suggestions = permission_suggestions_log(permission_result);
            format!(
                "ask: {}, \nsuggestions: {suggestions}",
                permission_result.transcript
            )
        }
        PermissionBehavior::Deny => format!("deny: {}", permission_result.transcript),
    }
}

fn permission_suggestions_log(permission_result: &PromptDecision) -> String {
    let rules = extract_rules(&permission_result.updates);
    if rules.is_empty() {
        "none".to_string()
    } else {
        rules
            .iter()
            .map(permission_rule_value_to_string)
            .collect::<Vec<_>>()
            .join(", ")
    }
}

/// Maps to: CC `usePermissionRequestLogging(...)`.
///
/// Safety behavior: telemetry/analytics are disabled; this returns the record
/// that would have been logged and marks `telemetry_sent=false`.
pub fn permission_request_logging_record(
    tool_use_confirm: &ToolUseConfirm,
    unary_event: UnaryEvent,
    sandbox_enabled: bool,
) -> PermissionRequestLoggingRecord {
    PermissionRequestLoggingRecord {
        event_name: "tengu_tool_use_show_permission_request".to_string(),
        message_id: if tool_use_confirm.request.id.trim().is_empty() {
            tool_use_confirm.request.tool_use_id.clone()
        } else {
            tool_use_confirm.request.id.clone()
        },
        tool_name: tool_use_confirm.request.tool_name.clone(),
        is_mcp: tool_use_confirm.request.tool_name.starts_with("mcp__"),
        decision_reason_type: None,
        sandbox_enabled,
        unary_event,
        telemetry_sent: false,
    }
}

/// Maps to: CC ant-only `tengu_internal_tool_use_permission_request_no_always_allow` guard.
pub fn should_log_no_always_allow_event(
    audience: crate::utils::build_profile::BuildAudience,
    tool_name: &str,
    permission_result: &PromptDecision,
) -> bool {
    crate::utils::build_profile::audience_has_internal_capability(
        audience,
        crate::utils::build_profile::InternalCapability::TelemetryPayloads,
    ) && tool_name == crate::tools::bash_tool::tool_name::BASH_TOOL_NAME
        && permission_result.behavior == PermissionBehavior::Ask
        && extract_rules(&permission_result.updates).is_empty()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::permissions::{
        PermissionMode, PermissionPromptChoice, PermissionRequest, PermissionRuleValue,
        PermissionUpdate, PermissionUpdateDestination,
    };

    fn confirm(tool_name: &str) -> ToolUseConfirm {
        ToolUseConfirm::new(PermissionRequest {
            permission_result: None,
            id: "msg_456".to_string(),
            tool_use_id: "toolu_456".to_string(),
            tool_name: tool_name.to_string(),
            mcp_info: None,
            decision_reason: None,
            description: String::new(),
            message: String::new(),
            input_summary: String::new(),
            input: serde_json::Value::Null,
            call_input: None,
            rule: PermissionRuleValue::new(tool_name, None),
            suggestions: Vec::new(),
            blocked_path: None,
            metadata: None,
            is_compound_command: false,
            mode: PermissionMode::Default,
        })
    }

    #[test]
    fn permission_logging_record_preserves_official_event_name_without_sending() {
        let record = permission_request_logging_record(
            &confirm("mcp__server__tool"),
            UnaryEvent::tool_use_single(),
            true,
        );
        assert_eq!(record.event_name, "tengu_tool_use_show_permission_request");
        assert_eq!(record.message_id, "msg_456");
        assert_eq!(record.tool_name, "mcp__server__tool");
        assert!(record.is_mcp);
        assert!(record.sandbox_enabled);
        assert_eq!(record.unary_event.completion_type, "tool_use_single");
        assert!(!record.telemetry_sent);
    }

    #[test]
    fn permission_result_log_formats_suggestions_like_official() {
        let decision = PromptDecision {
            behavior: PermissionBehavior::Ask,
            choice: PermissionPromptChoice::AlwaysAllow,
            updates: vec![PermissionUpdate::AddRules {
                destination: PermissionUpdateDestination::LocalSettings,
                behavior: PermissionBehavior::Allow,
                rules: vec![PermissionRuleValue::new(
                    "Bash",
                    Some("cargo test:*".to_string()),
                )],
            }],
            transcript: "needs confirmation".to_string(),
            updated_input: None,
        };
        let log = permission_result_to_log(&decision);
        assert!(log.contains("ask: needs confirmation"));
        assert!(log.contains("Bash(cargo test:*)"));
        assert!(!should_log_no_always_allow_event(
            crate::utils::build_profile::BuildAudience::AnthropicInternal,
            "Bash",
            &decision
        ));

        let no_rules = PromptDecision {
            updates: Vec::new(),
            ..decision
        };
        assert!(should_log_no_always_allow_event(
            crate::utils::build_profile::BuildAudience::AnthropicInternal,
            "Bash",
            &no_rules
        ));
        assert!(!should_log_no_always_allow_event(
            crate::utils::build_profile::BuildAudience::External,
            "Bash",
            &no_rules
        ));
    }
}
