//! Shell completion producer.
//!
//! Maps to: CC `utils/bash/shellCompletion.ts`.
//! The parser and command shapes stay here; `useTypeahead` only owns the
//! lifecycle and selection of the resulting rows.

use crate::components::prompt_input::prompt_input_footer_suggestions::SuggestionItem;
use crate::utils::bash::shell_quote::{ParseEntry, quote, try_parse_shell_command};
use regex::Regex;
use std::sync::OnceLock;
use std::time::Duration;

pub const MAX_SHELL_COMPLETIONS: usize = 15;
const SHELL_COMPLETION_TIMEOUT: Duration = Duration::from_secs(1);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShellCompletionType {
    Command,
    Variable,
    File,
}

impl ShellCompletionType {
    fn as_str(self) -> &'static str {
        match self {
            Self::Command => "command",
            Self::Variable => "variable",
            Self::File => "file",
        }
    }

    /// Reads the `completionType` carrier this module attaches to each row.
    /// CC consumers cast it inline:
    /// `suggestion.metadata as { completionType: ShellCompletionType }`.
    pub fn from_metadata(metadata: Option<&serde_json::Value>) -> Option<Self> {
        metadata
            .and_then(|metadata| metadata.get("completionType"))
            .and_then(serde_json::Value::as_str)
            .and_then(|kind| match kind {
                "command" => Some(Self::Command),
                "variable" => Some(Self::Variable),
                "file" => Some(Self::File),
                _ => None,
            })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct InputContext {
    prefix: String,
    completion_type: ShellCompletionType,
}

fn is_command_operator(entry: &ParseEntry) -> bool {
    matches!(
        entry,
        ParseEntry::Operator(operator) if matches!(operator.as_str(), "|" | "||" | "&&" | ";")
    )
}

fn completion_type_from_prefix(prefix: &str) -> ShellCompletionType {
    if prefix.starts_with('$') {
        ShellCompletionType::Variable
    } else if prefix.contains('/') || prefix.starts_with('~') || prefix.starts_with('.') {
        ShellCompletionType::File
    } else {
        ShellCompletionType::Command
    }
}

fn last_string_token(tokens: &[ParseEntry]) -> Option<(String, usize)> {
    tokens.iter().enumerate().rev().find_map(|(index, entry)| {
        if let ParseEntry::String(token) = entry {
            Some((token.clone(), index))
        } else {
            None
        }
    })
}

fn is_new_command_context(tokens: &[ParseEntry], token_index: usize) -> bool {
    token_index == 0 || tokens.get(token_index - 1).is_some_and(is_command_operator)
}

/// Maps to CC `parseInputContext`.
fn parse_input_context(input: &str, cursor_offset: usize) -> InputContext {
    let cursor = crate::utils::cursor::clamp_cursor(input, cursor_offset);
    let before_cursor = &input[..cursor];
    static VARIABLE: OnceLock<Regex> = OnceLock::new();
    if let Some(found) = VARIABLE
        .get_or_init(|| Regex::new(r"\$[a-zA-Z_][a-zA-Z0-9_]*$").expect("valid shell variable"))
        .find(before_cursor)
    {
        return InputContext {
            prefix: found.as_str().to_string(),
            completion_type: ShellCompletionType::Variable,
        };
    }

    let Ok(tokens) = try_parse_shell_command(before_cursor) else {
        let prefix = before_cursor
            .split(char::is_whitespace)
            .next_back()
            .unwrap_or_default()
            .to_string();
        let first_token = !before_cursor.contains(char::is_whitespace);
        return InputContext {
            completion_type: if first_token {
                ShellCompletionType::Command
            } else {
                completion_type_from_prefix(&prefix)
            },
            prefix,
        };
    };

    let Some((token, token_index)) = last_string_token(&tokens) else {
        return InputContext {
            prefix: String::new(),
            completion_type: ShellCompletionType::Command,
        };
    };
    if before_cursor.ends_with(char::is_whitespace) {
        return InputContext {
            prefix: String::new(),
            completion_type: ShellCompletionType::File,
        };
    }

    let base_type = completion_type_from_prefix(&token);
    if matches!(
        base_type,
        ShellCompletionType::Variable | ShellCompletionType::File
    ) {
        return InputContext {
            prefix: token,
            completion_type: base_type,
        };
    }
    InputContext {
        prefix: token,
        completion_type: if is_new_command_context(&tokens, token_index) {
            ShellCompletionType::Command
        } else {
            ShellCompletionType::File
        },
    }
}

fn bash_completion_command(prefix: &str, completion_type: ShellCompletionType) -> String {
    match completion_type {
        ShellCompletionType::Variable => {
            let name = prefix.strip_prefix('$').unwrap_or(prefix);
            format!("compgen -v {} 2>/dev/null", quote(&[name]))
        }
        ShellCompletionType::File => format!(
            "compgen -f {} 2>/dev/null | head -{} | while IFS= read -r f; do [ -d \"$f\" ] && echo \"$f/\" || echo \"$f \"; done",
            quote(&[prefix]),
            MAX_SHELL_COMPLETIONS
        ),
        ShellCompletionType::Command => {
            format!("compgen -c {} 2>/dev/null", quote(&[prefix]))
        }
    }
}

fn zsh_completion_command(prefix: &str, completion_type: ShellCompletionType) -> String {
    match completion_type {
        ShellCompletionType::Variable => {
            let name = prefix.strip_prefix('$').unwrap_or(prefix);
            format!(
                "print -rl -- ${{(k)parameters[(I){}*]}} 2>/dev/null",
                quote(&[name])
            )
        }
        ShellCompletionType::File => format!(
            "for f in {}*(N[1,{}]); do [[ -d \"$f\" ]] && echo \"$f/\" || echo \"$f \"; done",
            quote(&[prefix]),
            MAX_SHELL_COMPLETIONS
        ),
        ShellCompletionType::Command => format!(
            "print -rl -- ${{(k)commands[(I){}*]}} 2>/dev/null",
            quote(&[prefix])
        ),
    }
}

fn shell_completion_rows(
    stdout: &str,
    input: &str,
    completion_type: ShellCompletionType,
) -> Vec<SuggestionItem> {
    stdout
        .lines()
        .filter(|line| !line.trim().is_empty())
        .take(MAX_SHELL_COMPLETIONS)
        .map(|text| SuggestionItem {
            id: text.to_string(),
            display_text: text.to_string(),
            tag: None,
            command_text: text.to_string(),
            description: String::new(),
            metadata: Some(serde_json::json!({
                "completionType": completion_type.as_str(),
                "inputSnapshot": input,
            })),
            color: None,
        })
        .collect()
}

/// Maps to CC `getShellCompletions`.
pub async fn get_shell_completions(input: &str, cursor_offset: usize) -> Vec<SuggestionItem> {
    get_shell_completions_with_abort(
        input,
        cursor_offset,
        crate::tool::AbortController::default(),
    )
    .await
}

/// Maps to CC `getShellCompletions(input, cursorOffset, abortSignal)`.
/// `useTypeahead` owns the controller lifecycle; this owner only forwards the
/// signal into the existing shell process executor.
pub async fn get_shell_completions_with_abort(
    input: &str,
    cursor_offset: usize,
    abort: crate::tool::AbortController,
) -> Vec<SuggestionItem> {
    let context = parse_input_context(input, cursor_offset);
    if context.prefix.is_empty() {
        return Vec::new();
    }
    let shell = match crate::utils::shell::find_suitable_shell() {
        Ok(path) => path.display().to_string(),
        Err(_) => return Vec::new(),
    };
    let command = if shell.contains("zsh") {
        zsh_completion_command(&context.prefix, context.completion_type)
    } else if shell.contains("bash") {
        bash_completion_command(&context.prefix, context.completion_type)
    } else {
        return Vec::new();
    };
    let result = tokio::task::spawn_blocking(move || {
        let shell_command = crate::utils::shell::exec(
            &command,
            &abort,
            crate::utils::shell::shell_provider::ShellType::Bash,
            crate::utils::shell::ExecOptions {
                timeout: Some(SHELL_COMPLETION_TIMEOUT),
                ..Default::default()
            },
        )
        .ok()?;
        Some(shell_command.wait_result())
    })
    .await
    .ok()
    .flatten();
    result
        .map(|result| shell_completion_rows(&result.stdout, input, context.completion_type))
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shell_context_matches_command_variable_and_file_branches() {
        assert_eq!(
            parse_input_context("ec", 2),
            InputContext {
                prefix: "ec".into(),
                completion_type: ShellCompletionType::Command,
            }
        );
        assert_eq!(
            parse_input_context("echo $PA", 8),
            InputContext {
                prefix: "$PA".into(),
                completion_type: ShellCompletionType::Variable,
            }
        );
        assert_eq!(
            parse_input_context("echo ./sr", 9),
            InputContext {
                prefix: "./sr".into(),
                completion_type: ShellCompletionType::File,
            }
        );
        assert_eq!(
            parse_input_context("echo ", 5),
            InputContext {
                prefix: String::new(),
                completion_type: ShellCompletionType::File,
            }
        );
    }

    #[test]
    fn completion_rows_keep_the_source_input_snapshot() {
        let rows = shell_completion_rows("one\n\n two \n", "ec", ShellCompletionType::Command);
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].display_text, "one");
        assert_eq!(rows[1].display_text, " two ");
        assert_eq!(rows[0].metadata.as_ref().unwrap()["inputSnapshot"], "ec");
    }
}
