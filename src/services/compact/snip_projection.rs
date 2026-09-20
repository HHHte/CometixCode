//! Snip projection seam.
//! Maps to CC `services/compact/snipProjection.ts` `projectSnippedView(...)`.
//!
//! The checked-in upstream file is currently a generated empty stub, but
//! `utils/messages.ts#getMessagesAfterCompactBoundary(...)` still imports and
//! calls `projectSnippedView(...)` when HISTORY_SNIP is enabled. Keep this
//! no-op projection at the same responsibility boundary so `messagesForQuery`
//! construction has the official control-flow slot without inventing snip
//! semantics from missing source.

/// Maps to: CC `services/compact/snipProjection.ts` `projectSnippedView(...)`.
///
/// TODO: Port the real projection once upstream source is available. The safe
/// behavior for the current generated-stub source is identity projection.
pub fn project_snipped_view(
    messages: Vec<crate::types::message::Message>,
) -> Vec<crate::types::message::Message> {
    messages
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn project_snipped_view_is_safe_noop_for_current_empty_upstream_stub() {
        let messages = vec![crate::types::message::Message::User(
            crate::types::message::UserMessage {
                uuid: uuid::Uuid::new_v4().to_string(),
                timestamp: chrono::Utc::now(),
                content: vec![crate::types::message::UserContent::Text("kept".to_string())],
                is_compact_summary: false,
                plan_content: None,
                image_paste_ids: None,
                is_visible_in_transcript_only: false,
                mcp_meta: None,
                source_tool_assistant_uuid: None,
                permission_mode: None,
                origin: None,
                summarize_metadata: None,
            },
        )];

        assert_eq!(project_snipped_view(messages.clone()), messages);
    }
}
