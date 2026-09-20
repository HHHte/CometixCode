//! UI-only renderer helpers for official `NotebookEditTool/UI.tsx`.

use crate::components::messages::user_tool_result_message::utils::{
    ToolRenderLine, ToolRenderOptions, ToolRenderTone,
};
use crate::components::structured_diff::ColorFile;
use crate::types::message::ToolResultStatus;
use crate::utils::file::get_display_path;
use serde_json::Value;

use super::Output;

pub fn user_facing_name() -> &'static str {
    "Edit Notebook"
}

pub fn get_tool_use_summary(input: &Value) -> Option<String> {
    input
        .get("notebook_path")
        .and_then(|value| value.as_str())
        .filter(|path| !path.is_empty())
        .map(crate::utils::file::get_display_path)
}

/// Maps to: CC `NotebookEditTool/UI.tsx:41-53` `renderToolUseMessage`'s
/// linked-path representation: the path renders inside `<FilePathLink>`
/// (OSC-8 hyperlink) and the `@cell_id…` remainder stays outside it.
pub fn render_tool_use_path_link(input: &Value, verbose: bool) -> Option<(String, String)> {
    let notebook_path = input
        .get("notebook_path")
        .and_then(|value| value.as_str())
        .filter(|value| !value.is_empty())?;
    // The guards below mirror renderToolUseMessage: no header row at all
    // unless the other required fields are present.
    input
        .get("new_source")
        .and_then(|value| value.as_str())
        .filter(|value| !value.is_empty())?;
    input
        .get("cell_type")
        .and_then(|value| value.as_str())
        .filter(|value| !value.is_empty())?;
    let label = if verbose {
        notebook_path.to_string()
    } else {
        get_display_path(notebook_path)
    };
    Some((notebook_path.to_string(), label))
}

/// The `@cell_id…` remainder that follows the linked path — CC keeps it
/// outside `<FilePathLink>` (`UI.tsx:43-52`).
pub fn render_tool_use_message_suffix(input: &Value, verbose: bool) -> Option<String> {
    let message = render_tool_use_message(input, verbose)?;
    let path_len = render_tool_use_path_link(input, verbose)?.1.len();
    Some(message[path_len..].to_string())
}

pub fn render_tool_use_message(input: &Value, verbose: bool) -> Option<String> {
    // CC only accepts the snake_case schema keys; a camelCase payload fails
    // the strictObject safeParse at the call site
    // (`AssistantToolUseMessage.tsx:248-252`) and the row is hidden.
    let notebook_path = input
        .get("notebook_path")
        .and_then(|value| value.as_str())
        .filter(|value| !value.is_empty())?;
    let new_source = input
        .get("new_source")
        .and_then(|value| value.as_str())
        .filter(|value| !value.is_empty())?;
    let cell_type = input
        .get("cell_type")
        .and_then(|value| value.as_str())
        .filter(|value| !value.is_empty())?;
    let cell_id = input
        .get("cell_id")
        .and_then(|value| value.as_str())
        .unwrap_or("undefined");
    let edit_mode = input
        .get("edit_mode")
        .and_then(|value| value.as_str())
        .unwrap_or("replace");

    let display_path = if verbose {
        notebook_path.to_string()
    } else {
        get_display_path(notebook_path)
    };
    if verbose {
        Some(format!(
            "{display_path}@{cell_id}, content: {}…, cell_type: {cell_type}, edit_mode: {edit_mode}",
            js_slice_prefix(new_source, 30)
        ))
    } else {
        Some(format!("{display_path}@{cell_id}"))
    }
}

fn js_slice_prefix(value: &str, max_units: usize) -> String {
    let units = value.encode_utf16().take(max_units).collect::<Vec<_>>();
    String::from_utf16_lossy(&units)
}

/// The Rust stand-in for CC's `outputSchema.safeParse(toolUseResult)`
/// (`UserToolSuccessMessage.tsx:80`): the seven declared string fields are
/// required, `cell_type` must be the `code`/`markdown` enum, and `cell_id` /
/// `error` stay optional (`NotebookEditTool.ts:60-85`). Unknown keys are
/// stripped exactly as Zod's default object behavior.
pub fn parse_output(value: &Value) -> Option<Output> {
    let map = value.as_object()?;
    let required = |key: &str| {
        map.get(key)
            .and_then(Value::as_str)
            .map(ToString::to_string)
    };
    let optional = |key: &str| match map.get(key) {
        None => Some(None),
        Some(Value::String(value)) => Some(Some(value.clone())),
        Some(_) => None,
    };
    let cell_type = required("cell_type")?;
    if !matches!(cell_type.as_str(), "code" | "markdown") {
        return None;
    }
    Some(Output {
        new_source: required("new_source")?,
        cell_id: optional("cell_id")?,
        cell_type,
        language: required("language")?,
        edit_mode: required("edit_mode")?,
        error: optional("error")?,
        notebook_path: required("notebook_path")?,
        original_file: required("original_file")?,
        updated_file: required("updated_file")?,
        read_timestamp_ms: None,
    })
}

