//! Maps to: CC `components/FallbackToolUseErrorMessage.tsx`.
//!
//! Fallback rendering for tool errors that do not have a tool-specific result
//! component. The official component strips model-only tags, removes sandbox
//! violation payloads from UI copy, truncates non-verbose output to ten lines,
//! and shows the transcript shortcut hint. This port keeps the same component
//! boundary and accepts an already-known string result; `None` represents the
//! official non-string `ToolResultBlockParam['content']` branch.

use crate::components::message_response::MessageResponse;
use iocraft::prelude::*;

const MAX_RENDERED_LINES: usize = 10;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FallbackToolUseErrorRender {
    pub visible_error: String,
    pub hidden_lines: usize,
}

#[derive(Default, Props)]
pub struct FallbackToolUseErrorMessageProps {
    pub result: Option<String>,
    pub verbose: bool,
    /// Already-resolved display for `app:toggleTranscript`; official falls
    /// back to `ctrl+o`.
    pub transcript_shortcut: Option<String>,
}

/// Maps to: CC `FallbackToolUseErrorMessage` error normalization branch.
pub fn fallback_tool_use_error_render(
    result: Option<&str>,
    verbose: bool,
) -> FallbackToolUseErrorRender {
    let error = match result {
        None => "Tool execution failed".to_string(),
        Some(result) => normalize_tool_error_text(result, verbose),
    };

    let lines = error.lines().collect::<Vec<_>>();
    let hidden_lines = lines.len().saturating_sub(MAX_RENDERED_LINES);
    let visible_error = if !verbose && hidden_lines > 0 {
        lines[..MAX_RENDERED_LINES].join("\n")
    } else {
        error
    };

    FallbackToolUseErrorRender {
        visible_error: strip_underline_ansi(&visible_error),
        hidden_lines: if verbose { 0 } else { hidden_lines },
    }
}

fn normalize_tool_error_text(result: &str, verbose: bool) -> String {
    let extracted = crate::utils::messages::extract_tag(result, "tool_use_error")
        .unwrap_or_else(|| result.to_string());
    let without_sandbox_violations = remove_tag_section(&extracted, "sandbox_violations");
    let without_error_tags = without_sandbox_violations
        .replace("<error>", "")
        .replace("</error>", "");
    let trimmed = without_error_tags.trim();

    if !verbose && trimmed.contains("InputValidationError: ") {
        "Invalid tool parameters".to_string()
    } else if trimmed.starts_with("Error: ") || trimmed.starts_with("Cancelled: ") {
        trimmed.to_string()
    } else {
        format!("Error: {trimmed}")
    }
}

/// Maps to: CC `utils/sandbox/sandbox-ui-utils.ts#removeSandboxViolationTags`.
fn remove_tag_section(content: &str, tag: &str) -> String {
    let mut result = String::new();
    let mut rest = content;
    let open_prefix = format!("<{tag}");
    let close = format!("</{tag}>");

    while let Some(open_start) = rest.find(&open_prefix) {
        let before = &rest[..open_start];
        result.push_str(before);
        let after_open = &rest[open_start..];
        let Some(open_end) = after_open.find('>') else {
            result.push_str(after_open);
            return result;
        };
        let after_open_tag = &after_open[open_end + 1..];
        let Some(close_start) = after_open_tag.find(&close) else {
            return result;
        };
        rest = &after_open_tag[close_start + close.len()..];
    }
    result.push_str(rest);
    result
}

/// Maps to: CC `components/shell/OutputLine.tsx#stripUnderlineAnsi`.
fn strip_underline_ansi(input: &str) -> String {
    // Removes SGR underline toggles while preserving all other text. This is a
    // narrow UI sanitizer, not a full ANSI parser.
    input
        .replace("\u{1b}[4m", "")
        .replace("\u{1b}[24m", "")
        .replace("\u{1b}[04m", "")
        .replace("\u{1b}[0;4m", "\u{1b}[0m")
}

