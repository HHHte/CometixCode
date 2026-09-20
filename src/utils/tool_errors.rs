//! Maps to: CC `utils/toolErrors.ts` — the tool-facing error copy.
//!
//! Three exports, all producing text the model reads:
//!   - `format_error` / `get_error_parts` — a thrown tool error into content.
//!   - `format_zod_validation_error` — a schema validation failure into the
//!     `<tool_use_error>InputValidationError: …` body
//!     (`services/tools/toolExecution.ts:617`).
//!
//! The Zod half rebuilds the copy from issue *fields*, not from zod's own
//! message: it filters by `code`, and for `invalid_type` distinguishes a
//! missing parameter from a type mismatch by whether the message says
//! "received undefined". Anything it cannot rebuild falls back to
//! `ZodError.message` verbatim (the issues dump).

use crate::utils::errors::{AbortError, ShellError};
use crate::utils::messages::INTERRUPT_MESSAGE_FOR_TOOL_USE;
use crate::utils::zod::{IssueCode, PathSegment, ZodError};

/// Maps to: CC `toolErrors.ts:5-22` `formatError(error)`.
///
/// An `AbortError` renders its own message, falling back to the interrupt
/// notice. Everything else is joined from [`get_error_parts`]; an empty join
/// becomes `"Command failed with no output"`. Over 10 000 characters the middle
/// is elided, keeping 5 000 from each end.
pub fn format_error(error: &anyhow::Error) -> String {
    if let Some(abort) = error.downcast_ref::<AbortError>() {
        return if abort.message.is_empty() {
            INTERRUPT_MESSAGE_FOR_TOOL_USE.to_string()
        } else {
            abort.message.clone()
        };
    }

    let parts = get_error_parts(error);
    let joined = parts
        .into_iter()
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("\n");
    let full_message = joined.trim();
    let full_message = if full_message.is_empty() {
        "Command failed with no output"
    } else {
        full_message
    };

    // CC slices by UTF-16 code units; Rust slices by chars so a cut never lands
    // inside a codepoint. The limit is a guard, not a wire format.
    let total = full_message.chars().count();
    if total <= MAX_ERROR_LENGTH {
        return full_message.to_string();
    }
    let half_length = MAX_ERROR_LENGTH / 2;
    let start: String = full_message.chars().take(half_length).collect();
    let end: String = full_message.chars().skip(total - half_length).collect();
    format!(
        "{start}\n\n... [{} characters truncated] ...\n\n{end}",
        total - MAX_ERROR_LENGTH
    )
}

/// Maps to: CC `toolErrors.ts:24-39` `getErrorParts(error)`.
///
/// A `ShellError` reports exit code, the interruption notice when interrupted,
/// then stderr and stdout. Any other error reports its message, followed by
/// `stderr` / `stdout` when the error carries them — in Rust that shape is a
/// `ShellError`, so the generic arm is just the message.
pub fn get_error_parts(error: &anyhow::Error) -> Vec<String> {
    if let Some(shell) = error.downcast_ref::<ShellError>() {
        return vec![
            format!("Exit code {}", shell.code),
            if shell.interrupted {
                INTERRUPT_MESSAGE_FOR_TOOL_USE.to_string()
            } else {
                String::new()
            },
            shell.stderr.clone(),
            shell.stdout.clone(),
        ];
    }
    vec![error.to_string()]
}

/// Maps to: CC `toolErrors.ts:15,18` — the 10 000-character cap and its halves.
const MAX_ERROR_LENGTH: usize = 10_000;

/// Maps to: CC `toolErrors.ts:47-57` `formatValidationPath`.
///
/// `['todos', 0, 'activeForm']` → `todos[0].activeForm`: numeric segments are
/// bracketed, the first string segment has no leading dot.
fn format_validation_path(path: &[PathSegment]) -> String {
    if path.is_empty() {
        return String::new();
    }
    let mut acc = String::new();
    for (index, segment) in path.iter().enumerate() {
        match segment {
            PathSegment::Index(i) => {
                acc.push('[');
                acc.push_str(&i.to_string());
                acc.push(']');
            }
            PathSegment::Key(key) => {
                if index != 0 {
                    acc.push('.');
                }
                acc.push_str(key);
            }
        }
    }
    acc
}