/// Serializes [`Output`] back to CC's exact `toolUseResult` wire shape —
/// the nine schema fields only; the private `read_timestamp_ms` transport
/// never reaches the wire.
pub(crate) fn output_to_value(output: &Output) -> Value {
    // Key order mirrors CC's call() object construction
    // (NotebookEditTool.ts:443-453): cell_id sits after edit_mode, and only
    // an absent cell_id drops its key (`new_cell_id || undefined`).
    let mut map = serde_json::Map::new();
    map.insert(
        "new_source".to_string(),
        Value::String(output.new_source.clone()),
    );
    map.insert(
        "cell_type".to_string(),
        Value::String(output.cell_type.clone()),
    );
    map.insert(
        "language".to_string(),
        Value::String(output.language.clone()),
    );
    map.insert(
        "edit_mode".to_string(),
        Value::String(output.edit_mode.clone()),
    );
    if let Some(cell_id) = output.cell_id.as_ref() {
        map.insert("cell_id".to_string(), Value::String(cell_id.clone()));
    }
    // CC's call() always materializes `error` — '' on success
    // (NotebookEditTool.ts:449), the message on failure; only `cell_id` uses
    // the undefined-drops-key channel (:448).
    map.insert(
        "error".to_string(),
        Value::String(output.error.clone().unwrap_or_default()),
    );
    map.insert(
        "notebook_path".to_string(),
        Value::String(output.notebook_path.clone()),
    );
    map.insert(
        "original_file".to_string(),
        Value::String(output.original_file.clone()),
    );
    map.insert(
        "updated_file".to_string(),
        Value::String(output.updated_file.clone()),
    );
    Value::Object(map)
}

/// The `<Text color="error">` leaf of the result renderer — a component so
/// the theme is read from context (CC `UI.tsx:106`).
#[iocraft::component]
pub fn NotebookEditErrorText(
    props: &NotebookEditErrorTextProps,
    hooks: iocraft::Hooks,
) -> impl Into<iocraft::AnyElement<'static>> {
    use iocraft::prelude::*;
    let theme = hooks.use_context::<crate::utils::theme::Theme>();
    element! {
        Text(content: props.content.clone(), color: theme.error)
    }
}

#[derive(Default, iocraft::Props)]
pub struct NotebookEditErrorTextProps {
    pub content: String,
}

/// Maps to: CC `NotebookEditTool/UI.tsx:99-124` `renderToolResultMessage` —
/// the element-pipeline owner. The error branch wins even on a success row;
/// otherwise "Updated cell {bold cell_id}:" over a `marginLeft={2}`
/// highlighted-source block. `cell_id` is a JSX hole, so `undefined` renders
/// as nothing.
pub(crate) fn render_tool_result_message(
    output: &Output,
    _verbose: bool,
) -> iocraft::AnyElement<'static> {
    use crate::components::highlighted_code::HighlightedCode;
    use crate::components::message_response::MessageResponse;
    use iocraft::prelude::*;

    if let Some(error) = output.error.as_ref().filter(|error| !error.is_empty()) {
        let error = error.clone();
        return element! {
            MessageResponse {
                NotebookEditErrorText(content: error)
            }
        }
        .into_any();
    }
    let cell_id = output.cell_id.clone().unwrap_or_default();
    let new_source = output.new_source.clone();
    element! {
        MessageResponse {
            View(flex_direction: FlexDirection::Column) {
                View(flex_direction: FlexDirection::Row) {
                    Text(content: "Updated cell ".to_string(), wrap: TextWrap::NoWrap)
                    Text(content: cell_id, weight: Weight::Bold, wrap: TextWrap::NoWrap)
                    Text(content: ":".to_string(), wrap: TextWrap::NoWrap)
                }
                View(margin_left: 2) {
                    HighlightedCode(code: new_source, file_path: "notebook.py".to_string())
                }
            }
        }
    }
    .into_any()
}

