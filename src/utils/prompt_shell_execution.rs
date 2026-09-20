//! Executes embedded shell commands in skill/command markdown prompts.
//!
//! Maps to: CC `utils/promptShellExecution.ts`.
//!
//! The official runtime expands two markdown syntaxes before injecting a
//! prompt into the conversation:
//! - fenced blocks: <code>```!\ncommand\n```</code>
//! - inline commands: <code>!`command`</code>
//!
//! This Rust port preserves the same responsibility boundary: prompt expansion
//! lives in `utils/`, while concrete shell execution remains in Bash/PowerShell
//! tool modules and permission decisions remain in `utils/permissions`.

use crate::tool::ToolCall;
use crate::utils::frontmatter_parser::FrontmatterShell;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MalformedCommandError {
    pub message: String,
}

impl MalformedCommandError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl std::fmt::Display for MalformedCommandError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for MalformedCommandError {}

#[derive(Clone, Debug, PartialEq, Eq)]
struct PromptShellMatch {
    start: usize,
    end: usize,
    pattern: String,
    command: String,
}

/// Maps to: CC `utils/promptShellExecution.ts:69-74`
/// `executeShellCommandsInPrompt(text, context, slashCommandName, shell)`.
///
/// The `allowed-tools` widening is NOT applied here. CC's callers hand in a
/// context whose `getAppState` is already overridden — see
/// `loadSkillsDir.ts:375-395` (skills) and `loadPluginCommands.ts:378` — and
/// they do not agree on the composition rule, so the choice belongs to the
/// caller, not to this function.
pub fn execute_shell_commands_in_prompt(
    text: &str,
    context: &crate::tool::ToolUseContext,
    slash_command_name: &str,
    shell: Option<FrontmatterShell>,
) -> Result<String, MalformedCommandError> {
    let matches = collect_prompt_shell_matches(text);
    if matches.is_empty() {
        return Ok(text.to_string());
    }

    let mut replacements = Vec::with_capacity(matches.len());
    for prompt_match in matches {
        let output = execute_prompt_shell_command(
            &prompt_match.command,
            &prompt_match.pattern,
            context,
            slash_command_name,
            shell,
        )?;
        replacements.push((prompt_match.start, prompt_match.end, output));
    }

    let mut result = String::with_capacity(text.len());
    let mut cursor = 0;
    for (start, end, replacement) in replacements {
        if start < cursor {
            continue;
        }
        result.push_str(&text[cursor..start]);
        result.push_str(&replacement);
        cursor = end;
    }
    result.push_str(&text[cursor..]);
    Ok(result)
}

