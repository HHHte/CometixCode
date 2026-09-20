//! UI-only helpers for official `SendMessageTool/UI.tsx`.

use crate::components::messages::user_tool_result_message::utils::{
    ToolRenderLine, ToolRenderTone,
};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SendMessageResultView {
    pub message: Option<String>,
    pub has_routing: bool,
    pub has_request_target: bool,
}

pub fn render_tool_use_message(input: Option<&serde_json::Value>) -> Option<String> {
    let input = input?;
    let message = input.get("message")?.as_object()?;
    if message.get("type").and_then(|value| value.as_str()) != Some("plan_approval_response") {
        return None;
    }
    let to = input
        .get("to")
        .and_then(|value| value.as_str())
        .unwrap_or_default();
    let verb = if semantic_bool(message.get("approve")).unwrap_or(false) {
        "approve"
    } else {
        "reject"
    };
    Some(format!("{verb} plan from: {to}"))
}

/// The renderer's view of the raw `toolUseResult`. SendMessageTool defines
/// no outputSchema, so CC's `tool.outputSchema?.safeParse` short-circuits
/// and `renderToolResultMessage` receives the raw value — a string is
/// `jsonParse`d first (`SendMessageTool/UI.tsx:24-25`).
pub fn parse_result(value: &serde_json::Value) -> Option<SendMessageResultView> {
    let parsed;
    let output = match value {
        serde_json::Value::String(text) => {
            parsed = serde_json::from_str::<serde_json::Value>(text).ok()?;
            &parsed
        }
        other => other,
    };
    let map = output.as_object()?;
    Some(SendMessageResultView {
        message: map
            .get("message")
            .and_then(|value| value.as_str())
            .map(ToOwned::to_owned),
        // CC: `'routing' in result && result.routing` — key present and
        // JS-truthy.
        has_routing: map.get("routing").is_some_and(is_truthy_json),
        has_request_target: map.contains_key("request_id") && map.contains_key("target"),
    })
}

pub fn renders_result(view: &SendMessageResultView) -> bool {
    !view.has_routing && !view.has_request_target
}

/// Maps to: CC `SendMessageTool/UI.tsx:19-40` `renderToolResultMessage` — a
/// routed or request/target result renders null; everything else renders
/// `result.message` dim (an absent message interpolates empty, still one
/// line).
pub(crate) fn render_tool_result_message(
    raw_output: Option<&serde_json::Value>,
) -> Vec<ToolRenderLine> {
    let Some(view) = raw_output.and_then(parse_result) else {
        return Vec::new();
    };
    if !renders_result(&view) {
        return Vec::new();
    }
    vec![ToolRenderLine::new(
        view.message.unwrap_or_default(),
        ToolRenderTone::Inactive,
    )]
}

fn semantic_bool(value: Option<&serde_json::Value>) -> Option<bool> {
    match value? {
        serde_json::Value::Bool(value) => Some(*value),
        serde_json::Value::String(value) => match value.trim().to_ascii_lowercase().as_str() {
            "true" | "yes" | "1" => Some(true),
            "false" | "no" | "0" => Some(false),
            _ => None,
        },
        _ => None,
    }
}

/// JS truthiness: null, false, 0, NaN, and the empty string are falsy.
fn is_truthy_json(value: &serde_json::Value) -> bool {
    match value {
        serde_json::Value::Null | serde_json::Value::Bool(false) => false,
        serde_json::Value::Bool(true) => true,
        serde_json::Value::Number(number) => {
            number.as_f64().is_some_and(|n| n != 0.0 && !n.is_nan())
        }
        serde_json::Value::String(text) => !text.is_empty(),
        serde_json::Value::Array(_) | serde_json::Value::Object(_) => true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn send_message_tool_use_plan_approval_matches_official_copy() {
        assert_eq!(
            render_tool_use_message(Some(&json!({
                "to": "alice",
                "message": {"type": "plan_approval_response", "approve": true}
            }))),
            Some("approve plan from: alice".to_string())
        );
        assert_eq!(
            render_tool_use_message(Some(&json!({
                "to": "alice",
                "message": {"type": "plan_approval_response", "approve": false}
            }))),
            Some("reject plan from: alice".to_string())
        );
        assert_eq!(
            render_tool_use_message(Some(&json!({"to": "alice", "message": "hello"}))),
            None
        );
    }

    #[test]
    fn send_message_result_visibility_matches_official_routing_and_request_rules() {
        let visible = json!({"success": true, "message": "Response sent"});
        let lines = render_tool_result_message(Some(&visible));
        assert_eq!(lines[0].text, "Response sent");
        assert_eq!(lines[0].tone, ToolRenderTone::Inactive);

        let routed = json!({
            "success": true,
            "message": "Message sent",
            "routing": {"target": "@alice"}
        });
        assert!(render_tool_result_message(Some(&routed)).is_empty());

        let request = json!({
            "success": true,
            "message": "Request queued",
            "request_id": "req_1",
            "target": "alice"
        });
        assert!(render_tool_result_message(Some(&request)).is_empty());

        // CC: `'routing' in result && result.routing` — a falsy routing
        // value (null) does not hide the row.
        let null_routing = json!({"success": true, "message": "Sent", "routing": null});
        assert_eq!(
            render_tool_result_message(Some(&null_routing))[0].text,
            "Sent"
        );

        // No outputSchema: a legacy string raw is jsonParse'd first, and an
        // absent message still renders one (empty) dim line.
        let string_raw = json!("{\"success\": true, \"message\": \"Parsed\"}");
        assert_eq!(
            render_tool_result_message(Some(&string_raw))[0].text,
            "Parsed"
        );
        let no_message = json!({"success": true});
        assert_eq!(render_tool_result_message(Some(&no_message))[0].text, "");
    }
}
