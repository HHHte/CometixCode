//! Maps to: CC `commands/stats/stats.tsx`.

use crate::commands::Command;
use crate::constants::query_source::QuerySource;
use crate::utils::process_user_input::ProcessUserInputBaseResult;
use crate::utils::process_user_input::process_slash_command::{
    LocalCommandUi, SlashCommandAction, SlashCommandInvocation,
};

/// Maps to: CC `commands/stats/stats.tsx#call`.
pub fn call(
    command: &Command,
    args: &str,
    _uuid: Option<String>,
    _context: &crate::tool::ToolUseContext,
) -> ProcessUserInputBaseResult {
    ProcessUserInputBaseResult {
        messages: Vec::new(),
        should_query: false,
        allowed_tools: None,
        local_action: Some(SlashCommandAction::OpenLocalCommandUi {
            command: LocalCommandUi::Stats,
            invocation: SlashCommandInvocation::new(command.name.as_ref(), args),
        }),
        query_source: QuerySource::Prompt,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stats_call_opens_official_local_jsx_owner() {
        let command = Command::local_ui(super::super::NAME, super::super::DESCRIPTION);
        let result = call(
            &command,
            "",
            Some("stats-command".to_string()),
            &crate::tool::ToolUseContext::default(),
        );
        assert!(!result.should_query);
        assert!(result.messages.is_empty());
        assert_eq!(
            result.local_action,
            Some(SlashCommandAction::OpenLocalCommandUi {
                command: LocalCommandUi::Stats,
                invocation: SlashCommandInvocation::new("stats", ""),
            })
        );
    }
}
