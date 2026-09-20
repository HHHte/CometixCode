//! Maps to: CC `components/permissions/utils.ts`.
//!
//! The official module emits unary telemetry for accept/reject decisions. This
//! project intentionally does not implement telemetry; the helper below keeps
//! the official event/metadata shape as a pure value so permission handlers can
//! be tested without sending analytics.

use crate::types::permissions::ToolUseConfirm;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UnaryPermissionEventKind {
    Accept,
    Reject,
}

impl UnaryPermissionEventKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Accept => "accept",
            Self::Reject => "reject",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UnaryPermissionEventRecord {
    pub completion_type: String,
    pub event: UnaryPermissionEventKind,
    pub language_name: String,
    pub message_id: String,
    pub platform: String,
    pub has_feedback: bool,
    pub telemetry_sent: bool,
}

fn host_platform_for_analytics() -> &'static str {
    match crate::utils::env::get().platform {
        crate::utils::env::Platform::MacOS => "darwin",
        crate::utils::env::Platform::Linux => "linux",
        crate::utils::env::Platform::Windows => "win32",
    }
}

/// Maps to: CC `logUnaryPermissionEvent(...)`.
///
/// Safety behavior: telemetry is disabled, so this returns the exact metadata
/// that would be logged and marks `telemetry_sent=false`.
pub fn log_unary_permission_event(
    completion_type: impl Into<String>,
    tool_use_confirm: &ToolUseConfirm,
    event: UnaryPermissionEventKind,
    has_feedback: Option<bool>,
) -> UnaryPermissionEventRecord {
    UnaryPermissionEventRecord {
        completion_type: completion_type.into(),
        event,
        language_name: "none".to_string(),
        message_id: if tool_use_confirm.request.id.trim().is_empty() {
            tool_use_confirm.request.tool_use_id.clone()
        } else {
            tool_use_confirm.request.id.clone()
        },
        platform: host_platform_for_analytics().to_string(),
        has_feedback: has_feedback.unwrap_or(false),
        telemetry_sent: false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::permissions::{
        PermissionMode, PermissionRequest, PermissionRuleValue, ToolUseConfirm,
    };

    fn confirm() -> ToolUseConfirm {
        ToolUseConfirm::new(PermissionRequest {
            permission_result: None,
            id: "msg_123".to_string(),
            tool_use_id: "toolu_123".to_string(),
            tool_name: "Bash".to_string(),
            mcp_info: None,
            decision_reason: None,
            description: String::new(),
            message: String::new(),
            input_summary: "echo hi".to_string(),
            input: serde_json::json!({"command":"echo hi"}),
            call_input: None,
            rule: PermissionRuleValue::new("Bash", None),
            suggestions: Vec::new(),
            blocked_path: None,
            metadata: None,
            is_compound_command: false,
            mode: PermissionMode::Default,
        })
    }

    #[test]
    fn unary_permission_event_preserves_official_metadata_without_telemetry() {
        let event = log_unary_permission_event(
            "tool_use_single",
            &confirm(),
            UnaryPermissionEventKind::Reject,
            Some(true),
        );
        assert_eq!(event.completion_type, "tool_use_single");
        assert_eq!(event.event.as_str(), "reject");
        assert_eq!(event.language_name, "none");
        assert_eq!(event.message_id, "msg_123");
        assert!(event.has_feedback);
        assert!(!event.telemetry_sent);
    }
}