/// Maps to: CC `toolErrors.ts:66-131` `formatZodValidationError(toolName, error)`.
///
/// Converts a Zod validation error into human- and LLM-readable copy. Falls
/// back to the raw `error.message` when no issue matches the three rebuilt
/// categories — that fallback is the whole issues dump, and is what the model
/// sees for `too_small` / `too_big` / `invalid_value` / `invalid_union` /
/// `custom`.
pub fn format_zod_validation_error(tool_name: &str, error: &ZodError) -> String {
    // `invalid_type` + "received undefined" ⇒ the parameter was not provided.
    let missing_params: Vec<String> = error
        .issues
        .iter()
        .filter(|issue| {
            issue.code == IssueCode::InvalidType && issue.message.contains("received undefined")
        })
        .map(|issue| format_validation_path(&issue.path))
        .collect();

    let unexpected_params: Vec<String> = error
        .issues
        .iter()
        .filter(|issue| issue.code == IssueCode::UnrecognizedKeys)
        .flat_map(|issue| issue.keys.iter().cloned())
        .collect();

    // `invalid_type` without that marker ⇒ a real type mismatch. CC reads the
    // received bucket back out of the message with /received (\w+)/, because
    // zod does not put it on the issue as a field.
    let type_mismatch_params: Vec<(String, String, String)> = error
        .issues
        .iter()
        .filter(|issue| {
            issue.code == IssueCode::InvalidType && !issue.message.contains("received undefined")
        })
        .map(|issue| {
            let received = received_from_message(&issue.message).unwrap_or("unknown");
            (
                format_validation_path(&issue.path),
                issue.expected.unwrap_or("unknown").to_string(),
                received.to_string(),
            )
        })
        .collect();

    let mut error_parts: Vec<String> = Vec::new();

    for param in &missing_params {
        error_parts.push(format!("The required parameter `{param}` is missing"));
    }
    for param in &unexpected_params {
        error_parts.push(format!("An unexpected parameter `{param}` was provided"));
    }
    for (param, expected, received) in &type_mismatch_params {
        error_parts.push(format!(
            "The parameter `{param}` type is expected as `{expected}` but provided as `{received}`"
        ));
    }

    if error_parts.is_empty() {
        // Default to the original error message if we can't create a better one.
        return error.message();
    }

    let noun = if error_parts.len() > 1 {
        "issues"
    } else {
        "issue"
    };
    format!(
        "{tool_name} failed due to the following {noun}:\n{}",
        error_parts.join("\n")
    )
}

