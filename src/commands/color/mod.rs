//! Maps to: CC `commands/color/index.ts`.

pub mod color;

use crate::commands::Command;
use crate::constants::query_source::QuerySource;
use crate::tool::ToolUseContext;
use crate::utils::process_user_input::ProcessUserInputBaseResult;
use crate::utils::process_user_input::process_slash_command::{
    SlashCommandAction, SlashCommandInvocation,
};
use std::sync::Arc;

/// Maps to: CC `commands/color/index.ts:7-15` default descriptor.
pub fn command() -> Command {
    Command::local_ui("color", "Set the prompt bar color for this session")
        .immediate()
        .argument_hint("<color|default>")
        .executable(dispatch)
}

/// Policy-free synchronous descriptor transport for CC's lazy async `call`.
/// The established local-JSX `SlashCommandAction` carrier in PORTING.md keeps
/// validation, persistence and AppState mutation in `color::call` off-frame.
pub fn dispatch(
    command: &Command,
    args: &str,
    _uuid: Option<String>,
    context: &ToolUseContext,
) -> ProcessUserInputBaseResult {
    ProcessUserInputBaseResult {
        messages: Vec::new(),
        should_query: false,
        allowed_tools: None,
        local_action: Some(SlashCommandAction::SetSessionColor {
            args: args.to_string(),
            context: Arc::new(context.clone()),
            invocation: SlashCommandInvocation::new(command.name.as_ref(), args),
        }),
        query_source: QuerySource::Prompt,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn color_descriptor_matches_official_metadata_and_deferred_call() {
        // CC index.ts:7-15: immediate local-jsx, exact hint and description.
        let command = command();
        assert_eq!(command.name, "color");
        assert_eq!(command.kind, crate::commands::CommandKind::LocalUi);
        assert_eq!(
            command.description,
            "Set the prompt bar color for this session"
        );
        assert_eq!(command.argument_hint.as_deref(), Some("<color|default>"));
        assert!(command.immediate);
        assert!(!command.supports_non_interactive);
        let result = command.call.unwrap()(&command, "  GREEN  ", None, &ToolUseContext::default());
        assert!(result.messages.is_empty());
        assert!(!result.should_query);
        assert!(matches!(
            result.local_action,
            Some(SlashCommandAction::SetSessionColor { args, .. }) if args == "  GREEN  "
        ));
    }
}
