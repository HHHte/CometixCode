//! Main-screen-safe subset of official `WebFetchTool/UI.tsx`.

use crate::components::messages::user_tool_result_message::utils::{
    ToolRenderLine, ToolRenderOptions, ToolRenderTone,
};
use crate::types::message::ToolResultStatus;

/// Maps to: CC `tools/WebFetchTool/UI.tsx` `renderToolUseMessage`.
/// Maps to: CC `tools/WebFetchTool/UI.tsx:60-67` `getToolUseSummary`.
pub fn get_tool_use_summary(input: Option<&serde_json::Value>) -> Option<String> {
    let url = input?.get("url")?.as_str().filter(|url| !url.is_empty())?;
    Some(crate::utils::truncate::truncate_to_width(
        url,
        crate::constants::tool_limits::TOOL_SUMMARY_MAX_LENGTH,
    ))
}

pub fn render_tool_use_message(
    url: Option<&str>,
    prompt: Option<&str>,
    verbose: bool,
) -> Option<String> {
    let url = url.filter(|url| !url.is_empty())?;
    if verbose {
        if let Some(prompt) = prompt.filter(|prompt| !prompt.is_empty()) {
            return Some(format!("url: \"{url}\", prompt: \"{prompt}\""));
        }
        return Some(format!("url: \"{url}\""));
    }
    Some(url.to_string())
}

/// The Rust stand-in for CC's `outputSchema.safeParse(toolUseResult)`
/// (`UserToolSuccessMessage.tsx:80`): all six declared fields are required
/// (`WebFetchTool.ts:32-45`); unknown object keys are stripped exactly as
/// Zod's default object behavior.
pub(crate) fn parse_output(value: &serde_json::Value) -> Option<super::Output> {
    let map = value.as_object()?;
    Some(super::Output {
        bytes: usize::try_from(map.get("bytes")?.as_u64()?).ok()?,
        code: map.get("code")?.as_i64()?,
        code_text: map.get("codeText")?.as_str()?.to_string(),
        result: map.get("result")?.as_str()?.to_string(),
        duration_ms: map.get("durationMs")?.as_u64()?,
        url: map.get("url")?.as_str()?.to_string(),
    })
}

/// Serializes [`super::Output`] back to CC's exact `toolUseResult` wire shape.
pub(crate) fn output_to_value(output: &super::Output) -> serde_json::Value {
    serde_json::json!({
        "bytes": output.bytes,
        "code": output.code,
        "codeText": output.code_text,
        "result": output.result,
        "durationMs": output.duration_ms,
        "url": output.url,
    })
}

/// Maps to: CC `WebFetchTool/UI.tsx:31-58` `renderToolResultMessage` as
/// invoked by `UserToolSuccessMessage.tsx:80-96` — parse the raw
/// `toolUseResult` with the tool's own output schema, render the
/// "Received {size} ({code} {codeText})" summary (verbose adds the full
/// result body), and render nothing when it does not parse.
///
/// This is the by-tool-name entry the dispatch calls — the only WebFetch
/// render path now that the WebFetch display variant is gone.
pub(crate) fn render_tool_result_message(
    raw_output: Option<&serde_json::Value>,
    _status: ToolResultStatus,
    _fallback: &str,
    options: &ToolRenderOptions,
) -> Vec<ToolRenderLine> {
    // CC bails on a missing `toolUseResult` before touching the tool
    // (`UserToolSuccessMessage.tsx:72`).
    let Some(raw_output) = raw_output else {
        return Vec::new();
    };
    // CC: `safeParse` failure returns null, i.e. the row renders nothing
    // (`UserToolSuccessMessage.tsx:81`).
    let Some(output) = parse_output(raw_output) else {
        return Vec::new();
    };
    let formatted_size =
        crate::components::messages::user_tool_result_message::utils::format_file_size(
            output.bytes as u64,
        );
    let summary = format!(
        "Received {formatted_size} ({} {})",
        output.code, output.code_text
    );
    let mut lines = vec![ToolRenderLine::new(summary, ToolRenderTone::Normal)];
    if options.verbose && !output.result.trim().is_empty() {
        lines.extend(
            output
                .result
                .lines()
                .map(|line| ToolRenderLine::new(line, ToolRenderTone::Normal)),
        );
    }
    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn web_fetch_tool_use_message_matches_official_verbose_and_compact_copy() {
        assert_eq!(render_tool_use_message(None, None, false), None);
        assert_eq!(
            render_tool_use_message(Some("https://example.com"), None, false).as_deref(),
            Some("https://example.com")
        );
        assert_eq!(
            render_tool_use_message(Some("https://example.com"), Some("summarize"), true)
                .as_deref(),
            Some("url: \"https://example.com\", prompt: \"summarize\"")
        );
    }

    /// Maps to: CC `WebFetchTool/UI.tsx:31-58` — the raw entry is now the
    /// only WebFetch render path; missing/rejected raw renders nothing.
    #[test]
    fn web_fetch_summary_matches_official_copy() {
        let raw = serde_json::json!({
            "bytes": 25600,
            "code": 200,
            "codeText": "OK",
            "result": "# Title\nBody",
            "durationMs": 12,
            "url": "https://example.com"
        });
        let lines = render_tool_result_message(
            Some(&raw),
            ToolResultStatus::Success,
            "",
            &ToolRenderOptions::default(),
        );
        assert_eq!(lines[0].text, "Received 25 KB (200 OK)");
        assert_eq!(lines[0].tone, ToolRenderTone::Normal);
        assert_eq!(lines.len(), 1);

        let verbose = render_tool_result_message(
            Some(&raw),
            ToolResultStatus::Success,
            "",
            &ToolRenderOptions {
                verbose: true,
                ..ToolRenderOptions::default()
            },
        );
        assert_eq!(verbose[1].text, "# Title");
        assert_eq!(verbose[2].text, "Body");

        assert!(
            render_tool_result_message(
                None,
                ToolResultStatus::Success,
                "",
                &ToolRenderOptions::default(),
            )
            .is_empty()
        );
        assert!(
            render_tool_result_message(
                Some(&serde_json::json!({"bytes": 10})),
                ToolResultStatus::Success,
                "",
                &ToolRenderOptions::default(),
            )
            .is_empty()
        );
    }
}