/// CC: `err.message.match(/received (\w+)/)` — the word after "received ".
fn received_from_message(message: &str) -> Option<&str> {
    let rest = message.split("received ").nth(1)?;
    let end = rest
        .find(|c: char| !c.is_alphanumeric() && c != '_')
        .unwrap_or(rest.len());
    if end == 0 { None } else { Some(&rest[..end]) }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::zod::{Schema, safe_parse};
    use serde_json::json;

    fn error_for(schema: &Schema, input: serde_json::Value) -> ZodError {
        safe_parse(schema, &input).expect_err("expected a validation failure")
    }

    /// CC rebuilds a missing parameter from `invalid_type` + "received
    /// undefined", singular noun for one issue.
    #[test]
    fn missing_parameter_reads_like_official() {
        use crate::utils::zod::{object, string};
        let schema = object(vec![("pattern", string())]);
        let error = error_for(&schema, json!({}));
        assert_eq!(
            format_zod_validation_error("Glob", &error),
            "Glob failed due to the following issue:\nThe required parameter `pattern` is missing"
        );
    }

    /// Multiple issues switch the noun to plural, and `unrecognized_keys`
    /// follows the missing ones (zod orders it last).
    #[test]
    fn missing_and_unexpected_read_like_official() {
        use crate::utils::zod::{strict_object, string};
        let schema = strict_object(vec![("pattern", string())]);
        let error = error_for(&schema, json!({"unexpected": 1}));
        assert_eq!(
            format_zod_validation_error("Glob", &error),
            "Glob failed due to the following issues:\n\
             The required parameter `pattern` is missing\n\
             An unexpected parameter `unexpected` was provided"
        );
    }

    /// A type mismatch names expected and received; `received` comes out of the
    /// message text, as CC extracts it.
    #[test]
    fn type_mismatch_reads_like_official() {
        use crate::utils::zod::{object, string};
        let schema = object(vec![("pattern", string())]);
        let error = error_for(&schema, json!({"pattern": 5}));
        assert_eq!(
            format_zod_validation_error("Grep", &error),
            "Grep failed due to the following issue:\n\
             The parameter `pattern` type is expected as `string` but provided as `number`"
        );
    }

    /// Nested paths render as `a.b[0].c`.
    #[test]
    fn nested_path_formats_like_official() {
        let path = vec![
            PathSegment::Key("todos".to_string()),
            PathSegment::Index(0),
            PathSegment::Key("activeForm".to_string()),
        ];
        assert_eq!(format_validation_path(&path), "todos[0].activeForm");
    }

    /// `ShellError` reports exit code, stderr and stdout; an interrupted run
    /// inserts the tool-use interrupt notice, a clean one leaves it empty.
    #[test]
    fn shell_error_parts_read_like_official() {
        let error = anyhow::Error::new(ShellError::new("out", "err", 2, false));
        assert_eq!(
            get_error_parts(&error),
            vec![
                "Exit code 2".to_string(),
                String::new(),
                "err".to_string(),
                "out".to_string()
            ]
        );
        assert_eq!(format_error(&error), "Exit code 2\nerr\nout");

        // Interruption ADDS the notice after the exit code; it does not replace
        // the parts (CC pushes it as the second element).
        let interrupted = anyhow::Error::new(ShellError::new("", "", 143, true));
        assert_eq!(
            format_error(&interrupted),
            format!("Exit code 143\n{INTERRUPT_MESSAGE_FOR_TOOL_USE}")
        );
    }

    /// The no-output sentence is for an error with nothing to say at all — a
    /// ShellError always contributes `Exit code N`, so it never reaches it.
    #[test]
    fn empty_output_reads_like_official() {
        let bare = anyhow::Error::msg("");
        assert_eq!(format_error(&bare), "Command failed with no output");

        let shell = anyhow::Error::new(ShellError::new("", "", 0, false));
        assert_eq!(format_error(&shell), "Exit code 0");
    }

    /// An AbortError renders its own message, or the interrupt notice when it
    /// has none.
    #[test]
    fn abort_error_reads_like_official() {
        let named = anyhow::Error::new(AbortError::new("stopped by user"));
        assert_eq!(format_error(&named), "stopped by user");
        let bare = anyhow::Error::new(AbortError::new(""));
        assert_eq!(format_error(&bare), INTERRUPT_MESSAGE_FOR_TOOL_USE);
    }

    /// Past 10 000 characters the middle is elided, keeping 5 000 each side.
    #[test]
    fn long_output_is_elided_like_official() {
        let long = "x".repeat(10_050);
        let error = anyhow::Error::new(ShellError::new(long, "", 1, false));
        let formatted = format_error(&error);
        // "Exit code 1\n" + 10050 chars ⇒ over the cap.
        assert!(formatted.contains("characters truncated"));
        assert!(formatted.starts_with("Exit code 1"));
    }

    /// Codes the formatter does not rebuild fall through to the raw message —
    /// the issues dump.
    #[test]
    fn unhandled_codes_fall_back_to_the_issues_dump() {
        use crate::utils::zod::number;
        let schema = number().int();
        let error = error_for(&schema, json!(9007199254740992_f64));
        let formatted = format_zod_validation_error("Read", &error);
        assert_eq!(formatted, error.message());
        assert!(formatted.starts_with('['), "the dump is the issues array");
    }
}
