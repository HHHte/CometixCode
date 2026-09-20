//! UI-only port of official `tools/ReadMcpResourceTool/UI.tsx`.

use crate::components::messages::user_tool_result_message::utils::{
    ToolRenderLine, ToolRenderTone,
};

use super::{Output, ResourceContent};

/// The Rust stand-in for CC's `outputSchema.safeParse(toolUseResult)`
/// (`UserToolSuccessMessage.tsx:80`): `contents` is required and every
/// element requires `uri` (`ReadMcpResourceTool.ts:30-44`); a malformed
/// element rejects the whole payload, as Zod does.
pub(crate) fn parse_output(value: &serde_json::Value) -> Option<Output> {
    let optional =
        |map: &serde_json::Map<String, serde_json::Value>, key: &str| -> Option<Option<String>> {
            match map.get(key) {
                None => Some(None),
                Some(serde_json::Value::String(value)) => Some(Some(value.clone())),
                Some(_) => None,
            }
        };
    let contents = value
        .as_object()?
        .get("contents")?
        .as_array()?
        .iter()
        .map(|content| {
            let map = content.as_object()?;
            Some(ResourceContent {
                uri: map.get("uri")?.as_str()?.to_string(),
                mime_type: optional(map, "mimeType")?,
                text: optional(map, "text")?,
                blob_saved_to: optional(map, "blobSavedTo")?,
            })
        })
        .collect::<Option<Vec<_>>>()?;
    Some(Output { contents })
}

/// Maps to: CC `ReadMcpResourceTool/UI.tsx:24-46` `renderToolResultMessage`
/// — "(No content)" for empty contents, else the pretty-printed JSON body;
/// missing/rejected raw renders nothing (`UserToolSuccessMessage.tsx:72,81`).
pub(crate) fn render_tool_result_message(
    raw_output: Option<&serde_json::Value>,
) -> Vec<ToolRenderLine> {
    let Some(output) = raw_output.and_then(parse_output) else {
        return Vec::new();
    };
    if output.contents.is_empty() {
        return vec![ToolRenderLine::new(
            "(No content)",
            ToolRenderTone::Inactive,
        )];
    }
    let formatted =
        crate::components::messages::user_tool_result_message::utils::pretty_json_for_display(
            &super::output_to_value(&output),
        );
    vec![ToolRenderLine::new(
        formatted.trim(),
        ToolRenderTone::Normal,
    )]
}

pub(crate) fn read_mcp_resource_tool_use_summary(input: &serde_json::Value) -> Option<String> {
    let uri = crate::components::messages::user_tool_result_message::utils::first_string(
        input,
        &["uri"],
    )?;
    let server = crate::components::messages::user_tool_result_message::utils::first_string(
        input,
        &["server"],
    )?;
    Some(format!("Read resource \"{uri}\" from server \"{server}\""))
}
