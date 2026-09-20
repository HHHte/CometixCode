//! UI-only port of official `tools/LSPTool/UI.tsx`.

use super::Output;
use crate::components::messages::user_tool_result_message::utils::{
    ToolRenderLine, ToolRenderSegment, ToolRenderTone,
};

/// Maps to: CC `LSPTool/UI.tsx:12-26` `OPERATION_LABELS`, including the
/// `|| {singular: 'result', plural: 'results'}` fallback.
fn operation_labels(operation: &str) -> (&'static str, &'static str, Option<&'static str>) {
    match operation {
        "goToDefinition" => ("definition", "definitions", None),
        "findReferences" => ("reference", "references", None),
        "documentSymbol" | "workspaceSymbol" => ("symbol", "symbols", None),
        "hover" => ("hover info", "hover info", Some("available")),
        "goToImplementation" => ("implementation", "implementations", None),
        "prepareCallHierarchy" => ("call item", "call items", None),
        "incomingCalls" => ("caller", "callers", None),
        "outgoingCalls" => ("callee", "callees", None),
        _ => ("result", "results", None),
    }
}

const OPERATIONS: [&str; 9] = [
    "goToDefinition",
    "findReferences",
    "hover",
    "documentSymbol",
    "workspaceSymbol",
    "goToImplementation",
    "prepareCallHierarchy",
    "incomingCalls",
    "outgoingCalls",
];

/// The Rust stand-in for CC's `outputSchema.safeParse(toolUseResult)`
/// (`UserToolSuccessMessage.tsx:80`): `operation` must be one of the nine
/// enum values, `result` and `filePath` are required strings,
/// `resultCount`/`fileCount` optional non-negative integers
/// (`LSPTool.ts:89-121`).
pub(crate) fn parse_output(value: &serde_json::Value) -> Option<Output> {
    let map = value.as_object()?;
    let optional_count = |key: &str| -> Option<Option<usize>> {
        match map.get(key) {
            None => Some(None),
            Some(serde_json::Value::Number(value)) => {
                Some(Some(usize::try_from(value.as_u64()?).ok()?))
            }
            Some(_) => None,
        }
    };
    let operation = map.get("operation")?.as_str()?.to_string();
    if !OPERATIONS.contains(&operation.as_str()) {
        return None;
    }
    Some(Output {
        operation,
        result: map.get("result")?.as_str()?.to_string(),
        file_path: map.get("filePath")?.as_str()?.to_string(),
        result_count: optional_count("resultCount")?,
        file_count: optional_count("fileCount")?,
    })
}

/// Serializes [`Output`] to CC's exact `toolUseResult` wire shape — schema
/// declaration order, optional fields omitted.
pub(crate) fn output_to_value(output: &Output) -> serde_json::Value {
    let mut map = serde_json::Map::new();
    map.insert(
        "operation".to_string(),
        serde_json::Value::String(output.operation.clone()),
    );
    map.insert(
        "result".to_string(),
        serde_json::Value::String(output.result.clone()),
    );
    map.insert(
        "filePath".to_string(),
        serde_json::Value::String(output.file_path.clone()),
    );
    if let Some(count) = output.result_count {
        map.insert("resultCount".to_string(), serde_json::json!(count));
    }
    if let Some(count) = output.file_count {
        map.insert("fileCount".to_string(), serde_json::json!(count));
    }
    serde_json::Value::Object(map)
}

/// Maps to: CC `LSPTool/UI.tsx:160-176` `renderToolUseErrorMessage` — the
/// compact "LSP operation failed" replaces the tagged copy only when not
/// verbose; `None` falls to `FallbackToolUseErrorMessage`.
pub(crate) fn render_tool_use_error_message(content: &str, verbose: bool) -> Option<String> {
    if !verbose && crate::utils::messages::extract_tag(content, "tool_use_error").is_some() {
        return Some("LSP operation failed".to_string());
    }
    None
}

