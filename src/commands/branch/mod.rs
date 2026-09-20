//! Maps to: CC `commands/branch/index.ts`.

pub mod branch;

pub const NAME: &str = "branch";
pub const DESCRIPTION: &str = "Create a branch of the current conversation at this point";
pub const ARGUMENT_HINT: &str = "[name]";

/// Maps to: CC `commands/branch/index.ts:4-12` default descriptor.
pub fn command() -> crate::commands::Command {
    let command = crate::commands::Command::local_ui(NAME, DESCRIPTION)
        .argument_hint(ARGUMENT_HINT)
        .executable(dispatch);
    // The source reads the raw build flag, not the session-dependent
    // isForkSubagentEnabled predicate.
    if crate::utils::feature_flags::feature_enabled(
        crate::utils::feature_flags::FeatureFlag::ForkSubagent,
    ) {
        command
    } else {
        command.aliases(&["fork"])
    }
}

/// Native transport for the source descriptor's lazy async `call`.
pub fn dispatch(
    command: &crate::commands::Command,
    args: &str,
    _uuid: Option<String>,
    _context: &crate::tool::ToolUseContext,
) -> crate::utils::process_user_input::ProcessUserInputBaseResult {
    use crate::utils::process_user_input::process_slash_command::{
        SlashCommandAction, SlashCommandInvocation,
    };
    crate::utils::process_user_input::ProcessUserInputBaseResult {
        messages: Vec::new(),
        should_query: false,
        allowed_tools: None,
        local_action: Some(SlashCommandAction::BranchConversation {
            args: args.to_string(),
            invocation: SlashCommandInvocation::new(command.name.as_ref(), args),
        }),
        query_source: crate::constants::query_source::QuerySource::Prompt,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn branch_descriptor_matches_official_deferred_execution() {
        let command = command();
        assert_eq!(command.kind, crate::commands::CommandKind::LocalUi);
        assert!(!command.immediate);
        assert!(!command.supports_non_interactive);
        assert_eq!(command.argument_hint.as_deref(), Some("[name]"));
        let result = command.call.unwrap()(
            &command,
            "  title  ",
            None,
            &crate::tool::ToolUseContext::default(),
        );
        assert!(!result.should_query);
        assert!(result.messages.is_empty());
        assert!(matches!(result.local_action,
            Some(crate::utils::process_user_input::process_slash_command::SlashCommandAction::BranchConversation {args, ..})
            if args == "  title  "));
    }
}