fn execute_prompt_shell_command(
    command: &str,
    pattern: &str,
    context: &crate::tool::ToolUseContext,
    slash_command_name: &str,
    shell: Option<FrontmatterShell>,
) -> Result<String, MalformedCommandError> {
    let use_powershell = matches!(shell, Some(FrontmatterShell::PowerShell))
        && crate::utils::shell::shell_tool_utils::is_powershell_tool_enabled();
    let tool_name = if use_powershell { "PowerShell" } else { "Bash" };
    let input = serde_json::json!({ "command": command });
    let mut effective_context = context.clone();
    if let Some(permission_context) = effective_context
        .get_app_state()
        .map(|state| (*state.tool_permission_context).clone())
    {
        // Rust tool execution also carries a local permission snapshot; hydrate
        // it from the caller's canonical getAppState projection (which is where
        // the command-scoped `allowed-tools` widening lives) at this boundary.
        effective_context.tool_permission_context = permission_context;
    }

    let permission =
        crate::utils::permissions::permissions::has_permissions_to_use_tool_with_context(
            crate::utils::permissions::permissions::HasPermissionsToUseToolParams {
                tool_use_id: "",
                tool_name,
                mcp_info: None,
                input_summary: command,
                input: &input,
                context: &effective_context.tool_permission_context,
                // Maps to CC classifyYoloAction(context.messages, ...).
                messages: &effective_context.messages,
                app_store: None,
                local_denial_tracking: None,
                // Maps to: CC `context.abortController.signal` → classifyYoloAction / sideQuery.
                abort_signal: Some(effective_context.abort_controller.signal()),
            },
            Some(&effective_context),
        );

    if !matches!(
        permission,
        crate::utils::permissions::permissions::HasPermissionsToUseToolResult::Allow { .. }
    ) {
        let message = match permission {
            crate::utils::permissions::permissions::HasPermissionsToUseToolResult::Aborted(
                error,
            ) => {
                return Err(MalformedCommandError::new(format!("[Error]\n{error}")));
            }
            crate::utils::permissions::permissions::HasPermissionsToUseToolResult::Deny(_) => {
                "Permission denied"
            }
            crate::utils::permissions::permissions::HasPermissionsToUseToolResult::Ask(_) => {
                "Permission denied"
            }
            crate::utils::permissions::permissions::HasPermissionsToUseToolResult::Allow {
                ..
            } => {
                unreachable!()
            }
        };
        return Err(MalformedCommandError::new(format!(
            "Shell command permission check failed for command in {slash_command_name}: {command}. Error: {message}"
        )));
    }

    if use_powershell {
        return execute_powershell_prompt_command(&input, &effective_context, pattern);
    }
    execute_bash_prompt_command(&input, &effective_context, pattern)
}

fn execute_bash_prompt_command(
    input: &serde_json::Value,
    context: &crate::tool::ToolUseContext,
    pattern: &str,
) -> Result<String, MalformedCommandError> {
    match crate::tools::bash_tool::bash_output(input, context, None, None) {
        Ok(output) => {
            let data = crate::tool::ToolOutput::Bash(output);
            let (content, _status) = crate::tools::bash_tool::BashTool
                .map_tool_result_to_tool_result_block_param(&data, "");
            Ok(content)
        }
        Err(message) => Err(format_bash_error(message, pattern, false)),
    }
}

fn execute_powershell_prompt_command(
    input: &serde_json::Value,
    context: &crate::tool::ToolUseContext,
    pattern: &str,
) -> Result<String, MalformedCommandError> {
    match crate::tools::powershell_tool::powershell_output(
        input,
        &context.abort_controller,
        context.cwd_override.as_deref(),
    ) {
        Ok(output) => {
            let data = crate::tool::ToolOutput::PowerShell(output);
            let (content, _status) = crate::tools::powershell_tool::PowerShellTool
                .map_tool_result_to_tool_result_block_param(&data, "");
            Ok(content)
        }
        Err(error) => Err(format_bash_error(
            error.content().to_string(),
            pattern,
            false,
        )),
    }
}

fn format_bash_error(message: String, pattern: &str, inline: bool) -> MalformedCommandError {
    let formatted = if inline {
        format!("[Error: {message}]")
    } else {
        format!("[Error]\n{message}")
    };
    MalformedCommandError::new(format!(
        "Shell command failed for pattern \"{pattern}\": {formatted}"
    ))
}

fn collect_prompt_shell_matches(text: &str) -> Vec<PromptShellMatch> {
    let mut matches = Vec::new();
    matches.extend(collect_block_matches(text));
    let block_ranges = matches
        .iter()
        .map(|entry| (entry.start, entry.end))
        .collect::<Vec<_>>();
    if text.contains("!`") {
        matches.extend(
            collect_inline_matches(text)
                .into_iter()
                .filter(|entry| !range_overlaps_any(entry.start, entry.end, &block_ranges)),
        );
    }
    matches.sort_by_key(|entry| entry.start);
    matches
}

