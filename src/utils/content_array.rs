//! Utility for inserting a block into a content array relative to tool_result
//! blocks. Used by the API layer to position supplementary content (e.g.,
//! cache editing directives) correctly within user messages.
//!
//! Maps to: CC `utils/contentArray.ts`.

/// Maps to: CC `utils/contentArray.ts` `insertBlockAfterToolResults(...)`.
///
/// Placement rules:
/// - If tool_result blocks exist: insert after the last one
/// - Otherwise: insert before the last block
/// - If the inserted block would be the final element, a text continuation
///   block is appended (some APIs require the prompt not to end with
///   non-text content)
pub fn insert_block_after_tool_results(
    content: &mut Vec<serde_json::Value>,
    block: serde_json::Value,
) {
    let last_tool_result_index = content.iter().rposition(|item| {
        item.as_object()
            .and_then(|obj| obj.get("type"))
            .and_then(|value| value.as_str())
            == Some("tool_result")
    });

    if let Some(index) = last_tool_result_index {
        let insert_pos = index + 1;
        content.insert(insert_pos, block);
        if insert_pos == content.len() - 1 {
            content.push(serde_json::json!({ "type": "text", "text": "." }));
        }
    } else {
        let insert_index = content.len().saturating_sub(1);
        content.insert(insert_index, block);
    }
}
