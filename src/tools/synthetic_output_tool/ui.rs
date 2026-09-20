//! UI-only helpers for official `SyntheticOutputTool`.

use crate::components::messages::user_tool_result_message::utils::{
    ToolRenderLine, ToolRenderTone,
};

pub fn render_tool_use_message(input: Option<&serde_json::Value>) -> Option<String> {
    let object = input?.as_object()?;
    if object.is_empty() {
        return None;
    }

    let keys = object.keys().cloned().collect::<Vec<_>>();
    if keys.len() <= 3 {
        Some(
            keys.iter()
                .map(|key| {
                    let value = object.get(key).unwrap_or(&serde_json::Value::Null);
                    format!("{key}: {}", json_stringify(value))
                })
                .collect::<Vec<_>>()
                .join(", "),
        )
    } else {
        Some(format!(
            "{} fields: {}…",
            keys.len(),
            keys.iter().take(3).cloned().collect::<Vec<_>>().join(", ")
        ))
    }
}

pub fn render_rejected_message() -> &'static str {
    "Structured output rejected"
}

pub fn render_error_message() -> &'static str {
    "Structured output error"
}

/// The Rust stand-in for CC's `outputSchema.safeParse(toolUseResult)`
/// (`UserToolSuccessMessage.tsx:80`): the schema is a bare `z.string()`
/// (`SyntheticOutputTool.ts:14-18`), so the raw must be a JSON string.
pub(crate) fn parse_output(value: &serde_json::Value) -> Option<String> {
    match value {
        serde_json::Value::String(text) => Some(text.clone()),
        _ => None,
    }
}

/// Maps to: CC `SyntheticOutputTool.ts:91-93` `renderToolResultMessage` —
/// returns the output string itself (no MessageResponse chrome; the tool only
/// exists in non-interactive sessions).
pub(crate) fn render_tool_result_message(
    raw_output: Option<&serde_json::Value>,
) -> Vec<ToolRenderLine> {
    let Some(output) = raw_output.and_then(parse_output) else {
        return Vec::new();
    };
    vec![ToolRenderLine::new(output, ToolRenderTone::Normal)]
}

fn json_stringify(value: &serde_json::Value) -> String {
    serde_json::to_string(value).unwrap_or_else(|_| "null".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn structured_output_tool_use_message_matches_official_field_summary() {
        assert_eq!(render_tool_use_message(Some(&json!({}))), None);
        assert_eq!(
            render_tool_use_message(Some(&json!({"ok": true, "count": 2}))).as_deref(),
            Some("ok: true, count: 2")
        );
        assert_eq!(
            render_tool_use_message(Some(&json!({"a": 1, "b": 2, "c": 3, "d": 4}))).as_deref(),
            Some("4 fields: a, b, c…")
        );
    }

    #[test]
    fn structured_output_result_lines_are_plain_output_like_official() {
        // CC's `toolUseResult` for this tool is the bare Output string; a
        // non-string raw (e.g. an old invented object wrapper) fails safeParse
        // and renders nothing.
        let raw = json!("Structured output provided successfully");
        let lines = render_tool_result_message(Some(&raw));
        assert_eq!(lines[0].text, "Structured output provided successfully");
        assert_eq!(lines[0].tone, ToolRenderTone::Normal);
        assert!(
            render_tool_result_message(Some(&json!({
                "data": "Structured output provided successfully",
                "structured_output": {"ok": true}
            })))
            .is_empty()
        );
        assert!(render_tool_result_message(None).is_empty());
    }
}