/// Maps to: CC `NotebookEditTool/UI.tsx:99-127` `renderToolResultMessage` as
/// invoked by `UserToolSuccessMessage.tsx:80-96` — parse the raw
/// `toolUseResult` with the tool's own output schema, render the
/// error-or-updated-cell body from the parsed value, and render nothing when
/// it does not parse.
///
/// The line-pipeline projection of the same renderer, for the channels that
/// emit rows instead of elements (subagent collapsed progress, conversation
/// recovery). The main screen goes through the element version above.
pub(crate) fn render_tool_result_lines(
    raw_output: Option<&Value>,
    status: ToolResultStatus,
    fallback: &str,
    options: &ToolRenderOptions,
) -> Vec<ToolRenderLine> {
    // CC bails on a missing `toolUseResult` before touching the tool
    // (`UserToolSuccessMessage.tsx:72`).
    let Some(raw_output) = raw_output else {
        return Vec::new();
    };
    if status != ToolResultStatus::Success {
        let tagged_validation_error =
            fallback.contains("<tool_use_error>") && fallback.contains("</tool_use_error>");
        let text =
            if status == ToolResultStatus::Error && !options.show_full() && tagged_validation_error
            {
                "Error editing notebook".to_string()
            } else if fallback.trim().is_empty() {
                match status {
                    ToolResultStatus::Error => "Error editing notebook".to_string(),
                    ToolResultStatus::Rejected => "Tool use rejected".to_string(),
                    ToolResultStatus::Canceled => "Interrupted by user".to_string(),
                    ToolResultStatus::Success => unreachable!(),
                }
            } else {
                fallback.trim().to_string()
            };
        return vec![ToolRenderLine::new(text, status_tone(status))];
    }
    // CC: `safeParse` failure returns null, i.e. the row renders nothing
    // (`UserToolSuccessMessage.tsx:81`).
    let Some(output) = parse_output(raw_output) else {
        return Vec::new();
    };

    // CC `renderToolResultMessage` body (`UI.tsx:103-126`): the error branch
    // wins even on a success row, then "Updated cell {id}:" plus highlighted
    // source.
    if let Some(error) = output.error.as_ref().filter(|error| !error.is_empty()) {
        return vec![ToolRenderLine::new(error, ToolRenderTone::Error)];
    }
    let cell_id = output.cell_id.as_deref().unwrap_or("undefined");
    let mut lines = vec![ToolRenderLine::new(
        format!("Updated cell {cell_id}:"),
        ToolRenderTone::Normal,
    )];
    if !output.new_source.is_empty() {
        if options.syntax_highlighting {
            lines.extend(
                ColorFile::new(output.new_source.clone(), "notebook.py").render(
                    options.syntax_theme,
                    options.terminal_width.saturating_sub(12).max(1),
                    false,
                ),
            );
        } else {
            lines.extend(
                output
                    .new_source
                    .lines()
                    .map(|line| ToolRenderLine::new(line.to_string(), ToolRenderTone::Inactive)),
            );
        }
    }
    lines
}

