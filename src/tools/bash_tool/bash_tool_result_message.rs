//! Maps to: CC `tools/BashTool/BashToolResultMessage.tsx`.
//!
//! Render-state helpers for Bash tool results. The actual iocraft row painting
//! remains in `ui.rs`, mirroring CC's split between `UI.tsx` and this result
//! message component.

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct BashToolResultContent {
    pub stdout: String,
    pub stderr: String,
    pub is_image: bool,
    pub return_code_interpretation: Option<String>,
    pub no_output_expected: bool,
    pub background_task_id: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct BashToolResultMessageParts {
    pub stdout: String,
    pub stderr: String,
    pub cwd_reset_warning: Option<String>,
    pub empty_state_text: Option<String>,
    pub is_image: bool,
}

/// Maps to CC `extractSandboxViolations(stderr)`.
pub fn extract_sandbox_violations(stderr: &str) -> String {
    let mut output = String::new();
    let mut rest = stderr;
    const START: &str = "<sandbox_violations>";
    const END: &str = "</sandbox_violations>";

    while let Some(start) = rest.find(START) {
        output.push_str(&rest[..start]);
        let after_start = &rest[start + START.len()..];
        if let Some(end) = after_start.find(END) {
            rest = &after_start[end + END.len()..];
        } else {
            rest = "";
            break;
        }
    }
    output.push_str(rest);
    output.trim().to_string()
}

/// Maps to CC `extractCwdResetWarning(stderr)`.
pub fn extract_cwd_reset_warning(stderr: &str) -> (String, Option<String>) {
    const PREFIX: &str = "Shell cwd was reset to ";
    let trimmed = stderr.trim_end();
    if trimmed.is_empty() {
        return (String::new(), None);
    }

    let (before_last_line, last_line) = match trimmed.rsplit_once('\n') {
        Some((before, last)) => (before, last),
        None => ("", trimmed),
    };
    if !last_line.starts_with(PREFIX) {
        return (trimmed.to_string(), None);
    }

    (
        before_last_line.trim().to_string(),
        Some(last_line.to_string()),
    )
}

/// Maps to CC `BashToolResultMessage(...)` render branching, represented as
/// deterministic text parts for the Rust renderer.
pub fn bash_tool_result_message_parts(
    content: &BashToolResultContent,
) -> BashToolResultMessageParts {
    if content.is_image {
        return BashToolResultMessageParts {
            empty_state_text: Some("[Image data detected and sent to Claude]".to_string()),
            is_image: true,
            ..BashToolResultMessageParts::default()
        };
    }

    // Shell file mode merges stderr into stdout. Non-zero Bash results retain
    // sandbox tags in that model-facing merged string, so clean both channels
    // at the Rust transcript projection boundary.
    let stdout_without_violations = extract_sandbox_violations(&content.stdout);
    let stderr_without_violations = extract_sandbox_violations(&content.stderr);
    let (stderr, cwd_reset_warning) = extract_cwd_reset_warning(&stderr_without_violations);
    let empty_state_text =
        if content.stdout.is_empty() && stderr.trim().is_empty() && cwd_reset_warning.is_none() {
            Some(empty_result_text(content))
        } else {
            None
        };

    BashToolResultMessageParts {
        stdout: stdout_without_violations,
        stderr,
        cwd_reset_warning,
        empty_state_text,
        is_image: false,
    }
}

/// Maps to CC empty-output branch inside `BashToolResultMessage`.
pub fn empty_result_text(content: &BashToolResultContent) -> String {
    if content.background_task_id.is_some() {
        "Running in the background (↓ to manage)".to_string()
    } else if let Some(text) = content
        .return_code_interpretation
        .as_deref()
        .filter(|text| !text.is_empty())
    {
        text.to_string()
    } else if content.no_output_expected {
        "Done".to_string()
    } else {
        "(No output)".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_sandbox_violations_and_extracts_cwd_reset_warning() {
        let content = BashToolResultContent {
            stdout: "visible\n<sandbox_violations>merged hidden</sandbox_violations>".to_string(),
            stderr: "first error\n<sandbox_violations>hidden</sandbox_violations>\nShell cwd was reset to /tmp/project".to_string(),
            ..BashToolResultContent::default()
        };
        let parts = bash_tool_result_message_parts(&content);
        assert_eq!(parts.stdout, "visible");
        assert_eq!(parts.stderr, "first error");
        assert_eq!(
            parts.cwd_reset_warning.as_deref(),
            Some("Shell cwd was reset to /tmp/project")
        );
        assert!(parts.empty_state_text.is_none());
    }

    #[test]
    fn empty_output_branch_matches_official_priority_order() {
        assert_eq!(
            empty_result_text(&BashToolResultContent {
                background_task_id: Some("task_1".to_string()),
                return_code_interpretation: Some("No matches found".to_string()),
                no_output_expected: true,
                ..BashToolResultContent::default()
            }),
            "Running in the background (↓ to manage)"
        );
        assert_eq!(
            empty_result_text(&BashToolResultContent {
                return_code_interpretation: Some("No matches found".to_string()),
                no_output_expected: true,
                ..BashToolResultContent::default()
            }),
            "No matches found"
        );
        assert_eq!(
            empty_result_text(&BashToolResultContent {
                no_output_expected: true,
                ..BashToolResultContent::default()
            }),
            "Done"
        );
        assert_eq!(
            empty_result_text(&BashToolResultContent::default()),
            "(No output)"
        );
    }

    #[test]
    fn image_result_uses_official_placeholder_text() {
        let parts = bash_tool_result_message_parts(&BashToolResultContent {
            is_image: true,
            stdout: "data:image/png;base64,abc".to_string(),
            ..BashToolResultContent::default()
        });
        assert!(parts.is_image);
        assert_eq!(
            parts.empty_state_text.as_deref(),
            Some("[Image data detected and sent to Claude]")
        );
        assert!(parts.stdout.is_empty());
    }
}
