//! Maps to: CC `commands/reload-plugins/index.ts`.

pub mod reload_plugins;

/// Maps to: CC `commands/reload-plugins/index.ts:8-19` default descriptor.
pub fn command() -> crate::commands::Command {
    crate::commands::Command::local(
        "reload-plugins",
        "Activate pending plugin changes in the current session",
    )
    .executable(dispatch)
}

/// A3/A6: CC's lazy async local `call` becomes a retained REPL action.
/// No filesystem or refresh work runs in this synchronous dispatcher.
fn dispatch(
    command: &crate::commands::Command,
    args: &str,
    _uuid: Option<String>,
    _context: &crate::tool::ToolUseContext,
) -> crate::utils::process_user_input::ProcessUserInputBaseResult {
    use crate::utils::process_user_input::process_slash_command::{
        SlashCommandAction, SlashCommandInvocation, local_text_command_input,
    };
    let invocation = SlashCommandInvocation::new(command.name.as_ref(), args);
    let user_message = local_text_command_input(&invocation);
    crate::utils::process_user_input::ProcessUserInputBaseResult {
        messages: Vec::new(),
        should_query: false,
        allowed_tools: None,
        local_action: Some(SlashCommandAction::ReloadPlugins {
            invocation,
            user_message,
        }),
        query_source: crate::constants::query_source::QuerySource::Prompt,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reload_plugins_descriptor_matches_official_deferred_local_command() {
        let command = command();
        assert_eq!(command.kind, crate::commands::CommandKind::Local);
        assert!(!command.supports_non_interactive);
        assert!(!command.immediate);
        let result = command.call.unwrap()(
            &command,
            "ignored args",
            None,
            &crate::tool::ToolUseContext::default(),
        );
        assert!(!result.should_query);
        assert!(result.messages.is_empty());
        assert!(matches!(result.local_action,
            Some(crate::utils::process_user_input::process_slash_command::SlashCommandAction::ReloadPlugins {
                invocation, user_message,
            }) if invocation.command_name == "reload-plugins"
                && invocation.args == "ignored args" && !user_message.content.is_empty()));
    }
}