/// Maps to: CC `LSPTool/UI.tsx:178-203` `renderToolResultMessage` — the
/// collapsed/expanded `LSPResultSummary` when both counts are present, else
/// the bare `{output.result}` fallback (LSP server init failures, request
/// errors).
pub(crate) fn render_tool_result_message(
    raw_output: Option<&serde_json::Value>,
    verbose: bool,
) -> Vec<ToolRenderLine> {
    let Some(output) = raw_output.and_then(parse_output) else {
        return Vec::new();
    };
    let (Some(result_count), Some(file_count)) = (output.result_count, output.file_count) else {
        return vec![ToolRenderLine::new(output.result, ToolRenderTone::Normal)];
    };

    // LSPResultSummary (UI.tsx:31-96).
    let (singular, plural, special) = operation_labels(&output.operation);
    let mut summary = String::new();
    let mut segments = Vec::new();
    if output.operation == "hover" && result_count > 0 && special.is_some() {
        summary.push_str("Hover info available");
        segments.push(ToolRenderSegment::new("Hover info available"));
    } else {
        let label = if result_count == 1 { singular } else { plural };
        summary.push_str(&format!("Found {result_count} {label}"));
        segments.extend([
            ToolRenderSegment::new("Found "),
            ToolRenderSegment::new(format!("{result_count} ")).with_bold(true),
            ToolRenderSegment::new(label),
        ]);
    }
    if file_count > 1 {
        summary.push_str(&format!(" across {file_count} files"));
        segments.extend([
            ToolRenderSegment::new(" across "),
            ToolRenderSegment::new(format!("{file_count} ")).with_bold(true),
            ToolRenderSegment::new("files"),
        ]);
    }

    if verbose {
        let mut lines =
            vec![ToolRenderLine::new(summary, ToolRenderTone::Normal).with_segments(segments)];
        lines.push(ToolRenderLine::new(output.result, ToolRenderTone::Normal));
        return lines;
    }

    // `{primaryText}{secondaryText} {resultCount > 0 && <CtrlOToExpand />}` —
    // the JSX space is unconditional; the hint appends only with results.
    summary.push(' ');
    segments.push(ToolRenderSegment::new(" "));
    if result_count > 0 {
        let hint = crate::components::ctrl_o_to_expand::ctrl_o_to_expand_hint();
        summary.push_str(&hint);
        segments.push(ToolRenderSegment::new(hint).with_dim(true));
    }
    vec![ToolRenderLine::new(summary, ToolRenderTone::Normal).with_segments(segments)]
}

/// Maps to: CC `tools/LSPTool/UI.tsx:102-158` `renderToolUseMessage`.
pub(crate) fn lsp_tool_use_summary(input: &serde_json::Value, verbose: bool) -> Option<String> {
    let operation = crate::components::messages::user_tool_result_message::utils::first_string(
        input,
        &["operation"],
    )?;
    let file_path = crate::components::messages::user_tool_result_message::utils::first_string(
        input,
        &["filePath"],
    );
    let display_path = |path: &str| -> String {
        if verbose {
            path.to_string()
        } else {
            crate::utils::file::get_display_path(path)
        }
    };

    let position = matches!(
        operation.as_str(),
        "goToDefinition" | "findReferences" | "hover" | "goToImplementation"
    )
    .then(|| {
        Some((
            file_path.clone()?,
            crate::components::messages::user_tool_result_message::utils::first_i64(
                input,
                &["line"],
            )?,
            crate::components::messages::user_tool_result_message::utils::first_i64(
                input,
                &["character"],
            )?,
        ))
    })
    .flatten();

    if let Some((file_path, line, character)) = position {
        let symbol = super::symbol_context::get_symbol_at_position(
            &file_path,
            line.saturating_sub(1),
            character.saturating_sub(1),
        );
        let display_path = display_path(&file_path);
        return Some(match symbol {
            Some(symbol) => {
                format!("operation: \"{operation}\", symbol: \"{symbol}\", in: \"{display_path}\"")
            }
            None => format!(
                "operation: \"{operation}\", file: \"{display_path}\", position: {line}:{character}"
            ),
        });
    }

    let mut parts = vec![format!("operation: \"{operation}\"")];
    if let Some(file_path) = file_path {
        parts.push(format!("file: \"{}\"", display_path(&file_path)));
    }
    Some(parts.join(", "))
}
