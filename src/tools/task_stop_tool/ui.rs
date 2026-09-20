//! UI-only port of official `TaskStopTool/UI.tsx`.

use super::Output;
use crate::components::messages::user_tool_result_message::utils::{
    ToolRenderLine, ToolRenderTone,
};
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

const MAX_COMMAND_DISPLAY_LINES: usize = 2;
const MAX_COMMAND_DISPLAY_WIDTH: usize = 160;

/// The Rust stand-in for CC's `outputSchema.safeParse(toolUseResult)`
/// (`UserToolSuccessMessage.tsx:80`): `message`, `task_id`, and `task_type`
/// are required strings, `command` optional (`TaskStopTool.ts:22-34`).
pub(crate) fn parse_output(value: &serde_json::Value) -> Option<Output> {
    let map = value.as_object()?;
    let command = match map.get("command") {
        None => None,
        Some(serde_json::Value::String(command)) => Some(command.clone()),
        Some(_) => return None,
    };
    Some(Output {
        message: map.get("message")?.as_str()?.to_string(),
        task_id: map.get("task_id")?.as_str()?.to_string(),
        task_type: map.get("task_type")?.as_str()?.to_string(),
        command,
    })
}

/// Serializes [`Output`] to CC's exact `toolUseResult` wire shape — the
/// `call()` construction order (`TaskStopTool.ts:122-128`), `command` omitted
/// when absent.
pub(crate) fn output_to_value(output: &Output) -> serde_json::Value {
    let mut map = serde_json::Map::new();
    map.insert(
        "message".to_string(),
        serde_json::Value::String(output.message.clone()),
    );
    map.insert(
        "task_id".to_string(),
        serde_json::Value::String(output.task_id.clone()),
    );
    map.insert(
        "task_type".to_string(),
        serde_json::Value::String(output.task_type.clone()),
    );
    if let Some(command) = output.command.as_ref() {
        map.insert(
            "command".to_string(),
            serde_json::Value::String(command.clone()),
        );
    }
    serde_json::Value::Object(map)
}

/// Maps to: CC `TaskStopTool/UI.tsx:30-51` `renderToolResultMessage` —
/// `{command}{suffix}` where the suffix carries the ellipsis whenever
/// truncation changed the command. Verbose shows the raw command untrimmed;
/// the non-verbose `truncateCommand` trims, so even an untruncated command
/// with surrounding whitespace compares unequal and gains the ellipsis,
/// exactly as CC's `command !== rawCommand` does.
pub(crate) fn render_tool_result_message(
    raw_output: Option<&serde_json::Value>,
    verbose: bool,
) -> Vec<ToolRenderLine> {
    let Some(output) = raw_output.and_then(parse_output) else {
        return Vec::new();
    };
    let raw_command = output.command.unwrap_or_default();
    let command = if verbose {
        raw_command.clone()
    } else {
        truncate_command(&raw_command)
    };
    let suffix = if command != raw_command {
        "… · stopped"
    } else {
        " · stopped"
    };
    vec![ToolRenderLine::new(
        format!("{command}{suffix}"),
        ToolRenderTone::Normal,
    )]
}

fn truncate_command(command: &str) -> String {
    let mut truncated = command
        .lines()
        .take(MAX_COMMAND_DISPLAY_LINES)
        .collect::<Vec<_>>()
        .join("\n");

    if UnicodeWidthStr::width(truncated.as_str()) > MAX_COMMAND_DISPLAY_WIDTH {
        truncated = truncate_to_width_no_ellipsis(&truncated, MAX_COMMAND_DISPLAY_WIDTH);
    }

    truncated.trim().to_string()
}

fn truncate_to_width_no_ellipsis(value: &str, max_width: usize) -> String {
    let mut width = 0usize;
    let mut output = String::new();
    for ch in value.chars() {
        let ch_width = UnicodeWidthChar::width(ch).unwrap_or(0);
        if width + ch_width > max_width {
            break;
        }
        width += ch_width;
        output.push(ch);
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    fn raw(command: Option<&str>) -> serde_json::Value {
        let mut value = serde_json::json!({
            "message": "Successfully stopped task: task-1 (cmd)",
            "task_id": "task-1",
            "task_type": "local_bash",
        });
        if let Some(command) = command {
            value["command"] = serde_json::Value::String(command.to_string());
        }
        value
    }

    #[test]
    fn task_stop_truncates_command_like_official_ui() {
        let value = raw(Some("one\ntwo\nthree"));
        let lines = render_tool_result_message(Some(&value), false);
        assert_eq!(lines[0].text, "one\ntwo… · stopped");

        let verbose_lines = render_tool_result_message(Some(&value), true);
        assert_eq!(verbose_lines[0].text, "one\ntwo\nthree · stopped");
    }

    #[test]
    fn task_stop_absent_command_renders_bare_stopped_suffix() {
        // CC: `output.command ?? ''` — an old transcript without the optional
        // field renders " · stopped" with an empty command.
        let value = raw(None);
        let lines = render_tool_result_message(Some(&value), false);
        assert_eq!(lines[0].text, " · stopped");
    }

    #[test]
    fn task_stop_parse_rejects_missing_required_fields() {
        assert!(parse_output(&serde_json::json!({"command": "sleep 5"})).is_none());
        assert!(parse_output(&serde_json::json!("InputValidationError: x")).is_none());
        let output = parse_output(&raw(Some("sleep 5"))).unwrap();
        assert_eq!(output.command.as_deref(), Some("sleep 5"));
        assert_eq!(output_to_value(&output), raw(Some("sleep 5")));
    }
}
