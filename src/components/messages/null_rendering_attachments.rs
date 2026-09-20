//! Maps to: CC `components/messages/nullRenderingAttachments.ts`.
//!
//! Attachment types that `AttachmentMessage` renders as null unconditionally.
//! `Messages.tsx` filters these out BEFORE the 200-message render cap so
//! invisible entries don't consume the render budget (CC-724).
//!
//! Note this is a *narrower* question than "does the component draw
//! anything". CC's component switch has a `default:` arm returning null, but
//! `isNullRenderingAttachment` only consults the explicit
//! `NULL_RENDERING_TYPES` allowlist — so a type outside the list keeps its row
//! (counted, and occupying render budget) even when it draws nothing. Matching
//! on the typed variants instead of the wire tag lost that distinction twice:
//! `Unknown` and `agent_listing_delta` were dropped from the list entirely,
//! and payloads whose tag is known but whose shape failed to parse rode
//! `Unknown` and got filtered for the wrong reason.

use crate::types::message::{Attachment, RenderableMessage, RenderableMessageKind};

/// CC `NULL_RENDERING_TYPES` (nullRenderingAttachments.ts:14-48), verbatim.
///
/// Keyed on the wire tag rather than the Rust variant so entries CC lists
/// without a matching member of its own `Attachment` union — `pen_mode_enter`
/// and `pen_mode_exit`, which have no CC 2.1.88 union member and therefore no
/// Rust variant — still resolve, and so a known tag that failed to parse into
/// its typed shape is judged by what it says it is.
const NULL_RENDERING_TYPES: &[&str] = &[
    "hook_success",
    "hook_additional_context",
    "hook_cancelled",
    "command_permissions",
    "agent_mention",
    "budget_usd",
    "critical_system_reminder",
    "edited_image_file",
    "edited_text_file",
    "opened_file_in_ide",
    "output_style",
    "plan_mode",
    "plan_mode_exit",
    "plan_mode_reentry",
    "structured_output",
    "team_context",
    "todo_reminder",
    "context_efficiency",
    "deferred_tools_delta",
    "mcp_instructions_delta",
    "companion_intro",
    "token_usage",
    "ultrathink_effort",
    "max_turns_reached",
    "task_reminder",
    "auto_mode",
    "auto_mode_exit",
    "output_token_usage",
    "pen_mode_enter",
    "pen_mode_exit",
    "verify_plan_reminder",
    "current_session_memory",
    "compaction_reminder",
    "date_change",
];

/// CC `NULL_RENDERING_ATTACHMENT_TYPES.has(msg.attachment.type)` at the
/// attachment level.
pub fn is_null_rendering_attachment_type(attachment: &Attachment) -> bool {
    NULL_RENDERING_TYPES.contains(&attachment.wire_type())
}

pub fn is_null_rendering_attachment(message: &RenderableMessage) -> bool {
    match &message.kind {
        RenderableMessageKind::Attachment(attachment) => {
            is_null_rendering_attachment_type(attachment)
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The list is CC's verbatim, so every entry has to be reachable by tag.
    #[test]
    fn list_matches_official_size_and_membership() {
        assert_eq!(NULL_RENDERING_TYPES.len(), 34);
        let mut sorted = NULL_RENDERING_TYPES.to_vec();
        sorted.sort_unstable();
        sorted.dedup();
        assert_eq!(sorted.len(), NULL_RENDERING_TYPES.len(), "no duplicates");
    }

    /// CC's allowlist does not contain `agent_listing_delta`
    /// (nullRenderingAttachments.ts:14-48) — the component renders "N agent
    /// types available" for a non-initial delta (AttachmentMessage.tsx:291-300),
    /// so the row must survive the filter.
    #[test]
    fn agent_listing_delta_is_not_filtered_like_official() {
        let attachment = Attachment::AgentListingDelta {
            added_types: vec!["explorer".to_string()],
            added_lines: Vec::new(),
            removed_types: Vec::new(),
            is_initial: false,
            show_concurrency_note: false,
        };
        assert!(!is_null_rendering_attachment_type(&attachment));
    }

    /// CC's `has()` returns false for a tag outside the list, so an
    /// unrecognized attachment keeps its row and draws nothing through the
    /// component's `default:` arm. Filtering it here would desync the message
    /// count and the 200-row render budget.
    #[test]
    fn unknown_tag_keeps_its_row_like_official() {
        let attachment = Attachment::from_wire(serde_json::json!({
            "type": "attachment_from_a_newer_cc"
        }));
        assert!(matches!(attachment, Attachment::Unknown(_)));
        assert!(!is_null_rendering_attachment_type(&attachment));
    }

    /// …while a listed tag is filtered even when its payload failed to parse
    /// into the typed shape and had to ride `Unknown`.
    #[test]
    fn listed_tag_is_filtered_even_when_it_rides_unknown() {
        let attachment = Attachment::from_wire(serde_json::json!({
            "type": "deferred_tools_delta"
        }));
        assert!(
            matches!(attachment, Attachment::Unknown(_)),
            "payload is missing required fields, so it falls back"
        );
        assert!(is_null_rendering_attachment_type(&attachment));
    }

    /// CC lists these two but has no union member for them, so there is no
    /// Rust variant either; the tag still has to resolve.
    #[test]
    fn pen_mode_tags_resolve_without_a_typed_variant() {
        for tag in ["pen_mode_enter", "pen_mode_exit"] {
            let attachment = Attachment::from_wire(serde_json::json!({ "type": tag }));
            assert!(
                is_null_rendering_attachment_type(&attachment),
                "{tag} is in CC's list"
            );
        }
    }
}
