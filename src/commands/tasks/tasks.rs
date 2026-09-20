//! Maps to CC `commands/tasks/tasks.tsx#call`.
use crate::commands::Command;
use crate::utils::process_user_input::ProcessUserInputBaseResult;
use crate::utils::process_user_input::process_slash_command::{
    LocalCommandUi, SlashCommandInvocation, open_local_command_ui,
};

/// Maps to CC `commands/tasks/tasks.tsx#call`.
pub fn call(
    command: &Command,
    args: &str,
    _uuid: Option<String>,
    context: &crate::tool::ToolUseContext,
) -> ProcessUserInputBaseResult {
    open_local_command_ui(
        LocalCommandUi::Tasks {
            context: std::sync::Arc::new(context.clone()),
        },
        SlashCommandInvocation::new(command.name.as_ref(), args),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn tasks_call_preserves_context_and_opens_local_dialog_without_query() {
        let command = Command::local_ui(super::super::NAME, super::super::DESCRIPTION);
        let context = crate::tool::ToolUseContext::default();
        let result = call(&command, "", None, &context);
        assert!(!result.should_query);
        assert!(result.messages.is_empty());
        match result.local_action.unwrap() {
            crate::utils::process_user_input::process_slash_command::SlashCommandAction::OpenLocalCommandUi {
                command: LocalCommandUi::Tasks { context: actual }, invocation,
            } => {
                assert_eq!(*actual, context);
                assert_eq!(invocation.command_name, "tasks");
            }
            other => panic!("expected tasks dialog, got {other:?}"),
        }
    }
}