fn collect_block_matches(text: &str) -> Vec<PromptShellMatch> {
    // Maps to CC `BLOCK_PATTERN = /```!\s*\n?([\s\S]*?)\n?```/g`.
    let regex = regex::Regex::new(r"(?s)```!\s*\n?(.*?)\n?```").expect("block regex");
    regex
        .captures_iter(text)
        .filter_map(|captures| {
            let whole = captures.get(0)?;
            let command = captures.get(1)?.as_str().trim().to_string();
            (!command.is_empty()).then(|| PromptShellMatch {
                start: whole.start(),
                end: whole.end(),
                pattern: whole.as_str().to_string(),
                command,
            })
        })
        .collect()
}

fn collect_inline_matches(text: &str) -> Vec<PromptShellMatch> {
    // Maps to CC `INLINE_PATTERN = /(?<=^|\s)!`([^`]+)`/gm`. Rust regex has
    // no lookbehind support, so this scanner preserves the same predicate:
    // `!` must be at the start of the text or preceded by whitespace.
    let mut matches = Vec::new();
    let mut cursor = 0;
    while let Some(relative) = text[cursor..].find("!`") {
        let start = cursor + relative;
        let preceding_is_allowed = text[..start]
            .chars()
            .next_back()
            .map(|ch| ch.is_whitespace())
            .unwrap_or(true);
        let command_start = start + 2;
        let Some(command_relative_end) = text[command_start..].find('`') else {
            break;
        };
        let end = command_start + command_relative_end + 1;
        if preceding_is_allowed {
            let command = text[command_start..end - 1].trim().to_string();
            if !command.is_empty() {
                matches.push(PromptShellMatch {
                    start,
                    end,
                    pattern: text[start..end].to_string(),
                    command,
                });
            }
        }
        cursor = end;
    }
    matches
}

fn range_overlaps_any(start: usize, end: usize, ranges: &[(usize, usize)]) -> bool {
    ranges
        .iter()
        .any(|(range_start, range_end)| start < *range_end && end > *range_start)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prompt_shell_matcher_finds_official_block_and_inline_forms() {
        let matches = collect_prompt_shell_matches(
            "a !`echo inline` b foo!`skip`\n```!\necho block\n```\n`code`!`skip2`",
        );
        let commands = matches
            .iter()
            .map(|entry| entry.command.as_str())
            .collect::<Vec<_>>();
        assert_eq!(commands, vec!["echo inline", "echo block"]);
    }

    /// CC's callers own the widening, so the test does what
    /// `loadSkillsDir.ts:377-392` does: put the `allowed-tools` rules on the
    /// `command` source of the context handed in.
    fn context_allowing(specs: &[&str]) -> crate::tool::ToolUseContext {
        use crate::types::permissions::PermissionRuleSource;
        use crate::utils::permissions::permission_rule_parser::permission_rule_value_from_string;

        let mut context = crate::tool::ToolUseContext::default();
        context.tool_permission_context.always_allow_rules.insert(
            PermissionRuleSource::Command,
            specs
                .iter()
                .map(|spec| permission_rule_value_from_string(spec))
                .collect(),
        );
        context
    }

    #[test]
    fn execute_shell_commands_in_prompt_runs_allowed_bash_commands() {
        let context = context_allowing(&["Bash(printf *)"]);
        let expanded = execute_shell_commands_in_prompt(
            "Inline !`printf inline-output`\n```!\nprintf block-output\n```",
            &context,
            "/skill",
            None,
        )
        .expect("allowed shell commands expand");

        assert!(expanded.contains("Inline inline-output"));
        assert!(expanded.contains("block-output"));
        assert!(!expanded.contains("!`"));
        assert!(!expanded.contains("```!"));
    }

    #[test]
    fn execute_shell_commands_in_prompt_denies_unallowed_bash_commands() {
        let context = crate::tool::ToolUseContext::default();
        let error = execute_shell_commands_in_prompt(
            "Denied !`printf denied-output`",
            &context,
            "/skill",
            None,
        )
        .expect_err("default permissions ask, so prompt shell expansion fails safely");

        assert!(
            error
                .message
                .contains("Shell command permission check failed")
        );
        assert!(error.message.contains("printf denied-output"));
    }
}
