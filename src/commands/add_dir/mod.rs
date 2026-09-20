//! Maps to: CC `commands/add-dir/index.ts` and its lazy command module.

pub mod add_dir;
pub mod validation;

use crate::commands::Command;
use crate::utils::process_user_input::ProcessUserInputBaseResult;
use crate::utils::process_user_input::process_slash_command::{
    LocalCommandUi, SlashCommandInvocation, open_local_command_ui,
};

/// Maps to CC commands/add-dir/index.ts load + add-dir.tsx call transport.
pub fn dispatch(
    command: &Command,
    args: &str,
    _uuid: Option<String>,
    _context: &crate::tool::ToolUseContext,
) -> ProcessUserInputBaseResult {
    open_local_command_ui(
        LocalCommandUi::AddDir {
            args: args.to_string(),
        },
        SlashCommandInvocation::new(command.name.as_ref(), args),
    )
}