fn status_tone(status: ToolResultStatus) -> ToolRenderTone {
    // Maps to: CC `NotebookEditTool/UI.tsx` success — default `<Text>`.
    match status {
        ToolResultStatus::Success => ToolRenderTone::Normal,
        ToolResultStatus::Error => ToolRenderTone::Error,
        ToolResultStatus::Rejected => ToolRenderTone::Warning,
        ToolResultStatus::Canceled => ToolRenderTone::Inactive,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn notebook_edit_tool_use_matches_official_summary_shape() {
        let input = json!({
            "notebook_path": "/repo/notebooks/demo.ipynb",
            "cell_id": "abc123",
            "new_source": "print('hello')",
            "cell_type": "code"
        });

        assert_eq!(user_facing_name(), "Edit Notebook");
        assert_eq!(
            get_tool_use_summary(&input),
            Some("/repo/notebooks/demo.ipynb".to_string())
        );
        assert_eq!(
            render_tool_use_message(&input, false),
            Some("/repo/notebooks/demo.ipynb@abc123".to_string())
        );
    }

    #[test]
    fn notebook_edit_tool_use_preserves_official_truthiness_and_utf16_slice() {
        let empty_source = json!({
            "notebook_path": "/repo/demo.ipynb",
            "cell_id": "cell-a",
            "new_source": "",
            "cell_type": "code"
        });
        assert_eq!(render_tool_use_message(&empty_source, false), None);

        let input = json!({
            "notebook_path": "/repo/demo.ipynb",
            "cell_id": "cell-a",
            "new_source": "123456789012345678901234567890tail",
            "cell_type": "code"
        });
        let verbose = render_tool_use_message(&input, true).unwrap();
        assert!(verbose.contains("content: 123456789012345678901234567890…"));
        assert!(!verbose.contains("...…"));
    }

    #[test]
    fn notebook_edit_validation_error_compacts_only_outside_verbose_mode() {
        let raw = json!({
            "new_source": "",
            "cell_type": "code",
            "language": "python",
            "edit_mode": "replace",
            "error": "",
            "notebook_path": "/repo/demo.ipynb",
            "original_file": "",
            "updated_file": ""
        });
        let tagged = "<tool_use_error>Notebook file does not exist.</tool_use_error>";
        let compact = render_tool_result_lines(
            Some(&raw),
            ToolResultStatus::Error,
            tagged,
            &ToolRenderOptions::default(),
        );
        assert_eq!(compact[0].text, "Error editing notebook");

        let verbose = render_tool_result_lines(
            Some(&raw),
            ToolResultStatus::Error,
            tagged,
            &ToolRenderOptions {
                verbose: true,
                ..ToolRenderOptions::default()
            },
        );
        assert_eq!(verbose[0].text, tagged);
    }

    /// Maps to: CC `NotebookEditTool/UI.tsx:99-127` `renderToolResultMessage`
    /// — the error branch wins even on a success row; otherwise "Updated cell
    /// {id}:" plus the source body. Missing or schema-rejected raw renders
    /// nothing (`UserToolSuccessMessage.tsx:72,81`).
    #[test]
    fn render_tool_result_message_matches_official_success_and_error_bodies() {
        let success = json!({
            "new_source": "print('hi')",
            "cell_id": "cell-1",
            "cell_type": "code",
            "language": "python",
            "edit_mode": "replace",
            "error": "",
            "notebook_path": "/repo/demo.ipynb",
            "original_file": "{}",
            "updated_file": "{}"
        });
        let lines = render_tool_result_lines(
            Some(&success),
            ToolResultStatus::Success,
            "",
            &ToolRenderOptions::default(),
        );
        assert_eq!(lines[0].text, "Updated cell cell-1:");
        assert!(lines.iter().any(|line| line.text.contains("print('hi')")));

        let errored = json!({
            "new_source": "x",
            "cell_type": "code",
            "language": "python",
            "edit_mode": "replace",
            "error": "Notebook is not valid JSON.",
            "notebook_path": "/repo/demo.ipynb",
            "original_file": "",
            "updated_file": ""
        });
        let error_lines = render_tool_result_lines(
            Some(&errored),
            ToolResultStatus::Success,
            "",
            &ToolRenderOptions::default(),
        );
        assert_eq!(error_lines[0].text, "Notebook is not valid JSON.");
        assert_eq!(error_lines[0].tone, ToolRenderTone::Error);

        assert!(
            render_tool_result_lines(
                None,
                ToolResultStatus::Success,
                "",
                &ToolRenderOptions::default(),
            )
            .is_empty()
        );
        assert!(
            render_tool_result_lines(
                Some(&json!({"new_source": "x", "cell_type": "raw"})),
                ToolResultStatus::Success,
                "",
                &ToolRenderOptions::default(),
            )
            .is_empty()
        );
    }

    /// Maps to: CC `UI.tsx:113-123` — "Updated cell {bold id}:" over an
    /// indented HighlightedCode block; `cell_id` is a JSX hole, so a missing
    /// id renders as nothing rather than the literal "undefined".
    #[test]
    fn element_result_renders_official_body_and_omits_missing_cell_id() {
        use iocraft::prelude::ElementExt as _;
        let render = |cell_id: Option<&str>| {
            let output = Output {
                new_source: "print('hi')".to_string(),
                cell_id: cell_id.map(ToString::to_string),
                cell_type: "code".to_string(),
                language: "python".to_string(),
                edit_mode: "replace".to_string(),
                error: None,
                notebook_path: "/tmp/demo.ipynb".to_string(),
                original_file: String::new(),
                updated_file: String::new(),
                read_timestamp_ms: None,
            };
            iocraft::element! {
                iocraft::prelude::ContextProvider(
                    value: iocraft::prelude::Context::owned(*crate::utils::theme::current()),
                ) {
                    #(vec![render_tool_result_message(&output, false)])
                }
            }
            .render(Some(80))
            .to_string()
        };

        let with_id = render(Some("abc123"));
        assert!(with_id.contains("Updated cell abc123:"), "{with_id}");
        assert!(with_id.contains("print"), "{with_id}");

        let without_id = render(None);
        assert!(without_id.contains("Updated cell :"), "{without_id}");
        assert!(!without_id.contains("undefined"), "{without_id}");
    }
}
