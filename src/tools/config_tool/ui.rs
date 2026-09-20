//! UI-only port of official `tools/ConfigTool/UI.tsx`.

use super::Output;
use crate::components::messages::user_tool_result_message::utils::{
    ToolRenderLine, ToolRenderSegment, ToolRenderTone,
};

/// The Rust stand-in for CC's `outputSchema.safeParse(toolUseResult)`
/// (`UserToolSuccessMessage.tsx:80`): `success` is the only required field
/// (`ConfigTool.ts:51-61`); `operation` is a `get|set` enum; `value`,
/// `previousValue`, and `newValue` are `z.unknown()` and carry any JSON
/// value.
pub(crate) fn parse_output(value: &serde_json::Value) -> Option<Output> {
    let map = value.as_object()?;
    let operation = match map.get("operation") {
        None => None,
        Some(serde_json::Value::String(operation)) if operation == "get" || operation == "set" => {
            Some(operation.clone())
        }
        Some(_) => return None,
    };
    let optional_string = |key: &str| match map.get(key) {
        None => Some(None),
        Some(serde_json::Value::String(text)) => Some(Some(text.clone())),
        Some(_) => None,
    };
    Some(Output {
        success: map.get("success")?.as_bool()?,
        operation,
        setting: optional_string("setting")?,
        value: map.get("value").cloned(),
        previous_value: map.get("previousValue").cloned(),
        new_value: map.get("newValue").cloned(),
        error: optional_string("error")?,
    })
}

/// Serializes [`Output`] to CC's exact `toolUseResult` wire shape — schema
/// declaration order (every `call()` construction is a subsequence of it),
/// optional fields omitted.
pub(crate) fn output_to_value(output: &Output) -> serde_json::Value {
    let mut map = serde_json::Map::new();
    map.insert(
        "success".to_string(),
        serde_json::Value::Bool(output.success),
    );
    if let Some(operation) = output.operation.as_ref() {
        map.insert(
            "operation".to_string(),
            serde_json::Value::String(operation.clone()),
        );
    }
    if let Some(setting) = output.setting.as_ref() {
        map.insert(
            "setting".to_string(),
            serde_json::Value::String(setting.clone()),
        );
    }
    if let Some(value) = output.value.as_ref() {
        map.insert("value".to_string(), value.clone());
    }
    if let Some(previous_value) = output.previous_value.as_ref() {
        map.insert("previousValue".to_string(), previous_value.clone());
    }
    if let Some(new_value) = output.new_value.as_ref() {
        map.insert("newValue".to_string(), new_value.clone());
    }
    if let Some(error) = output.error.as_ref() {
        map.insert(
            "error".to_string(),
            serde_json::Value::String(error.clone()),
        );
    }
    serde_json::Value::Object(map)
}

/// `jsonStringify(value)` is bare `JSON.stringify` (`slowOperations.ts:189`):
/// an absent value stringifies to `undefined`, which the JSX interpolates as
/// empty text.
///
/// This is the JSX half of the pair. The model-facing half —
/// `super::interpolate_json_stringify`, used by
/// `mapToolResultToToolResultBlockParam` — renders the same `undefined` as the
/// literal text `undefined`, because a template literal stringifies it while
/// React drops it. Keep them separate.
fn json_stringify_interpolation(value: Option<&serde_json::Value>) -> String {
    match value {
        Some(value) => serde_json::to_string(value).unwrap_or_default(),
        None => String::new(),
    }
}