/// The line-pipeline projection of this component: same visible error and
/// truncation marker, emitted as ToolRenderLines for the channels that render
/// rows instead of elements (subagent collapsed progress, transcript,
/// conversation recovery).
pub fn fallback_tool_use_error_lines(
    content: &str,
    verbose: bool,
) -> Vec<crate::components::messages::user_tool_result_message::utils::ToolRenderLine> {
    use crate::components::messages::user_tool_result_message::utils::{
        ToolRenderLine, ToolRenderTone,
    };
    let render = fallback_tool_use_error_render(Some(content), verbose);
    let mut lines = render
        .visible_error
        .lines()
        .map(|line| ToolRenderLine::new(line, ToolRenderTone::Error))
        .collect::<Vec<_>>();
    if render.hidden_lines > 0 {
        lines.push(
            ToolRenderLine::new(
                format!(
                    "… +{} {} (ctrl+o to see all)",
                    render.hidden_lines,
                    if render.hidden_lines == 1 {
                        "line"
                    } else {
                        "lines"
                    },
                ),
                ToolRenderTone::Inactive,
            )
            .with_dim(true),
        );
    }
    lines
}

/// Maps to: CC `components/FallbackToolUseErrorMessage.tsx#FallbackToolUseErrorMessage`.
#[component]
pub fn FallbackToolUseErrorMessage(
    props: &FallbackToolUseErrorMessageProps,
    hooks: Hooks,
) -> impl Into<AnyElement<'static>> {
    let theme = hooks.use_context::<crate::utils::theme::Theme>();
    let render = fallback_tool_use_error_render(props.result.as_deref(), props.verbose);
    let shortcut = props
        .transcript_shortcut
        .clone()
        .unwrap_or_else(|| "ctrl+o".to_string());

    element! {
        MessageResponse {
            View(flex_direction: FlexDirection::Column) {
                // Ink's Text component interprets ANSI and applies its parent
                // color again after SGR reset. iocraft's plain Text does not,
                // so Ansi is the transport adapter for the same behavior.
                Ansi(content: render.visible_error, color: Some(theme.error), wrap: TextWrap::Wrap)
                #(if render.hidden_lines > 0 {
                    Some(element! {
                        View(flex_direction: FlexDirection::Row) {
                            Text(
                                content: format!(
                                    "… +{} {} (",
                                    render.hidden_lines,
                                    if render.hidden_lines == 1 { "line" } else { "lines" },
                                ),
                                color: theme.inactive,
                                dim: true,
                                wrap: TextWrap::NoWrap,
                            )
                            Text(content: shortcut, color: theme.inactive, dim: true, weight: Weight::Bold, wrap: TextWrap::NoWrap)
                            Text(content: " ".to_string(), wrap: TextWrap::NoWrap)
                            Text(content: "to see all)".to_string(), color: theme.inactive, dim: true, wrap: TextWrap::NoWrap)
                        }
                    })
                } else {
                    None
                })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::theme;

    fn render(result: Option<String>, verbose: bool) -> String {
        element! {
            ContextProvider(value: Context::owned(*theme::current())) {
                FallbackToolUseErrorMessage(result: result, verbose: verbose)
            }
        }
        .render(Some(120))
        .to_string()
    }

    #[test]
    fn fallback_tool_error_normalizes_tags_and_prefix_like_official() {
        let output = fallback_tool_use_error_render(
            Some(
                "<tool_use_error><sandbox_violations>secret</sandbox_violations><error>boom</error></tool_use_error>",
            ),
            false,
        );
        assert_eq!(output.visible_error, "Error: boom");
        assert_eq!(output.hidden_lines, 0);

        let invalid =
            fallback_tool_use_error_render(Some("InputValidationError: bad schema"), false);
        assert_eq!(invalid.visible_error, "Invalid tool parameters");
        let verbose =
            fallback_tool_use_error_render(Some("InputValidationError: bad schema"), true);
        assert_eq!(
            verbose.visible_error,
            "Error: InputValidationError: bad schema"
        );
    }

    #[test]
    fn fallback_tool_error_truncates_non_verbose_with_transcript_hint() {
        let long = (0..12)
            .map(|idx| format!("line {idx}"))
            .collect::<Vec<_>>()
            .join("\n");
        let text = render(Some(long), false);
        assert!(text.contains("line 0"), "canvas=\n{text}");
        assert!(!text.contains("line 11"), "canvas=\n{text}");
        assert!(
            text.contains("… +2 lines (ctrl+o to see all)"),
            "canvas=\n{text}"
        );
    }

    #[test]
    fn fallback_tool_error_non_string_result_uses_official_default() {
        let text = render(None, false);
        assert!(text.contains("Tool execution failed"), "canvas=\n{text}");
    }
}
