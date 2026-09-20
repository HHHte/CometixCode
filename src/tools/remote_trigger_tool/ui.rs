//! UI-only port of official `tools/RemoteTriggerTool/UI.tsx`.

use super::Output;
use crate::components::messages::user_tool_result_message::utils::{
    ToolRenderLine, ToolRenderSegment, ToolRenderTone,
};

/// The Rust stand-in for CC's `outputSchema.safeParse(toolUseResult)`
/// (`UserToolSuccessMessage.tsx:80`): `status` and `json` are both required
/// (`RemoteTriggerTool.ts:35-40`); `status` is an unconstrained `z.number()`.
pub(crate) fn parse_output(value: &serde_json::Value) -> Option<Output> {
    let map = value.as_object()?;
    Some(Output {
        status: match map.get("status")? {
            serde_json::Value::Number(status) => status.clone(),
            _ => return None,
        },
        json: map.get("json")?.as_str()?.to_string(),
    })
}

/// Serializes [`Output`] to CC's exact `toolUseResult` wire shape.
pub(crate) fn output_to_value(output: &Output) -> serde_json::Value {
    let mut map = serde_json::Map::new();
    map.insert(
        "status".to_string(),
        serde_json::Value::Number(output.status.clone()),
    );
    map.insert(
        "json".to_string(),
        serde_json::Value::String(output.json.clone()),
    );
    serde_json::Value::Object(map)
}

/// Maps to: CC `RemoteTriggerTool/UI.tsx:11-20` `renderToolResultMessage` —
/// `HTTP {status}` plus a dim `({lines} lines)` where the count is
/// `countCharInString(json, '\n') + 1` (an empty body still counts as one
/// line).
pub(crate) fn render_tool_result_message(
    raw_output: Option<&serde_json::Value>,
) -> Vec<ToolRenderLine> {
    let Some(output) = raw_output.and_then(parse_output) else {
        return Vec::new();
    };
    let lines = output.json.chars().filter(|ch| *ch == '\n').count() + 1;
    let text = format!("HTTP {} ({lines} lines)", output.status);
    vec![
        ToolRenderLine::new(text, ToolRenderTone::Normal).with_segments(vec![
            ToolRenderSegment::new(format!("HTTP {} ", output.status)),
            ToolRenderSegment::new(format!("({lines} lines)")).with_dim(true),
        ]),
    ]
}

pub(crate) fn remote_trigger_tool_use_summary(input: &serde_json::Value) -> Option<String> {
    let action = crate::components::messages::user_tool_result_message::utils::first_string(
        input,
        &["action"],
    )
    .unwrap_or_default();
    let trigger_id = crate::components::messages::user_tool_result_message::utils::first_string(
        input,
        &["trigger_id"],
    );
    Some(match trigger_id {
        Some(trigger_id) if !trigger_id.is_empty() => format!("{action} {trigger_id}"),
        _ => action,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn remote_trigger_result_renders_status_and_dim_line_count() {
        let raw = json!({"status": 200, "json": "{}\n{}"});
        let lines = render_tool_result_message(Some(&raw));
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0].segments[0].text, "HTTP 200 ");
        assert_eq!(lines[0].segments[1].text, "(2 lines)");
        assert!(lines[0].segments[1].dim);

        // CC: `countCharInString('', '\n') + 1` — an empty body is one line.
        let empty = json!({"status": 501, "json": ""});
        let lines = render_tool_result_message(Some(&empty));
        assert_eq!(lines[0].segments[1].text, "(1 lines)");
    }

    #[test]
    fn remote_trigger_parse_requires_both_fields() {
        assert!(parse_output(&json!({"status": 200})).is_none());
        assert!(parse_output(&json!({"json": "{}"})).is_none());
        assert!(parse_output(&json!("HTTP 200")).is_none());
        let output = parse_output(&json!({"status": 200, "json": "{}"})).unwrap();
        assert_eq!(
            output_to_value(&output),
            json!({"status": 200, "json": "{}"})
        );
    }
}
