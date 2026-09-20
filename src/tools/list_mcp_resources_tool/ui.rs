//! UI-only port of official `tools/ListMcpResourcesTool/UI.tsx`.

use crate::components::messages::user_tool_result_message::utils::{
    ToolRenderLine, ToolRenderTone,
};

use super::{McpResource, Output};

/// The Rust stand-in for CC's `outputSchema.safeParse(toolUseResult)`
/// (`UserToolSuccessMessage.tsx:80`): a top-level array whose every element
/// requires `uri`/`name`/`server` (`ListMcpResourcesTool.ts:25-35`); a
/// malformed element rejects the whole payload, as Zod does.
pub(crate) fn parse_output(value: &serde_json::Value) -> Option<Output> {
    let optional =
        |map: &serde_json::Map<String, serde_json::Value>, key: &str| -> Option<Option<String>> {
            match map.get(key) {
                None => Some(None),
                Some(serde_json::Value::String(value)) => Some(Some(value.clone())),
                Some(_) => None,
            }
        };
    value
        .as_array()?
        .iter()
        .map(|resource| {
            let map = resource.as_object()?;
            Some(McpResource {
                uri: map.get("uri")?.as_str()?.to_string(),
                name: map.get("name")?.as_str()?.to_string(),
                mime_type: optional(map, "mimeType")?,
                description: optional(map, "description")?,
                server: map.get("server")?.as_str()?.to_string(),
            })
        })
        .collect::<Option<Vec<_>>>()
}

/// Maps to: CC `ListMcpResourcesTool/UI.tsx:18-35` `renderToolResultMessage`
/// — "(No resources found)" for an empty array, else the pretty-printed JSON
/// body; missing/rejected raw renders nothing
/// (`UserToolSuccessMessage.tsx:72,81`).
pub(crate) fn render_tool_result_message(
    raw_output: Option<&serde_json::Value>,
) -> Vec<ToolRenderLine> {
    let Some(output) = raw_output.and_then(parse_output) else {
        return Vec::new();
    };
    if output.is_empty() {
        return vec![ToolRenderLine::new(
            "(No resources found)",
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

pub(crate) fn list_mcp_resources_tool_use_summary(input: &serde_json::Value) -> Option<String> {
    crate::components::messages::user_tool_result_message::utils::first_string(input, &["server"])
        .map(|server| format!("List MCP resources from server \"{server}\""))
        .or_else(|| Some("List all MCP resources".to_string()))
}