/// Maps to: CC `ConfigTool/UI.tsx:19-44` `renderToolResultMessage` — three
/// branches: `Failed: {error}` (error color), `{setting} = {json(value)}`
/// for `get` (setting bold), and `Set {setting} to {json(newValue)}`
/// otherwise (setting and newValue bold). Absent optionals interpolate as
/// empty, exactly as the JSX renders `undefined`.
pub(crate) fn render_tool_result_message(
    raw_output: Option<&serde_json::Value>,
) -> Vec<ToolRenderLine> {
    let Some(output) = raw_output.and_then(parse_output) else {
        return Vec::new();
    };
    if !output.success {
        return vec![ToolRenderLine::new(
            format!("Failed: {}", output.error.as_deref().unwrap_or_default()),
            ToolRenderTone::Error,
        )];
    }
    let setting = output.setting.as_deref().unwrap_or_default();
    if output.operation.as_deref() == Some("get") {
        let value = json_stringify_interpolation(output.value.as_ref());
        return vec![
            ToolRenderLine::new(format!("{setting} = {value}"), ToolRenderTone::Normal)
                .with_segments(vec![
                    ToolRenderSegment::new(setting).with_bold(true),
                    ToolRenderSegment::new(format!(" = {value}")),
                ]),
        ];
    }
    let new_value = json_stringify_interpolation(output.new_value.as_ref());
    vec![
        ToolRenderLine::new(
            format!("Set {setting} to {new_value}"),
            ToolRenderTone::Normal,
        )
        .with_segments(vec![
            ToolRenderSegment::new("Set "),
            ToolRenderSegment::new(setting).with_bold(true),
            ToolRenderSegment::new(" to "),
            ToolRenderSegment::new(new_value).with_bold(true),
        ]),
    ]
}

/// Maps to: CC `ConfigTool/UI.tsx:46-48` `renderToolUseRejectedMessage` —
/// the constant warning-colored string.
pub(crate) fn render_rejected_message() -> &'static str {
    "Config change rejected"
}

/// Maps to: CC `ConfigTool/UI.tsx:7-17` `renderToolUseMessage`.
pub(crate) fn config_tool_use_summary(input: &serde_json::Value) -> Option<String> {
    let setting = crate::components::messages::user_tool_result_message::utils::first_string(
        input,
        &["setting"],
    )?;
    input
        .get("value")
        .map(|value| format!("Setting {setting} to {}", crate::components::messages::user_tool_result_message::utils::json_stringify_for_display(value)))
        .or_else(|| Some(format!("Getting {setting}")))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn config_result_renders_official_three_branches() {
        let get = json!({"success": true, "operation": "get", "setting": "theme", "value": "dark"});
        let lines = render_tool_result_message(Some(&get));
        assert_eq!(lines[0].text, "theme = \"dark\"");
        assert!(lines[0].segments[0].bold);

        let set = json!({"success": true, "operation": "set", "setting": "theme", "previousValue": "dark", "newValue": "light"});
        let lines = render_tool_result_message(Some(&set));
        assert_eq!(lines[0].text, "Set theme to \"light\"");
        assert!(lines[0].segments[1].bold);
        assert!(lines[0].segments[3].bold);

        let failed = json!({"success": false, "operation": "set", "setting": "theme", "error": "unknown setting"});
        let lines = render_tool_result_message(Some(&failed));
        assert_eq!(lines[0].text, "Failed: unknown setting");
        assert_eq!(lines[0].tone, ToolRenderTone::Error);
    }

    #[test]
    fn config_parse_requires_success_and_round_trips_wire_shape() {
        assert!(parse_output(&json!({"operation": "get"})).is_none());
        assert!(parse_output(&json!({"success": true, "operation": "toggle"})).is_none());
        let raw = json!({"success": true, "operation": "set", "setting": "theme", "previousValue": "dark", "newValue": "light"});
        let output = parse_output(&raw).unwrap();
        assert_eq!(output_to_value(&output), raw);
    }

    /// The producer reaches this shape: `config_output`'s get branch emits
    /// `value: None` for any unset setting, and `output_to_value` then drops
    /// the key exactly like `JSON.stringify` drops an undefined property.
    /// Pinned end-to-end by
    /// `config_get_reads_an_unset_setting_back_as_undefined_not_null`.
    #[test]
    fn config_absent_optionals_interpolate_empty_like_official_jsx() {
        // `JSON.stringify(undefined)` is undefined; the JSX renders it empty.
        let bare = json!({"success": true, "operation": "get"});
        let lines = render_tool_result_message(Some(&bare));
        assert_eq!(lines[0].text, " = ");

        let failed = json!({"success": false});
        let lines = render_tool_result_message(Some(&failed));
        assert_eq!(lines[0].text, "Failed: ");
    }
}
